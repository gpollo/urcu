[![Latest Version](https://img.shields.io/crates/v/urcu2?logo=rust)](https://crates.io/crates/urcu2)
[![Latest Documentation](https://img.shields.io/docsrs/urcu2?logo=rust)](https://docs.rs/urcu2/latest/urcu/)
[![Pipeline Status](https://img.shields.io/gitlab/pipeline-status/gabrielpolloguilbert%2Furcu?branch=master&logo=gitlab)](https://gitlab.com/gabrielpolloguilbert/urcu/-/pipelines/latest)

This crate provides safe Rust API to [`liburcu`][liburcu] for Linux systems.

# Goals

The goal is to provide traits and primitives where RCU guarantees are always respected.

* Enforce RCU read locks when accessing RCU protected references.
* Enforce RCU syncronization when taking ownership of a RCU reference.
* Enforce memory cleanups in the exposed RCU data structures.

# Warnings

Even though [`liburcu`][liburcu] is well tested and used in many applications, this
crate is still *experimental*. It works well in toy applications and stress tests,
but I cannot guarantee it's bug free. There may be hidden race conditions or type
unsoundness that may lead to undefined behaviors.

# Features

This crate offers optional features. By default, all flavors are included.

* <code>**flavor-bp**</code>: Enable `liburcu-bp` flavor.
* <code>**flavor-mb**</code>: Enable `liburcu-mb` flavor.
* <code>**flavor-memb**</code>: Enable `liburcu-memb` flavor.
* <code>**flavor-qsbr**</code>: Enable `liburcu-qsbr` flavor.
* <code>**static**</code>: Build [`liburcu`][liburcu] and link statically.
  * This feature requires that [`liburcu`][liburcu] build dependencies are installed.
  * Without this feature, you need to install [`liburcu`][liburcu] our your system.

# Types

#### RCU Context

Every thread that does RCU operations needs to be registered. This is enforced through
the [`RcuContext`] trait. Depending on the RCU flavor, the implementator will be different.
In all cases, a context can be created using [`RcuContext::rcu_register`].

Currently, all contexts register the thread for read and defer operations. Even if your
thread don't execute RCU critical sections, it will still be registered for it. It's not
optimal, but it currenly simplifies things. In the future, it will be possible for threads
to register only on features it needs.

#### RCU Guard

When accessing RCU protected data, every data structure will require a RCU read guard.
It is obtained from [`RcuContext::rcu_read_lock`]. The lifetime of the references will
be the same as this guard. That way, the Rust compiler guarantees that the RCU read lock
is taken.

#### RCU Reference

When RCU protected data is removed from a container, it returns a [`RcuRef`]. This trait
defines a RCU protected reference that might still have RCU readers accessing it. To get
ownership of this reference, you need to wait for a RCU grace period. It is enforced by
calling [`RcuRef::take_ownership`]. Dropping a [`RcuRef`] without taking ownership will
still cleanup safely.

# Data Structures

All data structures, except [`RcuBox<T>`], are a wrapper around `liburcu-cds` API. They
all supports RCU read traversal.

| Type                 | Description                                       |
|:---------------------|:--------------------------------------------------|
| [`RcuBox<T>`]        | RCU [`Box<T>`] with wait-free updates.            |
| [`RcuHashMap<K, V>`] | RCU hashmap with lock-free updates.               |
| [`RcuList<T>`]       | RCU linked list with mutual exclusion on updates. |
| [`RcuQueue<T>`]      | RCU queue with lock-free updates.                 |
| [`RcuStack<T>`]      | RCU stack with wait-free updates.                 |

# Example

```rust
// register the current thread for RCU operations
let mut context = RcuContextMemb::rcu_register().unwrap();

// create a RCU queue (could be sent to other threads)
let queue = RcuQueue::<u32>::new();

// push/pop operations requires a RCU critical section
let guard = context.rcu_read_lock();

// push data into the queue
queue.push(Job { ... }, &guard);
queue.push(Job { ... }, &guard);
queue.push(Job { ... }, &guard);

// pop data from the queue
let job = queue.pop(&guard).unwrap();

// exit RCU critical section
drop(guard);

// wait for RCU grace period and get ownership
let mut job = job.take_ownership(&mut context);

// do something with the data
job.execute();
```

# Performance

Althought most of the API should have low-overhead on the existing C library, we
are currently linking [`liburcu`][liburcu] dynamically, meaning that all the inlined
functions are not used. This will have an overhead.

Unlike [`liburcu`][liburcu], we do not expose an [intrusive][intrusive] API to store
data in the data structures. This means you don't have to add a special head node in
your types. Intrusive containers are more efficient. Althought it's feasible, it is
currently not a goal to offer this.

#### Link-Time Optimisation

Performance can be improved by enabling link-time optimization (LTO). To do so, we need
to build [`liburcu`][liburcu] with LTO and link it statically into the final binary.

* Install `clang` compiler.
* Enable feature flag `static`.
* Enable `lto = true` in your build profile.
* Execute Cargo with `RUSTFLAGS="-Clinker-plugin-lto"`.

[liburcu]: https://liburcu.org/
[intrusive]: https://stackoverflow.com/questions/5004162/what-does-it-mean-for-a-data-structure-to-be-intrusive
