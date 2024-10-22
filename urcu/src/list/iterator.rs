use std::ops::Deref;

use crate::list::raw::RawIter;
use crate::rcu::RcuReadContext;

/// An iterator over the nodes of an [`RcuList`].
///
/// [`RcuList`]: crate::list::container::RcuList
pub struct Iter<'ctx, 'guard, T, C, const FORWARD: bool>
where
    C: RcuReadContext + 'ctx,
{
    raw: RawIter<T, FORWARD>,
    #[allow(dead_code)]
    guard: &'guard C::Guard<'ctx>,
}

impl<'ctx, 'guard, T, C, const FORWARD: bool> Iter<'ctx, 'guard, T, C, FORWARD>
where
    C: RcuReadContext + 'ctx,
{
    pub(crate) fn new(raw: RawIter<T, FORWARD>, guard: &'guard C::Guard<'ctx>) -> Self {
        Self { raw, guard }
    }
}

impl<'ctx, 'guard, T, C, const FORWARD: bool> Iterator for Iter<'ctx, 'guard, T, C, FORWARD>
where
    Self: 'guard,
    C: RcuReadContext + 'ctx,
{
    type Item = &'guard T;

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: The RCU critical section is enforced.
        unsafe { self.raw.next().as_ref() }.map(|node| node.deref())
    }
}
