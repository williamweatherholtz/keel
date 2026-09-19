//! keel-serve: the console - the localhost server (D0094), the obligation deck, the run launcher, the
//! console registry, the scorecard reports and the attestation sampling.
//!
//! The eighth D0479 extraction (sprint 738): six modules out of keel-cli/src, moved by
//! `scripts/extract_serve.py`, with the page the server embeds. keel-cli re-exports each at its old path
//! (`crate::serve`, `crate::deck`, `crate::launcher`, `crate::console_registry`, `crate::reports`,
//! `crate::attestation`), so no caller moved. The crate sits above every member and below keel-cli: it reads
//! the view layer, the guards, the write API, the processes and the item views, and nothing reads it but
//! the binary - so an edit to `serve.rs` rebuilds this crate and keel-cli only.
#![forbid(unsafe_code)]
#![deny(warnings, clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, clippy::todo, clippy::unimplemented)]
#![allow(clippy::implicit_hasher, clippy::too_long_first_doc_paragraph, clippy::module_name_repetitions)]
// Tests may use unwrap/expect/panic/indexing/asserts freely.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing))]

pub mod attestation;
pub mod console_registry;
pub mod deck;
pub mod launcher;
pub mod reports;
pub mod serve;
// The verbs this member owns, out of main.rs (D0479, sprint 750).
pub mod serve_verbs;
