use std::marker::PhantomData;

use crate::rcu::flavor::RcuFlavor;
use crate::rcu::RcuContext;
use crate::utility::{PhantomUnsend, PhantomUnsync};

/// This trait defines a guard for a read-side lock.
pub trait RcuGuard {
    /// Defines the flavor of the guard.
    type Flavor: RcuFlavor;
}

macro_rules! define_rcu_guard {
    ($kind:ident, $guard:ident, $flavor:ident) => {
        #[doc = concat!("Defines a guard for a RCU critical section (`liburcu-", stringify!($kind), "`).")]
        #[allow(dead_code)]
        pub struct $guard<'a>(PhantomUnsend<&'a ()>, PhantomUnsync<&'a ()>);

        impl<'a> $guard<'a> {
            pub(crate) fn new<C: RcuContext>(context: &'a C) -> Self {
                let _ = context;

                // SAFETY: The thread is initialized at context's creation.
                // SAFETY: The thread is read-registered at context's creation.
                // SAFETY: The critical section is unlocked at guard's drop.
                unsafe { $flavor::unchecked_rcu_read_lock() };

                Self(PhantomData, PhantomData)
            }
        }

        impl<'a> RcuGuard for $guard<'a> {
            type Flavor = $flavor;
        }

        impl<'a> Drop for $guard<'a> {
            fn drop(&mut self) {
                // SAFETY: The thread is initialized at context's creation.
                // SAFETY: The thread is read-registered at context's creation.
                // SAFETY: The critical section is locked at guard's creation.
                unsafe { $flavor::unchecked_rcu_read_unlock() };
            }
        }
    };
}

#[cfg(feature = "flavor-bp")]
mod bp {
    use super::*;

    use crate::rcu::flavor::RcuFlavorBp;

    define_rcu_guard!(bp, RcuGuardBp, RcuFlavorBp);
}

#[cfg(feature = "flavor-mb")]
mod mb {
    use super::*;

    use crate::rcu::flavor::RcuFlavorMb;

    define_rcu_guard!(mb, RcuGuardMb, RcuFlavorMb);
}

#[cfg(feature = "flavor-memb")]
mod memb {
    use super::*;

    use crate::rcu::flavor::RcuFlavorMemb;

    define_rcu_guard!(memb, RcuGuardMemb, RcuFlavorMemb);
}

#[cfg(feature = "flavor-qsbr")]
mod qsbr {
    use super::*;

    use crate::rcu::flavor::RcuFlavorQsbr;

    define_rcu_guard!(qsbr, RcuGuardQsbr, RcuFlavorQsbr);
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

        use crate::rcu::guard::RcuGuardBp;

        assert_not_impl_all!(RcuGuardBp: Send);
        assert_not_impl_all!(RcuGuardBp: Sync);
    }

    #[cfg(feature = "flavor-mb")]
    mod mb {
        use super::*;

        use crate::rcu::guard::RcuGuardMb;

        assert_not_impl_all!(RcuGuardMb: Send);
        assert_not_impl_all!(RcuGuardMb: Sync);
    }

    #[cfg(feature = "flavor-memb")]
    mod memb {
        use super::*;

        use crate::rcu::guard::RcuGuardMemb;

        assert_not_impl_all!(RcuGuardMemb: Send);
        assert_not_impl_all!(RcuGuardMemb: Sync);
    }

    #[cfg(feature = "flavor-qsbr")]
    mod qsbr {
        use super::*;

        use crate::rcu::guard::RcuGuardQsbr;

        assert_not_impl_all!(RcuGuardQsbr: Send);
        assert_not_impl_all!(RcuGuardQsbr: Sync);
    }
}
