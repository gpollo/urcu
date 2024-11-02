pub(crate) mod container;
pub(crate) mod iterator;
pub(crate) mod raw;
pub(crate) mod reference;

#[cfg(test)]
mod test;

pub use crate::collections::hashmap::iterator::*;
pub use crate::collections::hashmap::reference::*;

mod asserts {
    use super::*;

    use static_assertions::{assert_impl_all, assert_not_impl_all};

    use crate::collections::hashmap::container::RcuHashMap;
    use crate::flavor::DefaultFlavor;
    use crate::utility::asserts::*;

    mod rcu_hashmap {
        use super::*;

        // T: Send + Sync
        assert_impl_all!(RcuHashMap<SendAndSync, SendAndSync>: Send);
        assert_impl_all!(RcuHashMap<SendAndSync, SendAndSync>: Sync);
    }

    mod rcu_hashmap_ref {
        use super::*;

        // T: Send + !Sync
        assert_impl_all!(Ref<SendButNotSync, SendButNotSync, DefaultFlavor>: Send);
        assert_not_impl_all!(Ref<SendButNotSync, SendButNotSync, DefaultFlavor>: Sync);

        // T: Send + Sync
        assert_impl_all!(Ref<SendAndSync, SendAndSync, DefaultFlavor>: Send);
        assert_not_impl_all!(Ref<SendAndSync, SendAndSync, DefaultFlavor>: Sync);
    }

    mod rcu_hashmap_ref_owned {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(RefOwned<NotSendNotSync, NotSendNotSync>: Send);
        assert_not_impl_all!(RefOwned<NotSendNotSync, NotSendNotSync>: Sync);

        // T: Send + !Sync
        assert_impl_all!(RefOwned<SendButNotSync, SendButNotSync>: Send);
        assert_not_impl_all!(RefOwned<SendButNotSync, SendButNotSync>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(RefOwned<NotSendButSync, NotSendButSync>: Send);
        assert_impl_all!(RefOwned<NotSendButSync, NotSendButSync>: Sync);

        // T: Send + Sync
        assert_impl_all!(RefOwned<SendAndSync, SendAndSync>: Send);
        assert_impl_all!(RefOwned<SendAndSync, SendAndSync>: Sync);
    }

    mod rcu_hashmap_iter {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(Iter<'_, NotSendNotSync, NotSendNotSync, DefaultFlavor>: Send);
        assert_not_impl_all!(Iter<'_, NotSendNotSync, NotSendNotSync, DefaultFlavor>: Sync);

        // T: Send + !Sync
        assert_not_impl_all!(Iter<'_, SendButNotSync,  SendButNotSync, DefaultFlavor>: Send);
        assert_not_impl_all!(Iter<'_, SendButNotSync,  SendButNotSync, DefaultFlavor>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(Iter<'_, NotSendButSync, NotSendButSync, DefaultFlavor>: Send);
        assert_not_impl_all!(Iter<'_, NotSendButSync, NotSendButSync, DefaultFlavor>: Sync);

        // T: Send + Sync
        assert_not_impl_all!(Iter<'_, SendAndSync, SendAndSync, DefaultFlavor>: Send);
        assert_not_impl_all!(Iter<'_, SendAndSync, SendAndSync, DefaultFlavor>: Sync);
    }
}
