use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::Deref;

use container_of::container_of;
use urcu_cds_sys::lfq;

use crate::rcu::flavor::RcuFlavor;
use crate::rcu::RcuContext;
use crate::utility::*;

pub struct RawNode<T> {
    handle: lfq::NodeRcu,
    data: T,
}
impl<T> RawNode<T> {
    pub fn new(data: T) -> Box<Self> {
        let mut handle = MaybeUninit::<lfq::NodeRcu>::uninit();

        // SAFETY: We don't need to registered with RCU in any way.
        unsafe { lfq::node_init_rcu(handle.as_mut_ptr()) };

        Box::new(Self {
            // SAFETY: Data has been initialised by `lfq::node_init`.
            handle: unsafe { handle.assume_init() },
            data,
        })
    }

    fn into_handle(self: Box<Self>) -> *mut lfq::NodeRcu {
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

pub struct RawQueue<T, C> {
    handle: lfq::QueueRcu,
    _unsend: PhantomUnsend<(T, C)>,
    _unsync: PhantomUnsync<(T, C)>,
}

impl<T, C> RawQueue<T, C> {
    /// #### Safety
    ///
    /// The caller must call [`RawQueue::init`] once [`RawQueue`] is in a stable memory location.
    pub unsafe fn new() -> Self {
        Self {
            handle: lfq::QueueRcu {
                head: std::ptr::null_mut(),
                tail: std::ptr::null_mut(),
                queue_call_rcu: None,
            },
            _unsend: PhantomData,
            _unsync: PhantomData,
        }
    }

    /// #### Safety
    ///
    /// The caller must ensure [`RawQueue`] is in a stable memory location.
    /// The caller must remove all nodes before dropping this type.
    pub unsafe fn init(&mut self)
    where
        C: RcuContext,
    {
        // SAFETY: We don't need to registered with RCU in any way.
        // SAFETY: The unchecked API is used by the C code.
        unsafe {
            lfq::init_rcu(
                &mut self.handle,
                C::Flavor::unchecked_rcu_api().update_call_rcu,
            )
        };
    }

    /// #### Safety
    ///
    /// The caller must be inside a RCU critical section.
    pub unsafe fn enqueue(&self, node: Box<RawNode<T>>) {
        let handle = &self.handle as *const lfq::QueueRcu as *mut lfq::QueueRcu;

        // SAFETY: The C call safely mutate the state shared between threads.
        unsafe { lfq::enqueue_rcu(handle, node.into_handle()) }
    }

    // #### Safety
    //
    // The caller must be inside a RCU critical section.
    //
    // The caller must wait a RCU grace period before freeing the node.
    pub unsafe fn dequeue(&self) -> *mut RawNode<T> {
        let handle = &self.handle as *const lfq::QueueRcu as *mut lfq::QueueRcu;

        println!("DEQUEUE {:?}", handle);

        // SAFETY: The C call safely mutate the state shared between threads.
        let handle = unsafe { lfq::dequeue_rcu(handle) };
        if handle.is_null() {
            std::ptr::null_mut()
        } else {
            container_of!(handle, RawNode<T>, handle)
        }
    }

    // #### Safety
    //
    // The caller must be inside a RCU critical section.
    //
    // The caller must wait a RCU grace period before freeing the nodes.
    pub unsafe fn dequeue_all(&self) -> Vec<*mut RawNode<T>> {
        let mut ptrs = Vec::new();

        loop {
            let ptr = self.dequeue();
            if ptr.is_null() {
                break;
            }

            ptrs.push(ptr);
        }

        ptrs
    }
}

impl<T, C> Drop for RawQueue<T, C> {
    fn drop(&mut self) {
        // SAFETY: The queue creator must empty the queue before dropping.
        let ret = unsafe { lfq::destroy_rcu(&mut self.handle) };

        if ret != 0 {
            log::error!("raw queue was not emptied before dropping");
        }
    }
}
