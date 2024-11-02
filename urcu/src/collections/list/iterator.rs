use std::ops::Deref;

use crate::collections::list::raw::RawIter;
use crate::rcu::guard::RcuGuard;

/// An iterator over the nodes of an [`RcuList`].
///
/// [`RcuList`]: crate::collections::list::container::RcuList
pub struct Iter<'guard, T, G, const FORWARD: bool>
where
    G: RcuGuard,
{
    raw: RawIter<T, FORWARD>,
    #[allow(dead_code)]
    guard: &'guard G,
}

impl<'guard, T, G, const FORWARD: bool> Iter<'guard, T, G, FORWARD>
where
    G: RcuGuard,
{
    pub(crate) fn new(raw: RawIter<T, FORWARD>, guard: &'guard G) -> Self {
        Self { raw, guard }
    }
}

impl<'guard, T, G, const FORWARD: bool> Iterator for Iter<'guard, T, G, FORWARD>
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
