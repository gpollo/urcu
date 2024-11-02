pub(crate) mod builder;
pub(crate) mod callback;
pub(crate) mod cleanup;
pub(crate) mod context;
pub(crate) mod flavor;
pub(crate) mod guard;
pub(crate) mod poller;
pub(crate) mod reference;

/// Returns an immutable RCU-protected pointer.
///
/// > It does not actually dereference the pointer, instead, it protects the pointer
/// > for later dereferencing. It also executes any needed memory-barrier instructions
/// > for a given CPU architecture.
///
/// #### Safety
///
/// * The thread must be inside a RCU critical section.
pub unsafe fn dereference<T>(pointer: *const T) -> *const T {
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
pub unsafe fn dereference_mut<T>(pointer: *mut T) -> *mut T {
    // SAFETY: It is safe to cast the pointer to a void*.
    unsafe { urcu_sys::rcu_dereference(pointer as *mut std::ffi::c_void) as *mut T }
}

#[cfg(feature = "flavor-bp")]
pub mod bp {
    pub use crate::rcu::context::RcuContextBp;
    pub use crate::rcu::flavor::RcuFlavorBp;
    pub use crate::rcu::guard::RcuGuardBp;
    pub use crate::rcu::poller::RcuPollerBp;
}

#[cfg(feature = "flavor-mb")]
pub mod mb {
    pub use crate::rcu::context::RcuContextMb;
    pub use crate::rcu::flavor::RcuFlavorMb;
    pub use crate::rcu::guard::RcuGuardMb;
    pub use crate::rcu::poller::RcuPollerMb;
}

#[cfg(feature = "flavor-memb")]
pub mod memb {
    pub use crate::rcu::context::RcuContextMemb;
    pub use crate::rcu::flavor::RcuFlavorMemb;
    pub use crate::rcu::guard::RcuGuardMemb;
    pub use crate::rcu::poller::RcuPollerMemb;
}

#[cfg(feature = "flavor-qsbr")]
pub mod qsbr {
    pub use crate::rcu::context::RcuContextQsbr;
    pub use crate::rcu::flavor::RcuFlavorQsbr;
    pub use crate::rcu::guard::RcuGuardQsbr;
    pub use crate::rcu::poller::RcuPollerQsbr;
}
