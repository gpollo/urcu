use std::cell::Cell;

use urcu_sys::RcuFlavor;

/// This trait is used to manually poll the RCU grace period.
pub trait RcuPoller {
    /// Checks if the grace period is over for this poller.
    fn grace_period_finished(&self) -> bool;
}

/// This trait defines the per-thread RCU context.
///
/// #### Safety
///
/// You must enforce single context per thread for a specific RCU flavor.
/// Failure to do so can lead to a deadlock if a thread acquires an RCU read lock
/// from one context and tries to do an RCU syncronization from another context.
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

    /// Returns the API list for this RCU flavor.
    fn rcu_api() -> &'static RcuFlavor;
}

/// This trait defines an RCU reference that can be owned after a grace period.
///
/// #### Note
///
/// To prevent memory leak and enforce object cleanup, ownership must always be eventually taken.
#[must_use]
pub trait RcuRef<C> {
    /// The output type after taking ownership.
    type Output;

    /// Take ownership of the reference.
    ///
    /// #### Safety
    ///
    /// You must wait for the grace period before taking ownership.
    unsafe fn take_ownership(self) -> Self::Output;
}

impl<T, C> RcuRef<C> for Option<T>
where
    T: RcuRef<C>,
{
    type Output = Option<T::Output>;

    unsafe fn take_ownership(self) -> Self::Output {
        self.map(|r| r.take_ownership())
    }
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

macro_rules! define_rcu_guard {
    ($flavor:literal, $name:ident, $context:ident, $lock:ident, $unlock:ident) => {
        #[doc = concat!("Defines a guard for an RCU critical section (", $flavor, ").")]
        #[allow(dead_code)]
        pub struct $name<'a>(&'a $context);

        impl<'a> $name<'a> {
            fn new(context: &'a $context) -> Self {
                // SAFETY: The RCU region is unlocked upon dropping.
                unsafe { $lock() }

                Self(context)
            }
        }

        impl<'a> Drop for $name<'a> {
            fn drop(&mut self) {
                // SAFETY: The guard cannot be created without first locking.
                unsafe { $unlock() }
            }
        }
    };
}

macro_rules! define_rcu_poller {
    ($flavor:literal, $name:ident, $context:ident, $start:ident, $poll:ident) => {
        #[doc = concat!("Defines a grace period poller (", $flavor, ").")]
        #[allow(dead_code)]
        pub struct $name<'a>(&'a mut $context, urcu_sys::RcuPollState);

        impl<'a> $name<'a> {
            fn new(context: &'a mut $context) -> Self {
                // SAFETY: Context will be initialized and we may create multiple poller.
                Self(context, unsafe { $start() })
            }
        }

        impl<'a> RcuPoller for $name<'a> {
            fn grace_period_finished(&self) -> bool {
                unsafe { $poll(self.1) }
            }
        }
    };
}

macro_rules! define_rcu_context {
    ($flavor:literal, $name:ident, $guard:ident, $poller:ident, $api:ident, $init:ident, $register:ident, $unregister:ident, $synchronize:ident) => {
        #[doc = concat!("Defines an RCU context for the current thread (", $flavor, ").")]
        ///
        /// #### Note
        ///
        /// There can only be 1 instance per thread.
        /// The thread will be registered upon creation.
        /// It will be unregistered upon dropping.
        pub struct $name;

        impl $name {
            /// Creates the context instance.
            ///
            /// Only the first call will return a context.
            /// Subsequent calls on the same thread will return nothing.
            pub fn new() -> Option<Self> {
                thread_local! {static RCU_CONTEXT: Cell<bool> = Cell::new(false)};

                RCU_CONTEXT.with(|initialized| {
                    if !initialized.get() {
                        initialized.set(true);

                        // SAFETY: The registration is only called once per thread.
                        unsafe {
                            log::info!(
                                "registering thread '{}' ({}) with RCU ({})",
                                std::thread::current().name().unwrap_or("<unnamed>"),
                                libc::gettid(),
                                $flavor,
                            );

                            $init();
                            $register();
                        }

                        Some(Self)
                    } else {
                        None
                    }
                })
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                // SAFETY: The unregistration may only be called once per thread.
                unsafe {
                    log::info!(
                        "unregistering thread '{}' ({}) with RCU ({})",
                        std::thread::current().name().unwrap_or("<unnamed>"),
                        libc::gettid(),
                        $flavor,
                    );

                    $unregister();
                }
            }
        }

        // SAFETY: There can only be 1 instance per thread.
        unsafe impl RcuContext for $name {
            type Guard<'a> = $guard<'a>;

            type Poller<'a> = $poller<'a>;

            fn rcu_read_lock(&self) -> Self::Guard<'_> {
                $guard::new(self)
            }

            fn rcu_synchronize(&mut self) {
                // SAFETY: The method's mutability prevents a read lock while syncronizing.
                unsafe { $synchronize() }
            }

            fn rcu_synchronize_poller(&mut self) -> Self::Poller<'_> {
                $poller::new(self)
            }

            fn rcu_api() -> &'static RcuFlavor {
                &$api
            }
        }
    };
}

#[cfg(feature = "flavor-bp")]
pub(crate) mod bp {
    use super::*;

    use urcu_bp_sys::{
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

    define_rcu_guard!(
        "BP",
        RcuGuardBp,
        RcuContextBp,
        urcu_bp_read_lock,
        urcu_bp_read_unlock
    );

    define_rcu_poller!(
        "BP",
        RcuPollerBp,
        RcuContextBp,
        urcu_bp_start_poll_synchronize_rcu,
        urcu_bp_poll_state_synchronize_rcu
    );

    define_rcu_context!(
        "BP",
        RcuContextBp,
        RcuGuardBp,
        RcuPollerBp,
        RCU_API,
        urcu_bp_init,
        urcu_bp_register_thread,
        urcu_bp_unregister_thread,
        urcu_bp_synchronize_rcu
    );
}

#[cfg(feature = "flavor-mb")]
pub(crate) mod mb {
    use super::*;

    use urcu_mb_sys::{
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

    define_rcu_guard!(
        "MB",
        RcuGuardMb,
        RcuContextMb,
        urcu_mb_read_lock,
        urcu_mb_read_unlock
    );

    define_rcu_poller!(
        "MB",
        RcuPollerMb,
        RcuContextMb,
        urcu_mb_start_poll_synchronize_rcu,
        urcu_mb_poll_state_synchronize_rcu
    );

    define_rcu_context!(
        "MB",
        RcuContextMb,
        RcuGuardMb,
        RcuPollerMb,
        RCU_API,
        urcu_mb_init,
        urcu_mb_register_thread,
        urcu_mb_unregister_thread,
        urcu_mb_synchronize_rcu
    );
}

#[cfg(feature = "flavor-memb")]
pub(crate) mod memb {
    use super::*;

    use urcu_memb_sys::{
        urcu_memb_init,
        urcu_memb_poll_state_synchronize_rcu,
        urcu_memb_read_lock,
        urcu_memb_read_ongoing,
        urcu_memb_read_unlock,
        urcu_memb_register_thread,
        urcu_memb_start_poll_synchronize_rcu,
        urcu_memb_synchronize_rcu,
        urcu_memb_unregister_thread,
        RCU_API,
    };

    define_rcu_guard!(
        "MEMB",
        RcuGuardMemb,
        RcuContextMemb,
        urcu_memb_read_lock,
        urcu_memb_read_unlock
    );

    define_rcu_poller!(
        "MEMB",
        RcuPollerMemb,
        RcuContextMemb,
        urcu_memb_start_poll_synchronize_rcu,
        urcu_memb_poll_state_synchronize_rcu
    );

    define_rcu_context!(
        "MEMB",
        RcuContextMemb,
        RcuGuardMemb,
        RcuPollerMemb,
        RCU_API,
        urcu_memb_init,
        urcu_memb_register_thread,
        urcu_memb_unregister_thread,
        urcu_memb_synchronize_rcu
    );
}

#[cfg(feature = "flavor-qsbr")]
pub(crate) mod qsbr {
    use super::*;

    use urcu_qsbr_sys::{
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

    define_rcu_guard!(
        "QSBR",
        RcuGuardQsbr,
        RcuContextQsbr,
        urcu_qsbr_read_lock,
        urcu_qsbr_read_unlock
    );

    define_rcu_poller!(
        "QSBR",
        RcuPollerQsbr,
        RcuContextQsbr,
        urcu_qsbr_start_poll_synchronize_rcu,
        urcu_qsbr_poll_state_synchronize_rcu
    );

    define_rcu_context!(
        "QSBR",
        RcuContextQsbr,
        RcuGuardQsbr,
        RcuPollerQsbr,
        RCU_API,
        urcu_qsbr_init,
        urcu_qsbr_register_thread,
        urcu_qsbr_unregister_thread,
        urcu_qsbr_synchronize_rcu
    );
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
