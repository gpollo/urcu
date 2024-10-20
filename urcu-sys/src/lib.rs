#![doc = include_str!("../README.md")]

mod bindings {
    #![allow(warnings)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use bindings::{
    rcu_flavor_struct as RcuFlavorApi,
    rcu_head as RcuHead,
    urcu_atfork as RcuAtFork,
    urcu_gp_poll_state as RcuPollState,
};

pub use bindings::{
    rcu_cmpxchg_pointer_sym as rcu_cmpxchg_pointer,
    rcu_dereference_sym as rcu_dereference,
    rcu_set_pointer_sym as rcu_set_pointer,
    rcu_xchg_pointer_sym as rcu_xchg_pointer,
};
