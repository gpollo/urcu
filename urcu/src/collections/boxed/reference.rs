use std::marker::PhantomData;
use std::ops::Deref;

use crate::rcu::flavor::RcuFlavor;
use crate::rcu::reference::RcuRef;

/// A RCU reference to a element removed from a [`RcuBox`].
///
/// #### Note
///
/// To get ownership of the reference, you can use [`RcuRef::take_ownership`]. If ownership
/// is never taken, cleanup will be executed in a RCU cleanup thread.
///
/// #### Requirements
///
/// `T` must be [`Send`] because [`Drop::drop`] might execute cleanup in another thread.
///
/// [`RcuBox`]: crate::collections::boxed::container::RcuBox
pub struct Ref<T, F>
where
    T: Send + 'static,
    F: RcuFlavor + 'static,
{
    ptr: *mut T,
    context: PhantomData<F>,
}

impl<T, F> Ref<T, F>
where
    T: Send,
    F: RcuFlavor,
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
unsafe impl<T, F> RcuRef<F> for Ref<T, F>
where
    T: Send,
    F: RcuFlavor,
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
unsafe impl<T, F> Send for Ref<T, F>
where
    T: Send,
    F: RcuFlavor,
{
}

impl<T, F> Drop for Ref<T, F>
where
    T: Send + 'static,
    F: RcuFlavor + 'static,
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

impl<T, F> Deref for Ref<T, F>
where
    T: Send,
    F: RcuFlavor,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}
