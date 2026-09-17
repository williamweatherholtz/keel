//! Core logic for the `keel` CLI — validate, check, and orient commands.
//!
//! - [`validate_root`] / [`check_files`]: the validate authority, keel-model's since sprint 736, re-exported here.
//! - [`orient_root`]: compute orient view from `.tracking/` (cursor + ready/outstanding).
#![forbid(unsafe_code)]
#![deny(warnings, clippy::all, clippy::pedantic, clippy::nursery)]
// D0074 fail-loud: authority-bearing CLI code has no silent failure paths.
// (clippy::indexing_slicing deferred to M0b with the parser cleanup — see rustFailLoudLints.)
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented
)]
// Tests may use unwrap/expect/panic/indexing/asserts freely.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing))]

use std::collections::HashSet;
use std::path::Path;

use keel_parser::ast::{ActionDef, Item, Package, Part, Value};

pub use keel_model::activation;
pub use keel_schema::cli_facts;
pub use keel_json::color;
pub use keel_schema::cli_surface;
pub use keel_schema::control_defects;
pub use keel_view::control_proof;
pub use keel_actor::actor;
pub use keel_write::pin_skew;
pub use keel_model::algo;
pub use keel_view::arch;
pub mod verification;
pub use keel_schema::schema;
pub mod history;
pub mod adherence;
pub mod ci_runs;
pub mod cursor;
// The governance processes are member keel-process (D0479, sprint 736); `crate::workspace::` etc. keep resolving.
pub use keel_process::currency;
// The build-and-test tooling is member keel-suite (D0479, sprint 735); `crate::suite::` etc. keep resolving.
pub use keel_suite::suite;
pub use keel_suite::touched;
pub use keel_suite::verify;
pub use keel_git::eol;
pub use keel_github::github;
pub use keel_issues::github_ingest;
pub use keel_process::adoption_check;
pub mod attestation;
pub use keel_issues::intake_write;
pub use keel_process::workspace;
pub use keel_model::onboard;
pub mod proactive;
pub use keel_write::claim;
pub mod deck;
pub use keel_actor::device;
pub use keel_schema::embedded;
pub mod launcher;
pub use keel_process::library;
pub mod enroll;
pub use keel_git::gitx;
pub use keel_model::corpus;
pub use keel_guards::receipt;
pub use keel_suite::contentkey;
pub use keel_model::gitfacts;
pub use keel_model::binding;
pub use keel_model::done;
pub use keel_model::evidence;
pub use keel_view::priority;
pub mod reports;
pub use keel_model::ident;
pub use keel_model::suspect;
pub use keel_model::textscan;
pub use keel_view::govern;
// The forward guards are member keel-guards (D0479, sprint 733), one module per family; `crate::guards::` keeps resolving.
pub use keel_guards as guards;
pub use keel_suite::hook_binary;
pub use keel_guards::hardening;
pub use keel_model::indexer;
pub use keel_process::migrate;
use keel_json::json;
pub use keel_model::fingerprint;
pub use keel_perf::perf;
pub use keel_view::pm;
pub use keel_guards::plan_cover;
pub use keel_model::orient;
pub use keel_process::process_cmd;
pub use keel_write::claude_surface;
// The scaffolded pre-commit hook text rides with the surface whose probe it shares (sprint 733).
pub use keel_write::claude_surface::precommit_hook;
pub mod console_registry;
pub use keel_model::queries;
pub use keel_write::reverify;
pub use keel_write::scaffold;
pub mod serve;
pub use keel_process::status;
pub use keel_process::sync;
pub mod shellcheck;
// The item views and the issue write are member keel-issues' (D0480, sprint 737): `view` and `write` are
// wrapper modules so `crate::view::open_issues` and `crate::write::record_issue` keep resolving.
pub mod view {
    pub use keel_issues::views::{dispositions, intake, open_issues};
    pub use keel_view::view::*;
}
pub mod write {
    pub use keel_issues::issue_write::{record_issue, NewIssue};
    pub use keel_write::write::*;
}
pub use keel_issues as issues;
pub use keel_model::claims;
pub use keel_model::model;

// ── file discovery ────────────────────────────────────────────────────────────

// The corpus walk, the parse report type and the workflow cursor are the read model's (sprint 718).
pub use keel_model::corpus::{collect_sysml, collect_sysml_uncached, parse_pkg, CheckError};
pub use keel_model::indexer::{parse_cursor, Cursor};
// The supersede scan the kernel-free readers share descended with the guards (sprint 733).
pub use keel_model::corpus::{supersede_edges, supersede_targets};
// The validate authority is the read model's (sprint 736, D0479); the four keep resolving at the root.
pub use keel_model::validate::{check_files, validate_engine_instances, validate_root, Report};

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

// ── internal parse helper ─────────────────────────────────────────────────────

/// The character length of the longest path under `dir`, used by `keel init` to warn before a host
/// path limit turns into an opaque `git add` failure (issue313).
#[must_use]
pub fn walk_longest(dir: &std::path::Path) -> usize {
    let mut longest = 0;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = entry.path();
            longest = longest.max(p.to_string_lossy().chars().count());
            if p.is_dir() {
                stack.push(p);
            }
        }
    }
    longest
}


