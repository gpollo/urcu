use crate::collections::list::raw::RawNode;
use crate::shared::reference;

/// An owned RCU reference to a element removed from an [`RcuList`].
///
/// [`RcuList`]: crate::collections::list::container::RcuList
pub type RefOwned<T> = reference::BoxRefOwned<RawNode<T>>;

/// An RCU reference to a element removed from an [`RcuList`].
///
/// #### Requirements
///
/// `T` must be [`Send`] because [`Drop::drop`] might execute cleanup in another thread.
///
/// [`RcuList`]: crate::collections::list::container::RcuList
pub type Ref<T, F> = reference::BoxRef<RawNode<T>, F>;
