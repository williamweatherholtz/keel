//! The hook fire ledger (`.keel/metrics/hooks.jsonl`): one line per fire, refusal and commit-gate tier.
//!
//! Moved out of members/keel-hooks/src/lib.rs by `scripts/extract_seams.py` (sprint 740, D0479) so the
//! write API's refusals (`keel accept` / `reject` / `record`) can be ledgered from the member that refuses
//! them, without the binary in between; keel-hooks re-exports the three writers it still calls and keeps
//! `ledger_fire` beside the emitted-verdict cell it reads. The slow-fire threshold came here from
//! keel-view's pm.rs with it: the writer that stamps `phases` and the report that reads them share one
//! constant, and pm re-exports it.
use std::path::Path;

/// A hook fire at or past this many ms is SLOW (issue429 / D0414).
///
/// It carries `phases` in its ledger line and appears in `enforcement-report`'s `slowFires`. Set from
/// the measurement that opened issue429: an idle full stop-hook run on the reference host is
/// 2 650-2 990 ms, so a fire at 3 000 is one that did more than the idle run - and the tails (28 s,
/// 35 s, 38 s, the 120 s watchdog) are the fires this exists to explain. A fire under it explains
/// nothing and carries nothing.
pub const SLOW_FIRE_MS: u64 = 3000;

/// The `phases` value for a fire of `total_ms`, or `None` when the fire is not slow - the
/// known-negative of D0414's probe pair: a fast fire carries no field at all.
#[must_use]
pub fn slow_fire_phases(total_ms: u64) -> Option<serde_json::Value> {
    if total_ms < SLOW_FIRE_MS {
        return None;
    }
    let rows = keel_perf::perf::attribution(total_ms)
        .into_iter()
        .map(|(name, ms)| serde_json::json!({"name": name, "ms": ms}))
        .collect::<Vec<_>>();
    Some(serde_json::Value::Array(rows))
}

/// A write-path refusal is a ledger fact (issue445): `keel accept` / `reject` and the other API writes
/// that refuse used to exit non-zero and write nothing anywhere, so the question "has an agent ever
/// tried this" had no record to be read from. `event = refused`, `control` names the check, `verb` the
/// command; ms is 0 because the refusal is the whole run.
pub fn ledger_refused(root: &Path, verb: &str, control: &str) {
    let session = std::env::var("CLAUDE_CODE_SESSION_ID").unwrap_or_default();
    // issue449: the control name is a registry fact (`write::WRITE_PATH_REFUSALS`), not a free string.
    // A name the registry never declared is still a fire, so it is written - marked, so the census
    // cannot count it under a row that does not exist.
    let name = if crate::write::write_path_refusal(verb, control).is_some() {
        format!("{verb}:{control}")
    } else {
        eprintln!("keel: refusal {verb}:{control} is not in the write-path registry (issue449); ledgered as unregistered");
        format!("unregistered:{verb}:{control}")
    };
    ledger_line(root, &session, "refused", 1, 0, Some(("refused".to_string(), name)));
}

/// `record decision` / `record issue` refused prose that reads as captured tool output (D0224/issue256):
/// the refusal is a ledger fact - `record:tool-output-prose` is the census row (issue449) - and the
/// verb's exit code.
pub fn refused_injected_prose(root: &Path, e: &dyn std::fmt::Display) -> i32 {
    ledger_refused(root, "record", "tool-output-prose");
    eprintln!("error: {e}");
    1
}
/// The commit tier is in the ledger with the in-loop tiers (dcRefusalIsALedgerFact clause d): the
/// scaffolded pre-commit hook runs `keel gate validate`, `keel gate guard` and `keel gate check-engine` as separate
/// processes, so each writes one `commit-gate-<tier>` line - `allow` when green, `block` naming the
/// refusing controls (the failing guard names, or the tier itself) when red. A run by hand writes the same line; the ledger does not know who
/// invoked it, and a rate over both is still a rate.
pub fn ledger_gate(root: &Path, tier: &str, failing: &[String], ms: u128) {
    let session = std::env::var("CLAUDE_CODE_SESSION_ID").unwrap_or_default();
    let verdict = if failing.is_empty() { None } else { Some(("block".to_string(), failing.join(","))) };
    ledger_line(root, &session, &format!("commit-gate-{tier}"), i32::from(!failing.is_empty()), ms, verdict);
}

/// The kind of actor this process runs as - `human`, `ai` or `unknown` from actors.sysml, `undeclared`
/// when the bound actor has no part there, `unbound` when no actor resolves at all - so a refusal
/// line says whose act the control fell on.
fn ledger_actor_kind(root: &Path) -> String {
    keel_actor::actor::resolve(root, None).map_or_else(
        |_| "unbound".to_string(),
        |name| keel_actor::actor::kind_of(root, &name).unwrap_or_else(|| "undeclared".to_string()),
    )
}

/// The one line writer: every fire, refusal and commit-gate tier appends one JSON object to
/// `.keel/metrics/hooks.jsonl`. `verdict` is `(decision, control)` when the caller has one; otherwise the
/// exit code decides `allow` / `block`. Unavailable or failed writes are reported on stderr, never fatal:
/// the ledger observes the hook, it does not gate it.
pub fn ledger_line(root: &Path, session: &str, event: &str, exit: i32, ms: u128, verdict: Option<(String, String)>) {
    use std::io::Write as _;
    let dir = root.join(".keel").join("metrics");
    if std::fs::create_dir_all(&dir).is_err() {
        eprintln!("[keel] fire-ledger unavailable: cannot create {}", dir.display());
        return;
    }
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let (decision, control) = match verdict {
        Some((d, c)) => (d, Some(c)),
        None => ((if exit == 0 { "allow" } else { "block" }).to_string(), None),
    };
    let ms = u64::try_from(ms).unwrap_or(u64::MAX);
    // issue378 / GH#55: WHICH binary ran this hook, and which build - the turn-boundary surface's
    // answer to "did the pinned engine gate this", readable from `keel show status`.
    let mut record = serde_json::json!({"ts": ts, "session": session, "event": event, "decision": decision, "exit": exit, "ms": ms,
        "bin": std::env::current_exe().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(), "build": env!("KEEL_BUILD_COMMIT")});
    if let Some(obj) = record.as_object_mut() {
        // issue446: a line that is not an allow names the control that refused and the kind of actor
        // it refused. An allow carries neither - 16 000 lines a day should not each pay for a fact that
        // only a refusal has.
        if decision != "allow" {
            obj.insert("control".to_string(), serde_json::Value::String(control.unwrap_or_else(|| "exit-code".to_string())));
            obj.insert("actorKind".to_string(), serde_json::Value::String(ledger_actor_kind(root)));
        }
        // D0414 / issue429: a SLOW fire explains itself - the phases it measured, longest first, with the
        // remainder no counter covered named as unattributed. A fast fire carries no field (the threshold is
        // [`SLOW_FIRE_MS`] above; pm.rs is the reader).
        if let Some(phases) = slow_fire_phases(ms) {
            obj.insert("phases".to_string(), phases);
        }
    }
    let line = format!("{record}\n");
    let appended = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("hooks.jsonl"))
        .and_then(|mut f| f.write_all(line.as_bytes()));
    if let Err(e) = appended {
        eprintln!("[keel] fire-ledger write failed: {e}");
    }
}
