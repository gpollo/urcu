use std::marker::PhantomData;

use crate::rcu::context::RcuContext;

pub struct RcuContextBuilder<F, const READ: bool = false, const DEFER: bool = false>(
    PhantomData<F>,
);

impl<F, const READ: bool, const DEFER: bool> RcuContextBuilder<F, READ, DEFER> {
    pub fn new() -> RcuContextBuilder<F> {
        RcuContextBuilder::<F, false, false>(PhantomData)
    }
}

impl<F, const DEFER: bool> RcuContextBuilder<F, false, DEFER> {
    pub fn with_read_context(self) -> RcuContextBuilder<F, true, DEFER> {
        RcuContextBuilder::<F, true, DEFER>(PhantomData)
    }
}

impl<F, const READ: bool> RcuContextBuilder<F, READ, false> {
    pub fn with_defer_context(self) -> RcuContextBuilder<F, READ, true> {
        RcuContextBuilder::<F, READ, true>(PhantomData)
    }
}

#[cfg(feature = "flavor-bp")]
mod bp {
    use super::*;

    use crate::rcu::context::RcuContextBp;
    use crate::rcu::flavor::RcuFlavorBp;

    impl<const READ: bool, const DEFER: bool> RcuContextBuilder<RcuFlavorBp, READ, DEFER> {
        pub fn register_thread(self) -> Option<RcuContextBp<READ, DEFER>> {
            RcuContextBp::<READ, DEFER>::rcu_register()
        }
    }
}

#[cfg(feature = "flavor-mb")]
mod mb {
    use super::*;

    use crate::rcu::context::RcuContextMb;
    use crate::rcu::flavor::RcuFlavorMb;

    impl<const READ: bool, const DEFER: bool> RcuContextBuilder<RcuFlavorMb, READ, DEFER> {
        pub fn register_thread(self) -> Option<RcuContextMb<READ, DEFER>> {
            RcuContextMb::<READ, DEFER>::rcu_register()
        }
    }
}

#[cfg(feature = "flavor-memb")]
mod memb {
    use super::*;

    use crate::rcu::context::RcuContextMemb;
    use crate::rcu::flavor::RcuFlavorMemb;

    impl<const READ: bool, const DEFER: bool> RcuContextBuilder<RcuFlavorMemb, READ, DEFER> {
        pub fn register_thread(self) -> Option<RcuContextMemb<READ, DEFER>> {
            RcuContextMemb::<READ, DEFER>::rcu_register()
        }
    }
}

#[cfg(feature = "flavor-qsbr")]
mod qsbr {
    use super::*;

    use crate::rcu::context::RcuContextQsbr;
    use crate::rcu::flavor::RcuFlavorQsbr;

    impl<const READ: bool, const DEFER: bool> RcuContextBuilder<RcuFlavorQsbr, READ, DEFER> {
        pub fn register_thread(self) -> Option<RcuContextQsbr<READ, DEFER>> {
            RcuContextQsbr::<READ, DEFER>::rcu_register()
        }
    }
}
