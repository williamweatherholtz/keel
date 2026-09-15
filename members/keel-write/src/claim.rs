//! `keel claim` — one contributor's intent to work one item (D0147/D0129 srDcWorkClaim).
//!
//! # The frontier is one global list, and that is the problem
//!
//! `ready` has no owner and no claim, so several contributors — nearly all AI, on separate machines,
//! working asynchronously — rationally select the same top-ranked item, and the duplication surfaces
//! only at integration. A claim makes the intent visible BEFORE the work starts.
//!
//! # Liveness is computed, never stored
//!
//! A `Claim` carries who, what, when and against-which-commit, and nothing else. It is LIVE if it is
//! the earliest un-expired claim on its item; STALE once past the expiry window. Storing a
//! status would be a verdict that disagrees with its own facts the moment the window passes (§1.6),
//! and releasing a claim would then be a write that could be forgotten. Expiry that happens by
//! itself cannot be forgotten.
//!
//! # Exclusion is COMPUTED, because the push does not provide it
//!
//! The obvious design says two contributors claiming one item both push, the remote accepts exactly
//! one because a ref update is a compare-and-swap, and the loser re-syncs and picks again. A
//! two-clone test disproved that here. Claims are written to PER-ACTOR files
//! (srDcPerActorWriteTargets), which deliberately removes the write contention, so both claims merge
//! cleanly and BOTH land. The push rejection is real, but the loser resolves it by merging and
//! retrying, and then holds a claim just as valid as the winner's.
//!
//! So the holder is computed, and it cannot be "whoever landed first" — that is not recoverable from
//! merged history. See [`claims`] for why, and for the rule that replaced it.

use std::path::Path;

// Liveness is computed in the read model (keel-model::claims, sprint 718); this module is the verb.
pub use keel_model::claims::{claims, held_by_others, ClaimView, CLAIM_EXPIRY_DAYS};

/// `keel claim <item>` / `keel claim --list` / `keel claim --mine`.
#[must_use]
pub fn cmd(args: &[String], root: &Path) -> i32 {
    let actor = keel_actor::actor::resolve(root, None);
    if args.iter().any(|a| a == "--list") || args.is_empty() {
        let Ok(all) = claims(root) else {
            eprintln!("error: cannot read claims");
            return 1;
        };
        if all.is_empty() {
            println!("no claims recorded. `keel claim <item>` to take one.");
            return 0;
        }
        println!("claims ({} recorded, expiry {CLAIM_EXPIRY_DAYS}d, liveness COMPUTED not stored):", all.len());
        for c in &all {
            let state = if c.live {
                "LIVE     "
            } else if c.superseded {
                "superseded"
            } else {
                "stale    "
            };
            println!("  [{state}] {} by {} ({}d old, against {})", c.item, c.by, c.age_days, c.against);
        }
        return 0;
    }
    let Ok(actor) = actor else {
        eprintln!("{}", keel_actor::actor::unresolved_message());
        return 2;
    };
    if args.iter().any(|a| a == "--mine") {
        let Ok(all) = claims(root) else { return 1 };
        let mine: Vec<&ClaimView> = all.iter().filter(|c| c.by == actor && c.live).collect();
        println!("{} live claim(s) held by {actor}:", mine.len());
        for c in mine {
            println!("  {} ({}d old)", c.item, c.age_days);
        }
        return 0;
    }
    let Some(item) = args.first().filter(|a| !a.starts_with("--")) else {
        eprintln!("usage: keel claim <item> | --list | --mine");
        return 2;
    };
    // Refuse to take an item someone else holds LIVE. A stale one is fair to take — that is what
    // expiry is for — and the message says which case this is.
    match held_by_others(root, &actor) {
        Ok(held) => {
            if let Some((_, holder)) = held.iter().find(|(i, _)| i == item) {
                eprintln!("error: '{item}' is held LIVE by {holder}.");
                eprintln!("  Choose different work — `keel show whats-next` excludes what others hold. A claim held past");
                eprintln!("  {CLAIM_EXPIRY_DAYS} days without progress computes as STALE and is fair to take (D0129).");
                return 1;
            }
        }
        Err(e) => {
            eprintln!("error reading existing claims: {e}");
            return 1;
        }
    }
    match crate::write::record_claim(root, item, &actor) {
        Ok((name, path)) => {
            println!("claimed '{item}' as {name} -> {path}");
            println!("  LAND IT NOW: a claim is only exclusion once it is visible remotely. `keel land` pushes it,");
            println!("  and if the push is rejected another contributor claimed first — re-sync and pick again.");
            println!("  That rejection is the mechanism working, not an error.");
            0
        }
        Err(e) => {
            eprintln!("error: {e}");
            1
        }
    }
}
