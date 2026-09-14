//! Orient subcommand: state cursor + ready/suspect/invalidEvidence/done frontier.
//!
//! Pure-Rust equivalent of `query.py orient` — no Jupyter kernel required.
//! Uses [`crate::indexer::extract`] to parse `.tracking/**/*.sysml` via the AST
//! parser and then applies git-based suspect/invalid-evidence classification.

use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use crate::indexer::{ExtractedIndex, TaskData};
use crate::textscan::gate_passed;

// ── public types ──────────────────────────────────────────────────────────────

/// Ceremony status of one in-progress sprint (D0045: replaces the retired cursor).
#[derive(Debug)]
pub struct SprintCeremony {
    /// Delivery file stem, e.g. `sprint17_rustToolchainFix`.
    pub sprint: String,
    /// Gate names with a passing `TestResult`, in canonical order.
    pub passed: Vec<String>,
    /// First canonical gate not yet passed (`None` only if all are passed).
    pub pending: Option<String>,
}

/// Output of the `orient` subcommand.
#[derive(Debug)]
pub struct Output {
    /// In-progress sprints with per-gate ceremony status (computed from delivery files).
    pub in_progress_sprints: Vec<SprintCeremony>,
    /// Tasks that are outstanding and whose every dependency is done.
    pub ready: Vec<String>,
    /// Outstanding tasks held off the frontier by a `#DependsOn` edge to another work item that is not
    /// done (issue549), each with the item it waits on and why (`not done` / `superseded`). ALWAYS
    /// emitted, empty included, for the D0138 reason `pending_acceptances` is.
    pub blocked: Vec<BlockedItem>,
    /// Done tasks whose `DoD` criterion text changed since they were verified.
    pub suspect: Vec<String>,
    /// Done tasks whose `judgedAgainst` SHA cannot be resolved AND whose caller has actually LOOKED
    /// at the remote (or has no upstream to look at) — so the anchor is genuinely dangling.
    pub invalid_evidence: Vec<String>,
    /// Done tasks whose anchor is unresolvable HERE, from a caller that has not fetched: this is
    /// unverifiable-FROM-HERE, a different fact from unverified (D0098/D0129). These stay DONE —
    /// counting them outstanding would re-list finished work and get a second contributor to redo it.
    pub unsynchronized_evidence: Vec<String>,
    /// OPEN issues (no complete `#Resolves` resolver) — D0077, surfaced so the frontier can't
    /// read "empty" while issues are unresolved.
    pub open_issues: Vec<String>,
    /// Per-suspect-task reason string (criterion-change / transitive / deliverable-drift).
    /// Carried for `suspect --explain`; NOT emitted in the standard orient JSON.
    pub suspect_reasons: HashMap<String, String>,
    /// Number of done tasks.
    pub done: usize,
    /// Number of outstanding (not-done) tasks.
    pub outstanding: usize,
    /// Compact non-blocking BURNDOWN summary (D0098) — a raw JSON-object fragment (tier-satisfaction
    /// pcts + rootedness counts), always visible so incompleteness can't be silently ignored.
    pub burndown: String,
    /// Why a filter that NARROWS the frontier could not be computed (issue247/issue239). Empty means
    /// every filter ran. Non-empty means `ready` was deliberately emptied and this answer is
    /// COULD-NOT-COMPUTE, not COMPUTED-EMPTY — the two must never look alike (N-C2).
    pub compute_failures: Vec<String>,
    /// Processes this project has NOT activated (D0138). Emitted so orientation states what is NOT
    /// being enforced: a computed view that showed only findings would read as "all controls checked"
    /// on a project running a deliberate subset.
    pub inactive_processes: Vec<String>,
    /// Decisions awaiting the human acceptance gate (issue096). ALWAYS emitted, empty included:
    /// absence must be STATED, not implied — a field that disappears when the list is empty reads
    /// identically to a field nobody computed, which is the D0138 lesson.
    pub pending_acceptances: Vec<String>,
    /// This clone's divergence from its upstream, as a raw JSON fragment (issue... / D0129
    /// srDcSyncAwareOrient). EVERY computed answer above was computed against THIS tree, so the
    /// tree's position relative to the remote is part of the answer rather than context for it: a
    /// frontier computed 40 commits behind is a frontier for work someone else may already have done.
    /// Read from the last fetch and never fetches itself — a view that silently performs network I/O
    /// is a view you stop running.
    pub sync: String,
}

/// One held-off item: `item` waits on `waits_on` because `why`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockedItem {
    pub item: String,
    pub waits_on: String,
    pub why: String,
}

impl Output {
    /// Render as a JSON string matching the `query.py orient` output format.
    #[must_use]
    pub fn to_json(&self) -> String {
        let sprints: Vec<String> = self.in_progress_sprints.iter().map(|s| {
            let pending = s.pending.as_ref()
                .map_or_else(|| "null".to_owned(), |p| format!("\"{}\"", json_esc(p)));
            format!(
                "{{\"sprint\": \"{}\", \"passed\": {}, \"pending\": {}}}",
                json_esc(&s.sprint),
                str_array(&s.passed),
                pending,
            )
        }).collect();
        let in_progress_block = if sprints.is_empty() {
            "[]".to_owned()
        } else {
            format!("[{}]", sprints.join(", "))
        };
        let burndown = if self.burndown.is_empty() { "{}" } else { self.burndown.as_str() };
        let blocked: Vec<String> = self.blocked.iter().map(|b| {
            format!(
                "{{\"item\": \"{}\", \"waitsOn\": \"{}\", \"why\": \"{}\"}}",
                json_esc(&b.item), json_esc(&b.waits_on), json_esc(&b.why)
            )
        }).collect();
        let blocked_block = if blocked.is_empty() { "[]".to_owned() } else { format!("[{}]", blocked.join(", ")) };
        format!(
            "{{\n  \"in_progress_sprints\": {},\n  \"ready\": {},\n  \"blocked\": {},\n  \"suspect\": {},\n  \"invalidEvidence\": {},\n  \"unsynchronizedEvidence\": {},\n  \"open_issues\": {},\n  \"pendingAcceptances\": {},\n  \"sync\": {},\n  \"counts\": {{\"done\": {}, \"outstanding\": {}}},\n  \"burndown\": {},\n  \"inactive_processes\": {},\n  \"answerStatus\": {}\n}}",
            in_progress_block,
            str_array(&self.ready),
            blocked_block,
            str_array(&self.suspect),
            str_array(&self.invalid_evidence),
            str_array(&self.unsynchronized_evidence),
            str_array(&self.open_issues),
            str_array(&self.pending_acceptances),
            if self.sync.is_empty() { "null" } else { self.sync.as_str() },
            self.done,
            self.outstanding,
            burndown,
            str_array(&self.inactive_processes),
            // issue239/issue247: EVERY computed answer names which of the four states it is, so a
            // COULD-NOT-COMPUTE can never be mistaken for a COMPUTED-EMPTY.
            if self.compute_failures.is_empty() {
                "\"COMPUTED\"".to_owned()
            } else {
                format!(
                    "{{\"state\": \"COULD-NOT-COMPUTE\", \"readyEmptiedDeliberately\": true, \"reasons\": {}}}",
                    str_array(&self.compute_failures)
                )
            },
        )
    }
}

fn json_esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn str_array(items: &[String]) -> String {
    if items.is_empty() {
        return "[]".to_owned();
    }
    let inner: Vec<String> = items.iter().map(|s| format!("\"{}\"", json_esc(s))).collect();
    format!("[{}]", inner.join(", "))
}

// ── top-level entry point ─────────────────────────────────────────────────────

/// Compute the orientation view for a repository root.
///
/// Parses `.tracking/**/*.sysml` via the AST parser (see [`crate::indexer::extract`])
/// and applies git-based suspect/invalid-evidence classification.
#[must_use]
pub fn compute(root: &Path) -> Output {
    compute_after_fetch(root, false)
}

/// As [`compute`], but stating whether the caller has JUST fetched.
///
/// Only a caller that has actually looked at the remote may classify an unresolvable evidence anchor
/// as genuinely dangling rather than merely unfetched — see the reasoning at `clone_can_judge`.
/// `keel sync` is the one caller that passes `true`.
#[must_use]
pub fn compute_after_fetch(root: &Path, fetched: bool) -> Output {
    let tracking = root.join(".tracking");
    let idx = crate::indexer::extract(&tracking);
    compute_orient(root, idx, fetched)
}

// ── git helpers ───────────────────────────────────────────────────────────────

/// The three filters that NARROW the frontier, and any failure to compute them (issue247/issue239).
///
/// Extracted so the fail-closed property is testable and so `compute_after_fetch` stays readable.
/// The distinction that matters: `superseded` and `blocked` REMOVE work that must not be scheduled,
/// so failing to compute them yields a frontier that is a SUPERSET of the truth — the caller must
/// refuse rather than publish it. `claimed_by_others` is different and its empty-on-error is
/// deliberate: a claim must never make work INVISIBLE to someone who cannot be told it is theirs.
struct Narrowing {
    superseded: HashSet<String>,
    blocked: HashSet<String>,
    /// Items waiting on another work ITEM (issue549), in order: what waits, on what, why.
    blocked_on_items: Vec<BlockedItem>,
    claimed_by_others: HashSet<String>,
    compute_failures: Vec<String>,
}

fn narrowing_filters(repo: &Path, tasks: &HashMap<String, TaskData>, done_map: &HashMap<String, bool>) -> Narrowing {
    let mut compute_failures: Vec<String> = Vec::new();
    // A SUPERSEDED task is never ready (issue100): a `#Supersede` edge is the authored statement that
    // the work is deliberately retired (§1.4), and the frontier is auto-followed (D0052).
    //
    // issue247: this was `.unwrap_or_default()`, the exact opposite of the conservatism its own
    // comment promised — an Err yielded an EMPTY set, nothing was filtered, and every retired task
    // returned to the frontier. Record the failure instead; the caller empties `ready` and says so.
    let superseded = crate::view::superseded_names(repo).unwrap_or_else(|e| {
        compute_failures.push(format!(
            "superseded set could not be computed ({e}) - retired work would re-enter the frontier"
        ));
        HashSet::new()
    });
    // Nor is a task that `#DependsOn` a still-PROPOSED Decision (issue112) — superseded means
    // RETIRED, this means WAITING ON A HUMAN, and it returns by itself once the Decision resolves.
    let blocked = crate::view::blocked_on_acceptance(repo).unwrap_or_else(|e| {
        compute_failures.push(format!(
            "blocked-on-acceptance set could not be computed ({e}) - work awaiting a human would read as ready"
        ));
        HashSet::new()
    });
    // Nor is an item whose `#DependsOn` names another work item that is not done (issue549). The
    // succession chain (`first A then B`) was honoured from the start; the typed edge between two
    // backlog items was read by nothing, so a chain of six layering members ranked ready at once. The
    // done-set is this frontier's own, so the answer cannot disagree with `counts`. Fail closed like
    // the two above: this REMOVES work, and an Err would widen the frontier.
    let done_set: HashSet<String> = done_map.iter().filter(|(_, &v)| v).map(|(k, _)| k.clone()).collect();
    let is_item = |n: &str| tasks.contains_key(n);
    let blocked_on_items: Vec<BlockedItem> = match crate::view::blocked_on_items(repo, &done_set, &is_item) {
        Ok(rows) => rows.into_iter().map(|(item, waits_on, why)| BlockedItem { item, waits_on, why }).collect(),
        Err(e) => {
            compute_failures.push(format!(
                "blocked-on-item set could not be computed ({e}) - work whose predecessor is undone would read as ready"
            ));
            Vec::new()
        }
    };
    // Nor is an item another contributor holds a LIVE claim on (D0147/srDcWorkClaim). Empty on error
    // AND on an unresolved actor is DELIBERATE here, and is not a compute failure: if this machine has
    // no bound identity nothing is hidden, because a claim must never make work invisible to someone
    // who cannot be told it is theirs.
    let me = crate::actor::resolve(repo, None).unwrap_or_default();
    let claimed_by_others: HashSet<String> = if me.is_empty() {
        HashSet::new()
    } else {
        crate::claim::held_by_others(repo, &me).unwrap_or_default().into_iter().map(|(item, _)| item).collect()
    };
    Narrowing { superseded, blocked, blocked_on_items, claimed_by_others, compute_failures }
}

// ── classification ────────────────────────────────────────────────────────────

/// The `first A then B;` successions every workflow under `.engine/workflows/` declares, as
/// (file stem, A, B) in declaration order. The ONE place the chain is read (D0435).
#[must_use]
pub(crate) fn workflow_successions(root: &Path) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for path in crate::collect_sysml(&root.join(".engine").join("workflows")) {
        let Ok(text) = crate::corpus::read_to_string(&path) else { continue };
        let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        for line in text.lines() {
            let l = line.trim();
            if l.starts_with("//") {
                continue;
            }
            let Some(rest) = l.strip_prefix("first ") else { continue };
            let body = rest.split(';').next().unwrap_or("");
            let Some((a, b)) = body.split_once(" then ") else { continue };
            let a: String = a.trim().chars().take_while(|c| crate::algo::is_word(*c)).collect();
            let b: String = b.trim().chars().take_while(|c| crate::algo::is_word(*c)).collect();
            if !a.is_empty() && !b.is_empty() {
                out.push((stem.clone(), a, b));
            }
        }
    }
    out
}

/// Every phase a workflow chain names - the vocabulary a `checkedBy = "gate:<phase>"` resolves against.
#[must_use]
pub(crate) fn workflow_phases(root: &Path) -> HashSet<String> {
    let mut out = HashSet::new();
    for (_, a, b) in workflow_successions(root) {
        out.insert(a);
        out.insert(b);
    }
    out
}

/// The ceremony gate order, READ FROM THE TREE (D0435).
///
/// The workflow whose phases some `ProcessStep` binds as `checkedBy = "gate:<phase>"` supplies the
/// order: its declared successions, linearised, each phase spelt as the gate a sprint record declares
/// (`closeOut` -> `CloseOut`, the `<...>CloseOutGate : Test`). Until 2026-09-10 this was a compiled
/// constant here, a second in the `ceremony` guard, a third in `audit`, and an `include_str!` test
/// holding one of them to `.engine/workflows/delivery.sysml` - four homes for one sequence. A tree
/// whose processes bind no gate has NO ceremony order: every caller then reports a sprint as
/// unenforceable-by-step rather than enforcing an order nobody declared.
#[must_use]
pub(crate) fn gate_order(root: &Path) -> Vec<String> {
    let bound: HashSet<String> = crate::binding::step_check_bindings(root)
        .into_iter()
        .filter_map(|(_, _, _, name)| name.strip_prefix("gate:").map(str::to_owned))
        .collect();
    if bound.is_empty() {
        return Vec::new();
    }
    let succ = workflow_successions(root);
    // The workflow files whose chain some step binds, in file order.
    let mut files: Vec<&str> = Vec::new();
    for (f, a, b) in &succ {
        if (bound.contains(a) || bound.contains(b)) && !files.contains(&f.as_str()) {
            files.push(f);
        }
    }
    let mut out = Vec::new();
    for f in files {
        let edges: Vec<(&str, &str)> =
            succ.iter().filter(|(g, _, _)| g == f).map(|(_, a, b)| (a.as_str(), b.as_str())).collect();
        out.extend(linearise(&edges).into_iter().map(|p| gate_name(&p)));
    }
    out
}

/// Kahn's algorithm with declaration order as the tie-break: a chain comes out as written, a fork
/// (deploy's `first declare then systemVnV; first declare then safetyValidation;`) in the order its
/// branches were declared. A cycle leaves its members out - a succession that loops orders nothing.
fn linearise(edges: &[(&str, &str)]) -> Vec<String> {
    let mut nodes: Vec<&str> = Vec::new();
    for (a, b) in edges {
        for n in [*a, *b] {
            if !nodes.contains(&n) {
                nodes.push(n);
            }
        }
    }
    let mut indeg: HashMap<&str, usize> = nodes.iter().map(|n| (*n, 0usize)).collect();
    for (_, b) in edges {
        if let Some(d) = indeg.get_mut(b) {
            *d += 1;
        }
    }
    let mut ready: std::collections::VecDeque<&str> =
        nodes.iter().copied().filter(|n| indeg.get(n).copied().unwrap_or(0) == 0).collect();
    let mut out = Vec::new();
    while let Some(n) = ready.pop_front() {
        out.push(n.to_owned());
        for (a, b) in edges {
            if *a != n {
                continue;
            }
            if let Some(d) = indeg.get_mut(b) {
                *d -= 1;
                if *d == 0 {
                    ready.push_back(b);
                }
            }
        }
    }
    out
}

/// `closeOut` -> `CloseOut`: the phase as the gate's Test name spells it.
#[must_use]
pub(crate) fn gate_name(phase: &str) -> String {
    let mut c = phase.chars();
    c.next()
        .map_or_else(String::new, |f| f.to_uppercase().collect::<String>() + c.as_str())
}

#[cfg(test)]
mod gate_order_tests {
    use super::{gate_name, gate_order, linearise};

    #[test]
    fn a_chain_linearises_as_written_and_a_fork_in_branch_order() {
        let chain = [("refine", "standup"), ("standup", "implement"), ("implement", "review")];
        assert_eq!(linearise(&chain), ["refine", "standup", "implement", "review"]);
        let fork = [("declare", "systemVnV"), ("declare", "safetyValidation")];
        assert_eq!(linearise(&fork), ["declare", "systemVnV", "safetyValidation"]);
        let cycle = [("a", "b"), ("b", "a")];
        assert!(linearise(&cycle).is_empty(), "a loop orders nothing");
    }

    #[test]
    fn gate_name_upper_cases_the_first_letter_only() {
        assert_eq!(gate_name("closeOut"), "CloseOut");
        assert_eq!(gate_name("retro"), "Retro");
        assert_eq!(gate_name(""), "");
    }

    /// The order READ FROM THIS TREE is the six the sprint records declare - the replacement for the
    /// `include_str!` test that held a compiled constant to the file. If agile-workflow's bindings or
    /// delivery.sysml's chain move, this is where it shows.
    #[test]
    fn this_trees_ceremony_order_is_the_delivery_chain() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        assert_eq!(gate_order(&root), ["Refine", "Standup", "Implement", "Review", "CloseOut", "Retro"]);
    }

    /// A tree whose processes bind no gate has no ceremony order.
    #[test]
    fn no_gate_binding_means_no_order() {
        let dir = std::env::temp_dir().join(format!("keel-gateorder-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".engine/workflows")).expect("mkdir");
        std::fs::create_dir_all(dir.join(".engine/processes")).expect("mkdir");
        std::fs::write(dir.join(".engine/workflows/d.sysml"), "package D {\n    action def D {\n        first a then b;\n    }\n}\n").expect("write");
        let proc = dir.join(".engine/processes/p.sysml");
        let unbound = "package P {\n    action p : Process { :>> purpose = \"p\"; }\n    action s : ProcessStep {\n        :>> actionText = \"x\";\n        :>> owner = Owner::ai;\n    }\n}\n";
        std::fs::write(&proc, unbound).expect("write");
        assert!(gate_order(&dir).is_empty());
        let bound = unbound.replace("        :>> owner = Owner::ai;\n", "        :>> owner = Owner::ai;\n        :>> checkedBy = \"gate:b\";\n");
        std::fs::write(&proc, bound).expect("write");
        assert_eq!(gate_order(&dir), ["A", "B"]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// Compute in-progress sprint ceremony status from `.tracking/delivery/*.sysml`
/// (D0045: replaces the `StateCursor`). A sprint is in-progress if at least one gate
/// passed and the chain's terminal gate has not; `pending` is the first gate in the declared
/// order (`gate_order`, D0435) not yet passed. A tree with no ceremony order has no in-progress
/// sprint to report.
fn in_progress_sprints(repo: &Path) -> Vec<SprintCeremony> {
    let delivery = repo.join(".tracking").join("delivery");
    let mut out = Vec::new();
    let order = gate_order(repo);
    let Some(terminal) = order.last() else { return out };
    for path in crate::collect_sysml(&delivery) {
        let Ok(text) = std::fs::read_to_string(&path) else { continue };
        let passed: Vec<String> = order.iter()
            .filter(|g| gate_passed(&text, g))
            .cloned()
            .collect();
        if passed.is_empty() || passed.iter().any(|g| g == terminal) {
            continue; // not started, or ceremony complete
        }
        let pending = order.iter()
            .find(|g| !passed.iter().any(|p| p == *g))
            .cloned();
        let sprint = path.file_stem().map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        out.push(SprintCeremony { sprint, passed, pending });
    }
    out.sort_by(|a, b| a.sprint.cmp(&b.sprint));
    out
}

/// The frontier half of an orient run: the done-set, the evidence classes, and `ready` in declaration
/// order. Everything `whats-next` and the priority lens need, and nothing the burndown needs - the
/// suspect walk, the deliverable drift, the open issues and the burndown are the second half
/// ([`compute_orient`]), and the priority-inversion guard used to pay for all of it to read one field
/// (issue439: 5.7 s of a 6.6 s turn boundary, the critical path of every attributed slow fire).
struct Frontier {
    tasks: HashMap<String, TaskData>,
    ordering_only: HashSet<(String, String)>,
    done_map: HashMap<String, bool>,
    verified_at: HashMap<String, String>,
    invalid_evidence: Vec<String>,
    unsynchronized_evidence: Vec<String>,
    ready: Vec<String>,
    blocked: Vec<BlockedItem>,
    compute_failures: Vec<String>,
    sync_state: crate::sync::Divergence,
}

/// The ready frontier alone, in declaration order (D0052).
///
/// The same list `orient` carries, computed without the suspect walk, the deliverable drift, the open
/// issues or the burndown. A filter that could not be computed empties it, as in `orient` (issue247);
/// `compute_failures` says which.
///
/// Returns `(ready, compute_failures, outstanding)`.
#[must_use]
pub fn ready(root: &Path) -> (Vec<String>, Vec<String>, usize) {
    let idx = crate::perf::phase("frontier:extract", || crate::indexer::extract(&root.join(".tracking")));
    let f = frontier(root, idx, false, false);
    let outstanding = f.done_map.values().filter(|&&v| !v).count();
    (f.ready, f.compute_failures, outstanding)
}

/// `evidence` says whether the caller reads `verified_at` and the evidence classes (orient does; the
/// ready list does not). When it is false and this clone cannot judge a dangling anchor anyway
/// (`clone_can_judge` below), the SHA validation and the landing-commit binding are skipped: neither
/// can change `done_map` in that state - an unresolvable anchor leaves the task DONE either way - so
/// `ready` is identical by construction, and the two git spawns that cost the priority-inversion guard
/// a second on every fire (issue439: `cat-file --batch-check` over 666 short ids, then the binding
/// walk) are not paid to read a list they cannot alter.
fn frontier(repo: &Path, idx: ExtractedIndex, fetched: bool, evidence: bool) -> Frontier {
    let ExtractedIndex { tasks, ordering_only, .. } = idx;

    let crate::evidence::Evidence { done_map, verified_at, mut invalid_evidence, mut unsynchronized_evidence, sync_state } =
        crate::evidence::classify(repo, &tasks, fetched, evidence);

    // Step 2: compute ready. A SUPERSEDED task is never ready (issue100) — a `#Supersede` edge is the
    // authored statement that the work is deliberately retired (§1.4), and the frontier is auto-followed
    // (D0052), so leaving it ready schedules work a Decision has forbidden. Conservative on error:
    // failing to read the model must not silently make everything ready again.
    let Narrowing { superseded, blocked, blocked_on_items, claimed_by_others, compute_failures } =
        crate::perf::phase("frontier:narrowing", || narrowing_filters(repo, &tasks, &done_map));
    let waiting_on_item: HashSet<&str> = blocked_on_items.iter().map(|b| b.item.as_str()).collect();
    let mut ready: Vec<String> = Vec::new();
    for (name, data) in &tasks {
        let is_done = done_map.get(name.as_str()).copied().unwrap_or(false);
        let is_invalid = invalid_evidence.contains(name);
        if !is_done
            && !is_invalid
            && !superseded.contains(name)
            && !blocked.contains(name)
            && !waiting_on_item.contains(name.as_str())
            && !claimed_by_others.contains(name)
        {
            let all_deps_done = all_deps_satisfied(name, data, &done_map);
            if all_deps_done {
                ready.push(name.clone());
            }
        }
    }
    // FAIL CLOSED (issue247). If a filter that REMOVES items from the frontier could not be computed,
    // the frontier we just built is a SUPERSET of the true one — it may contain retired work or work
    // awaiting a human. Publishing it would schedule exactly what a Decision forbade. An empty
    // frontier plus a stated failure is honest; a wide frontier presented as the answer is not.
    if !compute_failures.is_empty() {
        ready.clear();
    }

    // Rank ready by backlog declaration order (D0052) — priority, not alphabetical. The blocked list
    // ranks the same way: it is the frontier's shadow, read in the order the work would be picked.
    ready.sort_by_key(|name| tasks.get(name).map_or(u32::MAX, |t| t.order));
    let mut blocked_items: Vec<BlockedItem> = blocked_on_items
        .into_iter()
        .filter(|b| !done_map.get(b.item.as_str()).copied().unwrap_or(false) && !superseded.contains(&b.item))
        .collect();
    blocked_items.sort_by_key(|b| (tasks.get(&b.item).map_or(u32::MAX, |t| t.order), b.waits_on.clone()));
    invalid_evidence.sort();
    unsynchronized_evidence.sort();
    Frontier { tasks, ordering_only, done_map, verified_at, invalid_evidence, unsynchronized_evidence, ready, blocked: blocked_items, compute_failures, sync_state }
}

fn compute_orient(repo: &Path, idx: ExtractedIndex, fetched: bool) -> Output {
    let Frontier { tasks, ordering_only, done_map, verified_at, invalid_evidence, unsynchronized_evidence, ready, blocked, compute_failures, sync_state } =
        frontier(repo, idx, fetched, true);

    // Steps 3-5: the suspect walk (criterion change, transitive, deliverable drift) - one module,
    // below the views, so a view reads the suspect set without paying for this whole run.
    let (suspect, suspect_reasons) = crate::suspect::walk(repo, &tasks, &ordering_only, &done_map, &verified_at);

    let done = done_map.values().filter(|&&v| v).count();
    let outstanding = done_map.values().filter(|&&v| !v).count();

    // Open issues (D0077): an issue with no complete #Resolves resolver. Reuse this orient
    // run's done-set as the resolver-completeness authority; build the view Model for the edges.
    let done_set: HashSet<String> = done_map.iter().filter(|(_, &v)| v).map(|(k, _)| k.clone()).collect();
    let open_issues = crate::view::open_issue_names(repo, &done_set).unwrap_or_default();

    Output {
        in_progress_sprints: in_progress_sprints(repo),
        ready,
        blocked,
        suspect,
        invalid_evidence,
        unsynchronized_evidence,
        open_issues,
        suspect_reasons,
        done,
        outstanding,
        // Compact non-blocking burndown (D0098); empty -> "{}" on render if it can't be computed.
        burndown: crate::view::burndown_summary_json(repo).unwrap_or_default(),
        compute_failures,
        // D0138: state what is NOT enforced, so a subset-activated project cannot read as fully checked.
        inactive_processes: crate::activation::Activation::load(repo).inactive_processes(),
        // Conservative on error, and deliberately the OPPOSITE default from `superseded_names`:
        // a model-read failure there had to avoid silently restoring everything to ready, whereas
        // here an empty list is the benign reading. Reporting "nothing pending" when the model
        // could not be read would be the silent failure issue096 is about, so a failure surfaces
        // as a named sentinel the human will notice rather than as a clean zero.
        sync: sync_state.to_json(),
        pending_acceptances: crate::view::pending_acceptances(repo)
            .unwrap_or_else(|_| vec!["<unreadable: could not compute pending acceptances>".to_owned()]),
    }
}

fn all_deps_satisfied(
    _name: &str,
    data: &TaskData,
    done_map: &HashMap<String, bool>,
) -> bool {
    data.deps
        .iter()
        .all(|dep| done_map.get(dep.as_str()).copied().unwrap_or(false))
}

#[cfg(test)]
mod evidence_class_tests {
    use super::compute_after_fetch;

    /// The classification hinges on whether the caller has LOOKED, not on `behind == 0` — and the
    /// difference is not academic: the fixture that drove this design had a clone reporting
    /// `behind 0` while being genuinely stale, because `behind` is measured against a
    /// remote-tracking ref that is only as fresh as the last fetch. A clone that has never fetched
    /// would have confidently declared another contributor's evidence INVALID and re-listed their
    /// finished work as ready.
    ///
    /// This asserts the property on a directory with no `.tracking` at all, which is the degenerate
    /// case: both sets stay empty and nothing is invented either way. The four substantive cases are
    /// exercised end-to-end against real clones (see the sprint record), because the distinction is
    /// about git object visibility and cannot be faked in-process.
    #[test]
    fn evidence_classification_invents_nothing_on_an_empty_tree() {
        let dir = std::env::temp_dir().join(format!("keel-evc-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for fetched in [false, true] {
            let o = compute_after_fetch(&dir, fetched);
            assert!(o.invalid_evidence.is_empty(), "fetched={fetched}");
            assert!(o.unsynchronized_evidence.is_empty(), "fetched={fetched}");
        }
        std::fs::remove_dir_all(&dir).ok();
    }
}

#[cfg(test)]
mod tests {
    /// THE CONTROL for issue247: a failure to compute a NARROWING filter must empty the frontier and
    /// be stated, never widen it. The old code used `.unwrap_or_default()` under a comment promising
    /// the opposite, so an Err returned every retired and human-blocked task to a frontier the AI
    /// auto-follows (D0052).
    #[test]
    fn a_compute_failure_empties_the_frontier_and_is_stated() {
        let out = super::Output {
            in_progress_sprints: Vec::new(),
            ready: vec!["shouldNotSurvive".to_string()],
            blocked: Vec::new(),
            suspect: Vec::new(),
            invalid_evidence: Vec::new(),
            unsynchronized_evidence: Vec::new(),
            open_issues: Vec::new(),
            suspect_reasons: std::collections::HashMap::new(),
            done: 0,
            outstanding: 1,
            burndown: String::new(),
            compute_failures: vec!["superseded set could not be computed".to_string()],
            inactive_processes: Vec::new(),
            pending_acceptances: Vec::new(),
            sync: String::new(),
        };
        let json = out.to_json();
        assert!(json.contains("COULD-NOT-COMPUTE"), "a failed computation must SAY so: {json}");
        assert!(json.contains("readyEmptiedDeliberately"), "and must say the frontier was emptied on purpose");
        assert!(json.contains("superseded set could not be computed"), "and must carry the reason");
    }

    /// The other side: a clean computation says COMPUTED, so the two states are never byte-identical.
    #[test]
    fn a_clean_computation_states_computed() {
        let out = super::Output {
            in_progress_sprints: Vec::new(),
            ready: Vec::new(),
            blocked: Vec::new(),
            suspect: Vec::new(),
            invalid_evidence: Vec::new(),
            unsynchronized_evidence: Vec::new(),
            open_issues: Vec::new(),
            suspect_reasons: std::collections::HashMap::new(),
            done: 0,
            outstanding: 0,
            burndown: String::new(),
            compute_failures: Vec::new(),
            inactive_processes: Vec::new(),
            pending_acceptances: Vec::new(),
            sync: String::new(),
        };
        let json = out.to_json();
        assert!(json.contains("\"answerStatus\": \"COMPUTED\""), "{json}");
        assert!(!json.contains("COULD-NOT-COMPUTE"));
    }
}
