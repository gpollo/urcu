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
    /// The function might internally call [`RcuContext::rcu_synchronize`] and block.
    ///
    /// The callback is guaranteed to be executed on the current thread.
    fn defer_cleanup(self, context: &mut C)
    where
        Self: Sized,
        C: RcuContext,
    {
        context.rcu_defer(RcuCleanupCallback::new(self))
    }

    /// Configure a cleanup callback to be called after the grace period.
    ///
    /// #### Note
    ///
    /// The reference must implement [`Send`] since the cleanup will be executed in an helper thread.
    fn call_cleanup(self)
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

/// #### Safety
///
/// It is the responsability of the underlying type to be safe.
unsafe impl<T, C> RcuRef<C> for Vec<T>
where
    T: RcuRef<C>,
{
    type Output = Vec<T::Output>;

    unsafe fn take_ownership(self) -> Self::Output {
        self.into_iter().map(|r| r.take_ownership()).collect()
    }
}
