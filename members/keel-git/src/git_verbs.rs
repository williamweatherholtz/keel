//! The `keel` verbs this member owns (D0479, sprint 750): helpers only.
//!
//! Moved out of `keel-cli/src/main.rs` by `scripts/extract_verbs.py`: the binary is argument dispatch, and a
//! verb body lives with the member whose reach it needs (`python scripts/verb_homes.py --by-member`).
//! Each `cmd_*` takes the arguments after its verb and returns the process exit code.
//
// These were the binary's private items, written under the same lint set; only the lints that fire on
// PUBLIC visibility are new here, and they ask for ceremony a verb body does not owe: its exit code goes
// straight back to `main`, and its `Err` and panics are the ones the body already documents in place.
#![allow(clippy::must_use_candidate, clippy::missing_errors_doc, clippy::missing_panics_doc)]

use crate::projects::find_repo_root;
use std::path::PathBuf;

pub fn resolve_guard_root(arg: Option<&String>) -> Option<PathBuf> {
    arg.map_or_else(find_repo_root, |p| Some(PathBuf::from(p)))
}


/// Say where a pass recorded on a dirty tree will bind (dcResultBindsToItsLandingCommit): the
/// `--sha` names the tree BEFORE the work lands, and every reader of `judgedAgainst` will read the
/// landing commit instead once it exists. stderr, so a caller parsing the uuid is unaffected.
pub fn binding_note(file: &std::path::Path, sha: &str, verdict: &str) {
    if verdict != "pass" {
        return;
    }
    let dir = file.parent().unwrap_or_else(|| std::path::Path::new("."));
    let Ok(out) = crate::gitx::git().arg("-C").arg(dir).args(["status", "--porcelain"]).output() else { return };
    if !out.status.success() {
        return;
    }
    let dirty = String::from_utf8_lossy(&out.stdout).lines().filter(|l| !l.trim().is_empty()).count();
    if dirty > 0 {
        eprintln!(
            "note: {dirty} uncommitted path(s) - this pass names {sha} but binds to the commit that lands it, \
             once {sha} is that commit's ancestor (dcResultBindsToItsLandingCommit)"
        );
    }
}
