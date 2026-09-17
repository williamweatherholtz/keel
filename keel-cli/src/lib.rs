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
pub use keel_view::verification;
pub use keel_schema::schema;
pub mod history;
pub mod adherence;
pub use keel_github::ci_runs;
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
pub use keel_serve::attestation;
pub use keel_issues::intake_write;
pub use keel_process::workspace;
pub use keel_model::onboard;
pub mod proactive;
pub use keel_write::claim;
pub use keel_serve::deck;
pub use keel_actor::device;
pub use keel_schema::embedded;
pub use keel_serve::launcher;
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
pub use keel_serve::reports;
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
pub use keel_model::fingerprint;
pub use keel_perf::perf;
pub use keel_view::pm;
pub use keel_guards::plan_cover;
pub use keel_model::orient;
pub use keel_process::process_cmd;
pub use keel_write::claude_surface;
// The scaffolded pre-commit hook text rides with the surface whose probe it shares (sprint 733).
pub use keel_write::claude_surface::precommit_hook;
pub use keel_serve::console_registry;
pub use keel_model::queries;
pub use keel_write::reverify;
pub use keel_write::scaffold;
// The console is member keel-serve (D0479, sprint 738): serve, deck, launcher, console_registry, reports, attestation; `crate::serve::` etc. keep resolving.
pub use keel_serve::serve;
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

// The orient computation is the read model's (sprint 738, D0479); the four keep resolving at the root.
pub use keel_model::readiness::{compute_orient_state, orient_root, whats_next_root, OrientReport};

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


