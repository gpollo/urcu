pub(crate) mod container;
pub(crate) mod raw;
pub(crate) mod reference;

#[cfg(test)]
mod test;

pub use crate::queue::reference::*;

mod asserts {
    use super::*;

    use static_assertions::{assert_impl_all, assert_not_impl_all};

    use crate::queue::container::RcuQueue;
    use crate::rcu::flavor::DefaultFlavor;
    use crate::utility::asserts::*;

    mod rcu_queue {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(RcuQueue<NotSendNotSync, DefaultFlavor>: Send);
        assert_not_impl_all!(RcuQueue<NotSendNotSync, DefaultFlavor>: Sync);

        // T: Send + !Sync
        assert_impl_all!(RcuQueue<SendButNotSync, DefaultFlavor>: Send);
        assert_not_impl_all!(RcuQueue<SendButNotSync, DefaultFlavor>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(RcuQueue<NotSendButSync, DefaultFlavor>: Send);
        assert_impl_all!(RcuQueue<NotSendButSync, DefaultFlavor>: Sync);

        // T: Send + Sync
        assert_impl_all!(RcuQueue<SendAndSync, DefaultFlavor>: Send);
        assert_impl_all!(RcuQueue<SendAndSync, DefaultFlavor>: Sync);
    }

    mod rcu_queue_ref {
        use super::*;

        // T: Send + !Sync
        assert_impl_all!(Ref<SendButNotSync, DefaultFlavor>: Send);
        assert_not_impl_all!(Ref<SendButNotSync, DefaultFlavor>: Sync);

        // T: Send + Sync
        assert_impl_all!(Ref<SendAndSync, DefaultFlavor>: Send);
        assert_not_impl_all!(Ref<SendAndSync, DefaultFlavor>: Sync);
    }

    mod rcu_queue_ref_owned {
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
}
