//! The done-set: an action is DONE when its `DoD`'s latest result passes at a commit git resolves.
//!
//! One authority, below orient and the views (D0479, dcGuardsViewOrientCycleIsBroken): orient's frontier,
//! the issue-resolution view (D0077), coverage and the reports all read this and never each other.

use std::collections::HashSet;
use std::path::Path;

/// Done action names: latest `DoD` result passes with a resolvable `judgedAgainst`.
///
/// Exposed as the single done-set authority for the issue-resolution view (D0077): a `#Resolves`
/// action resolver is "complete" iff it is in this set.
#[must_use]
pub fn done_names(root: &Path) -> HashSet<String> {
    let idx = crate::indexer::extract(&root.join(".tracking"));
    // Validate ALL distinct passing-result SHAs in ONE batched `git cat-file` spawn — was one
    // `git_sha_valid` spawn PER task (~N process spawns), which dominated `assured`/`report
    // assurance` cold time (~14s of N≈120 spawns on Windows). Same semantics as `compute` (which
    // already batches): empty SHA → done; otherwise commit-validity, conservative-true on git failure.
    let mut shas: Vec<String> = idx
        .tasks
        .values()
        .filter_map(|d| d.results.last())
        .filter(|r| r.outcome == "pass" && !r.judged_against.is_empty())
        .map(|r| r.judged_against.clone())
        .collect();
    shas.sort();
    shas.dedup();
    let sha_valid = crate::gitfacts::valid_commits(root, &shas);
    let mut done = HashSet::new();
    for (name, data) in &idx.tasks {
        if let Some(latest) = data.results.last() {
            if latest.outcome == "pass"
                && (latest.judged_against.is_empty() || sha_valid.get(&latest.judged_against).copied().unwrap_or(true))
            {
                done.insert(name.clone());
            }
        }
    }
    done
}
