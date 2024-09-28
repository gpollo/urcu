pub(crate) mod container;
pub(crate) mod iterator;
pub(crate) mod raw;
pub(crate) mod reference;

pub use crate::linked_list::container::{Entry, Reader, Writer};
pub use crate::linked_list::iterator::*;
pub use crate::linked_list::reference::*;

mod asserts {
    use super::*;

    use static_assertions::{assert_impl_all, assert_not_impl_all};

    use crate::linked_list::container::RcuList;
    use crate::rcu::DefaultContext;
    use crate::utility::asserts::*;

    mod rcu_list {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(RcuList<NotSendNotSync, DefaultContext>: Send);
        assert_not_impl_all!(RcuList<NotSendNotSync, DefaultContext>: Sync);

        // T: Send + !Sync
        assert_impl_all!(RcuList<SendButNotSync, DefaultContext>: Send);
        assert_not_impl_all!(RcuList<SendButNotSync, DefaultContext>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(RcuList<NotSendButSync, DefaultContext>: Send);
        assert_impl_all!(RcuList<NotSendButSync, DefaultContext>: Sync);

        // T: Send + Sync
        assert_impl_all!(RcuList<SendAndSync, DefaultContext>: Send);
        assert_impl_all!(RcuList<SendAndSync, DefaultContext>: Sync);
    }

    mod rcu_list_reader {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(Reader<'static, NotSendNotSync, DefaultContext>: Send);
        assert_not_impl_all!(Reader<'static, NotSendNotSync, DefaultContext>: Sync);

        // T: Send + !Sync
        assert_not_impl_all!(Reader<'static, SendButNotSync, DefaultContext>: Send);
        assert_not_impl_all!(Reader<'static, SendButNotSync, DefaultContext>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(Reader<'static, NotSendButSync, DefaultContext>: Send);
        assert_not_impl_all!(Reader<'static, NotSendButSync, DefaultContext>: Sync);

        // T: Send + Sync
        assert_not_impl_all!(Reader<'static, SendAndSync, DefaultContext>: Send);
        assert_not_impl_all!(Reader<'static, SendAndSync, DefaultContext>: Sync);
    }

    mod rcu_list_writer {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(Writer<NotSendNotSync, DefaultContext>: Send);
        assert_not_impl_all!(Writer<NotSendNotSync, DefaultContext>: Sync);

        // T: Send + !Sync
        assert_not_impl_all!(Writer<SendButNotSync, DefaultContext>: Send);
        assert_not_impl_all!(Writer<SendButNotSync, DefaultContext>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(Writer<NotSendButSync, DefaultContext>: Send);
        assert_not_impl_all!(Writer<NotSendButSync, DefaultContext>: Sync);

        // T: Send + Sync
        assert_not_impl_all!(Writer<SendAndSync, DefaultContext>: Send);
        assert_not_impl_all!(Writer<SendAndSync, DefaultContext>: Sync);
    }

    mod rcu_list_ref {
        use super::*;

        // T: Send + !Sync
        assert_impl_all!(Ref<SendButNotSync, DefaultContext>: Send);
        assert_not_impl_all!(Ref<SendButNotSync, DefaultContext>: Sync);

        // T: Send + Sync
        assert_impl_all!(Ref<SendAndSync, DefaultContext>: Send);
        assert_not_impl_all!(Ref<SendAndSync, DefaultContext>: Sync);
    }

    mod rcu_list_ref_owned {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(RefOwned<NotSendNotSync>: Send);
        assert_not_impl_all!(RefOwned<NotSendNotSync>: Sync);

        // T: Send + !Sync
        assert_impl_all!(RefOwned<SendButNotSync>: Send);
        assert_not_impl_all!(RefOwned<SendButNotSync>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(RefOwned<NotSendButSync>: Send);
        assert_impl_all!(RefOwned<NotSendButSync>: Sync);

        // T: Send + Sync
        assert_impl_all!(RefOwned<SendAndSync>: Send);
        assert_impl_all!(RefOwned<SendAndSync>: Sync);
    }

    mod rcu_list_iter {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(Iter<NotSendNotSync, DefaultContext>: Send);
        assert_not_impl_all!(Iter<NotSendNotSync, DefaultContext>: Sync);

        // T: Send + !Sync
        assert_not_impl_all!(Iter<SendButNotSync, DefaultContext>: Send);
        assert_not_impl_all!(Iter<SendButNotSync, DefaultContext>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(Iter<NotSendButSync, DefaultContext>: Send);
        assert_not_impl_all!(Iter<NotSendButSync, DefaultContext>: Sync);

        // T: Send + Sync
        assert_not_impl_all!(Iter<SendAndSync, DefaultContext>: Send);
        assert_not_impl_all!(Iter<SendAndSync, DefaultContext>: Sync);
    }

    mod rcu_list_entry {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(Entry<'static, NotSendNotSync, DefaultContext>: Send);
        assert_not_impl_all!(Entry<'static, NotSendNotSync, DefaultContext>: Sync);

        // T: Send + !Sync
        assert_not_impl_all!(Entry<'static, SendButNotSync, DefaultContext>: Send);
        assert_not_impl_all!(Entry<'static, SendButNotSync, DefaultContext>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(Entry<'static, NotSendButSync, DefaultContext>: Send);
        assert_not_impl_all!(Entry<'static, NotSendButSync, DefaultContext>: Sync);

        // T: Send + Sync
        assert_not_impl_all!(Entry<'static, SendAndSync, DefaultContext>: Send);
        assert_not_impl_all!(Entry<'static, SendAndSync, DefaultContext>: Sync);
    }
}
