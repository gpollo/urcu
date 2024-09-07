use crate::rcu::callback::RcuCleanupCallback;
use crate::rcu::RcuContext;

/// This trait defines an RCU reference that can be owned after an RCU grace period.
///
/// #### Safety
///
/// You need to ensure that dropping the type does not cause a memory leak. If the ownership
/// of the reference is never taken (e.g. [`RcuRef::take_ownership`] is not called), you need
/// to defer cleanup with [`RcuRef::defer_cleanup`] when [`Drop::drop`] is called.
#[must_use]
pub unsafe trait RcuRef<C> {
    /// The output type after taking ownership.
    type Output;

    /// Take ownership of the reference.
    ///
    /// #### Safety
    ///
    /// You must wait for the grace period before taking ownership.
    unsafe fn take_ownership(self) -> Self::Output;

    /// Configure a cleanup callback to be called after the grace period.
    ///
    /// #### Note
    ///
    /// The reference must implement [`Send`] since the cleanup will be executed in an helper thread.
    fn defer_cleanup(self)
    where
        Self: Sized + Send,
        C: RcuContext,
    {
        C::rcu_call(RcuCleanupCallback::new(self));
    }
}

/// #### Safety
///
/// It is the responsability of the underlying type to be safe.
unsafe impl<T, C> RcuRef<C> for Option<T>
where
    T: RcuRef<C>,
{
    type Output = Option<T::Output>;

    unsafe fn take_ownership(self) -> Self::Output {
        self.map(|r| r.take_ownership())
    }
}
