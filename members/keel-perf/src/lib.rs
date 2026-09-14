//! Process-wide performance counters (KEEL_PERF) - the one leaf every other member may count into.
//!
//! A D0479 leaf member: it depends on nothing above it, and keel-cli re-exports each module under
//! its old `crate::` path so no caller moved when the code did (sprint 714).

pub mod perf;
