pub(crate) mod container;
pub(crate) mod iterator;
pub(crate) mod raw;
pub(crate) mod reference;

#[cfg(test)]
mod test;

pub use crate::collections::list::iterator::*;
pub use crate::collections::list::reference::*;

mod asserts {
    use super::*;

    use static_assertions::{assert_impl_all, assert_not_impl_all};

    use crate::collections::list::container::RcuList;
    use crate::rcu::default::{RcuDefaultFlavor, RcuDefaultGuard};
    use crate::utility::asserts::*;

    mod rcu_list {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(RcuList<NotSendNotSync, RcuDefaultFlavor>: Send);
        assert_not_impl_all!(RcuList<NotSendNotSync, RcuDefaultFlavor>: Sync);

        // T: Send + !Sync
        assert_impl_all!(RcuList<SendButNotSync, RcuDefaultFlavor>: Send);
        assert_not_impl_all!(RcuList<SendButNotSync, RcuDefaultFlavor>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(RcuList<NotSendButSync, RcuDefaultFlavor>: Send);
        assert_impl_all!(RcuList<NotSendButSync, RcuDefaultFlavor>: Sync);

        // T: Send + Sync
        assert_impl_all!(RcuList<SendAndSync, RcuDefaultFlavor>: Send);
        assert_impl_all!(RcuList<SendAndSync, RcuDefaultFlavor>: Sync);
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
        assert_not_impl_all!(Iter<'_, NotSendNotSync, RcuDefaultGuard, true>: Send);
        assert_not_impl_all!(Iter<'_, NotSendNotSync, RcuDefaultGuard, true>: Sync);

        // T: Send + !Sync
        assert_not_impl_all!(Iter<'_, SendButNotSync, RcuDefaultGuard, true>: Send);
        assert_not_impl_all!(Iter<'_, SendButNotSync, RcuDefaultGuard, true>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(Iter<'_, NotSendButSync, RcuDefaultGuard, true>: Send);
        assert_not_impl_all!(Iter<'_, NotSendButSync, RcuDefaultGuard, true>: Sync);

        // T: Send + Sync
        assert_not_impl_all!(Iter<'_, SendAndSync, RcuDefaultGuard, true>: Send);
        assert_not_impl_all!(Iter<'_, SendAndSync, RcuDefaultGuard, true>: Sync);
    }

    mod rcu_list_iter_backward {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(Iter<'_, NotSendNotSync, RcuDefaultGuard, false>: Send);
        assert_not_impl_all!(Iter<'_, NotSendNotSync, RcuDefaultGuard, false>: Sync);

        // T: Send + !Sync
        assert_not_impl_all!(Iter<'_, SendButNotSync, RcuDefaultGuard, false>: Send);
        assert_not_impl_all!(Iter<'_, SendButNotSync, RcuDefaultGuard, false>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(Iter<'_, NotSendButSync, RcuDefaultGuard, false>: Send);
        assert_not_impl_all!(Iter<'_, NotSendButSync, RcuDefaultGuard, false>: Sync);

        // T: Send + Sync
        assert_not_impl_all!(Iter<'_, SendAndSync, RcuDefaultGuard, false>: Send);
        assert_not_impl_all!(Iter<'_, SendAndSync, RcuDefaultGuard, false>: Sync);
    }
}
