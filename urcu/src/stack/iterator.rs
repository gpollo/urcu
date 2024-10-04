use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;

use crate::rcu::RcuContext;
use crate::stack::raw::{RawIter, RawIterRef};
use crate::stack::reference::Ref;
use crate::utility::*;

/// An iterator over the nodes of an [`RcuStack`].
///
/// [`RcuStack`]: crate::stack::container::RcuStack
pub struct Iter<'a, T, C>
where
    C: RcuContext + 'a,
{
    raw: RawIter<T>,
    _guard: &'a C::Guard<'a>,
    _unsend: PhantomUnsend,
    _unsync: PhantomUnsync,
}

impl<'a, T, C> Iter<'a, T, C>
where
    C: RcuContext + 'a,
{
    pub(crate) fn new(raw: RawIter<T>, guard: &'a C::Guard<'a>) -> Self {
        Self {
            raw,
            _guard: guard,
            _unsend: PhantomData,
            _unsync: PhantomData,
        }
    }
}

impl<'a, T, C> Iterator for Iter<'a, T, C>
where
    Self: 'a,
    C: RcuContext,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: The RCU critical section is enforced.
        unsafe { self.raw.next().as_ref() }.map(|node| node.deref())
    }
}

/// An iterator over popped nodes of an [`RcuStack`].
///
/// [`RcuStack`]: crate::stack::container::RcuStack
pub struct IterRef<T, C> {
    raw: RawIterRef<T>,
    _unsend: PhantomUnsend<C>,
    _unsync: PhantomUnsync<C>,
}

impl<T, C> IterRef<T, C> {
    pub(crate) fn new(raw: RawIterRef<T>) -> Self {
        Self {
            raw,
            _unsend: PhantomData,
            _unsync: PhantomData,
        }
    }
}

impl<T, C> Iterator for IterRef<T, C>
where
    T: Send + 'static,
    C: RcuContext + 'static,
{
    type Item = Ref<T, C>;

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: The grace period is enforced by [`Ref`].
        NonNull::new(unsafe { self.raw.next() }).map(Ref::new)
    }
}
