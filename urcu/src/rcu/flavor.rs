use std::ffi::c_void;

use urcu_sys::{RcuFlavorApi, RcuHead, RcuPollState};

use crate::rcu::builder::RcuContextBuilder;
use crate::rcu::cleanup::{RcuCleaner, RcuCleanup, RcuCleanupMut};
use crate::rcu::context::{RcuContext, RcuDeferContext, RcuReadContext};

/// This trait defines the API from the C library.
pub trait RcuFlavor {
    /// Performs initialization on the RCU thread.
    ///
    /// #### Safety
    ///
    /// * Should be called once per thread.
    /// * Should be called before any other functions.
    unsafe fn unchecked_rcu_init();

    /// Registers a read-side RCU thread.
    ///
    /// #### Safety
    ///
    /// * The thread must be initialized with [`RcuFlavor::unchecked_rcu_init`].
    /// * The thread must not be registered with [`RcuFlavor::unchecked_rcu_read_register_thread`].
    /// * The thread must unregister with [`RcuFlavor::unchecked_rcu_read_unregister_thread`].
    unsafe fn unchecked_rcu_read_register_thread();

    /// Unregisters a read-side RCU thread.
    ///
    /// #### Safety
    ///
    /// * The thread must be initialized with [`RcuFlavor::unchecked_rcu_init`].
    /// * The thread must be registered with [`RcuFlavor::unchecked_rcu_read_register_thread`].
    unsafe fn unchecked_rcu_read_unregister_thread();

    /// Starts a RCU critical section.
    ///
    /// #### Safety
    ///
    /// * The thread must be initialized with [`RcuFlavor::unchecked_rcu_init`].
    /// * The thread must be registered with [`RcuFlavor::unchecked_rcu_read_register_thread`].
    /// * The thread must call [`RcuFlavor::unchecked_rcu_read_unlock`] later.
    unsafe fn unchecked_rcu_read_lock();

    /// Stops a RCU critical section.
    ///
    /// #### Safety
    ///
    /// * The thread must be initialized with [`RcuFlavor::unchecked_rcu_init`].
    /// * The thread must be registered with [`RcuFlavor::unchecked_rcu_read_register_thread`].
    /// * The thread must have called [`RcuFlavor::unchecked_rcu_read_lock`] before.
    unsafe fn unchecked_rcu_read_unlock();

    /// Registers a defer-enabled RCU thread.
    ///
    /// #### Safety
    ///
    /// * The thread must be initialized with [`RcuFlavor::unchecked_rcu_init`].
    /// * The thread must not be registered with [`RcuFlavor::unchecked_rcu_defer_register_thread`].
    /// * The thread must unregister with [`RcuFlavor::unchecked_rcu_defer_unregister_thread`].
    unsafe fn unchecked_rcu_defer_register_thread();

    /// Unregisters a defer-enabled RCU thread.
    ///
    /// #### Safety
    ///
    /// * The thread must be initialized with [`RcuFlavor::unchecked_rcu_init`].
    /// * The thread must be registered with [`RcuFlavor::unchecked_rcu_defer_register_thread`].
    unsafe fn unchecked_rcu_defer_unregister_thread();

    /// Executes a call after the next RCU grace period.
    ///
    /// #### Note
    ///
    /// The callback will be executed on the same thread. If the internal queue is full
    /// the call might block and the callback will be executed immediatly. In such case,
    /// [`RcuFlavor::unchecked_rcu_synchronize`] will be called internally.
    ///
    /// #### Safety
    ///
    /// * The thread must be initialized with [`RcuFlavor::unchecked_rcu_init`].
    /// * The thread must be registered with [`RcuFlavor::unchecked_rcu_defer_register_thread`].
    /// * The thread must call [`RcuFlavor::unchecked_rcu_defer_barrier`] at exit.
    /// * The thread must not be inside a RCU critical section.
    /// * The pointers must be valid until the callback is executed.
    unsafe fn unchecked_rcu_defer_call(
        func: Option<unsafe extern "C" fn(head: *mut c_void)>,
        head: *mut c_void,
    );

    /// Wait for all RCU deferred callbacks initiated by the current thread.
    ///
    /// #### Safety
    ///
    /// * The thread must be initialized with [`RcuFlavor::unchecked_rcu_init`].
    /// * The thread must be registered with [`RcuFlavor::unchecked_rcu_defer_register_thread`].
    /// * The thread must not be inside a RCU critical section.
    unsafe fn unchecked_rcu_defer_barrier();

    /// Waits until the RCU grace period is over.
    ///
    /// #### Safety
    ///
    /// * The thread must be initialized with [`RcuFlavor::unchecked_rcu_init`].
    /// * The thread must not be inside a RCU critical section.
    unsafe fn unchecked_rcu_synchronize();

    /// Indicates that this thread is in quiescent state.
    ///
    /// #### Safety
    ///
    /// * The thread must be initialized with [`RcuFlavor::unchecked_rcu_init`].
    /// * The thread must be registered with [`RcuFlavor::unchecked_rcu_read_register_thread`].
    /// * The thread must not be inside a RCU critical section.
    unsafe fn unchecked_rcu_quiescent_state();

    /// Indicates that this thread is offline.
    ///
    /// #### Safety
    ///
    /// * The thread must be initialized with [`RcuFlavor::unchecked_rcu_init`].
    /// * The thread must be registered with [`RcuFlavor::unchecked_rcu_read_register_thread`].
    /// * The thread must not read RCU data until [`RcuFlavor::unchecked_rcu_thread_online`] is called.
    /// * The thread must not be inside a RCU critical section.
    unsafe fn unchecked_rcu_thread_offline();

    /// Indicates that this thread is online.
    ///
    /// #### Safety
    ///
    /// * The thread must be initialized with [`RcuFlavor::unchecked_rcu_init`].
    /// * The thread must be registered with [`RcuFlavor::unchecked_rcu_read_register_thread`].
    /// * The thread must have called [`RcuFlavor::unchecked_rcu_thread_offline`] before.
    /// * The thread must not be inside a RCU critical section.
    unsafe fn unchecked_rcu_thread_online();

    /// Creates an [`RcuPollState`] used for checking if the grace period has ended.
    ///
    /// #### Safety
    ///
    /// * The thread must be initialized with [`RcuFlavor::unchecked_rcu_init`].
    /// * The thread must be registered with [`RcuFlavor::unchecked_rcu_read_register_thread`].
    unsafe fn unchecked_rcu_poll_start() -> RcuPollState;

    /// Polls if the grace period has ended.
    ///
    /// #### Safety
    ///
    /// * The thread must be initialized with [`RcuFlavor::unchecked_rcu_init`].
    /// * The thread must be registered with [`RcuFlavor::unchecked_rcu_read_register_thread`].
    /// * The thread must use an [`RcuPollState`] from [`RcuFlavor::unchecked_rcu_poll_start`].
    unsafe fn unchecked_rcu_poll_check(state: RcuPollState) -> bool;

    /// Executes a call after the next RCU grace period.
    ///
    /// #### Note
    ///
    /// This call nevers blocks because the callback will be executed on an helper thread.
    ///
    /// #### Safety
    ///
    /// * The thread must be initialized with [`RcuFlavor::unchecked_rcu_init`].
    /// * The thread must be registered with [`RcuFlavor::unchecked_rcu_read_register_thread`].
    /// * The thread must call [`RcuFlavor::unchecked_rcu_call_barrier`] at exit.
    /// * The pointers must be valid until the callback is executed.
    unsafe fn unchecked_rcu_call(
        func: Option<unsafe extern "C" fn(ptr: *mut RcuHead)>,
        ptr: *mut RcuHead,
    );

    /// Wait for all RCU callbacks initiated before the call by any thread to be completed.
    ///
    /// #### Safety
    ///
    /// * The thread must be initialized with [`RcuFlavor::unchecked_rcu_init`].
    /// * The thread must be registered with [`RcuFlavor::unchecked_rcu_read_register_thread`].
    unsafe fn unchecked_rcu_call_barrier();

    /// Returns the API list for this RCU flavor.
    ///
    /// #### Safety
    ///
    /// * The thread have the same requirements as the other functions when using this API.
    unsafe fn unchecked_rcu_api() -> &'static RcuFlavorApi;

    /// Defines the context used in cleanup calls.
    type CleanupContext: RcuContext + RcuReadContext + RcuDeferContext;

    /// Configures a callback to be called after the next RCU grace period is finished.
    ///
    /// Unlike [`RcuReadContext::rcu_call`] or [`RcuDeferContext::rcu_defer`], this
    /// function can be called by any thread whether it is registered or not.
    ///
    /// #### Note
    ///
    /// The callback must be [`Send`] because it will be executed by an helper thread.
    fn rcu_cleanup(callback: RcuCleanupMut<Self::CleanupContext>);

    /// Configures a callback to be called after the next RCU grace period is finished.
    ///
    /// Unlike [`RcuReadContext::rcu_call`] or [`RcuDeferContext::rcu_defer`], this
    /// function can be called by any thread whether it is registered or not.
    ///
    /// #### Note
    ///
    /// The callback must be [`Send`] because it will be executed by an helper thread.
    ///
    /// The callback does not receive a mutable context in order to prevent deadlock.
    fn rcu_cleanup_and_block(callback: RcuCleanup<Self::CleanupContext>);

    /// Creates a builder for a context of this flavor.
    fn rcu_context_builder() -> RcuContextBuilder<Self>
    where
        Self: Sized;
}

macro_rules! urcu_func {
    ($flavor:ident, $name:ident) => {
        paste::paste! {
            [<urcu _ $flavor _ $name>]
        }
    };
}

macro_rules! define_flavor {
    ($name:ident, $flavor:ident, $context:ident) => {
        #[doc = concat!("Defines a RCU flavor (`liburcu-", stringify!($flavor), "`).")]
        pub struct $name;

        impl RcuFlavor for $name {
            unsafe fn unchecked_rcu_init() {
                urcu_func!($flavor, init)()
            }

            unsafe fn unchecked_rcu_read_register_thread() {
                urcu_func!($flavor, register_thread)()
            }

            unsafe fn unchecked_rcu_read_unregister_thread() {
                urcu_func!($flavor, unregister_thread)()
            }

            unsafe fn unchecked_rcu_read_lock() {
                urcu_func!($flavor, read_lock)()
            }

            unsafe fn unchecked_rcu_read_unlock() {
                urcu_func!($flavor, read_unlock)()
            }

            unsafe fn unchecked_rcu_defer_register_thread() {
                urcu_func!($flavor, defer_register_thread)();
            }

            unsafe fn unchecked_rcu_defer_unregister_thread() {
                urcu_func!($flavor, defer_unregister_thread)()
            }

            unsafe fn unchecked_rcu_defer_call(
                func: Option<unsafe extern "C" fn(head: *mut c_void)>,
                head: *mut c_void,
            ) {
                urcu_func!($flavor, defer_rcu)(func, head)
            }

            unsafe fn unchecked_rcu_defer_barrier() {
                urcu_func!($flavor, defer_barrier)()
            }

            unsafe fn unchecked_rcu_synchronize() {
                urcu_func!($flavor, synchronize_rcu)()
            }

            unsafe fn unchecked_rcu_quiescent_state() {
                urcu_func!($flavor, quiescent_state)()
            }

            unsafe fn unchecked_rcu_thread_offline() {
                urcu_func!($flavor, thread_offline)()
            }

            unsafe fn unchecked_rcu_thread_online() {
                urcu_func!($flavor, thread_online)()
            }

            unsafe fn unchecked_rcu_poll_start() -> RcuPollState {
                urcu_func!($flavor, start_poll_synchronize_rcu)()
            }

            unsafe fn unchecked_rcu_poll_check(state: RcuPollState) -> bool {
                urcu_func!($flavor, poll_state_synchronize_rcu)(state)
            }

            unsafe fn unchecked_rcu_call(
                func: Option<unsafe extern "C" fn(ptr: *mut RcuHead)>,
                ptr: *mut RcuHead,
            ) {
                urcu_func!($flavor, call_rcu)(ptr, func)
            }

            unsafe fn unchecked_rcu_call_barrier() {
                urcu_func!($flavor, barrier)()
            }

            unsafe fn unchecked_rcu_api() -> &'static RcuFlavorApi {
                &RCU_API
            }

            type CleanupContext = $context<true, true>;

            fn rcu_cleanup(callback: RcuCleanupMut<Self::CleanupContext>) {
                RcuCleaner::<Self>::get().send_mut(callback);
            }

            fn rcu_cleanup_and_block(callback: RcuCleanup<Self::CleanupContext>) {
                RcuCleaner::<Self>::get().send(callback).barrier();
            }

            fn rcu_context_builder() -> RcuContextBuilder<Self>
            where
                Self: Sized,
            {
                RcuContextBuilder::<Self>::new()
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
        urcu_bp_defer_barrier,
        urcu_bp_defer_rcu,
        urcu_bp_defer_register_thread,
        urcu_bp_defer_unregister_thread,
        urcu_bp_init,
        urcu_bp_poll_state_synchronize_rcu,
        urcu_bp_quiescent_state,
        urcu_bp_read_lock,
        urcu_bp_read_unlock,
        urcu_bp_register_thread,
        urcu_bp_start_poll_synchronize_rcu,
        urcu_bp_synchronize_rcu,
        urcu_bp_thread_offline,
        urcu_bp_thread_online,
        urcu_bp_unregister_thread,
        RCU_API,
    };

    use crate::rcu::context::RcuContextBp;

    define_flavor!(RcuFlavorBp, bp, RcuContextBp);
}

#[cfg(feature = "flavor-mb")]
pub(crate) mod mb {
    use super::*;

    use urcu_mb_sys::{
        urcu_mb_barrier,
        urcu_mb_call_rcu,
        urcu_mb_defer_barrier,
        urcu_mb_defer_rcu,
        urcu_mb_defer_register_thread,
        urcu_mb_defer_unregister_thread,
        urcu_mb_init,
        urcu_mb_poll_state_synchronize_rcu,
        urcu_mb_quiescent_state,
        urcu_mb_read_lock,
        urcu_mb_read_unlock,
        urcu_mb_register_thread,
        urcu_mb_start_poll_synchronize_rcu,
        urcu_mb_synchronize_rcu,
        urcu_mb_thread_offline,
        urcu_mb_thread_online,
        urcu_mb_unregister_thread,
        RCU_API,
    };

    use crate::rcu::context::RcuContextMb;

    define_flavor!(RcuFlavorMb, mb, RcuContextMb);
}

#[cfg(feature = "flavor-memb")]
pub(crate) mod memb {
    use super::*;

    use urcu_memb_sys::{
        urcu_memb_barrier,
        urcu_memb_call_rcu,
        urcu_memb_defer_barrier,
        urcu_memb_defer_rcu,
        urcu_memb_defer_register_thread,
        urcu_memb_defer_unregister_thread,
        urcu_memb_init,
        urcu_memb_poll_state_synchronize_rcu,
        urcu_memb_quiescent_state,
        urcu_memb_read_lock,
        urcu_memb_read_unlock,
        urcu_memb_register_thread,
        urcu_memb_start_poll_synchronize_rcu,
        urcu_memb_synchronize_rcu,
        urcu_memb_thread_offline,
        urcu_memb_thread_online,
        urcu_memb_unregister_thread,
        RCU_API,
    };

    use crate::rcu::context::RcuContextMemb;

    define_flavor!(RcuFlavorMemb, memb, RcuContextMemb);
}

#[cfg(feature = "flavor-qsbr")]
pub(crate) mod qsbr {
    use super::*;

    use std::os::raw::c_void;

    use urcu_qsbr_sys::{
        urcu_qsbr_barrier,
        urcu_qsbr_call_rcu,
        urcu_qsbr_defer_barrier as urcu_qsbr_defer_barrier_inner,
        urcu_qsbr_defer_rcu as urcu_qsbr_defer_rcu_inner,
        urcu_qsbr_defer_register_thread,
        urcu_qsbr_defer_unregister_thread,
        urcu_qsbr_init,
        urcu_qsbr_poll_state_synchronize_rcu,
        urcu_qsbr_quiescent_state,
        urcu_qsbr_read_lock,
        urcu_qsbr_read_unlock,
        urcu_qsbr_register_thread,
        urcu_qsbr_start_poll_synchronize_rcu,
        urcu_qsbr_synchronize_rcu,
        urcu_qsbr_thread_offline,
        urcu_qsbr_thread_online,
        urcu_qsbr_unregister_thread,
        RCU_API,
    };

    use crate::rcu::context::RcuContextQsbr;

    unsafe fn urcu_qsbr_defer_barrier() {
        // From manual testing, this prevents deadlocks. This should be safe, because the thread
        // must never hold any references to RCU protected data when it waits for deferred processing.
        urcu_qsbr_thread_offline();
        urcu_qsbr_defer_barrier_inner();
        urcu_qsbr_thread_online();
    }

    unsafe fn urcu_qsbr_defer_rcu(
        fct: Option<unsafe extern "C" fn(p: *mut c_void)>,
        p: *mut c_void,
    ) {
        // From manual testing, this prevents deadlocks. This should be safe, because the thread
        // must never hold any references to RCU protected data when it defers processing.
        urcu_qsbr_thread_offline();
        urcu_qsbr_defer_rcu_inner(fct, p);
        urcu_qsbr_thread_online();
    }

    define_flavor!(RcuFlavorQsbr, qsbr, RcuContextQsbr);
}

#[cfg(feature = "flavor-bp")]
pub use bp::*;

#[cfg(feature = "flavor-mb")]
pub use mb::*;

#[cfg(feature = "flavor-memb")]
pub use memb::*;

#[cfg(feature = "flavor-qsbr")]
pub use qsbr::*;
