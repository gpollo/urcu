pub(crate) mod api;
pub(crate) mod callback;
pub(crate) mod cleanup;
pub(crate) mod reference;

use std::cell::Cell;
use std::marker::PhantomData;

use crate::rcu::api::RcuUnsafe;
use crate::rcu::callback::{RcuCall, RcuDefer};
use crate::rcu::cleanup::RcuCleanup;
use crate::rcu::reference::RcuRef;

/// This trait is used to manually poll the RCU grace period.
pub trait RcuPoller {
    /// Checks if the grace period is over for this poller.
    fn grace_period_finished(&self) -> bool;
}

/// This trait defines the per-thread RCU context.
///
/// #### Safety
///
/// 1. You must enforce single context per thread for a specific RCU flavor.
///    Failure to do so can lead to a deadlock if a thread acquires an RCU read lock
///    from one context and tries to do an RCU syncronization from another context.
/// 2. For callbacks (`rcu_call`), a barrier (`rcu_barrier`) should be executed
///    before cleaning up the context. Failure to do so might results in memory
///    leaks and object cleanups that don't happen.
/// 3. For deferred callbacks (`rcu_defer`), a barrier (`defer_barrier`) should be
///    executed before cleaning up the context. Failure to do so might results in
///    memory leaks and object cleanups that don't happen.
pub unsafe trait RcuContext {
    /// Defines an API for unchecked RCU primitives.
    type Unsafe: RcuUnsafe;

    /// Defines a guard for an RCU critical section.
    type Guard<'a>: 'a
    where
        Self: 'a;

    /// Defines a grace period poller;
    type Poller<'a>: RcuPoller + 'a
    where
        Self: 'a;

    /// Register the current thread to RCU.
    ///
    /// #### Note
    ///
    /// This can only be called once per thread.
    fn rcu_register() -> Option<Self>
    where
        Self: Sized;

    /// Starts an RCU critical section.
    ///
    /// #### Note
    ///
    /// RCU critical sections may be nested.
    fn rcu_read_lock(&self) -> Self::Guard<'_>;

    /// Waits until the RCU grace period is over.
    ///
    /// #### Note
    ///
    /// It cannot be called in an RCU critical section.
    fn rcu_synchronize(&mut self);

    /// Creates an RCU grace period poller.
    ///
    /// #### Note
    ///
    /// It may be called in an RCU critical section.
    fn rcu_synchronize_poller(&self) -> Self::Poller<'_>;

    /// Configures a callback to be called after the next RCU grace period is finished.
    ///
    /// #### Note
    ///
    /// The function might internally call [`RcuContext::rcu_synchronize`] and block.
    ///
    /// The callback is guaranteed to be executed on the current thread.
    fn rcu_defer<F>(&mut self, callback: Box<F>)
    where
        F: RcuDefer;

    /// Configures a callback to be called after the next RCU grace period is finished.
    ///
    /// #### Note
    ///
    /// The function will internally call [`RcuContext::rcu_read_lock`].
    ///
    /// The callback must be [`Send`] because it will be executed by an helper thread.
    fn rcu_call<F>(&self, callback: Box<F>)
    where
        F: RcuCall + Send + 'static;

    /// Configures a callback to be called after the next RCU grace period is finished.
    ///
    /// Unlike [`RcuContext::rcu_call`], this function can be called by any thread whether
    /// it is registered or not.
    ///
    /// #### Note
    ///
    /// The callback must be [`Send`] because it will be executed by an helper thread.
    fn rcu_cleanup(callback: RcuCleanup<Self>);
}

macro_rules! define_rcu_take_ownership {
    ($name:ident,$x:literal) => {
        pub fn $name<T1, C>(
            context: &mut C,
            r1: T1,
        ) -> T1::Output
        where
            T1: RcuRef<C>,
            C: RcuContext,
        {
            context.rcu_synchronize();

            // SAFETY: RCU grace period has ended.
            unsafe { T1::take_ownership(r1) }
        }
    };

    ($name:ident,$($x:literal),*) => {
        paste::paste!{
            pub fn $name<$([<T $x>]),*, C>(
                context: &mut C,
                $([<r $x>]: [<T $x>]),*,
            ) -> ($([<T $x>]::Output),*,)
            where
                $([<T $x>]: RcuRef<C>),*,
                C: RcuContext,
            {
                context.rcu_synchronize();

                // SAFETY: RCU grace period has ended.
                unsafe { ($([<T $x>]::take_ownership([<r $x>])),*,) }
            }
        }
    };
}

define_rcu_take_ownership!(rcu_take_ownership_1, 1);
define_rcu_take_ownership!(rcu_take_ownership_2, 1, 2);
define_rcu_take_ownership!(rcu_take_ownership_3, 1, 2, 3);
define_rcu_take_ownership!(rcu_take_ownership_4, 1, 2, 3, 4);
define_rcu_take_ownership!(rcu_take_ownership_5, 1, 2, 3, 4, 5);

/// Takes ownership of multiple [RcuRef] values.
///
/// This macro will wait for the RCU grace period before taking ownership.
#[macro_export]
macro_rules! rcu_take_ownership {
    ($c:expr, $r1:ident) => {
        urcu::rcu_take_ownership_1($c, $r1)
    };
    ($c:expr, $r1:ident, $r2:ident) => {
        urcu::rcu_take_ownership_2($c, $r1, $r2)
    };
    ($c:expr, $r1:ident, $r2:ident, $r3:ident) => {
        urcu::rcu_take_ownership_3($c, $r1, $r2, $r3)
    };
    ($c:expr, $r1:ident, $r2:ident, $r3:ident, $r4:ident) => {
        urcu::rcu_take_ownership_4($c, $r1, $r2, $r3, $r4)
    };
    ($c:expr, $r1:ident, $r2:ident, $r3:ident, $r4:ident, $r5:ident) => {
        urcu::rcu_take_ownership_5($c, $r1, $r2, $r3, $r4, $r5)
    };
}

macro_rules! define_rcu_guard {
    ($flavor:ident, $guard:ident, $unsafe:ident, $context:ident) => {
        #[doc = concat!("Defines a guard for an RCU critical section (`liburcu-", stringify!($flavor), "`).")]
        #[allow(dead_code)]
        pub struct $guard<'a>(PhantomData<&'a $context>);

        impl<'a> $guard<'a> {
            fn new(_context: &'a $context) -> Self {
                // SAFETY: The thread is initialized at context's creation.
                // SAFETY: The thread is read-registered at context's creation.
                // SAFETY: The critical section is unlocked at guard's drop.
                unsafe { $unsafe::unchecked_rcu_read_lock() };

                Self(PhantomData)
            }
        }

        impl<'a> Drop for $guard<'a> {
            fn drop(&mut self) {
                // SAFETY: The thread is initialized at context's creation.
                // SAFETY: The thread is read-registered at context's creation.
                // SAFETY: The critical section is locked at guard's creation.
                unsafe { $unsafe::unchecked_rcu_read_unlock() };
            }
        }
    };
}

macro_rules! define_rcu_poller {
    ($flavor:ident, $poller:ident, $unsafe:ident, $context:ident) => {
        #[doc = concat!("Defines a grace period poller (`liburcu-", stringify!($flavor), "`).")]
        #[allow(dead_code)]
        pub struct $poller<'a>(PhantomData<&'a $context>, urcu_sys::RcuPollState);

        impl<'a> $poller<'a> {
            fn new(_context: &'a $context) -> Self {
                Self(PhantomData, {
                    // SAFETY: The thread is initialized at context's creation.
                    // SAFETY: The thread is read-registered at context's creation.
                    unsafe { $unsafe::unchecked_rcu_poll_start() }
                })
            }
        }

        impl<'a> RcuPoller for $poller<'a> {
            fn grace_period_finished(&self) -> bool {
                // SAFETY: The thread is initialized at context's creation.
                // SAFETY: The thread is read-registered at context's creation.
                // SAFETY: The handle is created at poller's creation.
                unsafe { $unsafe::unchecked_rcu_poll_check(self.1) }
            }
        }
    };
}

macro_rules! define_rcu_context {
    ($flavor:ident, $context:ident, $unsafe:ident, $guard:ident, $poller:ident) => {
        #[doc = concat!("Defines an RCU context for the current thread (`liburcu-", stringify!($flavor), "`).")]
        ///
        /// #### Note
        ///
        /// There can only be 1 instance per thread.
        /// The thread will be registered upon creation.
        /// It will be unregistered upon dropping.
        pub struct $context(
            // Prevent Send+Send auto trait implementations.
            PhantomData<*const ()>,
        );

        impl $context {
            /// Creates the context instance.
            ///
            /// Only the first call will return a context.
            /// Subsequent calls on the same thread will return nothing.
            fn new() -> Option<Self> {
                thread_local! {static RCU_CONTEXT: Cell<bool> = Cell::new(false)};

                RCU_CONTEXT.with(|initialized| {
                    if initialized.replace(true) {
                        return None;
                    }

                    log::info!(
                        "registering thread '{}' ({}) with RCU (liburcu-{})",
                        std::thread::current().name().unwrap_or("<unnamed>"),
                        unsafe { libc::gettid() },
                        stringify!($flavor),
                    );

                    // SAFETY: Can only be called once per thread.
                    // SAFETY: It is the first RCU call for a thread.
                    unsafe { $unsafe::unchecked_rcu_init() };

                    // SAFETY: The thread is initialized.
                    // SAFETY: The thread is not read-registered.
                    // SAFETY: The thread is read-unregistered at context's drop.
                    unsafe { $unsafe::unchecked_rcu_read_register_thread() };

                    // SAFETY: The thread is initialized.
                    // SAFETY: The thread is not defer-registered.
                    // SAFETY: The thread is read-unregistered at context's drop.
                    unsafe { $unsafe::unchecked_rcu_defer_register_thread() };

                    Some(Self(PhantomData))
                })
            }
        }

        impl Drop for $context {
            fn drop(&mut self) {
                log::info!(
                    "unregistering thread '{}' ({}) with RCU (liburcu-{})",
                    std::thread::current().name().unwrap_or("<unnamed>"),
                    unsafe { libc::gettid() },
                    stringify!($flavor),
                );

                Self::cleanup_remove();

                // SAFETY: The thread is initialized at context's creation.
                // SAFETY: The thread is defer-registered at context's creation.
                // SAFETY: The thread can't be in a RCU critical section if it's dropping.
                unsafe { $unsafe::unchecked_rcu_defer_barrier() };

                // SAFETY: The thread is initialized at context's creation.
                // SAFETY: The thread is defer-registered at context's creation.
                unsafe { $unsafe::unchecked_rcu_defer_unregister_thread() };

                // SAFETY: The thread is initialized at context's creation.
                // SAFETY: The thread is read-registered at context's creation.
                unsafe { $unsafe::unchecked_rcu_call_barrier() };

                // SAFETY: The thread is initialized at context's creation.
                // SAFETY: The thread is read-registered at context's creation.
                unsafe { $unsafe::unchecked_rcu_read_unregister_thread() };
            }
        }

        /// #### Safety
        ///
        /// 1. There can only be 1 instance per thread.
        /// 2. `call_rcu` barrier is called before cleanups.
        /// 3. `defer_rcu` barrier is called before cleanups.
        unsafe impl RcuContext for $context {
            type Unsafe = $unsafe;

            type Guard<'a> = $guard<'a>;

            type Poller<'a> = $poller<'a>;

            fn rcu_register() -> Option<Self>
            where
                Self: Sized,
            {
                Self::new()
            }

            fn rcu_read_lock(&self) -> Self::Guard<'_> {
                $guard::new(self)
            }

            fn rcu_synchronize(&mut self) {
                // SAFETY: The thread is initialized at context's creation.
                // SAFETY: The thread cannot be in a critical section because of `&mut self`.
                unsafe { $unsafe::unchecked_rcu_synchronize() };
            }

            fn rcu_synchronize_poller(&self) -> Self::Poller<'_> {
                $poller::new(self)
            }

            fn rcu_defer<F>(&mut self, callback: Box<F>)
            where
                F: RcuDefer,
            {
                callback.configure(|mut ptr, func| {
                    // SAFETY: The thread is initialized at context's creation.
                    // SAFETY: The thread is defer-registered at context's creation.
                    // SAFETY: The thread executes a defer-barrier at context's drop.
                    // SAFETY: The thread cannot be in a critical section because of `&mut self`.
                    // SAFETY: The pointers validity is guaranteed by `RcuDefer`.
                    unsafe { $unsafe::unchecked_rcu_defer_call(Some(func), ptr.as_mut()) };
                });
            }

            fn rcu_call<F>(&self, callback: Box<F>)
            where
                F: RcuCall + Send + 'static,
            {
                callback.configure(|mut head, func| {
                    // SAFETY: The thread is initialized at context's creation.
                    // SAFETY: The thread is read-registered at context's creation.
                    // SAFETY: The thread executes a call-barrier at context's drop.
                    // SAFETY: The pointers validity is guaranteed by `RcuCall`.
                    unsafe { $unsafe::unchecked_rcu_call(Some(func), head.as_mut()) };
                });
            }

            fn rcu_cleanup(callback: RcuCleanup<Self>) {
                Self::cleanup_send(callback);
            }
        }
    };
}

pub mod flavor {
    use super::*;

    #[cfg(feature = "flavor-bp")]
    pub(crate) mod bp {
        use super::*;

        pub use crate::rcu::api::RcuUnsafeBp;

        define_rcu_guard!(bp, RcuGuardBp, RcuUnsafeBp, RcuContextBp);
        define_rcu_poller!(bp, RcuPollerBp, RcuUnsafeBp, RcuContextBp);
        define_rcu_context!(bp, RcuContextBp, RcuUnsafeBp, RcuGuardBp, RcuPollerBp);
    }

    #[cfg(feature = "flavor-mb")]
    pub(crate) mod mb {
        use super::*;

        pub use crate::rcu::api::RcuUnsafeMb;

        define_rcu_guard!(mb, RcuGuardMb, RcuUnsafeMb, RcuContextMb);
        define_rcu_poller!(mb, RcuPollerMb, RcuUnsafeMb, RcuContextMb);
        define_rcu_context!(mb, RcuContextMb, RcuUnsafeMb, RcuGuardMb, RcuPollerMb);
    }

    #[cfg(feature = "flavor-memb")]
    pub(crate) mod memb {
        use super::*;

        pub use crate::rcu::api::RcuUnsafeMemb;

        define_rcu_guard!(memb, RcuGuardMemb, RcuUnsafeMemb, RcuContextMemb);
        define_rcu_poller!(memb, RcuPollerMemb, RcuUnsafeMemb, RcuContextMemb);
        define_rcu_context!(
            memb,
            RcuContextMemb,
            RcuUnsafeMemb,
            RcuGuardMemb,
            RcuPollerMemb
        );
    }

    #[cfg(feature = "flavor-qsbr")]
    pub(crate) mod qsbr {
        use super::*;

        pub use crate::rcu::api::RcuUnsafeQsbr;

        define_rcu_guard!(qsbr, RcuGuardQsbr, RcuUnsafeQsbr, RcuContextQsbr);
        define_rcu_poller!(qsbr, RcuPollerQsbr, RcuUnsafeQsbr, RcuContextQsbr);
        define_rcu_context!(
            qsbr,
            RcuContextQsbr,
            RcuUnsafeQsbr,
            RcuGuardQsbr,
            RcuPollerQsbr
        );
    }

    #[cfg(feature = "flavor-bp")]
    pub use bp::*;

    #[cfg(feature = "flavor-mb")]
    pub use mb::*;

    #[cfg(feature = "flavor-memb")]
    pub use memb::*;

    #[cfg(feature = "flavor-qsbr")]
    pub use qsbr::*;
}

#[cfg(feature = "flavor-memb")]
pub type DefaultContext = flavor::memb::RcuContextMemb;

#[cfg(all(not(feature = "flavor-memb"), feature = "flavor-mb"))]
pub type DefaultContext = flavor::mb::RcuContextMb;

#[cfg(all(
    not(feature = "flavor-memb"),
    not(feature = "flavor-mb"),
    feature = "flavor-bp"
))]
pub type DefaultContext = flavor::bp::RcuContextBp;

#[cfg(all(
    not(feature = "flavor-memb"),
    not(feature = "flavor-mb"),
    not(feature = "flavor-bp"),
    feature = "flavor-qsbr"
))]
pub type DefaultContext = flavor::qsbr::RcuContextQsbr;

mod asserts {
    use static_assertions::assert_not_impl_all;

    mod bp {
        use super::*;

        use crate::rcu::flavor::bp::*;

        assert_not_impl_all!(RcuPollerBp: Send);
        assert_not_impl_all!(RcuPollerBp: Sync);

        assert_not_impl_all!(RcuGuardBp: Send);
        assert_not_impl_all!(RcuGuardBp: Sync);

        assert_not_impl_all!(RcuContextBp: Send);
        assert_not_impl_all!(RcuContextBp: Sync);
    }

    mod mb {
        use super::*;

        use crate::rcu::flavor::mb::*;

        assert_not_impl_all!(RcuPollerMb: Send);
        assert_not_impl_all!(RcuPollerMb: Sync);

        assert_not_impl_all!(RcuGuardMb: Send);
        assert_not_impl_all!(RcuGuardMb: Sync);

        assert_not_impl_all!(RcuContextMb: Send);
        assert_not_impl_all!(RcuContextMb: Sync);
    }

    mod memb {
        use super::*;

        use crate::rcu::flavor::memb::*;

        assert_not_impl_all!(RcuPollerMemb: Send);
        assert_not_impl_all!(RcuPollerMemb: Sync);

        assert_not_impl_all!(RcuGuardMemb: Send);
        assert_not_impl_all!(RcuGuardMemb: Sync);

        assert_not_impl_all!(RcuContextMemb: Send);
        assert_not_impl_all!(RcuContextMemb: Sync);
    }

    mod qsbr {
        use super::*;

        use crate::rcu::flavor::qsbr::*;

        assert_not_impl_all!(RcuPollerQsbr: Send);
        assert_not_impl_all!(RcuPollerQsbr: Sync);

        assert_not_impl_all!(RcuGuardQsbr: Send);
        assert_not_impl_all!(RcuGuardQsbr: Sync);

        assert_not_impl_all!(RcuContextQsbr: Send);
        assert_not_impl_all!(RcuContextQsbr: Sync);
    }
}
