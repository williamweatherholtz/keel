//! The orient computation (sprint 738, D0479): [`OrientReport`], [`compute_orient_state`], [`orient_root`]
//! and [`whats_next_root`], sliced whole out of keel-cli's lib.rs by `scripts/extract_serve.py` so the
//! console reads `whats_next_root` from the read model rather than from the binary crate. keel-cli
//! re-exports the four at its root, so `keel_cli::orient_root` and the cucumber steps still resolve.

use std::collections::HashSet;
use std::path::Path;

use keel_parser::ast::{ActionDef, Item, Package, Part, Value};

use crate::corpus::{collect_sysml, parse_pkg};
use crate::indexer::{parse_cursor, Cursor};

// ── orient types ─────────────────────────────────────────────────────────────

/// Results of an [`orient_root`] computation.
#[derive(Debug, Default)]
pub struct OrientReport {
    /// Workflow cursor, if one was found.
    pub cursor: Option<Cursor>,
    /// Task names that are not done but have all predecessors done.
    pub ready: Vec<String>,
    /// Number of tasks with a passing latest `TestResult`.
    pub done: usize,
    /// Number of tasks that are not yet done.
    pub outstanding: usize,
}

impl OrientReport {
    /// Serialize as JSON matching `query.py orient` output format.
    #[must_use]
    pub fn to_json(&self) -> String {
        let cursor_json = self.cursor.as_ref().map_or_else(
            || "null".to_owned(),
            |c| format!(
                "{{\"activeWorkflow\":{},\"activePhase\":{},\"enteredAt\":{},\"enteredBy\":{}}}",
                json_str(&c.active_workflow),
                json_str(&c.active_phase),
                json_str(&c.entered_at),
                json_str(&c.entered_by),
            ),
        );
        let ready_json = self
            .ready
            .iter()
            .map(|s| json_str(s))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            concat!(
                "{{\"cursor\":{cursor},\"ready\":[{ready}],",
                "\"suspect\":[],\"invalidEvidence\":[],",
                "\"counts\":{{\"done\":{done},\"outstanding\":{outstanding}}}}}"
            ),
            cursor = cursor_json,
            ready = ready_json,
            done = self.done,
            outstanding = self.outstanding,
        )
    }
}

// ── orient helpers ────────────────────────────────────────────────────────────

fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn dod_result_is_pass(task_name: &str, def: &ActionDef) -> bool {
    // Accept both `{task}DoDR{n}` (Sprint 6+ naming) and `{task}R{n}` (legacy naming).
    let dodr_prefix = format!("{task_name}DoDR");
    let legacy_prefix = format!("{task_name}R");
    let mut best: Option<(u32, bool)> = None;
    for part in &def.parts {
        let suffix = part.name.strip_prefix(&dodr_prefix)
            .or_else(|| part.name.strip_prefix(&legacy_prefix));
        if let Some(suffix) = suffix {
            if let Ok(n) = suffix.parse::<u32>() {
                let is_pass = part_outcome_is_pass(part);
                if best.is_none_or(|(b, _)| n > b) {
                    best = Some((n, is_pass));
                }
            }
        }
    }
    best.is_some_and(|(_, p)| p)
}

fn part_outcome_is_pass(part: &Part) -> bool {
    part.attributes.iter().any(|attr| {
        attr.name == "outcome"
            && matches!(
                &attr.value,
                Value::EnumLit { member, .. } if member == "pass"
            )
    })
}

// ── public orient API ─────────────────────────────────────────────────────────

/// Compute the orient state (ready/done/outstanding) from a set of parsed packages.
///
/// Returns `(ready, done_count, outstanding_count)`.
/// - `ready`: task names that are not done but have all predecessors done.
/// - `done_count`: number of tasks with a passing latest `TestResult`.
/// - `outstanding_count`: number of tasks that are not yet done.
#[must_use]
pub fn compute_orient_state(packages: &[Package]) -> (Vec<String>, usize, usize) {
    let mut actions: Vec<String> = Vec::new();
    let mut successions: Vec<(String, String)> = Vec::new();

    for pkg in packages {
        for item in &pkg.items {
            if let Item::ActionDef(def) = item {
                for action in &def.actions {
                    actions.push(action.name.clone());
                }
                for suc in &def.successions {
                    if !suc.is_ordering_only {
                        successions.push((suc.first.clone(), suc.then.clone()));
                    }
                }
            }
        }
    }

    let done_set: HashSet<String> = actions
        .iter()
        .filter(|name| {
            packages.iter().any(|pkg| {
                pkg.items.iter().any(|item| {
                    if let Item::ActionDef(def) = item {
                        dod_result_is_pass(name, def)
                    } else {
                        false
                    }
                })
            })
        })
        .cloned()
        .collect();

    let mut ready: Vec<String> = actions
        .iter()
        .filter(|name| !done_set.contains(name.as_str()))
        .filter(|name| {
            successions
                .iter()
                .filter(|(_, then)| then.as_str() == name.as_str())
                .all(|(first, _)| done_set.contains(first.as_str()))
        })
        .cloned()
        .collect();
    ready.sort();

    let done = done_set.len();
    let outstanding = actions.len().saturating_sub(done);
    (ready, done, outstanding)
}

/// Return the list of ready tasks from a project root — the `whats-next` view.
///
/// Identical to `orient_root(root).ready`; exported as a first-class API so
/// callers and tests can use it without constructing a full `OrientReport`.
#[must_use]
pub fn whats_next_root(root: &Path) -> Vec<String> {
    orient_root(root).ready
}

/// Compute the orient view from a project root.
///
/// Reads all `.sysml` files under `root/.tracking/`, extracts the cursor
/// from the first package containing an `activeWorkflow` attribute, and
/// computes done/ready/outstanding from action def `DoD` `TestResults`.
#[must_use]
pub fn orient_root(root: &Path) -> OrientReport {
    let tracking_dir = root.join(".tracking");
    let packages: Vec<Package> = collect_sysml(&tracking_dir)
        .iter()
        .filter_map(|path| parse_pkg(path).ok())
        .collect();

    let cursor = packages.iter().find_map(parse_cursor);
    let (ready, done, outstanding) = compute_orient_state(&packages);

    OrientReport { cursor, ready, done, outstanding }
}
