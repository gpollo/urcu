use std::ffi::c_void;

use urcu_sys::{RcuFlavorApi, RcuHead, RcuPollState};

type RcuCallSignature<T> = Option<unsafe extern "C" fn(head: T)>;

/// This trait defines the unsafe RCU API for a flavor.
pub trait RcuFlavor {
    /// Performs initialization on the RCU thread.
    ///
    /// #### Safety
    ///
    /// * The caller must call this function once before any other functions.
    unsafe fn unchecked_rcu_init();

    /// Registers a read-side RCU thread.
    ///
    /// #### Safety
    ///
    /// * The caller must unregister the thread manually.
    /// * The caller must not be a read-registered thread.
    unsafe fn unchecked_rcu_read_register_thread();

    /// Unregisters a read-side RCU thread.
    ///
    /// #### Safety
    ///
    /// * The caller must be a read-registered thread.
    unsafe fn unchecked_rcu_read_unregister_thread();

    /// Starts an RCU critical section.
    ///
    /// #### Safety
    ///
    /// * The caller must be an RCU read-registered thread.
    /// * The caller must unlock the RCU critical section manually.
    unsafe fn unchecked_rcu_read_lock();

    /// Stops an RCU critical section.
    ///
    /// #### Safety
    ///
    /// * The caller must be an RCU read-registered thread.
    /// * The caller must have activated an RCU critical section before.
    unsafe fn unchecked_rcu_read_unlock();

    /// Registers a defer-enabled RCU thread.
    ///
    /// #### Safety
    ///
    /// * The caller must unregister the thread manually.
    /// * The caller must not be an RCU defer-registered thread.
    unsafe fn unchecked_rcu_defer_register_thread();

    /// Unregisters a defer-enabled RCU thread.
    ///
    /// #### Safety
    ///
    /// * The caller must be an RCU defer-registered thread.
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
    /// * The caller must be an RCU defer-registered thread.
    /// * The caller must execute a defer barrier to prevent leaks.
    /// * The caller must not be inside an RCU critical section.
    unsafe fn unchecked_rcu_defer_call(func: RcuCallSignature<*mut c_void>, ptr: *mut c_void);

    /// Wait for all RCU deferred callbacks initiated by the current thread.
    ///
    /// #### Safety
    ///
    /// * The caller must be an RCU defer-registered thread.
    /// * The caller must not be inside an RCU critical section.
    unsafe fn unchecked_rcu_defer_barrier();

    /// Waits until the RCU grace period is over.
    ///
    /// #### Safety
    ///
    /// * The caller must not be inside an RCU critical section.
    unsafe fn unchecked_rcu_synchronize();

    /// Creates an [`RcuPollState`] used for checking if the grace period has ended.
    ///
    /// #### Safety
    ///
    /// * The caller must be an RCU read-registered thread.
    unsafe fn unchecked_rcu_poll_start() -> RcuPollState;

    /// Polls if the grace period has ended.
    ///
    /// #### Safety
    ///
    /// * The caller must be an RCU read-registered thread.
    /// * The caller must use a [`RcuPollState`] of the same flavor.
    unsafe fn unchecked_rcu_poll_check(state: RcuPollState) -> bool;

    /// Executes a call after the next RCU grace period.
    ///
    /// #### Note
    ///
    /// This call nevers blocks because the callback will be executed on an helper thread.
    ///
    /// #### Safety
    ///
    /// * The caller must be an RCU read-registered thread.
    /// * The caller must execute a call barrier to prevent leaks.
    unsafe fn unchecked_rcu_call(func: RcuCallSignature<*mut RcuHead>, ptr: *mut RcuHead);

    /// Wait for all RCU callbacks initiated before the call by any thread to be completed.
    ///
    /// #### Safety
    ///
    /// * The caller must be an RCU read-registered thread.
    /// * The caller must not be within a callback.
    unsafe fn unchecked_rcu_call_barrier();

    /// Returns the API list for this RCU flavor.
    fn rcu_api() -> &'static RcuFlavorApi;
}

macro_rules! urcu_func {
    ($flavor:ident, $name:ident) => {
        paste::paste! {
            [<urcu _ $flavor _ $name>]
        }
    };
}

macro_rules! define_flavor {
    ($name:ident, $flavor:ident) => {
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
                func: RcuCallSignature<*mut c_void>,
                ptr: *mut c_void,
            ) {
                urcu_func!($flavor, defer_rcu)(func, ptr)
            }

            unsafe fn unchecked_rcu_defer_barrier() {
                urcu_func!($flavor, defer_barrier)()
            }

            unsafe fn unchecked_rcu_synchronize() {
                urcu_func!($flavor, synchronize_rcu)()
            }

            unsafe fn unchecked_rcu_poll_start() -> RcuPollState {
                urcu_func!($flavor, start_poll_synchronize_rcu)()
            }

            unsafe fn unchecked_rcu_poll_check(state: RcuPollState) -> bool {
                urcu_func!($flavor, poll_state_synchronize_rcu)(state)
            }

            unsafe fn unchecked_rcu_call(func: RcuCallSignature<*mut RcuHead>, ptr: *mut RcuHead) {
                urcu_func!($flavor, call_rcu)(ptr, func)
            }

            unsafe fn unchecked_rcu_call_barrier() {
                urcu_func!($flavor, barrier)()
            }

            fn rcu_api() -> &'static RcuFlavorApi {
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
        urcu_bp_defer_barrier,
        urcu_bp_defer_rcu,
        urcu_bp_defer_register_thread,
        urcu_bp_defer_unregister_thread,
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

    define_flavor!(RcuFlavorBp, bp);
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
        urcu_mb_read_lock,
        urcu_mb_read_unlock,
        urcu_mb_register_thread,
        urcu_mb_start_poll_synchronize_rcu,
        urcu_mb_synchronize_rcu,
        urcu_mb_unregister_thread,
        RCU_API,
    };

    define_flavor!(RcuFlavorMb, mb);
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
        urcu_memb_read_lock,
        urcu_memb_read_unlock,
        urcu_memb_register_thread,
        urcu_memb_start_poll_synchronize_rcu,
        urcu_memb_synchronize_rcu,
        urcu_memb_unregister_thread,
        RCU_API,
    };

    define_flavor!(RcuFlavorMemb, memb);
}

#[cfg(feature = "flavor-qsbr")]
pub(crate) mod qsbr {
    use super::*;

    use urcu_qsbr_sys::{
        urcu_qsbr_barrier,
        urcu_qsbr_call_rcu,
        urcu_qsbr_defer_barrier,
        urcu_qsbr_defer_rcu,
        urcu_qsbr_defer_register_thread,
        urcu_qsbr_defer_unregister_thread,
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

    define_flavor!(RcuFlavorQsbr, qsbr);
}

#[cfg(feature = "flavor-bp")]
pub use bp::*;

#[cfg(feature = "flavor-mb")]
pub use mb::*;

#[cfg(feature = "flavor-memb")]
pub use memb::*;

#[cfg(feature = "flavor-qsbr")]
pub use qsbr::*;

#[cfg(feature = "flavor-memb")]
pub type DefaultFlavor = memb::RcuFlavorMemb;

#[cfg(all(not(feature = "flavor-memb"), feature = "flavor-mb"))]
pub type DefaultFlavor = mb::RcuFlavorMb;

#[cfg(all(
    not(feature = "flavor-memb"),
    not(feature = "flavor-mb"),
    feature = "flavor-bp"
))]
pub type DefaultFlavor = bp::RcuFlavorBp;

#[cfg(all(
    not(feature = "flavor-memb"),
    not(feature = "flavor-mb"),
    not(feature = "flavor-bp"),
    feature = "flavor-qsbr"
))]
pub type DefaultFlavor = qsbr::RcuFlavorQsbr;
