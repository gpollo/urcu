use std::marker::PhantomData;
use std::ops::Deref;

use crate::rcu::reference::RcuRef;
use crate::rcu::RcuContext;

/// A RCU reference to a element removed from a [`RcuBox`].
///
/// #### Note
///
/// To get ownership of the reference, you can use [`rcu_take_ownership`]. If ownership is
/// never taken, cleanup will be executed in a RCU cleanup thread.
///
/// #### Requirements
///
/// `T` must be [`Send`] because [`Drop::drop`] might execute cleanup in another thread.
///
/// [`rcu_take_ownership`]: crate::rcu_take_ownership
/// [`RcuBox`]: crate::boxed::container::RcuBox
pub struct Ref<T, C>
where
    T: Send + 'static,
    C: RcuContext + 'static,
{
    ptr: *mut T,
    context: PhantomData<C>,
}

impl<T, C> Ref<T, C>
where
    T: Send,
    C: RcuContext,
{
    pub fn new(ptr: *mut T) -> Self {
        Self {
            ptr,
            context: PhantomData,
        }
    }
}

/// #### Safety
///
/// * The reference is cleaned up upon dropping.
/// * The reference does not expose mutable borrows.
unsafe impl<T, C> RcuRef<C> for Ref<T, C>
where
    T: Send,
    C: RcuContext,
{
    type Output = Box<T>;

    unsafe fn take_ownership_unchecked(mut self) -> Self::Output {
        let output = Box::from_raw(self.ptr);

        // SAFETY: We don't want to cleanup when dropping `self`.
        self.ptr = std::ptr::null_mut();

        output
    }
}

/// #### Safety
///
/// An RCU reference can be sent to another thread if `T` implements [`Send`].
unsafe impl<T, C> Send for Ref<T, C>
where
    T: Send,
    C: RcuContext,
{
}

impl<T, C> Drop for Ref<T, C>
where
    T: Send + 'static,
    C: RcuContext + 'static,
{
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            Self {
                ptr: self.ptr,
                context: PhantomData,
            }
            .safe_cleanup();
        }
    }
}

impl<T, C> Deref for Ref<T, C>
where
    T: Send,
    C: RcuContext,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}
