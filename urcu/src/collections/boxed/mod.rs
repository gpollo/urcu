pub(crate) mod container;
pub(crate) mod reference;

pub use crate::collections::boxed::reference::*;

mod asserts {
    use super::*;

    use static_assertions::{assert_impl_all, assert_not_impl_all};

    use crate::collections::boxed::container::*;
    use crate::rcu::default::RcuDefaultFlavor;
    use crate::utility::asserts::*;

    mod rcu_box {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(RcuBox<NotSendNotSync, RcuDefaultFlavor>: Send);
        assert_not_impl_all!(RcuBox<NotSendNotSync, RcuDefaultFlavor>: Sync);

        // T: Send + !Sync
        assert_impl_all!(RcuBox<SendButNotSync, RcuDefaultFlavor>: Send);
        assert_not_impl_all!(RcuBox<SendButNotSync, RcuDefaultFlavor>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(RcuBox<NotSendButSync, RcuDefaultFlavor>: Send);
        assert_impl_all!(RcuBox<NotSendButSync, RcuDefaultFlavor>: Sync);

        // T: Send + Sync
        assert_impl_all!(RcuBox<SendAndSync, RcuDefaultFlavor>: Send);
        assert_impl_all!(RcuBox<SendAndSync, RcuDefaultFlavor>: Sync);
    }

    mod rcu_box_ref {
        use super::*;

        // T: Send + !Sync
        assert_impl_all!(Ref<SendButNotSync, RcuDefaultFlavor>: Send);
        assert_not_impl_all!(Ref<SendButNotSync, RcuDefaultFlavor>: Sync);

        // T: Send + Sync
        assert_impl_all!(Ref<SendAndSync, RcuDefaultFlavor>: Send);
        assert_not_impl_all!(Ref<SendAndSync, RcuDefaultFlavor>: Sync);
    }
}
