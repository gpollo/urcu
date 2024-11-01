use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::Arc;

use crate::rcu::{DefaultContext, RcuContext, RcuReadContext};
use crate::stack::iterator::{Iter, IterRef};
use crate::stack::raw::{RawNode, RawStack};
use crate::stack::reference::Ref;
use crate::utility::*;

/// Defines a RCU wait-free stack.
///
/// This stack supports multiple concurrents readers and writers. It is guaranteed to
/// never block on a call.
///
/// # Limitations
///
/// ##### Mutable References
///
/// Because there might always be readers borrowing a node's data, it is impossible
/// to get a mutable references to the data inside the stack. You should design the
/// type stored in the stack with [interior mutabillity] that can be shared between
/// threads.
///
/// [interior mutabillity]: https://doc.rust-lang.org/reference/interior-mutability.html
///
/// ##### List Length
///
/// Because a writer might concurrently modify the stack, the amount of node might change
/// at any moment. To prevent user error (e.g. allocate an array for each node), there is
/// no `.len()` method.
///
/// # Safety
///
/// It is safe to send an `Arc<RcuStack<T>>` to a non-registered RCU thread. A non-registered
/// thread may drop an `RcuStack<T>` without calling any RCU primitives since lifetime rules
/// prevent any other thread from accessing a RCU reference.
pub struct RcuStack<T, C = DefaultContext> {
    raw: RawStack<T>,
    _unsend: PhantomUnsend<(T, C)>,
    _unsync: PhantomUnsync<(T, C)>,
}

impl<T, C> RcuStack<T, C> {
    /// Creates a new RCU stack.
    pub fn new() -> Arc<Self> {
        Arc::new(RcuStack {
            // SAFETY: All node are pop'ed before dropping.
            raw: unsafe { RawStack::new() },
            _unsend: PhantomData,
            _unsync: PhantomData,
        })
    }

    /// Adds an element to the top of the stack.
    pub fn push(&self, data: T) {
        let node = RawNode::new(data);

        self.raw.push(node);
    }

    /// Removes an element from the top of the stack.
    pub fn pop(&self, guard: &C::Guard<'_>) -> Option<Ref<T, C::Flavor>>
    where
        T: Send,
        C: RcuReadContext,
    {
        let _ = guard;

        // SAFETY: The RCU critical section is enforced.
        // SAFETY: RCU grace period is enforced.
        let node = unsafe { self.raw.pop() };

        NonNull::new(node).map(Ref::new)
    }

    /// Removes all elements from the stack.
    pub fn pop_all(&self, _guard: &C::Guard<'_>) -> IterRef<T, C>
    where
        T: Send,
        C: RcuReadContext,
    {
        // SAFETY: The RCU critical section is enforced.
        // SAFETY: RCU grace period is enforced.
        IterRef::new(unsafe { self.raw.pop_all() })
    }

    /// Returns a reference to the element on top of the stack.
    pub fn peek<'me, 'ctx, 'guard>(&'me self, _guard: &'guard C::Guard<'ctx>) -> Option<&'guard T>
    where
        'me: 'guard,
        C: RcuReadContext,
    {
        // SAFETY: The RCU critical section is enforced.
        let node = unsafe { self.raw.head() };

        // SAFETY: The pointer can be safely converted to reference.
        unsafe { node.as_ref() }.map(|node| node.deref())
    }

    /// Returns an iterator over the stack.
    ///
    /// The iterator yields all items from top to bottom.
    pub fn iter<'me, 'ctx, 'guard>(
        &'me self,
        guard: &'guard C::Guard<'ctx>,
    ) -> Iter<'ctx, 'guard, T, C>
    where
        'me: 'guard,
        C: RcuReadContext,
    {
        // SAFETY: The RCU critical section is enforced.
        Iter::new(unsafe { self.raw.iter() }, guard)
    }

    /// Returns `true` if there is no node in the stack.
    pub fn is_empty(&self) -> bool {
        self.raw.empty()
    }
}

/// #### Safety
///
/// An [`RcuStack`] can be used to send `T` to another thread.
unsafe impl<T, C> Send for RcuStack<T, C>
where
    T: Send,
    C: RcuContext,
{
}

/// #### Safety
///
/// An [`RcuStack`] can be used to share `T` between threads.
unsafe impl<T, C> Sync for RcuStack<T, C>
where
    T: Sync,
    C: RcuContext,
{
}

impl<T, C> Drop for RcuStack<T, C> {
    fn drop(&mut self) {
        // SAFETY: The RCU read-lock is not needed there are no other writers.
        // SAFETY: The RCU grace period is not needed there are no other readers.
        let mut iter = unsafe { self.raw.pop_all() };

        // SAFETY: The RCU read-lock is not needed there are no other writers.
        while let Some(ptr) = unsafe { iter.next().as_mut() } {
            // SAFETY: The pointer is always non-null and valid.
            drop(unsafe { Box::from_raw(ptr) });
        }
    }
}
