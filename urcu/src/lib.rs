//! This crate provides safe Rust API to [`liburcu`] for Linux systems.
//!
//! # Goals
//!
//! The goal is to provide traits and primitives where RCU guarantees are always respected.
//!
//! * Enforce RCU read locks when accessing RCU protected references.
//! * Enforce RCU syncronization when taking ownership of an RCU reference.
//! * Enforce memory cleanups in the exposed RCU data structures.
//!
//! # Performance
//!
//! Althought most of the API should have low-overhead on the existing C library, we are
//! currently linking [`liburcu`] dynamically, meaning that all the inlined functions are
//! not used. This will have an overhead.
//!
//! Unlike [`liburcu`], we do not expose an [intrusive] API to store data in the data structures.
//! This means you don't have to add a special head node in your types. Intrusive containers are
//! more efficient. Althought it's not feasible, it is currently not a goal to offer this.
//!
//! # Usage
//!
//! Each RCU threads need to create an [`RcuContext`] before using any RCU operations.
//! Upon creation, the context will register the thread to [`liburcu`]. It will also
//! unregister the thread when [`Drop::drop`] is executed[^writers].
//!
//! ##### Reading
//!
//! When accessing an RCU data structure for reading, the RCU read lock must be taken with
//! [`RcuReader::rcu_read_lock`]. The resulting lock guard can be used to get references to
//! RCU protected data.
//!
//! ##### Writing
//!
//! When accessing an RCU data structure for writing, the thread don't need any RCU-specific
//! locking. If the thread removes an element, an [`RcuRef`] is returned. This trait means that
//! the thread do not have ownership of the data and some readers might still have access to it.
//!
//! * If the writer wants to take ownership, it can use [`rcu_take_ownership!`]. It will execute
//!   an RCU syncronization, making sure there are not readers left.
//! * If the writer never takes ownership, cleanup will be executed later using [`call_rcu`] on
//!   a helper thread that is automatically started[^helpers].
//!
//! [^writers]: Currently, the writing threads are also registered to [`liburcu`] even thought
//!             it's not needed. It is a goal to eventually offer this no registration.
//! [^helpers]: Currently, we do not offer an API to configure the helper threads. The default
//!             helper thread is always used. It is a goal to eventually offer an API.
//!
//! [`liburcu`]: https://liburcu.org/
//! [`call_rcu`]: https://github.com/urcu/userspace-rcu?tab=readme-ov-file#usage-of-urcu-call-rcu
//! [intrusive]: https://stackoverflow.com/questions/5004162/what-does-it-mean-for-a-data-structure-to-be-intrusive

mod rcu;

// pub mod hash_map;
pub mod linked_list;

pub use crate::rcu::callback::*;
pub use crate::rcu::context::*;
pub use crate::rcu::flavor::*;
pub use crate::rcu::reference::*;
pub use crate::rcu::*;
