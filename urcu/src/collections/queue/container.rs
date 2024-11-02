use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::Arc;

use crate::collections::queue::raw::{RawNode, RawQueue};
use crate::collections::queue::reference::Ref;
use crate::rcu::flavor::{DefaultFlavor, RcuFlavor};
use crate::rcu::guard::RcuGuard;
use crate::utility::*;

/// Defines a RCU wait-free queue.
///
/// This queue supports multiple concurrents readers and writers. It is guaranteed to
/// never block on a call.
///
/// # Limitations
///
/// ##### References
///
/// This queue currently do not offer a way to peek the back or front of the queue. It is
/// also currently not possible to iterate over the queue. Because of this, it is impossible
/// to get any sort of references. The only way to get data is to remove it from the queue
/// with [`RcuQueue::pop`].
///
/// # Safety
///
/// It is safe to send an `Arc<RcuQueue<T>>` to a non-registered RCU thread. A non-registered
/// thread may drop an `RcuQueue<T>` without calling any RCU primitives since lifetime rules
/// prevent any other thread from accessing a RCU reference.
pub struct RcuQueue<T, F = DefaultFlavor> {
    raw: RawQueue<T, F>,
    _unsend: PhantomUnsend,
    _unsync: PhantomUnsync,
}

impl<T, F> RcuQueue<T, F>
where
    F: RcuFlavor,
{
    /// Creates a new RCU queue.
    pub fn new() -> Arc<Self> {
        let mut queue = Arc::new(RcuQueue {
            // SAFETY: Initialisation is properly called.
            raw: unsafe { RawQueue::new() },
            _unsend: PhantomData,
            _unsync: PhantomData,
        });

        // SAFETY: Initialisation occurs when raw queue is in a stable memory location.
        // SAFETY: All the nodes are removed upon dropping.
        unsafe { Arc::<Self>::get_mut(&mut queue).unwrap().raw.init() };

        queue
    }

    /// Adds an element to the back of queue.
    pub fn push<G>(&self, data: T, _guard: &G)
    where
        T: Send,
        G: RcuGuard<Flavor = F>,
    {
        let node = RawNode::new(data);

        // SAFETY: The RCU read-lock is taken.
        unsafe { self.raw.enqueue(node) };
    }

    /// Removes an element to the front of the queue, if any.
    pub fn pop<G>(&self, _guard: &G) -> Option<Ref<T, F>>
    where
        T: Send,
        G: RcuGuard<Flavor = F>,
    {
        // SAFETY: The RCU read-lock is taken.
        // SAFETY: The RCU grace period is enforced using `Ref<T, F>`.
        NonNull::new(unsafe { self.raw.dequeue() }).map(Ref::<T, F>::new)
    }
}

/// #### Safety
///
/// An [`RcuQueue`] can be used to send `T` to another thread.
unsafe impl<T, F> Send for RcuQueue<T, F>
where
    T: Send,
    F: RcuFlavor,
{
}

/// #### Safety
///
/// An [`RcuQueue`] can be used to share `T` between threads.
unsafe impl<T, F> Sync for RcuQueue<T, F>
where
    T: Sync,
    F: RcuFlavor,
{
}

impl<T, F> Drop for RcuQueue<T, F> {
    fn drop(&mut self) {
        // SAFETY: The RCU read-lock is not needed there are no other writers.
        // SAFETY: The RCU grace period is not needed there are no other readers.
        for ptr in unsafe { self.raw.dequeue_all() } {
            // SAFETY: The pointer is always non-null and valid.
            drop(unsafe { Box::from_raw(ptr) });
        }
    }
}
