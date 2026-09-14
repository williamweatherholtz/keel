//! Evidence classification: which recorded passes COUNT, from this clone.
//!
//! Step 1 of an orient run. A task is done when its latest `DoD` result passes at a commit this clone
//! can resolve; an unresolvable anchor is INVALID only when the clone is in a position to judge
//! (it has just fetched, or has no upstream), and UNSYNCHRONIZED otherwise (D0129). Below orient and
//! the views (D0479, dcGuardsViewOrientCycleIsBroken) so the suspect walk can be read without the frontier.

use std::collections::HashMap;
use std::path::Path;

use crate::indexer::TaskData;

/// The classes step 1 sorts every task into, plus the sync state that decided the ambiguous ones.
pub struct Evidence {
    /// Task -> done. Every task has an entry.
    pub done_map: HashMap<String, bool>,
    /// Task -> the commit its pass binds to (the landing commit when the anchor is its ancestor).
    pub verified_at: HashMap<String, String>,
    /// Passes whose anchor this clone resolved as dangling. Unsorted.
    pub invalid_evidence: Vec<String>,
    /// Passes whose anchor is unresolvable HERE while the clone cannot judge. Unsorted.
    pub unsynchronized_evidence: Vec<String>,
    pub sync_state: crate::sync::Divergence,
}

/// `fetched`: the caller has JUST fetched (`keel sync`). `evidence`: the caller reads `verified_at`
/// and the evidence classes; when false and the clone cannot judge anyway, the SHA validation and the
/// landing-commit binding are skipped because neither can change `done_map` (issue439).
#[must_use]
pub(crate) fn classify(repo: &Path, tasks: &HashMap<String, TaskData>, fetched: bool, evidence: bool) -> Evidence {
    // Step 1: compute done/invalid-evidence/verified-at.
    let mut done_map: HashMap<String, bool> = HashMap::new();
    let mut verified_at: HashMap<String, String> = HashMap::new();
    let mut invalid_evidence: Vec<String> = Vec::new();
    let mut unsynchronized_evidence: Vec<String> = Vec::new();

    // Under PARTIAL SYNCHRONIZATION — the normal state of a distributed team — an anchor may be
    // missing from THIS clone simply because it has not been fetched. Collapsing that into
    // `invalidEvidence` treats finished work as not-done, so it re-enters the ready frontier and a
    // second contributor redoes it (D0129 srDcUnresolvedEvidenceClass).
    //
    // The distinguishing question is not about the anchor, it is about the CLONE: if this clone is
    // current with its upstream, an unresolvable anchor is genuinely dangling; if it is behind, or
    // the sync state cannot be read at all, then this clone is not in a position to judge. That is
    // the honest-state distinction D0098 asks for — unverifiable-FROM-HERE is a different fact from
    // unverified, and only the second is a burndown item.
    //
    // WHAT "CURRENT" CANNOT MEAN HERE. The obvious predicate — `behind == 0` — is WRONG, and the
    // fixture proved it: `behind` is measured against the remote-tracking ref, which is only as
    // fresh as the last fetch, so a clone that has never fetched reports `behind 0` while being
    // arbitrarily stale. It would then confidently declare another contributor's evidence INVALID.
    //
    // So the honest predicate is about whether this run actually LOOKED: an unresolvable anchor can
    // be called genuinely dangling only when a fetch has just confirmed there is nothing to find, or
    // when there is no upstream at all and therefore nothing that could be fetched. `orient` never
    // fetches — it runs constantly, and a view that silently performs network I/O is a view people
    // stop running — so from `orient` the answer is "unverifiable from here", with the remedy named.
    // `keel sync` fetches first and passes `fetched = true`, which is where the issue071 protection
    // against a truly orphaned anchor lands.
    let sync_state = crate::perf::phase("frontier:divergence", || crate::sync::divergence(repo));
    let no_upstream = sync_state.unknown.is_some();
    let clone_can_judge = fetched || no_upstream;

    // Validate ALL distinct passing-result SHAs in one batched git spawn (orientPerf/sr11).
    let mut shas: Vec<String> = tasks
        .values()
        .filter_map(|d| d.results.last())
        .filter(|r| r.outcome == "pass" && !r.judged_against.is_empty())
        .map(|r| r.judged_against.clone())
        .collect();
    shas.sort();
    shas.dedup();
    // With nothing to judge and no caller reading `verified_at`, every anchor is taken as it stands
    // (`unwrap_or(true)` below) - the same `done` the validated path reaches when `clone_can_judge` is false.
    let sha_valid = if evidence || clone_can_judge {
        crate::perf::phase("frontier:valid-commits", || crate::gitfacts::valid_commits(repo, &shas))
    } else {
        HashMap::new()
    };
    // A pass recorded at HEAD while the work sat uncommitted names the commit BEFORE the one that
    // carries the work; the binding is the commit that INTRODUCED the result when `judgedAgainst` is
    // its ancestor (dcResultBindsToItsLandingCommit). Cached after one history walk, so this is a
    // lookup on every run after the first.
    let asks: Vec<crate::binding::Ask<'_>> = tasks
        .values()
        .filter_map(|d| d.results.last())
        .filter(|r| r.outcome == "pass" && !r.judged_against.is_empty() && !r.id.is_empty())
        .map(|r| crate::binding::Ask { id: &r.id, judged_against: &r.judged_against })
        .collect();
    let bound = if evidence { crate::perf::phase("frontier:bind", || crate::binding::bind(repo, &asks)) } else { HashMap::new() };

    for (name, data) in tasks {
        if let Some(latest) = data.results.last() {
            if latest.outcome == "pass" {
                let sha = &latest.judged_against;
                let binding = bound.get(&latest.id).unwrap_or(sha);
                let valid = sha.is_empty() || sha_valid.get(sha).copied().unwrap_or(true);
                if !sha.is_empty() && !valid && clone_can_judge {
                    done_map.insert(name.clone(), false);
                    invalid_evidence.push(name.clone());
                } else if !sha.is_empty() && !valid {
                    // Unresolvable HERE, and this clone is behind or cannot read its sync state, so
                    // it cannot tell "never pushed" from "not fetched". The work stays DONE: calling
                    // it outstanding would re-list finished work on the frontier, which is the exact
                    // duplication this class exists to prevent. Reported separately so the
                    // uncertainty is visible rather than silently resolved in either direction.
                    done_map.insert(name.clone(), true);
                    verified_at.insert(name.clone(), sha.clone());
                    unsynchronized_evidence.push(name.clone());
                } else {
                    done_map.insert(name.clone(), true);
                    verified_at.insert(name.clone(), binding.clone());
                }
            } else {
                done_map.insert(name.clone(), false);
            }
        } else {
            done_map.insert(name.clone(), false);
        }
    }
    Evidence { done_map, verified_at, invalid_evidence, unsynchronized_evidence, sync_state }
}
