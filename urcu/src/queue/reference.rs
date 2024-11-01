use crate::queue::raw::RawNode;
use crate::shared::reference;

/// An owned RCU reference to a element removed from an [`RcuQueue`].
///
/// [`RcuQueue`]: crate::queue::container::RcuQueue
pub type RefOwned<T> = reference::BoxRefOwned<RawNode<T>>;

/// An RCU reference to a element removed from an [`RcuQueue`].
///
/// [`RcuQueue`]: crate::queue::container::RcuQueue
pub type Ref<T, F> = reference::BoxRef<RawNode<T>, F>;
