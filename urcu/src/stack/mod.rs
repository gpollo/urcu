pub(crate) mod container;
pub(crate) mod iterator;
pub(crate) mod raw;
pub(crate) mod reference;

#[cfg(test)]
mod test;

pub use crate::stack::iterator::*;
pub use crate::stack::reference::*;

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

    mod rcu_list_iter {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(Iter<'_, NotSendNotSync, DefaultContext>: Send);
        assert_not_impl_all!(Iter<'_, NotSendNotSync, DefaultContext>: Sync);

        // T: Send + !Sync
        assert_not_impl_all!(Iter<'_, SendButNotSync, DefaultContext>: Send);
        assert_not_impl_all!(Iter<'_, SendButNotSync, DefaultContext>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(Iter<'_, NotSendButSync, DefaultContext>: Send);
        assert_not_impl_all!(Iter<'_, NotSendButSync, DefaultContext>: Sync);

        // T: Send + Sync
        assert_not_impl_all!(Iter<'_, SendAndSync, DefaultContext>: Send);
        assert_not_impl_all!(Iter<'_, SendAndSync, DefaultContext>: Sync);
    }

    mod rcu_list_iter_ref {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(IterRef<NotSendNotSync, DefaultContext>: Send);
        assert_not_impl_all!(IterRef<NotSendNotSync, DefaultContext>: Sync);

        // T: Send + !Sync
        assert_not_impl_all!(IterRef<SendButNotSync, DefaultContext>: Send);
        assert_not_impl_all!(IterRef<SendButNotSync, DefaultContext>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(IterRef<NotSendButSync, DefaultContext>: Send);
        assert_not_impl_all!(IterRef<NotSendButSync, DefaultContext>: Sync);

        // T: Send + Sync
        assert_not_impl_all!(IterRef<SendAndSync, DefaultContext>: Send);
        assert_not_impl_all!(IterRef<SendAndSync, DefaultContext>: Sync);
    }
}
