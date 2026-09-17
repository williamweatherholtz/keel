//! keel-guards: the forward guards - one module per guard family, the runner and the shared scanners here.
//!
//! The fourth D0479 extraction (sprint 733): `keel-cli/src/guards.rs` split by `scripts/split_guards.py`
//! into family modules, each carrying its guards' dispatch arms, code and tests. keel-cli re-exports this
//! crate as `guards` under its old path, so no caller moved. The enforced list `GUARD_NAMES` is the schema
//! member's fact (D0503), re-exported here; the family tables are held equal to it by a test.
#![forbid(unsafe_code)]
#![deny(warnings, clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, clippy::todo, clippy::unimplemented)]
#![allow(clippy::implicit_hasher, clippy::too_long_first_doc_paragraph, clippy::wildcard_imports, clippy::module_name_repetitions)]
// The family modules are private and glob re-exported: a `pub(crate)` helper in one is reachable from the
// root and its siblings through `use super::*` yet stays out of the crate's public surface, which `pub`
// would not (the glob would export it). So the `pub(crate)` is not redundant, whatever the nursery lint says.
#![allow(clippy::redundant_pub_crate)]
// Tests may use unwrap/expect/panic/indexing/asserts freely.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing))]

pub mod hardening;
pub mod plan_cover;
pub mod receipt;

mod identity;
pub use identity::*;
mod acceptance;
pub use acceptance::*;
mod decisions;
pub use decisions::*;
mod sprints;
pub use sprints::*;
mod issues;
pub use issues::*;
mod enforcement;
pub use enforcement::*;
mod surface;
pub use surface::*;
mod release;
pub use release::*;
mod critique;
pub use critique::*;

/// A guard: the function that answers for one name.
pub type GuardFn = fn(&Path) -> GuardReport;

/// One dispatch arm: `(guard name, the function that answers for it)`.
pub type Arm = (&'static str, GuardFn);

/// One guard family: its name and, in `GUARD_NAMES` order, the guards it dispatches (sprint 733).
pub struct Family {
    /// The family's module name.
    pub name: &'static str,
    /// The family's dispatch arms.
    pub arms: &'static [Arm],
}

/// The family tables in family order; `run_one` dispatches through them and the union test holds
/// them equal to `GUARD_NAMES` (D0503: the enforced list is the schema member's fact).
pub const FAMILIES: &[&Family] = &[&identity::FAMILY, &acceptance::FAMILY, &decisions::FAMILY, &sprints::FAMILY, &issues::FAMILY, &enforcement::FAMILY, &surface::FAMILY, &release::FAMILY, &critique::FAMILY];

/// Guards `run_one` dispatches that are NOT enforced members of `GUARD_NAMES` (runnable-only).
pub const RUNNABLE_ONLY: [&str; 4] = ["critique", "assured", "critique-rigor", "defect-guard-coverage"];

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use keel_model::binding::step_check_bindings;

use keel_json::json::Json;

use keel_view::view::{ReadinessBlockers, ViewError};

use keel_schema::cli_facts::{parse_cli_facts, AuthoredCliFact};

use keel_model::ident::{is_v4_uuid, uuid_shaped};

use keel_model::textscan::{engine_markers, markers_declared, markers_used, named_items, retro_texts, strip_string_literals, RETRO_NO_ITEM_JUSTIFICATIONS};

use std::path::{Path, PathBuf};

use keel_model::algo::is_space;

/// A guard's outcome: scanned count, tolerated warnings, and blocking violations.
pub struct GuardReport {
    pub name: &'static str,
    pub scanned: usize,
    pub warnings: Vec<String>,
    pub violations: Vec<String>,
}

/// The mark a guard puts on a warning it COUNTS rather than enumerates (issue404 / D0413).
///
/// A counted-history line reports immutable history no edit can discharge - legacy actors in records
/// that predate the convention, resolutions before the naming cutoff, grandfathered attestations and
/// compound Decisions, a sprint closed before its ceremony existed. The guard says so by constructing
/// the line with [`history_line`]; everything that reads a report - the printer, the runner's summary,
/// orient's burndown - separates those from the ACTIONABLE warnings by this mark, so the population a
/// reader is asked to read is the one they can act on. Nothing classifies by reading the prose.
pub const HISTORY_PREFIX: &str = "HISTORY ";

/// A warning the guard counts rather than enumerates - see [`HISTORY_PREFIX`].
#[must_use]
pub fn history_line(text: &str) -> String {
    format!("{HISTORY_PREFIX}{text}")
}

/// Is `warning` a counted-history line?
#[must_use]
pub fn is_history(warning: &str) -> bool {
    warning.starts_with(HISTORY_PREFIX)
}

/// The mark a diff-reading guard puts on the note naming WHICH population it judged (D0440 / issue464).
///
/// Inside a git hook the guard reads the staged index - the population the commit carries; anywhere else
/// it reads the working tree, so a locked-file edit is a violation the moment it exists and a verifier's
/// verdict is the commit's. The note rides in `warnings` like a history line but is neither actionable
/// nor history: the printer folds it into the summary line as `(read: working tree)` and nothing counts it.
pub const READ_PREFIX: &str = "READ ";

/// The read-mode note for `read` - see [`READ_PREFIX`].
#[must_use]
pub fn read_line(read: ChangeRead) -> String {
    format!("{READ_PREFIX}{}", read.label())
}

/// Is `warning` a read-mode note?
#[must_use]
pub fn is_read(warning: &str) -> bool {
    warning.starts_with(READ_PREFIX)
}

/// Which population a diff-reading guard judges (D0440 / issue464).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChangeRead {
    /// Inside a git hook: `GIT_INDEX_FILE` is set by git for every hook (pre-commit in both the plain
    /// and the `-a` shape, and post-commit, probed 2026-09-10), or `KEEL_HOOK` by a caller that IS the
    /// commit gate. The staged index is the population the commit carries.
    Index,
    /// Anywhere else - the turn-boundary stop hook, a verifier, a human at the terminal: the working
    /// tree against HEAD, untracked files as additions. On 2026-09-10 the D0425 verifier read an empty
    /// index over an unstaged sprint and reported ALL PASS; the pre-commit hook then refused the same
    /// edits (issue464).
    WorkingTree,
}

impl ChangeRead {
    /// The mode this process runs in, from the environment git and the gate leave.
    #[must_use]
    pub fn current() -> Self {
        if std::env::var_os("GIT_INDEX_FILE").is_some() || std::env::var_os("KEEL_HOOK").is_some() {
            Self::Index
        } else {
            Self::WorkingTree
        }
    }

    /// The words the summary line carries.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Index => "index",
            Self::WorkingTree => "working tree",
        }
    }
}

impl GuardReport {
    /// True when there are no blocking violations.
    #[must_use]
    pub const fn ok(&self) -> bool {
        self.violations.is_empty()
    }

    /// SELF-CONTRADICTION CHECK (issue180). A guard cannot find a violation in a population of zero, so
    /// `0 scanned, 1 violation(s)` means the guard is not reporting what it examined. Three guards printed
    /// exactly that while working correctly, which made `scanned` useless as a liveness signal - the only
    /// signal separating a guard whose population is legitimately empty from one that is mis-aimed and
    /// can never fire. Surfaced by the RUNNER at print time rather than per guard, so it holds for every
    /// guard added after this one without anybody remembering to. `Some(line)` is the warning to print.
    #[must_use]
    pub fn self_contradiction(&self) -> Option<String> {
        (self.scanned == 0 && !self.violations.is_empty()).then(|| {
            format!(
                "guard `{}` reports {} violation(s) against a scan count of 0 - it is not reporting the population it examined (issue180)",
                self.name,
                self.violations.len()
            )
        })
    }

    /// The warnings a reader can act on - every warning that is not a counted-history line.
    pub fn actionable(&self) -> impl Iterator<Item = &String> {
        self.warnings.iter().filter(|w| !is_history(w) && !is_read(w))
    }

    /// The read-mode note, if this guard reads a diff (D0440): `Some("working tree")` / `Some("index")`.
    #[must_use]
    pub fn read_mode(&self) -> Option<&str> {
        self.warnings.iter().find_map(|w| w.strip_prefix(READ_PREFIX))
    }

    /// The counted-history lines (issue404): immutable history, reported so it is never mistaken for silence.
    pub fn history(&self) -> impl Iterator<Item = &String> {
        self.warnings.iter().filter(|w| is_history(w))
    }

    /// Print the human report (warnings, then violations, then a summary line).
    pub fn print(&self) {
        for w in self.warnings.iter().filter(|w| !is_read(w)) {
            match w.strip_prefix(HISTORY_PREFIX) {
                Some(h) => println!("  {}  {h}", keel_json::color::warn("HISTORY")),
                None => println!("  {}  {w}", keel_json::color::warn("WARN")),
            }
        }
        for v in &self.violations {
            println!("  {} {v}", keel_json::color::fail("ERROR"));
        }
        if let Some(line) = self.self_contradiction() {
            println!("  {}  {line}", keel_json::color::warn("WARN"));
        }
        let history = self.history().count();
        let counted = if history == 0 { String::new() } else { format!(" + {history} counted-history line(s)") };
        let read = self.read_mode().map_or_else(String::new, |r| format!(" (read: {r})"));
        println!(
            "[guard:{}] {} — {} scanned{read}, {} warning(s){counted}, {} violation(s)",
            self.name,
            keel_json::color::verdict(self.ok()),
            self.scanned,
            self.actionable().count(),
            self.violations.len()
        );
    }
}

/// The runner's summary of the warning population across `reports` (issue404 / D0413).
///
/// Empty when nothing warned. Otherwise the ACTIONABLE count and the guards carrying them come first -
/// that is the set a reader is asked to read - and the counted-history lines are a number with their
/// guards, never merged into it: a set that contains permanent noise cannot be read as a set worth
/// reading, and that was the mechanism by which 131 warnings went unread (issue404).
#[must_use]
pub fn warning_population(reports: &[GuardReport]) -> String {
    use std::fmt::Write as _;
    let actionable: usize = reports.iter().map(|r| r.actionable().count()).sum();
    let history: usize = reports.iter().map(|r| r.history().count()).sum();
    if actionable + history == 0 {
        return String::new();
    }
    let names = |pick: fn(&GuardReport) -> bool| reports.iter().filter(|r| pick(r)).map(|r| r.name).collect::<Vec<_>>().join(", ");
    let mut out = String::new();
    if actionable > 0 {
        let _ = write!(
            out,
            " — {actionable} actionable warning(s) across {} guard(s), NOT violations and NOT blocking, each naming a condition an edit can discharge (keel show orient lists them in its burndown): {}",
            reports.iter().filter(|r| r.actionable().count() > 0).count(),
            names(|r| r.actionable().count() > 0)
        );
    }
    if history > 0 {
        let _ = write!(
            out,
            "{} {history} counted-history line(s) ({}): immutable history, counted so it is not mistaken for silence, not dischargeable",
            if actionable > 0 { ";" } else { " —" },
            names(|r| r.history().count() > 0)
        );
    }
    out
}

fn relpath(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

/// Read a quoted `:>> <name> = "value"` off a single declaration line.
fn field(line: &str, name: &str) -> Option<String> {
    let needle = format!(":>> {name} = \"");
    let rest = line.split(&needle).nth(1)?;
    Some(rest.split('"').next()?.to_string())
}

const GRANDFATHERED: &[&str] = &["ceremonyGateGuard", "rustS8runtimeParser", "rustS9writeApi", "trackedMetadataReplan"];

// ── untrusted-taint guard (D0314 / issue347: the label travels with the derivation) ──────────────

/// How a Decision was accepted, as far as trust conferral is concerned.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Acceptance {
    /// A human's own act - their quoted words (D0289), the console, a terminal - confers trust.
    Human,
    /// Standing consent applied by the recorder at record time (D0291): nobody read it; confers nothing.
    Auto,
}

/// How `dname` was accepted, from its file: a passing `AcceptR` result, and whether the `Accept` Test
/// says the acceptance was automatic.
pub(crate) fn acceptance_kind(text: &str, dname: &str) -> Option<Acceptance> {
    let has_pass = text.match_indices(&format!("part {dname}AcceptR")).any(|(i, _)| text[i..].split('}').next().is_some_and(|seg| seg.contains("VerdictKind::pass")));
    if !has_pass {
        return None;
    }
    let auto = text.find(&format!("verification {dname}Accept : Test")).is_some_and(|i| text[i..].split('}').next().is_some_and(|seg| seg.contains("AUTO-ACCEPTED")));
    Some(if auto { Acceptance::Auto } else { Acceptance::Human })
}

// ── charter guard (newly-added delivery Story declares its #CharteredBy edge) ───────────────────

fn git_stdout(root: &Path, args: &[&str]) -> String {
    keel_git::gitx::git()
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

// ── process-change keystone guard (D0070 hard lock) ────────────────────────────────────────────

fn is_sysml(p: &str) -> bool {
    std::path::Path::new(p).extension().is_some_and(|e| e == "sysml")
}

/// Does this commit carry a staged Decision bearing a `#ProspectiveChange`/`#SafetyChange` marker?
///
/// The authorisation half of the keystone lock, exposed for the workspace gate. Additions AND
/// modifications, because the authorising Decision is almost always a NEW file — conflating the two
/// lists is the regression `process_change` documents catching on its own author.
/// A Decision at ANY depth, because in a workspace there is no project at the repository root and so
/// no `.engine/decisions/` there. Requiring the root-relative form would make the shared root hook
/// permanently unchangeable: no Decision could ever authorise a change to it, which is a different
/// failure from the one being fixed but just as wrong. A marked Decision recorded in ANY project in
/// the repository authorises the repository's own locked surface — the human signing it is the same
/// human either way, and the alternative is a surface with no legitimate path to change.
fn is_decision_file_at_any_depth(p: &str) -> bool {
    let s = p.replace('\\', "/");
    is_sysml(p) && (s.starts_with(".engine/decisions/") || s.contains("/.engine/decisions/"))
}

#[must_use]
pub fn staged_marked_decision(root: &Path) -> bool {
    let read = ChangeRead::current();
    // Either source of the keystone (D0465): a marked Decision in the commit, or a sprint record in
    // the commit chartered by a marked Decision the human has accepted.
    changed_files(root, read)
        .iter()
        .filter(|p| is_decision_file_at_any_depth(p))
        .any(|p| has_process_marker(&changed_text(root, p, read)))
        || !accepted_charters(root, read).is_empty()
}

fn is_decision_file(p: &str) -> bool {
    is_sysml(p) && p.starts_with(".engine/decisions/")
}

/// True if a line-anchored `#ProspectiveChange`/`#SafetyChange` marker is present (prose mentions
/// inside string literals start with `:>>`/`//`, so they never match). Mirrors `_MARKER`.
fn has_process_marker(text: &str) -> bool {
    text.lines().any(|line| {
        let t = line.trim_start_matches(is_space);
        ["#ProspectiveChange", "#SafetyChange"]
            .iter()
            .any(|kw| t.strip_prefix(kw).is_some_and(|rest| rest.chars().next().is_none_or(|c| !keel_model::algo::is_word(c))))
    })
}

/// The `dNNNN` targets of a sprint record's line-anchored `#CharteredBy dependency from <story> to
/// dNNNN;` edges - the engine's own statement of which Decision the work executes (D0068), which is
/// the second authorising source of the keystone lock (D0465).
fn charter_targets(sprint_text: &str) -> Vec<String> {
    sprint_text
        .lines()
        .filter_map(|line| {
            let rest = line.trim_start_matches(is_space).strip_prefix("#CharteredBy dependency from ")?;
            let (_, target) = rest.split_once(" to ")?;
            let target = target.trim().trim_end_matches(';').trim();
            let digits = target.strip_prefix('d')?;
            (!digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())).then(|| target.to_string())
        })
        .collect()
}

/// Does a Decision's text authorise a locked edit as a CHARTER (D0465)? Marked AND accepted by a
/// human - a passing `AcceptR` whose `Accept` Test is not `AUTO-ACCEPTED`. A proposed, rejected,
/// auto-accepted (standing consent never reaches the enforcement surface, D0337) or unmarked
/// Decision authorises nothing.
fn is_authorising_charter(decision_text: &str, dname: &str) -> bool {
    has_process_marker(decision_text) && acceptance_kind(decision_text, dname) == Some(Acceptance::Human)
}

fn is_sprint_record(p: &str) -> bool {
    let s = p.replace('\\', "/");
    is_sysml(p) && (s.starts_with(".tracking/delivery/") || s.contains("/.tracking/delivery/"))
}

/// `(decision, sprint path)` for every changed sprint record whose charter authorises the lock.
///
/// The Decision is read from the PROJECT that owns the sprint file - the path prefix before
/// `.tracking/` - so a workspace with several projects cannot borrow another project's Decision.
/// Read from disk, not the index: an accepted charter is almost always a Decision committed earlier
/// (its acceptance is why the edit is only now being made); one changed in this commit and marked is
/// already the first source.
fn accepted_charters(root: &Path, read: ChangeRead) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for p in changed_files(root, read).into_iter().filter(|p| is_sprint_record(p)) {
        let s = p.replace('\\', "/");
        let project = s.split(".tracking/").next().unwrap_or_default();
        let decisions = root.join(project).join(".engine").join("decisions");
        for dname in charter_targets(&changed_text(root, &p, read)) {
            let needle = format!("part {dname} : Decision");
            let authorises = keel_model::corpus::collect_sysml(&decisions)
                .iter()
                .filter_map(|f| keel_model::corpus::read_to_string(f).ok())
                .any(|t| t.contains(&needle) && is_authorising_charter(&t, &dname));
            if authorises {
                out.push((dname, p.clone()));
            }
        }
    }
    out
}

/// Added, copied, modified or renamed paths under `read` - the list a guard looks for an authorising
/// Decision, a doc update or a sprint record in.
fn changed_files(root: &Path, read: ChangeRead) -> Vec<String> {
    changed_paths(root, "ACMR", read)
}

/// A changed file's text under `read`: the index blob inside a hook, the file on disk outside it.
fn changed_text(root: &Path, path: &str, read: ChangeRead) -> String {
    match read {
        ChangeRead::Index => git_stdout(root, &["show", &format!(":{path}")]),
        ChangeRead::WorkingTree => keel_model::corpus::read_to_string(root.join(path)).unwrap_or_default(),
    }
}

/// Changed paths for a given `--diff-filter` under `read`, scoped to this project, with BOTH sides of
/// a rename and no C-quoting.
///
/// `Index` reads `git diff --cached` - the commit's population. `WorkingTree` (D0440 / issue464) reads
/// `git diff HEAD` with the same filter and status letters, and every untracked file as an `A` when the
/// filter admits additions - the authorising Decision is almost always a new file, and a read that could
/// not see it would refuse every properly signed change. A repository with no HEAD yet reads the index
/// either way: everything is an addition there, which is the shape issue272 exists to let through.
///
/// TWO CORRECTIONS AN ADVERSARIAL PANEL FOUND, both of which let a locked file change unsigned:
///
/// 1. **Renames.** `--name-only` prints only a rename's DESTINATION, and the keystone filter had been
///    narrowed to `MD`, which excludes `R` entirely. So `sed 's/blocking/warning/' rules.sysml >
///    rules2.sysml; rm rules.sysml; git add -A` presented git with `R096` — and the guard saw NOTHING.
///    Every blocking rule in a project could be downgraded to a warning by `git mv`, in a
///    single-project repo, with no Decision. `--name-status -M` is read instead, and BOTH paths of an
///    `R` are returned: the source matters (a locked file left its path) and so does the destination
///    (a locked file arrived carrying modified content).
/// 2. **Non-ASCII paths.** git C-QUOTES a path containing a non-ASCII byte in this output
///    (`"pr\303\264j/..."`) while `rev-parse --show-prefix` does NOT, so the prefix comparison below
///    stripped nothing and a project named `prôj` was invisible to the workspace gate AND to its own
///    guards — a full rules downgrade there passed. `core.quotePath=false` with `-z` records makes
///    both sides of that comparison the same bytes.
fn changed_paths(root: &Path, filter: &str, read: ChangeRead) -> Vec<String> {
    // SCOPED TO THIS PROJECT (D0234/issue271). `git diff --cached` answers for the whole REPOSITORY
    // no matter which subdirectory you ask from, so in a repo holding several keel projects every
    // project's guards saw every other project's staged files — and the workspace-level ones too.
    // Found live: a two-project workspace failed BOTH projects' process-change guard over
    // `.githooks/pre-commit`, a file belonging to neither. Paths are returned repo-relative, so they
    // are re-based onto the project and anything outside it is dropped.
    let flag = format!("--diff-filter={filter}");
    // `-c core.quotePath=false` with `-z`: see (2) above. `--name-status -M` so a rename yields BOTH
    // of its paths: see (1). Records are NUL-separated; an `R`/`C` status is followed by TWO paths.
    let has_head = !git_stdout(root, &["rev-parse", "--verify", "-q", "HEAD"]).trim().is_empty();
    let mut raw = if read == ChangeRead::WorkingTree && has_head {
        git_stdout(root, &["-c", "core.quotePath=false", "diff", "HEAD", "--name-status", "-M", "-z", &flag])
    } else {
        git_stdout(root, &["-c", "core.quotePath=false", "diff", "--cached", "--name-status", "-M", "-z", &flag])
    };
    if read == ChangeRead::WorkingTree && filter.contains('A') {
        // Untracked files are additions on disk. `--full-name` makes them repo-relative like the diff's.
        for p in git_stdout(root, &["-c", "core.quotePath=false", "ls-files", "--others", "--exclude-standard", "--full-name", "-z"]).split('\0').filter(|p| !p.is_empty()) {
            raw.push_str("A\0");
            raw.push_str(p);
            raw.push('\0');
        }
    }
    let mut fields = raw.split('\0').filter(|f| !f.is_empty());
    let mut repo_relative: Vec<String> = Vec::new();
    while let Some(status) = fields.next() {
        let renamed = status.starts_with('R') || status.starts_with('C');
        let Some(first) = fields.next() else { break };
        repo_relative.push(first.to_string());
        if renamed {
            // BOTH sides. Dropping the source is what let a locked file be edited-and-moved unsigned.
            if let Some(second) = fields.next() {
                repo_relative.push(second.to_string());
            }
        }
    }
    let prefix = git_stdout(root, &["rev-parse", "--show-prefix"]).trim().replace('\\', "/");
    repo_relative
        .into_iter()
        .map(|l| l.trim().replace('\\', "/"))
        .filter(|l| !l.is_empty())
        .filter_map(|l| {
            if prefix.is_empty() {
                // The project IS the repo root: every staged file is its own, as it always was.
                Some(l)
            } else {
                // Keep only what lives under this project, re-based to project-relative.
                l.strip_prefix(&prefix).map(str::to_string)
            }
        })
        .collect()
}

/// The VALUE of a string attribute: every quoted segment of `:>> key = "a" + "b";` concatenated.
/// The early Decisions write long fields as `+`-joined segments across lines, and the indentation
/// between segments is source formatting, not signed text - comparing raw source read ten of them as
/// drifted on whitespace alone.
fn field_of(text: &str, key: &str) -> Option<String> {
    let start = text.find(&format!(":>> {key} = "))? + key.len() + 7;
    let rest = &text[start..];
    let mut value = String::new();
    let mut in_str = false;
    let mut saw_any = false;
    for c in rest.chars() {
        if in_str {
            if c == '"' {
                in_str = false;
            } else {
                value.push(c);
            }
        } else if c == '"' {
            in_str = true;
            saw_any = true;
        } else if c == ';' {
            break;
        } else if !(c.is_whitespace() || c == '+') {
            break; // not a string attribute
        }
    }
    saw_any.then_some(value)
}

// The resolver predicate is the read model's (sprint 737, D0508); the guards and `record issue` read one function.
pub use keel_model::resolvers::declared_task_names;

/// Readiness, composed (D0079 c): the suspect walk, the guard suite and the view's own categories.
///
/// # Errors
/// Returns [`ViewError`] if a tracking/instance file fails to parse.
pub fn compute_readiness(root: &Path) -> Result<ReadinessBlockers, ViewError> {
    let task_suspect = keel_perf::perf::phase("suspectWalk", || keel_model::suspect::suspect(root));
    // Base invariant guards only — EXCLUDE `assured` (would recurse) and `critique` (composed
    // separately as critique_gaps). This is what "invariants green" means for readiness.
    // THE WHOLE GUARD SUITE, INSIDE A VIEW. Legitimate - readiness means invariants hold - but it makes
    // `keel gate assured` cost `keel gate guard` PLUS every composed view, which is the single largest term and was
    // invisible until it was named.
    let invariant_violations: Vec<String> = keel_perf::perf::phase("allGuards", || {
        GUARD_NAMES
            .iter()
            .copied()
            .filter(|n| !matches!(*n, "assured" | "critique"))
            .filter_map(|n| run_one(n, root))
            .flat_map(|r| r.violations.into_iter().map(move |v| format!("{}: {v}", r.name)))
            .collect()
    });
    keel_view::view::readiness(root, task_suspect, invariant_violations)
}

/// Assurance-readiness view (D0079 c) as JSON: the composite READY/NOT-READY verdict + per-category
/// blocker counts and samples. The single "is the deliverable assured?" answer; never stored.
///
/// # Errors
/// Returns [`ViewError`] if a tracking/instance file fails to parse.
pub fn assured_report(root: &Path) -> Result<String, ViewError> {
    let b = compute_readiness(root)?;
    let cat = |label: &str, v: &[String]| {
        Json::Obj(vec![
            ("category".to_string(), Json::s(label)),
            ("count".to_string(), Json::Int(i64::try_from(v.len()).unwrap_or(i64::MAX))),
            ("sample".to_string(), Json::Arr(v.iter().take(10).map(|s| Json::s(s.clone())).collect())),
        ])
    };
    let blockers = Json::Arr(vec![
        cat("coverage_gaps", &b.coverage_gaps),
        cat("critique_gaps", &b.critique_gaps),
        cat("undispositioned_findings", &b.undispositioned_findings),
        cat("unfixed_critical", &b.unfixed_critical),
        cat("invariant_violations", &b.invariant_violations),
    ]);
    // Advisory: surfaced for the full picture but NOT gating (cleared by re-verification, D0050).
    let advisories = Json::Arr(vec![cat("stale_verifications", &b.stale_verifications)]);
    let out = Json::Obj(vec![
        (
            "assured".to_string(),
            Json::s("assurance readiness (D0079 c; charter-time scoped, D0081): READY iff GOVERNED coverage complete AND GOVERNED critique complete AND every >=Medium finding dispositioned AND no Critical open AND invariants green. stale_verifications is advisory (re-verify; not gating)"),
        ),
        ("ready".to_string(), Json::Bool(b.ready())),
        ("verdict".to_string(), Json::s(b.verdict())),
        ("governed".to_string(), Json::Int(i64::try_from(b.governed).unwrap_or(0))),
        ("blockers".to_string(), blockers),
        ("advisories".to_string(), advisories),
    ]);
    Ok(out.dump())
}

/// The ENFORCED forward guards, in CLI/runner order - declared in `keel_schema::guard_names` (sprint 732)
/// so the view layer's proof census reads it without reaching up into this file; the list's doctrine
/// (D0077, D0098) is documented there. Re-exported here so every caller keeps `crate::GUARD_NAMES`.
pub use keel_schema::guard_names::GUARD_NAMES;

/// Run one guard by name - `None` if the name is unknown (a name outside every family table).
///
/// Dispatch is the family tables' (sprint 733): each family names its guards beside the code that
/// answers for them, and this function only walks the tables. A guard in one home is in all: the
/// union test below holds the tables equal to `GUARD_NAMES` plus `RUNNABLE_ONLY`.
#[must_use]
pub fn run_one(name: &str, root: &Path) -> Option<GuardReport> {
    FAMILIES.iter().flat_map(|f| f.arms.iter()).find(|(n, _)| *n == name).map(|(_, run)| run(root))
}

/// Run all enforced guards over `root`, returning their reports in `GUARD_NAMES` order.
#[must_use]
pub fn run_all(root: &Path) -> Vec<GuardReport> {
    run_all_timed(root).0
}

/// [`run_all`] with every guard's wall clock beside the reports - what the receipt stores (issue455).
#[must_use]
pub fn run_all_timed(root: &Path) -> (Vec<GuardReport>, Durations) {
    let act = keel_model::activation::Activation::load(root);
    // The activation filter runs first and serially: it decides WHICH guards run, and the answer for an
    // inactive one is a report, not a computation.
    let mut slots: Vec<Option<GuardReport>> = Vec::with_capacity(GUARD_NAMES.len());
    let mut to_run: Vec<(usize, &'static str)> = Vec::new();
    for n in GUARD_NAMES {
        // A process the project has not adopted: SKIP the check, but say so. Silence here would be
        // the issue090 defect inverted — instead of failing a project for a control it never
        // adopted, we would be passing it while hiding that the control is off (D0138).
        if let keel_model::activation::GuardState::Inactive(p) = act.guard_state(n) {
            slots.push(Some(GuardReport {
                name: n,
                scanned: 0,
                warnings: vec![format!(
                    "NOT ACTIVE — process `{p}` is not in this project's active set, so this control was NOT checked (D0138; `keel activate {p}` to adopt it)"
                )],
                violations: Vec::new(),
            }));
        } else {
            to_run.push((slots.len(), n));
            slots.push(None);
        }
    }
    // DISPATCH ORDER IS DECLARATION ORDER, on measurement (dcGuardPoolDispatchesLongestFirst, issue455,
    // 2026-09-10). Longest-first from the receipt's per-guard durations was built and A/B-timed on this
    // host (14 cores / 20 threads), six interleaved pairs of `KEEL_PERF=2 keel gate guard --no-receipt`:
    // list-scheduling simulation over three measured runs had promised 1849 -> 1314 ms
    // (scripts/probes/guard_dispatch_order.py); the binary measured wall medians 2567 (declared) vs
    // 2544 ms (longest-first) - one percent, inside the run-to-run spread - while the guard sum rose
    // 21.3 -> 23.3 s, `git x40` 5.2 -> 7.3 s, and the longest guard itself, `acceptance-binds-to-text`,
    // stretched 1.55-1.68 -> 2.16-2.34 s. Dispatched first it runs beside the other git-spawning,
    // tree-walking guards and contends with them, so the makespan's lower bound grows by what the
    // schedule saves. The simulation assumed a guard's duration is independent of what runs beside it;
    // it is not. The floor is shared-resource contention (dcGitReadsStayInProcess, dcBatchGitReads),
    // not the schedule - so the pool takes `to_run` as declared, and the receipt keeps each guard's
    // `ms` so the next assessment reads history instead of promising a number. (Two earlier timings in
    // separate windows read 2.2 and 2.9 s for the same code: the host, not the order.)
    let durations = run_in_parallel(root, &to_run, &mut slots);
    (slots.into_iter().flatten().collect(), durations)
}

/// Run `to_run` across a fixed pool of OS threads, writing each guard's report into its slot
/// (dcGuardsRunInParallelAndTimed, resolving the serial term of issue409).
///
/// WHY A POOL AND NOT ONE THREAD PER GUARD: 65 guards on a 20-core host would oversubscribe the model
/// cache lock on a cold start; a pool the size of the host's parallelism does one parse and shares it.
/// WHY SLOTS: the reports come back in `GUARD_NAMES` order exactly as the serial loop returned them -
/// every consumer (the hook, `keel show status`, the workspace gate, the history audit) prints or compares
/// them positionally, and a thread finishing order is not a fact about the tree. Each guard is a
/// `perf::phase("guard:<name>")` so `KEEL_PERF=2` attributes the run per guard in-process without the
/// measurement patch the spike needed (docs/reviews/perf-spike-2026-09-07/phase_patch.py).
/// Guards are pure reads of the tree (no guard writes a file or sets process state - the one
/// `fs::write` in this module is in a test) so running them concurrently changes no answer.
/// THE CRITICAL PATH of the last guard run (issue429 / D0414): the longest single guard by wall clock.
///
/// The guards run in parallel, so the set's wall clock is bounded below by its longest member and by
/// nothing else - a sum of guard times is not a duration. Timed always, `KEEL_PERF` or not: one
/// `Instant` per guard is nothing against a guard, and both the receipt (which states it) and a slow
/// hook fire (which attributes to it) need the number without anyone having asked for a report.
static CRITICAL_PATH: std::sync::Mutex<Option<(&'static str, u64)>> = std::sync::Mutex::new(None);

/// Note one guard's wall clock against the critical path and return it in ms.
fn note_critical_path(name: &'static str, took: std::time::Duration) -> u64 {
    let ms = u64::try_from(took.as_millis()).unwrap_or(u64::MAX);
    if let Ok(mut g) = CRITICAL_PATH.lock() {
        if g.is_none_or(|(_, best)| ms > best) {
            *g = Some((name, ms));
        }
    }
    ms
}

/// Every guard's wall clock in ONE run, `(name, ms)`.
///
/// The receipt stores them as the run's measured
/// profile (issue455: the number a dispatch decision is judged against, read from history). A value
/// the run RETURNS, not process state: the first cut kept them in a static cleared per run, and two
/// `run_all`s in one process - which is what `cargo test` is - interleaved their pushes, so the count
/// held in a filtered test run and failed in the full one (the land refusal of 2026-09-10).
pub type Durations = Vec<(&'static str, u64)>;

/// The longest guard of the last run in this process, `(name, ms)`, or `None` when no guard has run.
#[must_use]
pub fn critical_path() -> Option<(&'static str, u64)> {
    CRITICAL_PATH.lock().ok().and_then(|g| *g)
}

/// The critical path as the receipt and the runner print it: `priority-inversion 2102 ms`.
#[must_use]
pub fn critical_path_line() -> String {
    critical_path().map(|(n, ms)| format!("{n} {ms} ms")).unwrap_or_default()
}

fn run_in_parallel(root: &Path, to_run: &[(usize, &'static str)], slots: &mut [Option<GuardReport>]) -> Durations {
    let workers = std::thread::available_parallelism().map_or(4, std::num::NonZeroUsize::get).min(to_run.len().max(1));
    let next = std::sync::atomic::AtomicUsize::new(0);
    let done: std::sync::Mutex<Vec<(usize, Option<GuardReport>, u64)>> = std::sync::Mutex::new(Vec::with_capacity(to_run.len()));
    std::thread::scope(|s| {
        for _ in 0..workers {
            let next = &next;
            let done = &done;
            // 16 MiB: a guard may parse the whole corpus on a cache miss, and a spawned thread's default
            // stack is a quarter of the main thread's on some hosts.
            let spawned = std::thread::Builder::new().stack_size(16 * 1024 * 1024).spawn_scoped(s, move || loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let Some(&(slot, name)) = to_run.get(i) else { break };
                let t0 = std::time::Instant::now();
                let report = keel_perf::perf::phase(&format!("guard:{name}"), || run_one(name, root));
                let ms = note_critical_path(name, t0.elapsed());
                if let Ok(mut d) = done.lock() {
                    d.push((slot, report, ms));
                }
            });
            // A host that refuses a thread gets the guards run on this one - slower, same answer.
            if spawned.is_err() {
                loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let Some(&(slot, name)) = to_run.get(i) else { break };
                    let t0 = std::time::Instant::now();
                    let report = keel_perf::perf::phase(&format!("guard:{name}"), || run_one(name, root));
                    let ms = note_critical_path(name, t0.elapsed());
                    if let Ok(mut d) = done.lock() {
                        d.push((slot, report, ms));
                    }
                }
            }
        }
    });
    let done = done.into_inner().unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut durations = Durations::with_capacity(done.len());
    for (slot, report, ms) in done {
        if let Some(cell) = slots.get_mut(slot) {
            if let Some(r) = cell.insert_report(report) {
                durations.push((r, ms));
            }
        }
    }
    durations
}

/// Fill a slot and hand back the guard's name, so a duration is recorded only for a report that exists.
trait SlotFill {
    fn insert_report(&mut self, report: Option<GuardReport>) -> Option<&'static str>;
}

impl SlotFill for Option<GuardReport> {
    fn insert_report(&mut self, report: Option<GuardReport>) -> Option<&'static str> {
        let name = report.as_ref().map(|r| r.name);
        *self = report;
        name
    }
}

/// The repository root for this crate's tests, found from the manifest's ancestors: a member's cwd
/// under `cargo test` is its own directory two levels down, so `..` and `src/...` name the wrong tree
/// (sprints 714, 718, 733; the control is touched.rs `no_member_test_anchors_on_a_cwd_relative_path`).
#[cfg(test)]
pub(crate) fn test_repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|a| a.join(".git").exists())
        .expect("a member crate sits inside the keel repository")
        .to_path_buf()
}

#[cfg(test)]
mod warning_population_tests {
    use super::{history_line, is_history, warning_population, GuardReport};

    fn report(name: &'static str, warnings: &[String]) -> GuardReport {
        GuardReport { name, scanned: 1, warnings: warnings.to_vec(), violations: Vec::new() }
    }

    /// issue404 / D0413, the probe pair: the KNOWN-POSITIVE is a warning built with `history_line`
    /// (counted history) and the KNOWN-NEGATIVE is a plain warning naming a fixable condition. The
    /// summary states the two classes apart, the actionable one first with its guards, and never
    /// merges the history count into it.
    #[test]
    fn a_counted_history_line_is_history_and_a_plain_warning_is_actionable() {
        let history = history_line("52 legacy actor reference(s) - immutable history");
        let plain = "task 'x' is not in deliverable-manifest.txt".to_string();
        assert!(is_history(&history) && !is_history(&plain));
        let a = report("actors", std::slice::from_ref(&history));
        let m = report("manifest-coverage", &[plain.clone(), plain]);
        assert_eq!(a.actionable().count(), 0);
        assert_eq!(a.history().count(), 1);
        assert_eq!(m.actionable().count(), 2);
        let s = warning_population(&[a, m]);
        assert!(s.starts_with(" — 2 actionable warning(s) across 1 guard(s)"), "{s}");
        assert!(s.contains("manifest-coverage") && s.contains("1 counted-history line(s) (actors)"), "{s}");
        assert!(!s.contains("3 "), "the two classes are never summed: {s}");
    }

    #[test]
    fn the_summary_is_empty_with_no_warnings_and_names_only_the_class_present() {
        assert_eq!(warning_population(&[report("a", &[])]), "");
        let only_history = warning_population(&[report("issues", &[history_line("111 resolutions before the cutoff")])]);
        assert!(only_history.starts_with(" — 1 counted-history line(s) (issues)"), "{only_history}");
        assert!(!only_history.contains("actionable"), "{only_history}");
        let only_live = warning_population(&[report("viewpoint-renderer", &["baselines: renderer planned".to_string()])]);
        assert!(only_live.contains("1 actionable warning(s)") && !only_live.contains("counted-history"), "{only_live}");
    }

    /// The live tree: the actor guard's legacy line carries the mark, so the classification is the
    /// guard's declaration and not a reading of the prose.
    #[test]
    fn the_live_actor_guard_counts_its_legacy_line_as_history() {
        let r = super::actors(std::path::Path::new("../.."));
        assert!(r.history().any(|w| w.contains("legacy actor reference(s)")), "{:?}", r.warnings);
        assert_eq!(r.actionable().count(), 0, "{:?}", r.warnings);
    }
}

#[cfg(test)]
mod issue_naming_tests {
    /// issue333 / D0304: a resolver that never names its issue - and whose issue never names it - is
    /// a violation for an issue created on/after the cutoff, a counted history line before it; either
    /// direction of naming satisfies the check; the shape issue323 had (carried-over resolver, neither
    /// text mentioning the other) is what fires.
    #[test]
    fn a_resolver_that_names_neither_way_is_a_mistriage_forward_only() {
        let root = std::env::temp_dir().join(format!("keel-issuename-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".tracking")).expect("mkdir");
        let iss = |n: &str, created: &str, desc: &str| {
            format!("    part {n} : Issue {{ :>> id = \"00000000-0000-4000-8000-0000000{}\"; :>> title = \"t\"; :>> createdAt = \"{created}\"; :>> createdBy = \"bot\"; :>> description = \"{desc}\"; :>> severity = Severity::Low; }}
", &n[5..])
        };
        let text = format!(
            "package Fx {{
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    action def Build {{
        action dcNamesIt;
        verification dcNamesItDoD : Test {{ :>> id = \"00000000-0000-4000-8000-000000000001\"; :>> method = VerificationMethod::test; :>> procedureText = \"resolves issue901 by doing the thing\"; }}
        action dcCarriedOver;
        verification dcCarriedOverDoD : Test {{ :>> id = \"00000000-0000-4000-8000-000000000002\"; :>> method = VerificationMethod::test; :>> procedureText = \"an unrelated task\"; }}
        action dcNamedByIssue;
        verification dcNamedByIssueDoD : Test {{ :>> id = \"00000000-0000-4000-8000-000000000003\"; :>> method = VerificationMethod::test; :>> procedureText = \"another task\"; }}
    }}
{}{}{}{}    #Resolves dependency from dcNamesIt to issue901;
    #Resolves dependency from dcCarriedOver to issue902;
    #Resolves dependency from dcNamedByIssue to issue903;
    #Resolves dependency from dcCarriedOver to issue904;
}}
",
            iss("issue901", "2026-09-10", "the resolver names me"),
            iss("issue902", "2026-09-10", "nothing here names the resolver - the issue323 shape"),
            iss("issue903", "2026-09-10", "PREVENTING CHANGE: dcNamedByIssue"),
            iss("issue904", "2026-01-01", "old, unnamed both ways - history"),
        );
        std::fs::write(root.join(".tracking").join("fx.sysml"), text).expect("fixture");
        let (scanned, forward, historical) = keel_view::view::unnamed_resolutions(&root, "2026-09-04").expect("model");
        assert_eq!(scanned, 4);
        assert_eq!(forward, vec![("issue902".to_string(), "dcCarriedOver".to_string())], "only the issue323 shape, forward of the cutoff, is a violation");
        assert_eq!(historical, 1, "the pre-cutoff miss is counted, not reported");
        let _ = std::fs::remove_dir_all(&root);
    }
}

#[cfg(test)]
mod guard_catalogue_tests {
    /// D0195 clause 1 / issue370: EVERY enforced guard has a constraint-def identity in
    /// `.engine/rules/guard-constraints.sysml` (its camelCased name). Twelve did not, for weeks, while
    /// the file said all did - because nothing computed the two lists against each other.
    #[test]
    fn every_guard_has_a_constraint_def_identity() {
        let text = keel_model::corpus::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../.engine/rules/guard-constraints.sysml")).expect("guard-constraints.sysml");
        let declared: std::collections::HashSet<String> = text
            .lines()
            .filter_map(|l| l.trim().strip_prefix("constraint def "))
            .map(|r| r.split(|c: char| c == ';' || c.is_whitespace()).next().unwrap_or_default().to_string())
            .collect();
        let camel = |k: &str| {
            let mut out = String::new();
            for (i, part) in k.split('-').enumerate() {
                if i == 0 {
                    out.push_str(part);
                } else if let Some(c) = part.chars().next() {
                    out.push(c.to_ascii_uppercase());
                    out.push_str(&part[c.len_utf8()..]);
                }
            }
            out
        };
        let missing: Vec<&str> = super::GUARD_NAMES.iter().copied().filter(|g| !declared.contains(&camel(g))).collect();
        assert!(missing.is_empty(), "guards with no constraint-def identity (D0195 clause 1): {missing:?}");
    }

    /// THE CONTROL for the catalogue: every enforced guard has a row in `.engine/docs/guards.md`. Six
    /// guards had none on 2026-09-02 - five of them older than a week - because `doc-guard-count` polices
    /// the COUNT claim and nothing policed the rows. A guard nobody can look up is a refusal nobody can
    /// act on.
    #[test]
    fn every_guard_has_a_catalogue_row() {
        let md = keel_model::corpus::read_to_string(crate::test_repo_root().join(".engine/docs/guards.md")).expect("guards.md ships with the engine");
        let missing: Vec<&str> = super::GUARD_NAMES.iter().copied().filter(|n| !md.contains(&format!("| `{n}` |"))).collect();
        assert!(missing.is_empty(), "guards with no row in .engine/docs/guards.md: {missing:?}");
    }
}

#[cfg(test)]
mod scan_count_tests {
    use super::{GuardReport, GUARD_NAMES};

    /// No guard may be written to report violations against a zero scan count (issue180). The RUNNER
    /// surfaces it at print time through `GuardReport::self_contradiction`; a silent regression there
    /// makes `scanned` untrustworthy again and untrustworthy numbers get used. Asserted on the value,
    /// not on the spelling of the check (D0401).
    #[test]
    fn the_runner_flags_a_violation_against_an_empty_scan() {
        let report = |scanned: usize, violations: &[&str]| GuardReport {
            name: "probe",
            scanned,
            warnings: vec![],
            violations: violations.iter().map(|v| (*v).to_string()).collect(),
        };
        let flagged = report(0, &["x"]).self_contradiction().expect("0 scanned with a violation is the contradiction");
        assert!(flagged.contains("probe") && flagged.contains("1 violation(s)") && flagged.contains("issue180"), "{flagged}");
        assert_eq!(report(0, &[]).self_contradiction(), None, "an empty population with nothing found is legitimate");
        assert_eq!(report(3, &["x"]).self_contradiction(), None, "a violation in a population is a finding, not a contradiction");
        assert_eq!(report(3, &[]).self_contradiction(), None);
    }

    /// Every guard reporting a real population today keeps reporting one. Guards whose population is
    /// legitimately empty at this commit are excluded BY NAME rather than by a blanket allowance, so a
    /// guard silently going quiet cannot hide behind them.
    #[test]
    fn the_guards_that_report_a_population_still_do() {
        const LEGITIMATELY_EMPTY: [&str; 15] = [
            // scans priority-ordering pairs on the ready frontier; D0189's scope closure emptied
            // the frontier down to a handful of same-rank resolvers, so there is nothing to order.
            // The population returns the moment the backlog holds ranked work again.
            "priority-inversion",
            // SHALLOW-CLONE dependent: claim-ancestry skips loudly when history is unavailable, which
            // is correct (a depth-dependent verdict is the K15 machine-dependence it exists to
            // prevent) but means its population is zero in any job that checks out shallow. Guard 51
            // now refuses a GATING workflow that does so; this test runs in jobs that may not be one.
            "claim-ancestry",
            // both D0191 guards scan ENVIRONMENT-DEPENDENT populations: release-recorded scans git
            // tags (absent in CI's tag-less clone) and enrollment-binding scans the machine-local
            // .keel/actor binding (gitignored, absent on CI) - locally both scan real populations.
            "release-recorded",
            "enrollment-binding",
            // scans done-work -> SR pairs lacking a #Verify edge; sprints 393/394 verified every
            // live SR (verification lens: neither = 0), so the population emptied by completion.
            "verification-trace",
            "charter",                   // scans CHANGED files
            "process-change",            // scans CHANGED process definitions
            "judgment-request-quality",  // scans PROPOSED fork decisions - zero between forks is the healthy state (D0207: most decisions auto-accept and never ask)
            "retro-backlog",             // scans retro findings needing a backlog item
            "doc-sync",                  // scans CHANGED doc surface
            "decision-amends-process",   // scans STAGED Decisions - zero between commits
            "unit-extras-present",       // scans declared unit extras; no installed unit declares any since the channel left (D0292)
            "control-defect-registry",   // scans registered control defects - EMPTY since 2026-09-04, when the last of D0278's three left with D0303 C; a registered defect is the unhealthy state
            "base-first-justification",  // scans new base-construct adoptions
            "ownership",                 // scans cross-owner edits
        ];
        let root = std::path::Path::new("../..");
        for name in GUARD_NAMES {
            if LEGITIMATELY_EMPTY.contains(&name) {
                continue;
            }
            let Some(r): Option<GuardReport> = super::run_one(name, root) else { continue };
            assert!(r.scanned > 0, "guard `{name}` reports a zero population - mis-aimed, or newly empty and needing an entry in LEGITIMATELY_EMPTY");
        }
    }
}

#[cfg(test)]
mod identity_form_tests {
    use super::{id_values, tool_reference, uuid_shaped};

    /// The shape predicate, on the two ids I actually mangled and the deliberate mnemonic convention it
    /// must NOT break. Written as a table because the interesting cases are the near-misses.
    #[test]
    fn the_shape_predicate_accepts_the_convention_and_rejects_the_real_defects() {
        for good in [
            "3a86f04d-2c19-4b57-9e80-64bc17ea5d38",
            "4b78e0c1-3f52-4d96-a814-000000000i01", // the intake process-step convention: shaped, not hex
            "0d800620-0814-6620-9b3e-000000300620",
        ] {
            assert!(uuid_shaped(good), "{good} is well shaped and must pass");
        }
        for bad in [
            "not-a-uuid-at-all",
            "3a86f04d-2c19-4b57-9e80-64bc17ea5device", // 15 chars in the last group
            "4b07d2f6-9e35-4c81-a period-placeholder", // a SPACE, which is how this was found
            "eb5ebf9-6a7b-4c8d-efa9-0b1c2d3e4f5a",     // 7 in the first group - the historical class
            "3a86f04d-2c19-4b57-9e80-64BC17EA5D38",    // uppercase: one id, two spellings, is not identity
            "",
        ] {
            assert!(!uuid_shaped(bad), "{bad:?} is malformed and must fail");
        }
    }

    /// A line may declare an item AND its result, so the scan must not stop at the first id. Missing
    /// this would make the guard blind to exactly the sprint records where most ids live.
    #[test]
    fn every_id_on_a_line_is_read_not_just_the_first() {
        let line = r#"part a : X { :>> id = "aaaaaaaa-1111-2222-3333-444444444444"; } part b : Y { :>> id = "bad"; }"#;
        assert_eq!(id_values(line), vec!["aaaaaaaa-1111-2222-3333-444444444444", "bad"]);
    }

    /// The grandfather set is an explicit LIST of 15, and its size is asserted so growing it is a
    /// deliberate edit to a failing test rather than a quiet accommodation. Guard 36's first version
    /// keyed its exemption on a DATE and thereby exempted the defect it existed to catch.
    #[test]
    fn the_grandfather_set_cannot_grow_quietly() {
        let src = keel_model::corpus::read_to_string(crate::test_repo_root().join("members/keel-guards/src/identity.rs")).expect("identity.rs is readable");
        let body = src
            .split_once("const GRANDFATHERED: [&str; 15]")
            .expect("the set is declared with its size, so a 16th entry does not compile")
            .1;
        let listed = body[..body.find("];").expect("the list is closed")].matches('"').count() / 2;
        assert_eq!(listed, 15, "the declared size and the actual entries must agree");
    }

    /// Guard 39: a living-surface reference to a missing tool is a violation; a reference to an
    /// existing tool and a bare directory mention are not; `.tracking` (history) is out of scope.
    #[test]
    fn tool_reference_flags_only_missing_files_on_the_living_surface() {
        let root = keel_fs::scratch("keel-toolref-guard");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".engine").join("skills")).expect("mkdir");
        std::fs::create_dir_all(root.join(".engine").join("tools")).expect("mkdir");
        std::fs::create_dir_all(root.join(".tracking")).expect("mkdir");
        std::fs::write(root.join(".engine").join("tools").join("real.py"), "# real").expect("write");
        std::fs::write(
            root.join(".engine").join("skills").join("s.md"),
            "run `.engine/tools/real.py` then .engine/tools/gone.py. See .engine/tools/ for more.\n",
        )
        .expect("write");
        std::fs::write(
            root.join(".tracking").join("h.sysml"),
            "// history may truthfully say .engine/tools/retired.py existed\n",
        )
        .expect("write");
        let report = tool_reference(&root);
        assert_eq!(report.scanned, 2, "two file references scanned (the bare directory mention is not one)");
        assert_eq!(report.violations.len(), 1, "{:?}", report.violations);
        assert!(report.violations[0].contains(".engine/tools/gone.py"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_receipt_is_demanded_of_the_ai_and_never_of_the_human() {
        // D0232. The exemption is the DESIGN, not a courtesy: governance binds the AI, and a human's
        // word IS the evidence. Tested in four directions, because a guard that only ever passes is
        // indistinguishable from one aimed at nothing, and one that fires on the human would point
        // the whole control at the wrong party.
        let root = keel_fs::scratch("keel-evidence-cited-test");
        let _ = std::fs::remove_dir_all(&root);
        let tr = root.join(".tracking");
        std::fs::create_dir_all(&tr).unwrap();
        std::fs::write(
            tr.join("actors.sysml"),
            "package A {\n    part wweatherholtz : Person { :>> name = \"W\"; }\n    part bot : Actor { :>> kind = ActorKind::ai; }\n}\n",
        )
        .unwrap();

        let res = |name: &str, by: &str, at: &str, ran: bool| {
            let receipt = if ran { "        // RAN: cargo test -> 3 passed\n" } else { "" };
            format!("{receipt}        part {name}R1 : TestResult {{ :>> id = \"x\"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = \"abc\"; :>> judgedAt = \"{at}\"; :>> judgedBy = \"{by}\"; }}\n")
        };
        let write = |body: &str| {
            std::fs::write(
                tr.join("s.sysml"),
                format!(
                    "package S {{\n    action def R {{\n        action t;\n        verification tDoD : Test {{ :>> id = \"y\"; :>> method = VerificationMethod::test; }}\n{body}    }}\n}}\n"
                ),
            )
            .unwrap();
        };

        // 1. AI, method=test, NO receipt -> refused.
        write(&res("tDoD", "bot", "2026-08-25", false));
        let r = process_and_report(&root);
        assert_eq!((r.scanned, r.violations.len()), (1, 1), "an AI test-claim with no receipt must be refused");

        // 2. AI, WITH a receipt -> clean.
        write(&res("tDoD", "bot", "2026-08-25", true));
        let r = process_and_report(&root);
        assert_eq!((r.scanned, r.violations.len()), (1, 0), "a receipt satisfies it");

        // 3. HUMAN, no receipt -> NOT EVEN SCANNED. Their word is the evidence.
        write(&res("tDoD", "wweatherholtz", "2026-08-25", false));
        let r = process_and_report(&root);
        assert_eq!((r.scanned, r.violations.len()), (0, 0), "a human's attestation is out of scope entirely");

        // 4. AI, but dated BEFORE the cutover -> out of scope; retro-fitting evidence nobody
        //    captured would mean inventing it, which is the failure this guard exists to prevent.
        write(&res("tDoD", "bot", "2026-08-01", false));
        let r = process_and_report(&root);
        assert_eq!((r.scanned, r.violations.len()), (0, 0), "the guard binds forward only");
        let _ = std::fs::remove_dir_all(&root);
    }

    fn process_and_report(root: &std::path::Path) -> GuardReport {
        super::evidence_cited(root)
    }

    #[test]
    fn a_process_that_cannot_say_when_it_applies_is_refused() {
        // D0225. Both directions, because a guard that only ever passes is indistinguishable from a
        // guard that does nothing - and this codebase has already shipped two checks that passed on
        // an empty population (issue250, claude-surface-drift on zero skills).
        let root = keel_fs::scratch("keel-guard-applicability");
        let _ = std::fs::remove_dir_all(&root);
        let dir = root.join(".engine").join("processes");
        std::fs::create_dir_all(&dir).unwrap();

        // Out of scope until the project adopts project-onboarding (issue259): the fact serves that
        // process, and a project that never adopted it has violated nothing (D0164).
        std::fs::write(dir.join("other.sysml"), "package O {
    action o : Process {
        :>> id = \"y\";
    }
}
").unwrap();
        assert_eq!(process_applicability(&root).scanned, 0, "not adopted -> out of scope, and it says 0 scanned");
        std::fs::write(dir.join("project-onboarding.sysml"), "package PO {
    action po : Process {
        :>> id = \"z\";
        // APPLIES-WHEN: adopted
    }
}
").unwrap();
        std::fs::remove_file(dir.join("other.sysml")).unwrap();

        let declares = "package P {
    action p : Process {
        :>> id = \"x\";
";
        std::fs::write(dir.join("with.sysml"), format!("{declares}        // APPLIES-WHEN: a stated situation
    }}
}}
")).unwrap();
        let r = process_applicability(&root);
        assert_eq!((r.scanned, r.violations.len()), (2, 0), "both declared conditions pass");

        std::fs::write(dir.join("without.sysml"), format!("{declares}    }}
}}
")).unwrap();
        let r = process_applicability(&root);
        assert_eq!(r.scanned, 3, "every process file is in scope once adopted");
        assert_eq!(r.violations.len(), 1, "the one lacking a condition is refused");
        assert!(r.violations[0].contains("without.sysml"), "{:?}", r.violations);

        // A file that declares no Process at all is out of scope, not a violation.
        std::fs::write(dir.join("helper.sysml"), "package H {
    private import X::*;
}
").unwrap();
        assert_eq!(process_applicability(&root).scanned, 3, "a non-Process file is not scanned");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// D0209 clause 2 coverage audit, made executable: every file that DEFINES a guard (`-> GuardReport`)
    /// must sit inside the enforcement-surface lock, so a new guard file cannot be added OUTSIDE it and
    /// thereby be editable without a signed Decision. Scans the real `src/` tree of EVERY workspace member
    /// (sprint 733: the guards are a member, and a guard written into any other member is the same hole)
    /// rather than trusting the hand-list in `GUARD_SOURCE_FILES` / `GUARD_SOURCE_DIRS` -- if the two
    /// diverge, this fails CI, which is the point.
    #[test]
    fn enforcement_surface_covers_every_guard_source() {
        fn collect(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
            let Ok(entries) = std::fs::read_dir(dir) else { return };
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    collect(&p, out);
                } else if p.extension().is_some_and(|x| x == "rs") {
                    out.push(p);
                }
            }
        }
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap();
        let mut files = Vec::new();
        let manifest = keel_model::corpus::read_to_string(repo.join("Cargo.toml")).expect("the workspace manifest");
        let members = keel_model::corpus::workspace_members(&manifest);
        assert!(members.len() >= 12, "the workspace manifest lists its members: {members:?}");
        for member in &members {
            collect(&repo.join(member).join("src"), &mut files);
        }
        assert!(files.iter().any(|f| f.ends_with("identity.rs")), "the scan reached this member's own files");
        let mut uncovered = Vec::new();
        for f in files {
            let Ok(text) = keel_model::corpus::read_to_string(&f) else { continue };
            if !text.contains("-> GuardReport") {
                continue;
            }
            let repo_rel = f.strip_prefix(repo).unwrap().to_string_lossy().replace('\\', "/");
            if !is_enforcement_surface(&repo_rel) {
                uncovered.push(repo_rel);
            }
        }
        assert!(
            uncovered.is_empty(),
            "guard-defining file(s) OUTSIDE the enforcement-surface lock (add to GUARD_SOURCE_FILES / GUARD_SOURCE_DIRS): {uncovered:?}"
        );
    }

    #[test]
    fn enforcement_surface_locks_workflows_hooks_and_guard_source() {
        assert!(is_enforcement_surface(".github/workflows/ci.yml"));
        assert!(is_enforcement_surface(".githooks/pre-commit"));
        assert!(is_enforcement_surface("members/keel-guards/src/lib.rs"));
        assert!(is_enforcement_surface("members/keel-guards/src/identity.rs"));
        assert!(is_enforcement_surface("members/keel-guards/src/receipt.rs"));
        assert!(is_enforcement_surface("keel-cli/src/adherence.rs"));
        assert!(is_enforcement_surface("members/keel-schema/src/guard_names.rs"));
        // NOT locked: ordinary source, docs, a workflow-shaped path outside the dir, the old path.
        assert!(!is_enforcement_surface("keel-cli/src/main.rs"));
        assert!(!is_enforcement_surface("keel-cli/src/guards.rs"));
        assert!(!is_enforcement_surface("members/keel-guards/Cargo.toml"));
        assert!(!is_enforcement_surface(".engine/docs/guards.md"));
        assert!(!is_enforcement_surface("README.md"));
    }

    /// Every enforced guard must actually DISPATCH. `run_one` was a hand-written match, so a name could sit
    /// in `GUARD_NAMES` -- counted in the control inventory, listed in `--help`, documented in guards.md --
    /// while `run_one` returned `None` for it and `run_all` silently ran 35 of 36. Since sprint 733 dispatch
    /// is the family tables', so the check reads the tables (no source-text scan, no model build); the
    /// union test in `family_union_tests` holds the converse, that no table names a guard outside the list.
    #[test]
    fn every_enforced_guard_dispatches() {
        for name in GUARD_NAMES {
            assert!(
                super::FAMILIES.iter().flat_map(|f| f.arms.iter()).any(|(n, _)| *n == name),
                "guard `{name}` is in GUARD_NAMES but no family table dispatches it -- it would be counted in the control inventory and never actually run"
            );
        }
    }

    #[test]
    fn actor_refs_extracted() {
        let line = "    part x { :>> authoredBy = \"ana\"; :>> judgedBy = \"bob\"; :>> title = \"z\"; }";
        assert_eq!(scan_actor_refs(line), vec!["ana".to_string(), "bob".to_string()]);
    }

    #[test]
    fn doc_sync_flags_undocumented_definitional_change() {
        // D0113: a schema/process/workflow change with no co-committed doc = warn; with a doc = clean.
        let warn = doc_sync_warnings(&[".engine/processes/foo.sysml".to_string(), "keel-cli/src/x.rs".to_string()]);
        assert_eq!(warn.len(), 1, "definitional change + no doc must warn: {warn:?}");
        assert!(doc_sync_warnings(&[".engine/processes/foo.sysml".to_string(), "CLAUDE.md".to_string()]).is_empty());
        assert!(doc_sync_warnings(&[".engine/schema/core.sysml".to_string(), ".engine/docs/guide.md".to_string()]).is_empty());
        assert!(doc_sync_warnings(&[".tracking/backlog.sysml".to_string()]).is_empty()); // nothing definitional
    }

    #[test]
    fn engine_lint_counts_tracked_instances() {
        // D0112 phase 1: only `part|verification|requirement <name> : <IdType>` count toward the
        // missing-id check; non-ID types (SystemRequirement) and non-instance lines don't.
        let text = "package P {\n    part d1 : Decision { :>> id = \"x\"; }\n    verification t1 : Test { :>> id = \"y\"; }\n    requirement r1 : SystemRequirement {}\n    part note : SomeOtherType {}\n    action a1;\n}";
        assert_eq!(count_tracked_instances(text), 2); // Decision + Test only
    }

    #[test]
    fn viewpoint_renderer_classification() {
        // D0056/issue034: retired-tool refs + unknown commands are violations; planned is a warning;
        // a real keel subcommand is ok.
        assert_eq!(classify_renderer("query.py governing-version <item>"), "retired");
        assert_eq!(classify_renderer("report.py:tab_decisions"), "retired");
        assert_eq!(classify_renderer("(planned) baselines view — not yet rendered"), "planned");
        assert_eq!(classify_renderer("keel render model (interactive HTML #View)"), "ok");
        assert_eq!(classify_renderer("keel render report <assurance|...> [--html]"), "ok");
        assert_eq!(classify_renderer("keel frobnicate"), "unknown");
        assert_eq!(classify_renderer("some hand-wave"), "unknown");
    }

    #[test]
    fn manifest_parses_per_task_entries() {
        // D0050/issue033: `task: NAME | p1 p2` lines parse to (name, paths); comments/blanks skipped.
        let text = "# header comment\n\ntask: rustS1Lexer | keel-parser/src/lexer.rs keel-parser/src/token.rs\ntask: writeApi | keel-cli/src/write.rs\n";
        let entries = parse_manifest(text);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].0, "rustS1Lexer");
        assert_eq!(entries[0].1, vec!["keel-parser/src/lexer.rs".to_string(), "keel-parser/src/token.rs".to_string()]);
        assert_eq!(entries[1], ("writeApi".to_string(), vec!["keel-cli/src/write.rs".to_string()]));
    }

    #[test]
    fn dodr_task_stripped() {
        assert_eq!(strip_dodr("portOrphansAuditDoDR1"), Some("portOrphansAudit".to_string()));
        assert_eq!(strip_dodr("portOrphansAuditDoDR12"), Some("portOrphansAudit".to_string()));
        assert_eq!(strip_dodr("fooRefineGateR1"), None);
        assert_eq!(strip_dodr("DoDR1"), None);
    }

    #[test]
    fn sprint_coverage_selftest() {
        // Mirrors validate_sprint_coverage.selftest: covered passes, orphan flagged.
        let backlog = "action fakeCovered;\npart fakeCoveredDoDR1 : TestResult { :>> outcome = VerdictKind::pass; }\naction fakeOrphan;\npart fakeOrphanDoDR1 : TestResult { :>> outcome = VerdictKind::pass; }\n";
        let done = done_tasks(backlog);
        assert!(done.contains("fakeCovered") && done.contains("fakeOrphan"));
        let blob = "package ProjectDeliveryX { part s : Story { :>> title = \"delivers fakeCovered\"; } }";
        let grandfathered: HashSet<&str> = HashSet::new();
        let uncovered: Vec<String> = done.iter().filter(|t| !blob.contains(t.as_str()) && !grandfathered.contains(t.as_str())).cloned().collect();
        assert_eq!(uncovered, vec!["fakeOrphan".to_string()]);
    }

    fn strs(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn ceremony_ordering_violation_detected() {
        // Implement passed while Standup (defined) is unpassed -> violation.
        let order = strs(&["Refine", "Standup", "Implement", "Review", "CloseOut", "Retro"]);
        let defined: HashSet<String> = strs(&["Refine", "Standup", "Implement"]).into_iter().collect();
        let passed: HashSet<String> = strs(&["Refine", "Implement"]).into_iter().collect(); // Standup skipped
        let v = ordering_violations(&order, &defined, &passed);
        assert_eq!(v, vec![("Implement".to_owned(), "Standup".to_owned())]);
    }

    /// D0437 / issue470: a PROPOSED earlier gate is recorded in sequence (known positive: no violation);
    /// an earlier gate with no result at all is not (known negative: violation). `gate_passed` keeps
    /// reading the proposed gate as unpassed - the guard's reader changed, orient's did not.
    #[test]
    fn ceremony_order_reads_recorded_gates_not_passed_ones() {
        let order = strs(&["Refine", "Standup", "Implement", "Review", "CloseOut", "Retro"]);
        let proposed_then_pass = "verification xRefineGate : Test { }\n\
            part xRefineGateR1 : TestResult { :>> outcome = VerdictKind::proposed; }\n\
            verification xStandupGate : Test { }\n\
            part xStandupGateR1 : TestResult { :>> outcome = VerdictKind::proposed; }\n\
            verification xImplementGate : Test { }\n\
            part xImplementGateR1 : TestResult { :>> outcome = VerdictKind::pass; }\n";
        let recorded = gates_recorded(proposed_then_pass, &order);
        assert_eq!(recorded.len(), 3, "{recorded:?}");
        let mut defined = gates_defined(proposed_then_pass, &order);
        defined.extend(recorded.iter().cloned());
        assert!(ordering_violations(&order, &defined, &recorded).is_empty(), "a proposed gate is recorded in its turn");
        assert!(!keel_model::textscan::gate_passed(proposed_then_pass, "Refine"), "but it is still not PASSED");
        assert!(keel_model::textscan::gate_passed(proposed_then_pass, "Implement"));

        let missing_then_pass = "verification xRefineGate : Test { }\n\
            verification xStandupGate : Test { }\n\
            part xStandupGateR1 : TestResult { :>> outcome = VerdictKind::proposed; }\n\
            verification xImplementGate : Test { }\n\
            part xImplementGateR1 : TestResult { :>> outcome = VerdictKind::pass; }\n";
        let recorded = gates_recorded(missing_then_pass, &order);
        let mut defined = gates_defined(missing_then_pass, &order);
        defined.extend(recorded.iter().cloned());
        let v = ordering_violations(&order, &defined, &recorded);
        assert_eq!(
            v,
            vec![("Standup".to_owned(), "Refine".to_owned()), ("Implement".to_owned(), "Refine".to_owned())],
            "a gate with NO result is unrecorded, whatever comes after it"
        );

        // A proposed Retro owes its scan evidence exactly as a passed one does.
        let mut retro: HashSet<String> = HashSet::new();
        retro.insert("Retro".to_owned());
        let without = "verification xRetroGate : Test { :>> procedureText = \"rubber stamp\"; }";
        assert_eq!(retro_scan_missing(without, &retro).as_deref(), Some("xRetroGate"));
    }

    /// issue544 (D0437's clause completed): a `CloseOut` recorded FAIL followed by a `Retro` recorded pass is
    /// in sequence - known positive, no violation, and `gate_passed` still reads that `CloseOut` as not
    /// passed; a `CloseOut` with NO result followed by a `Retro` is still a violation - known negative. This
    /// is sprint708 at 593147c, whose closeOut clause (CI success) could not hold on issue542 and was
    /// recorded as the fail it was; the guard then called its Retro out of turn.
    #[test]
    fn ceremony_order_counts_a_failed_gate_as_recorded() {
        let order = strs(&["Refine", "Standup", "Implement", "Review", "CloseOut", "Retro"]);
        let failed_then_pass = "verification xCloseOutGate : Test { }\n\
            part xCloseOutGateR1 : TestResult { :>> outcome = VerdictKind::fail; }\n\
            verification xRetroGate : Test { :>> procedureText = \"Avoidable issues scanned\"; }\n\
            part xRetroGateR1 : TestResult { :>> outcome = VerdictKind::pass; }\n";
        let recorded = gates_recorded(failed_then_pass, &order);
        assert_eq!(recorded.len(), 2, "{recorded:?}");
        let mut defined = gates_defined(failed_then_pass, &order);
        defined.extend(recorded.iter().cloned());
        assert!(ordering_violations(&order, &defined, &recorded).is_empty(), "a failed gate was recorded in its turn");
        assert!(!keel_model::textscan::gate_passed(failed_then_pass, "CloseOut"), "but it is still not PASSED");
        assert!(!keel_model::textscan::gate_recorded(failed_then_pass, "CloseOut"), "and the flow view's finish reader still refuses it");

        let missing_then_pass = "verification xCloseOutGate : Test { }\n\
            verification xRetroGate : Test { :>> procedureText = \"Avoidable issues scanned\"; }\n\
            part xRetroGateR1 : TestResult { :>> outcome = VerdictKind::pass; }\n";
        let recorded = gates_recorded(missing_then_pass, &order);
        let mut defined = gates_defined(missing_then_pass, &order);
        defined.extend(recorded.iter().cloned());
        assert_eq!(
            ordering_violations(&order, &defined, &recorded),
            vec![("Retro".to_owned(), "CloseOut".to_owned())],
            "a gate with NO result is unrecorded"
        );
    }

    /// The live tree's order is the delivery chain, and the guard's per-file helpers read it (D0435).
    #[test]
    fn ceremony_reads_the_order_from_the_tree() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let order = keel_model::orient::gate_order(&root);
        assert_eq!(order, strs(&["Refine", "Standup", "Implement", "Review", "CloseOut", "Retro"]));
        let text = "verification xRefineGate : Test { }\nverification xStandupGate : Test { }\n";
        let defined = gates_defined(text, &order);
        assert_eq!(defined.len(), 2, "{defined:?}");
        assert!(gates_defined(text, &[]).is_empty(), "no order, nothing defined against it");
    }

    #[test]
    fn retro_scan_evidence_required() {
        let mut passed: HashSet<String> = HashSet::new();
        passed.insert("Retro".to_owned());
        let with = "verification xRetroGate : Test { :>> procedureText = \"no avoidable issue found\"; }";
        let without = "verification xRetroGate : Test { :>> procedureText = \"rubber stamp\"; }";
        assert!(retro_scan_missing(with, &passed).is_none());
        // issue566: the finding names the TEST read, so the violation line can name it and not the result.
        assert_eq!(retro_scan_missing(without, &passed).as_deref(), Some("xRetroGate"));
        let unrecorded: HashSet<String> = HashSet::new();
        assert!(retro_scan_missing(without, &unrecorded).is_none(), "an unrecorded Retro owes nothing yet");

        // Regression: "RetroGate" mentioned in an EARLIER gate's prose must not be mistaken for
        // the retro verification (the bug the unified runner caught on sprint56).
        let prose_then_real = "verification xStandupGate : Test { :>> procedureText = \"approach: retro_scan_missing (RetroGate prose)\"; }\nverification xRetroGate : Test { :>> procedureText = \"no avoidable issue\"; }";
        assert!(retro_scan_missing(prose_then_real, &passed).is_none());
    }

    // charter_selftest retired (D0107 CONTRACT): the charter guard now sources from charterRule; its
    // logic + parity are covered by view::tests::edge_rule_newly_added_scope_restricts_to_staged_files.

    #[test]
    fn keystone_selftest() {
        // Mirrors validate_process_change.selftest (incl. prose-marker-does-not-count).
        let marked = "package D {\n    #ProspectiveChange part d99 : Decision { :>> id = \"x\"; }\n}";
        let plain = "package D {\n    part d98 : Decision { :>> id = \"y\"; }\n}";
        let prose = "package D {\n    part d97 : Decision {\n        :>> decision = \"example: #ProspectiveChange part dNNNN : Decision { ... }\";\n    }\n}";

        let pos = keystone_violations(
            &[".engine/workflows/delivery.sysml".to_string(), ".engine/decisions/0099-x.sysml".to_string()],
            &[(".engine/decisions/0099-x.sysml".to_string(), marked.to_string())],
            &[],
        );
        let neg = keystone_violations(
            &[".engine/processes/agile-workflow.sysml".to_string(), ".engine/decisions/0098-y.sysml".to_string()],
            &[(".engine/decisions/0098-y.sysml".to_string(), plain.to_string())],
            &[],
        );
        let neg2 = keystone_violations(&[".engine/processes/agile-workflow.sysml".to_string()], &[], &[]);
        let neutral = keystone_violations(&[".tracking/backlog.sysml".to_string()], &[], &[]);
        let prose_only = keystone_violations(
            &[".engine/workflows/delivery.sysml".to_string(), ".engine/decisions/0097-z.sysml".to_string()],
            &[(".engine/decisions/0097-z.sysml".to_string(), prose.to_string())],
            &[],
        );

        assert!(pos.is_empty(), "marked Decision co-committed -> pass");
        assert_eq!(neg.len(), 1, "unmarked Decision -> fail");
        assert_eq!(neg2.len(), 1, "no Decision -> fail");
        assert!(neutral.is_empty(), "no process-def -> silent");
        assert_eq!(prose_only.len(), 1, "prose marker does NOT count");
    }

    /// D0465 / issue517, the D0388 pair. Positive: a locked edit with a co-committed sprint record
    /// chartered by a marked Decision a HUMAN accepted passes. Negative: the same charter proposed,
    /// auto-accepted under standing consent, or unmarked authorises nothing, and the lock refuses.
    #[test]
    fn keystone_accepts_a_human_accepted_marked_charter_and_nothing_weaker() {
        let sprint = "package S {\n    private import EngineRelationships::*;\n    #CharteredBy dependency from story to d0900;\n    part story : Story { :>> id = \"s\"; }\n}";
        assert_eq!(charter_targets(sprint), vec!["d0900".to_string()], "the charter edge names its Decision");
        assert!(charter_targets("package S {\n    :>> decision = \"#CharteredBy dependency from story to d0900;\";\n}").is_empty(), "prose is not an edge");

        let human = "package D {\n    #ProspectiveChange part d0900 : Decision { :>> id = \"x\"; :>> status = DecisionStatus::accepted; }\n    verification d0900Accept : Test { :>> method = VerificationMethod::confirmation; :>> procedureText = \"their words: 'yes, do it'\"; }\n    part d0900AcceptR1 : TestResult { :>> outcome = VerdictKind::pass; :>> judgedBy = \"person\"; }\n}";
        let auto = human.replace("their words: 'yes, do it'", "AUTO-ACCEPTED under standing consent (D0291)");
        let proposed = "package D {\n    #ProspectiveChange part d0900 : Decision { :>> id = \"x\"; :>> status = DecisionStatus::proposed; }\n}";
        let unmarked = human.replace("#ProspectiveChange part", "part");
        assert!(is_authorising_charter(human, "d0900"), "known-positive: marked and human-accepted");
        assert!(!is_authorising_charter(&auto, "d0900"), "known-negative: auto-accepted confers nothing");
        assert!(!is_authorising_charter(proposed, "d0900"), "known-negative: proposed is a plan");
        assert!(!is_authorising_charter(&unmarked, "d0900"), "known-negative: an unmarked Decision is not a process change");

        let locked = [".engine/skills/test-result/SKILL.md".to_string(), ".tracking/delivery/sprint692_x.sysml".to_string()];
        let charter = [("d0900".to_string(), ".tracking/delivery/sprint692_x.sysml".to_string())];
        assert!(keystone_violations(&locked, &[], &charter).is_empty(), "accepted charter co-committed -> pass");
        let refused = keystone_violations(&locked, &[], &[]);
        assert_eq!(refused.len(), 1, "no charter, no Decision -> fail");
        assert!(refused[0].contains("D0465"), "the refusal names the second path");
    }

    #[test]
    fn process_skill_flags_inert_and_dangling() {
        let procs = vec!["doc-sync.sysml".to_string(), "lonely.sysml".to_string()];
        let reg = "purpose = \"deploying skill for .engine/processes/doc-sync.sysml.\"\npurpose = \"for .engine/processes/ghost.sysml (dangling)\"";
        let v = process_skill_violations(&procs, reg);
        assert!(v.iter().any(|m| m.contains("lonely.sysml") && m.contains("NO deploying skill")), "inert process flagged");
        assert!(v.iter().any(|m| m.contains("ghost.sysml") && m.contains("dangling")), "dangling claim flagged");
        // doc-sync.sysml is referenced -> not flagged as inert.
        assert!(!v.iter().any(|m| m.contains("doc-sync.sysml") && m.contains("NO deploying skill")));
        // All real -> clean.
        let clean = process_skill_violations(&["doc-sync.sysml".to_string()], "x .engine/processes/doc-sync.sysml, y");
        assert!(clean.is_empty(), "every process referenced -> clean");
    }

    #[test]
    fn duplicate_scan_catches_each_identity_class() {
        // issue074/D0129. The danger is that NONE of these produce a git conflict when the two
        // claims live in different files, so without this guard the corruption lands green.
        let a = (
            "a.sysml".to_string(),
            "package P {\n    part x : Need { :>> id = \"11111111-1111-4111-9111-111111111111\"; }\n}".to_string(),
        );
        // Same id in a different file -> collision of identity itself.
        let dup_id = (
            "b.sysml".to_string(),
            "package Q {\n    part y : Need { :>> id = \"11111111-1111-4111-9111-111111111111\"; }\n}".to_string(),
        );
        // Same package name in a different file -> the registry silently MERGES these.
        let dup_pkg = (
            "c.sysml".to_string(),
            "package P {\n    part z : Need { :>> id = \"22222222-2222-4222-9222-222222222222\"; }\n}".to_string(),
        );
        // Same declared name inside one package.
        let dup_name = (
            "d.sysml".to_string(),
            "package R {\n    part dup : Need { :>> id = \"33333333-3333-4333-9333-333333333333\"; }\n    part dup : Need { :>> id = \"44444444-4444-4444-9444-444444444444\"; }\n}".to_string(),
        );

        let (_, v_id) = duplicate_scan(&[a.clone(), dup_id]);
        assert!(v_id.iter().any(|m| m.contains("duplicate element id")), "duplicate id must fail: {v_id:?}");

        let (_, v_pkg) = duplicate_scan(&[a.clone(), dup_pkg]);
        assert!(v_pkg.iter().any(|m| m.contains("duplicate package name")), "duplicate package must fail: {v_pkg:?}");

        let (_, v_name) = duplicate_scan(&[dup_name]);
        assert!(v_name.iter().any(|m| m.contains("duplicate declared name")), "duplicate name must fail: {v_name:?}");

        // A clean pair must stay silent — no false positives on distinct ids/names/packages.
        let (w, v) = duplicate_scan(&[a, ("e.sysml".to_string(),
            "package S {\n    part other : Need { :>> id = \"55555555-5555-4555-9555-555555555555\"; }\n}".to_string())]);
        assert!(v.is_empty() && w.is_empty(), "distinct identities -> clean: {v:?} {w:?}");
    }

    #[test]
    fn a_duplicate_id_now_fails_with_no_exemption_list() {
        // issue080 RESOLVED. The 18 bootstrap duplicates across 26 records were re-identified by a
        // D0067 migration (control totals reconciled: 7135 records before and after, distinct ids
        // 7109 -> 7135), so the grandfather list is GONE rather than emptied — an empty exemption
        // list is an invitation to refill it. Every duplicate from here is a live corruption.
        let id = "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d"; // one of the 18, now unique in the model
        let files = vec![
            ("a.sysml".to_string(), format!("package P {{
    part x : Need {{ :>> id = \"{id}\"; }}
}}")),
            ("b.sysml".to_string(), format!("package Q {{
    part y : Need {{ :>> id = \"{id}\"; }}
}}")),
        ];
        let (warnings, violations) = duplicate_scan(&files);
        assert_eq!(violations.len(), 1, "a formerly-grandfathered id must now FAIL: {violations:?}");
        assert!(!warnings.iter().any(|m| m.contains("GRANDFATHERED")), "no exemption path remains: {warnings:?}");
    }

    #[test]
    fn declared_name_ignores_edges_and_references() {
        // Edges and successions MENTION names without declaring any; treating them as declarations
        // would make the guard unusable through false positives.
        assert_eq!(declared_name("part foo : Need {"), Some("foo".to_string()));
        assert_eq!(declared_name("action bar;"), Some("bar".to_string()));
        assert_eq!(declared_name("verification gate : Test {"), Some("gate".to_string()));
        assert_eq!(declared_name("#ProspectiveChange part d0129 : Decision {"), Some("d0129".to_string()));
        assert_eq!(declared_name("part def Thing :> Element {"), Some("Thing".to_string()));
        assert_eq!(declared_name("first brief then personas;"), None);
        assert_eq!(declared_name("satisfy nNeed by srReq;"), None);
        assert_eq!(declared_name("flow from a.o to b.i;"), None);
        assert_eq!(declared_name("private import EngineElement::*;"), None);
        assert_eq!(declared_name(":>> id = \"x\";"), None);
    }

    #[test]
    fn inline_attr_reads_attributes_written_mid_line() {
        // Records are routinely written as one-liners, so anchoring at line start would miss them.
        let line = "part r : TestResult { :>> id = \"abc-123\"; :>> outcome = VerdictKind::pass; }";
        assert_eq!(inline_attr(line, "id"), Some("abc-123".to_string()));
        assert_eq!(inline_attr(line, "title"), None);
        assert_eq!(inline_attr("    :>> id  =  \"spaced\";", "id"), Some("spaced".to_string()));
    }
}

// ── attribute-vocabulary guard (issue118) ────────────────────────────────────

/// Nearest declared attribute to `name`, for the "did you mean" hint.
///
/// Edit distance capped at 2 relative to length: a typo is a slip of a character or two, and
/// suggesting a name three edits away is noise that teaches the reader to skim the message.
fn nearest_attr<'a>(name: &str, declared: &'a HashSet<String>) -> Option<&'a str> {
    let budget = if name.len() <= 4 { 1 } else { 2 };
    declared
        .iter()
        .map(|d| (edit_distance(name, d), d.as_str()))
        .filter(|(d, _)| *d <= budget)
        .min_by_key(|(d, s)| (*d, s.len()))
        .map(|(_, s)| s)
}

/// Levenshtein distance, two rows. Index-free so a panic is not reachable by construction — this
/// runs over authored text, where an unexpected shape must never abort the gate.
fn edit_distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        if let Some(first) = cur.first_mut() {
            *first = i + 1;
        }
        for (j, cb) in b.iter().enumerate() {
            let sub = prev.get(j).copied().unwrap_or(0) + usize::from(ca != cb);
            let del = prev.get(j + 1).copied().unwrap_or(0) + 1;
            let ins = cur.get(j).copied().unwrap_or(0) + 1;
            if let Some(slot) = cur.get_mut(j + 1) {
                *slot = sub.min(del).min(ins);
            }
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev.last().copied().unwrap_or(0)
}


#[cfg(test)]
mod family_union_tests {
    use super::{FAMILIES, GUARD_NAMES, RUNNABLE_ONLY};

    /// D0503 / sprint 733: a guard in one home is in all. The union of the family tables, ordered by
    /// `GUARD_NAMES`, IS `GUARD_NAMES`; every other arm is a declared runnable-only guard; no name sits
    /// in two families; and each family lists its guards in `GUARD_NAMES` order.
    #[test]
    fn the_family_tables_union_to_guard_names_in_order() {
        let mut seen: Vec<&str> = Vec::new();
        for fam in FAMILIES {
            let mut last: Option<usize> = None;
            for (name, _) in fam.arms {
                assert!(!seen.contains(name), "{name} is dispatched by two families");
                seen.push(name);
                if let Some(pos) = GUARD_NAMES.iter().position(|g| g == name) {
                    assert!(last.is_none_or(|l| l < pos), "family {} lists {name} out of GUARD_NAMES order", fam.name);
                    last = Some(pos);
                } else {
                    assert!(RUNNABLE_ONLY.contains(name), "{name} is in a family table but neither enforced nor declared runnable-only");
                }
            }
        }
        let mut enforced: Vec<&str> = seen.iter().copied().filter(|n| GUARD_NAMES.contains(n)).collect();
        enforced.sort_by_key(|n| GUARD_NAMES.iter().position(|g| g == n));
        assert_eq!(enforced, GUARD_NAMES.to_vec(), "the union of the family tables is not GUARD_NAMES");
        for n in RUNNABLE_ONLY {
            assert!(seen.contains(&n), "runnable-only {n} is in no family table");
        }
    }
}
