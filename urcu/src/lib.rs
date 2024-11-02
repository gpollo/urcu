#![feature(ptr_as_ref_unchecked)]
#![doc = include_str!("../../README.md")]

mod utility;

pub mod collections;
pub mod rcu;

pub use crate::collections::boxed::container::RcuBox;
pub use crate::collections::hashmap::container::RcuHashMap;
pub use crate::collections::list::container::RcuList;
pub use crate::collections::queue::container::RcuQueue;
pub use crate::collections::stack::container::RcuStack;
pub use crate::rcu::callback::*;
pub use crate::rcu::cleanup::*;
pub use crate::rcu::context::{DefaultContext, RcuContext, RcuDeferContext, RcuReadContext};
pub use crate::rcu::flavor::DefaultFlavor;
pub use crate::rcu::flavor::RcuFlavor;
pub use crate::rcu::guard::RcuGuard;
pub use crate::rcu::poller::RcuPoller;
pub use crate::rcu::reference::{RcuBoxRef, RcuRef};
pub use crate::rcu::{rcu_dereference, rcu_dereference_mut};

pub mod prelude {
    pub use crate::{RcuFlavor, RcuGuard, RcuPoller, RcuRef};

    pub use crate::{RcuContext, RcuDeferContext, RcuReadContext};

    pub use crate::{RcuBox, RcuHashMap, RcuList, RcuQueue, RcuStack};

    pub use crate::{DefaultContext, DefaultFlavor};
}
