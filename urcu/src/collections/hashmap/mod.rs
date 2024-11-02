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
    use crate::rcu::default::RcuDefaultFlavor;
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
        assert_impl_all!(Ref<SendButNotSync, SendButNotSync, RcuDefaultFlavor>: Send);
        assert_not_impl_all!(Ref<SendButNotSync, SendButNotSync, RcuDefaultFlavor>: Sync);

        // T: Send + Sync
        assert_impl_all!(Ref<SendAndSync, SendAndSync, RcuDefaultFlavor>: Send);
        assert_not_impl_all!(Ref<SendAndSync, SendAndSync, RcuDefaultFlavor>: Sync);
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
        assert_not_impl_all!(Iter<'_, NotSendNotSync, NotSendNotSync, RcuDefaultFlavor>: Send);
        assert_not_impl_all!(Iter<'_, NotSendNotSync, NotSendNotSync, RcuDefaultFlavor>: Sync);

        // T: Send + !Sync
        assert_not_impl_all!(Iter<'_, SendButNotSync,  SendButNotSync, RcuDefaultFlavor>: Send);
        assert_not_impl_all!(Iter<'_, SendButNotSync,  SendButNotSync, RcuDefaultFlavor>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(Iter<'_, NotSendButSync, NotSendButSync, RcuDefaultFlavor>: Send);
        assert_not_impl_all!(Iter<'_, NotSendButSync, NotSendButSync, RcuDefaultFlavor>: Sync);

        // T: Send + Sync
        assert_not_impl_all!(Iter<'_, SendAndSync, SendAndSync, RcuDefaultFlavor>: Send);
        assert_not_impl_all!(Iter<'_, SendAndSync, SendAndSync, RcuDefaultFlavor>: Sync);
    }
}
