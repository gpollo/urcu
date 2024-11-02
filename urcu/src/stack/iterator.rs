use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;

use crate::rcu::flavor::RcuFlavor;
use crate::rcu::guard::RcuGuard;
use crate::stack::raw::{RawIter, RawIterRef};
use crate::stack::reference::Ref;
use crate::utility::*;

/// An iterator over the nodes of an [`RcuStack`].
///
/// [`RcuStack`]: crate::stack::container::RcuStack
pub struct Iter<'guard, T, G>
where
    G: RcuGuard,
{
    raw: RawIter<T>,
    _guard: &'guard G,
    _unsend: PhantomUnsend,
    _unsync: PhantomUnsync,
}

impl<'guard, T, G> Iter<'guard, T, G>
where
    G: RcuGuard,
{
    pub(crate) fn new(raw: RawIter<T>, guard: &'guard G) -> Self {
        Self {
            raw,
            _guard: guard,
            _unsend: PhantomData,
            _unsync: PhantomData,
        }
    }
}

impl<'guard, T, G> Iterator for Iter<'guard, T, G>
where
    Self: 'guard,
    G: RcuGuard,
{
    type Item = &'guard T;

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: The RCU critical section is enforced.
        unsafe { self.raw.next().as_ref() }.map(|node| node.deref())
    }
}

/// An iterator over popped nodes of an [`RcuStack`].
///
/// [`RcuStack`]: crate::stack::container::RcuStack
pub struct IterRef<T, F> {
    raw: RawIterRef<T>,
    _unsend: PhantomUnsend<F>,
    _unsync: PhantomUnsync<F>,
}

impl<T, F> IterRef<T, F> {
    pub(crate) fn new(raw: RawIterRef<T>) -> Self {
        Self {
            raw,
            _unsend: PhantomData,
            _unsync: PhantomData,
        }
    }
}

impl<T, F> Iterator for IterRef<T, F>
where
    T: Send + 'static,
    F: RcuFlavor + 'static,
{
    type Item = Ref<T, F>;

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: The grace period is enforced by [`Ref`].
        NonNull::new(unsafe { self.raw.next() }).map(Ref::new)
    }
}
