pub(crate) mod container;
pub(crate) mod raw;
pub(crate) mod reference;

#[cfg(test)]
mod test;

pub use crate::collections::queue::reference::*;

mod asserts {
    use super::*;

    use static_assertions::{assert_impl_all, assert_not_impl_all};

    use crate::collections::queue::container::RcuQueue;
    use crate::rcu::default::RcuDefaultFlavor;
    use crate::utility::asserts::*;

    mod rcu_queue {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(RcuQueue<NotSendNotSync, RcuDefaultFlavor>: Send);
        assert_not_impl_all!(RcuQueue<NotSendNotSync, RcuDefaultFlavor>: Sync);

        // T: Send + !Sync
        assert_impl_all!(RcuQueue<SendButNotSync, RcuDefaultFlavor>: Send);
        assert_not_impl_all!(RcuQueue<SendButNotSync, RcuDefaultFlavor>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(RcuQueue<NotSendButSync, RcuDefaultFlavor>: Send);
        assert_impl_all!(RcuQueue<NotSendButSync, RcuDefaultFlavor>: Sync);

        // T: Send + Sync
        assert_impl_all!(RcuQueue<SendAndSync, RcuDefaultFlavor>: Send);
        assert_impl_all!(RcuQueue<SendAndSync, RcuDefaultFlavor>: Sync);
    }

    mod rcu_queue_ref {
        use super::*;

        // T: Send + !Sync
        assert_impl_all!(Ref<SendButNotSync, RcuDefaultFlavor>: Send);
        assert_not_impl_all!(Ref<SendButNotSync, RcuDefaultFlavor>: Sync);

        // T: Send + Sync
        assert_impl_all!(Ref<SendAndSync, RcuDefaultFlavor>: Send);
        assert_not_impl_all!(Ref<SendAndSync, RcuDefaultFlavor>: Sync);
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
