use urcu_sys::RcuFlavorApi;

mod bindings {
    #![allow(warnings)]

    use urcu_sys::{
        RcuAtFork as urcu_atfork,
        RcuFlavorApi as rcu_flavor_struct,
        RcuHead as rcu_head,
        RcuPollState as urcu_gp_poll_state,
    };

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use bindings::{
    urcu_mb_barrier,
    urcu_mb_call_rcu,
    urcu_mb_defer_barrier,
    urcu_mb_defer_rcu,
    urcu_mb_defer_register_thread,
    urcu_mb_defer_unregister_thread,
    urcu_mb_init,
    urcu_mb_poll_state_synchronize_rcu,
    urcu_mb_read_lock,
    urcu_mb_read_ongoing,
    urcu_mb_read_unlock,
    urcu_mb_register_rculfhash_atfork,
    urcu_mb_register_thread,
    urcu_mb_start_poll_synchronize_rcu,
    urcu_mb_synchronize_rcu,
    urcu_mb_unregister_rculfhash_atfork,
    urcu_mb_unregister_thread,
};

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn urcu_mb_quiescent_state() {}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn urcu_mb_thread_offline() {}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn urcu_mb_thread_online() {}

pub static RCU_API: RcuFlavorApi = RcuFlavorApi {
    read_lock: Some(urcu_mb_read_lock),
    read_unlock: Some(urcu_mb_read_unlock),
    read_ongoing: Some(urcu_mb_read_ongoing),
    read_quiescent_state: Some(urcu_mb_quiescent_state),
    update_call_rcu: Some(urcu_mb_call_rcu),
    update_synchronize_rcu: Some(urcu_mb_synchronize_rcu),
    update_defer_rcu: Some(urcu_mb_defer_rcu),
    thread_offline: Some(urcu_mb_thread_offline),
    thread_online: Some(urcu_mb_thread_online),
    register_thread: Some(urcu_mb_register_thread),
    unregister_thread: Some(urcu_mb_unregister_thread),
    barrier: Some(urcu_mb_barrier),
    register_rculfhash_atfork: Some(urcu_mb_register_rculfhash_atfork),
    unregister_rculfhash_atfork: Some(urcu_mb_unregister_rculfhash_atfork),
    update_start_poll_synchronize_rcu: Some(urcu_mb_start_poll_synchronize_rcu),
    update_poll_state_synchronize_rcu: Some(urcu_mb_poll_state_synchronize_rcu),
};
