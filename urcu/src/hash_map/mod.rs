pub(crate) mod iterator;
pub(crate) mod raw;
pub(crate) mod reference;

use std::hash::Hash;
use std::ops::Deref;
use std::sync::Arc;

use anyhow::Result;

use crate::hash_map::iterator::RcuHashMapIterator;
use crate::hash_map::raw::RawMap;
use crate::hash_map::reference::RcuHashMapRef;
use crate::RcuContext;

pub struct RcuHashMap<K, V, C>(RawMap<K, V, C>);

impl<K, V, C> RcuHashMap<K, V, C> {
    pub fn new() -> Result<Arc<Self>>
    where
        C: RcuContext,
    {
        Ok(Arc::new(Self(RawMap::new()?)))
    }

    pub fn access<'a>(
        self: &'a Arc<Self>,
        guard: &'a C::Guard<'a>,
    ) -> RcuHashMapAccessor<'a, K, V, C>
    where
        C: RcuContext + 'a,
    {
        RcuHashMapAccessor {
            map: self.clone(),
            guard,
        }
    }
}

impl<K, V, C> Drop for RcuHashMap<K, V, C> {
    fn drop(&mut self) {
        // unsafe { self.map.0.iter() },
    }
}

pub struct RcuHashMapAccessor<'a, K, V, C>
where
    C: RcuContext + 'a,
{
    map: Arc<RcuHashMap<K, V, C>>,
    #[allow(dead_code)]
    guard: &'a C::Guard<'a>,
}

impl<'a, K, V, C> RcuHashMapAccessor<'a, K, V, C>
where
    C: RcuContext + 'a,
{
    pub fn insert(&self, key: K, value: V) -> Option<RcuHashMapRef<K, V, C>>
    where
        K: Send + Eq + Hash,
        V: Send,
    {
        // SAFETY: The read-side RCU lock is taken.
        // SAFETY: The RCU grace period is enforced through the RcuRef.
        unsafe { self.map.0.add_replace(key, value).map(RcuHashMapRef::new) }
    }

    pub fn get(&self, key: &K) -> Option<&V>
    where
        K: Eq + Hash,
    {
        // SAFETY: The read-side RCU lock is taken.
        unsafe { self.map.0.lookup(key).get().map(|node| node.deref()) }
    }

    pub fn remove(&self, key: &K) -> Option<RcuHashMapRef<K, V, C>>
    where
        K: Send + Eq + Hash,
        V: Send,
    {
        // SAFETY: The read-side RCU lock is taken.
        // SAFETY: The RCU grace period is enforced through the RcuRef.
        unsafe {
            self.map
                .0
                .lookup(key)
                .get_mut()
                .map(|node_ptr| self.map.0.del(node_ptr))
                .flatten()
                .map(RcuHashMapRef::new)
        }
    }

    pub fn iter(&self) -> RcuHashMapIterator<'_, K, V, C> {
        RcuHashMapIterator::new(
            // SAFETY: The read-side RCU lock is taken.
            unsafe { self.map.0.iter() },
        )
    }
}
