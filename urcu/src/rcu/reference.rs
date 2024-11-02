use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;

use crate::rcu::callback::{RcuCallFn, RcuDeferFn};
use crate::rcu::context::{RcuContext, RcuDeferContext, RcuReadContext};
use crate::rcu::flavor::RcuFlavor;
use crate::utility::*;

/// This trait defines a RCU reference that can be owned after a RCU grace period.
///
/// #### Safety
///
/// * The underlying reference must be cleaned up upon dropping (see below).
/// * There may be immutable borrows to the underlying reference.
/// * There must not be any mutable borrows to the underlying reference.
///
/// #### Dropping
///
/// An [`RcuRef`] should always cleanup when [`Drop::drop`] is executed by taking
/// ownership and dropping the underlying value.
///
/// * We cannot call [`RcuContext::rcu_synchronize`] since we can't be sure that
///   a RCU read lock is currently held or not[^mborrow].
///
/// Because an [`RcuRef`] can be sent to any thread, we cannot guarantee that a
/// thread executing [`Drop::drop`] is properly registered.
///
/// * We cannot call [`RcuDeferContext::rcu_defer`] since we can't enforce that the
///   thread is registered with the RCU defer mecanisms[^mborrow].
/// * We cannot call [`RcuReadContext::rcu_call`] since we can't enforce that the
///   thread is registered with the RCU read mecanisms[^cborrow].
///
/// The only way to keep the safety guarantees of this crate is to use the custom
/// cleanup thread through [`RcuRef::safe_cleanup`]. It is similar to the built-in
/// [`RcuReadContext::rcu_call`], except it doesn't expect the calling thread to be
/// registered with RCU in any way.
///
/// The downside is that it is most likely worst than [`RcuReadContext::rcu_call`] in
/// every way. If it is a performance problem, the owner of an [`RcuRef`] can alway
/// use [`RcuRef::defer_cleanup`] and [`RcuRef::call_cleanup`] before [`Drop::drop`]
/// is called.
///
/// [^mborrow]: Unless your [`RcuRef`] has a mutable borrow of an [`RcuContext`].
/// [^cborrow]: Unless your [`RcuRef`] has an immutable borrow of an [`RcuContext`].
#[must_use]
pub unsafe trait RcuRef<F> {
    /// The output type after taking ownership.
    type Output;

    /// Take ownership of the reference.
    ///
    /// #### Safety
    ///
    /// You must wait for the grace period before taking ownership.
    unsafe fn take_ownership_unchecked(self) -> Self::Output;

    /// Take ownership of the reference.
    fn take_ownership<C>(self, context: &mut C) -> Self::Output
    where
        Self: Sized,
        C: RcuContext<Flavor = F>,
    {
        context.rcu_synchronize();

        // SAFETY: RCU grace period has ended.
        unsafe { self.take_ownership_unchecked() }
    }

    /// Configure a cleanup callback to be called after the grace period.
    ///
    /// #### Note
    ///
    /// The function might internally call [`RcuContext::rcu_synchronize`] and block.
    ///
    /// The callback is guaranteed to be executed on the current thread.
    fn defer_cleanup<C>(self, context: &mut C)
    where
        Self: Sized,
        C: RcuDeferContext<Flavor = F>,
    {
        context.rcu_defer(RcuDeferFn::<_, F>::new(move || {
            // SAFETY: The caller already executed a RCU syncronization.
            unsafe {
                self.take_ownership_unchecked();
            }
        }))
    }

    /// Configure a cleanup callback to be called after the grace period.
    ///
    /// #### Note
    ///
    /// The function will internally call [`RcuReadContext::rcu_read_lock`].
    ///
    /// The reference must implement [`Send`] since the cleanup will be executed in an helper thread.
    fn call_cleanup<C>(self, context: &C)
    where
        Self: Sized + Send + 'static,
        C: RcuReadContext<Flavor = F> + 'static,
    {
        context.rcu_call(RcuCallFn::new(move || {
            // SAFETY: The caller already executed a RCU syncronization.
            unsafe {
                self.take_ownership_unchecked();
            }
        }));
    }

    fn safe_cleanup(self)
    where
        Self: Sized + Send + 'static,
        F: RcuFlavor,
    {
        F::rcu_cleanup(Box::new(move |context| {
            context.rcu_synchronize();

            // SAFETY: An RCU syncronization barrier was called.
            unsafe {
                self.take_ownership_unchecked();
            }
        }));
    }
}

/// #### Safety
///
/// It is the responsability of the underlying type to be safe.
unsafe impl<T, F> RcuRef<F> for Option<T>
where
    T: RcuRef<F>,
{
    type Output = Option<T::Output>;

    unsafe fn take_ownership_unchecked(self) -> Self::Output {
        self.map(|r| r.take_ownership_unchecked())
    }
}

/// #### Safety
///
/// It is the responsability of the underlying type to be safe.
unsafe impl<T, F> RcuRef<F> for Vec<T>
where
    T: RcuRef<F>,
{
    type Output = Vec<T::Output>;

    unsafe fn take_ownership_unchecked(self) -> Self::Output {
        self.into_iter()
            .map(|r| r.take_ownership_unchecked())
            .collect()
    }
}

macro_rules! impl_rcu_ref_for_tuple {
    ($($x:literal),*) => {
        paste::paste!{
            /// #### Safety
            ///
            /// It is the responsability of the underlying types to be safe.
            unsafe impl<$([<T $x>]),*, F> RcuRef<F> for ($([<T $x>]),*)
            where
                $([<T $x>]: RcuRef<F>),*,
            {
                type Output = ($([<T $x>]::Output),*,);

                unsafe fn take_ownership_unchecked(self) -> Self::Output {
                    (
                        $(self.$x.take_ownership_unchecked()),*,
                    )
                }
            }
        }
    };
}

impl_rcu_ref_for_tuple!(0, 1);
impl_rcu_ref_for_tuple!(0, 1, 2);
impl_rcu_ref_for_tuple!(0, 1, 2, 3);
impl_rcu_ref_for_tuple!(0, 1, 2, 3, 4);
impl_rcu_ref_for_tuple!(0, 1, 2, 3, 4, 5);
impl_rcu_ref_for_tuple!(0, 1, 2, 3, 4, 5, 6);

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
pub struct RcuBoxRef<T, F>
where
    T: Send + 'static,
    F: RcuFlavor + 'static,
{
    ptr: *mut T,
    _unsend: PhantomUnsend<(T, F)>,
    _unsync: PhantomUnsync<(T, F)>,
}

impl<T, F> RcuBoxRef<T, F>
where
    T: Send,
    F: RcuFlavor,
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
unsafe impl<T, F> RcuRef<F> for RcuBoxRef<T, F>
where
    T: Send,
    F: RcuFlavor,
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
unsafe impl<T, F> Send for RcuBoxRef<T, F>
where
    T: Send,
    F: RcuFlavor,
{
}

impl<T, F> Drop for RcuBoxRef<T, F>
where
    T: Send + 'static,
    F: RcuFlavor + 'static,
{
    fn drop(&mut self) {
        if let Some(ptr) = NonNull::new(self.ptr) {
            Self::new(ptr).safe_cleanup();
        }
    }
}

impl<T, F> Deref for RcuBoxRef<T, F>
where
    T: Send + Deref,
    F: RcuFlavor,
{
    type Target = T::Target;

    fn deref(&self) -> &Self::Target {
        // SAFETY: The pointer is only null when dropping.
        unsafe { self.ptr.as_ref_unchecked().deref() }
    }
}

mod asserts {
    use super::*;

    use static_assertions::{assert_impl_all, assert_not_impl_all};

    use crate::rcu::default::DefaultFlavor;
    use crate::utility::asserts::*;

    mod rcu_ref {
        use super::*;

        // T: Send + !Sync
        assert_impl_all!(RcuBoxRef<SendButNotSync, DefaultFlavor>: Send);
        assert_not_impl_all!(RcuBoxRef<SendButNotSync, DefaultFlavor>: Sync);

        // T: Send + Sync
        assert_impl_all!(RcuBoxRef<SendAndSync, DefaultFlavor>: Send);
        assert_not_impl_all!(RcuBoxRef<SendAndSync, DefaultFlavor>: Sync);
    }

    mod rcu_ref_owned {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(BoxRefOwned<NotSendNotSync>: Send);
        assert_not_impl_all!(BoxRefOwned<NotSendNotSync>: Sync);

        // T: Send + !Sync
        assert_impl_all!(BoxRefOwned<SendButNotSync>: Send);
        assert_not_impl_all!(BoxRefOwned<SendButNotSync>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(BoxRefOwned<NotSendButSync>: Send);
        assert_impl_all!(BoxRefOwned<NotSendButSync>: Sync);

        // T: Send + Sync
        assert_impl_all!(BoxRefOwned<SendAndSync>: Send);
        assert_impl_all!(BoxRefOwned<SendAndSync>: Sync);
    }
}
