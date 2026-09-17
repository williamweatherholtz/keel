//! keel-suite: the build-and-test tooling - the full suite, the touched set, the pre-commit ladder and the
//! hook-binary refresh, each writing the receipt its command is judged by.
//!
//! The fifth D0479 extraction (sprint 735): `suite`, `touched`, `verify` and `hook_binary` out of keel-cli
//! and `contentkey` out of the guard member, moved by `scripts/extract_suite.py`. keel-cli re-exports every
//! module at its old path, so no caller moved. The crate depends on the leaves and the read model only -
//! not on the views or the guards - so an edit to either leaves it Fresh and the ladder binary stable.
#![forbid(unsafe_code)]
#![deny(warnings, clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, clippy::todo, clippy::unimplemented)]
#![allow(clippy::implicit_hasher, clippy::too_long_first_doc_paragraph, clippy::module_name_repetitions)]
// Tests may use unwrap/expect/panic/indexing/asserts freely.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing))]

pub mod contentkey;
pub mod hook_binary;
pub mod suite;
pub mod touched;
pub mod verify;
