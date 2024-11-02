use std::marker::PhantomData;
use std::ptr::NonNull;

use crate::collections::hashmap::raw::RawNode;
use crate::rcu::flavor::RcuFlavor;
use crate::RcuRef;

/// An owned RCU reference to a element removed from an [`RcuHashMap`].
///
/// [`RcuHashMap`]: crate::collections::hashmap::container::RcuHashMap
pub struct RefOwned<K, V>(Box<RawNode<K, V>>);

impl<K, V> RefOwned<K, V> {
    /// Returns the key of the entry.
    pub fn key(&self) -> &K {
        &self.0.key
    }

    /// Returns the value of the entry.
    pub fn value(&self) -> &V {
        &self.0.value
    }
}

/// #### Safety
///
/// It is safe to send to another thread if the underlying `K` and `V` are `Send`.
unsafe impl<K: Send, V: Send> Send for RefOwned<K, V> {}

/// #### Safety
///
/// It is safe to have references from multiple threads if the underlying `K` and `V` are `Sync`.
unsafe impl<K: Sync, V: Sync> Sync for RefOwned<K, V> {}

/// An owned RCU reference to a element removed from an [`RcuHashMap`].
///
/// [`RcuHashMap`]: crate::collections::hashmap::container::RcuHashMap
pub struct Ref<K, V, F>
where
    K: Send + 'static,
    V: Send + 'static,
    F: RcuFlavor + 'static,
{
    ptr: *mut RawNode<K, V>,
    _context: PhantomData<*const F>,
}

impl<K, V, F> Ref<K, V, F>
where
    K: Send,
    V: Send,
    F: RcuFlavor,
{
    pub(crate) fn new(ptr: NonNull<RawNode<K, V>>) -> Self {
        Self {
            ptr: ptr.as_ptr(),
            _context: PhantomData,
        }
    }

    pub fn key(&self) -> &K {
        // SAFETY: The pointer is never null.
        &unsafe { self.ptr.as_ref_unchecked() }.key
    }

    pub fn value(&self) -> &V {
        // SAFETY: The pointer is never null.
        &unsafe { self.ptr.as_ref_unchecked() }.value
    }
}

impl<K, V, F> Drop for Ref<K, V, F>
where
    K: Send + 'static,
    V: Send + 'static,
    F: RcuFlavor + 'static,
{
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            Self {
                ptr: self.ptr,
                _context: Default::default(),
            }
            .safe_cleanup();
        }
    }
}

/// #### Safety
///
/// The memory reclamation upon dropping is properly deferred after the RCU grace period.
unsafe impl<K, V, F> RcuRef<F> for Ref<K, V, F>
where
    K: Send,
    V: Send,
    F: RcuFlavor,
{
    type Output = RefOwned<K, V>;

    unsafe fn take_ownership_unchecked(mut self) -> Self::Output {
        let output = RefOwned(Box::from_raw(self.ptr));

        // SAFETY: We don't want deferred cleanup when dropping `self`.
        self.ptr = std::ptr::null_mut();

        output
    }
}

unsafe impl<K, V, F> Send for Ref<K, V, F>
where
    K: Send,
    V: Send,
    F: RcuFlavor,
{
}
