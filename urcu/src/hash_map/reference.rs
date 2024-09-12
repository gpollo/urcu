use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use crate::hash_map::raw::RawNode;
use crate::{RcuContext, RcuRef};

pub struct RcuHashMapRefOwned<K, V>(Box<RawNode<K, V>>);

impl<K, V> Deref for RcuHashMapRefOwned<K, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<K, V> DerefMut for RcuHashMapRefOwned<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

pub struct RcuHashMapRef<K, V, C>
where
    K: Send,
    V: Send,
    C: RcuContext,
{
    ptr: *mut RawNode<K, V>,
    _context: PhantomData<*const C>,
}

impl<K, V, C> RcuHashMapRef<K, V, C>
where
    K: Send,
    V: Send,
    C: RcuContext,
{
    pub(crate) fn new(ptr: *mut RawNode<K, V>) -> Self {
        Self {
            ptr,
            _context: PhantomData,
        }
    }
}

impl<K, V, C> Drop for RcuHashMapRef<K, V, C>
where
    K: Send,
    V: Send,
    C: RcuContext,
{
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            Self {
                ptr: self.ptr,
                _context: Default::default(),
            }
            .call_cleanup();
        }
    }
}

/// #### Safety
///
/// The memory reclamation upon dropping is properly deferred after the RCU grace period.
unsafe impl<K, V, C> RcuRef<C> for RcuHashMapRef<K, V, C>
where
    K: Send,
    V: Send,
    C: RcuContext,
{
    type Output = RcuHashMapRefOwned<K, V>;

    unsafe fn take_ownership(mut self) -> Self::Output {
        let output = RcuHashMapRefOwned(Box::from_raw(self.ptr));

        // SAFETY: We don't want deferred cleanup when dropping `self`.
        self.ptr = std::ptr::null_mut();

        output
    }
}

impl<K, V, C> Deref for RcuHashMapRef<K, V, C>
where
    K: Send,
    V: Send,
    C: RcuContext,
{
    type Target = V;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(&*self.ptr) }
    }
}

unsafe impl<K, V, C> Send for RcuHashMapRef<K, V, C>
where
    K: Send,
    V: Send,
    C: RcuContext,
{
}
