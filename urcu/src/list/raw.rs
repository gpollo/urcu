use std::marker::PhantomData;
use std::ops::Deref;

use container_of::container_of;
use urcu_sys::list::{self, ListHead};

use crate::utility::*;

pub struct RawNode<T> {
    handle: ListHead,
    data: T,
}

impl<T> RawNode<T> {
    pub fn new(data: T) -> Box<Self> {
        Box::new(Self {
            handle: Default::default(),
            data,
        })
    }

    fn into_handle(self: Box<Self>) -> *mut ListHead {
        let node_ptr = Box::into_raw(self);
        let node = unsafe { node_ptr.as_mut_unchecked() };
        &mut node.handle
    }
}

impl<T> Deref for RawNode<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

/// #### Safety
///
/// It is safe to send a [`RawNode<T>`] to another thread if `T` is [`Send`].
unsafe impl<T: Send> Send for RawNode<T> {}

/// #### Safety
///
/// It is safe to share a [`RawNode<T>`] between threads if `T` is [`Sync`].
unsafe impl<T: Sync> Sync for RawNode<T> {}

pub struct RawList<T> {
    back: ListHead,
    front: ListHead,
    _unsend: PhantomUnsend<T>,
    _unsync: PhantomUnsync<T>,
}

impl<T> RawList<T> {
    /// #### Safety
    ///
    /// The caller must call [`RawList::init`] once [`RawList`] is in a stable memory location.
    pub unsafe fn new() -> Self {
        Self {
            back: Default::default(),
            front: Default::default(),
            _unsend: PhantomData,
            _unsync: PhantomData,
        }
    }

    /// #### Safety
    ///
    /// The caller must ensure [`RawList`] is in a stable memory location.
    /// The caller must remove all nodes before dropping this type.
    pub unsafe fn init(&mut self) {
        self.back.next = &mut self.front;
        self.front.prev = &mut self.back;
    }

    /// #### Safety
    ///
    /// The caller must have mutual exclusion from other writers.
    pub unsafe fn insert_back(&self, node: Box<RawNode<T>>) {
        let back = &self.back as *const ListHead as *mut ListHead;

        // SAFETY: The C call safely mutate the state shared between threads.
        unsafe { list::add_rcu(node.into_handle(), back) }
    }

    /// #### Safety
    ///
    /// The caller must have mutual exclusion from other writers.
    pub unsafe fn insert_front(&self, node: Box<RawNode<T>>) {
        let front = &self.front as *const ListHead as *mut ListHead;

        // SAFETY: The C call safely mutate the state shared between threads.
        unsafe { list::add_tail_rcu(node.into_handle(), front) }
    }

    /// #### Safety
    ///
    /// The caller must have mutual exclusion from other writers.
    ///
    /// The caller must wait an RCU grace period before freeing the node.
    pub unsafe fn remove_back(&self) -> *mut RawNode<T> {
        let handle = self.back.next;

        if handle as *const ListHead != &self.front {
            // SAFETY: The C call safely mutate the state shared between threads.
            unsafe { list::del_rcu(handle) };
            container_of!(handle, RawNode<T>, handle)
        } else {
            std::ptr::null_mut()
        }
    }

    /// #### Safety
    ///
    /// The caller must have mutual exclusion from other writers.
    ///
    /// The caller must wait an RCU grace period before freeing the node.
    pub unsafe fn remove_front(&self) -> *mut RawNode<T> {
        let handle = self.front.prev;

        if handle as *const ListHead != &self.back {
            // SAFETY: The C call safely mutate the state shared between threads.
            unsafe { list::del_rcu(handle) };
            container_of!(handle, RawNode<T>, handle)
        } else {
            std::ptr::null_mut()
        }
    }

    /// #### Safety
    ///
    /// The caller must be in an RCU critical section.
    pub unsafe fn get_back(&self) -> *const RawNode<T> {
        let handle = self.back.next as *const ListHead;

        if handle != &self.front {
            container_of!(handle, RawNode<T>, handle)
        } else {
            std::ptr::null_mut()
        }
    }

    /// #### Safety
    ///
    /// The caller must be in an RCU critical section.
    pub unsafe fn get_front(&self) -> *const RawNode<T> {
        let handle = self.front.prev as *const ListHead;

        if handle != &self.back {
            container_of!(handle, RawNode<T>, handle)
        } else {
            std::ptr::null_mut()
        }
    }

    pub fn empty(&self) -> bool {
        self.back.next as *const ListHead == &self.front
    }
}

pub struct RawIter<T, const FORWARD: bool> {
    current: *const ListHead,
    last: *const ListHead,
    _unsend: PhantomUnsend<T>,
    _unsync: PhantomUnsync<T>,
}

impl<T> RawIter<T, true> {
    /// #### Safety
    ///
    /// The caller must be in an RCU critical section.
    pub unsafe fn from_back(list: &RawList<T>) -> Self {
        Self {
            current: crate::rcu_dereference(list.back.next),
            last: &list.front,
            _unsend: PhantomData,
            _unsync: PhantomData,
        }
    }
}

impl<T> RawIter<T, false> {
    /// #### Safety
    ///
    /// The caller must be in an RCU critical section.
    pub unsafe fn from_front(list: &RawList<T>) -> Self {
        Self {
            current: crate::rcu_dereference(list.front.prev),
            last: &list.back,
            _unsend: PhantomData,
            _unsync: PhantomData,
        }
    }
}

impl<T, const FORWARD: bool> RawIter<T, FORWARD> {
    /// #### Safety
    ///
    /// The caller must be in an RCU critical section.
    pub unsafe fn next(&mut self) -> *const RawNode<T> {
        if self.current == self.last {
            return std::ptr::null();
        }

        match self.current.as_ref() {
            None => std::ptr::null(),
            Some(handle) => {
                self.current = if FORWARD {
                    crate::rcu_dereference_mut(handle.next)
                } else {
                    crate::rcu_dereference_mut(handle.prev)
                };

                container_of!(handle as *const ListHead, RawNode<T>, handle)
            }
        }
    }
}
