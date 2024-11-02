use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::Deref;

use container_of::container_of;
use urcu_cds_sys::lfs;

use crate::utility::*;

pub struct RawNode<T> {
    handle: lfs::Node,
    data: T,
}

impl<T> RawNode<T> {
    pub fn new(data: T) -> Box<Self> {
        let mut handle = MaybeUninit::<lfs::Node>::uninit();

        // SAFETY: We don't need to registered with RCU in any way.
        unsafe { lfs::node_init(handle.as_mut_ptr()) };

        Box::new(Self {
            // SAFETY: Data has been initialised by `lfs::node_init`.
            handle: unsafe { handle.assume_init() },
            data,
        })
    }

    fn into_handle(self: Box<Self>) -> *mut lfs::Node {
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

pub struct RawStack<T> {
    handle: lfs::__Stack,
    _unsend: PhantomUnsend<T>,
    _unsync: PhantomUnsync<T>,
}

impl<T> RawStack<T> {
    /// #### Safety
    ///
    /// The caller must pop all node before dropping this type.
    pub unsafe fn new() -> Self {
        let mut handle = MaybeUninit::<lfs::__Stack>::uninit();

        // SAFETY: We don't need to registered with RCU in any way.
        unsafe { lfs::__init(handle.as_mut_ptr()) };

        Self {
            // SAFETY: Data has been initialised by `lfs::init`.
            handle: unsafe { handle.assume_init() },
            _unsend: PhantomData,
            _unsync: PhantomData,
        }
    }

    pub fn push(&self, node: Box<RawNode<T>>) {
        let handle = &self.handle as *const lfs::__Stack as *mut lfs::__Stack;

        // SAFETY: The C call safely mutate the state shared between threads.
        unsafe { lfs::push(lfs::StackPtr { _s: handle }, node.into_handle()) };
    }

    /// #### Safety
    ///
    /// The caller must be inside a RCU critical section.
    ///
    /// The caller must wait a RCU grace period before freeing the node.
    pub unsafe fn pop(&self) -> *mut RawNode<T> {
        let handle = &self.handle as *const lfs::__Stack as *mut lfs::__Stack;

        // SAFETY: The C call safely mutate the state shared between threads.
        let handle = unsafe { lfs::__pop(lfs::StackPtr { _s: handle }) };
        if handle.is_null() {
            std::ptr::null_mut()
        } else {
            container_of!(handle, RawNode<T>, handle)
        }
    }

    /// #### Safety
    ///
    /// The caller must be inside a RCU critical section.
    ///
    /// The caller must wait a RCU grace period before freeing the nodes.
    pub unsafe fn pop_all(&self) -> RawIterRef<T> {
        let handle = &self.handle as *const lfs::__Stack as *mut lfs::__Stack;

        RawIterRef::new(
            // SAFETY: The C call safely mutate the state shared between threads.
            unsafe { lfs::__pop_all(lfs::StackPtr { _s: handle }) },
        )
    }

    /// #### Safety
    ///
    /// The caller must be inside a RCU critical section.
    pub unsafe fn head(&self) -> *const RawNode<T> {
        let handle = crate::rcu::dereference(self.handle.head);
        if handle.is_null() {
            std::ptr::null()
        } else {
            container_of!(handle, RawNode<T>, handle)
        }
    }

    /// #### Safety
    ///
    /// The caller must be inside a RCU critical section.
    pub unsafe fn iter(&self) -> RawIter<T> {
        RawIter::<T>::new(&self.handle)
    }

    pub fn empty(&self) -> bool {
        let handle = &self.handle as *const lfs::__Stack as *mut lfs::__Stack;

        // SAFETY: The C call does not mutate the shared state.
        unsafe { lfs::empty(lfs::StackPtr { _s: handle }) }
    }
}

pub struct RawIter<T> {
    node: *const lfs::Node,
    _unsend: PhantomUnsend<T>,
    _unsync: PhantomUnsync<T>,
}

impl<T> RawIter<T> {
    /// #### Safety
    ///
    /// The caller must be in a RCU critical section.
    unsafe fn new(stack: &lfs::__Stack) -> Self {
        Self {
            node: crate::rcu::dereference(stack.head)
                .as_ref()
                .map(|head| crate::rcu::dereference(&head.node as *const lfs::Node))
                .unwrap_or(std::ptr::null()),
            _unsend: PhantomData,
            _unsync: PhantomData,
        }
    }

    /// #### Safety
    ///
    /// The caller must be in a RCU critical section.
    pub unsafe fn next(&mut self) -> *const RawNode<T> {
        match self.node.as_ref() {
            None => std::ptr::null(),
            Some(handle) => {
                self.node = crate::rcu::dereference(handle.next);
                container_of!(handle as *const lfs::Node, RawNode<T>, handle)
            }
        }
    }
}

pub struct RawIterRef<T> {
    node: *mut lfs::Node,
    _unsend: PhantomUnsend<T>,
    _unsync: PhantomUnsync<T>,
}

impl<T> RawIterRef<T> {
    /// #### Safety
    ///
    /// The head must be removed from the stack.
    unsafe fn new(head: *mut lfs::Head) -> Self {
        Self {
            node: head
                .as_mut()
                .map(|head| &mut head.node as *mut lfs::Node)
                .unwrap_or(std::ptr::null_mut()),
            _unsend: PhantomData,
            _unsync: PhantomData,
        }
    }

    /// #### Safety
    ///
    /// The caller must wait a RCU grace period before freeing the node.
    pub unsafe fn next(&mut self) -> *mut RawNode<T> {
        match self.node.as_mut() {
            None => std::ptr::null_mut(),
            Some(handle) => {
                self.node = handle.next;
                container_of!(handle as *mut lfs::Node, RawNode<T>, handle)
            }
        }
    }
}
