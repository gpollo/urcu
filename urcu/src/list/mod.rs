pub(crate) mod container;
pub(crate) mod iterator;
pub(crate) mod raw;
pub(crate) mod reference;

#[cfg(test)]
mod test;

pub use crate::list::iterator::*;
pub use crate::list::reference::*;

mod asserts {
    use super::*;

    use static_assertions::{assert_impl_all, assert_not_impl_all};

    use crate::list::container::RcuList;
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

    mod rcu_list_iter_forward {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(Iter<'_, '_, NotSendNotSync, DefaultContext, true>: Send);
        assert_not_impl_all!(Iter<'_, '_, NotSendNotSync, DefaultContext, true>: Sync);

        // T: Send + !Sync
        assert_not_impl_all!(Iter<'_, '_, SendButNotSync, DefaultContext, true>: Send);
        assert_not_impl_all!(Iter<'_, '_, SendButNotSync, DefaultContext, true>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(Iter<'_, '_, NotSendButSync, DefaultContext, true>: Send);
        assert_not_impl_all!(Iter<'_, '_, NotSendButSync, DefaultContext, true>: Sync);

        // T: Send + Sync
        assert_not_impl_all!(Iter<'_, '_, SendAndSync, DefaultContext, true>: Send);
        assert_not_impl_all!(Iter<'_, '_, SendAndSync, DefaultContext, true>: Sync);
    }

    mod rcu_list_iter_backward {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(Iter<'_, '_, NotSendNotSync, DefaultContext, false>: Send);
        assert_not_impl_all!(Iter<'_, '_, NotSendNotSync, DefaultContext, false>: Sync);

        // T: Send + !Sync
        assert_not_impl_all!(Iter<'_, '_, SendButNotSync, DefaultContext, false>: Send);
        assert_not_impl_all!(Iter<'_, '_, SendButNotSync, DefaultContext, false>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(Iter<'_, '_, NotSendButSync, DefaultContext, false>: Send);
        assert_not_impl_all!(Iter<'_, '_, NotSendButSync, DefaultContext, false>: Sync);

        // T: Send + Sync
        assert_not_impl_all!(Iter<'_, '_, SendAndSync, DefaultContext, false>: Send);
        assert_not_impl_all!(Iter<'_, '_, SendAndSync, DefaultContext, false>: Sync);
    }
}
