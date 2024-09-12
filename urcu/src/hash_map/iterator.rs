use crate::hash_map::raw::RawIterator;

pub struct RcuHashMapIterator<'a, K, V, C>(RawIterator<'a, K, V, C>)
where
    K: 'a,
    V: 'a;

impl<'a, K, V, C> RcuHashMapIterator<'a, K, V, C> {
    pub fn new(raw: RawIterator<'a, K, V, C>) -> Self {
        Self(raw)
    }
}

impl<'a, K, V, C> Iterator for RcuHashMapIterator<'a, K, V, C> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.get().map(|entry| {
            self.0.next();
            entry.as_refs()
        })
    }
}
