use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use crate::linked_list::raw::Node;
use crate::{RcuContext, RcuRef};

/// An owned RCU reference to a element removed from an [`RcuList`].
///
/// [`RcuList`]: crate::linked_list::RcuList
pub struct RefOwned<T>(Box<Node<T>>);

impl<T> Deref for RefOwned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T> DerefMut for RefOwned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

/// An RCU reference to a element removed from an [`RcuList`].
///
/// #### Note
///
/// To get ownership of the reference, you can use [`rcu_take_ownership`]. If ownership is
/// never taken, cleanup will be automatically executed after the next RCU grace period.
///
/// #### Requirements
///
/// `T` must be [`Send`] because [`Drop::drop`] might defer cleanup in another thread.
///
/// [`rcu_take_ownership`]: crate::rcu_take_ownership
/// [`RcuList`]: crate::linked_list::RcuList
#[must_use]
pub struct Ref<T, C>
where
    T: Send + 'static,
    C: RcuContext + 'static,
{
    ptr: *mut Node<T>,
    context: PhantomData<C>,
}

impl<T, C> Ref<T, C>
where
    T: Send,
    C: RcuContext,
{
    pub(crate) fn new(ptr: *mut Node<T>) -> Self {
        Self {
            ptr,
            context: PhantomData,
        }
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
                context: Default::default(),
            }
            .safe_cleanup();
        }
    }
}

/// #### Safety
///
/// The memory reclamation upon dropping is properly deferred after the RCU grace period.
unsafe impl<T, C> RcuRef<C> for Ref<T, C>
where
    T: Send,
    C: RcuContext,
{
    type Output = RefOwned<T>;

    unsafe fn take_ownership(mut self) -> Self::Output {
        let output = RefOwned(Box::from_raw(self.ptr));

        // SAFETY: We don't want deferred cleanup when dropping `self`.
        self.ptr = std::ptr::null_mut();

        output
    }
}

impl<T, C> Deref for Ref<T, C>
where
    T: Send,
    C: RcuContext,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { (*self.ptr).deref() }
    }
}
