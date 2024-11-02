use std::marker::PhantomData;

use crate::rcu::context::RcuContext;
use crate::rcu::flavor::RcuFlavor;
use crate::utility::{PhantomUnsend, PhantomUnsync};

/// This trait defines a poller of the grace period.
pub trait RcuPoller {
    /// Checks if the grace period is over for this poller.
    fn grace_period_finished(&self) -> bool;
}

macro_rules! define_rcu_poller {
    ($kind:ident, $poller:ident, $flavor:ident) => {
        #[doc = concat!("Defines a grace period poller (`liburcu-", stringify!($kind), "`).")]
        #[allow(dead_code)]
        pub struct $poller<'a>(
            PhantomUnsend<&'a ()>,
            PhantomUnsync<&'a ()>,
            urcu_sys::RcuPollState,
        );

        impl<'a> $poller<'a> {
            pub(crate) fn new<C: RcuContext>(context: &'a C) -> Self {
                let _ = context;

                Self(PhantomData, PhantomData, {
                    // SAFETY: The thread is initialized at context's creation.
                    // SAFETY: The thread is read-registered at context's creation.
                    unsafe { $flavor::unchecked_rcu_poll_start() }
                })
            }
        }

        impl<'a> RcuPoller for $poller<'a> {
            fn grace_period_finished(&self) -> bool {
                // SAFETY: The thread is initialized at context's creation.
                // SAFETY: The thread is read-registered at context's creation.
                // SAFETY: The handle is created at poller's creation.
                unsafe { $flavor::unchecked_rcu_poll_check(self.2) }
            }
        }
    };
}

#[cfg(feature = "flavor-bp")]
mod bp {
    use super::*;

    use crate::rcu::flavor::RcuFlavorBp;

    define_rcu_poller!(bp, RcuPollerBp, RcuFlavorBp);
}

#[cfg(feature = "flavor-mb")]
mod mb {
    use super::*;

    use crate::rcu::flavor::RcuFlavorMb;

    define_rcu_poller!(mb, RcuPollerMb, RcuFlavorMb);
}

#[cfg(feature = "flavor-memb")]
mod memb {
    use super::*;

    use crate::rcu::flavor::RcuFlavorMemb;

    define_rcu_poller!(memb, RcuPollerMemb, RcuFlavorMemb);
}

#[cfg(feature = "flavor-qsbr")]
mod qsbr {
    use super::*;

    use crate::rcu::flavor::RcuFlavorQsbr;

    define_rcu_poller!(qsbr, RcuPollerQsbr, RcuFlavorQsbr);
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

        use crate::rcu::poller::RcuPollerBp;

        assert_not_impl_all!(RcuPollerBp: Send);
        assert_not_impl_all!(RcuPollerBp: Sync);
    }

    #[cfg(feature = "flavor-mb")]
    mod mb {
        use super::*;

        use crate::rcu::poller::RcuPollerMb;

        assert_not_impl_all!(RcuPollerMb: Send);
        assert_not_impl_all!(RcuPollerMb: Sync);
    }

    #[cfg(feature = "flavor-memb")]
    mod memb {
        use super::*;

        use crate::rcu::poller::RcuPollerMemb;

        assert_not_impl_all!(RcuPollerMemb: Send);
        assert_not_impl_all!(RcuPollerMemb: Sync);
    }

    #[cfg(feature = "flavor-qsbr")]
    mod qsbr {
        use super::*;

        use crate::rcu::poller::RcuPollerQsbr;

        assert_not_impl_all!(RcuPollerQsbr: Send);
        assert_not_impl_all!(RcuPollerQsbr: Sync);
    }
}
