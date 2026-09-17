//! The view layer (D0479, sprint 732): every computed lens over the model.
//!
//! `view` and its ten submodules (census, checks, `control_structure`, critique, delta, flow, knowledge,
//! reports, staleness, `stpa_diagram`), the governance queries (`govern`), the flow metrics (`pm`), the
//! architecture drift (`arch`) and the control-proof census (`control_proof`). Depends on the read model,
//! the write API's declared refusal tables and the leaves; nothing here runs a guard, reads a receipt
//! or composes orient - those are keel-cli's, which passes what the burndown needs down as
//! [`view::BurndownExtras`]. keel-cli re-exports each module under its old `crate::` path so no caller
//! moved when the code did.

#![forbid(unsafe_code)]
#![deny(warnings, clippy::all, clippy::pedantic, clippy::nursery)]
// D0074 fail-loud: authority-bearing code has no silent failure paths.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented
)]
// These items were `pub(crate)` inside keel-cli and became `pub` only because the crate boundary moved
// between them and their callers (sprint 732); the two lints below judge a PUBLISHED API's generality
// and doc shape, and this member is a workspace-internal one with a single consumer.
#![allow(clippy::implicit_hasher, clippy::too_long_first_doc_paragraph)]
// Tests may use unwrap/expect/panic/indexing/asserts freely.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing))]

pub mod arch;
pub mod control_proof;
pub mod govern;
pub mod pm;
pub mod priority;
pub mod view;
