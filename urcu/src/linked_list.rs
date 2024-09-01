use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use guardian::ArcMutexGuardian;

use crate::{RcuContext, RcuRef};

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
    unsafe fn remove<C>(ptr: *mut Self) -> RcuListRef<T, C> {
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
            context: PhantomData::default(),
        }
    }
}

#[must_use]
pub struct RcuListRef<T, C> {
    ptr: *mut RcuListNode<T>,
    context: PhantomData<C>,
}

impl<T, C> RcuRef<C> for RcuListRef<T, C> {
    type Output = T;

    unsafe fn take_ownership(self) -> Self::Output {
        Box::from_raw(self.ptr).data
    }
}

/// RCU linked list.
///
/// # Limitations
///
/// ##### List Length
///
/// Because a writer might concurrently modify the list, the amount of node might change at any moment.
/// To prevent user error (e.g. allocate an array for each node), there is no `.len()` method.
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
pub struct RcuList<T, C> {
    head: AtomicPtr<RcuListNode<T>>,
    tail: AtomicPtr<RcuListNode<T>>,
    mutex: Arc<Mutex<()>>,
    context: PhantomData<C>,
}

impl<T, C> RcuList<T, C> {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            head: AtomicPtr::new(std::ptr::null_mut()),
            tail: AtomicPtr::new(std::ptr::null_mut()),
            mutex: Arc::default(),
            context: PhantomData::default(),
        })
    }

    pub fn reader<'a>(self: &'a Arc<Self>, guard: &'a C::Guard<'a>) -> RcuListReader<T, C>
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
            let next_ptr = unsafe { &*node_ptr }.next.load(Ordering::Relaxed);
            drop(unsafe { Box::from_raw(node_ptr) });
            node_ptr = next_ptr;
        }
    }
}

/// Read-side API of an RCU list.
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

pub struct RcuListWriter<T, C> {
    list: Arc<RcuList<T, C>>,
    #[allow(dead_code)]
    guard: ArcMutexGuardian<()>,
}

impl<T, C> RcuListWriter<T, C> {
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
                life: PhantomData::default(),
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
                life: PhantomData::default(),
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
            unsafe { (&mut *tail_node_ptr).insert_after(new_node) };
        }

        self.list.tail.store(new_node, Ordering::Release);
    }

    pub fn push_front(&mut self, data: T) {
        let new_node = RcuListNode::new(data);

        let head_node_ptr = self.list.head.load(Ordering::Acquire);
        if head_node_ptr.is_null() {
            self.list.tail.store(new_node, Ordering::Relaxed);
        } else {
            unsafe { (&mut *head_node_ptr).insert_before(new_node) };
        }

        self.list.head.store(new_node, Ordering::Release);
    }
}

pub struct RcuListEntry<'a, T, C> {
    list: Arc<RcuList<T, C>>,
    node: NonNull<RcuListNode<T>>,
    #[allow(dead_code)]
    life: PhantomData<&'a T>,
}

impl<'a, T, C> RcuListEntry<'a, T, C> {
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

pub struct RcuListIterator<T, O> {
    #[allow(dead_code)]
    reader: O,
    forward: bool,
    ptr: *const RcuListNode<T>,
}

impl<'a, T, C> Iterator for RcuListIterator<T, &'a RcuListReader<'a, T, C>>
where
    T: 'a,
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
    T: 'a,
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
    T: 'a,
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
                life: PhantomData::default(),
            })
        } else {
            None
        }
    }
}
