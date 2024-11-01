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

    use crate::rcu::context::RcuGuardMemb;
    use crate::rcu::flavor::DefaultFlavor;
    use crate::stack::container::RcuStack;
    use crate::utility::asserts::*;

    mod rcu_list {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(RcuStack<NotSendNotSync, DefaultFlavor>: Send);
        assert_not_impl_all!(RcuStack<NotSendNotSync, DefaultFlavor>: Sync);

        // T: Send + !Sync
        assert_impl_all!(RcuStack<SendButNotSync, DefaultFlavor>: Send);
        assert_not_impl_all!(RcuStack<SendButNotSync, DefaultFlavor>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(RcuStack<NotSendButSync, DefaultFlavor>: Send);
        assert_impl_all!(RcuStack<NotSendButSync, DefaultFlavor>: Sync);

        // T: Send + Sync
        assert_impl_all!(RcuStack<SendAndSync, DefaultFlavor>: Send);
        assert_impl_all!(RcuStack<SendAndSync, DefaultFlavor>: Sync);
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
        assert_not_impl_all!(Iter<'_, NotSendNotSync, RcuGuardMemb>: Send);
        assert_not_impl_all!(Iter<'_, NotSendNotSync, RcuGuardMemb>: Sync);

        // T: Send + !Sync
        assert_not_impl_all!(Iter<'_, SendButNotSync, RcuGuardMemb>: Send);
        assert_not_impl_all!(Iter<'_, SendButNotSync, RcuGuardMemb>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(Iter<'_, NotSendButSync, RcuGuardMemb>: Send);
        assert_not_impl_all!(Iter<'_, NotSendButSync, RcuGuardMemb>: Sync);

        // T: Send + Sync
        assert_not_impl_all!(Iter<'_, SendAndSync, RcuGuardMemb>: Send);
        assert_not_impl_all!(Iter<'_, SendAndSync, RcuGuardMemb>: Sync);
    }

    mod rcu_list_iter_ref {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(IterRef<NotSendNotSync, DefaultFlavor>: Send);
        assert_not_impl_all!(IterRef<NotSendNotSync, DefaultFlavor>: Sync);

        // T: Send + !Sync
        assert_not_impl_all!(IterRef<SendButNotSync, DefaultFlavor>: Send);
        assert_not_impl_all!(IterRef<SendButNotSync, DefaultFlavor>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(IterRef<NotSendButSync, DefaultFlavor>: Send);
        assert_not_impl_all!(IterRef<NotSendButSync, DefaultFlavor>: Sync);

        // T: Send + Sync
        assert_not_impl_all!(IterRef<SendAndSync, DefaultFlavor>: Send);
        assert_not_impl_all!(IterRef<SendAndSync, DefaultFlavor>: Sync);
    }
}
