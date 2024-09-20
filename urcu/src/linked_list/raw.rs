use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::linked_list::reference::Ref;
use crate::RcuContext;

pub struct Node<T> {
    prev: AtomicPtr<Self>,
    next: AtomicPtr<Self>,
    data: T,
}

impl<T> Node<T> {
    pub fn new(data: T) -> *mut Self {
        Box::into_raw(Box::new(Self {
            prev: AtomicPtr::new(std::ptr::null_mut()),
            next: AtomicPtr::new(std::ptr::null_mut()),
            data,
        }))
    }

    /// Returns a mutable pointer to the previous node.
    ///
    /// #### Safety
    ///
    /// The node pointer must be non-null.
    pub unsafe fn prev_node(node: *const Self, ordering: Ordering) -> *mut Self {
        (*node).prev.load(ordering)
    }

    /// Returns a mutable pointer to the next node.
    ///
    /// #### Safety
    ///
    /// The node pointer must be non-null.
    pub unsafe fn next_node(node: *const Self, ordering: Ordering) -> *mut Self {
        (*node).next.load(ordering)
    }

    /// Insert a node after this one.
    ///
    /// #### Safety
    ///
    /// Require mutual exclusion on the list.
    pub unsafe fn insert_after(&mut self, new_prev_ptr: *mut Self) {
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
    /// #### Safety
    ///
    /// Require mutual exclusion on the list.
    pub unsafe fn insert_before(&mut self, new_prev_ptr: *mut Self) {
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
    /// #### Safety
    ///
    /// Require mutual exclusion on the list.
    pub unsafe fn remove<C>(ptr: *mut Self) -> Ref<T, C>
    where
        T: Send,
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

        Ref::new(ptr)
    }
}

impl<T> Deref for Node<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for Node<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
