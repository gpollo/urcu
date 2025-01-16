[![Repository](https://img.shields.io/badge/Repository-Gitlab-blue?style=for-the-badge&logo=gitlab
)](https://gitlab.com/gpollo/urcu)
[![Latest Version](https://img.shields.io/crates/v/urcu2-cds-sys?style=for-the-badge&logo=rust)](https://crates.io/crates/urcu2-cds-sys)
[![Latest Documentation](https://img.shields.io/docsrs/urcu2-cds-sys?style=for-the-badge&logo=rust)](https://docs.rs/urcu2-cds-sys/latest/urcu_cds_sys/)

# Userspace RCU

This crate provides unsafe Rust API to [`liburcu-cds`][liburcu] for Linux systems.

This crate should not be used directly, please use the [`urcu2`] crate.

[liburcu]: https://liburcu.org/
[`urcu2`]: https://crates.io/crates/urcu2