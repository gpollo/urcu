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
    use crate::rcu::DefaultContext;
    use crate::utility::asserts::*;

    mod rcu_queue {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(RcuQueue<NotSendNotSync, DefaultContext>: Send);
        assert_not_impl_all!(RcuQueue<NotSendNotSync, DefaultContext>: Sync);

        // T: Send + !Sync
        assert_impl_all!(RcuQueue<SendButNotSync, DefaultContext>: Send);
        assert_not_impl_all!(RcuQueue<SendButNotSync, DefaultContext>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(RcuQueue<NotSendButSync, DefaultContext>: Send);
        assert_impl_all!(RcuQueue<NotSendButSync, DefaultContext>: Sync);

        // T: Send + Sync
        assert_impl_all!(RcuQueue<SendAndSync, DefaultContext>: Send);
        assert_impl_all!(RcuQueue<SendAndSync, DefaultContext>: Sync);
    }

    mod rcu_queue_ref {
        use super::*;

        // T: Send + !Sync
        assert_impl_all!(Ref<SendButNotSync, DefaultContext>: Send);
        assert_not_impl_all!(Ref<SendButNotSync, DefaultContext>: Sync);

        // T: Send + Sync
        assert_impl_all!(Ref<SendAndSync, DefaultContext>: Send);
        assert_not_impl_all!(Ref<SendAndSync, DefaultContext>: Sync);
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
