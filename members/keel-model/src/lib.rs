//! The read model (D0479, sprint 718): everything that turns the tree into facts and reads them back.
//!
//! Depends on the leaf members only. Nothing here writes the model, runs a guard or renders a view;
//! keel-cli re-exports each module under its old `crate::` path so no caller moved when the code did.

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
// between them and their callers (sprint 718); the two lints below judge a PUBLISHED API's generality
// and doc shape, and this member is a workspace-internal one with a single consumer.
#![allow(clippy::implicit_hasher, clippy::too_long_first_doc_paragraph)]
// Tests may use unwrap/expect/panic/indexing/asserts freely.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing))]

pub mod corpus;
pub mod ident;
pub mod textscan;
pub mod gitfacts;
pub mod indexer;
pub mod done;
pub mod binding;
pub mod evidence;
pub mod fingerprint;
pub mod model;
pub mod queries;
pub mod claims;
pub mod suspect;
pub mod algo;
pub mod activation;
pub mod orient;
pub mod onboard;
pub mod validate;
pub mod resolvers;
pub mod readiness;
// The verbs this member owns, out of main.rs (D0479, sprint 750).
pub mod model_verbs;
