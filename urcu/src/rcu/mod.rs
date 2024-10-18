pub(crate) mod api;
pub(crate) mod callback;
pub(crate) mod cleanup;
pub(crate) mod reference;

use std::cell::Cell;
use std::marker::PhantomData;

use crate::rcu::api::RcuUnsafe;
use crate::rcu::callback::{RcuCall, RcuDefer};
use crate::rcu::cleanup::RcuCleanup;

/// This trait is used to manually poll the RCU grace period.
pub trait RcuPoller {
    /// Checks if the grace period is over for this poller.
    fn grace_period_finished(&self) -> bool;
}

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
/// example, [`RcuContext::rcu_read_lock`] requires (`&self`), meaning we can
/// nest as many read locks as we want. On the other hand, [`RcuContext::rcu_synchronize`]
/// requires `&mut self`, meaning we can never call it while a read guard borrows
/// `&self`.
///
/// #### Safety
///
/// 1. You must enforce single context per thread for a specific RCU flavor.
///    Failure to do so can lead to a deadlock if a thread acquires a RCU read lock
///    from one context and tries to do a RCU syncronization from another context.
/// 2. For callbacks (`rcu_call`), a barrier (`rcu_barrier`) should be executed
///    before cleaning up the context. Failure to do so might results in memory
///    leaks and object cleanups that don't happen.
/// 3. For deferred callbacks (`rcu_defer`), a barrier (`defer_barrier`) should be
///    executed before cleaning up the context. Failure to do so might results in
///    memory leaks and object cleanups that don't happen.
pub unsafe trait RcuContext {
    /// Defines an API for unchecked RCU primitives.
    type Unsafe: RcuUnsafe;

    /// Defines a guard for a RCU critical section.
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

    /// Starts a RCU critical section.
    ///
    /// #### Note
    ///
    /// RCU critical sections may be nested.
    fn rcu_read_lock(&self) -> Self::Guard<'_>;

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

    /// Configures a callback to be called after the next RCU grace period is finished.
    ///
    /// Unlike [`RcuContext::rcu_call`], this function can be called by any thread whether
    /// it is registered or not.
    ///
    /// #### Note
    ///
    /// The callback must be [`Send`] because it will be executed by an helper thread.
    // TODO: To prevent deadlock, we should not give a mutable context.
    fn rcu_cleanup_and_block(callback: RcuCleanup<Self>);
}

macro_rules! define_rcu_guard {
    ($flavor:ident, $guard:ident, $unsafe:ident, $context:ident) => {
        #[doc = concat!("Defines a guard for a RCU critical section (`liburcu-", stringify!($flavor), "`).")]
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
        #[doc = concat!("Defines a RCU context for the current thread (`liburcu-", stringify!($flavor), "`).")]
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

            fn rcu_cleanup_and_block(callback: RcuCleanup<Self>) {
                Self::cleanup_send_and_block(callback);
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

/// Defines the default RCU flavor.
#[cfg(feature = "flavor-memb")]
pub type DefaultContext = flavor::memb::RcuContextMemb;

/// Defines the default RCU flavor.
#[cfg(all(not(feature = "flavor-memb"), feature = "flavor-mb"))]
pub type DefaultContext = flavor::mb::RcuContextMb;

/// Defines the default RCU flavor.
#[cfg(all(
    not(feature = "flavor-memb"),
    not(feature = "flavor-mb"),
    feature = "flavor-bp"
))]
pub type DefaultContext = flavor::bp::RcuContextBp;

/// Defines the default RCU flavor.
#[cfg(all(
    not(feature = "flavor-memb"),
    not(feature = "flavor-mb"),
    not(feature = "flavor-bp"),
    feature = "flavor-qsbr"
))]
pub type DefaultContext = flavor::qsbr::RcuContextQsbr;

/// Returns an immutable RCU-protected pointer.
///
/// > It does not actually dereference the pointer, instead, it protects the pointer
/// > for later dereferencing. It also executes any needed memory-barrier instructions
/// > for a given CPU architecture.
///
/// #### Safety
///
/// * The thread must be inside a RCU critical section.
pub unsafe fn rcu_dereference<T>(pointer: *const T) -> *const T {
    // SAFETY: It is safe to cast the pointer to a void*.
    unsafe { urcu_sys::rcu_dereference(pointer as *mut std::ffi::c_void) as *const T }
}

/// Returns a mutable RCU-protected pointer.
///
/// > It does not actually dereference the pointer, instead, it protects the pointer
/// > for later dereferencing. It also executes any needed memory-barrier instructions
/// > for a given CPU architecture.
///
/// #### Safety
///
/// * The thread must be inside a RCU critical section.
pub unsafe fn rcu_dereference_mut<T>(pointer: *mut T) -> *mut T {
    // SAFETY: It is safe to cast the pointer to a void*.
    unsafe { urcu_sys::rcu_dereference(pointer as *mut std::ffi::c_void) as *mut T }
}

mod asserts {
    use static_assertions::assert_not_impl_all;

    #[cfg(feature = "flavor-bp")]
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

    #[cfg(feature = "flavor-mb")]
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

    #[cfg(feature = "flavor-memb")]
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

    #[cfg(feature = "flavor-qsbr")]
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
