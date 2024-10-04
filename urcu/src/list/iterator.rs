use std::ops::Deref;

use crate::list::raw::RawIter;
use crate::rcu::RcuContext;

/// An iterator over the nodes of an [`RcuList`].
///
/// [`RcuList`]: crate::list::container::RcuList
pub struct Iter<'a, T, C, const FORWARD: bool>
where
    C: RcuContext + 'a,
{
    raw: RawIter<T, FORWARD>,
    _guard: &'a C::Guard<'a>,
}

impl<'a, T, C, const FORWARD: bool> Iter<'a, T, C, FORWARD>
where
    C: RcuContext + 'a,
{
    pub(crate) fn new(raw: RawIter<T, FORWARD>, guard: &'a C::Guard<'a>) -> Self {
        Self { raw, _guard: guard }
    }
}

impl<'a, T, C, const FORWARD: bool> Iterator for Iter<'a, T, C, FORWARD>
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
