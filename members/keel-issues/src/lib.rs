//! keel-issues: the item and issue processes - recording an issue, a statement or a story, the GitHub
//! intake, and the open-issues, dispositions and intake views (D0480).
//!
//! The seventh D0479 extraction (sprint 737): `github_ingest` out of keel-cli, `intake_write` out of
//! keel-write, the issue write sliced from write.rs and the three item views sliced from keel-view, moved
//! by `scripts/extract_issues.py`. keel-cli re-exports every name at its old path (`crate::view::open_issues`,
//! `crate::write::record_issue`, `crate::intake_write`, `crate::github_ingest`), so no caller moved. The
//! crate depends on the read model, the write API, the schema and the JSON leaf - on no view, no guard,
//! no process and nothing that serves - so a project that only tracks items can depend on it without the
//! engine-governance stack, and an edit to `process_cmd.rs` or `serve.rs` leaves it Fresh (D0508).
#![forbid(unsafe_code)]
#![deny(warnings, clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, clippy::todo, clippy::unimplemented)]
#![allow(clippy::implicit_hasher, clippy::too_long_first_doc_paragraph, clippy::module_name_repetitions)]
// Tests may use unwrap/expect/panic/indexing/asserts freely.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing))]

pub mod github_ingest;
pub mod intake_write;
pub mod issue_write;
pub mod views;
// The verbs this member owns, out of main.rs (D0479, sprint 750).
pub mod issues_verbs;

/// `keel record task` adds a `DoD` `Test` to a declared action - an item verb, reachable here by name. The
/// write itself stays keel-write's: it reads eight of write.rs's private insertion helpers, which
/// `append_result` shares, and a fact lives where its readers are (D0508).
pub use keel_write::write::add_task;
