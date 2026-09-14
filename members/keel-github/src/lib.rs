//! GitHub gesture parsing and receipts (D0263/D0264) - the reading side of the intake router.
//!
//! A D0479 leaf member: it depends on nothing above it, and keel-cli re-exports each module under
//! its old `crate::` path so no caller moved when the code did (sprint 714).

pub mod github;
