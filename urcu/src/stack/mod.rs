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

    use crate::rcu::DefaultContext;
    use crate::stack::container::RcuStack;
    use crate::utility::asserts::*;

    mod rcu_list {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(RcuStack<NotSendNotSync, DefaultContext>: Send);
        assert_not_impl_all!(RcuStack<NotSendNotSync, DefaultContext>: Sync);

        // T: Send + !Sync
        assert_impl_all!(RcuStack<SendButNotSync, DefaultContext>: Send);
        assert_not_impl_all!(RcuStack<SendButNotSync, DefaultContext>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(RcuStack<NotSendButSync, DefaultContext>: Send);
        assert_impl_all!(RcuStack<NotSendButSync, DefaultContext>: Sync);

        // T: Send + Sync
        assert_impl_all!(RcuStack<SendAndSync, DefaultContext>: Send);
        assert_impl_all!(RcuStack<SendAndSync, DefaultContext>: Sync);
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
        assert_not_impl_all!(Iter<'_, '_, NotSendNotSync, DefaultContext>: Send);
        assert_not_impl_all!(Iter<'_, '_, NotSendNotSync, DefaultContext>: Sync);

        // T: Send + !Sync
        assert_not_impl_all!(Iter<'_, '_, SendButNotSync, DefaultContext>: Send);
        assert_not_impl_all!(Iter<'_, '_, SendButNotSync, DefaultContext>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(Iter<'_, '_, NotSendButSync, DefaultContext>: Send);
        assert_not_impl_all!(Iter<'_, '_, NotSendButSync, DefaultContext>: Sync);

        // T: Send + Sync
        assert_not_impl_all!(Iter<'_, '_, SendAndSync, DefaultContext>: Send);
        assert_not_impl_all!(Iter<'_, '_, SendAndSync, DefaultContext>: Sync);
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
