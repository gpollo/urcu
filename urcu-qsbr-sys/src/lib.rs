use urcu_sys::RcuFlavor;

mod bindings {
    #![allow(warnings)]

    use urcu_sys::{
        RcuAtFork as urcu_atfork,
        RcuFlavor as rcu_flavor_struct,
        RcuHead as rcu_head,
        RcuPollState as urcu_gp_poll_state,
    };

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use bindings::{
    urcu_qsbr_barrier,
    urcu_qsbr_call_rcu,
    urcu_qsbr_defer_rcu,
    urcu_qsbr_poll_state_synchronize_rcu,
    urcu_qsbr_quiescent_state,
    urcu_qsbr_read_ongoing,
    urcu_qsbr_register_rculfhash_atfork,
    urcu_qsbr_register_thread,
    urcu_qsbr_start_poll_synchronize_rcu,
    urcu_qsbr_synchronize_rcu,
    urcu_qsbr_thread_offline,
    urcu_qsbr_thread_online,
    urcu_qsbr_unregister_rculfhash_atfork,
    urcu_qsbr_unregister_thread,
};

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn urcu_qsbr_init() {}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn urcu_qsbr_read_lock() {
    #[cfg(feature = "debug")]
    bindings::urcu_qsbr_read_lock();
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn urcu_qsbr_read_unlock() {
    #[cfg(feature = "debug")]
    bindings::urcu_qsbr_read_unlock();
}

pub static RCU_API: RcuFlavor = RcuFlavor {
    read_lock: Some(urcu_qsbr_read_lock),
    read_unlock: Some(urcu_qsbr_read_unlock),
    read_ongoing: Some(urcu_qsbr_read_ongoing),
    read_quiescent_state: Some(urcu_qsbr_quiescent_state),
    update_call_rcu: Some(urcu_qsbr_call_rcu),
    update_synchronize_rcu: Some(urcu_qsbr_synchronize_rcu),
    update_defer_rcu: Some(urcu_qsbr_defer_rcu),
    thread_offline: Some(urcu_qsbr_thread_offline),
    thread_online: Some(urcu_qsbr_thread_online),
    register_thread: Some(urcu_qsbr_register_thread),
    unregister_thread: Some(urcu_qsbr_unregister_thread),
    barrier: Some(urcu_qsbr_barrier),
    register_rculfhash_atfork: Some(urcu_qsbr_register_rculfhash_atfork),
    unregister_rculfhash_atfork: Some(urcu_qsbr_unregister_rculfhash_atfork),
    update_start_poll_synchronize_rcu: Some(urcu_qsbr_start_poll_synchronize_rcu),
    update_poll_state_synchronize_rcu: Some(urcu_qsbr_poll_state_synchronize_rcu),
};
