use crate::shared::reference;
use crate::stack::raw::RawNode;

/// An owned RCU reference to a element removed from an [`RcuQueue`].
///
/// [`RcuQueue`]: crate::queue::container::RcuQueue
pub type RefOwned<F> = reference::BoxRefOwned<RawNode<F>>;

/// An RCU reference to a element removed from an [`RcuQueue`].
///
/// #### Requirements
///
/// `T` must be [`Send`] because [`Drop::drop`] might execute cleanup in another thread.
///
/// [`RcuQueue`]: crate::queue::container::RcuQueue
pub type Ref<T, F> = reference::BoxRef<RawNode<T>, F>;
