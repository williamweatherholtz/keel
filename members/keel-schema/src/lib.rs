//! The engine's baked-in facts: the embedded .engine tree, the schema vocabulary, the authored CLI surface and the control-defect vocabulary.
//!
//! A D0479 leaf member: it depends on nothing above it, and keel-cli re-exports each module under
//! its old `crate::` path so no caller moved when the code did (sprint 714).

pub mod schema;
pub mod cli_facts;
pub mod cli_surface;
pub mod control_defects;
pub mod embedded;
pub mod guard_names;
