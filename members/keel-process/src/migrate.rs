//! `keel migrate` — bring a DOWNSTREAM project's on-disk `.engine/` and `.tracking/` up to the
//! vintage of THIS binary (dcProjectMigrationTool).
//!
//! A downstream `.engine/` is a COPY taken at `keel init` time, so it diverges from the binary the
//! moment the engine changes. That divergence has already blocked two projects outright (issue089,
//! issue090), and both were repaired by making the binary tolerant — which works for a vocabulary
//! set and does NOT work for a schema conversion: no amount of tolerance retypes a project's
//! `part def Process` into an `action def`.
//!
//! # There is no vintage stamp, and there should not be one
//!
//! `keel init` writes no version marker (main.rs `cmd_init`), so the vintage cannot be READ. It is
//! DETECTED, and the detection is the same code that does the work: every step is a pure
//! `plan(root) -> StepPlan` over the tree's actual content. A step whose pattern no longer matches
//! plans zero edits. That single property delivers three of the required guarantees at once —
//! vintage detection, idempotency (a second run is a no-op because the pattern is gone), and dry-run
//! fidelity (the dry run and the apply call the SAME function, so the totals cannot disagree).
//!
//! A stored stamp would be strictly worse: it can drift from the tree it claims to describe, it is
//! absent from every project inited before it existed, and it would let a step run twice.
//!
//! # What a migration may and may not do (D0067)
//!
//! It rewrites authored facts only where the transform is MECHANICAL and TOTAL. Where a change
//! removed a type outright there is no target to rewrite to, so the migration REFUSES: it reports
//! every affected item with `file:line` and exits non-zero rather than guessing a replacement or
//! leaving a partial migration behind. Blockers abort the whole run — never some steps.

use include_dir::Dir;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

// ── engine-scaffold path rules (shared with `keel init`) ─────────────────────

/// Remap an embedded engine-relative path for scaffolding: `decisions/*` -> `reference/decisions/*`
/// (read-only reference, not instance — D0093 boundary); everything else is unchanged.
#[must_use]
pub fn remap_engine_path(rel: &Path) -> PathBuf {
    rel.strip_prefix("decisions")
        .map_or_else(|_| rel.to_path_buf(), |rest| Path::new("reference").join("decisions").join(rest))
}

/// Content transform paired with [`remap_engine_path`] (issue291): a decision file copied into
/// `reference/decisions/` gets its PACKAGE renamed `DecisionNNNN` -> `ReferenceDecisionNNNN`.
///
/// # Why the copy must be renamed
///
/// The reference copy is the ENGINE's governance history, shipped read-only so a downstream reader
/// can see why the engine is the way it is. It is still PARSED (`collect_sysml` has no exclusions),
/// so without this rename a fresh project ships 236 `package DecisionNNNN` declarations while its own
/// `.engine/decisions/` is empty — and `next_decision_number` scans only the project's directory. So
/// the project's FIRST recorded decision allocates `Decision0001`/`d0001` and collides with
/// `reference/decisions/0001-text-files-are-truth.sysml`: the registry silently merges same-named
/// packages, `validate` and `check-engine` both report clean, and only `duplicate-identity` catches
/// it — naming the read-only reference file as the offender, which is the one file the author must
/// not edit. Hit live in a field project (issue291), where the workaround was to renumber the
/// project's decisions into a 1xxx series.
///
/// # Why the PACKAGE and not the number
///
/// Numbering past the highest reference decision (the other option the `DoD` allowed) fixes the first
/// allocation and reopens the hole at the next `keel migrate`: the project takes 0237, the engine
/// later reaches 0237, resync copies it in, and the two collide again. Renaming the package makes the
/// two namespaces DISJOINT BY PREFIX, so number overlap stops mattering permanently — verified with
/// a project `Decision0001` and a `ReferenceDecision0001` coexisting green.
///
/// The rename is one line per file, always at identifier position, so it cannot touch prose: 189 of
/// the 514 `dNNNN` occurrences in the decision corpus sit inside `procedureText` strings, which is
/// why the part names are left alone. `dNNNN` stays resolvable because name resolution is global —
/// the 10 `#JustifiedBy` edges in `.engine/rules/rules.sysml` still resolve after the rename.
///
/// Returns `None` when the file is not a remapped decision or declares no such package, so
/// `step_engine_resync` compares TRANSFORMED content against disk and stays idempotent: a project
/// already holding `ReferenceDecisionNNNN` plans zero edits, and an older one plans exactly the
/// rename.
#[must_use]
pub fn remap_engine_content(rel: &Path, contents: &str) -> Option<String> {
    if !rel.starts_with("decisions") {
        return None;
    }
    let mut out = String::with_capacity(contents.len() + 9);
    let mut renamed = false;
    for line in contents.split_inclusive('\n') {
        if !renamed {
            if let Some(rest) = line.strip_prefix("package Decision") {
                let digits = rest.chars().take_while(char::is_ascii_digit).count();
                if digits == 4 {
                    out.push_str("package ReferenceDecision");
                    out.push_str(rest);
                    renamed = true;
                    continue;
                }
            }
        }
        out.push_str(line);
    }
    renamed.then_some(out)
}

/// Contracts under `.engine/contracts/` that are the PROJECT'S state, not the engine's content.
///
/// The engine ships a default for each so a fresh `keel init` has one; from then on the file is
/// written by the project's own commands and an engine resync must never overwrite it. Determined
/// by the rule "a keel COMMAND writes this file": `activate`/`deactivate` write activation,
/// `init`/`migrate` write the pin, `import`/`publish` write the install record, `init` writes the
/// adoption profile, the decision channel reads the actor map, and the parser baseline is a ratchet
/// over the project's OWN corpus.
///
/// issue314, found by the federation suite: `keel migrate` reverted a project's deactivation of
/// `render` — silently re-arming a control the project had turned off, and equally able to disarm
/// one it had on.
fn is_project_owned_contract(mapped: &Path) -> bool {
    const PROJECT_OWNED: [&str; 7] = [
        "activation.toml",
        // issue376 / GH#57: the grants in the attestation policy (standing consent, recording
        // delegation, the human's standing words) are the project's human's - a resync must not
        // replace them with the engine's, which is another project's human's.
        "attestation-policy.toml",
        "adoption-profile.toml",
        "engine-version.toml",
        "installed-units.toml",
        "github-actors.toml",
        "parser-coverage-baseline.toml",
    ];
    mapped.parent().is_some_and(|p| p.ends_with("contracts"))
        && mapped.file_name().and_then(|f| f.to_str()).is_some_and(|f| PROJECT_OWNED.contains(&f))
}

/// Project-owned contracts whose body is a COUNT over the project's own corpus (GH#90).
///
/// A shipped default is a starting state for a policy, but for a ratchet it is a claim about a
/// corpus the engine has not seen. These are added by the project when it adopts the ratchet, at
/// the count it measures, with its reason - never seeded by a resync.
fn is_ratchet_over_project_corpus(mapped: &Path) -> bool {
    mapped.parent().is_some_and(|p| p.ends_with("contracts"))
        && mapped.file_name().and_then(|f| f.to_str()) == Some("parser-coverage-baseline.toml")
}

/// `declaredAt = "<date>"` rewritten to `today`; every other line untouched.
fn stamp_declared_at(text: &str, today: &str) -> String {
    let mut out: Vec<String> = text
        .lines()
        .map(|l| if l.trim_start().starts_with("declaredAt") { format!("declaredAt = \"{today}\"") } else { l.to_string() })
        .collect();
    if text.ends_with('\n') {
        out.push(String::new());
    }
    out.join("\n")
}

#[cfg(test)]
mod stamp_tests {
    use super::stamp_declared_at;

    /// D0319: a profile the migration ADDS is dated by the migration, not by the engine's own history.
    #[test]
    fn an_added_adoption_profile_is_declared_today() {
        let shipped = "# profile\nprofile = \"guided\"\ndeclaredAt = \"2026-08-21\"\n";
        assert_eq!(stamp_declared_at(shipped, "2026-09-05"), "# profile\nprofile = \"guided\"\ndeclaredAt = \"2026-09-05\"\n");
    }
}

// The engine-resync text and the portable-tool predicates descended to the schema member beside the
// embedded tree they read (sprint 733); re-exported so `crate::migrate::` paths keep resolving.
pub use keel_schema::embedded::{is_engine_dev_only, is_portable_engine_tool, resync_text};
use keel_schema::embedded::{is_sectioned_contract, merge_project_sections, project_sections};

// ── plan types ───────────────────────────────────────────────────────────────

/// One file this step would rewrite, with its full post-transform content.
///
/// Holding the RESULT (not a recipe) is what makes the dry run honest: `--dry-run` and the apply
/// path both call `plan`, so what is reported is exactly what would be written.
pub struct FileEdit {
    pub path: PathBuf,
    pub new_content: String,
    /// Number of individual transformations inside this file — the step's control total.
    pub edits: usize,
    /// One human-readable line per transformation, `line N: before -> after`.
    pub detail: Vec<String>,
}

/// Something the migration cannot transform mechanically. Blockers abort the run.
pub struct Blocker {
    pub path: PathBuf,
    pub line: usize,
    pub reason: String,
    pub advice: String,
}

pub struct StepPlan {
    pub id: &'static str,
    pub title: &'static str,
    pub files: Vec<FileEdit>,
    pub blockers: Vec<Blocker>,
    /// Observations that are neither a change nor a blocker (e.g. unrecognised engine files).
    pub notes: Vec<String>,
}

impl StepPlan {
    const fn empty(id: &'static str, title: &'static str) -> Self {
        Self { id, title, files: Vec::new(), blockers: Vec::new(), notes: Vec::new() }
    }
    #[must_use]
    pub fn edits(&self) -> usize {
        self.files.iter().map(|f| f.edits).sum()
    }
    #[must_use]
    pub const fn is_noop(&self) -> bool {
        self.files.is_empty() && self.blockers.is_empty()
    }
}

pub struct MigrationPlan {
    pub steps: Vec<StepPlan>,
}

impl MigrationPlan {
    #[must_use]
    pub fn blockers(&self) -> usize {
        self.steps.iter().map(|s| s.blockers.len()).sum()
    }
    #[must_use]
    pub fn edits(&self) -> usize {
        self.steps.iter().map(StepPlan::edits).sum()
    }
    #[must_use]
    pub fn files(&self) -> usize {
        self.steps.iter().map(|s| s.files.len()).sum()
    }
    /// The steps that would actually do something — this project's DETECTED vintage, expressed as
    /// the distance from current rather than as a version number nothing stamps.
    #[must_use]
    pub fn active(&self) -> Vec<&StepPlan> {
        self.steps.iter().filter(|s| !s.is_noop()).collect()
    }
}

// ── the removal set (D0142 leanness sweep) ───────────────────────────────────

/// Types the engine REMOVED. There is no mechanical target for an instance of one of these, so each
/// occurrence is a blocker with advice, never a rewrite.
///
/// D0142 removed these as dead schema under its own explicit rule — "re-add it THEN, with an
/// instance" — which is exactly the advice a downstream project needs, because dead HERE does not
/// mean dead THERE.
const REMOVED_TYPES: &[(&str, &str)] = &[
    ("Assumption", "assumptions are COMPUTED now — run `keel assumptions` (issue105). If you author them as standing facts, re-add the def in your own `.engine/schema/`."),
    ("Risk", "removed as dead schema (D0142). Re-add `part def Risk :> Element` in your own `.engine/schema/` if your project keeps a risk register."),
    ("Mitigation", "removed as dead schema (D0142). Re-add it alongside your own `Risk` def."),
    ("Task", "removed as dead schema (D0142). Work items are `Story`; delivery steps are native `action` members of a delivery `action def`."),
    ("Epic", "removed as dead schema (D0142). Aggregate progress is computed from children — group with typed edges rather than an Epic item."),
    ("Role", "removed as dead schema (D0142). Re-add in your own `.engine/schema/` with an instance if you model roles."),
    ("DesignElement", "removed as dead schema (D0142). Use `Component`, or re-add the def in your own `.engine/schema/`."),
    ("ComponentRequirement", "removed as dead schema (D0142). Use `SystemRequirement`, or re-add the def in your own `.engine/schema/`."),
    ("Agent", "removed as dead schema (D0142). An AI actor is `Actor` with `kind = ActorKind::ai` (D0129)."),
    ("WorkflowDefinition", "removed as dead schema (D0142). Workflows are `action def`s with native `first..then` succession — see `.engine/workflows/`."),
    ("InterfacePort", "removed as dead schema (D0142). Use a native `port def` of your own."),
    ("ICD", "an ICD is a computed VIEW, never an authored type (`.engine/schema/core/architecture.sysml`)."),
];

/// Enum types the engine REMOVED. Detected by their `Name::` value form rather than by `: Name`,
/// because an enum is referenced where a VALUE is assigned, not where a type is declared.
///
/// This step exists because of who the removal's consumer is (D0144). `RiskLevel` lived in
/// `element.sysml`, which the engine SHIPS and `engine-resync` overwrites — so a downstream project
/// assigning `RiskLevel::high` would break the moment it migrated. `RiskStatus` is deliberately
/// absent from this list for the opposite reason: it lived in `risk.sysml`, which the engine no
/// longer ships at all, and resync never deletes a file it does not ship — so a project that has it
/// simply keeps it, and blocking on it would refuse a migration over a type the project still owns.
///
/// The distinction is the whole point of the rule that a conversion sprint writes its own migration
/// step while it still knows what it changed: from the outside, both look like "an enum was deleted".
const REMOVED_ENUMS: &[(&str, &str)] = &[
    ("RiskLevel", "removed as dead schema (D0145) — its only consumers were the deleted `Risk`'s likelihood/impact/residual attributes. It lived in `element.sysml`, which this binary OVERWRITES on resync, so re-declare it in your own `.engine/schema/` if you use it."),
];

// ── line-level transforms ────────────────────────────────────────────────────

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Does `line` reference `: <type_name>` as a whole word (an instance's type, not a substring)?
fn types_as(line: &str, type_name: &str) -> bool {
    let mut from = 0;
    while let Some(hit) = line[from..].find(type_name) {
        let at = from + hit;
        let before_ok = line[..at].trim_end().ends_with(':') || line[..at].trim_end().ends_with(":>");
        let after_ok = line[at + type_name.len()..].chars().next().is_none_or(|c| !is_ident_char(c));
        // Guard the ident boundary on the left too: `MyProcess` must not match `Process`.
        let left_ok = at == 0 || !is_ident_char(line[..at].chars().next_back().unwrap_or(' '));
        if before_ok && after_ok && left_ok {
            return true;
        }
        from = at + type_name.len();
    }
    false
}

/// Replace the leading declaration keyword on `line` (`part` -> `action`, etc.).
///
/// Refuses on `part def X` — a DEFINITION is engine content, resynced from the binary, never
/// rewritten in place.
fn retype_keyword(line: &str, from_kw: &str, to_kw: &str) -> Option<String> {
    let indent_len = line.len() - line.trim_start().len();
    let (indent, rest) = line.split_at(indent_len);
    let after = rest.strip_prefix(from_kw)?;
    if !after.starts_with(char::is_whitespace) {
        return None;
    }
    if after.trim_start().starts_with("def ") {
        return None; // a def, not an instance
    }
    Some(format!("{indent}{to_kw}{after}"))
}

/// `part x : T` -> `<to_kw> x : T`, for every instance of `type_name` in `content`.
fn retype_instances(content: &str, type_name: &str, from_kw: &str, to_kw: &str) -> (String, usize, Vec<String>) {
    let mut out = String::with_capacity(content.len());
    let mut edits = 0;
    let mut detail = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let new = if types_as(line, type_name) { retype_keyword(line, from_kw, to_kw) } else { None };
        match new {
            Some(n) => {
                detail.push(format!("line {}: {} -> {}", i + 1, clip(line, 72), clip(&n, 72)));
                edits += 1;
                out.push_str(&n);
            }
            None => out.push_str(line),
        }
        out.push('\n');
    }
    (out, edits, detail)
}

/// Strip every `:>> order = N;` inside a `ProcessStep` instance body.
///
/// `ProcessStep.order` was deleted when Process became behaviour: sequence is now native
/// `first <step> then <step>` succession, so a surviving `order` attribute is a second, silent
/// source of truth for the same fact. The successions themselves are NOT synthesised here — that
/// would be fabricating an authored fact from a guess about intent. The migration removes the dead
/// attribute and the `process-succession` guard then reports the process as unordered, which is the
/// true state until a human writes the succession.
fn drop_processstep_order(content: &str) -> (String, usize, Vec<String>) {
    let mut out = String::with_capacity(content.len());
    let mut edits = 0;
    let mut detail = Vec::new();
    let mut depth: i32 = 0;
    let mut in_step_from: Option<i32> = None;
    for (i, line) in content.lines().enumerate() {
        let entering = types_as(line, "ProcessStep")
            && (line.trim_start().starts_with("part ") || line.trim_start().starts_with("action "));
        if entering && in_step_from.is_none() {
            in_step_from = Some(depth);
        }
        let mut emit = line.to_string();
        if in_step_from.is_some() {
            if let Some(stripped) = strip_order_assignment(line) {
                detail.push(format!("line {}: dropped `:>> order = ...` ({})", i + 1, clip(line, 72)));
                edits += 1;
                if stripped.trim().is_empty() {
                    // The whole line was the assignment — drop the line, and its brace delta with it.
                    depth += brace_delta(line);
                    if in_step_from.is_some_and(|d| depth <= d) {
                        in_step_from = None;
                    }
                    continue;
                }
                emit = stripped;
            }
        }
        out.push_str(&emit);
        out.push('\n');
        depth += brace_delta(line);
        if in_step_from.is_some_and(|d| depth <= d) {
            in_step_from = None;
        }
    }
    (out, edits, detail)
}

fn brace_delta(line: &str) -> i32 {
    let open = i32::try_from(line.matches('{').count()).unwrap_or(0);
    let close = i32::try_from(line.matches('}').count()).unwrap_or(0);
    open - close
}

/// Remove a `:>> order = <value>;` assignment from `line`, returning the remainder if one was found.
fn strip_order_assignment(line: &str) -> Option<String> {
    let at = line.find(":>> order")?;
    // Confirm it is `order` as a whole attribute name, then take through the terminating `;`.
    let after_name = &line[at + ":>> order".len()..];
    if after_name.chars().next().is_some_and(is_ident_char) {
        return None;
    }
    let end = after_name.find(';')? + at + ":>> order".len() + 1;
    let mut kept = String::with_capacity(line.len());
    kept.push_str(&line[..at]);
    kept.push_str(&line[end..]);
    // Collapse the double space a mid-line removal leaves behind.
    if kept.contains("  ") && !kept.trim_start().is_empty() {
        let indent_len = kept.len() - kept.trim_start().len();
        let (indent, rest) = kept.split_at(indent_len);
        kept = format!("{indent}{}", rest.replace("  ", " "));
    }
    Some(kept.trim_end().to_string())
}


/// Display a path relative to the project root, with `/` separators.
///
/// An absolute path here is noise: every line of the report starts with the same 120 characters of
/// temp-dir prefix, which buries the part that identifies the file.
fn rel(root: &Path, p: &Path) -> String {
    p.strip_prefix(root).unwrap_or(p).to_string_lossy().replace('\\', "/")
}

/// Trim a quoted source line so one long single-line item body cannot swamp the report.
fn clip(s: &str, max: usize) -> String {
    let s = s.trim();
    if s.chars().count() <= max {
        return s.to_string();
    }
    let head: String = s.chars().take(max).collect();
    format!("{head}...")
}

// ── steps ────────────────────────────────────────────────────────────────────

/// Every `.sysml` file whose content this project AUTHORED: all of `.tracking/`, plus the project's
/// own `.engine/decisions/` (D0093 — a downstream project authors its decisions there; the engine's
/// ship read-only under `.engine/reference/decisions/` and are resynced, never rewritten).
fn authored_files(root: &Path) -> Vec<PathBuf> {
    let mut v = keel_model::corpus::collect_sysml(&root.join(".tracking"));
    v.extend(keel_model::corpus::collect_sysml(&root.join(".engine").join("decisions")));
    v.sort();
    v
}

/// The in-flight content of the tree, so that steps COMPOSE.
///
/// Steps are independent transforms that can land on the same file — a `ProcessStep` instance needs
/// both the `part` -> `action` retype and the `order` drop. If each step read the file from disk,
/// each would produce a full-file result computed from the ORIGINAL, and applying them in sequence
/// would leave only the last one's work: the earlier steps' edits would be silently overwritten.
/// (The post-apply re-plan catches that, so it fails loudly rather than corrupting — but it would
/// fail on every multi-step file.) Each step therefore reads through this overlay and writes back
/// into it, so a later step builds on the earlier one and the final write is cumulative.
struct Working {
    overlay: std::collections::BTreeMap<PathBuf, String>,
}

impl Working {
    const fn new() -> Self {
        Self { overlay: std::collections::BTreeMap::new() }
    }
    fn read(&self, path: &Path) -> Option<String> {
        self.overlay.get(path).cloned().or_else(|| std::fs::read_to_string(path).ok())
    }
    fn stage(&mut self, path: &Path, content: String) {
        self.overlay.insert(path.to_path_buf(), content);
    }
}

/// Step 1b — the resync's own record, written into the change the resync makes.
///
/// TWO CONTROLS THAT COULD NOT BOTH BE SATISFIED DOWNSTREAM. D0067 requires a bulk transform that
/// crosses ownership boundaries to carry a COMMITTED transform as its record, and the `ownership`
/// guard suspends D0108 on exactly that evidence: a file under `.engine/tools/migrations/` in the
/// same change. A project-migration (D0275) is such a transform by construction — `.engine/` carries
/// items five different upstream actors created, so no running actor can ever own them all, and the
/// resync rewrites their fields in place. But `is_engine_dev_only` excludes everything under
/// `tools/` from shipping, so the one path that lifts D0108 was the one path a migration could not
/// write. `keel migrate` therefore failed its own post-migrate gate in every project it exists to
/// serve, and reverted; the ownership violations named the engine's upstream authors, who are not on
/// any downstream roll and cannot be put there for five actors at once.
///
/// The record is GENERATED here rather than shipped from the engine tree, which is what keeps the
/// exemption honest: it names the build the tree is moving to and the control totals of the step it
/// describes, so the reviewer reading the commit sees what crossed the boundary and why. It is not a
/// flag a caller can set, and it appears only when something actually crossed — a no-op resync
/// writes nothing.
fn step_resync_record(root: &Path, resync: &StepPlan) -> StepPlan {
    let mut plan = StepPlan::empty(
        "resync-record",
        "Record the engine resync as the co-committed transform its own gate reads (D0067/D0108)",
    );
    if resync.files.is_empty() {
        return plan; // nothing crossed an ownership boundary, so there is nothing to account for
    }
    let date = keel_write::scaffold::today();
    let build = env!("KEEL_BUILD_COMMIT");
    let version = env!("CARGO_PKG_VERSION");
    let dst = root.join(".engine").join("tools").join("migrations").join(format!("{date}-engine-resync-{build}.md"));
    if dst.exists() {
        return plan; // this build already recorded a resync today — re-running is not a second migration
    }
    let engine_dir = root.join(".engine");
    let listed = resync
        .files
        .iter()
        .map(|f| {
            let shown = f.path.strip_prefix(&engine_dir).unwrap_or(&f.path);
            format!("- .engine/{}", shown.display().to_string().replace('\\', "/"))
        })
        .collect::<Vec<_>>()
        .join("\n");
    let body = format!(
        "# Engine resync -> keel {version} (build {build})\n\
         \n\
         Written by `keel migrate` on {date}. This file IS the transform record for the resync\n\
         committed beside it, in the sense D0067 means: a bulk change that crosses ownership\n\
         boundaries has to carry, in its own commit, a readable account of what it touched. The\n\
         `ownership` guard reads this directory to suspend D0108 for that commit.\n\
         \n\
         .engine/ is a copy of the engine tree, and the items in it name the upstream actors who\n\
         authored them. A resync rewrites those items in place; no actor running the migration owns\n\
         them all. That is the boundary this record accounts for.\n\
         \n\
         ## Control totals\n\
         \n\
         - files written: {}\n\
         - edits: {}\n\
         \n\
         ## Files\n\
         \n\
         {listed}\n",
        resync.files.len(),
        resync.edits(),
    );
    plan.files.push(FileEdit {
        path: dst,
        new_content: body,
        edits: 1,
        detail: vec![format!("add tools/migrations/{date}-engine-resync-{build}.md ({} file(s) accounted for)", resync.files.len())],
    });
    plan
}

/// Step 1 — resync `.engine/` from the engine embedded in this binary.
///
/// For everything the engine OWNS this is not a transform at all: the binary carries the current
/// tree (`include_dir!`), so bringing a project current means writing it out. Only files whose
/// content differs are listed, which is what makes the step both a vintage probe and a no-op when
/// the project is already current.
///
/// PRESERVED, never written: `.engine/decisions/` (the project's own), `deliverable-manifest.txt`
/// (instance-specific), and any file the project added that the engine does not ship.
fn step_engine_resync(root: &Path, engine: &Dir) -> StepPlan {
    let mut plan = StepPlan::empty("engine-resync", "Resync .engine/ from the engine embedded in this binary");
    let dst_engine = root.join(".engine");
    if !dst_engine.is_dir() {
        plan.notes.push(format!("no .engine/ at {} — is this a keel project?", root.display()));
        return plan;
    }
    let mut shipped: BTreeSet<PathBuf> = BTreeSet::new();
    collect_embedded(engine, &mut |f| {
        let rel = f.path();
        if is_engine_dev_only(rel) {
            return;
        }
        let mapped = remap_engine_path(rel);
        shipped.insert(mapped.clone());
        if mapped == Path::new("deliverable-manifest.txt") {
            return; // instance-specific — a project's own manifest is never overwritten
        }
        let dst = dst_engine.join(&mapped);
        // PROJECT-OWNED CONTRACTS ARE ADDED, NEVER OVERWRITTEN (issue314). The engine ships a
        // default so a fresh `init` has one; after that the file is the PROJECT'S state, written by
        // the project's own commands. Resyncing it reverts choices nobody revisited: a project that
        // deactivated a process had it silently switched back ON by `keel migrate` — demonstrated,
        // not hypothetical — and the same path can silently switch a control OFF, which is the
        // unsigned control-weakening the keystone lock exists to prevent.
        if is_project_owned_contract(&mapped) && dst.exists() {
            return;
        }
        // AND A RATCHET OVER THE PROJECT'S OWN CORPUS IS NEVER SEEDED BY THE RESYNC (GH#90). The
        // check above stops the resync OVERWRITING a project-owned contract; it does not stop it
        // ADDING one. For every other project-owned contract the shipped default is a sane starting
        // state, but `parser-coverage-baseline.toml` carries a COUNT, and the engine's count is the
        // engine's own legacy census (7 successions in its workflow files). Written into a
        // first-time adopter it becomes that project's ratchet while describing a corpus that is not
        // theirs: the guard totals `.tracking/` and `.engine/` together, so an adopter holding even
        // one skipped statement of its own goes red at `1 + 7 > 7` the instant migrate lands the
        // file - and red inside the run, so migrate reverts its own output with no action the
        // adopter can take. Absence is already a stated, passing state (D0136), and the contract's
        // own comment asks that the number be adopted deliberately with its reason recorded - which
        // is a thing only the project can do, at the count the project actually has.
        if is_ratchet_over_project_corpus(&mapped) && !dst.exists() {
            return;
        }
        let Ok(shipped_text) = std::str::from_utf8(f.contents()) else { return };
        // issue291: compare the TRANSFORMED content, so this step IS the migration for a project
        // inited before the rename existed - and a no-op for one inited after it.
        let new_content: &str = &remap_engine_content(rel, shipped_text)
            .unwrap_or_else(|| shipped_text.to_owned());
        let current = std::fs::read_to_string(&dst).ok();
        // SECTIONED CONTRACTS MERGE, NEVER OVERWRITE (issue349, GH#44). `unit-extras.toml` is
        // engine-shipped but a project PUBLISHING its own unit adds a `[unit]` section of its own;
        // the resync rewrote the file wholesale and a published unit's declared payload vanished
        // with nothing reported. A section the engine's copy does not carry is the project's and
        // travels through; a section BOTH carry with different bodies is a conflict this step cannot
        // decide, so it BLOCKS naming the section (the issue314 shape: refuse, never hand-repair).
        let merged: Option<String> = if is_sectioned_contract(&mapped) {
            current.as_deref().and_then(|cur| match merge_project_sections(new_content, cur) {
                Ok(m) => Some(m),
                Err(conflicts) => {
                    for sec in conflicts {
                        plan.blockers.push(Blocker {
                            path: dst.clone(),
                            line: 0,
                            reason: format!("section [{sec}] differs between the engine's copy and this project's, and both claim it"),
                            advice: format!("a section the engine ships is the engine's; if [{sec}] is YOUR unit, rename it (or its id) so the two do not collide, then re-run"),
                        });
                    }
                    None
                }
            })
        } else {
            None
        };
        // A project that never had `adoption-profile.toml` adopts the rules TODAY (D0319, issue352):
        // the engine's copy carries this repository's own declaredAt, and shipping that date would
        // retro-fail the project's events on a rule it never had. The migration date is the adoption.
        let stamped: Option<String> = (mapped == Path::new("contracts/adoption-profile.toml") && current.is_none())
            .then(|| stamp_declared_at(new_content, &keel_write::scaffold::today()));
        let new_content: &str = stamped.as_deref().or(merged.as_deref()).unwrap_or(new_content);
        if current.as_deref() == Some(new_content) {
            return;
        }
        let verb = if current.is_some() { "update" } else { "add" };
        let mut detail = vec![format!("{verb} {}", mapped.display())];
        if merged.is_some() {
            let kept = project_sections(new_content).len().saturating_sub(project_sections(&remap_engine_content(rel, shipped_text).unwrap_or_else(|| shipped_text.to_owned())).len());
            if kept > 0 {
                detail.push(format!("{kept} project-authored section(s) PRESERVED through the resync (issue349)"));
            }
        }
        plan.files.push(FileEdit { path: dst, new_content: new_content.to_string(), edits: 1, detail });
    });

    // Files under .engine/ that this binary does not ship. Never deleted — deleting a project's
    // content is precisely what "never fabricate, never clobber" forbids, and there is no way to
    // tell a project's own addition from an engine file left over by a rename. Reported instead,
    // because a stale schema file from an older vintage duplicate-defines and IS worth a look.
    let mut unknown = Vec::new();
    for f in keel_model::corpus::collect_sysml(&dst_engine) {
        let Ok(rel) = f.strip_prefix(&dst_engine) else { continue };
        if rel.starts_with("decisions") || shipped.contains(rel) {
            continue;
        }
        unknown.push(rel.to_path_buf());
    }
    if !unknown.is_empty() {
        plan.notes.push(format!(
            "{} .engine/ file(s) this binary does not ship — PRESERVED, review manually (a leftover from an older vintage duplicate-defines; your own addition is fine):",
            unknown.len()
        ));
        for u in unknown.iter().take(20) {
            plan.notes.push(format!("    .engine/{}", u.display()));
        }
    }
    plan
}

/// Step 1c — the project's OWN living docs are respelled to the verbs this binary dispatches
/// (D0523, issue622).
///
/// Five verb folds each shipped a committed transform over THIS repository's call sites and a
/// stricter `cli-reference` guard; nothing carried the fold into a downstream project's own text. So
/// the downloaded 0.5.0, run over a tree `keel init` 0.4.1 had scaffolded, applied its resync and
/// then failed its own gate on sixteen references in files the run never writes - the adopter's
/// CLAUDE.md as the 0.4.1 template wrote it (`keel orient`, `keel add-task`, `keel report`) and the
/// comments of the two project-owned contracts `is_project_owned_contract` rightly keeps from the
/// resync - and rolled itself back, on words the engine itself had written at 0.4.1.
///
/// THE POPULATION IS THE GUARD'S, MINUS EVERY FILE THE RESYNC IS THE AUTHORITY FOR. `living_doc_files`
/// is the set the guard reads (CLAUDE.md and `.engine/{processes,skills,docs,contracts,workflows,
/// rules}`); a path the embedded engine ships and the resync may write is at its shipped content
/// after this plan and is not read here - not only the files the resync stages THIS run: a shipped
/// file at its shipped bytes is planned by neither step, and one this step rewrote would drift from
/// them for the re-plan's resync to revert, forever (the first run of this step did exactly that on
/// the engine's own `verb-renames.toml` comment). What remains is the project's: CLAUDE.md, a
/// project-owned contract the resync never overwrites, a file it added under `.engine/`. In each,
/// exactly the references the guard would fail AND a spelling exists for are rewritten
/// (`respell_cli_references`: code spans and command-shaped phrases, never a prose mention), so the
/// step is green precisely where the guard would be; a reference with no spelling stays as written
/// and the guard still names it. Idempotent: a rewritten reference names a verb the binary
/// dispatches, so a second run plans nothing.
fn step_verb_respell(root: &Path, engine: &Dir, w: &mut Working) -> StepPlan {
    let mut plan = StepPlan::empty("verb-respell", "Respell retired `keel <verb>` references in the project's own living docs to the spelling this binary dispatches");
    let norm = |p: &Path| p.to_string_lossy().replace('\\', "/");
    let dst_engine = root.join(".engine");
    // The resync's own ownership predicate, path for path (`step_engine_resync`).
    let mut engine_owned: BTreeSet<String> = BTreeSet::new();
    collect_embedded(engine, &mut |f| {
        let rel = f.path();
        if is_engine_dev_only(rel) {
            return;
        }
        let mapped = remap_engine_path(rel);
        let dst = dst_engine.join(&mapped);
        if is_project_owned_contract(&mapped) && dst.exists() {
            return;
        }
        engine_owned.insert(norm(&dst));
    });
    for path in keel_guards::living_doc_files(root) {
        if engine_owned.contains(&norm(&path)) {
            continue;
        }
        let Some(text) = w.read(&path) else { continue };
        let markdown = path.extension().is_some_and(|x| x.eq_ignore_ascii_case("md"));
        let Some((out, notes)) = keel_guards::respell_cli_references(&text, markdown) else { continue };
        let shown = rel(root, &path);
        let detail = notes.iter().map(|n| format!("{shown}:{n}")).collect();
        w.stage(&path, out.clone());
        plan.files.push(FileEdit { path, new_content: out, edits: notes.len(), detail });
    }
    plan
}

fn collect_embedded(dir: &Dir, f: &mut impl FnMut(&include_dir::File)) {
    for file in dir.files() {
        f(file);
    }
    for d in dir.dirs() {
        collect_embedded(d, f);
    }
}

/// Step 2 — `Process` and `ProcessStep` became BEHAVIOUR (D0143): `part def` -> `action def`.
/// An instance must follow its definition's metaclass, so `part p : Process` -> `action p : Process`.
fn step_process_as_action(root: &Path, w: &mut Working) -> StepPlan {
    let mut plan = StepPlan::empty("process-as-action", "Process/ProcessStep instances become `action` (D0143)");
    for path in authored_files(root) {
        let Some(content) = w.read(&path) else { continue };
        let (c1, e1, mut d1) = retype_instances(&content, "Process", "part", "action");
        let (c2, e2, d2) = retype_instances(&c1, "ProcessStep", "part", "action");
        d1.extend(d2);
        if e1 + e2 > 0 {
            w.stage(&path, c2.clone());
            plan.files.push(FileEdit { path, new_content: c2, edits: e1 + e2, detail: d1 });
        }
    }
    plan
}

/// Step 3 — `ProcessStep.order` was deleted; sequence is native `first..then` succession.
fn step_processstep_order(root: &Path, w: &mut Working) -> StepPlan {
    let mut plan = StepPlan::empty("processstep-order", "Drop the deleted `ProcessStep.order` attribute (D0143)");
    for path in authored_files(root) {
        let Some(content) = w.read(&path) else { continue };
        let (new, edits, detail) = drop_processstep_order(&content);
        if edits > 0 {
            w.stage(&path, new.clone());
            plan.notes.push(format!(
                "{}: successions are NOT synthesised — write `first <a> then <b>;` yourself; guessing the order from a dropped attribute would fabricate an authored fact",
                rel(root, &path)
            ));
            plan.files.push(FileEdit { path, new_content: new, edits, detail });
        }
    }
    plan
}

/// Step 4 — `Release` was retyped `part def` -> `occurrence def` (it is a point in time).
fn step_release_as_occurrence(root: &Path, w: &mut Working) -> StepPlan {
    let mut plan = StepPlan::empty("release-as-occurrence", "Release instances become `occurrence` (D0142)");
    for path in authored_files(root) {
        let Some(content) = w.read(&path) else { continue };
        let (new, edits, detail) = retype_instances(&content, "Release", "part", "occurrence");
        if edits > 0 {
            w.stage(&path, new.clone());
            plan.files.push(FileEdit { path, new_content: new, edits, detail });
        }
    }
    plan
}

/// Step 5 — types the engine REMOVED. Blockers only: there is no mechanical target to rewrite to.
/// The line with its double-quoted string literals blanked (structure preserved), so the removed-type
/// and removed-enum scanners read SCHEMA, not PROSE (issue377 / GH#59).
///
/// A critic's finding quoting Rust - `ureq::Agent` - inside an authored `description` satisfied
/// `types_as`, because a path separator ends in a colon exactly as a typing colon does, and BLOCKED a
/// healthy project from moving vintage with no override surface. A `SysML` type annotation never appears
/// inside a string literal, so blanking the literals loses nothing the scanners are entitled to see;
/// a qualified reference OUTSIDE quotes still matches. Escaped quotes inside a literal are honoured.
fn without_string_literals(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_str = false;
    let mut escaped = false;
    for c in line.chars() {
        if in_str {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_str = false;
                out.push('"');
            }
            continue;
        }
        if c == '"' {
            in_str = true;
        }
        out.push(c);
    }
    out
}

fn step_removed_types(root: &Path, w: &Working) -> StepPlan {
    let mut plan = StepPlan::empty("removed-types", "Instances of REMOVED engine types (no mechanical transform)");
    for path in authored_files(root) {
        let Some(content) = w.read(&path) else { continue };
        for (i, line) in content.lines().enumerate() {
            // issue377: scan the line with its prose blanked; report the ORIGINAL line.
            let scan = without_line_comment(&without_string_literals(line));
            for (ty, advice) in REMOVED_TYPES {
                if types_as(&scan, ty) {
                    plan.blockers.push(Blocker {
                        path: path.clone(),
                        line: i + 1,
                        reason: format!("`{ty}` no longer exists in the engine schema: {}", clip(line, 72)),
                        advice: (*advice).to_string(),
                    });
                }
            }
            for (en, advice) in REMOVED_ENUMS {
                if scan.contains(&format!("{en}::")) {
                    plan.blockers.push(Blocker {
                        path: path.clone(),
                        line: i + 1,
                        reason: format!("`{en}` no longer exists in the engine schema: {}", clip(line, 72)),
                        advice: (*advice).to_string(),
                    });
                }
            }
        }
    }
    plan
}

/// Compute the whole plan. Pure: reads the tree, writes nothing.
#[must_use]
pub fn plan(root: &Path, engine: &Dir) -> MigrationPlan {
    // Ordered, and the order matters: each step reads what the previous one staged.
    let mut w = Working::new();
    let resync = step_engine_resync(root, engine);
    // The record accounts for the resync, so it is computed from that step and lands in the same
    // change — the gate that reads it runs over the whole applied plan, never over one step.
    let record = step_resync_record(root, &resync);
    // The respell reads the guard's population minus what the resync owns, so it is computed from
    // the same embedded engine — and before the record steps, whose overlay it shares (D0523).
    let respell = step_verb_respell(root, engine, &mut w);
    let mut steps = vec![resync, record, respell];
    steps.push(step_process_as_action(root, &mut w));
    steps.push(step_processstep_order(root, &mut w));
    steps.push(step_release_as_occurrence(root, &mut w));
    steps.push(step_removed_types(root, &w));
    MigrationPlan { steps }
}

// ── refusals ─────────────────────────────────────────────────────────────────

/// Why a migration must not run. Returned before any plan is computed.
pub enum Refusal {
    SelfBuild,
    NotAKeelProject,
    NotAGitRepo,
    DirtyTree(String),
}

impl Refusal {
    fn explain(&self, root: &Path) -> String {
        match self {
            Self::SelfBuild => format!(
                "{} is the keel SELF-BUILD repo — `.engine/` here is the source the binary embeds, not a copy of it.\n\
                 Migrating it would overwrite the engine with whatever the last build baked in. `keel migrate` is for downstream projects.",
                root.display()
            ),
            Self::NotAKeelProject => format!("{} has no .engine/ — nothing to migrate. Use `keel init` to scaffold a new project.", root.display()),
            Self::NotAGitRepo => format!(
                "{} is not a git repository (or git is unavailable).\n\
                 A migration rewrites authored facts in place, and without version control there is no way back from a bad run.\n\
                 Use `keel migrate --dry-run` to see the plan without writing anything.",
                root.display()
            ),
            Self::DirtyTree(s) => format!(
                "uncommitted changes in .tracking/ or .engine/ — refusing to migrate.\n\
                 A half-migrated tree mixed with uncommitted edits cannot be told apart afterwards, so commit or stash first.\n\
                 (`keel migrate --dry-run` inspects a dirty tree safely.)\n{s}"
            ),
        }
    }
}

/// Preconditions. `dry_run` relaxes only the tree-cleanliness check — a dry run writes nothing.
#[must_use]
/// Split one `git status --porcelain` line into `(status, path)`.
fn porcelain_parts(line: &str) -> Option<(&str, &str)> {
    (line.len() > 3).then(|| (&line[..2], line[3..].trim()))
}

/// May migrate proceed despite this uncommitted entry? (issue324)
///
/// # Why anything is tolerated at all
///
/// Three individually-correct controls composed into a lock with no non-bypassing exit. Migrate
/// refuses a dirty tree; committing to clean it fails because the pre-commit gate refuses under
/// engine-version skew (D0251); and skew clears only by running the pinned binary or by migrating.
/// The file that sprang it was written BY THE ENGINE — `record_obligation` under D0176/K7, whose own
/// doc comment reasons about deadlock at the FILE level and misses it at the TREE level. And it fires
/// preferentially when a project is ALREADY unhealthy, because that is when an obligation gets
/// recorded. Reported by a downstream session; unreachable here, since this tree refuses as a
/// self-build.
///
/// # Why THIS carve-out and not a looser one
///
/// Migrate's refusal exists because "a half-migrated tree mixed with uncommitted edits cannot be told
/// apart afterwards". A brand-new file in a dedicated engine-owned directory CAN be told apart, which
/// is exactly the property that makes it safe to skip and the reason the test is this narrow:
///
/// - `.tracking/obligations/` only — the one directory nothing but the engine writes.
/// - NEW entries only (`??` untracked, `A` staged-add). A MODIFIED obligation record is a hand-edit,
///   is no longer purely additive, and stays refused.
///
/// Two looser shapes were rejected. Staging the file in the recorder does not help — staged is still
/// uncommitted and `git status --porcelain` reports it either way. Degrading the pre-commit gate under
/// skew weakens one control to work around another, which is how a control gets hollowed out.
fn is_tolerable_obligation(status: &str, path: &str) -> bool {
    let p = path.replace('\\', "/");
    let is_new = status == "??" || status.starts_with('A');
    let is_sysml = std::path::Path::new(&p).extension().is_some_and(|e| e.eq_ignore_ascii_case("sysml"));
    is_new && p.starts_with(".tracking/obligations/") && is_sysml
}

/// Say which uncommitted entries were tolerated, and why they were safe to move with (issue324).
///
/// Separate from `cmd` so the exemption has a named home: a carve-out nobody is told about is
/// indistinguishable from a check that quietly stopped working, and this one exists precisely so an
/// ALREADY-UNHEALTHY project can move — which is when a reader most needs to see what moved with it.
fn report_tolerated(tolerated: &[String]) {
    if tolerated.is_empty() {
        return;
    }
    println!("  tolerated {} uncommitted engine-authored obligation record(s) — additive, and", tolerated.len());
    println!("  separable from migration state, so they do not make this tree ambiguous (issue324):");
    for line in tolerated {
        println!("    {line}");
    }
}

// ── tolerated through the run (D0522 / issue613, GH#87) ───────────────────────────────────────
//
// The door tolerated a record and the run then held it against the tree twice: the post-apply gate
// failed `guard issues` on the same untriaged record, and `restore`'s `git clean` deleted the
// never-committed file on the way out. One command, two definitions of separable. The tolerated
// paths now flow from `check_preconditions` to the gate (their parts are discounted) and to
// `restore` (their paths are excluded from the clean).

/// The root-relative paths behind the tolerated porcelain lines.
fn tolerated_paths(tolerated: &[String]) -> Vec<String> {
    tolerated.iter().filter_map(|l| porcelain_parts(l).map(|(_, p)| p.replace('\\', "/"))).collect()
}

/// The part names a record declares (`part <name> : Issue`): the subjects a guard names when it
/// fails on that record.
fn declared_parts(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|l| {
            let rest = l.trim().strip_prefix("part ")?;
            let name = rest.split(|c: char| c.is_whitespace() || c == ':').next()?;
            (!name.is_empty()).then(|| name.to_string())
        })
        .collect()
}

/// Every part declared across the tolerated records.
fn tolerated_parts(root: &Path, paths: &[String]) -> Vec<String> {
    paths.iter().filter_map(|p| std::fs::read_to_string(root.join(p)).ok()).flat_map(|t| declared_parts(&t)).collect()
}

/// Judge a red guard run against the tolerated parts. `Ok(discounted)` when at least one failing
/// line names a tolerated part and every other failing line is a guard header or the runner's
/// summary (`[guard:...] FAIL`, `[guard] FAILED`) - the run is green with those lines discounted.
/// `Err(red)` names the lines that stay red: a violation on any other subject fails the run as today.
fn discount_tolerated(output: &str, parts: &[String]) -> Result<Vec<String>, Vec<String>> {
    let failing = gate_failure_lines(output);
    let (discounted, rest): (Vec<String>, Vec<String>) = failing.into_iter().partition(|l| parts.iter().any(|p| l.contains(p.as_str())));
    let red: Vec<String> = rest.into_iter().filter(|l| !l.trim_start().starts_with("[guard")).collect();
    if discounted.is_empty() || !red.is_empty() {
        Err(red)
    } else {
        Ok(discounted)
    }
}

/// `git clean` over the run's directories, keeping every tolerated path (`-e` is an exclude
/// pattern in addition to the ignore rules). Empty `keep` is today's argument list exactly.
fn clean_args(keep: &[String]) -> Vec<String> {
    let mut args = vec!["clean".to_string(), "-fdq".to_string()];
    for k in keep {
        args.push("-e".to_string());
        args.push(k.clone());
    }
    args.extend(["--", ".engine", ".tracking", ".claude"].map(str::to_string));
    args
}

/// Preconditions. `dry_run` relaxes only the tree-cleanliness check — a dry run writes nothing.
///
/// `Ok` carries the uncommitted paths that were TOLERATED (see `is_tolerable_obligation`), so the
/// caller can name them: an exemption nobody is told about is indistinguishable from a check that
/// stopped working.
///
/// # Errors
/// A [`Refusal`] naming why this tree may not be migrated: it is the engine's own build, it is not a
/// keel project, it is not under version control, or it holds uncommitted changes that are not
/// tolerable obligation records.
pub fn check_preconditions(root: &Path, dry_run: bool) -> Result<Vec<String>, Refusal> {
    if root.join("keel-cli").join("Cargo.toml").is_file() {
        return Err(Refusal::SelfBuild);
    }
    if !root.join(".engine").is_dir() {
        return Err(Refusal::NotAKeelProject);
    }
    let out = keel_git::gitx::git()
        .arg("-C")
        .arg(root)
        // `-uall` because git COLLAPSES a wholly-untracked directory to one entry — the first run of
        // the issue324 test got `?? .tracking/obligations/` and no filename, so a per-file decision
        // was impossible. Listing files individually is also strictly better for the refusal path: it
        // names the actual blocking files instead of a directory the reader then has to go inspect.
        // `.claude` is in scope because the migration REGENERATES the surface (it is deployed from
        // the `.engine/skills/` the resync moves) and rolls it back with everything else. A
        // directory this run rewrites and restores has to be clean going in, or the restore would
        // discard an edit the run never made. CLAUDE.md is in scope because the `verb-respell` step
        // rewrites it (D0523) - the same argument, one file wide.
        .args(["status", "--porcelain", "-uall", "--", ".tracking", ".engine", ".claude", "CLAUDE.md"])
        .output();
    let Ok(out) = out else { return Err(Refusal::NotAGitRepo) };
    if !out.status.success() {
        return Err(Refusal::NotAGitRepo);
    }
    if dry_run {
        return Ok(Vec::new());
    }
    let status = String::from_utf8_lossy(&out.stdout);
    let (tolerated, blocking): (Vec<&str>, Vec<&str>) = status
        .lines()
        .filter(|l| !l.trim().is_empty())
        .partition(|l| porcelain_parts(l).is_some_and(|(s, p)| is_tolerable_obligation(s, p)));
    if blocking.is_empty() {
        Ok(tolerated.into_iter().map(str::to_string).collect())
    } else {
        Err(Refusal::DirtyTree(blocking.iter().take(20).map(|l| format!("    {l}")).collect::<Vec<_>>().join("\n")))
    }
}

// ── the command ──────────────────────────────────────────────────────────────

// ── rollback (srMigrationIsReversible) ────────────────────────────────────────────────────────
//
// D0252 promised that a migration which cannot finish rolls back. It did not exist: the only
// recovery was an error message advising the human to run `git checkout -- .`, which is a hope
// rather than a mechanism. The failure is not hypothetical — during design, piping migrate's output
// to `head` closed the pipe, killed the command mid-apply, and left one file written with the pin
// unstamped: a tree that is neither vintage.
//
// TWO CASES, and only one of them can be handled from inside the process:
//   DETECTED FAILURE — a write error or a mid-apply blocker. The process is alive, so it restores.
//   INTERRUPTION — the process is killed and runs no code at all. Nothing in-process can help, so
//     a MARKER is written before the first byte and removed after the last. Its presence on the
//     next run means the previous one did not finish, and that run restores before planning.
// The marker lives in `.keel/` (machine-local, untracked): it describes an interrupted RUN, not a
// fact about the model.

const IN_PROGRESS: &str = "migrate-in-progress";

fn marker_path(root: &Path) -> PathBuf {
    root.join(".keel").join(IN_PROGRESS)
}

fn head_sha(root: &Path) -> Option<String> {
    let out = keel_git::gitx::git().arg("-C").arg(root).args(["rev-parse", "HEAD"]).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Restore `.engine/`, `.tracking/`, `.claude/` - and `CLAUDE.md` when `sha` holds it - to `sha`,
/// discarding anything the interrupted run wrote.
///
/// Safe precisely BECAUSE migrate refuses a dirty tree: everything under those directories was
/// committed before the run, so resetting them to the pre-migration commit cannot destroy work.
/// `checkout` restores modified and deleted files; `clean` removes ones the run created - except the
/// tolerated records in `keep`, which the door admitted and the run never wrote (D0522). CLAUDE.md
/// (D0523) is only ever REWRITTEN by the run, never created, so it is restored when the pre-run
/// commit holds it and is not `clean`ed: a file the commit lacks is one the run did not write.
fn restore(root: &Path, sha: &str, keep: &[String]) -> Result<(), String> {
    let holds_claude_md = keel_git::gitx::git()
        .arg("-C")
        .arg(root)
        .args(["ls-tree", "--name-only", sha, "--", "CLAUDE.md"])
        .output()
        .is_ok_and(|o| o.status.success() && !o.stdout.is_empty());
    let mut scope = vec![".engine", ".tracking", ".claude"];
    if holds_claude_md {
        scope.push("CLAUDE.md");
    }
    let run = |args: &[String]| -> Result<(), String> {
        let out = keel_git::gitx::git()
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .map_err(|e| format!("git {args:?}: {e}"))?;
        if out.status.success() {
            Ok(())
        } else {
            Err(format!("git {args:?}: {}", String::from_utf8_lossy(&out.stderr).trim()))
        }
    };
    let words = |w: &[&str]| w.iter().map(|s| (*s).to_string()).collect::<Vec<_>>();
    let with_scope = |head: &[&str]| words(&[head, &["--"], &scope].concat());
    run(&with_scope(&["checkout", sha]))?;
    run(&clean_args(keep))?;
    // `checkout <sha> -- <paths>` also STAGES the restored content; unstage so the tree looks
    // untouched rather than merely having the right bytes.
    run(&with_scope(&["reset", "-q"]))
}

/// Roll back a detected failure, and say plainly whether the rollback itself worked.
fn rollback_after_failure(root: &Path, sha: Option<&String>, written: usize, keep: &[String]) -> i32 {
    let Some(sha) = sha else {
        eprintln!("  {written} file(s) were already written and NO pre-migration commit was recorded,");
        eprintln!("  so this tree is PARTIALLY MIGRATED and cannot be restored automatically.");
        return 1;
    };
    match restore(root, sha, keep) {
        Ok(()) => {
            let _ = std::fs::remove_file(marker_path(root));
            eprintln!("  ROLLED BACK: {written} written file(s) discarded; .engine/, .tracking/, .claude/ and CLAUDE.md restored to {sha}.");
            eprintln!("  The tree is as it was before this run. Nothing is half-migrated.");
            1
        }
        Err(e) => {
            eprintln!("  ROLLBACK FAILED ({e}) — this tree IS partially migrated after {written} file(s).");
            eprintln!("  Restore by hand: git checkout {sha} -- .engine .tracking .claude CLAUDE.md && git clean -fd -- .engine .tracking .claude");
            1
        }
    }
}

/// If a previous run was interrupted, restore before doing anything else. Returns a note to print.
fn recover_interrupted(root: &Path) -> Option<String> {
    let marker = marker_path(root);
    let text = std::fs::read_to_string(&marker).ok()?;
    // Line one is the pre-migration commit; any further lines are the tolerated paths the
    // interrupted run admitted at its door, kept through this restore too (D0522).
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
    let sha = lines.next().unwrap_or_default().to_string();
    let keep: Vec<String> = lines.map(str::to_string).collect();
    if sha.is_empty() {
        let _ = std::fs::remove_file(&marker);
        return None;
    }
    let note = match restore(root, &sha, &keep) {
        Ok(()) => format!(
            "recovered: a previous migration did not finish. .engine/ and .tracking/ restored to {sha} before planning."
        ),
        Err(e) => format!(
            "WARNING: a previous migration did not finish and could not be restored ({e}). Restore by hand: git checkout {sha} -- .engine .tracking .claude"
        ),
    };
    let _ = std::fs::remove_file(&marker);
    Some(note)
}

/// Record the pre-migration commit BEFORE the first byte is written. An interruption after this
/// point is detectable on the next run, which is the only recovery a killed process can have — so a
/// failure to arm it REFUSES the apply rather than proceeding unrecoverably.
fn arm_marker(root: &Path, pre_sha: Option<&String>, keep: &[String]) -> Result<(), i32> {
    let Some(sha) = pre_sha else { return Ok(()) };
    let _ = std::fs::create_dir_all(root.join(".keel"));
    let mut text = sha.clone();
    for k in keep {
        text.push('\n');
        text.push_str(k);
    }
    if let Err(e) = std::fs::write(marker_path(root), text) {
        eprintln!("keel migrate: cannot write the in-progress marker ({e}) — refusing to apply.");
        eprintln!("  Without it an interrupted run could not be detected, and this command's whole");
        eprintln!("  reversibility guarantee rests on that detection.");
        return Err(1);
    }
    Ok(())
}

/// Write every planned file, reporting how many succeeded before a failure so the rollback can say
/// what it discarded. Extracted from `cmd` to keep that function within its line budget.
/// Name every file the migration wrote, repo-relative (issue328).
///
/// # Why a COUNT was not enough
///
/// Migrate reported "wrote 2 file(s)" and nothing else, so a project that had CUSTOMISED an
/// engine-shipped file learned nothing when the resync replaced it. Observed directly: a local edit
/// to `.engine/skills/actor-enrollment/SKILL.md` was present before the run and gone after, with the
/// count as the only output. That is the issue314 class one file-kind over — that fix stopped the
/// resync reverting a project's ADOPTION choices; this is its edited CONTENT.
///
/// # Why naming, rather than preserving or three-way merging
///
/// Overwriting is arguably the correct POLICY: engine files belong to the engine, and a project that
/// wants durable local content has the unit mechanism for it. What is not defensible is doing it
/// silently. Preserving instead would leave a project pinned to a stale engine file forever with no
/// signal; a three-way merge would need the engine content at the project's OLD vintage, which is
/// exactly what keel deliberately does not store (there is no vintage stamp — the vintage IS which
/// steps still match). Naming the files costs nothing, is always truthful, and hands the reader the
/// one tool that settles it: `git diff` on a specific path, before the commit that pm3Reconcile
/// already tells them to make.
fn report_written(p: &MigrationPlan, root: &Path) {
    let mut paths: Vec<String> = p
        .steps
        .iter()
        .flat_map(|s| &s.files)
        .map(|f| f.path.strip_prefix(root).unwrap_or(&f.path).display().to_string().replace('\\', "/"))
        .collect();
    paths.sort();
    paths.dedup();
    if paths.is_empty() {
        return;
    }
    println!("  REVIEW THESE BEFORE COMMITTING — a resync overwrites engine files in place, so any");
    println!("  local edit to one of them is now gone. `git diff` says what changed (issue328):");
    for path in &paths {
        println!("    {path}");
    }
}

/// Regenerate the `.claude/` surface as part of the migration.
///
/// The surface is DEPLOYED from `.engine/skills/`, which the resync just moved, so it belongs to the
/// migration rather than to a chore afterwards. Leaving it out made the run fail its own gate:
/// `claude-surface-drift` compared a surface generated from the old vintage against the engine of
/// the new one, reported every skill stale, and the migration reverted — while `keel sync-claude`,
/// the command that would have fixed it, cannot run until the migration it is blocking has landed.
/// The same shape as the resync record: a post-condition the run's own gate demands, which only the
/// run can satisfy. A failure here is the migration's failure and rolls back with it — `Some(code)`
/// is that rollback's exit code, `None` means the surface is current.
fn resync_surface(root: &Path, pre_sha: Option<&String>, written: usize, keep: &[String]) -> Option<i32> {
    let Err(e) = keel_write::claude_surface::sync_claude(root, false) else { return None };
    eprintln!("keel migrate: wrote {written} file(s), but the .claude/ surface could not be regenerated: {e}");
    eprintln!("  The surface is deployed from the .engine/skills/ this run moved, so a tree with one vintage's");
    eprintln!("  engine and the other's surface is exactly the half-migrated state that must not survive.");
    Some(rollback_after_failure(root, pre_sha, written, keep))
}

fn apply_files(p: &MigrationPlan) -> Result<usize, (usize, PathBuf, std::io::Error)> {
    let mut written = 0usize;
    for s in &p.steps {
        for f in &s.files {
            if let Some(parent) = f.path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| (written, f.path.clone(), e))?;
            }
            std::fs::write(&f.path, &f.new_content).map_err(|e| (written, f.path.clone(), e))?;
            written += 1;
        }
    }
    Ok(written)
}

/// Re-stamp the binding engine pin, PRESERVING an existing file: only the `engine =` line is
/// rewritten, so comments and any other key the project added survive.
///
/// Regenerating it from `fresh` instead is the issue293 class — a writer rebuilding a
/// human-editable file from the engine's model of it drops whatever that model cannot represent.
/// This instance was found by the no-op invariance floor (issue310), not by a second field
/// incident, which is the whole point of having the floor. `fresh` is used only when no pin file
/// exists yet.
fn restamp_pin(root: &Path, fresh: &str) -> std::io::Result<()> {
    let path = root.join(".engine").join("contracts").join("engine-version.toml");
    let stamp = std::fs::read_to_string(&path).map_or_else(
        |_| fresh.to_string(),
        |existing| {
            existing
                .split_inclusive('\n')
                .map(|line| {
                    if line.trim_start().starts_with("engine") && line.contains('=') {
                        let eol = &line[line.trim_end().len()..];
                        format!("engine = \"{}\"{eol}", env!("CARGO_PKG_VERSION"))
                    } else {
                        line.to_string()
                    }
                })
                .collect()
        },
    );
    std::fs::write(&path, stamp)
}

/// `keel migrate [ROOT] [--dry-run]`. Returns the process exit code.
///
/// Order is deliberate: refuse, then PLAN and print, then apply. The plan is always shown before
/// anything is written, and any blocker aborts the entire run rather than applying the steps that
/// happen to be clean — a partially migrated project that reports success is the most expensive
/// possible outcome.
#[must_use]
pub fn cmd(root: &Path, engine: &Dir, dry_run: bool) -> i32 {
    cmd_with(root, engine, dry_run, true)
}

/// `verify = false` skips the post-apply gate (`--no-verify`): the tree is written and NOT verified,
/// and the run says so. The default is the D0336 shape - verified-green or reverted, no third state.
#[must_use]
pub fn cmd_with(root: &Path, engine: &Dir, dry_run: bool, verify: bool) -> i32 {
    // RECOVERY RUNS FIRST, before the preconditions. An interrupted migration leaves the tree
    // DIRTY, and the dirty-tree precondition would refuse — so a genuine interruption could never
    // be recovered by the very command that recovers it. Found by the interruption test, which is
    // the case this ordering exists for.
    let recovery = if dry_run { None } else { recover_interrupted(root) };
    let tolerated = match check_preconditions(root, dry_run) {
        Ok(t) => t,
        Err(r) => {
            if let Some(note) = &recovery {
                println!("  {note}");
            }
            eprintln!("keel migrate: {}", r.explain(root));
            return 2;
        }
    };
    let pre_sha = if dry_run { None } else { head_sha(root) };
    // srUnprovenUpdateAsksRatherThanActs: a version that was reverted here before is not retried in
    // silence - the attempt record names it and the gate that refused it.
    if let Some(prior) = last_attempt(root).filter(|a| a.version == env!("CARGO_PKG_VERSION") && a.outcome == "reverted") {
        println!("  NOTE: {} was applied here on {} and REVERTED - {} failed: {}", prior.version, prior.at, prior.gate, clip(&prior.output, 160));
        println!("        Re-running because you asked; the same gate decides again.");
    }
    let p = plan(root, engine);
    let active = p.active();
    if let Some(note) = &recovery {
        println!("  {note}");
    }

    println!("keel migrate — {}", root.display());
    println!("  binary engine: keel {} (build {})", env!("CARGO_PKG_VERSION"), env!("KEEL_BUILD_COMMIT"));
    report_tolerated(&tolerated);
    let keep = tolerated_paths(&tolerated);
    if active.is_empty() {
        println!("  detected vintage: CURRENT — no step applies. Nothing to do.");
        return 0;
    }
    println!("  detected vintage: {} of {} step(s) apply (no version is stamped; the vintage IS which steps still match):", active.len(), p.steps.len());
    println!();
    for s in &p.steps {
        let mark = if s.is_noop() { "  ·" } else { "  →" };
        println!("{mark} [{}] {}", s.id, s.title);
        if s.is_noop() && s.notes.is_empty() {
            println!("      already current");
            continue;
        }
        for n in &s.notes {
            println!("      note: {n}");
        }
        for f in &s.files {
            println!("      {} ({} edit(s))", rel(root, &f.path), f.edits);
            for d in f.detail.iter().take(8) {
                println!("          {d}");
            }
            if f.detail.len() > 8 {
                println!("          ... {} more", f.detail.len() - 8);
            }
        }
        for b in &s.blockers {
            println!("      BLOCKED {}:{} — {}", rel(root, &b.path), b.line, b.reason);
            println!("              advice: {}", b.advice);
        }
    }
    println!();
    println!("  totals: {} file(s), {} edit(s), {} blocker(s)", p.files(), p.edits(), p.blockers());

    if p.blockers() > 0 {
        eprintln!();
        eprintln!(
            "keel migrate: REFUSING — {} item(s) have no mechanical transform (listed above).\n\
             Nothing was written. Resolve each one (the advice names the replacement or tells you to re-add the def\n\
             in your own .engine/schema/, which D0093 explicitly permits), then re-run.",
            p.blockers()
        );
        return 1;
    }
    if dry_run {
        println!("  --dry-run: nothing written.");
        return 0;
    }

    if let Err(code) = arm_marker(root, pre_sha.as_ref(), &keep) {
        return code;
    }
    let written = match apply_files(&p) {
        Ok(n) => n,
        Err((n, path, e)) => {
            eprintln!("keel migrate: FAILED writing {}: {e}", rel(root, &path));
            return rollback_after_failure(root, pre_sha.as_ref(), n, &keep);
        }
    };

    if let Some(code) = resync_surface(root, pre_sha.as_ref(), written, &keep) { return code }

    // Reconcile against the plan by RE-PLANNING. Every step is content-detected, so a correct run
    // leaves nothing matching; a non-empty re-plan means a transform did not do what it reported.
    // This is the idempotency guarantee checked at runtime rather than asserted in a comment.
    let after = plan(root, engine);
    if !after.active().is_empty() {
        eprintln!("keel migrate: wrote {written} file(s), but re-planning still finds {} edit(s) and {} blocker(s).", after.edits(), after.blockers());
        eprintln!("  A step did not do what it reported, so this migration is NOT verified.");
        // Rolled back rather than left for inspection (srMigrationIsReversible): "migrated but not
        // verified" is precisely the state that must not survive a run. Previously this advised a
        // `git diff` and left the tree written.
        return rollback_after_failure(root, pre_sha.as_ref(), written, &keep);
    }
    // D0190: a completed migration re-stamps the declared engine version. D0251 ESCALATED what the
    // stamp means: it is no longer a parity-warning input but a BINDING pin — a binary whose version
    // differs from it now REFUSES writes and gates (reads warn; `version`/`migrate` never refuse).
    // Migrate is the repair path, so migrate is where an existing project hears about the change.
    // (`restamp_pin` below PRESERVES an existing pin file — see its doc comment.)
    println!(
        "note (D0251): engine-version.toml is now BINDING — a binary that does not match the stamped {} will REFUSE writes and gates on this tree (reads warn; `keel migrate` re-stamps).",
        env!("CARGO_PKG_VERSION")
    );
    let stamp = format!(
        "# engine-version - the BINDING engine pin: the version whose writes and gates this project accepts\n# (D0190 stamped it; D0251 made it bite). Written by `keel init`, re-stamped by `keel migrate`; a\n# mismatched binary refuses writes and gates, warns on reads. keelw resolves this pin (D0251 B).\nengine = \"{}\"\n",
        env!("CARGO_PKG_VERSION")
    );
    if let Err(e) = restamp_pin(root, &stamp) {
        eprintln!("keel migrate: migration complete but the version re-stamp failed ({e}) - the parity warning will keep firing until engine-version.toml is updated.");
    }
    finish_applied(root, &p, pre_sha.as_ref(), written, verify, &keep)
}

/// VERIFIED OR REVERTED (D0336; srUpdateIsVerifiedOrReverted). The re-plan proves the transform did
/// what it said; it does not prove the PROJECT still gates. So the project's own gate runs - validate,
/// every enforced guard, check-engine - under the binary that just wrote the tree. Green retains and
/// reports what moved including the pin; ANY red restores the pre-update commit and reports the gate's
/// output verbatim. There is no third outcome: the tree is verified-green or it is byte-for-byte what
/// it was. NOT gated on "empty plan": an empty plan returned before the re-stamp and never reaches
/// here, and a real upgrade always plans at least the resync. `verify = false` writes and says UNVERIFIED.
fn finish_applied(root: &Path, p: &MigrationPlan, pre_sha: Option<&String>, written: usize, verify: bool, keep: &[String]) -> i32 {
    if verify {
        match project_gate(root, &tolerated_parts(root, keep)) {
            Ok(()) => {
                let _ = std::fs::remove_file(marker_path(root));
                println!("  wrote {written} file(s). Re-plan is empty and the project's own gate is GREEN under {}: the update is RETAINED.", env!("CARGO_PKG_VERSION"));
                report_written(p, root);
                println!("  pin: engine-version.toml now reads {}", env!("CARGO_PKG_VERSION"));
                record_attempt(root, "retained", "validate + guard + check-engine", "green");
                println!();
                println!("  NEXT: commit. The gate that matters has already run on the tree you are in.");
                0
            }
            Err((gate, output)) => {
                eprintln!("keel migrate: the update was written but this project's own gate went RED under {} - {gate}:", env!("CARGO_PKG_VERSION"));
                for line in gate_failure_lines(&output) {
                    eprintln!("    {line}");
                }
                record_attempt(root, "reverted", &gate, &gate_failure_lines(&output).join("\n"));
                eprintln!("  REVERTING: verified-green or byte-for-byte as before - there is no third state (srUpdateIsVerifiedOrReverted).");
                let code = rollback_after_failure(root, pre_sha, written, keep);
                eprintln!("  RECORDED in .keel/update-attempts.toml (`keel show status` shows it): version {}, gate {gate}. A re-run will say this version was reverted here.", env!("CARGO_PKG_VERSION"));
                code
            }
        }
    } else {
        let _ = std::fs::remove_file(marker_path(root));
        println!("  wrote {written} file(s). Re-plan is empty: the migration is complete and idempotent.");
        report_written(p, root);
        println!();
        println!("  --no-verify: the project's own gate was NOT run. This tree is written and UNVERIFIED (D0336).");
        println!("  Run `keel gate validate . && keel gate guard && keel gate check-engine .` yourself, then commit.");
        0
    }
}

/// The project's own gate under THIS binary: validate, every enforced guard, check-engine. `Err((gate,
/// verbatim output))` on the first red.
/// The lines of a gate run that EXPLAIN a failure, in order: anything failing, then the tail.
///
/// A gate prints one PASS per check, so `take(40)` showed forty passes and hid the violation that
/// caused the revert - the 0.4.0 bump reverted on `claude-surface-drift` and neither the printed
/// report nor the recorded attempt contained the word. Relevance, not position.
fn gate_failure_lines(output: &str) -> Vec<String> {
    let interesting: Vec<&str> = output
        .lines()
        .filter(|l| {
            let u = l.to_uppercase();
            (u.contains("FAIL") || u.contains("ERROR") || u.contains("VIOLATION") || u.contains("REFUSED"))
                && !u.contains("0 VIOLATION(S)")
        })
        .collect();
    if interesting.is_empty() {
        // nothing named itself: the tail is where a crash or a summary lands
        let all: Vec<&str> = output.lines().collect();
        return all.iter().skip(all.len().saturating_sub(20)).map(|l| (*l).to_string()).collect();
    }
    let mut out: Vec<String> = interesting.iter().take(20).map(|l| (*l).to_string()).collect();
    if interesting.len() > 20 {
        out.push(format!("... and {} more failing line(s)", interesting.len() - 20));
    }
    out
}

///
/// A red `guard` whose every failing line names a part of a tolerated record is DISCOUNTED and
/// printed as such (D0522): the door admitted that record as separable, so the gate does not hold
/// the run to it. Any other failing line is the run's red, as before.
fn project_gate(root: &Path, tolerated_parts: &[String]) -> Result<(), (String, String)> {
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("keel"));
    let r = root.to_string_lossy().to_string();
    for (gate, args) in [("validate", vec!["gate", "validate", r.as_str()]), ("guard", vec!["gate", "guard", "all", r.as_str()]), ("check-engine", vec!["gate", "check-engine", r.as_str()])] {
        let out = std::process::Command::new(&exe).args(&args).output().map_err(|e| (gate.to_string(), format!("could not run keel {gate}: {e}")))?;
        if !out.status.success() {
            let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
            if gate == "guard" && !tolerated_parts.is_empty() {
                if let Ok(discounted) = discount_tolerated(&text, tolerated_parts) {
                    println!("  guard: {} failing line(s) DISCOUNTED - each names a record tolerated at the door (D0522):", discounted.len());
                    for line in &discounted {
                        println!("    {}", line.trim());
                    }
                    continue;
                }
            }
            return Err((gate.to_string(), text));
        }
    }
    Ok(())
}

/// One recorded update attempt (machine-local `.keel/update-attempts.toml`, D0336).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateAttempt {
    pub version: String,
    pub at: String,
    pub outcome: String,
    pub gate: String,
    pub output: String,
}

fn attempts_path(root: &Path) -> PathBuf {
    root.join(".keel").join("update-attempts.toml")
}

/// Append an attempt: the durable artefact srUnprovenUpdateAsksRatherThanActs requires, read back by
/// `keel show status` and by the next `keel migrate`.
fn record_attempt(root: &Path, outcome: &str, gate: &str, output: &str) {
    use std::fmt::Write as _;
    let path = attempts_path(root);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut text = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        "# Engine update attempts on this machine (D0336): version, outcome, the gate that decided, its output.\n# `keel show status` shows the latest; `keel migrate` names a version that was reverted here before retrying it.\n".to_string()
    });
    let clean = |s: &str| s.replace('"', "'").replace('\r', "").lines().take(40).collect::<Vec<_>>().join("\\n");
    let _ = write!(
        text,
        "\n[[attempt]]\nversion = \"{}\"\nat = \"{}\"\noutcome = \"{outcome}\"\ngate = \"{}\"\noutput = \"{}\"\n",
        env!("CARGO_PKG_VERSION"),
        keel_write::scaffold::today(),
        clean(gate),
        clean(output)
    );
    let _ = std::fs::write(&path, text);
}

/// Every recorded attempt, oldest first.
#[must_use]
pub fn attempts(root: &Path) -> Vec<UpdateAttempt> {
    let Ok(text) = std::fs::read_to_string(attempts_path(root)) else { return Vec::new() };
    parse_attempts(&text)
}

/// The most recent attempt, if any.
#[must_use]
pub fn last_attempt(root: &Path) -> Option<UpdateAttempt> {
    attempts(root).pop()
}

/// Pure parse of the attempts file.
#[must_use]
pub fn parse_attempts(text: &str) -> Vec<UpdateAttempt> {
    let mut out = Vec::new();
    let mut cur: Option<UpdateAttempt> = None;
    for line in text.lines() {
        let l = line.trim();
        if l == "[[attempt]]" {
            if let Some(a) = cur.take() {
                out.push(a);
            }
            cur = Some(UpdateAttempt { version: String::new(), at: String::new(), outcome: String::new(), gate: String::new(), output: String::new() });
            continue;
        }
        let (Some(a), Some((k, v))) = (cur.as_mut(), l.split_once('=')) else { continue };
        let v = v.trim().trim_matches('"').replace("\\n", "\n");
        match k.trim() {
            "version" => a.version = v,
            "at" => a.at = v,
            "outcome" => a.outcome = v,
            "gate" => a.gate = v,
            "output" => a.output = v,
            _ => {}
        }
    }
    if let Some(a) = cur {
        out.push(a);
    }
    out
}

/// The line up to its `//` comment, for the removed-types scan (issue675). Applied after
/// `without_string_literals`, so a `//` inside a literal is already gone and the first `//` left is
/// a comment. The `// RAN: <evidence>` line the write API emits is prose: evidence beginning "Task"
/// put `: Task` on the line and read as a typing colon.
fn without_line_comment(line: &str) -> String {
    line.find("//").map_or(line, |at| &line[..at]).to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        clean_args, declared_parts, discount_tolerated, drop_processstep_order, is_engine_dev_only, remap_engine_path, retype_instances,
        step_engine_resync, step_process_as_action, step_processstep_order, step_release_as_occurrence, step_removed_types, step_resync_record,
        strip_order_assignment, tolerated_paths, types_as, without_line_comment, without_string_literals, FileEdit, Path, StepPlan, Working,
    };

    /// D0388 probe pair for the resync record, chosen before the real tree was read.
    ///
    /// KNOWN-POSITIVE: a resync that wrote files crossed an ownership boundary, so the record has to
    /// exist — it is the evidence the `ownership` guard reads to suspend D0108 (D0067), and without it
    /// `keel migrate` reverts itself in every downstream project.
    #[test]
    fn a_resync_that_wrote_files_records_the_transform_its_own_gate_reads() {
        let mut resync = StepPlan::empty("engine-resync", "t");
        resync.files.push(FileEdit {
            path: Path::new("/p/.engine/cli/commands.sysml").to_path_buf(),
            new_content: String::new(),
            edits: 3,
            detail: vec![],
        });
        let plan = step_resync_record(Path::new("/p"), &resync);
        let [file] = &plan.files[..] else { panic!("expected exactly one record, got {}", plan.files.len()) };
        let parent = file.path.parent().expect("the record has a parent");
        assert!(
            parent.ends_with(Path::new(".engine/tools/migrations")),
            "the record must land on the path the ownership guard reads, not beside it: {}",
            file.path.display()
        );
        assert!(file.new_content.contains("- .engine/cli/commands.sysml"), "it accounts for the file that crossed: {}", file.new_content);
        assert!(file.new_content.contains("- edits: 3"), "and carries the step's control total: {}", file.new_content);
    }

    /// KNOWN-NEGATIVE: a project already current resyncs nothing, so nothing crossed an ownership
    /// boundary and there is nothing to account for. A record written anyway would be a standing
    /// D0108 exemption in the tree — the claimable shape the guard exists to refuse.
    #[test]
    fn a_noop_resync_records_nothing() {
        let plan = step_resync_record(Path::new("/p"), &StepPlan::empty("engine-resync", "t"));
        assert!(plan.files.is_empty(), "a no-op resync must not leave an exemption behind: {:?}", plan.files.len());
        assert!(plan.is_noop(), "and the step itself is a no-op");
    }

    #[test]
    fn type_match_respects_identifier_boundaries() {
        assert!(types_as("    part p : Process {", "Process"));
        assert!(types_as("    part p : Process;", "Process"));
        // The bug this guards: `MyProcess` and `ProcessStep` must not both answer to `Process`.
        assert!(!types_as("    part p : MyProcess {", "Process"));
        assert!(!types_as("    part p : ProcessStep {", "Process"));
        assert!(types_as("    part p : ProcessStep {", "ProcessStep"));
        assert!(!types_as("    // a comment mentioning Process", "Process"));
    }

    #[test]
    fn retype_converts_instances_and_never_definitions() {
        let src = "package P {\n    part def Process :> Element { }\n    part daily : Process {\n        :>> id = \"x\";\n    }\n}\n";
        let (out, edits, _) = retype_instances(src, "Process", "part", "action");
        assert_eq!(edits, 1);
        assert!(out.contains("part def Process :> Element"), "a DEF is engine content, resynced not rewritten:\n{out}");
        assert!(out.contains("    action daily : Process {"), "{out}");
    }

    #[test]
    fn order_is_dropped_only_inside_a_processstep_body() {
        let src = "package P {\n    part s : ProcessStep {\n        :>> order = 3;\n        :>> title = \"t\";\n    }\n    part w : WorkItem {\n        :>> order = 9;\n    }\n}\n";
        let (out, edits, _) = drop_processstep_order(src);
        assert_eq!(edits, 1, "only the ProcessStep's order is dead:\n{out}");
        assert!(!out.contains(":>> order = 3;"), "{out}");
        assert!(out.contains(":>> order = 9;"), "an unrelated `order` attribute is untouched:\n{out}");
        assert!(out.contains(":>> title = \"t\";"), "{out}");
        // Idempotent: a second pass finds nothing.
        let (again, edits2, _) = drop_processstep_order(&out);
        assert_eq!(edits2, 0);
        assert_eq!(again, out);
    }

    #[test]
    fn order_is_dropped_from_a_single_line_body() {
        let src = "    part s : ProcessStep { :>> id = \"a\"; :>> order = 2; :>> title = \"t\"; }\n";
        let (out, edits, _) = drop_processstep_order(src);
        assert_eq!(edits, 1);
        assert!(!out.contains("order"), "{out}");
        assert!(out.contains(":>> id = \"a\";") && out.contains(":>> title = \"t\";"), "{out}");
    }

    #[test]
    fn strip_order_requires_a_whole_attribute_name() {
        assert!(strip_order_assignment(":>> order = 1;").is_some());
        assert!(strip_order_assignment(":>> orderIndex = 1;").is_none(), "a longer name must not be truncated");
        assert!(strip_order_assignment(":>> title = \"x\";").is_none());
    }

    #[test]
    fn engine_path_rules_hold() {
        assert_eq!(remap_engine_path(Path::new("decisions/0001-x.sysml")), Path::new("reference/decisions/0001-x.sysml"));
        assert_eq!(remap_engine_path(Path::new("schema/core/element.sysml")), Path::new("schema/core/element.sysml"));
        assert!(is_engine_dev_only(Path::new("tools/validate/x.py")));
        assert!(!is_engine_dev_only(Path::new("schema/core/element.sysml")));
    }

    /// issue377 / GH#59: a removed type NAMED IN PROSE is not a declaration. A critic's quoted Rust
    /// (`ureq::Agent`) inside a description blocked a healthy project; the same name as a real `: Agent`
    /// annotation outside quotes must still block, so the fix cannot pass by never matching. The
    /// removed-enum scan has the same two arms.
    #[test]
    fn a_removed_type_named_inside_a_string_literal_is_prose_not_schema() {
        let dir = std::env::temp_dir().join(format!("keel-migrate-prose-{}", std::process::id()));
        let tracking = dir.join(".tracking");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&tracking).unwrap();
        std::fs::write(
            tracking.join("i.sysml"),
            "package I {\n    part i : Issue {\n        :>> description = \"get() uses a concrete ureq::Agent with no injection point, so the 429 arm has no executing test; std::task::Task is quoted too, and RiskLevel::high is named.\";\n    }\n}\n",
        )
        .unwrap();
        let w = Working::new();
        let prose = step_removed_types(&dir, &w);
        assert!(
            prose.blockers.is_empty(),
            "a removed type NAMED IN PROSE must not block: {:?}",
            prose.blockers.iter().map(|b| &b.reason).collect::<Vec<_>>()
        );
        // The other arm: the same names as REAL schema references outside quotes still block.
        std::fs::write(
            tracking.join("i.sysml"),
            "package I {\n    part a : Agent { :>> description = \"quoted words\"; }\n    part r : Story { :>> level = RiskLevel::high; }\n}\n",
        )
        .unwrap();
        let schema = step_removed_types(&dir, &w);
        let reasons: Vec<&String> = schema.blockers.iter().map(|b| &b.reason).collect();
        assert!(reasons.iter().any(|r| r.contains("`Agent`")), "a real `: Agent` annotation still blocks: {reasons:?}");
        assert!(reasons.iter().any(|r| r.contains("`RiskLevel`")), "a real enum value still blocks: {reasons:?}");
        // Escaped quotes inside a literal do not end it early.
        assert_eq!(without_string_literals("a = \"x \\\" : Agent\" : Real"), "a = \"\" : Real");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// issue675: the `// RAN: <evidence>` line `keel record result` writes is a comment, not schema.
    /// Evidence beginning `Task DoD` put `: Task` on the line and blocked keel-next's move to 0.5.2 on
    /// four of the writer's own lines. The other arm: a real annotation before a trailing comment, and
    /// a `//` inside a string literal, still leave the schema visible.
    #[test]
    fn a_removed_type_named_inside_a_line_comment_is_prose_not_schema() {
        let dir = std::env::temp_dir().join(format!("keel-migrate-comment-{}", std::process::id()));
        let tracking = dir.join(".tracking");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&tracking).unwrap();
        std::fs::write(
            tracking.join("b.sysml"),
            "package B {\n    action def W {\n        action w;\n        // RAN: Task DoD (method analyze). // RAN: grep -c x -> 4 ; level RiskLevel::high\n    }\n}\n",
        )
        .unwrap();
        let w = Working::new();
        let comment = step_removed_types(&dir, &w);
        assert!(
            comment.blockers.is_empty(),
            "a removed type NAMED IN A COMMENT must not block: {:?}",
            comment.blockers.iter().map(|b| &b.reason).collect::<Vec<_>>()
        );
        std::fs::write(tracking.join("b.sysml"), "package B {\n    part a : Agent; // an agent\n}\n").unwrap();
        let schema = step_removed_types(&dir, &w);
        assert!(
            schema.blockers.iter().any(|b| b.reason.contains("`Agent`")),
            "a real `: Agent` annotation before a comment still blocks"
        );
        assert_eq!(without_line_comment(&without_string_literals("a = \"http://x\" : Real // : Agent")), "a = \"\" : Real ");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The end-to-end shape on a real directory: an old-vintage tree plans edits and blockers, and
    /// the migrated tree plans neither.
    ///
    /// The load-bearing assertion is the LAST one. Two steps land on the same `ProcessStep` — the
    /// retype and the `order` drop — and each produces a whole-file result. Before the `Working`
    /// overlay, both were computed from the on-disk original, so applying them in order silently
    /// discarded the retype: the file ended up `part s1 : ProcessStep` with the order gone. Asserting
    /// only "a second run is a no-op" would NOT have caught it, because the second run reads the
    /// clobbered file and the retype step happily plans the same edit again forever.
    #[test]
    fn old_vintage_tree_migrates_then_is_a_noop() {
        let dir = std::env::temp_dir().join(format!("keel-migrate-test-{}", std::process::id()));
        let tracking = dir.join(".tracking");
        std::fs::create_dir_all(&tracking).unwrap();
        let src = tracking.join("p.sysml");
        std::fs::write(
            &src,
            "package P {\n    part daily : Process {\n        :>> id = \"a\";\n    }\n    part s1 : ProcessStep {\n        :>> order = 1;\n    }\n    part v : Release { :>> id = \"c\"; }\n    part t : Task { :>> id = \"b\"; }\n}\n",
        )
        .unwrap();

        let mut w = Working::new();
        let retype = step_process_as_action(&dir, &mut w);
        assert_eq!(retype.edits(), 2, "both the Process and the ProcessStep instance");
        let order = step_processstep_order(&dir, &mut w);
        assert_eq!(order.edits(), 1);
        let release = step_release_as_occurrence(&dir, &mut w);
        assert_eq!(release.edits(), 1);
        let removed = step_removed_types(&dir, &w);
        assert_eq!(removed.blockers.len(), 1, "`Task` was removed and has no mechanical target");
        assert!(removed.blockers[0].advice.contains("Story"));

        // A removed ENUM is referenced by value (`RiskLevel::high`), never by `: RiskLevel`, so the
        // type detector cannot see it. It lived in a file the engine ships and resync OVERWRITES.
        std::fs::write(tracking.join("e.sysml"), "package E {\n    part r : Story { :>> level = RiskLevel::high; }\n}\n").unwrap();
        let mut w2 = Working::new();
        let enums = step_removed_types(&dir, &w2);
        assert!(
            enums.blockers.iter().any(|b| b.reason.contains("RiskLevel")),
            "a removed enum's VALUE form must block: {:?}",
            enums.blockers.iter().map(|b| &b.reason).collect::<Vec<_>>()
        );
        // `RiskStatus` must NOT block: its file is no longer shipped, and resync never deletes a
        // file it does not ship, so the project keeps its own copy and the reference still resolves.
        std::fs::write(tracking.join("e.sysml"), "package E {\n    part r : Story { :>> s = RiskStatus::open; }\n}\n").unwrap();
        w2 = Working::new();
        assert!(
            !step_removed_types(&dir, &w2).blockers.iter().any(|b| b.reason.contains("RiskStatus")),
            "RiskStatus lives in a file the engine stopped shipping — the project keeps it"
        );
        std::fs::remove_file(tracking.join("e.sysml")).ok();

        // Apply in plan order, exactly as `cmd` does.
        for s in [&retype, &order, &release] {
            for f in &s.files {
                std::fs::write(&f.path, &f.new_content).unwrap();
            }
        }
        let final_content = std::fs::read_to_string(&src).unwrap();
        assert!(final_content.contains("action daily : Process"), "step 2's retype survived step 3:\n{final_content}");
        assert!(final_content.contains("action s1 : ProcessStep"), "the retype was NOT clobbered by the order drop:\n{final_content}");
        assert!(!final_content.contains("order"), "the order drop survived too:\n{final_content}");
        assert!(final_content.contains("occurrence v : Release"), "{final_content}");

        let mut w2 = Working::new();
        assert_eq!(step_process_as_action(&dir, &mut w2).edits(), 0, "a second run is a no-op");
        assert_eq!(step_processstep_order(&dir, &mut w2).edits(), 0);
        assert_eq!(step_release_as_occurrence(&dir, &mut w2).edits(), 0);

        std::fs::remove_dir_all(&dir).ok();
    }

    /// D0388 pair for issue622 (D0523), chosen before the tree was read. The downloaded 0.5.0 rolled
    /// itself back on sixteen `cli-reference` violations in files the run never writes.
    ///
    /// KNOWN-POSITIVE: a fixture CLAUDE.md naming `keel orient`, `keel add-task` and `keel report`,
    /// and a project-owned contract whose comment names `keel guard attestation-authority`, plan one
    /// `FileEdit` each; the planned content names `keel show orient`, `keel record task`,
    /// `keel render report` and `keel gate guard attestation-authority`; the plan's detail names each
    /// rewrite by file and line. KNOWN-NEGATIVE: a file the embedded engine ships (`docs/guards.md`)
    /// is the resync's and is not read, even holding `keel orient` off its shipped bytes; the fixture
    /// already at the current spellings plans zero edits and is byte-identical; a second run over the
    /// applied plan plans nothing.
    #[test]
    fn the_projects_own_living_docs_are_respelled_and_the_resyncs_files_are_not() {
        let engine = &keel_schema::embedded::ENGINE_DIR;
        let dir = std::env::temp_dir().join(format!("keel-respell-test-{}", std::process::id()));
        let contracts = dir.join(".engine").join("contracts");
        let docs = dir.join(".engine").join("docs");
        std::fs::create_dir_all(&contracts).unwrap();
        std::fs::create_dir_all(&docs).unwrap();
        let claude = dir.join("CLAUDE.md");
        let old_claude = "# how to work here\n\n```\nkeel orient .\nkeel add-task --title x\n```\n\nRun `keel report assurance` weekly; `keel gate validate .` before every commit.\n";
        std::fs::write(&claude, old_claude).unwrap();
        let policy = contracts.join("attestation-policy.toml");
        std::fs::write(&policy, "# Read by `keel guard attestation-authority` (D0312).\n[policy]\nmode = \"strict\"\n").unwrap();
        // A file the embedded engine ships: the resync's authority, so never read here - even off
        // its shipped bytes, where the resync (not this step) plans the write.
        assert!(engine.get_file("docs/guards.md").is_some(), "the negative names a file the engine ships");
        let shipped = docs.join("guards.md");
        std::fs::write(&shipped, "run `keel orient .`\n").unwrap();

        let mut w = Working::new();
        let step = super::step_verb_respell(&dir, engine, &mut w);
        assert_eq!(step.id, "verb-respell");
        let mut touched: Vec<String> = step.files.iter().map(|f| f.path.file_name().unwrap().to_string_lossy().into_owned()).collect();
        touched.sort();
        assert_eq!(touched, ["CLAUDE.md", "attestation-policy.toml"], "one edit per file the guard reads and the resync does not write");
        let new_claude = &step.files.iter().find(|f| f.path == claude).unwrap().new_content;
        for want in ["keel show orient .", "keel record task --title x", "keel render report assurance", "keel gate validate ."] {
            assert!(new_claude.contains(want), "expected `{want}` in:\n{new_claude}");
        }
        assert!(!new_claude.contains("keel orient") && !new_claude.contains("keel add-task") && !new_claude.contains("keel report "), "{new_claude}");
        let new_policy = &step.files.iter().find(|f| f.path == policy).unwrap().new_content;
        assert!(new_policy.contains("`keel gate guard attestation-authority`"), "{new_policy}");
        assert!(step.files.iter().all(|f| f.edits == f.detail.len() && f.edits > 0), "{:?}", step.files.iter().map(|f| (&f.edits, &f.detail)).collect::<Vec<_>>());
        assert!(step.files.iter().any(|f| f.detail.iter().any(|d| d.starts_with("CLAUDE.md:4: keel orient -> keel show orient"))), "{:?}", step.files.iter().map(|f| &f.detail).collect::<Vec<_>>());
        assert_eq!(std::fs::read_to_string(&shipped).unwrap(), "run `keel orient .`\n", "the resync's file was not read");

        // Apply as `cmd` does, then a second run plans nothing.
        for f in &step.files {
            std::fs::write(&f.path, &f.new_content).unwrap();
        }
        let mut w2 = Working::new();
        let again = super::step_verb_respell(&dir, engine, &mut w2);
        assert_eq!(again.edits(), 0, "idempotent: {:?}", again.files.iter().map(|f| &f.detail).collect::<Vec<_>>());
        assert!(std::fs::read_to_string(&claude).unwrap().contains("keel show orient ."));

        // The negative on its own: a tree already at the current spellings is untouched, byte for byte.
        let current = "run `keel show orient .`, then `keel record task --title x` and `keel render report assurance`.\n";
        std::fs::write(&claude, current).unwrap();
        let mut w3 = Working::new();
        assert_eq!(super::step_verb_respell(&dir, engine, &mut w3).edits(), 0);
        assert_eq!(std::fs::read_to_string(&claude).unwrap(), current);

        std::fs::remove_dir_all(&dir).ok();
    }

    /// D0388 pair for issue613 (GH#87), chosen before the tree was read. The door tolerated a new
    /// obligation record; the run then failed `guard issues` on it and `git clean` deleted it.
    /// KNOWN-POSITIVE: a guard output whose only failing lines are the `[guard:issues] FAIL` header and
    /// `ERROR obligationabc: untriaged` is green with the discount named; the clean arguments carry
    /// the record as a `-e` exclusion. KNOWN-NEGATIVE: one more `ERROR issue999: untriaged` stays red;
    /// an empty tolerated list yields today's clean arguments exactly.
    #[test]
    fn a_record_tolerated_at_the_door_is_discounted_by_the_gate_and_kept_by_the_clean() {
        let record = "package Obligations {\n    part obligationabc : Issue {\n        :>> id = \"x\";\n    }\n}\n";
        let parts = declared_parts(record);
        assert_eq!(parts, vec!["obligationabc".to_string()]);

        let green = "[guard:ownership] PASS — 3 scanned, 0 warning(s), 0 violation(s)\n  ERROR obligationabc: untriaged — no #Resolves edge (D0077; link a resolving action or Decision)\n[guard:issues] FAIL — 12 scanned, 0 warning(s), 1 violation(s)\n[guard] FAILED — 0 warning(s) across 0 guard(s)\n";
        let discounted = discount_tolerated(green, &parts).expect("every failing line is the tolerated record's or a header");
        assert_eq!(discounted.len(), 1, "{discounted:?}");
        assert!(discounted[0].contains("obligationabc: untriaged"), "{discounted:?}");

        let red = format!("{green}  ERROR issue999: untriaged — no #Resolves edge (D0077)\n");
        let stays = discount_tolerated(&red, &parts).expect_err("a violation on another subject is the run's red");
        assert_eq!(stays.len(), 1, "{stays:?}");
        assert!(stays[0].contains("issue999"), "{stays:?}");
        assert!(discount_tolerated(green, &[]).is_err(), "nothing tolerated, nothing discounted");

        let keep = tolerated_paths(&["?? .tracking/obligations/red-yield-abc.sysml".to_string()]);
        assert_eq!(keep, vec![".tracking/obligations/red-yield-abc.sysml".to_string()]);
        let args = clean_args(&keep);
        assert_eq!(args, ["clean", "-fdq", "-e", ".tracking/obligations/red-yield-abc.sysml", "--", ".engine", ".tracking", ".claude"].map(str::to_string).to_vec());
        assert_eq!(clean_args(&[]), ["clean", "-fdq", "--", ".engine", ".tracking", ".claude"].map(str::to_string).to_vec());
    }

    /// D0388 pair for issue610 (GH#90), chosen before the tree was read. The resync ADDED the engine's
    /// own parser-coverage count into a first-time adopter as that project's ratchet. KNOWN-POSITIVE:
    /// a project with no `contracts/parser-coverage-baseline.toml` plans no such file.
    /// KNOWN-NEGATIVE: a project-authored one keeps its bytes, and `activation.toml` - a project-owned
    /// contract the resync still seeds - is planned when absent.
    #[test]
    fn a_ratchet_over_the_projects_corpus_is_never_seeded_and_an_authored_one_is_kept() {
        let dir = std::env::temp_dir().join(format!("keel-migrate-ratchet-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".engine").join("contracts")).unwrap();
        let engine = &keel_schema::embedded::ENGINE_DIR;
        assert!(engine.get_file("contracts/parser-coverage-baseline.toml").is_some(), "the engine ships the ratchet");
        assert!(engine.get_file("contracts/activation.toml").is_some(), "the engine ships the activation manifest");

        let plan = step_engine_resync(&dir, engine);
        let planned = |name: &str| plan.files.iter().any(|f| f.path.ends_with(Path::new("contracts").join(name)));
        assert!(!planned("parser-coverage-baseline.toml"), "the engine's count is not this project's ratchet");
        assert!(planned("activation.toml"), "a project-owned contract with a sane default is still seeded");

        let own = "# adopted at our own count\n[baseline]\nskipped = 3\n";
        std::fs::write(dir.join(".engine/contracts/parser-coverage-baseline.toml"), own).unwrap();
        let again = step_engine_resync(&dir, engine);
        assert!(!again.files.iter().any(|f| f.path.ends_with(Path::new("contracts").join("parser-coverage-baseline.toml"))), "an authored ratchet is the project's");
        assert_eq!(std::fs::read_to_string(dir.join(".engine/contracts/parser-coverage-baseline.toml")).unwrap(), own);
        std::fs::remove_dir_all(&dir).ok();
    }
}
