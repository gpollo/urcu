use crate::collections::hashmap::raw::RawIter;

/// An iterator over the nodes of an [`RcuHashMap`].
///
/// [`RcuHashMap`]: crate::collections::hashmap::container::RcuHashMap
pub struct Iter<'guard, K, V, F>(RawIter<'guard, K, V, F>)
where
    K: 'guard,
    V: 'guard;

impl<'guard, K, V, F> Iter<'guard, K, V, F> {
    pub fn new(raw: RawIter<'guard, K, V, F>) -> Self {
        Self(raw)
    }
}

impl<'guard, K, V, F> Iterator for Iter<'guard, K, V, F> {
    type Item = (&'guard K, &'guard V);

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: The node pointer is convertible to a reference is non-null.
        unsafe { self.0.get().as_ref() }.map(|entry| {
            self.0.next();
            entry.as_refs()
        })
    }
}
