mod iterator;
mod raw;
mod reference;

use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use guardian::ArcMutexGuardian;

use crate::linked_list::raw::RcuListNode;
use crate::{DefaultContext, RcuContext};

pub use crate::linked_list::iterator::*;
pub use crate::linked_list::reference::*;

/// RCU linked list.
///
/// # Limitations
///
/// ##### Mutable References
///
/// Because there might always be readers borrowing a node's data, it is impossible
/// to get a mutable references to the data inside the linked list. You should design
/// the type stored in the list with [interior mutabillity] that can be shared between
/// threads.
///
/// [interior mutabillity]: https://doc.rust-lang.org/reference/interior-mutability.html
///
/// ##### List Length
///
/// Because a writer might concurrently modify the list, the amount of node might change
/// at any moment. To prevent user error (e.g. allocate an array for each node), there is
/// no `.len()` method.
///
/// ##### Bidirectional Iteration
///
/// Because a writer might concurrently modify the list, it is possible that `node.next.prev != node`.
/// To prevent user error, this linked list does not support bidirectional iteration.
/// For example, if you create an forward iterator, it can only go forward.
///
/// # Safety
///
/// It is safe to send an `Arc<RcuList<T>>` to a non-registered RCU thread. A non-registered
/// thread may drop an `RcuList< T>` without calling any RCU primitives since lifetime rules
/// prevent any other thread from accessing an RCU reference.
pub struct RcuList<T, C = DefaultContext> {
    head: AtomicPtr<RcuListNode<T>>,
    tail: AtomicPtr<RcuListNode<T>>,
    mutex: Arc<Mutex<()>>,
    // Also prevents auto-trait implementation of [`Send`] and [`Sync`].
    context: PhantomData<*const C>,
}

impl<T, C> RcuList<T, C> {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            head: AtomicPtr::new(std::ptr::null_mut()),
            tail: AtomicPtr::new(std::ptr::null_mut()),
            mutex: Arc::default(),
            context: PhantomData,
        })
    }

    pub fn reader<'a>(self: &'a Arc<Self>, guard: &'a C::Guard<'a>) -> RcuListReader<'a, T, C>
    where
        C: RcuContext + 'a,
    {
        RcuListReader {
            list: self.clone(),
            guard,
        }
    }

    pub fn writer(self: &Arc<Self>) -> Result<RcuListWriter<T, C>> {
        Ok(RcuListWriter {
            list: self.clone(),
            guard: ArcMutexGuardian::take(self.mutex.clone())
                .map_err(|_| anyhow!("mutex has been poisoned"))?,
        })
    }
}

impl<T, C> Drop for RcuList<T, C> {
    fn drop(&mut self) {
        // SAFETY: Because of reference counting, there are no reader/writer threads accessing this object.
        let mut node_ptr = self.head.load(Ordering::Relaxed);
        while !node_ptr.is_null() {
            // SAFETY: The pointer is non-null.
            let next_ptr = unsafe { RcuListNode::next_node(node_ptr, Ordering::Relaxed) };
            drop(unsafe { Box::from_raw(node_ptr) });
            node_ptr = next_ptr;
        }
    }
}

/// #### Safety
///
/// An [`RcuList`] can be used to send `T` to another thread.
unsafe impl<T, C> Send for RcuList<T, C> where T: Send {}

/// #### Safety
///
/// An [`RcuList`] can be used to share `T` between threads.
unsafe impl<T, C> Sync for RcuList<T, C> where T: Sync {}

/// The read-side API of an [`RcuList`].
pub struct RcuListReader<'a, T, C>
where
    C: RcuContext + 'a,
{
    list: Arc<RcuList<T, C>>,
    #[allow(dead_code)]
    guard: &'a C::Guard<'a>,
}

impl<'a, T, C> RcuListReader<'a, T, C>
where
    C: RcuContext + 'a,
{
    pub fn iter_forward(&self) -> RcuListIterator<T, &Self> {
        RcuListIterator::new_forward(self, self.list.head.load(Ordering::Acquire))
    }

    pub fn iter_reverse(&self) -> RcuListIterator<T, &Self> {
        RcuListIterator::new_reverse(self, self.list.tail.load(Ordering::Acquire))
    }
}

/// The write-side API of an [`RcuList`].
pub struct RcuListWriter<T, C> {
    list: Arc<RcuList<T, C>>,
    #[allow(dead_code)]
    guard: ArcMutexGuardian<()>,
}

impl<T, C> RcuListWriter<T, C> {
    pub fn pop_back(&mut self) -> Option<RcuListRef<T, C>>
    where
        T: Send,
        C: RcuContext,
    {
        let node_ptr = self.list.tail.load(Ordering::Acquire);
        if node_ptr.is_null() {
            return None;
        }

        Some(
            RcuListEntry {
                list: self.list.clone(),
                // SAFETY: The pointer is not null.
                node: unsafe { NonNull::new_unchecked(node_ptr) },
                life: PhantomData,
            }
            .remove(),
        )
    }

    pub fn pop_front(&mut self) -> Option<RcuListRef<T, C>>
    where
        T: Send,
        C: RcuContext,
    {
        let node_ptr = self.list.head.load(Ordering::Acquire);
        if node_ptr.is_null() {
            return None;
        }

        Some(
            RcuListEntry {
                list: self.list.clone(),
                // SAFETY: The pointer is not null.
                node: unsafe { NonNull::new_unchecked(node_ptr) },
                life: PhantomData,
            }
            .remove(),
        )
    }

    pub fn push_back(&mut self, data: T) {
        let new_node = RcuListNode::new(data);

        let tail_node_ptr = self.list.tail.load(Ordering::Acquire);
        if tail_node_ptr.is_null() {
            self.list.head.store(new_node, Ordering::Relaxed);
        } else {
            unsafe { (*tail_node_ptr).insert_after(new_node) };
        }

        self.list.tail.store(new_node, Ordering::Release);
    }

    pub fn push_front(&mut self, data: T) {
        let new_node = RcuListNode::new(data);

        let head_node_ptr = self.list.head.load(Ordering::Acquire);
        if head_node_ptr.is_null() {
            self.list.tail.store(new_node, Ordering::Relaxed);
        } else {
            unsafe { (*head_node_ptr).insert_before(new_node) };
        }

        self.list.head.store(new_node, Ordering::Release);
    }
}

/// An individual entry in an [`RcuList`].
pub struct RcuListEntry<'a, T, C> {
    list: Arc<RcuList<T, C>>,
    node: NonNull<RcuListNode<T>>,
    #[allow(dead_code)]
    life: PhantomData<&'a T>,
}

impl<'a, T, C> RcuListEntry<'a, T, C> {
    /// Inserts a node after this entry.
    pub fn insert_after(&mut self, data: T) {
        let node = unsafe { self.node.as_mut() };
        let node_new = RcuListNode::new(data);

        // SAFETY: The pointer is non-null.
        let node_next = unsafe { RcuListNode::next_node(self.node.as_mut(), Ordering::Acquire) };

        unsafe {
            node.insert_after(node_new);
        }

        if node_next.is_null() {
            self.list.tail.store(node_new, Ordering::Release);
        }
    }

    /// Inserts a node before this entry.
    pub fn insert_before(&mut self, data: T) {
        let node = unsafe { self.node.as_mut() };
        let node_new = RcuListNode::new(data);

        // SAFETY: The pointer is non-null.
        let node_prev = unsafe { RcuListNode::prev_node(self.node.as_mut(), Ordering::Acquire) };

        unsafe {
            node.insert_after(node_new);
        }

        if node_prev.is_null() {
            self.list.head.store(node_new, Ordering::Release);
        }
    }

    /// Removes the node from the list.
    pub fn remove(mut self) -> RcuListRef<T, C>
    where
        T: Send,
        C: RcuContext,
    {
        // SAFETY: The pointer is non-null.
        let node_prev = unsafe { RcuListNode::prev_node(self.node.as_mut(), Ordering::Acquire) };

        // SAFETY: The pointer is non-null.
        let node_next = unsafe { RcuListNode::next_node(self.node.as_mut(), Ordering::Acquire) };

        if node_prev.is_null() {
            self.list.head.store(node_next, Ordering::Release);
        }

        if node_next.is_null() {
            self.list.tail.store(node_prev, Ordering::Release);
        }

        unsafe { RcuListNode::remove(self.node.as_ptr()) }
    }
}
