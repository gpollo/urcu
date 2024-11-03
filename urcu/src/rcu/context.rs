use std::cell::Cell;
use std::marker::PhantomData;

use crate::rcu::callback::{RcuCall, RcuDefer};
use crate::rcu::flavor::RcuFlavor;
use crate::rcu::guard::RcuGuard;
use crate::rcu::poller::RcuPoller;
use crate::utility::{PhantomUnsend, PhantomUnsync};

/// This trait defines the per-thread RCU context.
///
/// #### Design
///
/// This trait exploits the borrowing rule of Rust.
///
/// > At any given time, you can have either one mutable reference (`&mut T`) or
/// > any number of immutable references (`&T`).
///
/// By exploiting this rule, we can enforce that a thread never executes a RCU
/// synchronization barrier at the same time as it holds a RCU read lock. For
/// example, [`RcuReadContext::rcu_read_lock`] requires (`&self`), meaning we can
/// nest as many read locks as we want. On the other hand, [`RcuContext::rcu_synchronize`]
/// requires `&mut self`, meaning we can never call it while a read guard borrows
/// `&self`.
///
/// #### Safety
///
/// You must enforce single context per thread for a specific RCU flavor.
/// Failure to do so can lead to a deadlock if a thread acquires a RCU read lock
/// from one context and tries to do a RCU syncronization from another context.
pub unsafe trait RcuContext {
    /// Defines an API for unchecked RCU primitives.
    type Flavor: RcuFlavor;

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

    /// Waits until the RCU grace period is over.
    ///
    /// #### Note
    ///
    /// It cannot be called in a RCU critical section.
    fn rcu_synchronize(&mut self);

    /// Creates a RCU grace period poller.
    ///
    /// #### Note
    ///
    /// It may be called in a RCU critical section.
    fn rcu_synchronize_poller(&self) -> Self::Poller<'_>;
}

/// This trait defines the per-thread RCU read context.
///
/// #### Safety
///
/// For callbacks (`rcu_call`), a barrier (`rcu_barrier`) should be executed
/// before cleaning up the context. Failure to do so might results in memory
/// leaks and object cleanups that don't happen.
pub unsafe trait RcuReadContext: RcuContext {
    /// Defines a guard for a RCU critical section.
    type Guard<'a>: RcuGuard<Flavor = Self::Flavor> + 'a
    where
        Self: 'a;

    /// Starts a RCU critical section.
    ///
    /// #### Note
    ///
    /// RCU critical sections may be nested.
    fn rcu_read_lock(&self) -> Self::Guard<'_>;

    /// Configures a callback to be called after the next RCU grace period is finished.
    ///
    /// #### Note
    ///
    /// The function will internally call [`RcuReadContext::rcu_read_lock`].
    ///
    /// The callback must be [`Send`] because it will be executed by an helper thread.
    fn rcu_call<F>(&self, callback: Box<F>)
    where
        F: RcuCall + Send + 'static;
}

/// This trait defines the per-thread RCU defer context.
///
/// #### Safety
///
/// For deferred callbacks (`rcu_defer`), a barrier (`defer_barrier`) should be
/// executed before cleaning up the context. Failure to do so might results in
/// memory leaks and object cleanups that don't happen.
pub unsafe trait RcuDeferContext: RcuContext {
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
}

macro_rules! define_rcu_context {
    ($kind:ident, $context:ident, $flavor:ident, $guard:ident, $poller:ident) => {
        #[doc = concat!("Defines a RCU context for the current thread (`liburcu-", stringify!($kind), "`).")]
        ///
        /// #### Note
        ///
        /// There can only be 1 instance per thread.
        /// The thread will be registered upon creation.
        /// It will be unregistered upon dropping.
        pub struct $context<const READ: bool = false, const DEFER: bool = false>(
            PhantomUnsend,
            PhantomUnsync,
        );

        impl<const READ: bool, const DEFER: bool> $context<READ, DEFER> {
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
                        stringify!($kind),
                    );

                    // SAFETY: Can only be called once per thread.
                    // SAFETY: It is the first RCU call for a thread.
                    unsafe { $flavor::unchecked_rcu_init() };

                    if READ {
                        // SAFETY: The thread is initialized.
                        // SAFETY: The thread is not read-registered.
                        // SAFETY: The thread is read-unregistered at context's drop.
                        unsafe { $flavor::unchecked_rcu_read_register_thread() };
                    }

                    if DEFER {
                        // SAFETY: The thread is initialized.
                        // SAFETY: The thread is not defer-registered.
                        // SAFETY: The thread is read-unregistered at context's drop.
                        unsafe { $flavor::unchecked_rcu_defer_register_thread() };
                    }

                    Some(Self(PhantomData, PhantomData))
                })
            }
        }

        impl<const READ: bool, const DEFER: bool> Drop for $context<READ, DEFER> {
            fn drop(&mut self) {
                log::info!(
                    "unregistering thread '{}' ({}) with RCU (liburcu-{})",
                    std::thread::current().name().unwrap_or("<unnamed>"),
                    unsafe { libc::gettid() },
                    stringify!($kind),
                );

                if DEFER {
                    // SAFETY: The thread is initialized at context's creation.
                    // SAFETY: The thread is defer-registered at context's creation.
                    // SAFETY: The thread can't be in a RCU critical section if it's dropping.
                    unsafe { $flavor::unchecked_rcu_defer_barrier() };

                    // SAFETY: The thread is initialized at context's creation.
                    // SAFETY: The thread is defer-registered at context's creation.
                    unsafe { $flavor::unchecked_rcu_defer_unregister_thread() };
                }

                if READ {
                    // SAFETY: The thread is initialized at context's creation.
                    // SAFETY: The thread is read-registered at context's creation.
                    unsafe { $flavor::unchecked_rcu_call_barrier() };

                    // SAFETY: The thread is initialized at context's creation.
                    // SAFETY: The thread is read-registered at context's creation.
                    unsafe { $flavor::unchecked_rcu_read_unregister_thread() };
                }
            }
        }

        /// #### Safety
        ///
        /// There can only be 1 instance per thread.
        unsafe impl<const READ: bool, const DEFER: bool> RcuContext for $context<READ, DEFER> {
            type Flavor = $flavor;

            type Poller<'a> = $poller<'a>;

            fn rcu_register() -> Option<Self>
            where
                Self: Sized,
            {
                Self::new()
            }

            fn rcu_synchronize(&mut self) {
                // SAFETY: The thread is initialized at context's creation.
                // SAFETY: The thread cannot be in a critical section because of `&mut self`.
                unsafe { $flavor::unchecked_rcu_synchronize() };
            }

            fn rcu_synchronize_poller(&self) -> Self::Poller<'_> {
                $poller::new(self)
            }
        }

        /// #### Safety
        ///
        /// `call_rcu` barrier is called before cleanups.
        unsafe impl<const DEFER: bool> RcuReadContext for $context<true, DEFER> {
            type Guard<'a> = $guard<'a>;

            fn rcu_read_lock(&self) -> Self::Guard<'_> {
                $guard::<'_>::new(self)
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
                    unsafe { $flavor::unchecked_rcu_call(Some(func), head.as_mut()) };
                });
            }
        }

        /// #### Safety
        ///
        /// `defer_rcu` barrier is called before cleanups.
        unsafe impl<const READ: bool> RcuDeferContext for $context<READ, true> {
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
                    unsafe { $flavor::unchecked_rcu_defer_call(Some(func), ptr.as_mut()) };
                });
            }
        }
    };
}

#[cfg(feature = "flavor-bp")]
mod bp {
    use super::*;

    use crate::rcu::flavor::RcuFlavorBp;
    use crate::rcu::guard::RcuGuardBp;
    use crate::rcu::poller::RcuPollerBp;

    define_rcu_context!(bp, RcuContextBp, RcuFlavorBp, RcuGuardBp, RcuPollerBp);
}

#[cfg(feature = "flavor-mb")]
mod mb {
    use super::*;

    use crate::rcu::flavor::RcuFlavorMb;
    use crate::rcu::guard::RcuGuardMb;
    use crate::rcu::poller::RcuPollerMb;

    define_rcu_context!(mb, RcuContextMb, RcuFlavorMb, RcuGuardMb, RcuPollerMb);
}

#[cfg(feature = "flavor-memb")]
mod memb {
    use super::*;

    use crate::rcu::flavor::RcuFlavorMemb;
    use crate::rcu::guard::RcuGuardMemb;
    use crate::rcu::poller::RcuPollerMemb;

    define_rcu_context!(
        memb,
        RcuContextMemb,
        RcuFlavorMemb,
        RcuGuardMemb,
        RcuPollerMemb
    );
}

#[cfg(feature = "flavor-qsbr")]
mod qsbr {
    use super::*;

    use crate::rcu::flavor::RcuFlavorQsbr;
    use crate::rcu::guard::RcuGuardQsbr;
    use crate::rcu::poller::RcuPollerQsbr;

    define_rcu_context!(
        qsbr,
        RcuContextQsbr,
        RcuFlavorQsbr,
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

mod asserts {
    use static_assertions::assert_not_impl_all;

    #[cfg(feature = "flavor-bp")]
    mod bp {
        use super::*;

        use crate::rcu::context::bp::RcuContextBp;

        assert_not_impl_all!(RcuContextBp: Send);
        assert_not_impl_all!(RcuContextBp: Sync);
    }

    #[cfg(feature = "flavor-mb")]
    mod mb {
        use super::*;

        use crate::rcu::context::mb::RcuContextMb;

        assert_not_impl_all!(RcuContextMb: Send);
        assert_not_impl_all!(RcuContextMb: Sync);
    }

    #[cfg(feature = "flavor-memb")]
    mod memb {
        use super::*;

        use crate::rcu::context::memb::RcuContextMemb;

        assert_not_impl_all!(RcuContextMemb: Send);
        assert_not_impl_all!(RcuContextMemb: Sync);
    }

    #[cfg(feature = "flavor-qsbr")]
    mod qsbr {
        use super::*;

        use crate::rcu::context::qsbr::RcuContextQsbr;

        assert_not_impl_all!(RcuContextQsbr: Send);
        assert_not_impl_all!(RcuContextQsbr: Sync);
    }
}
