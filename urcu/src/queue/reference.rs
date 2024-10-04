use crate::queue::raw::RawNode;
use crate::shared::reference;

pub type RefOwned<T> = reference::BoxRefOwned<RawNode<T>>;
pub type Ref<T, C> = reference::BoxRef<RawNode<T>, C>;
