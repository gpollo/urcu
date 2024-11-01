#![feature(ptr_as_ref_unchecked)]
#![doc = include_str!("../../README.md")]

mod rcu;
mod utility;

pub mod boxed;
pub mod hashmap;
pub mod list;
pub mod queue;
pub mod shared;
pub mod stack;

pub use crate::boxed::container::RcuBox;
pub use crate::hashmap::container::RcuHashMap;
pub use crate::list::container::RcuList;
pub use crate::queue::container::RcuQueue;
pub use crate::rcu::callback::*;
pub use crate::rcu::cleanup::*;
pub use crate::rcu::flavor::DefaultFlavor;
pub use crate::rcu::flavor::RcuFlavor;
pub use crate::rcu::reference::*;
pub use crate::rcu::*;
pub use crate::stack::container::RcuStack;
