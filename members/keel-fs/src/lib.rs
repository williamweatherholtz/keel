//! The atomic file write every keel writer passes through (issue184 / issue366).
//!
//! A D0479 leaf member: it depends on the parser alone, so the read model's fact cache and the write
//! API share one choke point without either depending on the other (sprint 718).

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
// Tests may use unwrap/expect/panic/indexing/asserts freely.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing))]

pub mod fsx;
pub use fsx::scratch;
