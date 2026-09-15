//! The claim read side (D0147 srDcWorkClaim): who holds which item, LIVE or STALE, computed against
//! git-derived time. Out of `claim.rs` in sprint 718 (D0479) so the frontier can ask "held by others"
//! without the write API. `keel claim` itself is `keel_write::claim`.
//!
//! A `Claim` carries who, what, when and against-which-commit, and nothing else. Storing a status
//! would be a verdict that disagrees with its own facts the moment the expiry window passes (§1.6).

use std::path::Path;

/// How long a claim stays live without progress.
///
/// Deliberately generous: the cost of a stale claim is a brief duplication, while the cost of
/// expiring a live one is two contributors on one item each believing they hold it. Wrong in the
/// safe direction.
pub const CLAIM_EXPIRY_DAYS: i64 = 2;

/// A claim as computed, with the liveness the model does not store.
pub struct ClaimView {
    pub name: String,
    pub item: String,
    pub by: String,
    pub at: String,
    pub against: String,
    pub age_days: i64,
    pub live: bool,
    /// Outranked by another LIVE claim on the same item — distinct from stale, and worth separating:
    /// superseded means someone else holds it, stale means nobody does and it is fair to take.
    pub superseded: bool,
}

/// Every claim in the model, holder first per item, with liveness computed against git-derived time.
///
/// # Errors
/// Returns [`crate::model::ViewError`] if a tracking file fails to parse.
// @audit-hash ceClaimHolderRuleMember
pub fn claims(root: &Path) -> Result<Vec<ClaimView>, crate::model::ViewError> {
    let today = crate::queries::repo_today(root);
    let mut rows = crate::queries::claim_rows(root)?;
    // WHO HOLDS THE ITEM, and why it cannot be "whoever landed first".
    //
    // The process says exclusion comes from the remote accepting exactly one of two concurrent
    // claims, because a ref update is a compare-and-swap. THAT IS NOT TRUE HERE, and the two-clone
    // test proved it: per-actor claim files (srDcPerActorWriteTargets) deliberately remove the write
    // contention, so both claims merge cleanly and BOTH land. Exclusion has to be computed.
    //
    // "Whoever landed first" is then not recoverable. Two claims committed in parallel are SIBLINGS:
    // measured against the merged history they have the same ancestry depth, and their commit
    // timestamps tie whenever the work happens in the same second — which it did. Any rule reading
    // git order either disagrees between clones or falls through to an arbitrary tie-break anyway.
    //
    // So the rule is EARLIEST `claimedAt`, then lowest claim id. The id is a UUID: unbiased, total,
    // and identical in every clone, which is the only property that actually matters — every
    // contributor must compute the same holder without coordinating. Sorting by NAME instead would
    // have handed the item to whoever sorts alphabetically first, which is deterministic and unfair,
    // and the test caught exactly that (alpha held it over beta whichever one landed first).
    let ids = crate::queries::claim_ids(root)?;
    rows.sort_by(|a, b| {
        a.3.cmp(&b.3).then_with(|| ids.get(&a.0).cmp(&ids.get(&b.0))).then_with(|| a.0.cmp(&b.0))
    });
    // EXPIRY REMOVES A CLAIM FROM CONTENTION, not merely from the top of the ranking. Ordering first
    // and expiring second looked equivalent and was not: a two-clone test aged the earliest claim past
    // the window and the item became held by NOBODY, because the stale claim still occupied the holder
    // slot and marked the fresh claim `superseded`. An expired claim must not be able to supersede a
    // live one — so eligibility is decided BEFORE the holder is picked, and the holder is the earliest
    // claim among those still inside the window.
    let mut seen_item: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut out = Vec::new();
    for (name, item, by, at, against) in rows {
        let age = if today.is_empty() || at.is_empty() { -1 } else { crate::queries::days_between(&at, &today) };
        let eligible = (0..=CLAIM_EXPIRY_DAYS).contains(&age);
        let superseded = eligible && !seen_item.insert(item.clone());
        out.push(ClaimView {
            live: eligible && !superseded,
            name,
            item,
            by,
            at,
            against,
            age_days: age,
            superseded,
        });
    }
    Ok(out)
}

/// Item names held LIVE by someone other than `actor`.
///
/// # Errors
/// Returns [`crate::model::ViewError`] if a tracking file fails to parse.
pub fn held_by_others(root: &Path, actor: &str) -> Result<Vec<(String, String)>, crate::model::ViewError> {
    Ok(claims(root)?
        .into_iter()
        .filter(|c| c.live && c.by != actor)
        .map(|c| (c.item, c.by))
        .collect())
}

