pub(crate) mod container;
pub(crate) mod reference;

pub use crate::collections::boxed::reference::*;

mod asserts {
    use super::*;

    use static_assertions::{assert_impl_all, assert_not_impl_all};

    use crate::collections::boxed::container::*;
    use crate::rcu::default::DefaultFlavor;
    use crate::utility::asserts::*;

    mod rcu_box {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(RcuBox<NotSendNotSync, DefaultFlavor>: Send);
        assert_not_impl_all!(RcuBox<NotSendNotSync, DefaultFlavor>: Sync);

        // T: Send + !Sync
        assert_impl_all!(RcuBox<SendButNotSync, DefaultFlavor>: Send);
        assert_not_impl_all!(RcuBox<SendButNotSync, DefaultFlavor>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(RcuBox<NotSendButSync, DefaultFlavor>: Send);
        assert_impl_all!(RcuBox<NotSendButSync, DefaultFlavor>: Sync);

        // T: Send + Sync
        assert_impl_all!(RcuBox<SendAndSync, DefaultFlavor>: Send);
        assert_impl_all!(RcuBox<SendAndSync, DefaultFlavor>: Sync);
    }

    mod rcu_box_ref {
        use super::*;

        // T: Send + !Sync
        assert_impl_all!(Ref<SendButNotSync, DefaultFlavor>: Send);
        assert_not_impl_all!(Ref<SendButNotSync, DefaultFlavor>: Sync);

        // T: Send + Sync
        assert_impl_all!(Ref<SendAndSync, DefaultFlavor>: Send);
        assert_not_impl_all!(Ref<SendAndSync, DefaultFlavor>: Sync);
    }
}
