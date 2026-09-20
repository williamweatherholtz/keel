//! Guard family `enforcement` - split from `keel-cli/src/guards.rs` by `scripts/split_guards.py` (sprint 733).
//!
//! Each guard's dispatch arm, code and tests sit together; the shared scanners, the runner and the
//! lock predicates are the crate root's (`super`). Nothing here was retyped: the text is guards.rs's,
//! with `crate::` paths pointing at the members and private items opened to the crate.

use super::*;

/// The `enforcement` family: every guard it dispatches, in `GUARD_NAMES` order, with the tier note each
/// arm carried in `run_one` (sprint 733). The root's union test holds these tables equal to `GUARD_NAMES`.
pub(crate) const FAMILY: Family = Family {
    name: "enforcement",
    arms: &[
        ("process-applicability", process_applicability),
        ("doc-guard-count", doc_guard_count),
        ("process-change", process_change),
        ("manifest-coverage", manifest_coverage),
        ("process-skill", process_skill),
        ("doc-sync", doc_sync), // WARNING-level member of GUARD_NAMES (D0113) — definitional change w/o doc update
        ("hook-config-integrity", hook_config_integrity), // warning-only (D0047/issue093) — a hook pointing at a deleted script
        ("activation-manifest", activation_manifest), // hard (D0138) — a typo silently disables a control
        ("claude-surface-drift", claude_surface_drift), // hard (D0174/K7) — a mutated hook command is a silently weakened control
        ("control-event-coverage", control_event_coverage), // WARNING-tier (D0193) — a control-relevant event with no counted record
        ("manifest-key-portability", manifest_key_portability), // issue301/D0250 — a unit manifest key naming one machine
        ("control-map-reconciled", control_map_reconciled), // issue304/D0255 — a firing control absent from the map
        ("unit-extras-present", unit_extras_present), // hard (issue290/D0300) - a unit's declared mechanism is in the tree
        ("stpa-currency", stpa_currency), // WARNING-tier (D0313) - the computed control structure says when STPA must run again
        ("instruments-declared", instruments_declared), // hard (D0361) - an undeclared instrument is outside every analysis // hard (issue385) - a gating workflow that supplies less than its twin gates on a different environment
        ("step-check-resolves", step_check_resolves), // hard (D0434) - a step naming a check nothing runs is EHZ5
        ("defect-guard-coverage", defect_guard_coverage), // runnable-only (D0047/issue039)
    ],
};

pub(crate) fn is_process_def(p: &str) -> bool {
    // D0184/p3aKeystoneExtension: a skill body or an activation/attestation policy IS process
    // definition — the keystone lock covers them exactly as it covers processes and workflows.
    // Contracts that are MACHINE STATE registries (unit-ids, installed-units) are import/export
    // bookkeeping, not process definition, and stay outside the lock.
    if p.starts_with(".engine/skills/")
        && (is_sysml(p) || std::path::Path::new(p).extension().is_some_and(|e| e.eq_ignore_ascii_case("md")))
    {
        return true;
    }
    if p.starts_with(".engine/contracts/")
        && !p.ends_with("unit-ids.toml")
        && !p.ends_with("installed-units.toml")
        && !p.ends_with("deck-inbox.toml")
        && !p.ends_with("engine-version.toml") // D0190: machine-stamped by init/migrate, instance data
        && !p.ends_with("adoption-profile.toml")
    {
        return true;
    }
    // issue236 (D0208 control evaluation, HIGH): `.engine/rules/` holds the DECLARED ElementRule/
    // EdgeRule instances the guards enforce — downgrading a rule's severity from blocking to warning
    // silently disarms a control, and the evaluation proved that edit passed with no guard firing
    // (the monitor was modifiable by the monitored). Rules are enforcement DEFINITION, so the
    // keystone lock covers them: any `.engine/rules/*.sysml` edit needs a co-committed marked
    // Decision, i.e. a human-signed act — the panel's self-modification gap, closed.
    is_sysml(p)
        && (p.starts_with(".engine/processes/")
            || p.starts_with(".engine/workflows/")
            || p.starts_with(".engine/rules/"))
}

/// The files that DEFINE the enforcement LOGIC (every `-> GuardReport` guard plus the audit-adherence
/// gate). Kept identical to the real set by `enforcement_surface_covers_every_guard_source` — that
/// test fails CI if a new guard-defining file appears outside this list, which is the D0209-clause-2
/// "diff `is_process_def` against the actual guard-definition paths" audit made executable.
// guard_names.rs holds GUARD_NAMES (sprint 732): removing a name there disarms a guard as surely as
// deleting its arm here would, so the list is locked with the source that dispatches it. The
// audit-adherence gate (adherence.rs) and the audit-history re-derivation (history.rs) are this member's
// since sprint 748 (D0513), locked by GUARD_SOURCE_DIRS below; the file entry that named the audit in
// keel-cli/src is retired here because that path no longer exists.
pub(crate) const GUARD_SOURCE_FILES: &[&str] = &["members/keel-schema/src/guard_names.rs"];

/// The directories whose EVERY file is enforcement logic: member keel-guards (sprint 733) - the family
/// modules, the runner, the receipt that lets a green guard be skipped (D0371), the content key it is
/// keyed on. Locked by prefix so a new family file is inside the lock by construction; the coverage
/// test still scans every member for a `-> GuardReport` outside it.
pub(crate) const GUARD_SOURCE_DIRS: &[&str] = &["members/keel-guards/src/"];

/// The ENFORCEMENT SURFACE (D0209 clause 2, dcFreezeEnforcementSurface): the paths a guard reads its
/// own DEFINITION or CONFIG from. issue236 proved a control could be silently disarmed by editing its
/// definition; `.engine/rules/` (in `is_process_def`) closed the DECLARED-rule leg, and this closes
/// the rest — the guard SOURCE, the local hook CONFIG, and the CI WORKFLOW files that run the gates on
/// infra the agent cannot touch. A change to any of them needs a co-committed human-signed marked
/// Decision, exactly like a process definition. Kept SEPARATE from `is_process_def` so the two intents
/// stay legible and the coverage audit can diff this set against the real guard sources.
pub(crate) fn is_enforcement_surface(p: &str) -> bool {
    // CI workflow files — where audit-adherence / audit-history / the keel gates actually run.
    if p.starts_with(".github/workflows/")
        && std::path::Path::new(p)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("yml") || e.eq_ignore_ascii_case("yaml"))
    {
        return true;
    }
    // Local git hooks — the per-commit / per-merge / per-push gate wiring.
    if p.starts_with(".githooks/") {
        return true;
    }
    // Guard SOURCE — the enforcement logic itself.
    GUARD_SOURCE_FILES.contains(&p) || GUARD_SOURCE_DIRS.iter().any(|d| p.starts_with(d))
}

/// Is this repo-relative path under the KEYSTONE LOCK — process definition or enforcement surface?
///
/// Public because the workspace gate has to ask the same question about paths that belong to no
/// project (issue276). One predicate, one answer: a second copy in `workspace.rs` would be a second
/// place for the locked set to be true, and it would drift the first time a path class is added here.
#[must_use]
pub fn is_locked_path(p: &str) -> bool {
    is_process_def(p) || is_enforcement_surface(p)
}

/// Is this locked path's text under `read` exactly what the engine embedded in this binary WRITES -
/// an engine RESYNC (D0441 / issue475) or its verb RESPELL (D0523 / issue622) - not a
/// self-modification?
///
/// `keel migrate` writes `.engine/` from the engine embedded in the binary (D0275), and skills,
/// contracts and rules under it are locked paths - so every resync wrote locked files no Decision
/// authored. Under the staged read that refusal fell at the downstream project's post-migrate
/// commit; under D0440's working-tree read it fell inside migrate's own D0336 gate and reverted the
/// update (six fixture tests, 2026-09-10). Decided by CONTENT, never by the pin: a locked file whose
/// text equals the embedded file at the same relative path (line endings normalised) is the engine
/// arriving; one edited byte puts it back under the lock. The respell is the same shape one step on:
/// migrate's `verb-respell` rewrites the retired `keel <verb>` references in a locked file the resync
/// does not write - a project-owned contract's comments, a project-added process - and the first
/// adopter run of it went red HERE, on the two contracts it had just made true (D0523). A file whose
/// text equals the respell of its own HEAD text carries nothing but the engine's fold; one edited
/// byte beyond that puts it back under the lock. Two exclusions: the self-build, where the embedded
/// engine IS the tree and a rebuild after the edit would launder any change (`keel migrate` refuses
/// the self-build for the same reason); and a deleted file, which has no text to compare.
pub(crate) fn is_engine_written(root: &Path, path: &str, read: ChangeRead) -> bool {
    if keel_model::corpus::is_self_build(root) {
        return false;
    }
    let Some(rel) = path.replace('\\', "/").strip_prefix(".engine/").map(str::to_owned) else { return false };
    let ours = changed_text(root, path, read);
    if ours.is_empty() {
        return false;
    }
    let same = |expected: &str| expected.replace("\r\n", "\n") == ours.replace("\r\n", "\n");
    let before = git_stdout(root, &["show", &format!("HEAD:{path}")]);
    // What the resync would write onto THIS project: a sectioned contract keeps the project's own
    // sections (issue349), so the expected text is the merge over the file as it was at HEAD.
    if let Some(shipped) = keel_schema::embedded::engine_text(&rel) {
        if let Some(expected) = keel_schema::embedded::resync_text(Path::new(&rel), shipped, (!before.is_empty()).then_some(before.as_str())) {
            if same(&expected) {
                return true;
            }
        }
    }
    // What the respell would write: HEAD's text with each retired reference at today's spelling.
    if !before.is_empty() {
        let markdown = rel.to_ascii_lowercase().ends_with(".md");
        if let Some((expected, _)) = crate::surface::respell_cli_references(&before, markdown) {
            if same(&expected) {
                return true;
            }
        }
    }
    false
}

pub(crate) fn keystone_violations(changed: &[String], decision_texts: &[(String, String)], charters: &[(String, String)]) -> Vec<String> {
    // The keystone covers process DEFINITION (D0070) AND the ENFORCEMENT SURFACE (D0209 clause 2):
    // guard source, hook config, CI workflows. A change to either needs a co-committed marked Decision
    // - or (D0465) a co-committed sprint record chartered by a marked Decision a HUMAN has accepted,
    // which is the path the held-then-accepted process change of D0337 takes.
    let mut locked: Vec<&str> = changed
        .iter()
        .map(String::as_str)
        .filter(|p| is_process_def(p) || is_enforcement_surface(p))
        .collect();
    locked.sort_unstable();
    if locked.is_empty() {
        return Vec::new(); // nothing under the lock changed — guard is silent
    }
    let marked = decision_texts.iter().any(|(p, t)| is_decision_file(p) && has_process_marker(t));
    if marked || !charters.is_empty() {
        return Vec::new();
    }
    vec![format!(
        "locked file(s) changed ({}) with NO co-committed process-change Decision (a #ProspectiveChange/#SafetyChange-marked .engine/decisions/*.sysml) and NO co-committed sprint record chartered by a marked Decision the human has ACCEPTED (D0465). HARD LOCK: process definitions (D0070) AND the enforcement surface — guard source, hook config, CI workflows (D0209 clause 2) — may change only with a human-signed Decision, because a silently self-modified control is the issue236 self-modification class. Record one with `keel record decision --process-change ...` (the flag emits the marker; issue213), or land the edit in a sprint whose #CharteredBy names the accepted Decision.",
        locked.join(", ")
    )]
}

/// Guard: a staged process-def change must carry a co-committed marked Decision (D0070).
///
/// A staged change to `.engine/processes|workflows/*.sysml` MUST be co-committed with a
/// `#ProspectiveChange`/`#SafetyChange`-marked Decision (the keystone hard lock). Mirrors
/// `validate_process_change.py`.
#[must_use]
pub fn process_change(root: &Path) -> GuardReport {
    // MODIFIED or DELETED, never merely ADDED (issue272). The keystone lock exists so a control
    // cannot be weakened without a signed Decision. An ADDED locked file is a control ARRIVING, and
    // treating that as a weakening made a freshly scaffolded project unable to make its FIRST
    // commit: `keel init` stages every `.engine/processes/*` file, so the guard demanded a
    // process-change Decision for a scaffold the author did not write. That is the first thing a new
    // user does, and it failed.
    //
    // STATED RESIDUAL: an added process CAN weaken, by asserting a constraint that claims a CORE
    // guard and so makes it switchable (the issue242 capture). That transition is caught by
    // `audit-adherence`'s guard-state monotonicity, which ranks CORE above ACTIVE and fails the
    // build on a downgrade (D0209 clause 1) — so the hole is covered, by the check built for it.
    let read = ChangeRead::current();
    let (resynced, changed): (Vec<String>, Vec<String>) = changed_paths(root, "MDR", read)
        .into_iter()
        .partition(|p| is_locked_path(p) && is_engine_written(root, p, read));
    // TWO LISTS, and conflating them was a regression this very guard caught on its author within
    // the minute: what TRIGGERS the lock is a locked file being modified or deleted, but the
    // co-committed Decision that AUTHORISES it is almost always a NEW file — so searching for it in
    // the modify-only list found nothing and refused a properly signed change. The Decision is
    // looked for among all staged additions and modifications.
    let decision_texts: Vec<(String, String)> = changed_files(root, read)
        .iter()
        .filter(|p| is_decision_file(p))
        .map(|p| (p.clone(), changed_text(root, p, read)))
        .collect();
    // D0465: the second source - a co-committed sprint record chartered by a marked Decision the
    // human has ACCEPTED. Named in a warning line so the authorisation is visible in the gate output.
    let charters = accepted_charters(root, read);
    let violations = keystone_violations(&changed, &decision_texts, &charters);
    let scanned = changed.iter().filter(|p| is_locked_path(p)).count() + resynced.len();
    let mut warnings = vec![read_line(read)];
    if violations.is_empty() && changed.iter().any(|p| is_locked_path(p)) {
        for (dname, sprint) in &charters {
            warnings.push(format!("locked edit authorised by {dname}, a marked Decision the human accepted, cited as the charter of the co-committed sprint record {sprint} (D0465)"));
        }
    }
    if !resynced.is_empty() {
        warnings.push(format!(
            "engine-written: {} locked file(s) carry exactly the text the engine embedded in this binary writes - its resync (D0441) or its verb respell (D0523) - and are outside the lock: the engine arriving, not a control edited: {}",
            resynced.len(),
            resynced.join(", ")
        ));
    }
    // D0204 (pullOversight): the short-lived D0200 clause-5 throttle - refusing process-def commits
    // while the human's review queue aged - is deliberately ABSENT. Nothing gates the AI's work on
    // the human's attention cadence; the records stay auditable, the ask is gone.
    GuardReport { name: "process-change", scanned, warnings, violations }
}

// ── unit-extras-present (issue290 / D0300): a unit's declared mechanism is in the tree ──────────

/// Every `[unit]` section of `unit-extras.toml` with its declared `files`, in file order.
pub(crate) fn declared_extras(root: &Path) -> Vec<(String, Vec<String>)> {
    let Ok(text) = keel_model::corpus::read_to_string(root.join(".engine/contracts/unit-extras.toml")) else {
        return Vec::new();
    };
    let mut out: Vec<(String, Vec<String>)> = Vec::new();
    let mut in_files = false;
    for line in text.lines() {
        let l = line.trim();
        if l.starts_with('#') {
            continue;
        }
        if let Some(name) = l.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
            out.push((name.to_string(), Vec::new()));
            in_files = false;
            continue;
        }
        // A key line may carry its array INLINE (`files = ["a", "b"]`) or open a multi-line one; both
        // are TOML, and the first draft of this parser (like `process_cmd::unit_extras`) read only the
        // second - a one-line declaration scanned as zero files, silently.
        if let Some(rest) = l.strip_prefix("files") {
            let rest = rest.trim_start_matches([' ', '=']).trim();
            if let Some(inline) = rest.strip_prefix('[').filter(|r| r.contains(']')) {
                if let Some(body) = inline.split(']').next() {
                    if let Some((_, files)) = out.last_mut() {
                        files.extend(body.split(',').map(|v| v.trim().trim_matches('"').to_string()).filter(|v| !v.is_empty()));
                    }
                }
                in_files = false;
            } else {
                in_files = true;
            }
            continue;
        }
        if l.starts_with("requires") || l == "]" {
            in_files = false;
            continue;
        }
        if in_files {
            let v = l.trim_end_matches(',').trim().trim_matches('"');
            if let (false, Some((_, files))) = (v.is_empty(), out.last_mut()) {
                files.push(v.to_string());
            }
        }
    }
    out
}

/// The process names `installed-units.toml` records as installed here.
pub(crate) fn installed_unit_names(root: &Path) -> Vec<String> {
    keel_model::corpus::read_to_string(root.join(".engine/contracts/installed-units.toml"))
        .unwrap_or_default()
        .lines()
        .filter_map(|l| l.trim().strip_prefix("process = "))
        .map(|v| v.trim().trim_matches('"').to_string())
        .collect()
}

/// Pure core (issue290): for every INSTALLED unit that declares extras, each declared file must exist;
/// a missing one is a violation naming unit and path. A unit not installed here is not judged - its
/// extras are somebody else's mechanism.
pub(crate) fn extras_violations(declared: &[(String, Vec<String>)], installed: &[String], exists: &dyn Fn(&str) -> bool) -> (usize, Vec<String>) {
    let mut scanned = 0usize;
    let mut violations = Vec::new();
    for (unit, files) in declared {
        if !installed.iter().any(|u| u == unit) {
            continue;
        }
        for f in files {
            scanned += 1;
            if !exists(f) {
                violations.push(format!(
                    "unit `{unit}` declares `{f}` as part of its mechanism (unit-extras.toml) and the file is not in this tree - the process definition and its skill reference machinery that does not exist here (issue290: penumbra adopted a unit whose four mechanism files were hand-staged out). Import the unit with `keel process import` rather than staging files by hand, or remove the declaration if the unit no longer carries it."
                ));
            }
        }
    }
    (scanned, violations)
}

/// Guard: a declared unit extra exists in every project that installed the unit (D0300).
///
/// The tool-reference guard already refuses a doc naming a deleted `.engine/tools/` file; this is the
/// same predicate for a unit's declared MECHANISM - workflows, scripts, contracts a process needs to
/// RUN. issue290: a project adopted a unit whose definition and skill were present and whose four
/// mechanism files were not, because they were hand-staged out of the PR; nothing caught it, since
/// adoption-check gates what EXPORT produces and never what a target received.
#[must_use]
pub fn unit_extras_present(root: &Path) -> GuardReport {
    let declared = declared_extras(root);
    let installed = installed_unit_names(root);
    let (scanned, violations) = extras_violations(&declared, &installed, &|f| root.join(f).exists());
    GuardReport { name: "unit-extras-present", scanned, warnings: Vec::new(), violations }
}

// ── stpa-currency guard (D0313: the computed control structure says when STPA must run again) ────

/// WARNING-tier: every computed control action (`keel show control-structure`, D0284) has been walked
/// by at least one recorded `stpa-self` run, or the commit names the ones no run has looked at.
///
/// A run is a `Test` whose `procedureText` opens `ANALYSED: <name>, <name>, ...` - the computed action
/// names it walked (stpa5, `.engine/processes/stpa-self.sysml`). The union over every recorded run is
/// what counts as analysed: an analysis of `cmdRecord` does not lapse because a later run walked
/// `cmdLand`. What DOES make it lapse is the structure growing a name - a new hook event, workflow
/// step or write command - which is exactly the re-run trigger the process defines, and the only one
/// a text model can compute (a changed body behind an unchanged name is the stated residual).
///
/// Why a warning and not a block (D0098): an unanalysed action is not dishonest state; it is work the
/// structure has queued. Why it compares against the LOCAL structure only: the remote's
/// branch-protection row is fetched live with `gh`, and a commit gate must not need the network - so
/// `remoteRefusesRewrite` is never asked for here.
///
/// A project with no recorded run has not adopted the process and hears nothing (D0136: absence is
/// a state, stated as a zero population).
#[must_use]
pub fn stpa_currency(root: &Path) -> GuardReport {
    let (runs, analysed) = analysed_actions(root);
    let computed = keel_view::view::control_structure::local_actions(root);
    let warnings = currency_warnings(runs, &analysed, &computed);
    let scanned = if runs == 0 { 0 } else { computed.len() };
    GuardReport { name: "stpa-currency", scanned, warnings, violations: Vec::new() }
}

/// Every `ANALYSED:` list under `.tracking`: how many runs, and the union of the names they walked.
pub(crate) fn analysed_actions(root: &Path) -> (usize, HashSet<String>) {
    let mut runs = 0usize;
    let mut names = HashSet::new();
    for path in keel_model::corpus::collect_sysml(&root.join(".tracking")) {
        let Ok(text) = keel_model::corpus::read_to_string(&path) else { continue };
        for (i, _) in text.match_indices(":>> procedureText = \"ANALYSED:") {
            runs += 1;
            let rest = &text[i + ":>> procedureText = \"ANALYSED:".len()..];
            names.extend(analysed_list(rest));
        }
    }
    (runs, names)
}

/// The names in one `ANALYSED:` list - everything up to the first `.` or closing quote, comma-split.
pub(crate) fn analysed_list(rest: &str) -> Vec<String> {
    let end = rest.find(['.', '"']).unwrap_or(rest.len());
    rest[..end].split(',').map(str::trim).filter(|n| !n.is_empty()).map(str::to_string).collect()
}

/// One warning a reader can act on (D0410, issue403): the remainder grouped by the edge each action
/// sits on, the target stated (every edge walked in full; a tranche is one edge), and the NEXT tranche
/// named - the open edge with the fewest unanalysed actions, ties broken by edge name - with the
/// exact `ANALYSED:` list its run record will carry. Silence when there is no run or nothing is open.
///
/// The count stays in the message because it is the burndown; what changed is that the count is no
/// longer the whole message. A standing "39 of 48" told the reader nothing about where to start, so
/// nobody did (the D0359 failure in another surface: accurate every time, and therefore unread).
pub(crate) fn currency_warnings(runs: usize, analysed: &HashSet<String>, computed: &[keel_view::view::control_structure::LocalAction]) -> Vec<String> {
    // edges in structure order; each carries the names still open and the edge's total
    struct Edge<'a> {
        name: String,
        open: Vec<&'a str>,
        total: usize,
    }
    if runs == 0 {
        return Vec::new();
    }
    let mut edges: Vec<Edge<'_>> = Vec::new();
    for a in computed {
        let name = format!("{}->{}", a.issued_by, a.acts_on);
        if !edges.iter().any(|e| e.name == name) {
            edges.push(Edge { name: name.clone(), open: Vec::new(), total: 0 });
        }
        if let Some(e) = edges.iter_mut().find(|e| e.name == name) {
            e.total += 1;
            if !analysed.contains(&a.name) && !e.open.contains(&a.name.as_str()) {
                e.open.push(a.name.as_str());
            }
        }
    }
    let open: Vec<&Edge<'_>> = edges.iter().filter(|e| !e.open.is_empty()).collect();
    let Some(next) = open.iter().min_by(|a, b| a.open.len().cmp(&b.open.len()).then_with(|| a.name.cmp(&b.name))) else {
        return Vec::new();
    };
    let missing: usize = open.iter().map(|e| e.open.len()).sum();
    let sorted = |names: &[&str]| {
        let mut v = names.to_vec();
        v.sort_unstable();
        v.join(", ")
    };
    let groups: Vec<String> = open
        .iter()
        .map(|e| {
            let count = if e.open.len() == e.total { e.open.len().to_string() } else { format!("{} of {} open", e.open.len(), e.total) };
            format!("{} ({count}): {}", e.name, sorted(&e.open))
        })
        .collect();
    vec![format!(
        "stpa-currency: {} of {} computed control action(s) no stpa-self run has analysed. TARGET: every controller->process edge walked in full; a run's tranche is one edge. OPEN EDGES: {}. NEXT TRANCHE: {} - run the stpa-self process over it and record `ANALYSED: {}.` (D0313, D0410)",
        missing,
        computed.len(),
        groups.join("; "),
        next.name,
        sorted(&next.open)
    )]
}

// ── doc-sync guard (D0113: the doc-sync discipline made a CONTROL — was pure vigilance) ────────────

/// A staged path whose change is DEFINITIONAL and near-always carries doc implications: `.engine`
/// schema / process / workflow definitions. (Tool `.rs` code + skills are deliberately EXCLUDED to keep
/// this low-noise; the scope can widen once proven, D0113.)
pub(crate) fn is_doc_governed_def(p: &str) -> bool {
    p.starts_with(".engine/schema/")
        || p.starts_with(".engine/processes/")
        || p.starts_with(".engine/workflows/")
}

/// A staged path that COUNTS as a doc update (satisfies doc-sync): `CLAUDE.md`, `.engine/docs/`, or any
/// `README.md`.
pub(crate) fn is_doc_file(p: &str) -> bool {
    p == "CLAUDE.md" || p.starts_with(".engine/docs/") || p.ends_with("README.md")
}

/// Pure core: a staged definitional change (schema/process/workflow) with NO co-committed doc update
/// yields one warning naming the offending files. Empty when nothing definitional changed OR a doc did.
pub(crate) fn doc_sync_warnings(changed: &[String]) -> Vec<String> {
    let mut governed: Vec<&str> = changed.iter().map(String::as_str).filter(|p| is_doc_governed_def(p)).collect();
    governed.sort_unstable();
    if governed.is_empty() || changed.iter().any(|p| is_doc_file(p)) {
        return Vec::new();
    }
    vec![format!(
        "definitional change ({}) with NO co-committed doc update (CLAUDE.md / .engine/docs / README) — run the doc-sync skill; fix any doc claim this change invalidates in THIS commit",
        governed.join(", ")
    )]
}

/// Guard (WARNING-level, D0113): a staged schema/process/workflow change co-committed with no doc update.
///
/// Converts the doc-sync discipline from pure vigilance into a visible control — documentation drift was
/// a recorded HIGH critique finding, and doc-sync had no enforcing guard (only the skill). Heuristic +
/// WARNING (the D0102 promote-once-low-noise pattern, like `decision-requirement-link`): a definitional
/// change MIGHT legitimately need no doc, so this NUDGES (never blocks); promote to hard once proven
/// low-noise. Shares the `staged_files` git mechanism with `process_change`.
#[must_use]
pub fn doc_sync(root: &Path) -> GuardReport {
    let read = ChangeRead::current();
    let changed = changed_files(root, read);
    let scanned = changed.iter().filter(|p| is_doc_governed_def(p)).count();
    let mut warnings = doc_sync_warnings(&changed);
    warnings.push(read_line(read));
    GuardReport { name: "doc-sync", scanned, warnings, violations: Vec::new() }
}

// ── manifest-coverage guard (deliverable-suspicion manifest stays valid + complete) ────────────

/// Name fragments that mark a task as likely deliverable-source-dependent (a verification whose
/// evidence is the Rust deliverable behaving correctly) — used for the unlisted-task WARNING.
pub(crate) const DELIVERABLE_TASK_HINTS: &[&str] = &["rust", "Parser", "writeApi", "runtimeParser", "specVersion"];

/// Parse the deliverable manifest into `(task, paths)` entries (`task: NAME | p1 p2`; `#` comments).
pub(crate) fn parse_manifest(text: &str) -> Vec<(String, Vec<String>)> {
    let mut out = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        let Some(rest) = t.strip_prefix("task:") else { continue };
        let mut parts = rest.splitn(2, '|');
        let Some(name) = parts.next().map(str::trim) else { continue };
        let paths: Vec<String> = parts.next().unwrap_or("").split_whitespace().map(str::to_string).collect();
        if !name.is_empty() {
            out.push((name.to_string(), paths));
        }
    }
    out
}

/// Guard (D0050/issue033): the deliverable-suspicion manifest stays VALID + complete.
///
/// VIOLATION: a manifest entry names a task that no longer exists, or lists a path that no longer
/// exists (a dead entry silently drops deliverable-suspicion coverage — the d0050 finding).
/// WARNING: a declared task whose name looks deliverable-dependent but is not manifest-listed
/// (a possible unguarded verification — the manifest is a hand-maintained allow-list).
#[must_use]
pub fn manifest_coverage(root: &Path) -> GuardReport {
    let path = root.join(".engine").join("deliverable-manifest.txt");
    let Ok(text) = keel_model::corpus::read_to_string(&path) else {
        return GuardReport { name: "manifest-coverage", scanned: 0, warnings: Vec::new(), violations: vec![format!("cannot read {}", relpath(root, &path))] };
    };
    let entries = parse_manifest(&text);
    let tasks = declared_task_names(root);
    let listed: HashSet<&str> = entries.iter().map(|(n, _)| n.as_str()).collect();
    let mut warnings = Vec::new();
    let mut violations = Vec::new();
    for (name, paths) in &entries {
        if !tasks.contains(name) {
            violations.push(format!("manifest entry '{name}' names a task that no longer exists (dead entry — deliverable-suspicion coverage silently lost)"));
        }
        for p in paths {
            if !root.join(p).exists() {
                violations.push(format!("manifest entry '{name}' lists path '{p}' which no longer exists"));
            }
        }
    }
    // Exclude sprint-wrapper actions (story*) — the manifest is about BACKLOG deliverable tasks, and
    // a "storyParser*" wrapper matching the "Parser" hint is a false positive, not a manifest gap.
    let mut unlisted: Vec<&String> = tasks
        .iter()
        .filter(|t| !t.starts_with("story") && !listed.contains(t.as_str()) && DELIVERABLE_TASK_HINTS.iter().any(|h| t.contains(h)))
        .collect();
    unlisted.sort();
    for t in unlisted {
        warnings.push(format!("task '{t}' looks deliverable-dependent (name) but is NOT in deliverable-manifest.txt — confirm it needs no source-drift suspicion"));
    }
    GuardReport { name: "manifest-coverage", scanned: entries.len(), warnings, violations }
}

// ── process-skill guard (D0059/issue036: no inert process — every process has a deploying skill) ──

/// Every `.engine/processes/<file>.sysml` path referenced anywhere in the skills-registry text.
pub(crate) fn referenced_processes(reg: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    for tok in reg.split(|c: char| c.is_whitespace() || c == '"') {
        if let Some(rest) = tok.strip_prefix(".engine/processes/") {
            if let Some(idx) = rest.find(".sysml") {
                out.insert(rest[..idx + ".sysml".len()].to_string());
            }
        }
    }
    out
}

/// Coverage logic for the process-skill guard (pure, for self-test): every process file must be
/// referenced by ≥1 skill, and every referenced path must name an existing process.
pub(crate) fn process_skill_violations(processes: &[String], reg: &str) -> Vec<String> {
    let referenced = referenced_processes(reg);
    let mut violations = Vec::new();
    for p in processes {
        if !referenced.contains(p) {
            violations.push(format!("process '.engine/processes/{p}' has NO deploying skill (inert process — D0059; a deploying skill's purpose must name the process .sysml it deploys)"));
        }
    }
    let proc_set: HashSet<&str> = processes.iter().map(String::as_str).collect();
    let mut dangling: Vec<&String> = referenced.iter().filter(|r| !proc_set.contains(r.as_str())).collect();
    dangling.sort();
    for r in dangling {
        violations.push(format!("skill registry references '.engine/processes/{r}' which does not exist (dangling deploying claim — orphan skill edge)"));
    }
    violations
}

/// Guard (D0059/issue036): every process definition has a DEPLOYING skill ("no inert process").
///
/// D0059 establishes that a process with no deploying skill is applied by inconsistent vigilance (a
/// HIGH finding that recurred); the d0059 critique found the claimed coverage audit never existed.
/// The correspondence is a uniform CONVENTION — a deploying skill's `purpose` names the
/// `.engine/processes/<name>.sysml` it deploys — and this guard makes it machine-checkable.
///
/// VIOLATION: a process file referenced by NO skill (inert), or a skill referencing a process that
/// does not exist (a dangling deploying claim). A view-only skill that deploys no process is fine
/// (the audit is process→skill, not the reverse).
#[must_use]
pub fn process_skill(root: &Path) -> GuardReport {
    let proc_dir = root.join(".engine").join("processes");
    let processes: Vec<String> = keel_model::corpus::collect_sysml(&proc_dir)
        .iter()
        .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(str::to_string))
        .collect();
    let reg_path = root.join(".engine").join("skills").join("skills-registry.sysml");
    let Ok(central) = keel_model::corpus::read_to_string(&reg_path) else {
        return GuardReport { name: "process-skill", scanned: 0, warnings: Vec::new(), violations: vec![format!("cannot read {}", relpath(root, &reg_path))] };
    };
    // D0220: a skill may declare its deployment BESIDE ITSELF, in any `.sysml` under
    // `.engine/skills/`, not only in the central registry. Adopting decision-channel on penumbra
    // proved why: the unit carried the SKILL.md but the registry ENTRY that binds skill->process
    // stayed home, so `process-skill` failed in the receiving project on the very first run - a new
    // project's first experience of an adopted unit was a red gate. Reading the whole directory lets
    // a unit ship its own registration as a file, so nothing has to be text-merged into a shared
    // registry (the hazard the rules layer already avoids by carrying rules BY NAME).
    let mut reg = central;
    for f in keel_model::corpus::collect_sysml(&root.join(".engine").join("skills")) {
        if f == reg_path {
            continue;
        }
        if let Ok(extra) = keel_model::corpus::read_to_string(&f) {
            reg.push('\n');
            reg.push_str(&extra);
        }
    }
    let violations = process_skill_violations(&processes, &reg);
    GuardReport { name: "process-skill", scanned: processes.len(), warnings: Vec::new(), violations }
}

/// Diagnostic (D0047/issue039): a `#ProcessDefect` finding must resolve to a guard-producing action.
///
/// RUNNABLE via `keel gate guard defect-guard-coverage` but NOT in the enforced `GUARD_NAMES` — whether
/// a defect class "needs a guard" is judgment-bound (a shallow heuristic on the resolver name), so it
/// is a WARN for human attention, not a commit gate. Closes issue039: the "corrections become guards"
/// rule (D0047) now has an audit instead of relying purely on vigilance.
#[must_use]
pub fn defect_guard_coverage(root: &Path) -> GuardReport {
    match keel_view::view::defect_guard_coverage(root) {
        Ok((examined, warnings)) => GuardReport { name: "defect-guard-coverage", scanned: examined, warnings, violations: Vec::new() },
        Err(e) => GuardReport { name: "defect-guard-coverage", scanned: 0, warnings: Vec::new(), violations: vec![format!("error reading defect-guard coverage: {e}")] },
    }
}

// ── doc-guard-count guard (issue246: a count typed into prose beside a computable one) ──────────

/// Guard: the doc surface must not hardcode the TOTAL guard count.
///
/// issue246. `CLAUDE.md` said "all 45 forward guards" and `.engine/docs/guards.md` said "runs **45**
/// forward guards" while 48 ran — and guards.md's very next sentence said `keel version` reports the
/// split "so the number has one home". A number typed into prose beside a number the engine can
/// compute is the highest-frequency drift mechanism in this repository, and the 2026-08-24 panel found
/// the brief that DIAGNOSED report-vs-truth drift committing it twice itself.
///
/// The rule is DELETION, not reconciliation: D0105 gives every fact one canonical home, and the home
/// of this one is `keel version`. Syncing 45 to 48 would drift again on guard 49; forbidding the
/// literal cannot. Narrow by construction — it fires only on a digit immediately preceding
/// "forward guards" or on "all N guards", so a SUBSET count ("5 guards are rule-sourced") and an
/// ordinal reference ("Guard 37 checks...") are untouched.
///
/// Second clause (issue584, sprint 756): a catalogue row's `Family` cell equals the family whose table
/// dispatches the guard. guards.md groups guards by TIER while the code groups them by family
/// (`members/keel-guards/src/<family>.rs`, sprint 733), so a reader of the catalogue could not find a
/// guard's code without grepping; the column is the pointer, and this clause is what keeps it true when
/// a guard moves between family files without its row. The clause reads a file ONLY when its table header
/// declares the column - a downstream copy shipped before the column existed claims nothing (D0419's
/// lesson: an engine-self check that reads a scaffold turned CI red for six pushes).
#[must_use]
pub fn doc_guard_count(root: &Path) -> GuardReport {
    let mut files: Vec<PathBuf> = vec![root.join("CLAUDE.md")];
    if let Ok(rd) = std::fs::read_dir(root.join(".engine").join("docs")) {
        files.extend(rd.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "md")));
    }
    let mut scanned = 0usize;
    let mut violations = Vec::new();
    let actual = GUARD_NAMES.len();
    for f in files {
        let Ok(text) = keel_model::corpus::read_to_string(&f) else { continue };
        scanned += 1;
        let rel = f.strip_prefix(root).unwrap_or(&f).to_string_lossy().replace('\\', "/");
        for (n, line) in text.lines().enumerate() {
            if let Some(claim) = total_guard_count_claim(line) {
                violations.push(format!(
                    "{rel}:{}: states a TOTAL guard count (`{claim}`) while {actual} guards are enforced - the count has ONE home, `keel version` (D0105/issue246). Delete the number; point at the computed source.",
                    n + 1
                ));
            }
        }
        violations.extend(family_cell_violations(&rel, &text));
    }
    GuardReport { name: "doc-guard-count", scanned, warnings: Vec::new(), violations }
}

/// The header a catalogue table carries once it declares the family column.
const FAMILY_COLUMN_HEADER: &str = "| Guard | Family |";

/// Every guard row of a catalogue file whose table header declares a `Family` column, as
/// `(line number, guard, the cell)`; a file without the header claims nothing.
pub(crate) fn guard_family_claims(text: &str) -> Vec<(usize, String, String)> {
    if !text.lines().any(|l| l.starts_with(FAMILY_COLUMN_HEADER)) {
        return Vec::new();
    }
    text.lines()
        .enumerate()
        .filter_map(|(n, line)| {
            let (name, rest) = line.strip_prefix("| `")?.split_once("` |")?;
            let cell = rest.split('|').next()?.trim();
            Some((n + 1, name.to_owned(), cell.to_owned()))
        })
        .collect()
}

/// The family whose table dispatches `guard`, read from `FAMILIES`; `None` for a name no table holds.
pub(crate) fn family_of(guard: &str) -> Option<&'static str> {
    FAMILIES.iter().find(|f| f.arms.iter().any(|(n, _)| *n == guard)).map(|f| f.name)
}

/// The rows of `text` whose family cell is absent or names a family other than the one dispatching the
/// guard. A row for a name no family holds is not this clause's (the catalogue-row test owns the
/// reverse direction).
pub(crate) fn family_cell_violations(rel: &str, text: &str) -> Vec<String> {
    guard_family_claims(text)
        .into_iter()
        .filter_map(|(n, name, cell)| {
            let actual = family_of(&name)?;
            if cell == actual {
                return None;
            }
            Some(if FAMILIES.iter().any(|f| f.name == cell) {
                format!("{rel}:{n}: row `{name}` names family `{cell}` but `{name}` is dispatched by family `{actual}` (members/keel-guards/src/{actual}.rs) - the row moves with the guard (issue584)")
            } else {
                format!("{rel}:{n}: row `{name}` carries no family cell (`{cell}`) while the table declares the column; `{name}` is dispatched by family `{actual}` (members/keel-guards/src/{actual}.rs, issue584)")
            })
        })
        .collect()
}

/// The hardcoded TOTAL-count phrase in a line, if any. `None` for subset counts and ordinals.
pub(crate) fn total_guard_count_claim(line: &str) -> Option<String> {
    // "<digits>[**] forward guards" — the digits may be wrapped in markdown emphasis.
    for pat in ["forward guards", "guards, kernel-free"] {
        if let Some(i) = line.find(pat) {
            let before: String = line[..i].chars().rev().take(12).collect::<String>().chars().rev().collect();
            let stripped: String = before.chars().filter(|c| *c != '*' && *c != '_').collect();
            if stripped.trim_end().chars().last().is_some_and(|c| c.is_ascii_digit()) {
                return Some(format!("{}{pat}", before.trim_start()));
            }
        }
    }
    // The "all <N> guards" branch was REMOVED after a probe against the real corpus: guards.md
    // legitimately narrates history ("passed validate and all 37 guards"), a TRUE statement about the
    // past that must not be forbidden. Only the canonical CURRENT-total phrasing is checked, which is
    // what both drifted sites actually used. A guard that fires on true prose gets bypassed.
    None
}

// ── control-map-reconciled guard (issue304, chartered by D0255) ──────────────────────────────────

/// Every control event names a DECLARED control, or says why it is instrumentation instead.
///
/// # The third failure class, and why only a check closes it
///
/// D0217 named declared-but-never-fired. D0253 named declared-and-unprobeable. Both are visible to a
/// reader who looks. This closes the one that is visible to NOBODY: a control that is IMPLEMENTED and
/// FIRING while the control map does not know it exists. `keel controls` computes the hazard/control
/// diff over DECLARED controls and DECLARED hazards, so an undeclared control cannot appear as a gap —
/// and neither can a hazard only that control covers.
///
/// The perverse property is what makes a check mandatory rather than a habit: the coverage measure
/// IMPROVES as the map gets less complete, because fewer declared controls with no gaps reads better
/// than more declared controls with gaps. Reconciling by hand once would leave that incentive intact.
///
/// Found this way, not by reasoning: the map declared nine controls while `control-events.toml`
/// declared fourteen events, two of which — the post-edit fast tier and the turn-boundary stop gate —
/// BLOCK, and neither was in the map.
///
/// # Why events are the anchor
///
/// A control that can fire leaves a counted record, and `control-event-coverage` (D0193) already
/// cross-checks that declaration against the event names the binary actually emits. Anchoring here
/// chains binary → events → controls, so the map is reconciled against something already tied to the
/// code rather than against a second hand-maintained list — which would be one more thing to drift.
pub(crate) fn control_map_reconciled(root: &Path) -> GuardReport {
    let path = root.join(".engine").join("contracts").join("control-events.toml");
    let Ok(text) = keel_model::corpus::read_to_string(&path) else {
        // D0136: absence is a state, stated. A project that never adopted control events has nothing
        // to reconcile, and reporting a violation would fire on a project that opted out.
        return GuardReport { name: "control-map-reconciled", scanned: 0, warnings: Vec::new(), violations: Vec::new() };
    };
    let declared: std::collections::HashSet<String> = keel_model::corpus::collect_sysml(&root.join(".tracking"))
        .iter()
        .filter_map(|p| keel_model::corpus::read_to_string(p).ok())
        .flat_map(|t| {
            t.match_indices("part ctl")
                .filter_map(|(i, _)| {
                    t.get(i + 5..).and_then(|rest| {
                        rest.split_whitespace().next().map(std::string::ToString::to_string)
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect();

    // SCOPED TO ADOPTION (D0231/issue090, caught by the adoption-check test on a fresh scaffold).
    // `keel init` ships this contract because it is engine vocabulary, but the control MAP is a
    // project's own instance data and is not shipped — so a fresh project inherits events naming
    // controls it has never declared, and an unscoped check fires on every downstream tree for
    // controls that are none of its business. A project with no control map has nothing to
    // reconcile: report the zero rather than a violation, so out-of-scope reads as out-of-scope.
    //
    // RESIDUAL, stated because it is real: a downstream project that builds its OWN control map
    // still inherits the shipped `control` bindings, which name this project's controls. Those
    // bindings are instance knowledge riding in an engine file, and until they are separated the
    // guard would mis-fire there too. Recorded rather than hidden behind a passing check.
    if declared.is_empty() {
        return GuardReport { name: "control-map-reconciled", scanned: 0, warnings: Vec::new(), violations: Vec::new() };
    }
    let mut scanned = 0usize;
    let mut violations = Vec::new();
    let mut event = String::new();
    for line in text.lines() {
        let l = line.trim();
        if let Some(name) = l.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
            event = name.to_string();
            scanned += 1;
        } else if let Some(v) = l.strip_prefix("control = ") {
            let ctl = v.trim().trim_matches('"');
            if ctl != "none" && !declared.contains(ctl) {
                violations.push(format!(
                    "control-events.toml [{event}] names control `{ctl}`, which no .tracking/ file declares — an event that fires for an UNDECLARED control is the third failure class (D0254): `keel controls` computes over declared controls, so this one cannot appear as a gap and its absence makes coverage read cleaner rather than worse"
                ));
            }
        }
    }
    // An event with NO control line at all is the silent case this guard exists for: it neither
    // claims a control nor states that it is instrumentation.
    for block in text.split('[').skip(1) {
        let Some((name, body)) = block.split_once(']') else { continue };
        if !body.contains("control = ") {
            violations.push(format!(
                "control-events.toml [{name}] declares no `control` — say which declared control it is the firing of, or `control = \"none\"` with a `controlNote` saying why it is instrumentation. Silence must read as a gap, never as consent (the process-enforcement.toml convention)"
            ));
        }
    }
    // issue308 (propriety panel, pf33): a `provenBy` in control-arming.toml naming a file the tree
    // does not hold is a receipt-shaped TESTIMONY — worse than no claim, in the very contract that
    // exists to separate the two (D0253). Existence is the objective half and is checked here; that
    // the named test EXERCISES the control stays with the panel, being judgment (pf35).
    if let Ok(arming) = keel_model::corpus::read_to_string(root.join(".engine").join("contracts").join("control-arming.toml")) {
        for line in arming.lines() {
            let Some(v) = line.trim().strip_prefix("provenBy = ") else { continue };
            let rel = v.trim().trim_matches('"');
            scanned += 1;
            if !rel.is_empty() && !root.join(rel).exists() {
                violations.push(format!(
                    "control-arming.toml names provenBy `{rel}`, which does not exist in the tree — a dangling proof pointer reads as a receipt while being a testimony (issue308/D0253). Fix the path, or remove the claim"
                ));
            }
        }
    }
    GuardReport { name: "control-map-reconciled", scanned, warnings: Vec::new(), violations }
}

// ── manifest-key-portability guard (issue301, chartered by D0250) ────────────────────────────────

/// Every key in `installed-units.toml` must be repository-relative.
///
/// # Why a guard and not a one-time cleanup
///
/// Four keys under the `decision-channel` unit named this machine's home directory
/// (`file.C:__SL__Users__SL__<user>__SL__...`), because `unit_files` puts a unit's declared EXTRAS at
/// `root/<extra>` while the key builder stripped only the `.engine` prefix and fell through to the
/// absolute path. Fixing the builder and rewriting the four keys leaves nothing to stop the fifth:
/// the next extra added outside `.engine` would reintroduce it silently. D0047 — a defect that can
/// recur becomes a control, never a corrected file.
///
/// # Why this is now blocking rather than advisory
///
/// D0250 makes the library a git repository that other machines clone. A key naming the exporting
/// machine resolves to nothing on the importing one, so the three-way base that `--update` merges
/// against stops being found — silently, in the one file whose entire purpose is portability. The
/// failure surfaces on the SECOND machine, days later, as content mysteriously not updating.
///
/// Detection is by shape, not by this machine's paths: a Windows drive letter, a POSIX absolute
/// path, or a home-directory prefix. A guard that looked for `WilliamWeatherholtz` would pass on
/// every machine except the one that already got it right.
pub(crate) fn manifest_key_portability(root: &Path) -> GuardReport {
    let path = root.join(".engine").join("contracts").join("installed-units.toml");
    let Ok(text) = keel_model::corpus::read_to_string(&path) else {
        // D0136: absence is a state, stated. A project with no installed units has no manifest.
        return GuardReport { name: "manifest-key-portability", scanned: 0, warnings: Vec::new(), violations: Vec::new() };
    };
    let mut scanned = 0usize;
    let mut violations = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.trim().strip_prefix("file.") else { continue };
        let Some((key, _)) = rest.split_once(" = ") else { continue };
        scanned += 1;
        let decoded = key.replace("__SL__", "/");
        let absolute = decoded.starts_with('/')
            || decoded.starts_with('~')
            || decoded.as_bytes().get(1).is_some_and(|c| *c == b':');
        // issue307 (propriety panel, pf02): the class is "resolves outside the receiving project",
        // and absolute keys are only its loudest members. A RELATIVE key with traversal segments
        // (`../../elsewhere`) escapes the project root identically — the mutation the original
        // predicate did not kill. Segment-wise, not substring: a filename containing ".." is legal.
        let traverses = decoded.split('/').any(|seg| seg == "..");
        if absolute || traverses {
            let kind = if absolute { "an ABSOLUTE path" } else { "a TRAVERSAL path (contains a `..` segment)" };
            violations.push(format!(
                "{}: unit-manifest key `{decoded}` is {kind} — it resolves outside the receiving project, so a clone of this library cannot reconstruct it and the three-way `--update` base is silently lost (issue301/issue307/D0250). Keys are repository-relative and stay inside the root; a unit file outside it is refused at export, never absolutised",
                relpath(root, &path)
            ));
        }
    }
    GuardReport { name: "manifest-key-portability", scanned, warnings: Vec::new(), violations }
}

/// Guard 65: every measurement instrument in the tree is a declared `Sensor`, and every Sensor's
/// mechanism exists (D0361/D0363, scenario S-F7).
///
/// The feedback half of this project's control structure is derived from `CliCommand` facts, so an
/// instrument that is not a CLI command is invisible to it: measured 2026-09-06, none of twenty
/// appeared in `keel show control-structure`, and six defects in one day landed in those channels.
///
/// They are now MODEL ITEMS rather than a contract file (the owner's correction: assessment is a
/// verification case, and a verification case needs an endpoint), so this guard joins the tree against
/// `Sensor.mechanism`. Two-way, like `cli-surface-declared`: a script under a watched directory with
/// no Sensor is UNDECLARED; a Sensor naming a path that does not exist is a stale claim.
///
/// A deliberate exclusion is ARGUED IN THE FILE ITSELF: a first line containing
/// `not-an-instrument:` followed by the reason. A project with no Sensor items declares nothing and
/// is not accused - the activation convention.
pub(crate) fn instruments_declared(root: &Path) -> GuardReport {
    let mut violations = Vec::new();
    let declared = keel_view::view::control_structure::declared_sensor_mechanisms(root);
    if declared.is_empty() {
        return GuardReport { name: "instruments-declared", scanned: 0, warnings: Vec::new(), violations };
    }

    let mut found: Vec<String> = Vec::new();
    for dir in [".engine/tools", ".engine/tools/validate", "scripts", "scripts/exec_brief"] {
        let Ok(entries) = std::fs::read_dir(root.join(dir)) else { continue };
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) != Some("py") {
                continue;
            }
            let stem = p.file_stem().and_then(|x| x.to_str()).unwrap_or_default().to_owned();
            if stem.starts_with('_') {
                continue; // a private helper is not a channel
            }
            let rel = format!("{dir}/{stem}.py");
            // A project declares the instruments IT wrote; the tools the ENGINE ships are not its
            // to declare, and this guard's own test caught it accusing a fresh scaffold of them.
            if keel_schema::embedded::is_portable_engine_tool(std::path::Path::new(&rel)) {
                continue;
            }
            // An exclusion argued in the file itself, where a reader of the file can see it.
            let head = keel_model::corpus::read_to_string(&p).unwrap_or_default();
            if head.lines().take(3).any(|l| l.contains("not-an-instrument:")) {
                continue;
            }
            found.push(rel);
        }
    }

    let scanned = found.len();
    for path in &found {
        if !declared.iter().any(|d| d == path) {
            violations.push(format!(
                "{path}: produces a number, verdict or figure but no `Sensor` item declares it - an undeclared measure is outside the computed control structure, so no analysis reaches it and no verification case can verify it (D0361/D0363). Author a Sensor, or argue the exclusion with a `not-an-instrument:` line in the file"
            ));
        }
    }
    for d in &declared {
        if !root.join(d).exists() {
            violations.push(format!(
                "a Sensor declares mechanism `{d}`, which does not exist - a stale measure claims something is being watched that is not"
            ));
        }
    }
    GuardReport { name: "instruments-declared", scanned, warnings: Vec::new(), violations }
}

/// Guard 50: every declared process states the SITUATION in which a project needs it (D0225).
///
/// Onboarding recommends a process set by matching a project's elicited facts against each process's
/// `// APPLIES-WHEN:` condition. A process that declares none is invisible to that match — it can be
/// recommended neither for nor against — so the author's chartered set silently omits it and nobody
/// can tell the omission from a decision. That is the honest-state class (D0098): the guard does not
/// require the set to be COMPLETE, only that a process which exists can be reasoned about.
///
/// Beside the process rather than in a central table, because a central table cannot travel with one
/// unit — the defect that made 23 of 24 units land red on adoption (issue253/D0222).
pub(crate) fn process_applicability(root: &Path) -> GuardReport {
    let dir = root.join(".engine").join("processes");
    let mut scanned = 0usize;
    let mut violations = Vec::new();
    // SCOPED TO ADOPTION (issue259, D0164). The APPLIES-WHEN fact exists to serve `project-onboarding`;
    // a project that has not adopted that process has not violated anything by lacking it, and gating
    // it anyway is the failure D0164 names - a control a project never adopted, enforced against it.
    // Found the hard way: this guard, hours after landing, failed all 22 processes of the first
    // project keel was adopted onto. Reported as 0 scanned rather than silently skipped, so an
    // out-of-scope guard is visible rather than a vacuous pass.
    if !dir.join("project-onboarding.sysml").exists() {
        return GuardReport { name: "process-applicability", scanned, warnings: Vec::new(), violations };
    }
    let Ok(entries) = std::fs::read_dir(&dir) else {
        // No processes directory at all is not a violation: a project may hold no process definitions.
        return GuardReport { name: "process-applicability", scanned, warnings: Vec::new(), violations };
    };
    let mut files: Vec<std::path::PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("sysml"))
        .collect();
    files.sort();
    for path in &files {
        let Ok(text) = keel_model::corpus::read_to_string(path) else { continue };
        // Only files that actually DECLARE a process are in scope - a helper or an include is not.
        if !text.contains(": Process {") {
            continue;
        }
        scanned += 1;
        if !text.lines().any(|l| l.trim().starts_with("// APPLIES-WHEN:")) {
            violations.push(format!(
                "{}: declares a Process but no `// APPLIES-WHEN:` condition - onboarding cannot recommend it for OR against, so a chartered set would omit it silently (D0225)",
                relpath(root, path)
            ));
        }
    }
    GuardReport { name: "process-applicability", scanned, warnings: Vec::new(), violations }
}

/// Guard 41: the keel-owned `.claude/` enforcement surface matches this binary's generator
/// (D0174/P0.2). The check IS `keel sync-claude --check` — one implementation, one surface.
///
/// A project with NO `.claude/` directory has not adopted the in-loop surface and passes with a
/// note (CLI + commit/CI gates remain its enforcement; the D0186 harness-support matrix states
/// this). Version skew is a WARNING ("regenerate"), never a violation — the entries may be
/// semantically current under an older stamp. Drift in the keel-owned subset is a violation:
/// a mutated hook command is a silently weakened control (K7).
#[must_use]
pub fn claude_surface_drift(root: &Path) -> GuardReport {
    if !root.join(".claude").exists() {
        return GuardReport { name: "claude-surface-drift", scanned: 0, warnings: Vec::new(), violations: Vec::new() };
    }
    match keel_write::claude_surface::sync_claude(root, true) {
        Ok(r) => {
            let warnings = r
                .version_skew
                .map(|(old, new)| vec![format!("surface stamped by generator {old}, binary is {new} — run `keel sync-claude` (regenerate obligation)")])
                .unwrap_or_default();
            GuardReport {
                name: "claude-surface-drift",
                scanned: r.registry_count + 2, // settings.json + output style + the skills
                warnings,
                violations: r.drift,
            }
        }
        Err(e) => GuardReport {
            name: "claude-surface-drift",
            scanned: 0,
            warnings: Vec::new(),
            violations: vec![format!("cannot evaluate the surface: {e}")],
        },
    }
}

/// Guard 45 (D0193, WARNING tier): every control-relevant event is DECLARED with its required
/// record, and the declaration matches what the binary emits.
///
/// The family it closes (issues 203/205/207 + the sr13 sentinel): a control-relevant event with no
/// counted record stays invisible until a verification campaign trips over it. The check is a
/// two-way diff between `.engine/contracts/control-events.toml`'s ledger-record sections and the
/// event names the binary's emitters use - a declared event nothing emits warns (dead declaration),
/// an emitted event nothing declares warns (uncounted event). Inventory-record events are checked
/// against the hardening lens's point list by name. Absent contract = not adopted, reported (D0136).
#[must_use]
pub fn control_event_coverage(root: &Path) -> GuardReport {
    /// Every ledger event name the binary emits. A NEW `ledger_emit` call site must add its event
    /// here AND to the contract - this constant going stale is exactly what the two-way diff warns on.
    const EMITTED_LEDGER: [&str; 15] = [
        "post-edit", "stop", "user-prompt", "pre-bash", "pre-write", "subagent-start", "subagent-stop",
        "launch-dirty-refusal", "override-consumed", "override-obligation-UNSYNCED",
        "red-yield-obligation-UNSYNCED", "actor-rebind", "hook-watchdog-timeout",
        "advisory-issued", "advisory-repeated", // issue230: spoken vs silent, and the ignore signal
    ];
    const INVENTORY_POINTS: [(&str, &str); 2] = [("spec-pin-check", "build-time spec pin"), ("pre-push-behind", "pre-push .githooks")];
    let path = root.join(".engine").join("contracts").join("control-events.toml");
    let Ok(text) = keel_model::corpus::read_to_string(&path) else {
        return GuardReport {
            name: "control-event-coverage",
            scanned: EMITTED_LEDGER.len(),
            warnings: vec!["control-events.toml is ABSENT - this control is NOT ADOPTED by this project (D0136: absence is a state, never a violation); the binary's control events go uncounted-by-declaration".to_string()],
            violations: Vec::new(),
        };
    };
    let mut declared_ledger: Vec<String> = Vec::new();
    let mut declared_inventory: Vec<String> = Vec::new();
    let mut current: Option<String> = None;
    for line in text.lines() {
        let l = line.trim();
        if l.starts_with('[') && l.ends_with(']') {
            current = Some(l[1..l.len() - 1].to_string());
        } else if let (Some(name), Some(rest)) = (&current, l.strip_prefix("record")) {
            let value = rest.trim_start_matches(['=', ' ']).trim_matches('"');
            match value {
                v if v.starts_with("ledger") => declared_ledger.push(name.clone()),
                v if v.starts_with("inventory") => declared_inventory.push(name.clone()),
                _ => {}
            }
        }
    }
    let mut warnings = Vec::new();
    for d in &declared_ledger {
        if !EMITTED_LEDGER.contains(&d.as_str()) {
            warnings.push(format!("declared control event `{d}` (record=ledger) has NO emitter in the binary - a dead declaration reads as coverage that does not exist (D0193)"));
        }
    }
    for e in EMITTED_LEDGER {
        if !declared_ledger.iter().any(|d| d == e) {
            warnings.push(format!("the binary emits ledger event `{e}` that control-events.toml does not declare - an uncounted-by-declaration control event, the issue203/205/207 family (D0193)"));
        }
    }
    // Inventory-record events: the named point must exist in the enforcementPoints inventory text.
    let inventory = crate::hardening::hardening(root).unwrap_or_default(); // the lens is the inventory's one authority; a compute failure reads as absent points, which warns rather than passes
    for (event, point_needle) in INVENTORY_POINTS {
        if declared_inventory.iter().any(|d| d == event) && !inventory.contains(point_needle) {
            warnings.push(format!("declared control event `{event}` (record=inventory) names no matching enforcement point (`{point_needle}`) in the hardening lens (D0193/issue203)"));
        }
    }
    let scanned = declared_ledger.len() + declared_inventory.len();
    GuardReport { name: "control-event-coverage", scanned, warnings, violations: Vec::new() }
}

/// Script extensions a hook command may invoke. Deliberately EXCLUDES `.exe` and extensionless
/// binaries: a not-yet-built `target/release/keel.exe` is a legitimate transient state that the hook
/// commands already probe for, whereas a script is committed source that must exist to be referenced.
pub(crate) const HOOK_SCRIPT_EXTS: [&str; 6] = ["py", "sh", "ps1", "js", "mjs", "rb"];

/// WARNING-level: a hook command referencing a script that DOES NOT EXIST (issue093).
///
/// Why this guard exists, and why it is a guard rather than a reminder (D0047): migrating the in-loop
/// gates into the binary (D0134) deleted `.engine/tools/stop_gate.py`, and `.claude/settings.json` got
/// the replacement — but `.claude/settings.local.json` ALSO declared a Stop hook pointing at the
/// deleted script. Claude Code MERGES hooks across settings files, so both fired and the stale one
/// failed on every single turn end. It survived because `settings.local.json` is GITIGNORED: it never
/// appeared in `git status`, never in a diff, and the doc-sync sweep covers the tracked surface only.
/// So the delete-completely discipline had a blind spot exactly where hook wiring lives.
///
/// WARNING, not hard-blocking, and the level is the point: this config is machine-local and partly
/// gitignored, so CI cannot see it and one contributor's personal hook must never block another's
/// commit. A warning still surfaces on every `keel gate guard` — including inside the Stop hook itself,
/// which is what makes a sibling hook's breakage self-reporting rather than something the human has to
/// notice in scrollback.
pub(crate) fn hook_config_integrity(root: &Path) -> GuardReport {
    let mut warnings = Vec::new();
    let mut scanned = 0usize;

    for rel in [".claude/settings.json", ".claude/settings.local.json"] {
        let path = root.join(rel);
        let Ok(text) = keel_model::corpus::read_to_string(&path) else {
            continue; // absent is fine — neither file is required
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
            warnings.push(format!("{rel}: not valid JSON — every hook declared here is silently inert"));
            continue;
        };
        let Some(events) = json.get("hooks").and_then(serde_json::Value::as_object) else {
            continue;
        };
        for (event, groups) in events {
            for group in groups.as_array().into_iter().flatten() {
                for hook in group.get("hooks").and_then(serde_json::Value::as_array).into_iter().flatten() {
                    let Some(cmd) = hook.get("command").and_then(serde_json::Value::as_str) else {
                        continue;
                    };
                    scanned += 1;
                    for tok in cmd.split([' ', '"', '\'', '\t', ';', '|', '&', '(', ')']) {
                        let t = tok.trim();
                        if !Path::new(t).extension().is_some_and(|e| {
                            HOOK_SCRIPT_EXTS.iter().any(|x| e.eq_ignore_ascii_case(x))
                        }) {
                            continue;
                        }
                        // Only repo-relative references are checkable; an absolute path may live on a
                        // machine we are not inspecting, so claiming it is missing would be wrong.
                        if Path::new(t).is_absolute() {
                            continue;
                        }
                        if !root.join(t).exists() {
                            warnings.push(format!(
                                "{rel}: {event} hook references `{t}`, which does not exist — that hook FAILS every time it fires (issue093)"
                            ));
                        }
                    }
                }
            }
        }
    }

    GuardReport { name: "hook-config-integrity", scanned, warnings, violations: Vec::new() }
}

/// HARD: the activation manifest itself must be well-formed (D0138).
///
/// Hard-blocking is safe here because every check is EXACT set-membership against `GUARD_NAMES` and the
/// processes on disk — no heuristic. And it must be hard: a typo in either contract file silently
/// disables a control, which is strictly worse than the control failing loudly. Absence of either file
/// is NOT a violation (the issue090 lesson: a project that never adopted a control has not violated it).
pub(crate) fn activation_manifest(root: &Path) -> GuardReport {
    let act = keel_model::activation::Activation::load(root);
    // issue241: scanned counted only the guard-bearing units, so the guard reported "11 scanned"
    // against a project declaring 23 processes — a scan count that understates its own population is
    // the same class of misreport as the catalogue that denied those 12 existed.
    let scanned = keel_model::activation::declared_processes(root).len().max(act.unit_names().len());
    let mut warnings = Vec::new();
    if act.is_declared() {
        let inactive = act.inactive_processes();
        if !inactive.is_empty() {
            warnings.push(format!(
                "this project has NOT activated: {} — their guards are skipped (visible above, never silent)",
                inactive.join(", ")
            ));
        }
    }
    let mut violations = act.errors.clone();
    violations.extend(act.unknown_guard_refs(&GUARD_NAMES));
    // issue380 / GH#56: a `charteredBy` that names no Decision in THIS project's decisions is a
    // provenance claim the tree cannot back - the silent form of an engine resync writing another
    // project's charter. Loud, so the set is re-chartered rather than read as chartered.
    // GH#89 / issue611: both directories a project can hold a Decision in are read - its own
    // `.engine/decisions/` and the `.engine/reference/decisions/` the resync deploys - so the
    // message names both places it looked.
    if let Some(charter) = keel_model::onboard::chartered_by(root) {
        if !keel_model::onboard::charter_resolves(root, &charter) {
            let n = charter.trim_start_matches('d');
            // issue617: the advice is computed for the tree it is printed in - "restore" only where
            // git holds a version to restore; a first-time adopter is told what it can do instead.
            violations.push(format!(
                "activation.toml: charteredBy = \"{charter}\" does not resolve - no .engine/decisions/{n}-*.sysml and no .engine/reference/decisions/{n}-*.sysml in this project, so the process set is NOT chartered here (issue380/GH#56); {}",
                keel_model::onboard::unresolved_charter_advice(root)
            ));
        }
    }
    GuardReport { name: "activation-manifest", scanned, warnings, violations }
}

// ── step-check-resolves guard (D0321 option A / D0434) ──────────────────────────────────────────

/// The names a `checkedBy` may resolve to.
///
/// Every guard in [`GUARD_NAMES`] plus every rule DECLARED under `.engine/rules/` as
/// `part <name> : EdgeRule|ElementRule`. Shared with `hardening::step_enforcement` so the lens and
/// the guard read one vocabulary.
#[must_use]
pub fn declared_check_names(root: &Path) -> HashSet<String> {
    let mut names: HashSet<String> = GUARD_NAMES.iter().map(std::string::ToString::to_string).collect();
    for path in keel_model::corpus::collect_sysml(&root.join(".engine/rules")) {
        let Ok(text) = keel_model::corpus::read_to_string(&path) else { continue };
        for raw in text.lines() {
            let t = raw.trim_start();
            if t.starts_with("//") {
                continue;
            }
            let Some(rest) = t.strip_prefix("part ") else { continue };
            let Some((name, ty)) = rest.split_once(':') else { continue };
            let ty = ty.trim_start();
            if ty.starts_with("EdgeRule") || ty.starts_with("ElementRule") {
                names.insert(name.trim().to_string());
            }
        }
    }
    // D0435: a per-run check - the phase's `<...><Phase>Gate` result in a run of the process - is
    // named `gate:<phase>`, and resolves against the phases the workflow chains declare.
    for phase in keel_model::orient::workflow_phases(root) {
        names.insert(format!("gate:{phase}"));
    }
    names
}

/// A `ProcessStep` that names its check names one that RUNS.
///
/// # Why a guard and not a comment
///
/// D0321 measured 37 processes of which only agile-workflow's ceremony steps carried a per-step check;
/// seven processes had a guard that checks one of their steps and nowhere to write that fact. D0434
/// gives the step an attribute, `checkedBy : String [0..1]`, and this guard is what makes the attribute a
/// FACT a control can read rather than a comment: a step that names a guard nothing runs - a rename, a
/// retirement, a typo - would otherwise claim enforcement forever, which is EHZ5 (an enforcement point
/// silently dead) in its purest shape. The vocabulary is [`declared_check_names`]: the binary's own
/// `GUARD_NAMES` and the rules `.engine/rules/` declares; an empty string fails too, since a binding that
/// names nothing is a binding to nothing.
///
/// HARD. What it does NOT check: whether the named guard's predicate actually covers the step's
/// `actionText` - that is a judgment, reported per step by `keel show hardening` (`stepEnforcement`) and
/// judged in the sitting review (D0254). An unbound step is not scanned: optional multiplicity is the
/// EXPAND step of the migration, and nothing here contracts.
#[must_use]
pub fn step_check_resolves(root: &Path) -> GuardReport {
    let names = declared_check_names(root);
    let mut scanned = 0usize;
    let mut violations = Vec::new();
    for (path, line, step, name) in step_check_bindings(root) {
        scanned += 1;
        if names.contains(&name) {
            continue;
        }
        let rel = relpath(root, &path);
        let hint = if name.is_empty() {
            "the value is empty - a binding to nothing".to_string()
        } else if let Some(phase) = name.strip_prefix("gate:") {
            nearest_attr(&name, &names).map_or_else(
                || format!("no `first A then B;` under .engine/workflows/ declares a phase `{phase}` (D0435)"),
                |n| format!("did you mean `{n}`?"),
            )
        } else {
            nearest_attr(&name, &names).map_or_else(
                || "no guard in GUARD_NAMES and no `part <name> : EdgeRule|ElementRule` under .engine/rules/ has that name".to_string(),
                |n| format!("did you mean `{n}`?"),
            )
        };
        violations.push(format!(
            "{rel}:{line}: step `{step}` sets checkedBy = \"{name}\" and nothing of that name runs - {hint}. A step claiming a check that does not exist is an enforcement point that is silently dead (EHZ5, D0434): bind it to a live guard or rule, or remove the attribute so `keel show hardening` reports the step as judgment."
        ));
    }
    GuardReport { name: "step-check-resolves", scanned, warnings: Vec::new(), violations }
}

/// The D0388 pair for doc-guard-count's family clause (issue584), chosen before the real catalogue was
/// read: a fixture row naming a guard under a family other than its table's FAILS naming the row and
/// both families; the catalogue as shipped PASSES, and a file without the column claims nothing.
#[cfg(test)]
mod family_cell_tests {
    use super::{family_cell_violations, family_of, guard_family_claims, FAMILIES, GUARD_NAMES};

    const WRONG_FAMILY: &str = "## Hard-blocking\n\n| Guard | Family | What it enforces |\n|---|---|---|\n| `doc-guard-count` | identity | the count has one home |\n| `actors` | identity | registered actors |\n| `charter` |  | work traces to its charter |\n";

    #[test]
    fn a_row_under_another_family_fails_naming_the_row_and_both_families() {
        let v = family_cell_violations("x/guards.md", WRONG_FAMILY);
        assert_eq!(v.len(), 2, "{v:?}");
        assert!(v[0].starts_with("x/guards.md:5: row `doc-guard-count` names family `identity` but `doc-guard-count` is dispatched by family `enforcement`"), "{}", v[0]);
        assert!(v[1].starts_with("x/guards.md:7: row `charter` carries no family cell (``)"), "{}", v[1]);
        assert!(v[1].contains("dispatched by family `sprints`"), "{}", v[1]);
    }

    #[test]
    fn a_file_without_the_column_claims_nothing() {
        let no_column = "| Guard | What it enforces |\n|---|---|\n| `doc-guard-count` | the count has one home |\n";
        assert!(guard_family_claims(no_column).is_empty());
        assert!(family_cell_violations("x/guards.md", no_column).is_empty());
        assert!(family_cell_violations("x/guards.md", "this project enforces 999 forward guards\n").is_empty(), "the cursor.rs fixture stays a count-only red");
    }

    /// THE CONTROL for the column: every enforced guard's row carries the family that dispatches it, and
    /// the guard itself is green on the shipped catalogue.
    #[test]
    fn the_shipped_catalogue_names_every_guards_family() {
        let root = crate::test_repo_root();
        let md = keel_model::corpus::read_to_string(root.join(".engine/docs/guards.md")).expect("guards.md ships with the engine");
        let claims = guard_family_claims(&md);
        assert!(!claims.is_empty(), "guards.md declares the Family column (issue584)");
        let missing: Vec<&str> = GUARD_NAMES.iter().copied().filter(|g| !claims.iter().any(|(_, n, c)| n == g && Some(c.as_str()) == family_of(g))).collect();
        assert!(missing.is_empty(), "guards with no row carrying their dispatching family: {missing:?}");
        assert!(family_cell_violations(".engine/docs/guards.md", &md).is_empty());
        assert!(FAMILIES.iter().all(|f| root.join("members/keel-guards/src").join(format!("{}.rs", f.name)).exists()), "a family's name is its module file");
    }
}

#[cfg(test)]
mod extras_tests {
    use super::extras_violations;

    fn decl() -> Vec<(String, Vec<String>)> {
        vec![("channel".to_string(), vec![".github/workflows/decision-issue.yml".to_string(), ".github/scripts/decide.py".to_string()])]
    }

    /// Both TOML array shapes are read - inline and multi-line - because a one-line declaration used to
    /// scan as zero files, which is a guard passing over the thing it was built to see.
    #[test]
    fn declared_extras_reads_inline_and_multiline_arrays() {
        let root = std::env::temp_dir().join(format!("keel-extras-parse-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".engine").join("contracts")).expect("mkdir");
        std::fs::write(
            root.join(".engine").join("contracts").join("unit-extras.toml"),
            "# header\n[one]\nfiles = [\"a.yml\", \"b.py\"]\nrequires = [\"x\"]\n[two]\nfiles = [\n  \"c.yml\",\n]\n",
        )
        .expect("toml");
        let d = super::declared_extras(&root);
        assert_eq!(d, vec![("one".to_string(), vec!["a.yml".to_string(), "b.py".to_string()]), ("two".to_string(), vec!["c.yml".to_string()])]);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// issue290, the penumbra shape: the unit is installed, two of its mechanism files are absent - two
    /// violations naming unit and path; present -> none; not installed here -> not judged.
    #[test]
    fn an_installed_unit_with_absent_mechanism_fails_and_present_passes() {
        let installed = vec!["channel".to_string()];
        let (scanned, v) = extras_violations(&decl(), &installed, &|_| false);
        assert_eq!((scanned, v.len()), (2, 2), "{v:?}");
        assert!(v[0].contains("channel") && v[0].contains("decision-issue.yml") && v[0].contains("keel process import"), "{}", v[0]);
        let (_, v) = extras_violations(&decl(), &installed, &|_| true);
        assert!(v.is_empty(), "present files are not a finding");
        let (scanned, v) = extras_violations(&decl(), &[], &|_| false);
        assert_eq!((scanned, v.len()), (0, 0), "a unit not installed here is not judged");
    }
}

#[cfg(test)]
mod stpa_currency_tests {
    use super::{analysed_list, currency_warnings};
    use keel_view::view::control_structure::LocalAction;
    use std::collections::HashSet;

    fn set(names: &[&str]) -> HashSet<String> {
        names.iter().map(std::string::ToString::to_string).collect()
    }

    /// A structure of (name, `issued_by`, `acts_on`) rows in structure order.
    fn structure(rows: &[(&str, &'static str, &'static str)]) -> Vec<LocalAction> {
        rows.iter().map(|(n, by, on)| LocalAction { name: (*n).to_string(), issued_by: by, acts_on: on }).collect()
    }

    /// The run record's shape: names up to the first full stop; the FRAME sentence after it is prose.
    #[test]
    fn the_analysed_list_stops_at_the_first_sentence() {
        assert_eq!(analysed_list(" cmdRecord, cmdLand. FRAME: EHZ1-EHZ9; CONSIDERED SAFE: cmdRecord notProvided"), vec!["cmdRecord", "cmdLand"]);
        assert_eq!(analysed_list(" hookStop\";"), vec!["hookStop"]);
    }

    /// A structure that grew a name the runs never walked warns ONCE, naming every missing action.
    #[test]
    fn a_grown_structure_warns_once_and_names_the_gap() {
        let computed = structure(&[
            ("cmdRecord", "agent", "model"),
            ("cmdLand", "agent", "main-ref"),
            ("hookStop", "hooks", "agent-turn"),
            ("workflowCi", "ci", "main-ref"),
        ]);
        let w = currency_warnings(1, &set(&["cmdRecord", "cmdLand"]), &computed);
        assert_eq!(w.len(), 1, "{w:?}");
        assert!(w[0].contains("2 of 4") && w[0].contains("hookStop") && w[0].contains("workflowCi") && !w[0].contains("cmdRecord"), "{}", w[0]);
    }

    /// The remainder is grouped by edge (D0410): a fully open edge shows its count, a partly walked
    /// edge shows `k of n open`, and the `analysed` names are absent from every group.
    #[test]
    fn the_remainder_is_grouped_by_edge() {
        let computed = structure(&[
            ("cmdRecord", "agent", "model"),
            ("cmdAddTask", "agent", "model"),
            ("cmdNew", "agent", "model"),
            ("hookStop", "hooks", "agent-turn"),
            ("hookPreToolUse", "hooks", "agent-turn"),
        ]);
        let w = currency_warnings(1, &set(&["cmdRecord"]), &computed);
        assert_eq!(w.len(), 1, "{w:?}");
        assert!(w[0].contains("agent->model (2 of 3 open): cmdAddTask, cmdNew"), "{}", w[0]);
        assert!(w[0].contains("hooks->agent-turn (2): hookPreToolUse, hookStop"), "{}", w[0]);
        assert!(w[0].contains("TARGET: every controller->process edge walked in full"), "{}", w[0]);
    }

    /// The next tranche is the open edge with the fewest unanalysed actions, and the message carries
    /// the exact `ANALYSED:` list the run record will need - a reader can act without deriving anything.
    #[test]
    fn the_next_tranche_is_the_smallest_open_edge_with_its_analysed_list() {
        let computed = structure(&[
            ("cmdRecord", "agent", "model"),
            ("cmdAddTask", "agent", "model"),
            ("cmdNew", "agent", "model"),
            ("workflowCi", "ci", "main-ref"),
            ("workflowRelease", "ci", "main-ref"),
            ("hookStop", "hooks", "agent-turn"),
            ("hookPreToolUse", "hooks", "agent-turn"),
        ]);
        let w = currency_warnings(1, &set(&["cmdRecord"]), &computed);
        // agent->model has 2 open, ci->main-ref 2, hooks->agent-turn 2: the tie breaks on the edge name
        assert!(w[0].contains("NEXT TRANCHE: agent->model - run the stpa-self process over it and record `ANALYSED: cmdAddTask, cmdNew.`"), "{}", w[0]);
        // walk that edge and the next smallest is named
        let w2 = currency_warnings(2, &set(&["cmdRecord", "cmdAddTask", "cmdNew", "hookStop"]), &computed);
        assert!(w2[0].contains("NEXT TRANCHE: hooks->agent-turn - run the stpa-self process over it and record `ANALYSED: hookPreToolUse.`"), "{}", w2[0]);
        assert!(!w2[0].contains("agent->model"), "a fully walked edge is not listed: {}", w2[0]);
    }

    /// The union over runs is what counts: a second run walking the rest clears the first's gap.
    #[test]
    fn runs_accumulate_and_a_full_walk_is_silent() {
        let computed = structure(&[("cmdRecord", "agent", "model"), ("hookStop", "hooks", "agent-turn")]);
        assert!(currency_warnings(2, &set(&["cmdRecord", "hookStop"]), &computed).is_empty());
    }

    /// No recorded run: the project never adopted the process; nothing is owed and nothing is said.
    #[test]
    fn a_project_with_no_run_hears_nothing() {
        let computed = structure(&[("cmdRecord", "agent", "model")]);
        assert!(currency_warnings(0, &HashSet::new(), &computed).is_empty());
    }
}

#[cfg(test)]
mod engine_written_tests {
    use super::is_engine_written;
    use crate::ChangeRead;
    use std::path::Path;

    struct Repo(std::path::PathBuf);
    impl Drop for Repo {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn git(dir: &Path, args: &[&str]) {
        let ok = keel_git::gitx::git().arg("-C").arg(dir).args(args).output().is_ok_and(|o| o.status.success());
        assert!(ok, "git {args:?}");
    }

    const POLICY: &str = ".engine/contracts/attestation-policy.toml";
    const HEAD_TEXT: &str = "# This file is the policy; `keel guard attestation-authority` reads it.\n[policy]\nmode = \"strict\"\n";

    /// A committed project-owned contract whose comment names a retired verb.
    fn committed() -> Repo {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let repo = Repo(std::env::temp_dir().join(format!("keel-engine-written-{}-{n}", std::process::id())));
        let _ = std::fs::remove_dir_all(&repo.0);
        std::fs::create_dir_all(repo.0.join(".engine/contracts")).expect("mkdir");
        std::fs::write(repo.0.join(POLICY), HEAD_TEXT).expect("write");
        git(&repo.0, &["init", "-q"]);
        git(&repo.0, &["config", "user.email", "p@e.invalid"]);
        git(&repo.0, &["config", "user.name", "probe"]);
        git(&repo.0, &["add", "-A"]);
        git(&repo.0, &["-c", "commit.gpgsign=false", "commit", "-q", "-m", "at 0.4.1"]);
        repo
    }

    /// D0388 pair for the D0523 clause, chosen before the tree was read.
    /// KNOWN-POSITIVE: a locked contract whose working text is exactly the respell of its HEAD text
    /// (`keel guard` -> `keel gate guard`) is engine-written, outside the lock.
    /// KNOWN-NEGATIVE: the respell plus one more edited line is back under the lock; an edit that is
    /// not the respell at all is under it too; the unchanged file is not engine-written (nothing to
    /// exempt).
    #[test]
    fn a_respelled_locked_contract_is_the_engine_arriving_and_one_more_byte_is_not() {
        let repo = committed();
        let read = ChangeRead::WorkingTree;
        let respelled = HEAD_TEXT.replace("`keel guard attestation-authority`", "`keel gate guard attestation-authority`");
        assert_ne!(respelled, HEAD_TEXT);
        std::fs::write(repo.0.join(POLICY), &respelled).expect("write");
        assert!(is_engine_written(&repo.0, POLICY, read), "the respell of HEAD is the engine's fold arriving");

        std::fs::write(repo.0.join(POLICY), format!("{respelled}strict_for = [\"ai\"]\n")).expect("write");
        assert!(!is_engine_written(&repo.0, POLICY, read), "one edited line beyond the fold is a control edited");

        std::fs::write(repo.0.join(POLICY), HEAD_TEXT.replace("strict", "lax")).expect("write");
        assert!(!is_engine_written(&repo.0, POLICY, read), "an edit that is not the respell is under the lock");

        std::fs::write(repo.0.join(POLICY), HEAD_TEXT).expect("write");
        assert!(!is_engine_written(&repo.0, POLICY, read), "unchanged: the respell has something to say and the file does not say it");
    }
}

#[cfg(test)]
mod step_check_resolves_tests {
    use std::path::Path;

    struct Fixture(std::path::PathBuf);
    impl Fixture {
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn fixture(binding: &str) -> Fixture {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let dir = Fixture(std::env::temp_dir().join(format!("keel-stepcheck-{}-{n}", std::process::id())));
        let _ = std::fs::remove_dir_all(dir.path());
        std::fs::create_dir_all(dir.path().join(".engine/processes")).expect("mkdir");
        std::fs::create_dir_all(dir.path().join(".engine/rules")).expect("mkdir");
        std::fs::create_dir_all(dir.path().join(".engine/workflows")).expect("mkdir");
        std::fs::write(
            dir.path().join(".engine/workflows/w.sysml"),
            "package W {\n    action def W {\n        first refine then standup;\n    }\n}\n",
        )
        .expect("write");
        std::fs::write(
            dir.path().join(".engine/rules/rules.sysml"),
            "package Rules {\n    part issuesTriagedRule : EdgeRule { :>> id = \"r\"; }\n}\n",
        )
        .expect("write");
        std::fs::write(
            dir.path().join(".engine/processes/probe.sysml"),
            format!(
                "package ProcessProbe {{\n    action probe : Process {{ :>> purpose = \"p\"; }}\n    action p1 : ProcessStep {{\n        :>> actionText = \"do\";\n        :>> owner = Owner::ai;\n{binding}    }}\n    action p2 : ProcessStep {{\n        :>> actionText = \"judge\";\n        :>> owner = Owner::human;\n    }}\n}}\n"
            ),
        )
        .expect("write");
        dir
    }

    /// Known-positive: a step naming a guard that does not exist FAILS, and the report names the step.
    #[test]
    fn a_ghost_guard_name_fails() {
        let dir = fixture("        :>> checkedBy = \"marker-vocabularyy\";\n");
        let r = super::step_check_resolves(dir.path());
        assert_eq!(r.scanned, 1);
        assert_eq!(r.violations.len(), 1, "{:?}", r.violations);
        assert!(r.violations[0].contains("step `p1`"), "{}", r.violations[0]);
        assert!(r.violations[0].contains("did you mean `marker-vocabulary`?"), "{}", r.violations[0]);
    }

    /// Known-negative: a guard name and a declared rule name both resolve; an unbound step is not scanned.
    #[test]
    fn a_guard_or_a_declared_rule_resolves() {
        let dir = fixture("        :>> checkedBy = \"marker-vocabulary\";\n");
        let r = super::step_check_resolves(dir.path());
        assert_eq!((r.scanned, r.violations.len()), (1, 0), "{:?}", r.violations);
        let dir = fixture("        :>> checkedBy = \"issuesTriagedRule\";\n");
        let r = super::step_check_resolves(dir.path());
        assert_eq!((r.scanned, r.violations.len()), (1, 0), "{:?}", r.violations);
    }

    /// Known-positive / known-negative for the `gate:` class (D0435): a phase the workflow chain
    /// declares resolves; a spelling no chain declares fails naming the phase.
    #[test]
    fn a_gate_binding_resolves_against_the_workflow_chain() {
        let dir = fixture("        :>> checkedBy = \"gate:refine\";\n");
        let r = super::step_check_resolves(dir.path());
        assert_eq!((r.scanned, r.violations.len()), (1, 0), "{:?}", r.violations);
        let dir = fixture("        :>> checkedBy = \"gate:closeout\";\n");
        let r = super::step_check_resolves(dir.path());
        assert_eq!(r.violations.len(), 1, "{:?}", r.violations);
        assert!(r.violations[0].contains("phase `closeout`") || r.violations[0].contains("did you mean"), "{}", r.violations[0]);
    }

    /// An empty binding is a binding to nothing.
    #[test]
    fn an_empty_binding_fails() {
        let dir = fixture("        :>> checkedBy = \"\";\n");
        let r = super::step_check_resolves(dir.path());
        assert_eq!(r.violations.len(), 1, "{:?}", r.violations);
        assert!(r.violations[0].contains("empty"), "{}", r.violations[0]);
    }

    /// The live tree: the seven D0434 guard bindings and the six D0435 gate bindings resolve.
    #[test]
    fn the_live_bindings_resolve() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let r = super::step_check_resolves(&root);
        assert!(r.violations.is_empty(), "{:?}", r.violations);
        assert!(r.scanned >= 13, "expected the 7 D0434 + 6 D0435 bindings, scanned {}", r.scanned);
    }
}
