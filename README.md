This crate provides safe Rust API to [`liburcu`][liburcu] for Linux systems.

# Goals

The goal is to provide traits and primitives where RCU guarantees are always respected.

* Enforce RCU read locks when accessing RCU protected references.
* Enforce RCU syncronization when taking ownership of an RCU reference.
* Enforce memory cleanups in the exposed RCU data structures.

# Performance

Althought most of the API should have low-overhead on the existing C library, we
are currently linking [`liburcu`][liburcu] dynamically, meaning that all the inlined
functions are not used. This will have an overhead.

Unlike [`liburcu`][liburcu], we do not expose an [intrusive][intrusive] API to store
data in the data structures. This means you don't have to add a special head node in
your types. Intrusive containers are more efficient. Althought it's not feasible, it
is currently not a goal to offer this.

[liburcu]: https://liburcu.org/
[intrusive]: https://stackoverflow.com/questions/5004162/what-does-it-mean-for-a-data-structure-to-be-intrusive
