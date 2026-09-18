//! A WORKSPACE: one git repository holding one or more keel projects (D0234/issue270).
//!
//! WHY (measured, not supposed). Asked whether several small projects could share a repo, I built the
//! arrangement and ran it. Three things work already: root resolution walks up to `.engine/`, drift is
//! file-scoped through each project's own `deliverable-manifest.txt`, and the write lock sits beside
//! each model root. Four things break, and they are exactly the ones that matter:
//!
//! 1. **Decision numbering collides** — two fresh projects both get `d0001`.
//! 2. **The channel cannot tell them apart** — its marker is `keel-decision: d0001` and its lookup is
//!    repo-scoped in GitHub, so the second project's issue never opens and `reject d0001` is ambiguous.
//! 3. **The gate cannot cover them** — git allows ONE `core.hooksPath` per repository, so at most one
//!    project is gated; and `keel gate validate` at the parent printed "0 tracking file(s) validated clean"
//!    and exited 0, so a hook there would gate NOTHING while reporting success (issue269).
//! 4. **`land` gates one root and pushes the whole repo** — the other projects ride out ungated.
//!
//! DISCOVERED, NOT DECLARED. A project is a directory holding both `.engine/` and `.tracking/`. A
//! manifest at the repo root would be a second place to keep the list true, and git already knows —
//! the same reasoning `/api/projects` records for the console.
//!
//! Measured across the NINE keel repositories on this machine: exactly one project each, no nested
//! false positives. An earlier version of this note claimed the same thing about "both real
//! repositories", which undercounted by seven and was VACUOUS besides — discovery short-circuited at
//! a root project, so it examined no subdirectory in any of them and could not have found a false
//! positive if one existed. Discovery now descends (`discover`), so the claim is a measurement.
//!
//! Discovery itself is `keel_git::projects` since sprint 740; this module is the workspace GATE.
use std::path::{Path, PathBuf};

// Project discovery is keel-git's (`keel_git::projects`, sprint 740, D0479); the gate below keeps using
// the names, and every `keel_process::workspace::` / `keel_cli::workspace::` path resolves unchanged.
pub use keel_git::projects::{canon, discover, engine_version_skew, find_repo_root, is_project, missing_project_dirs, require_project, Workspace};

/// Staged paths that are KEYSTONE events and that no project gate covers (issue276).
///
/// # The hole this closes
///
/// In a workspace the repo-root enforcement surface — the ONE `.githooks/pre-commit` that gates every
/// project, and `.github/workflows/` — belongs to no project, and the keystone lock matches
/// project-relative paths. So the lock could never fire on it. Verified: the single hook that gates
/// every project, replaced with a two-line `exit 0` and staged alone with no Decision, and
/// `gate --workspace` reported 2 projects gated clean, exit 0 — after PRINTING a line that named the
/// unowned file. A gate that reports what it is not checking and passes anyway is the false-green
/// class, stated out loud.
///
/// Two rules, because the second is not a special case of the first:
///
/// 1. An unowned staged path that `guards::is_locked_path` claims is a keystone event. At a workspace
///    root the staged paths are repo-relative, so `.githooks/**` and `.github/workflows/**` match the
///    same predicate the per-project lock uses.
/// 2. Any staged DELETION of a path under an `.engine/` directory, at any depth, owned or not. This
///    is not covered by rule 1: deleting a whole project makes its directory stop satisfying
///    `is_project`, so every one of its paths becomes UNOWNED — which is how staging the deletion of
///    an entire project, 445 paths including every process definition and its guards, passed the gate
///    silently. A project's engine files are owned while the project exists and unowned at exactly
///    the moment that matters.
fn unowned_keystone_events(unowned: &[String], deleted: &[String]) -> Vec<String> {
    let mut events: Vec<String> = unowned
        .iter()
        .filter(|p| keel_guards::is_locked_path(p))
        .map(|p| format!("{p} (repo-root enforcement surface — owned by no project)"))
        .collect();
    for d in deleted {
        let slashed = d.replace('\\', "/");
        if slashed.starts_with(".engine/") || slashed.contains("/.engine/") {
            events.push(format!("{d} (DELETED engine path — removing a control is a keystone event)"));
        }
    }
    events.sort();
    events.dedup();
    events
}

/// `keel projects [ROOT] [--json]` — every project in this workspace, and which one you are in.
#[must_use]
pub fn cmd(args: &[String]) -> i32 {
    let here = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .map_or_else(|| std::path::PathBuf::from("."), std::path::PathBuf::from);
    let here = here.canonicalize().unwrap_or(here);
    let ws = discover(&here);
    let current = ws.owner_of(&here);

    if args.iter().any(|a| a == "--json") {
        let esc = |s: &str| s.replace('\\', "/").replace('"', "'");
        let rows: Vec<String> = ws
            .projects
            .iter()
            .map(|p| {
                format!(
                    "{{\"label\":\"{}\",\"root\":\"{}\",\"current\":{}}}",
                    esc(&ws.label(p)),
                    esc(&p.display().to_string()),
                    current.is_some_and(|c| c == p)
                )
            })
            .collect();
        println!(
            "{{\"workspaceRoot\":\"{}\",\"multiProject\":{},\"projects\":[{}]}}",
            esc(&ws.root.display().to_string()),
            ws.is_multi(),
            rows.join(",")
        );
        return 0;
    }

    println!("workspace: {}", ws.root.display());
    if ws.projects.is_empty() {
        println!("  NO keel project found in this repository (a project holds both .engine/ and .tracking/).");
        println!("  `keel init <dir>` scaffolds one. Reported rather than passed over in silence: an");
        println!("  absent project is why a gate can pass while checking nothing (issue269).");
        return 1;
    }
    for p in &ws.projects {
        let here = if current.is_some_and(|c| c == p) { " <- you are here" } else { "" };
        println!("  {}{here}", ws.label(p));
    }
    if ws.is_multi() {
        println!();
        println!("{} projects share this repository, so three things are workspace-scoped:", ws.projects.len());
        println!("  - the COMMIT GATE: git allows one core.hooksPath per repo, so the hook lives at the");
        println!("    repo root and runs `keel gate --workspace`; a per-project hook can only cover one.");
        println!("  - `keel land`: a push carries the whole repo, so every project is gated before it.");
        println!("  - DECISION identity: `dNNNN` is unique per project, so the channel qualifies it with");
        println!("    the project label - `alpha/d0001` - or two projects' first decisions collide.");
    }
    0
}

/// The enforced gate for ONE project: validate, every guard, and the DECLARED rules — returned as a
/// problem list rather than printed, so every caller holds it to the identical bar (issue282).
///
/// # Why this is the only body
///
/// There were THREE, with three different bars. This one ran validate + guards + declared rules; the
/// scaffolded pre-commit hook ran validate + guard + `rules --enforce`; and the `sync`/`land` gate ran
/// validate + guards and NO rules at all — that file contained zero occurrences of the word, while its
/// own doc comment claimed it was "deliberately the SAME entry point the commit hook uses".
///
/// The declared-rule layer is precisely the one a downstream project uses to add a BLOCKING control
/// without writing Rust. So such a project had its only control enforced at commit and unenforced on
/// the merged tree — which is the exact class the merged-tree gate exists to catch, since a merge can
/// violate an edge rule neither side violated alone.
///
/// Returning the list instead of printing is what makes one body serve both callers: the commit gate
/// wants it printed per project as it goes, and `sync`/`land` want it aggregated across the workspace
/// with a project tag. Formatting is the caller's; the BAR is not.
#[must_use]
pub fn gate_problems(project: &Path, tag: &str) -> Vec<String> {
    gate_outcome(project, tag).0
}

/// [`gate_problems`] plus, when the verdict came from the guard receipt, the one line that says so.
///
/// THE GUARD RECEIPT (dcGateAnswersFromItsReceipt): a project whose tree, binary and `.keel/` inputs
/// equal the ones a green full run judged is answered from that run's receipt; a green run here writes
/// one covering validate, guards and rules; a red run deletes it. The pin check is not in the key and
/// runs every time - it is the one problem a binary can have over an unchanged tree.
#[must_use]
pub fn gate_outcome(project: &Path, tag: &str) -> (Vec<String>, Option<String>) {
    let mut problems = Vec::new();
    // THE PIN BITES HERE for verdicts, as it does in `with_file_lock` for writes (D0251 clause C).
    // One body means one check covers gate, sync, land and the pre-commit hook identically — and
    // workspace coherence (srWorkspacePinIsCoherent) falls out: projects with DISAGREEING pins
    // cannot both match one binary, so the mismatched one refuses under its own tag, naming it.
    if let Some((declared, binary)) = keel_write::pin_skew(&project.join(".tracking").join("x")) {
        problems.push(format!(
            "{tag}PIN: this project declares engine {declared} and this binary is {binary} — a verdict from an undeclared engine is not this project's verdict (D0251/srProjectPinsItsEngine). Run the pinned version, or `keel migrate` to bring the tree to this one"
        ));
    }
    let receipt_key = if keel_guards::receipt::forced(&[]) { None } else { keel_guards::receipt::key(project) };
    if let Some(r) = receipt_key.as_ref().and_then(|k| keel_guards::receipt::read(project, k)) {
        if r.covers_all(&keel_guards::receipt::ALL_LAYERS) {
            return (problems, Some(r.line(&format!("{tag}gate"))));
        }
    }
    let report = keel_model::validate::validate_root(project);
    for (path, d) in &report.diagnostics {
        problems.push(format!("{tag}ERROR {}:{} — {}", path.display(), d.line, d.message));
    }
    for e in &report.errors {
        problems.push(format!("{tag}PARSE {} — {}", e.file.display(), e.message));
    }
    // Guards and rules are still evaluated when validate failed, because a caller aggregating across
    // a workspace wants the whole picture in one run rather than one layer per invocation.
    let (reports, durations) = keel_guards::run_all_timed(project);
    for g in &reports {
        for v in &g.violations {
            problems.push(format!("{tag}VIOLATION {}: {v}", g.name));
        }
    }
    for v in declared_rule_violations(project) {
        problems.push(format!("{tag}RULE {v}"));
    }
    if let Some(k) = &receipt_key {
        if problems.is_empty() {
            let _ = keel_guards::receipt::record_green(project, k, &keel_guards::receipt::ALL_LAYERS, &reports, &durations);
        } else {
            keel_guards::receipt::delete(project);
        }
    }
    (problems, None)
}

/// Blocking violations of this project's DECLARED rules (`.engine/rules/`), warnings excluded.
fn declared_rule_violations(project: &Path) -> Vec<String> {
    keel_view::view::check(project)
        .ok()
        .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
        .and_then(|v| {
            v.get("rules").and_then(|r| r.as_array()).map(|rules| {
                rules
                    .iter()
                    .filter(|r| r.get("severity").and_then(|s| s.as_str()) != Some("warning"))
                    .flat_map(|r| {
                        let name = r.get("rule").and_then(|n| n.as_str()).unwrap_or("rule").to_string();
                        r.get("violations")
                            .and_then(|x| x.as_array())
                            .cloned()
                            .unwrap_or_default()
                            .into_iter()
                            .map(move |v| format!("{name}: {v}"))
                    })
                    .collect::<Vec<String>>()
            })
        })
        .unwrap_or_default()
}

/// The commit gate's per-project step: run [`gate_problems`], print what it found, and name the layer
/// that failed. One body, printed here.
fn gate_one(p: &Path, label: &str) -> Result<(), String> {
    println!("gate [{label}] validate + guard + rules");
    let (problems, from_receipt) = gate_outcome(p, "");
    if problems.is_empty() {
        match from_receipt {
            Some(line) => println!("  {line}"),
            None => println!("  clean ({} file(s))", keel_model::validate::validate_root(p).validated),
        }
        return Ok(());
    }
    for v in problems.iter().take(10) {
        println!("  {v}");
    }
    if problems.len() > 10 {
        println!("  ... and {} more", problems.len() - 10);
    }
    // Name the FIRST layer that failed, in the order the gate applies them — that is what the caller
    // reports, and it tells the reader which tool to reach for.
    let layer = if problems.iter().any(|p| p.contains("ERROR ") || p.contains("PARSE ")) {
        "validate"
    } else if problems.iter().any(|p| p.contains("VIOLATION ")) {
        "guard"
    } else {
        "rules"
    };
    Err(format!("{label} ({layer})"))
}

/// `keel gate --workspace [ROOT]` — the COMMIT gate for a repo holding several projects.
///
/// git allows one `core.hooksPath` per repository, so a per-project hook can only ever gate one
/// project; the hook has to live at the repo root and gate every project the commit touches. That is
/// what this is: staged files are mapped to their owning projects, and each such project gets the full
/// gate. Projects the commit does not touch are NAMED as skipped rather than passed over quietly —
/// silence is how a gate that checked nothing comes to look like a gate that passed.
///
/// FAILS LOUDLY when it cannot run (K2): no projects at all is a non-zero exit, not a clean tree.
#[must_use]
pub fn gate_cmd(args: &[String]) -> i32 {
    let here = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .map_or_else(|| std::path::PathBuf::from("."), std::path::PathBuf::from);
    let here = here.canonicalize().unwrap_or(here);
    let ws = discover(&here);

    if ws.projects.is_empty() {
        eprintln!("gate: NO keel project in {} — a gate that cannot run must not pass (K2).", ws.root.display());
        return 1;
    }

    // Which projects does this commit actually touch? Staged paths are repo-relative.
    let staged = keel_git::gitx::git()
        .arg("-C")
        .arg(&ws.root)
        .args(["diff", "--cached", "--name-only"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    let mut touched: Vec<PathBuf> = Vec::new();
    let mut workspace_level: Vec<String> = Vec::new();
    for line in staged.lines().filter(|l| !l.trim().is_empty()) {
        let abs = ws.root.join(line);
        match ws.owner_of(&abs) {
            Some(p) if !touched.contains(p) => touched.push(p.clone()),
            Some(_) => {}
            None => workspace_level.push(line.to_string()),
        }
    }
    // Nothing staged (or no git) means gate EVERYTHING: a caller asking for the workspace gate
    // without a commit in flight wants the whole answer, and guessing narrower would under-report.
    let gated: Vec<PathBuf> = if touched.is_empty() { ws.projects.clone() } else { touched };

    let mut failed: Vec<String> = Vec::new();
    for p in &gated {
        let label = ws.label(p);
        if let Err(which) = gate_one(p, &label) {
            failed.push(which);
        }
    }
    // Say what was NOT gated. A skipped project is a fact the reader needs, not noise.
    let skipped: Vec<String> =
        ws.projects.iter().filter(|p| !gated.contains(p)).map(|p| ws.label(p)).collect();
    if !skipped.is_empty() {
        println!("gate: {} project(s) untouched by this commit, so NOT gated: {}", skipped.len(), skipped.join(", "));
    }
    if !workspace_level.is_empty() {
        println!(
            "gate: {} staged file(s) belong to no project (workspace-level), so no project gate covers them: {}",
            workspace_level.len(),
            workspace_level.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
        );
    }
    // issue276: the keystone lock, applied to what no project gate covers. Printing the unowned files
    // and then exiting 0 was the whole defect — this is the line that makes the report a verdict.
    let deleted: Vec<String> = keel_git::gitx::git()
        .arg("-C")
        .arg(&ws.root)
        .args(["-c", "core.quotePath=false", "diff", "--cached", "--name-only", "--diff-filter=D"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().map(str::to_owned).collect())
        .unwrap_or_default();
    let events = unowned_keystone_events(&workspace_level, &deleted);
    if !events.is_empty() && !keel_guards::staged_marked_decision(&ws.root) {
        eprintln!("gate: KEYSTONE — {} staged change(s) to the locked surface carry no co-committed", events.len());
        eprintln!("  Decision bearing #ProspectiveChange or #SafetyChange (D0070/D0209 clause 2):");
        for e in events.iter().take(10) {
            eprintln!("    {e}");
        }
        if events.len() > 10 {
            eprintln!("    ... and {} more", events.len() - 10);
        }
        failed.push("workspace keystone".to_string());
    }
    if failed.is_empty() {
        println!("gate: {} project(s) gated clean.", gated.len());
        0
    } else {
        eprintln!("gate: FAILED in {}: {}", failed.len(), failed.join(", "));
        1
    }
}

#[cfg(test)]
mod tests {
    use super::unowned_keystone_events;

    /// issue276: the two rules that make the unowned surface a verdict rather than a printed note.
    #[test]
    fn the_unowned_surface_is_under_the_keystone_lock() {
        // Rule 1: the ONE repo-root hook that gates every project, and the root workflows dir.
        let unowned = vec![
            ".githooks/pre-commit".to_string(),
            ".github/workflows/keel-gate.yml".to_string(),
            "README.md".to_string(),          // workspace-level but NOT locked
            "scripts/build.sh".to_string(),   // ditto
        ];
        let events = unowned_keystone_events(&unowned, &[]);
        assert_eq!(events.len(), 2, "exactly the locked pair, got {events:?}");
        assert!(events.iter().any(|e| e.contains(".githooks/pre-commit")), "{events:?}");
        assert!(events.iter().any(|e| e.contains("keel-gate.yml")), "{events:?}");

        // Rule 2 is NOT a special case of rule 1: deleting a project makes its directory stop being a
        // project, so its engine paths become unowned at exactly the moment they are removed. They do
        // not match the root-relative locked predicate, which is how 445 staged deletions passed.
        let deleted = vec![
            "beta/.engine/processes/delivery.sysml".to_string(),
            ".engine/guards/x.sysml".to_string(),
            "beta/.tracking/backlog.sysml".to_string(), // tracking is instance data, not a control
            "docs/readme.md".to_string(),
        ];
        let events = unowned_keystone_events(&[], &deleted);
        assert_eq!(events.len(), 2, "both engine deletions, at either depth: {events:?}");
        assert!(events.iter().all(|e| e.contains("DELETED engine path")), "{events:?}");

        // A commit touching neither is not a keystone event, or every commit would need a signature.
        assert!(unowned_keystone_events(&["README.md".to_string()], &["docs/x.md".to_string()]).is_empty());
    }
}
