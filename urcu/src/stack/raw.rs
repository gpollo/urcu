use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::Deref;

use container_of::container_of;
use urcu_sys::lfs;
use urcu_sys::lfs::{Stack, StackHead, StackNode, StackPtr};

use crate::utility::*;

pub struct RawNode<T> {
    handle: StackNode,
    data: T,
}

impl<T> RawNode<T> {
    pub fn new(data: T) -> Box<Self> {
        let mut handle = MaybeUninit::<StackNode>::uninit();

        // SAFETY: We don't need to registered with RCU in any way.
        unsafe { lfs::node_init(handle.as_mut_ptr()) };

        Box::new(Self {
            // SAFETY: Data has been initialised by `lfs::node_init`.
            handle: unsafe { handle.assume_init() },
            data,
        })
    }

    fn into_handle(self: Box<Self>) -> *mut StackNode {
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
    handle: Stack,
    _unsend: PhantomUnsend<T>,
    _unsync: PhantomUnsync<T>,
}

impl<T> RawStack<T> {
    /// #### Safety
    ///
    /// The caller must pop all node before dropping this type.
    pub unsafe fn new() -> Self {
        let mut handle = MaybeUninit::<Stack>::uninit();

        // SAFETY: We don't need to registered with RCU in any way.
        unsafe { lfs::init(handle.as_mut_ptr()) };

        Self {
            // SAFETY: Data has been initialised by `lfs::init`.
            handle: unsafe { handle.assume_init() },
            _unsend: PhantomData,
            _unsync: PhantomData,
        }
    }

    pub fn push(&self, node: Box<RawNode<T>>) {
        let handle = &self.handle as *const Stack as *mut Stack;

        // SAFETY: The C call safely mutate the state shared between threads.
        unsafe { lfs::push(StackPtr { _s: handle }, node.into_handle()) };
    }

    /// #### Safety
    ///
    /// The caller must be inside an RCU critical section.
    ///
    /// The caller must wait an RCU grace period before freeing the node.
    pub unsafe fn pop(&self) -> *mut RawNode<T> {
        let handle = &self.handle as *const Stack as *mut Stack;

        // SAFETY: The C call safely mutate the state shared between threads.
        let handle = unsafe { lfs::pop(StackPtr { _s: handle }) };
        if handle.is_null() {
            std::ptr::null_mut()
        } else {
            container_of!(handle, RawNode<T>, handle)
        }
    }

    /// #### Safety
    ///
    /// The caller must be inside an RCU critical section.
    ///
    /// The caller must wait an RCU grace period before freeing the nodes.
    pub unsafe fn pop_all(&self) -> RawIterRef<T> {
        let handle = &self.handle as *const Stack as *mut Stack;

        RawIterRef::new(
            // SAFETY: The C call safely mutate the state shared between threads.
            unsafe { lfs::pop_all(StackPtr { _s: handle }) },
        )
    }

    /// #### Safety
    ///
    /// The caller must be inside an RCU critical section.
    pub unsafe fn head(&self) -> *const RawNode<T> {
        let handle = crate::rcu_dereference(self.handle.head);
        if handle.is_null() {
            std::ptr::null()
        } else {
            container_of!(handle, RawNode<T>, handle)
        }
    }

    /// #### Safety
    ///
    /// The caller must be inside an RCU critical section.
    pub unsafe fn iter(&self) -> RawIter<T> {
        RawIter::<T>::new(&self.handle)
    }

    pub fn empty(&self) -> bool {
        let handle = &self.handle as *const Stack as *mut Stack;

        // SAFETY: The C call does not mutate the shared state.
        unsafe { lfs::empty(StackPtr { _s: handle }) }
    }
}

pub struct RawIter<T> {
    node: *const StackNode,
    _unsend: PhantomUnsend<T>,
    _unsync: PhantomUnsync<T>,
}

impl<T> RawIter<T> {
    /// #### Safety
    ///
    /// The caller must be in an RCU critical section.
    unsafe fn new(stack: &Stack) -> Self {
        Self {
            node: crate::rcu_dereference(stack.head)
                .as_ref()
                .map(|head| crate::rcu_dereference(&head.node as *const StackNode))
                .unwrap_or(std::ptr::null()),
            _unsend: PhantomData,
            _unsync: PhantomData,
        }
    }

    /// #### Safety
    ///
    /// The caller must be in an RCU critical section.
    pub unsafe fn next(&mut self) -> *const RawNode<T> {
        match self.node.as_ref() {
            None => std::ptr::null(),
            Some(handle) => {
                self.node = crate::rcu_dereference(handle.next);
                container_of!(handle as *const StackNode, RawNode<T>, handle)
            }
        }
    }
}

pub struct RawIterRef<T> {
    node: *mut StackNode,
    _unsend: PhantomUnsend<T>,
    _unsync: PhantomUnsync<T>,
}

impl<T> RawIterRef<T> {
    /// #### Safety
    ///
    /// The head must be removed from the stack.
    unsafe fn new(head: *mut StackHead) -> Self {
        Self {
            node: head
                .as_mut()
                .map(|head| &mut head.node as *mut StackNode)
                .unwrap_or(std::ptr::null_mut()),
            _unsend: PhantomData,
            _unsync: PhantomData,
        }
    }

    /// #### Safety
    ///
    /// The caller must wait an RCU grace period before freeing the node.
    pub unsafe fn next(&mut self) -> *mut RawNode<T> {
        match self.node.as_mut() {
            None => std::ptr::null_mut(),
            Some(handle) => {
                self.node = handle.next;
                container_of!(handle as *mut StackNode, RawNode<T>, handle)
            }
        }
    }
}
