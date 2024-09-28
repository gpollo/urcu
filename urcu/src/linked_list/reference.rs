use crate::linked_list::raw::Node;
use crate::shared::reference;

/// An owned RCU reference to a element removed from an [`RcuList`].
///
/// [`RcuList`]: crate::linked_list::container::RcuList
pub type RefOwned<T> = reference::BoxRefOwned<Node<T>>;

/// An RCU reference to a element removed from an [`RcuList`].
///
/// #### Requirements
///
/// `T` must be [`Send`] because [`Drop::drop`] might execute cleanup in another thread.
///
/// [`RcuList`]: crate::linked_list::container::RcuList
pub type Ref<T, C> = reference::BoxRef<Node<T>, C>;
