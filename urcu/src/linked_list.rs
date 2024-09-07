use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use guardian::ArcMutexGuardian;

use crate::{DefaultContext, RcuContext, RcuRef};

struct RcuListNode<T> {
    prev: AtomicPtr<Self>,
    next: AtomicPtr<Self>,
    data: T,
}

impl<T> RcuListNode<T> {
    fn new(data: T) -> *mut Self {
        Box::into_raw(Box::new(Self {
            prev: AtomicPtr::new(std::ptr::null_mut()),
            next: AtomicPtr::new(std::ptr::null_mut()),
            data,
        }))
    }

    /// Insert a node after this one.
    ///
    /// SAFETY: Require mutual exclusion on the list.
    unsafe fn insert_after(&mut self, new_prev_ptr: *mut Self) {
        let old_next_ptr = self.next.load(Ordering::Relaxed);
        let old_next = unsafe { &mut *old_next_ptr };
        let new_next = unsafe { &mut *new_prev_ptr };

        new_next.next.store(old_next, Ordering::Relaxed);
        new_next.prev.store(self, Ordering::Relaxed);

        if !old_next_ptr.is_null() {
            old_next.prev.store(new_next, Ordering::Release);
        }

        self.next.store(new_next, Ordering::Release);
    }

    /// Insert a node before this one.
    ///
    /// SAFETY: Require mutual exclusion on the list.
    unsafe fn insert_before(&mut self, new_prev_ptr: *mut Self) {
        let old_prev_ptr = self.prev.load(Ordering::Relaxed);
        let old_prev = unsafe { &mut *old_prev_ptr };
        let new_prev = unsafe { &mut *new_prev_ptr };

        new_prev.next.store(self, Ordering::Relaxed);
        new_prev.prev.store(old_prev, Ordering::Relaxed);

        if !old_prev_ptr.is_null() {
            old_prev.next.store(new_prev, Ordering::Release);
        }

        self.prev.store(new_prev, Ordering::Release);
    }

    /// Delete this node from the list.
    ///
    /// SAFETY: Require mutual exclusion on the list.
    unsafe fn remove<C>(ptr: *mut Self) -> RcuListRef<T, C>
    where
        T: Send + Sync,
        C: RcuContext,
    {
        let node = unsafe { &*ptr };

        let prev_ptr = node.prev.load(Ordering::Relaxed);
        let prev = unsafe { &mut *prev_ptr };

        let next_ptr = node.next.load(Ordering::Relaxed);
        let next = unsafe { &mut *next_ptr };

        if !next_ptr.is_null() {
            next.prev.store(prev, Ordering::Release);
        }

        if !prev_ptr.is_null() {
            prev.next.store(next, Ordering::Release);
        }

        RcuListRef {
            ptr,
            context: Default::default(),
        }
    }
}

/// An owned RCU reference to a element removed from an [`RcuList`].
pub struct RcuListRefOwned<T>(Box<RcuListNode<T>>);

impl<T> Deref for RcuListRefOwned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0.deref().data
    }
}

impl<T> DerefMut for RcuListRefOwned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.deref_mut().data
    }
}

/// An RCU reference to a element removed from an [`RcuList`].
///
/// #### Note
///
/// To get ownership of the reference, you can use [`rcu_take_ownership`]. If ownership is
/// never taken, cleanup will be automatically executed after the next RCU grace period.
///
/// [`rcu_take_ownership`]: crate::rcu_take_ownership
#[must_use]
pub struct RcuListRef<T, C>
where
    T: Send + Sync,
    C: RcuContext,
{
    ptr: *mut RcuListNode<T>,
    context: PhantomData<C>,
}

/// #### Safety
///
/// It is safe to send an RCU reference if `T` implements [`Send`].
unsafe impl<T, C> Send for RcuListRef<T, C>
where
    T: Send + Sync,
    C: RcuContext,
{
}

impl<T, C> Drop for RcuListRef<T, C>
where
    T: Send + Sync,
    C: RcuContext,
{
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            Self {
                ptr: self.ptr,
                context: Default::default(),
            }
            .defer_cleanup();
        }
    }
}

/// #### Safety
///
/// The memory reclamation upon dropping is properly deferred after the RCU grace period.
unsafe impl<T, C> RcuRef<C> for RcuListRef<T, C>
where
    T: Send + Sync,
    C: RcuContext,
{
    type Output = RcuListRefOwned<T>;

    unsafe fn take_ownership(mut self) -> Self::Output {
        let output = RcuListRefOwned(Box::from_raw(self.ptr));

        // SAFETY: We don't want deferred cleanup when dropping `self`.
        self.ptr = std::ptr::null_mut();

        output
    }
}

impl<T, C> Deref for RcuListRef<T, C>
where
    T: Send + Sync,
    C: RcuContext,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.ptr).data }
    }
}

/// RCU linked list.
///
/// # Limitations
///
/// ##### Mutable References
///
/// Because there might always be readers borrowing a node's data, it is impossible
/// to get a mutable references to the data inside the linked list. You should design
/// the type stored in the list with [interior mutabillity] that can be shared between
/// threads (`T` requires `Send` and `Sync`).
///
/// [interior mutabillity]: https://doc.rust-lang.org/reference/interior-mutability.html
///
/// ##### List Length
///
/// Because a writer might concurrently modify the list, the amount of node might change
/// at any moment. To prevent user error (e.g. allocate an array for each node), there is
/// no `.len()` method.
///
/// That said, it could be implemented by the writer since it has exclusive access.
///
/// ##### Bidirectional Iteration
///
/// Because a writer might concurrently modify the list, it is possible that `node.next.prev != node`.
/// To prevent user error, this linked list does not support bidirectional iteration.
/// For example, if you create an forward iterator, it can only go forward.
///
/// That said, it could be implemented by the writer since it has exclusive access.
pub struct RcuList<T, C = DefaultContext>
where
    T: Send + Sync,
{
    head: AtomicPtr<RcuListNode<T>>,
    tail: AtomicPtr<RcuListNode<T>>,
    mutex: Arc<Mutex<()>>,
    // Also prevents auto-trait implementation of [`Send`] and [`Sync`].
    context: PhantomData<*const C>,
}

impl<T, C> RcuList<T, C>
where
    T: Send + Sync,
{
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

impl<T, C> Drop for RcuList<T, C>
where
    T: Send + Sync,
{
    fn drop(&mut self) {
        // SAFETY: Because of reference counting, there are no reader/writer threads accessing this object.
        let mut node_ptr = self.head.load(Ordering::Relaxed);
        while !node_ptr.is_null() {
            let next_ptr = unsafe { &*node_ptr }.next.load(Ordering::Relaxed);
            drop(unsafe { Box::from_raw(node_ptr) });
            node_ptr = next_ptr;
        }
    }
}

/// #### Safety
///
/// An [`RcuList`] can be used to send data to another thread.
unsafe impl<T, C> Send for RcuList<T, C> where T: Send + Sync {}

/// #### Safety
///
/// An [`RcuList`] can be used to share data between threads.
unsafe impl<T, C> Sync for RcuList<T, C> where T: Send + Sync {}

/// The read-side API of an [`RcuList`].
pub struct RcuListReader<'a, T, C>
where
    T: Send + Sync,
    C: RcuContext + 'a,
{
    list: Arc<RcuList<T, C>>,
    #[allow(dead_code)]
    guard: &'a C::Guard<'a>,
}

impl<'a, T, C> RcuListReader<'a, T, C>
where
    T: Send + Sync,
    C: RcuContext + 'a,
{
    pub fn iter_forward(&self) -> RcuListIterator<T, &Self> {
        RcuListIterator {
            reader: self,
            forward: true,
            ptr: self.list.head.load(Ordering::Acquire),
        }
    }

    pub fn iter_reverse(&self) -> RcuListIterator<T, &Self> {
        RcuListIterator {
            reader: self,
            forward: false,
            ptr: self.list.tail.load(Ordering::Acquire),
        }
    }
}

/// The write-side API of an [`RcuList`].
pub struct RcuListWriter<T, C>
where
    T: Send + Sync,
{
    list: Arc<RcuList<T, C>>,
    #[allow(dead_code)]
    guard: ArcMutexGuardian<()>,
}

impl<T, C> RcuListWriter<T, C>
where
    T: Send + Sync,
    C: RcuContext,
{
    pub fn pop_back(&mut self) -> Option<RcuListRef<T, C>> {
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

    pub fn pop_front(&mut self) -> Option<RcuListRef<T, C>> {
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
pub struct RcuListEntry<'a, T, C>
where
    T: Send + Sync,
{
    list: Arc<RcuList<T, C>>,
    node: NonNull<RcuListNode<T>>,
    #[allow(dead_code)]
    life: PhantomData<&'a T>,
}

impl<'a, T, C> RcuListEntry<'a, T, C>
where
    T: Send + Sync,
    C: RcuContext,
{
    /// Inserts a node after this entry.
    pub fn insert_after(&mut self, data: T) {
        let node = unsafe { self.node.as_mut() };
        let node_next = node.next.load(Ordering::Acquire);
        let node_new = RcuListNode::new(data);

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
        let node_prev = node.prev.load(Ordering::Acquire);
        let node_new = RcuListNode::new(data);

        unsafe {
            node.insert_after(node_new);
        }

        if node_prev.is_null() {
            self.list.head.store(node_new, Ordering::Release);
        }
    }

    /// Removes the node from the list.
    pub fn remove(self) -> RcuListRef<T, C> {
        let node = unsafe { self.node.as_ref() };
        let node_prev = node.prev.load(Ordering::Acquire);
        let node_next = node.next.load(Ordering::Acquire);

        if node_prev.is_null() {
            self.list.head.store(node_next, Ordering::Release);
        }

        if node_next.is_null() {
            self.list.tail.store(node_prev, Ordering::Release);
        }

        unsafe { RcuListNode::remove(self.node.as_ptr()) }
    }
}

/// An iterator over the nodes of an [`RcuList`].
pub struct RcuListIterator<T, O> {
    #[allow(dead_code)]
    reader: O,
    forward: bool,
    ptr: *const RcuListNode<T>,
}

impl<'a, T, C> Iterator for RcuListIterator<T, &'a RcuListReader<'a, T, C>>
where
    T: Send + Sync + 'a,
    C: RcuContext + 'a,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.ptr.is_null() {
            let item = unsafe { &*self.ptr };

            if self.forward {
                self.ptr = item.next.load(Ordering::Acquire);
            } else {
                self.ptr = item.prev.load(Ordering::Acquire);
            }

            Some(&item.data)
        } else {
            None
        }
    }
}

impl<'a, T, C> Iterator for RcuListIterator<T, &'a RcuListWriter<T, C>>
where
    T: Send + Sync + 'a,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.ptr.is_null() {
            let item = unsafe { &*self.ptr };

            if self.forward {
                self.ptr = item.next.load(Ordering::Acquire);
            } else {
                self.ptr = item.prev.load(Ordering::Acquire);
            }

            Some(&item.data)
        } else {
            None
        }
    }
}

impl<'a, T, C> Iterator for RcuListIterator<T, &'a mut RcuListWriter<T, C>>
where
    T: Send + Sync + 'a,
{
    type Item = RcuListEntry<'a, T, C>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.ptr.is_null() {
            let item = unsafe { &*self.ptr };

            if self.forward {
                self.ptr = item.next.load(Ordering::Acquire);
            } else {
                self.ptr = item.prev.load(Ordering::Acquire);
            }

            Some(RcuListEntry {
                node: unsafe { NonNull::new_unchecked(self.ptr as *mut RcuListNode<T>) },
                list: self.reader.list.clone(),
                life: PhantomData,
            })
        } else {
            None
        }
    }
}
