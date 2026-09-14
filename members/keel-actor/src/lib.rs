//! Who is acting: actor resolution and enrolment (D0129), and the device-bound gesture receipts (D0201).
//!
//! A D0479 leaf member: it depends on nothing above it, and keel-cli re-exports each module under
//! its old `crate::` path so no caller moved when the code did (sprint 714).

pub mod actor;
pub mod device;
