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
    urcu_memb_barrier,
    urcu_memb_call_rcu,
    urcu_memb_defer_rcu,
    urcu_memb_init,
    urcu_memb_poll_state_synchronize_rcu,
    urcu_memb_read_lock,
    urcu_memb_read_ongoing,
    urcu_memb_read_unlock,
    urcu_memb_register_rculfhash_atfork,
    urcu_memb_register_thread,
    urcu_memb_start_poll_synchronize_rcu,
    urcu_memb_synchronize_rcu,
    urcu_memb_unregister_rculfhash_atfork,
    urcu_memb_unregister_thread,
};

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn urcu_memb_quiescent_state() {}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn urcu_memb_thread_offline() {}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn urcu_memb_thread_online() {}

pub static RCU_API: RcuFlavor = RcuFlavor {
    read_lock: Some(urcu_memb_read_lock),
    read_unlock: Some(urcu_memb_read_unlock),
    read_ongoing: Some(urcu_memb_read_ongoing),
    read_quiescent_state: Some(urcu_memb_quiescent_state),
    update_call_rcu: Some(urcu_memb_call_rcu),
    update_synchronize_rcu: Some(urcu_memb_synchronize_rcu),
    update_defer_rcu: Some(urcu_memb_defer_rcu),
    thread_offline: Some(urcu_memb_thread_offline),
    thread_online: Some(urcu_memb_thread_online),
    register_thread: Some(urcu_memb_register_thread),
    unregister_thread: Some(urcu_memb_unregister_thread),
    barrier: Some(urcu_memb_barrier),
    register_rculfhash_atfork: Some(urcu_memb_register_rculfhash_atfork),
    unregister_rculfhash_atfork: Some(urcu_memb_unregister_rculfhash_atfork),
    update_start_poll_synchronize_rcu: Some(urcu_memb_start_poll_synchronize_rcu),
    update_poll_state_synchronize_rcu: Some(urcu_memb_poll_state_synchronize_rcu),
};
