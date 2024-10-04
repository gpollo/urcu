#![feature(ptr_as_ref_unchecked)]

mod rcu;
mod utility;

pub mod boxed;
pub mod list;
pub mod shared;

pub use crate::boxed::container::RcuBox;
pub use crate::list::container::RcuList;
pub use crate::rcu::api::RcuUnsafe;
pub use crate::rcu::callback::*;
pub use crate::rcu::cleanup::*;
pub use crate::rcu::reference::*;
pub use crate::rcu::*;
