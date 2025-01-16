pub(crate) mod container;
pub(crate) mod iterator;
pub(crate) mod raw;
pub(crate) mod reference;

#[cfg(test)]
mod test;

pub use crate::collections::stack::iterator::*;
pub use crate::collections::stack::reference::*;

mod asserts {
    use super::*;

    use static_assertions::{assert_impl_all, assert_not_impl_all};

    use crate::collections::stack::container::RcuStack;
    use crate::rcu::default::{RcuDefaultFlavor, RcuDefaultGuard};
    use crate::utility::asserts::*;

    mod rcu_list {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(RcuStack<NotSendNotSync, RcuDefaultFlavor>: Send);
        assert_not_impl_all!(RcuStack<NotSendNotSync, RcuDefaultFlavor>: Sync);

        // T: Send + !Sync
        assert_impl_all!(RcuStack<SendButNotSync, RcuDefaultFlavor>: Send);
        assert_not_impl_all!(RcuStack<SendButNotSync, RcuDefaultFlavor>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(RcuStack<NotSendButSync, RcuDefaultFlavor>: Send);
        assert_impl_all!(RcuStack<NotSendButSync, RcuDefaultFlavor>: Sync);

        // T: Send + Sync
        assert_impl_all!(RcuStack<SendAndSync, RcuDefaultFlavor>: Send);
        assert_impl_all!(RcuStack<SendAndSync, RcuDefaultFlavor>: Sync);
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
        assert_not_impl_all!(Iter<'_, NotSendNotSync, RcuDefaultGuard>: Send);
        assert_not_impl_all!(Iter<'_, NotSendNotSync, RcuDefaultGuard>: Sync);

        // T: Send + !Sync
        assert_not_impl_all!(Iter<'_, SendButNotSync, RcuDefaultGuard>: Send);
        assert_not_impl_all!(Iter<'_, SendButNotSync, RcuDefaultGuard>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(Iter<'_, NotSendButSync, RcuDefaultGuard>: Send);
        assert_not_impl_all!(Iter<'_, NotSendButSync, RcuDefaultGuard>: Sync);

        // T: Send + Sync
        assert_not_impl_all!(Iter<'_, SendAndSync, RcuDefaultGuard>: Send);
        assert_not_impl_all!(Iter<'_, SendAndSync, RcuDefaultGuard>: Sync);
    }

    mod rcu_list_iter_ref {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(IterRef<NotSendNotSync, RcuDefaultFlavor>: Send);
        assert_not_impl_all!(IterRef<NotSendNotSync, RcuDefaultFlavor>: Sync);

        // T: Send + !Sync
        assert_not_impl_all!(IterRef<SendButNotSync, RcuDefaultFlavor>: Send);
        assert_not_impl_all!(IterRef<SendButNotSync, RcuDefaultFlavor>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(IterRef<NotSendButSync, RcuDefaultFlavor>: Send);
        assert_not_impl_all!(IterRef<NotSendButSync, RcuDefaultFlavor>: Sync);

        // T: Send + Sync
        assert_not_impl_all!(IterRef<SendAndSync, RcuDefaultFlavor>: Send);
        assert_not_impl_all!(IterRef<SendAndSync, RcuDefaultFlavor>: Sync);
    }
}
