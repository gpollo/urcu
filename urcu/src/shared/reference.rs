use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;

use crate::rcu::reference::RcuRef;
use crate::{utility::*, RcuContext};

/// An owned RCU reference to a element removed from a container.
pub struct BoxRefOwned<T>(Box<T>);

impl<T> Deref for BoxRefOwned<T>
where
    T: Deref,
{
    type Target = T::Target;

    fn deref(&self) -> &Self::Target {
        self.0.deref().deref()
    }
}

/// #### Safety
///
/// It is safe to send to another thread if the underlying `T` is `Send`.
unsafe impl<T: Send> Send for BoxRefOwned<T> {}

/// #### Safety
///
/// It is safe to have references from multiple threads if the underlying `T` is `Sync`.
unsafe impl<T: Sync> Sync for BoxRefOwned<T> {}

/// A RCU reference to a element removed from a container.
pub struct BoxRef<T, C>
where
    T: Send + 'static,
    C: RcuContext + 'static,
{
    ptr: *mut T,
    _unsend: PhantomUnsend<(T, C)>,
    _unsync: PhantomUnsync<(T, C)>,
}

impl<T, C> BoxRef<T, C>
where
    T: Send,
    C: RcuContext,
{
    pub(crate) fn new(ptr: NonNull<T>) -> Self {
        Self {
            ptr: ptr.as_ptr(),
            _unsend: PhantomData,
            _unsync: PhantomData,
        }
    }
}

/// #### Safety
///
/// * The underlying reference is cleaned up upon dropping.
/// * There may be immutable borrows to the underlying reference.
/// * There cannot be mutable borrows to the underlying reference.
unsafe impl<T, C> RcuRef<C> for BoxRef<T, C>
where
    T: Send,
    C: RcuContext,
{
    type Output = BoxRefOwned<T>;

    unsafe fn take_ownership_unchecked(mut self) -> Self::Output {
        // SAFETY: There are no readers after the RCU grace period.
        let output = BoxRefOwned(Box::from_raw(self.ptr));

        // SAFETY: We don't want to cleanup when dropping `self`.
        self.ptr = std::ptr::null_mut();

        output
    }
}

/// #### Safety
///
/// An RCU reference can be sent to another thread if `T` implements [`Send`].
unsafe impl<T, C> Send for BoxRef<T, C>
where
    T: Send,
    C: RcuContext,
{
}

impl<T, C> Drop for BoxRef<T, C>
where
    T: Send + 'static,
    C: RcuContext + 'static,
{
    fn drop(&mut self) {
        if let Some(ptr) = NonNull::new(self.ptr) {
            Self::new(ptr).safe_cleanup();
        }
    }
}

impl<T, C> Deref for BoxRef<T, C>
where
    T: Send + Deref,
    C: RcuContext,
{
    type Target = T::Target;

    fn deref(&self) -> &Self::Target {
        // SAFETY: The pointer is only null when dropping.
        unsafe { self.ptr.as_ref_unchecked().deref() }
    }
}
