//! The `keel` verbs this member owns (D0479, sprint 750): `cmd_audit`, `cmd_guard`, `cmd_hardening`, `cmd_assured`, `cmd_activation`.
//!
//! Moved out of `keel-cli/src/main.rs` by `scripts/extract_verbs.py`: the binary is argument dispatch, and a
//! verb body lives with the member whose reach it needs (`python scripts/verb_homes.py --by-member`).
//! Each `cmd_*` takes the arguments after its verb and returns the process exit code.
//
// These were the binary's private items, written under the same lint set; only the lints that fire on
// PUBLIC visibility are new here, and they ask for ceremony a verb body does not owe: its exit code goes
// straight back to `main`, and its `Err` and panics are the ones the body already documents in place.
#![allow(clippy::must_use_candidate, clippy::missing_errors_doc, clippy::missing_panics_doc)]

use keel_args::{refuse_flag_as_path, root_arg};
use keel_git::git_verbs::resolve_guard_root;
use keel_git::projects::{engine_version_skew, find_repo_root};
use keel_model::model_verbs::{recorded_active, switch_viewpoint, write_activation};
use keel_write::ledger::ledger_gate;
use std::path::{Path, PathBuf};

/// D0452: the three audits are sub-verbs of `audit`, resolved before a positional is read as a root.
///
/// issue281: `history` and `adherence` read a MODEL, so they must not answer over nothing — they take
/// their root from `find_repo_root`, the repository-scoped resolver `sync`/`land` use, which carries
/// no project precondition. Found by sweeping every command at a workspace root rather than by
/// trusting that one chokepoint covered them all: nine refused, one still exited 0.
/// D0323 / issue374: `ci-runs` is the external-fact gate - a ci-run receipt is checked against the run itself.
pub fn audit_subverb(args: &[String]) -> Option<i32> {
    let rest = args.get(1..).unwrap_or(&[]);
    let repo = || find_repo_root().unwrap_or_else(|| PathBuf::from("."));
    Some(match args.first().map(String::as_str)? {
        "history" => crate::history::cmd(rest, &repo()),
        "adherence" => crate::adherence::cmd(rest, &repo()),
        "ci-runs" => refuse_flag_as_path(rest.first(), "audit ci-runs").unwrap_or_else(|| {
            let root = rest.first().filter(|a| !a.starts_with('-')).map_or_else(repo, PathBuf::from);
            keel_github::ci_runs::cmd(rest, &root)
        }),
        _ => return None,
    })
}


pub fn cmd_audit(args: &[String]) -> i32 {
    if let Some(code) = audit_subverb(args) {
        return code;
    }
    let root = match root_arg(args, "keel audit [ROOT] | keel audit history|adherence|ci-runs ...", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match keel_model::algo::audit(&root) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(e) => {
            eprintln!("audit error: {e}");
            1
        }
    }
}


/// A string that names a runnable guard (an enforced one, or a runnable-only diagnostic).
pub fn is_guard_name(s: &str) -> bool {
    crate::GUARD_NAMES.contains(&s) || matches!(s, "assured" | "critique" | "critique-rigor" | "defect-guard-coverage")
}


/// Classify `keel gate guard` args into `(guard name to run, root arg)`. A first arg that is a known guard
/// name runs THAT guard; `all`, no arg, or a non-name first arg (a ROOT path like `.` or a dir) runs
/// ALL guards on that root. This is what lets `keel gate guard <ROOT>` work like `keel gate validate <ROOT>`.
pub fn classify_guard_args(args: &[String]) -> (Option<&str>, Option<&str>) {
    match args.first().map(String::as_str) {
        None => (None, None),
        Some("all") => (None, args.get(1).map(String::as_str)),
        Some(a) if is_guard_name(a) => (Some(a), args.get(1).map(String::as_str)),
        Some(a) => (None, Some(a)), // a bare ROOT, not a guard name
    }
}


pub fn cmd_guard(args: &[String]) -> i32 {
    // `keel gate guard` / `guard [ROOT]` / `guard all [ROOT]` → run all; `guard <name> [ROOT]` → run one.
    let bare: Vec<String> = args.iter().filter(|a| *a != "--no-receipt").cloned().collect();
    let (name, root_arg) = classify_guard_args(&bare);
    // GH#14: a mistyped flag must not become the ROOT and turn every guard green over nothing.
    if let Some(code) = refuse_flag_as_path(root_arg.map(String::from).as_ref(), "gate guard") {
        return code;
    }
    let Some(root) = resolve_guard_root(root_arg.map(String::from).as_ref()) else {
        eprintln!("error: no .engine/ directory found. usage: keel gate guard [<name>] [ROOT]");
        return 2;
    };
    if let Some(w) = engine_version_skew(&root) {
        eprintln!("{w}");
        eprintln!("guard REFUSED under engine-version skew (D0251). Run the pinned version, or `keel migrate`.");
        return 2;
    }
    let Some(name) = name else {
        // THE GUARD RECEIPT (dcGateAnswersFromItsReceipt): an equal key means the same inputs, and the
        // stored reports are printed as they were; one line names the receipt's age. `--no-receipt`
        // forces the run. A red run deletes the receipt so nothing green is ever answered over it.
        let guard_started = std::time::Instant::now();
        let receipt_key = if crate::receipt::forced(args) { None } else { crate::receipt::key(&root) };
        let receipt = receipt_key
            .as_ref()
            .and_then(|k| crate::receipt::read(&root, k))
            .filter(|r| r.covers_all(&[crate::receipt::GUARDS]));
        let (reports, durations, from_receipt) = if let Some(r) = receipt {
            let line = r.line("[guard]");
            (r.guards, Vec::new(), Some(line))
        } else {
            let (reports, durations) = crate::run_all_timed(&root);
            (reports, durations, None)
        };
        // D0278: a control with a KNOWN defect says so beside its own verdict. Printed here rather
        // than inside `GuardReport::print` because the runner is what holds the root — and because
        // the note belongs to the reading, not to the report: the moment someone needs to know a
        // green is unreliable is the moment they are looking at it.
        let defects = keel_schema::control_defects::load(&root);
        let mut all_ok = true;
        for r in &reports {
            r.print();
            if let Some(d) = defects.get(r.name) {
                println!("{}", keel_json::color::defect(&d.note(r.name)));
            }
            all_ok &= r.ok();
        }
        // issue244: the verdict used to be the bare word ALL PASS while 80 warnings scrolled above
        // it, including a live recorded-release contradiction that had shipped unread. Detected-but-
        // unread is this repo's highest-frequency drift mechanism, and it grows with every control
        // added — so the SUBTRACTIVE fix is to state the warning population where the verdict is
        // read, rather than build another detector for what was already detected.
        // issue404 / D0413: the population is stated in two classes - the actionable warnings, which
        // are the set a reader is asked to read, and the counted-history lines, which are not.
        let tail = crate::warning_population(&reports);
        println!("[guard] {}{tail}", if all_ok { keel_json::color::pass("ALL PASS") } else { keel_json::color::fail("FAILED") });
        let failing: Vec<String> = reports.iter().filter(|r| !r.ok()).map(|r| r.name.to_string()).collect();
        ledger_gate(&root, "guard", &failing, guard_started.elapsed().as_millis());
        // D0414 / issue429: the set's wall clock is bounded below by its longest guard; name it.
        if keel_perf::perf::enabled() {
            if let Some((name, ms)) = crate::critical_path() {
                println!("[guard] critical path: {name} {ms} ms - the parallel set's wall clock is bounded below by it");
            }
        }
        if let Some(line) = from_receipt {
            println!("{line}");
        } else if let Some(k) = &receipt_key {
            if all_ok {
                let _ = crate::receipt::record_green(&root, k, &[crate::receipt::GUARDS], &reports, &durations);
            } else {
                crate::receipt::delete(&root);
            }
        }
        return i32::from(!all_ok);
    };
    let Some(report) = crate::run_one(name, &root) else {
        eprintln!(
            "unknown guard '{name}' (enforced: {} | runnable diagnostics: assured, critique, critique-rigor, defect-guard-coverage)",
            crate::GUARD_NAMES.join(", ")
        );
        return 2;
    };
    report.print();
    // Asking for ONE guard by name is a diagnostic, so the check still RUNS and its findings are still
    // shown — but the exit code must agree with the enforced gate (D0138). Without this, `keel gate guard
    // issues` exits 1 on a project that never adopted issue-resolution while `keel gate guard` exits 0, and a
    // script wired to the single-guard form would block on a control the project deliberately does not
    // enforce.
    if let keel_model::activation::GuardState::Inactive(p) =
        keel_model::activation::Activation::load(&root).guard_state(name)
    {
        println!(
            "[guard:{name}] NOT ACTIVE — process `{p}` is not in this project's active set, so the findings above are informational and do NOT block (`keel activate {p}` to enforce them)"
        );
        return 0;
    }
    i32::from(!report.ok())
}


pub fn cmd_hardening(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel hardening [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match crate::hardening::hardening(&root) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(e) => {
            eprintln!("keel hardening: {e}");
            1
        }
    }
}


pub fn cmd_assured(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel gate assured [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match crate::assured_report(&root) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(e) => {
            eprintln!("assured error: {e}");
            1
        }
    }
}


/// D0396 / D0375 option C: accept a marker Decision at record time when a plan the human signed
/// THEMSELVES names its step, or leave it proposed naming the clause that failed. Returns the process
/// exit code; the caller returns it directly.
#[allow(clippy::too_many_arguments)] // the record context this runs inside
pub fn try_plan_cover(root: &Path, path: &str, dname: &str, nnnn: &str, date: &str, author: &str, plan_id: &str, step: &str) -> i32 {
    match crate::plan_cover::assess(root, dname, plan_id, step) {
        crate::plan_cover::Cover::Covered { judge } => {
            let sha = keel_git::gitx::git().arg("-C").arg(root).args(["rev-parse", "--short", "HEAD"]).output().ok().and_then(|o| String::from_utf8(o.stdout).ok()).map(|s| s.trim().to_owned()).unwrap_or_default();
            let cover_note = crate::plan_cover::note(plan_id, step, &judge);
            match keel_write::write::accept_decision(std::path::Path::new(path), dname, &sha, date, &judge, author, &cover_note) {
                Ok(_) => {
                    println!("accepted D{nnnn} at record time - PLAN-COVERED by {plan_id} step '{step}' (D0396); the human signed {plan_id}, and this enumerated step does not re-ask. Judge: {judge}.");
                    0
                }
                Err(e) => {
                    eprintln!("D{nnnn}: plan {plan_id} covers step '{step}', but recording the acceptance failed: {e}");
                    1
                }
            }
        }
        crate::plan_cover::Cover::Refused { clause, detail } => {
            println!("D{nnnn} stays proposed - plan cover clause ({clause}) does not hold: {detail}. Accept with the human's own word (D0289), the console, or their terminal; or fix the plan / step name and re-record.");
            0
        }
    }
}


/// `keel activate <process>` / `keel deactivate <process>` / `keel activation` (D0138).
///
/// The subtle case is activating when NO manifest exists. Absence means "everything is active", so
/// writing a manifest containing only the named process would silently DEACTIVATE every other control —
/// the opposite of what the caller asked for. So a first write MATERIALISES the current effective state
/// (all declared units) and then applies the change, and says that it did.
pub fn cmd_activation(mode: &str, args: &[String]) -> i32 {
    // issue179: `activate`/`deactivate` take a process or viewpoint NAME; a flag would be looked up as
    // one and reported as unknown, which reads as "no such process" rather than "unrecognised flag".
    if let Some(f) = args.first().filter(|a| a.starts_with('-')) {
        eprintln!("error: `{f}` looks like a flag, not a process or viewpoint name (issue179).");
        return 2;
    }
    let (target, root_arg) = match mode {
        "activation" => (None, args.first()),
        _ => (args.first(), args.get(1)),
    };
    let Some(root) = resolve_guard_root(root_arg.map(String::from).as_ref()) else {
        eprintln!("error: no .engine/ directory found. usage: keel {mode} [<process>] [ROOT]");
        return 2;
    };
    let act = keel_model::activation::Activation::load(&root);

    if mode == "activation" {
        println!("declared manifest: {}", if act.is_declared() { "yes" } else { "no — everything present is active" });
        // EVERY declared process, not only the switchable ones (issue149): listing 6 of 18 with no note
        // that the rest exist reads as "this project has 6 processes".
        for p in keel_model::activation::declared_processes(&root) {
            match act.unit(&p) {
                Some(unit) => println!(
"  [{}] {p}  ({} guard(s))",
                    if act.is_process_active(&p) { "active  " } else { "INACTIVE" },
                    unit.guards.len()
                ),
                None => println!("  [always  ] {p}  (asserts no guard — nothing to switch off)"),
            }
        }
        // VIEWPOINTS (D0164), listed alongside processes because the human's direction was that they be
        // switchable "just like processes" - and a switch whose state nobody can see is not a switch.
        let vps = keel_view::view::declared_viewpoints(&root).unwrap_or_default();
        println!("
viewpoints ({} declared):", vps.len());
        for vp in &vps {
            println!(
"  [{}] {}",
                if act.is_viewpoint_active(&vp.name) { "active  " } else { "INACTIVE" },
                vp.name
            );
        }
        println!("\ncore guards (never deactivatable):");
        for g in crate::GUARD_NAMES {
            if act.guard_state(g) == keel_model::activation::GuardState::Core {
                println!("  {g}");
            }
        }
        return 0;
    }

    let Some(target) = target else {
        eprintln!("usage: keel {mode} <process> [ROOT]");
        return 2;
    };
    // A name may be a process or a VIEWPOINT (D0164). Resolve in that order and refuse an ambiguous name
    // rather than guessing: switching off the wrong thing is worse than asking again.
    let vp_names: Vec<String> =
        keel_view::view::declared_viewpoints(&root).unwrap_or_default().into_iter().map(|v| v.name).collect();
    let vp_active: Vec<String> = vp_names.iter().filter(|v| act.is_viewpoint_active(v)).cloned().collect();
    if vp_names.iter().any(|v| v == target) && act.unit(target).is_some() {
        eprintln!("error: `{target}` names both a process unit and a viewpoint - rename one; keel will not guess which to {mode}");
        return 2;
    }
    if vp_names.iter().any(|v| v == target) {
        return switch_viewpoint(&root, &act, mode, target, &vp_names, vp_active);
    }
    if act.unit(target).is_none() {
        // Say WHICH of the two cases this is (issue149). "Not a declared process unit" was true and
        // read as "no such process", which is a different and wrong answer.
        if keel_model::activation::declared_processes(&root).iter().any(|p| p == target) {
            eprintln!(
                // issue242: this text used to end "or give it an `assert constraint` so it becomes
                // switchable" -- advising the exact edit that CAPTURES a core guard and disarms it.
                // Adding an assert is a process-definition change under the keystone and is now
                // caught by audit-adherence as a Core -> Active/Inactive weakening; the CLI must not
                // recommend it as a convenience.
                "error: `{target}` is a declared process but asserts no guard, so there is nothing for {mode} to switch. Activation governs GUARDS. To stop running this process, remove the facts it authors -- a process whose inputs are absent produces nothing. Do NOT add an `assert constraint` to make it switchable: claiming a guard converts it from CORE to that process's switchable property, which DISARMS it, and audit-adherence gates that transition."
            );
        } else {
            eprintln!(
                "error: `{target}` is not a declared process. Declared: {}",
                keel_model::activation::declared_processes(&root).join(", ")
            );
        }
        return 2;
    }

    let materialising = !act.is_declared();
    // The RECORDED list is the base when a manifest exists (issue293) — rebuilding it from
    // `unit_names()` drops every `[always]` process, and omission from a present list reads as
    // INACTIVE. Only a project with no manifest gets the computed effective state.
    let mut set: Vec<String> = recorded_active(&root, "processes")
        .unwrap_or_else(|| act.unit_names().into_iter().filter(|p| act.is_process_active(p)).collect());
    match mode {
        "activate" => {
            if !set.iter().any(|p| p == target) {
                set.push(target.clone());
            }
        }
        _ => set.retain(|p| p != target),
    }
    set.sort();
    if let Err(e) = write_activation(&root, &set, &vp_active) {
        eprintln!("error writing .engine/contracts/activation.toml: {e}");
        return 1;
    }
    if materialising {
        println!(
            "No manifest existed (everything was active), so one was written with the current effective\nstate before applying the change — behaviour is otherwise unchanged."
        );
    }
    println!("{mode}d `{target}`. Active: {}", if set.is_empty() { "(none)".to_string() } else { set.join(", ") });
    // D0348 / GH#49: a deactivated process's skill is deployed saying so, so the activation set and
    // the deployed surface are one fact - regenerate the surface here, where the fact changed, rather
    // than leave claude-surface-drift to report the stale skill at the next gate.
    if root.join(".claude").is_dir() {
        match keel_write::claude_surface::sync_claude(&root, false) {
            Ok(r) => println!("claude surface regenerated: {}/{} skill(s) - a deactivated process's skill now opens with its INACTIVE state (D0348)", r.skills_written, r.registry_count),
            Err(e) => eprintln!("claude surface NOT regenerated ({e}) - run `keel sync-claude` so the deployed skills follow the active set"),
        }
    }
    println!("Read it back: keel activation | keel gate guard");
    0
}
