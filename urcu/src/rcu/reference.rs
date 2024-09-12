use crate::rcu::callback::RcuCleanupCallback;
use crate::rcu::context::{RcuDeferrer, RcuReader, RcuThread};
use crate::rcu::flavor::RcuFlavor;

/// This trait defines an RCU reference that can be owned after an RCU grace period.
///
/// #### Safety
///
/// You need to ensure that dropping the type does not cause a memory leak. If the ownership
/// of the reference is never taken (e.g. [`RcuRef::take_ownership`] is not called), you need
/// to defer cleanup with [`RcuRef::defer_cleanup`] when [`Drop::drop`] is called.
#[must_use]
pub unsafe trait RcuRef<C>
where
    C: RcuThread,
{
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
    /// The function might internally execute an RCU syncronization and block.
    ///
    /// The callback is guaranteed to be executed on the current thread.
    fn defer_cleanup(self, context: &mut C)
    where
        Self: Sized,
        C: RcuDeferrer,
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
        C: RcuReader,
    {
        C::rcu_call(RcuCleanupCallback::new(self));
    }
}

/// #### Safety
///
/// It is the responsability of the underlying type to be safe.
unsafe impl<T, F> RcuRef<F> for Option<T>
where
    T: RcuRef<F>,
    F: RcuFlavor,
{
    type Output = Option<T::Output>;

    unsafe fn take_ownership(self) -> Self::Output {
        self.map(|r| r.take_ownership())
    }
}
