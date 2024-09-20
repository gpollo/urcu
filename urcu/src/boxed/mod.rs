pub(crate) mod container;
pub(crate) mod reference;

pub use crate::boxed::container::Accessor;
pub use crate::boxed::reference::*;

mod asserts {
    use static_assertions::{assert_impl_all, assert_not_impl_all};

    use crate::boxed::container::*;
    use crate::boxed::reference::*;
    use crate::rcu::DefaultContext;
    use crate::utility::asserts::*;

    mod rcu_box {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(RcuBox<NotSendNotSync, DefaultContext>: Send);
        assert_not_impl_all!(RcuBox<NotSendNotSync, DefaultContext>: Sync);

        // T: Send + !Sync
        assert_impl_all!(RcuBox<SendButNotSync, DefaultContext>: Send);
        assert_not_impl_all!(RcuBox<SendButNotSync, DefaultContext>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(RcuBox<NotSendButSync, DefaultContext>: Send);
        assert_impl_all!(RcuBox<NotSendButSync, DefaultContext>: Sync);

        // T: Send + Sync
        assert_impl_all!(RcuBox<SendAndSync, DefaultContext>: Send);
        assert_impl_all!(RcuBox<SendAndSync, DefaultContext>: Sync);
    }

    mod rcu_box_accessor {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(Accessor<'static, NotSendNotSync, DefaultContext>: Send);
        assert_not_impl_all!(Accessor<'static, NotSendNotSync, DefaultContext>: Sync);

        // T: Send + !Sync
        assert_not_impl_all!(Accessor<'static, SendButNotSync, DefaultContext>: Send);
        assert_not_impl_all!(Accessor<'static, SendButNotSync, DefaultContext>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(Accessor<'static, NotSendButSync, DefaultContext>: Send);
        assert_not_impl_all!(Accessor<'static, NotSendButSync, DefaultContext>: Sync);

        // T: Send + Sync
        assert_not_impl_all!(Accessor<'static, SendAndSync, DefaultContext>: Send);
        assert_not_impl_all!(Accessor<'static, SendAndSync, DefaultContext>: Sync);
    }

    mod rcu_box_ref {
        use super::*;

        // T: Send + !Sync
        assert_impl_all!(Ref<SendButNotSync, DefaultContext>: Send);
        assert_not_impl_all!(Ref<SendButNotSync, DefaultContext>: Sync);

        // T: Send + Sync
        assert_impl_all!(Ref<SendAndSync, DefaultContext>: Send);
        assert_not_impl_all!(Ref<SendAndSync, DefaultContext>: Sync);
    }
}
