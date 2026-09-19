//! The write API (D0479, sprint 718): the one path an authored fact takes into the tree.
//!
//! Depends on the read model and the leaves. Nothing here renders a view or runs a guard; keel-cli
//! re-exports each module under its old `crate::` path so no caller moved when the code did.

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

pub mod write;
pub mod scaffold;
pub mod reverify;
pub mod claim;
pub mod claude_surface;
pub mod ledger;
// The verbs this member owns, out of main.rs (D0479, sprint 750).
pub mod write_verbs;

use std::path::Path;

/// The declared-vs-binary version skew for the project owning `target`, if any (D0251).
///
/// Root discovery mirrors `write::model_lock_path`: walk up to the `.tracking`/`.engine` parent. Returns
/// `None` when there is no declaration (D0136: absence is a state — a pre-D0190 tree keeps working),
/// when the declaration matches, or when no project root is findable (a caller writing outside a
/// model tree has no pin to honour).
#[must_use]
pub fn pin_skew(target: &Path) -> Option<(String, String)> {
    // The target may BE the .tracking dir (set_attr locks on it directly), or live under it.
    let mut cur = target;
    let root = loop {
        if matches!(cur.file_name().and_then(|n| n.to_str()), Some(".tracking" | ".engine")) {
            break cur.parent()?;
        }
        cur = cur.parent()?;
    };
    let text = std::fs::read_to_string(root.join(".engine").join("contracts").join("engine-version.toml")).ok()?;
    let declared = text
        .lines()
        .find_map(|l| l.trim().strip_prefix("engine"))
        .and_then(|r| r.split('"').nth(1))
        .map(str::to_string)?;
    let binary = env!("CARGO_PKG_VERSION");
    (declared != binary).then(|| (declared, binary.to_string()))
}
