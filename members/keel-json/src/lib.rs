//! Text encodings the surfaces share: the Python-compatible JSON writer and the terminal colour palette.
//!
//! A D0479 leaf member: it depends on nothing above it, and keel-cli re-exports each module under
//! its old `crate::` path so no caller moved when the code did (sprint 714).

pub mod json;
pub mod color;
