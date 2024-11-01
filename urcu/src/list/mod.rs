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

    use crate::flavor::DefaultFlavor;
    use crate::list::container::RcuList;
    use crate::rcu::context::RcuGuardMemb;
    use crate::utility::asserts::*;

    mod rcu_list {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(RcuList<NotSendNotSync, DefaultFlavor>: Send);
        assert_not_impl_all!(RcuList<NotSendNotSync, DefaultFlavor>: Sync);

        // T: Send + !Sync
        assert_impl_all!(RcuList<SendButNotSync, DefaultFlavor>: Send);
        assert_not_impl_all!(RcuList<SendButNotSync, DefaultFlavor>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(RcuList<NotSendButSync, DefaultFlavor>: Send);
        assert_impl_all!(RcuList<NotSendButSync, DefaultFlavor>: Sync);

        // T: Send + Sync
        assert_impl_all!(RcuList<SendAndSync, DefaultFlavor>: Send);
        assert_impl_all!(RcuList<SendAndSync, DefaultFlavor>: Sync);
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
        assert_not_impl_all!(Iter<'_, NotSendNotSync, RcuGuardMemb, true>: Send);
        assert_not_impl_all!(Iter<'_, NotSendNotSync, RcuGuardMemb, true>: Sync);

        // T: Send + !Sync
        assert_not_impl_all!(Iter<'_, SendButNotSync, RcuGuardMemb, true>: Send);
        assert_not_impl_all!(Iter<'_, SendButNotSync, RcuGuardMemb, true>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(Iter<'_, NotSendButSync, RcuGuardMemb, true>: Send);
        assert_not_impl_all!(Iter<'_, NotSendButSync, RcuGuardMemb, true>: Sync);

        // T: Send + Sync
        assert_not_impl_all!(Iter<'_, SendAndSync, RcuGuardMemb, true>: Send);
        assert_not_impl_all!(Iter<'_, SendAndSync, RcuGuardMemb, true>: Sync);
    }

    mod rcu_list_iter_backward {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(Iter<'_, NotSendNotSync, RcuGuardMemb, false>: Send);
        assert_not_impl_all!(Iter<'_, NotSendNotSync, RcuGuardMemb, false>: Sync);

        // T: Send + !Sync
        assert_not_impl_all!(Iter<'_, SendButNotSync, RcuGuardMemb, false>: Send);
        assert_not_impl_all!(Iter<'_, SendButNotSync, RcuGuardMemb, false>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(Iter<'_, NotSendButSync, RcuGuardMemb, false>: Send);
        assert_not_impl_all!(Iter<'_, NotSendButSync, RcuGuardMemb, false>: Sync);

        // T: Send + Sync
        assert_not_impl_all!(Iter<'_, SendAndSync, RcuGuardMemb, false>: Send);
        assert_not_impl_all!(Iter<'_, SendAndSync, RcuGuardMemb, false>: Sync);
    }
}
