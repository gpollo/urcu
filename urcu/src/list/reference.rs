use crate::list::raw::RawNode;
use crate::shared::reference;

/// An owned RCU reference to a element removed from an [`RcuList`].
///
/// [`RcuList`]: crate::list::container::RcuList
pub type RefOwned<T> = reference::BoxRefOwned<RawNode<T>>;

/// An RCU reference to a element removed from an [`RcuList`].
///
/// #### Requirements
///
/// `T` must be [`Send`] because [`Drop::drop`] might execute cleanup in another thread.
///
/// [`RcuList`]: crate::list::container::RcuList
pub type Ref<T, C> = reference::BoxRef<RawNode<T>, C>;
