use std::hash::Hash;
use std::ptr::NonNull;
use std::sync::Arc;

use anyhow::Result;

use crate::collections::hashmap::iterator::Iter;
use crate::collections::hashmap::raw::RawMap;
use crate::collections::hashmap::reference::Ref;
use crate::rcu::default::RcuDefaultFlavor;
use crate::rcu::flavor::RcuFlavor;
use crate::{RcuGuard, RcuReadContext, RcuRef};

/// Defines a RCU lock-free hashmap.
///
/// This hashmap supports multiple concurrents readers and writers. It is guaranteed
/// to never block on a call.
///
/// # Limitations
///
/// ##### Mutable References
///
/// Because there might always be readers borrowing a node's data, it is impossible
/// to get a mutable references to the data inside the stack. You should design the
/// type stored in the stack with [interior mutabillity] that can be shared between
/// threads.
///
/// [interior mutabillity]: https://doc.rust-lang.org/reference/interior-mutability.html
///
/// # Safety
///
/// It is safe to send an `Arc<RcuHashMap<T>>` to a non-registered RCU thread. A
/// non-registered thread may drop an `RcuHashMap<T>` without calling any RCU
/// primitives since lifetime rules prevent any other thread from accessing an
/// RCU reference.
pub struct RcuHashMap<K, V, F = RcuDefaultFlavor>(RawMap<K, V, F>)
where
    K: Send + 'static,
    V: Send + 'static,
    F: RcuFlavor + 'static;

impl<K, V, F> RcuHashMap<K, V, F>
where
    K: Send,
    V: Send,
    F: RcuFlavor,
{
    /// Creates a new RCU hashmap.
    pub fn new() -> Result<Arc<Self>> {
        Ok(Arc::new(Self(RawMap::new()?)))
    }

    /// Inserts a key-value pair in the hashmap.
    ///
    /// If the hashmap did not have this key present, [`None`] is returned.
    pub fn insert<G>(&self, key: K, value: V, guard: &G) -> Option<Ref<K, V, F>>
    where
        K: Send + Eq + Hash,
        V: Send,
        G: RcuGuard<Flavor = F>,
    {
        let _ = guard;

        // SAFETY: The read-side RCU lock is taken.
        // SAFETY: The RCU grace period is enforced through the RcuRef.
        let node = unsafe { self.0.add_replace(key, value) };

        NonNull::new(node).map(Ref::new)
    }

    /// Returns `true` if the hashmap contains a value for the specified key.
    pub fn contains<G>(&self, key: &K, guard: &G) -> bool
    where
        K: Eq + Hash,
        G: RcuGuard<Flavor = F>,
    {
        let _ = guard;

        // SAFETY: The RCU read-side lock is taken.
        let mut iter = unsafe { self.0.lookup(key) };

        !iter.get().is_null()
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get<'me, 'guard, G>(&'me self, key: &K, _guard: &'guard G) -> Option<&'guard V>
    where
        'me: 'guard,
        K: Eq + Hash,
        G: RcuGuard<Flavor = F>,
    {
        // SAFETY: The RCU read-side lock is taken.
        let mut iter = unsafe { self.0.lookup(key) };

        // SAFETY: The node pointer is convertible to a reference is non-null.
        unsafe { iter.get().as_ref() }.map(|node| &node.value)
    }

    /// Removes a key from the hashmap, returning the key-value pair if successful.
    pub fn remove<G>(&self, key: &K, guard: &G) -> Option<Ref<K, V, F>>
    where
        K: Send + Eq + Hash,
        V: Send,
        G: RcuGuard<Flavor = F>,
    {
        let _ = guard;

        // SAFETY: The RCU read-side lock is taken.
        let mut iter = unsafe { self.0.lookup(key) };

        // SAFETY: The node pointer is convertible to a reference is non-null.
        let node = match unsafe { iter.get().as_ref() } {
            None => std::ptr::null_mut(),
            Some(node) => {
                // SAFETY: The RCU read-side lock is taken.
                // SAFETY: The RCU grace period is enforced through RcuRef.
                unsafe { self.0.del(node.into()) }
            }
        };

        NonNull::new(node).map(Ref::new)
    }

    /// Returns an iterator visiting all key-value pairs in arbitrary order.
    pub fn iter<'me, 'guard, G>(&'me self, guard: &'guard G) -> Iter<'guard, K, V, F>
    where
        'me: 'guard,
        G: RcuGuard<Flavor = F>,
    {
        let _ = guard;

        Iter::new(
            // SAFETY: The read-side RCU lock is taken.
            unsafe { self.0.iter() },
        )
    }
}

impl<K, V, F> Drop for RcuHashMap<K, V, F>
where
    K: Send + 'static,
    V: Send + 'static,
    F: RcuFlavor + 'static,
{
    fn drop(&mut self) {
        let mut raw = self.0.clone();

        F::rcu_cleanup_and_block(Box::new(move |context| {
            let guard = context.rcu_read_lock();

            // SAFETY: The read-side RCU lock is taken.
            unsafe { raw.del_all() }
                .iter()
                .copied()
                .map(Ref::<K, V, F>::new)
                .collect::<Vec<_>>()
                .safe_cleanup();

            drop(guard);

            // SAFETY: The read-side RCU lock is not taken.
            // SAFETY: We are a registered RCU read-side thread.
            unsafe { raw.destroy() };
        }));
    }
}
