use crate::collections::stack::raw::RawNode;
use crate::rcu::reference;

/// An owned RCU reference to a element removed from an [`RcuQueue`].
///
/// [`RcuQueue`]: crate::collections::queue::container::RcuQueue
pub type RefOwned<F> = reference::BoxRefOwned<RawNode<F>>;

/// An RCU reference to a element removed from an [`RcuQueue`].
///
/// #### Requirements
///
/// `T` must be [`Send`] because [`Drop::drop`] might execute cleanup in another thread.
///
/// [`RcuQueue`]: crate::collections::queue::container::RcuQueue
pub type Ref<T, F> = reference::RcuBoxRef<RawNode<T>, F>;
