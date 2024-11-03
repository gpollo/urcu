use crate::collections::queue::raw::RawNode;
use crate::rcu::reference;

/// An owned RCU reference to a element removed from an [`RcuQueue`].
///
/// [`RcuQueue`]: crate::collections::queue::container::RcuQueue
pub type RefOwned<T> = reference::BoxRefOwned<RawNode<T>>;

/// An RCU reference to a element removed from an [`RcuQueue`].
///
/// [`RcuQueue`]: crate::collections::queue::container::RcuQueue
pub type Ref<T, F> = reference::RcuRefBox<RawNode<T>, F>;
