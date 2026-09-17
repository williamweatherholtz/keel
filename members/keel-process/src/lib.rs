//! keel-process: the governance processes - the process cursor, activation and adoption, the unit library,
//! the migrations, the workspace gate, sync and land, status and the unattended currency pass.
//!
//! The sixth D0479 extraction (sprint 736): `process_cmd`, `library`, `adoption_check`, `migrate`,
//! `workspace`, `sync`, `status` and `currency` out of keel-cli, moved by `scripts/extract_process.py`.
//! keel-cli re-exports every module at its old path, so no caller moved. The crate sits above the guards
//! and the build-and-test tooling because these processes RUN them (`keel gate --workspace`, `keel show
//! status`, the touched set before a land); it depends on nothing that serves or ingests, so an edit to
//! serve.rs or the GitHub intake leaves it Fresh.
#![forbid(unsafe_code)]
#![deny(warnings, clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, clippy::todo, clippy::unimplemented)]
#![allow(clippy::implicit_hasher, clippy::too_long_first_doc_paragraph, clippy::module_name_repetitions)]
// Tests may use unwrap/expect/panic/indexing/asserts freely.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing))]

pub mod adoption_check;
pub mod currency;
pub mod library;
pub mod migrate;
pub mod process_cmd;
pub mod status;
pub mod sync;
pub mod workspace;
