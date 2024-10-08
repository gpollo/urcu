use crate::hashmap::raw::RawIter;

/// An iterator over the nodes of an [`RcuHashMap`].
///
/// [`RcuHashMap`]: crate::hashmap::container::RcuHashMap
pub struct Iter<'a, K, V, C>(RawIter<'a, K, V, C>)
where
    K: 'a,
    V: 'a;

impl<'a, K, V, C> Iter<'a, K, V, C> {
    pub fn new(raw: RawIter<'a, K, V, C>) -> Self {
        Self(raw)
    }
}

impl<'a, K, V, C> Iterator for Iter<'a, K, V, C> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: The node pointer is convertible to a reference is non-null.
        unsafe { self.0.get().as_ref() }.map(|entry| {
            self.0.next();
            entry.as_refs()
        })
    }
}
