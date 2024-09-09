pub(crate) mod callback;
pub(crate) mod reference;

use std::cell::Cell;
use std::marker::PhantomData;

use urcu_sys::RcuFlavor;

use crate::rcu::callback::RcuCallback;
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
/// 2. For callbacks (e.g. `rcu_call`), a barrier (e.g. `rcu_barrier`) should be
///    executed before cleaning up the context. Failure to do so might results in
///    memory leaks and object cleanups that don't happen.
pub unsafe trait RcuContext {
    /// Defines a guard for an RCU critical section.
    type Guard<'a>: 'a
    where
        Self: 'a;

    /// Defines a grace period poller;
    type Poller<'a>: RcuPoller + 'a
    where
        Self: 'a;

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
    /// It cannot be called in an RCU critical section.
    fn rcu_synchronize_poller(&mut self) -> Self::Poller<'_>;

    /// Configures a callback to be called after the next RCU grace period is finished.
    ///
    /// #### Note
    ///
    /// The callback must be [`Send`] because it will be executed by a helper thread.
    fn rcu_call<F>(callback: Box<F>)
    where
        F: RcuCallback + Send;

    /// Returns the API list for this RCU flavor.
    fn rcu_api() -> &'static RcuFlavor;
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
                unsafe {
                    ($([<T $x>]::take_ownership([<r $x>])),*,)
                }
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

macro_rules! urcu_func {
    ($flavor:ident, $name:ident) => {
        paste::paste! {
            [<urcu _ $flavor _ $name>]
        }
    };
}

macro_rules! define_rcu_guard {
    ($flavor:ident, $guard:ident, $context:ident) => {
        #[doc = concat!("Defines a guard for an RCU critical section (`liburcu-", stringify!($flavor), "`).")]
        #[allow(dead_code)]
        pub struct $guard<'a>(PhantomData<&'a $context>);

        impl<'a> $guard<'a> {
            fn new(_context: &'a $context) -> Self {
                // SAFETY: The RCU region is unlocked upon dropping.
                unsafe { urcu_func!($flavor, read_lock)() }

                Self(PhantomData)
            }
        }

        impl<'a> Drop for $guard<'a> {
            fn drop(&mut self) {
                // SAFETY: The guard cannot be created without first locking.
                unsafe { urcu_func!($flavor, read_unlock)() }
            }
        }
    };
}

macro_rules! define_rcu_poller {
    ($flavor:ident, $poller:ident, $context:ident) => {
        #[doc = concat!("Defines a grace period poller (`liburcu-", stringify!($flavor), "`).")]
        #[allow(dead_code)]
        pub struct $poller<'a>(PhantomData<&'a mut $context>, urcu_sys::RcuPollState);

        impl<'a> $poller<'a> {
            fn new(_context: &'a mut $context) -> Self {
                // SAFETY: Context will be initialized and we may create multiple poller.
                Self(PhantomData, unsafe {
                    urcu_func!($flavor, start_poll_synchronize_rcu)()
                })
            }
        }

        impl<'a> RcuPoller for $poller<'a> {
            fn grace_period_finished(&self) -> bool {
                unsafe { urcu_func!($flavor, poll_state_synchronize_rcu)(self.1) }
            }
        }
    };
}

macro_rules! define_rcu_context {
    ($flavor:ident, $context:ident, $guard:ident, $poller:ident) => {
        #[doc = concat!("Defines an RCU context for the current thread (`liburcu-", stringify!($flavor), "`).")]
        ///
        /// #### Note
        ///
        /// There can only be 1 instance per thread.
        /// The thread will be registered upon creation.
        /// It will be unregistered upon dropping.
        pub struct $context(
            // Prevent Send+Send auto trait implementations.
            PhantomData<*const ()>
        );

        impl $context {
            /// Creates the context instance.
            ///
            /// Only the first call will return a context.
            /// Subsequent calls on the same thread will return nothing.
            pub fn new() -> Option<Self> {
                thread_local! {static RCU_CONTEXT: Cell<bool> = Cell::new(false)};

                RCU_CONTEXT.with(|initialized| {
                    if initialized.replace(true) {
                        return None;
                    }

                    // SAFETY: The registration is only called once per thread.
                    unsafe {
                        log::info!(
                            "registering thread '{}' ({}) with RCU (liburcu-{})",
                            std::thread::current().name().unwrap_or("<unnamed>"),
                            libc::gettid(),
                            stringify!($flavor),
                        );

                        urcu_func!($flavor, init)();
                        urcu_func!($flavor, register_thread)();
                    }

                    Some(Self(PhantomData))
                })
            }
        }

        impl Drop for $context {
            fn drop(&mut self) {
                // SAFETY: The unregistration may only be called once per thread.
                unsafe {
                    log::info!(
                        "unregistering thread '{}' ({}) with RCU (liburcu-{})",
                        std::thread::current().name().unwrap_or("<unnamed>"),
                        libc::gettid(),
                        stringify!($flavor),
                    );

                    urcu_func!($flavor, barrier)();

                    urcu_func!($flavor, unregister_thread)();
                }
            }
        }

        /// #### Safety
        ///
        /// 1. There can only be 1 instance per thread.
        /// 2. A barrier is called before cleanups.
        unsafe impl RcuContext for $context {
            type Guard<'a> = $guard<'a>;

            type Poller<'a> = $poller<'a>;

            fn rcu_read_lock(&self) -> Self::Guard<'_> {
                $guard::new(self)
            }

            fn rcu_synchronize(&mut self) {
                // SAFETY: The method's mutability prevents a read lock while syncronizing.
                unsafe { urcu_func!($flavor, synchronize_rcu)() }
            }

            fn rcu_synchronize_poller(&mut self) -> Self::Poller<'_> {
                $poller::new(self)
            }

            fn rcu_call<F>(callback: Box<F>)
            where
                F: RcuCallback + Send {
                    callback.configure(|mut head, func| unsafe {
                        urcu_func!($flavor, call_rcu)(head.as_mut(), Some(func));
                    });
                }

            fn rcu_api() -> &'static RcuFlavor {
                &RCU_API
            }
        }
    };
}

#[cfg(feature = "flavor-bp")]
pub(crate) mod bp {
    use super::*;

    use urcu_bp_sys::{
        urcu_bp_barrier,
        urcu_bp_call_rcu,
        urcu_bp_init,
        urcu_bp_poll_state_synchronize_rcu,
        urcu_bp_read_lock,
        urcu_bp_read_unlock,
        urcu_bp_register_thread,
        urcu_bp_start_poll_synchronize_rcu,
        urcu_bp_synchronize_rcu,
        urcu_bp_unregister_thread,
        RCU_API,
    };

    define_rcu_guard!(bp, RcuGuardBp, RcuContextBp);

    define_rcu_poller!(bp, RcuPollerBp, RcuContextBp);

    define_rcu_context!(bp, RcuContextBp, RcuGuardBp, RcuPollerBp);
}

#[cfg(feature = "flavor-mb")]
pub(crate) mod mb {
    use super::*;

    use urcu_mb_sys::{
        urcu_mb_barrier,
        urcu_mb_call_rcu,
        urcu_mb_init,
        urcu_mb_poll_state_synchronize_rcu,
        urcu_mb_read_lock,
        urcu_mb_read_unlock,
        urcu_mb_register_thread,
        urcu_mb_start_poll_synchronize_rcu,
        urcu_mb_synchronize_rcu,
        urcu_mb_unregister_thread,
        RCU_API,
    };

    define_rcu_guard!(mb, RcuGuardMb, RcuContextMb);

    define_rcu_poller!(mb, RcuPollerMb, RcuContextMb);

    define_rcu_context!(mb, RcuContextMb, RcuGuardMb, RcuPollerMb);
}

#[cfg(feature = "flavor-memb")]
pub(crate) mod memb {
    use super::*;

    use urcu_memb_sys::{
        urcu_memb_barrier,
        urcu_memb_call_rcu,
        urcu_memb_init,
        urcu_memb_poll_state_synchronize_rcu,
        urcu_memb_read_lock,
        urcu_memb_read_unlock,
        urcu_memb_register_thread,
        urcu_memb_start_poll_synchronize_rcu,
        urcu_memb_synchronize_rcu,
        urcu_memb_unregister_thread,
        RCU_API,
    };

    define_rcu_guard!(memb, RcuGuardMemb, RcuContextMemb);

    define_rcu_poller!(memb, RcuPollerMemb, RcuContextMemb);

    define_rcu_context!(memb, RcuContextMemb, RcuGuardMemb, RcuPollerMemb);
}

#[cfg(feature = "flavor-qsbr")]
pub(crate) mod qsbr {
    use super::*;

    use urcu_qsbr_sys::{
        urcu_qsbr_barrier,
        urcu_qsbr_call_rcu,
        urcu_qsbr_init,
        urcu_qsbr_poll_state_synchronize_rcu,
        urcu_qsbr_read_lock,
        urcu_qsbr_read_unlock,
        urcu_qsbr_register_thread,
        urcu_qsbr_start_poll_synchronize_rcu,
        urcu_qsbr_synchronize_rcu,
        urcu_qsbr_unregister_thread,
        RCU_API,
    };

    define_rcu_guard!(qsbr, RcuGuardQsbr, RcuContextQsbr);

    define_rcu_poller!(qsbr, RcuPollerQsbr, RcuContextQsbr);

    define_rcu_context!(qsbr, RcuContextQsbr, RcuGuardQsbr, RcuPollerQsbr);
}

#[cfg(feature = "flavor-memb")]
pub type DefaultContext = memb::RcuContextMemb;

#[cfg(all(not(feature = "flavor-memb"), feature = "flavor-mb"))]
pub type DefaultContext = mb::RcuContextMb;

#[cfg(all(
    not(feature = "flavor-memb"),
    not(feature = "flavor-mb"),
    feature = "flavor-bp"
))]
pub type DefaultContext = bp::RcuContextBp;

#[cfg(all(
    not(feature = "flavor-memb"),
    not(feature = "flavor-mb"),
    not(feature = "flavor-bp"),
    feature = "flavor-qsbr"
))]
pub type DefaultContext = qsbr::RcuContextQsbr;
