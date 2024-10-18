use crate::hashmap::raw::RawIter;

/// An iterator over the nodes of an [`RcuHashMap`].
///
/// [`RcuHashMap`]: crate::hashmap::container::RcuHashMap
pub struct Iter<'guard, K, V, C>(RawIter<'guard, K, V, C>)
where
    K: 'guard,
    V: 'guard;

impl<'guard, K, V, C> Iter<'guard, K, V, C> {
    pub fn new(raw: RawIter<'guard, K, V, C>) -> Self {
        Self(raw)
    }
}

impl<'guard, K, V, C> Iterator for Iter<'guard, K, V, C> {
    type Item = (&'guard K, &'guard V);

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: The node pointer is convertible to a reference is non-null.
        unsafe { self.0.get().as_ref() }.map(|entry| {
            self.0.next();
            entry.as_refs()
        })
    }
}
