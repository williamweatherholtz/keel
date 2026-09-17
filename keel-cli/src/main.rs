//! `keel` — CLI entry point.
//!
//! Subcommands:
//!   `validate [ROOT]`         — semantic-validate all `.tracking/` files
//!   `check FILE...`           — parse-check one or more `.sysml` files
//!   `orient [ROOT]`           — print orient state (cursor + ready/done/outstanding) as JSON
//!   `whats-next [ROOT]`       — print ready task names, one per line
//!   `advance <sprint> [--to G]` — process cursor: the sprint's current ceremony step; `--to` is
//!                               refused until every earlier step's verify-Test passes (D0209 clause 3)
//!   `record result [FLAGS]`   — append a `TestResult` to a tracking file (D0451)
//!   `record gate-result [FLAGS]` — append a `TestResult` for a ceremony gate (`verification`)
//!   `record task [FLAGS]`     — add a task + `DoD` verification to an action def
//!   `coverage [ROOT]`         — assurance-coverage view (D0079 C): Need/Requirement/Decision evidence
//!   `critique-coverage [ROOT]` — per-element x required-lens critique coverage (D0080)
//!   `critique-policy [ROOT]`   — the active declared critique policy: required lenses per type (D0097)
//!   `concern-coverage [ROOT]` — which declared viewpoint concerns are served vs planned (D0057)
//!   `dispositions [ROOT]`     — >= Medium findings + their typed disposition verdict (D0092)
//!   `sitting-coverage [ROOT]` — which delivery sprints are covered by a per-sitting review (D0049)
//!   `assured [ROOT]`           — composite assurance-readiness verdict + blockers (D0079 c)
//!   `decisions [ROOT]`         — load-bearing decisions ranked by dependence + antiquation flags
//!   `render model [--root ROOT]` — comprehensive interactive traceability diagram (HTML; computed #View, D0449)
//!   `init DIR`                 — scaffold the engine into a new project (D0093 cold start)
//!   `serve [--port N] [ROOT]`  — the interactive console: localhost read dashboard (D0094 m1)
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

use std::{path::{Path, PathBuf}, process};

use keel_cli::embedded::ENGINE_DIR;
use keel_cli::{check_files, collect_sysml, validate_root};
use keel_cli::orient;
use keel_cli::write as w;
// The hooks are member keel-hooks (D0479, sprint 739); project discovery is the process layer's.
use keel_cli::hooks::{cmd_hook, ledger_gate, ledger_refused, override_path, override_target, refused_injected_prose, OVERRIDE_TTL_SECS, RECALL_BUDGET};
use keel_cli::workspace::{engine_version_skew, find_repo_root};

// ── engine scaffold payload (D0093 `init`): `keel_cli::embedded::ENGINE_DIR`, embedded ONCE in the
//    library so the process-change guard reads the same bytes `init` and `migrate` write (D0441). ──
// A DOWNSTREAM CLAUDE.md template (issue057): a fresh project is TRACKED BY keel, not keel itself.
// The self-build repo's own CLAUDE.md (about building the engine) is NEVER shipped to init'd projects.
const CLAUDE_MD: &str = include_str!("../assets/claude-md-template.md");
const TRACKING_STARTER: &str = "# .tracking/ — your project's instance data\n\nThis directory holds THIS project's authored facts (needs, requirements, work items, issues,\ndecisions, test results) — the per-project INSTANCE. The reusable engine lives in `.engine/`.\n\nGetting started: run the `introduction` skill (guided onboarding), or author your first `Need`\nfollowing `.engine/docs/tracking-template.sysml`. State is COMPUTED — run `keel show orient .` to\nsee where things stand. The engine's design rationale is read-only in `.engine/reference/decisions/`;\nyour project authors its OWN decisions fresh in `.engine/decisions/`.\n";
/// A fresh project's deliverable-suspicion manifest is EMPTY — the shipped one lists the ENGINE's own
/// deliverable tasks (instance-specific), which would fail manifest-coverage on a new project (D0093
/// engine/instance boundary). The new project adds entries as it builds source-dependent verifications.
const STARTER_MANIFEST: &str = "# deliverable-manifest.txt — declares which verification tasks depend on which DELIVERABLE SOURCE\n# files (D0050), so `keel suspect` flags a task suspect when its source changed since it was\n# verified. One entry per line:  task: <taskName> | <relpath> <relpath> ...\n# Empty for a new project — add an entry when you have a deliverable-source-dependent verification.\n";
/// A starter actor registry scaffolded into a fresh project (`.tracking/actors.sysml`). Without it
/// the newcomer's FIRST recorded fact (any `createdBy`/`judgedBy`) fails the actors guard (D0037) —
/// there'd be no `ProjectActors` to reference. Ships placeholder actors (a human + the AI) the newcomer
/// edits to their real identities; the declared part name is the id that `createdBy`/`judgedBy` reference.
const STARTER_ACTORS: &str = "// ProjectActors — this project's actor registry (INSTANCE data). EDIT to your real actors.\n// The declared part name is the id that createdBy/judgedBy reference (enforced by `keel gate guard actors`).\npackage ProjectActors {\n    private import EngineElement::*;\n\n    part you : Person { :>> name = \"Your Name\"; :>> email = \"you@example.com\"; }\n    part ai : Actor { :>> name = \"AI assistant\"; :>> kind = ActorKind::ai; }\n}\n";
use keel_cli::precommit_hook;

/// Scaffolded `.gitignore`. Machine-local state only — nothing here is a build artifact of the
/// project, it is state that is TRUE OF ONE CLONE and false of every other.
const GITIGNORE: &str = "# keel machine-local state — never commit these.
#
# .keel/actor is THIS MACHINE's identity binding (D0129). Committing it hands your identity to every
# other clone: two contributors who both commit one end up conflicting over who they are, and a
# merge that picks a side silently makes one of them write as the other.
.keel/

# `keel serve` per-clone preferences.
.keel-serve.json

# Generated views/reports — regenerable from the model, so they are outputs, not facts.
*.keel.html
";

/// Drop each named flag AND ITS VALUE, leaving only true positionals for [`root_arg`].
///
/// THE CLASS THIS ENDS, third instance in one session: `root_arg` takes the first bare token as ROOT
/// and cannot know which flags consume the token after them, so every command that adds a
/// value-taking flag re-creates the bug. `keel github decider --root X` read X as a login,
/// `keel recall --prompt -` read `-` as a root, and `keel why t --budget 1500` read 1500 as a root —
/// each silently answering about the wrong thing until the issue281 project precondition started
/// refusing outright, which is the only reason the last two were visible at all.
///
/// It is NOT fixed inside `root_arg` because a known flag's following positional is legitimately the
/// root for existing callers (`--explain /r`), so the distinction has to be stated by the caller that
/// knows it.
fn without_flag_values(args: &[String], value_flags: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    let mut skip = false;
    for a in args {
        if skip {
            skip = false;
            continue;
        }
        if let Some(name) = a.strip_prefix("--") {
            if value_flags.contains(&name) {
                skip = true;
                continue;
            }
        }
        out.push(a.clone());
    }
    out
}

fn root_arg(args: &[String], usage: &str, known: &[&str], positionals: usize) -> Result<PathBuf, i32> {
    let mut positional: Vec<&String> = Vec::new();
    for a in args {
        if let Some(name) = a.strip_prefix("--") {
            // A flag's VALUE is consumed by the caller's own parse; only the flag NAME is judged here.
            if !known.contains(&name) {
                eprintln!("error: unknown flag `{a}`");
                eprintln!("usage: {usage}");
                return Err(2);
            }
        } else {
            positional.push(a);
        }
    }
    if let Some(p) = positional.get(positionals) {
        let root = PathBuf::from(p.as_str());
        keel_cli::workspace::require_project(&root, usage)?;
        return Ok(root);
    }
    let root = find_repo_root().ok_or_else(|| {
        eprintln!("error: no .engine/ directory found from the current directory upward");
        eprintln!("  (the search stops at the repository boundary — it will not answer for another repo).");
        eprintln!("usage: {usage}");
        2
    })?;
    keel_cli::workspace::require_project(&root, usage)?;
    Ok(root)
}

// ── subcommands ───────────────────────────────────────────────────────────────


fn cmd_validate(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel gate validate [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };

    // REFUSE A NON-PROJECT (issue269/D0234). Pointed at a directory with no `.engine/`, validate used
    // to print "0 tracking file(s) validated clean" and exit 0 — a vacuous pass on the FIRST line of
    // every gate. A hook placed at a repo root holding several projects would therefore gate NOTHING
    // while reporting success. `keel gate guard` already fails in that position; the two halves of the gate
    // disagreed about whether an absent project is a clean tree or a usage error. It is a usage error.
    //
    // A real project holding zero tracking files still exits 0: that is a true statement about a
    // project that exists.
    if !keel_cli::workspace::is_project(&root) {
        // NAME THE DIRECTORY THAT IS MISSING (issue283).
        eprintln!("error: {} is not a keel project — it has no {} directory.", root.display(), keel_cli::workspace::missing_project_dirs(&root));
        let ws = keel_cli::workspace::discover(&root);
        if ws.is_multi() {
            eprintln!("  This repository holds {} projects. validate takes ONE project root:", ws.projects.len());
            for p in &ws.projects {
                eprintln!("    keel gate validate {}", ws.label(p));
            }
            // issue278: this used to advise `keel hook pre-commit`, which is not a hook event -
            // at a workspace root it prints nothing and exits 0, so anyone who wired it in
            // installed a gate that passes while checking nothing. Name the real command.
            eprintln!("  To gate every project in this repository, run `keel gate --workspace .`");
            eprintln!("  (that is what the scaffolded .githooks/pre-commit at the REPO ROOT runs).");
        } else {
            eprintln!("  Validating a directory that is not a project would report a clean tree over zero");
            eprintln!("  files, which is how a gate passes while checking nothing (issue269).");
        }
        return 2;
    }
    if let Some(w) = engine_version_skew(&root) {
        // D0251: for a GATE surface the skew REFUSES rather than warns — a verdict from an engine
        // the project did not declare is not the project's verdict. `orient` still answers (reads
        // warn), and `migrate` is the repair path.
        eprintln!("{w}");
        eprintln!("validate REFUSED under engine-version skew (D0251). Run the pinned version, or `keel migrate`.");
        return 2;
    }
    let report = validate_root(&root);

    for (path, diag) in &report.diagnostics {
        println!("ERROR: {}:{} — {}", path.display(), diag.line, diag.message);
        if let Some(hint) = &diag.suggestion {
            println!("       hint: {hint}");
        }
    }
    for err in &report.errors {
        println!("FAIL:  {} — {}", err.file.display(), err.message);
    }

    let failing: Vec<String> = if report.is_clean() { Vec::new() } else { vec!["validate".to_string()] };
    ledger_gate(&root, "validate", &failing, 0);
    if report.is_clean() {
        println!("{}", keel_cli::color::pass(&format!("{} tracking file(s) validated clean.", report.validated)));
        0
    } else {
        eprintln!(
            "{}",
            keel_cli::color::fail(&format!(
                "{} tracking file(s) validated — {} parse error(s), {} semantic diagnostic(s).",
                report.validated,
                report.errors.len(),
                report.diagnostics.len()
            ))
        );
        1
    }
}

/// D0452: the gating family's seven verbs are sub-verbs of `gate`, each keeping its word and its
/// arguments. A reserved word is resolved BEFORE a positional is read as a root, so a directory named
/// `validate` is gated with `keel gate --fast validate`, never mistaken for the sub-verb's absence.
fn gate_subverb(args: &[String]) -> Option<i32> {
    let rest = args.get(1..).unwrap_or(&[]);
    Some(match args.first().map(String::as_str)? {
        "validate" => refuse_flag_as_path(rest.first(), "gate validate").unwrap_or_else(|| cmd_validate(rest)),
        "check" => cmd_check(rest),
        "check-engine" => refuse_flag_as_path(rest.first(), "gate check-engine").unwrap_or_else(|| cmd_check_engine(rest)),
        "guard" => cmd_guard(rest),
        "rules" => cmd_rules(rest),
        "assured" => cmd_assured(rest),
        "adoption-check" => keel_cli::adoption_check::cmd(rest),
        _ => return None,
    })
}

/// The `keel gate` usage: the two tiers, then one line per sub-verb the router resolves.
fn print_gate_usage() {
    eprintln!("usage: keel gate --fast [ROOT]   (the per-edit in-loop gate: validate + duplicate-identity + marker-vocabulary + scaffold-placeholder)");
    eprintln!("       keel gate --workspace [ROOT]   (the COMMIT gate for a repo holding several projects: every project the commit touches, D0234)");
    eprintln!("       keel gate validate|check|check-engine|guard|rules|assured|adoption-check ...   (D0452: the seven gating verbs under one router, each keeping its arguments - `keel gate <sub-verb> --help` is its own usage)");
}

/// `keel gate --fast [ROOT]` (D0128 Tier-2) — the per-EDIT in-loop gate.
///
/// Runs only the checks that are (a) fast enough for every edit and (b) EXACT, so blocking is safe:
/// `validate` (227ms — parse + semantic reference resolution), `duplicate-identity` (128ms) and
/// `marker-vocabulary` (140ms). Measured total ~0.5s, against 1.9s for the full guard suite — which is
/// why the full set stays at the TURN boundary (Tier-3 Stop hook) and commit, not per edit.
///
/// Deliberately excludes every heuristic/warning-level guard: a per-edit gate that fires on a prose
/// heuristic would block work mid-thought and train the actor to disable it — the issue076/issue081
/// dynamic that cost eight bypassed commits this sitting.
fn cmd_gate(args: &[String]) -> i32 {
    if let Some(code) = gate_subverb(args) {
        return code;
    }
    // D0234: `--workspace` is a SCOPE (every project in this git repo), `--fast` is a TIER (the
    // per-edit subset). A repo holding several projects can only have one core.hooksPath, so its
    // pre-commit hook calls this rather than a per-project gate that could cover just one of them.
    if args.iter().any(|a| a == "--workspace") {
        return keel_cli::workspace::gate_cmd(args);
    }
    let fast = args.iter().any(|a| a == "--fast");
    let root = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .map(PathBuf::from)
        .or_else(find_repo_root)
        .unwrap_or_else(|| PathBuf::from("."));
    // D0251: the fast tier is a gate too — a per-edit verdict from an undeclared engine misleads
    // in-loop exactly as a commit verdict would at the boundary.
    if let Some(w) = engine_version_skew(&root) {
        eprintln!("{w}");
        eprintln!("gate REFUSED under engine-version skew (D0251). Run the pinned version, or `keel migrate`.");
        return 2;
    }
    if !fast {
        print_gate_usage();
        return 2;
    }

    // THE GUARD RECEIPT: a fast gate over the tree the last green full run judged is answered from
    // that receipt (validate + every guard covers the fast tier's subset). The fast tier never WRITES
    // a receipt - three guards are not the guard set.
    if !keel_cli::receipt::forced(args) {
        if let Some(r) = keel_cli::receipt::key(&root).and_then(|k| keel_cli::receipt::read(&root, &k)) {
            if r.covers_all(&[keel_cli::receipt::VALIDATE, keel_cli::receipt::GUARDS]) {
                println!("{}", r.line("gate: fast gate clean -"));
                return 0;
            }
        }
    }
    let report = keel_cli::validate_root(&root);
    let mut failed = false;
    for (path, d) in &report.diagnostics {
        println!("ERROR: {}:{} — {}", path.display(), d.line, d.message);
        failed = true;
    }
    for e in &report.errors {
        println!("PARSE: {} — {}", e.file.display(), e.message);
        failed = true;
    }
    // The EXACT fast-tier guards — set membership, duplicate detection, unfilled scaffolds. No heuristics.
    for name in ["duplicate-identity", "marker-vocabulary", "scaffold-placeholder"] {
        if let Some(r) = keel_cli::guards::run_one(name, &root) {
            for v in &r.violations {
                println!("GUARD [{name}]: {v}");
                failed = true;
            }
        }
    }
    if failed {
        println!("\ngate: FAST GATE FAILED — fix before continuing (this is the per-edit tier; the full guard set runs at turn end + commit).");
        return 1;
    }
    println!("gate: fast gate clean ({} file(s))", report.validated);
    0
}

/// `keel gate check-engine [ROOT]` (D0112 phase 2, issue067) — semantically validate the `.engine` INSTANCE
/// files (decisions/processes/views + registry + template) against the schema, KERNEL-FREE — the Rust
/// backstop for the `unresolved` reference class the JVM `validate_instances.py` used to be the sole
/// source of.
fn cmd_check_engine(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel gate check-engine [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    let diags = keel_cli::validate_engine_instances(&root);
    for (path, d) in &diags {
        println!("ERROR: {}:{} — {}", path.display(), d.line, d.message);
        if let Some(hint) = &d.suggestion {
            println!("       hint: {hint}");
        }
    }
    let failing: Vec<String> = if diags.is_empty() { Vec::new() } else { vec!["check-engine".to_string()] };
    ledger_gate(&root, "check-engine", &failing, 0);
    if diags.is_empty() {
        println!("{}", keel_cli::color::pass(".engine instance files validated clean (kernel-free; D0112 phase 2)."));
        0
    } else {
        eprintln!("{}", keel_cli::color::fail(&format!("{} .engine semantic diagnostic(s).", diags.len())));
        1
    }
}

fn cmd_spec_version(args: &[String]) -> i32 {
    use keel_parser::spec_compat as sc;
    println!("grammar version (baked): {}", sc::SYSML_V2_GRAMMAR_VERSION);
    println!("pinned sha:              {}", sc::SYSML_V2_GRAMMAR_SHA);
    println!("spec url:                {}", sc::SYSML_V2_SPEC_URL);
    if sc::is_offline() || args.iter().any(|a| a == "--no-fetch") {
        println!("live check:              skipped (offline)");
        return 0;
    }
    let fetched = std::process::Command::new("curl")
        .args(["-sSL", sc::SYSML_V2_SPEC_URL])
        .output();
    let Ok(out) = fetched else {
        println!("live check:              unavailable (curl not found)");
        return 0;
    };
    if !out.status.success() || out.stdout.is_empty() {
        println!("live check:              unavailable (no network)");
        return 0;
    }
    let live = sc::sha256_hex(&out.stdout);
    println!("live sha:                {live}");
    let pinned = sc::SYSML_V2_GRAMMAR_SHA;
    if pinned.bytes().all(|b| b == b'0') {
        println!("status:                  not pinned — baked version is the reference; pin the live sha to enable drift detection");
        0
    } else if live == pinned {
        println!("status:                  CURRENT");
        0
    } else {
        println!("status:                  STALE — upstream changed since the pin");
        1
    }
}

fn cmd_check(args: &[String]) -> i32 {
    if args.iter().any(|a| a == "--spec-version") {
        return cmd_spec_version(args);
    }
    if args.is_empty() {
        eprintln!("usage: keel gate check FILE [FILE...]  |  keel gate check --spec-version [--no-fetch]");
        return 2;
    }
    let files: Vec<PathBuf> = args.iter().map(PathBuf::from).collect();
    let report = check_files(&files);

    for err in &report.errors {
        println!("ERROR: {} — {}", err.file.display(), err.message);
    }
    if report.is_clean() {
        println!("{} file(s) checked clean.", files.len());
        0
    } else {
        eprintln!(
            "{} file(s) checked — {} error(s).",
            files.len(),
            report.errors.len()
        );
        1
    }
}

fn cmd_serve(args: &[String]) -> i32 {
    let mut port: u16 = 7777;
    let mut root_arg: Option<String> = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == "--port" {
            if let Some(v) = it.next() {
                if let Ok(p) = v.parse::<u16>() {
                    port = p;
                }
            }
        } else if !a.starts_with("--") {
            root_arg = Some(a.clone());
        }
    }
    let root = match root_arg {
        Some(p) => PathBuf::from(p),
        None => {
            if let Some(r) = find_repo_root() {
                r
            } else {
                eprintln!("usage: keel serve [--port N] [ROOT] [--stop] [--forget]");
                return 2;
            }
        }
    };
    if !keel_cli::workspace::is_project(&root) {
        eprintln!("serve: {} is not a keel project (needs .engine/ and .tracking/).", root.display());
        return 2;
    }

    // `--forget`: stop listing this project in the console selector, without touching the project.
    if args.iter().any(|a| a == "--forget") {
        return match keel_cli::console_registry::deregister(&root) {
            Ok(true) => {
                println!("console: {} deregistered — it will no longer appear in the selector.", root.display());
                0
            }
            Ok(false) => {
                println!("console: {} was not registered; nothing to forget.", root.display());
                0
            }
            Err(e) => {
                eprintln!("console: {e}");
                1
            }
        };
    }

    // ATTACH, DO NOT SPAWN (D0245 clause 2). This is the whole fix for "too many keel serve windows":
    // running the command in a second project used to start a second server, because binding was the
    // first thing tried. Now the first thing asked is whether one of ours is already answering — and
    // the check distinguishes OUR console from any program holding the socket, because those two
    // situations need opposite responses: attach, or refuse loudly.
    let today = keel_cli::scaffold::today();
    if keel_cli::console_registry::console_on(port) {
        if let Err(e) = keel_cli::console_registry::register(&root, Some(port), &today) {
            eprintln!("console: registered nothing ({e}) — the console is running but this project");
            eprintln!("  will not appear in its selector until the registry is writable.");
            return 1;
        }
        println!("Keel console is ALREADY RUNNING on http://127.0.0.1:{port} — attached, did not start a second.");
        println!("  registered: {}", keel_cli::workspace::canon(&root).display());
        println!("  open:       http://127.0.0.1:{port}/  then pick it from the project selector");
        println!("  forget it:  keel serve --forget {}", root.display());
        return 0;
    }
    // No console of ours. Register BEFORE binding, so the project is in the selector the moment the
    // surface comes up rather than one restart later.
    if let Err(e) = keel_cli::console_registry::register(&root, Some(port), &today) {
        eprintln!("console: could not record this project in the registry: {e}");
        eprintln!("  starting anyway — the selector will show only the active project.");
    }
    keel_cli::serve::run(root, port)
}

fn cmd_orient(args: &[String]) -> i32 {
    let html = args.iter().any(|a| a == "--html");
    let root = match root_arg(args, "keel show orient [ROOT] [--html]", &["html"], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    // D0251 clause C: a READ view proceeds under skew — blocking orient would block the command
    // that diagnoses the skew — but it warns LOUDLY, because what this view shows may not be what
    // the pinned engine's gate would say. Stderr, so the JSON stays parseable.
    if let Some(w) = engine_version_skew(&root) {
        eprintln!("{w}");
    }
    if html {
        return match keel_cli::reports::orient_html(&root) {
            Ok(h) => {
                println!("{h}");
                0
            }
            Err(e) => {
                eprintln!("orient --html error: {e}");
                1
            }
        };
    }
    // K2 visibility (D0174/P0.3): a scaffolded commit gate that git is not wired to run is a
    // silently-open enforcement point. Warn LOUDLY on stderr — the JSON on stdout stays pure.
    // issue240: ARMED means git can REACH the hook, not that a setting points somewhere. The old
    // check passed on `core.hooksPath = nul`, so the gate was silently dead while this warned nothing.
    if let Err(why) = keel_cli::gitx::commit_gate_armed(&root) {
        if root.join(".githooks").join("pre-commit").exists() {
            eprintln!("[keel] WARNING: the commit gate is NOT ARMED — {why}. Fix: git config core.hooksPath .githooks (D0174/K2).");
        }
    }
    println!("{}", keel_cli::reports::orient(&root).to_json());
    0
}

fn cmd_attestation_coverage(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel attestation-coverage [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match keel_cli::view::attestation_coverage(&root) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(e) => {
            eprintln!("attestation-coverage error: {e}");
            1
        }
    }
}

fn cmd_orphans(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel orphans [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match keel_cli::algo::orphans(&root) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(e) => {
            eprintln!("orphans error: {e}");
            1
        }
    }
}

/// D0452: the three audits are sub-verbs of `audit`, resolved before a positional is read as a root.
///
/// issue281: `history` and `adherence` read a MODEL, so they must not answer over nothing — they take
/// their root from `find_repo_root`, the repository-scoped resolver `sync`/`land` use, which carries
/// no project precondition. Found by sweeping every command at a workspace root rather than by
/// trusting that one chokepoint covered them all: nine refused, one still exited 0.
/// D0323 / issue374: `ci-runs` is the external-fact gate - a ci-run receipt is checked against the run itself.
fn audit_subverb(args: &[String]) -> Option<i32> {
    let rest = args.get(1..).unwrap_or(&[]);
    let repo = || find_repo_root().unwrap_or_else(|| PathBuf::from("."));
    Some(match args.first().map(String::as_str)? {
        "history" => keel_cli::history::cmd(rest, &repo()),
        "adherence" => keel_cli::adherence::cmd(rest, &repo()),
        "ci-runs" => refuse_flag_as_path(rest.first(), "audit ci-runs").unwrap_or_else(|| {
            let root = rest.first().filter(|a| !a.starts_with('-')).map_or_else(repo, PathBuf::from);
            keel_cli::ci_runs::cmd(rest, &root)
        }),
        _ => return None,
    })
}

fn cmd_audit(args: &[String]) -> i32 {
    if let Some(code) = audit_subverb(args) {
        return code;
    }
    let root = match root_arg(args, "keel audit [ROOT] | keel audit history|adherence|ci-runs ...", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match keel_cli::algo::audit(&root) {
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

fn resolve_guard_root(arg: Option<&String>) -> Option<PathBuf> {
    arg.map_or_else(find_repo_root, |p| Some(PathBuf::from(p)))
}

/// D0453: the five channel verbs route under `keel github`, each arm MOVED verbatim from the top-level
/// dispatch - its function and its trust classification untouched, its arguments the same. `None`
/// when the first argument is not one of the five, so the router can print its own usage.
fn github_subverb(args: &[String]) -> Option<i32> {
    let rest = args.get(1..).unwrap_or(&[]);
    Some(match args.first().map(String::as_str)? {
        "gesture" => keel_cli::github::gesture_cmd(),
        "pull" => {
            let root = resolve_guard_root(
                rest.iter().position(|a| a == "--root").and_then(|i| rest.get(i + 1)),
            )
            .unwrap_or_else(|| std::path::PathBuf::from("."));
            keel_cli::github_ingest::pull_cmd(rest, &root)
        }
        "ingest" => {
            // ROOT is an explicit --root, never a trailing positional: the trailing argument here is
            // the value of --at, and guessing it as a path made the command fail with an opaque
            // filesystem error on its very first live run.
            let root = resolve_guard_root(
                rest.iter().position(|a| a == "--root").and_then(|i| rest.get(i + 1)),
            )
            .unwrap_or_else(|| std::path::PathBuf::from("."));
            keel_cli::github_ingest::cmd(rest, &root)
        }
        "decision-id" => keel_cli::github::decision_id_cmd(rest),
        "decider" => keel_cli::github::decider_cmd(rest, &find_repo_root().unwrap_or_else(|| PathBuf::from("."))),
        _ => return None,
    })
}

/// `keel github <sub-verb> ...` - the channel family under one router (D0453). Bare, or with a word
/// that is not a sub-verb, it prints the five and refuses: there is no bare meaning to default to.
fn cmd_github(args: &[String]) -> i32 {
    if let Some(code) = github_subverb(args) {
        return code;
    }
    // a word that is not a sub-verb is named; a flag (`--help`) falls through to the usage alone
    if let Some(a) = args.first().filter(|a| !a.starts_with('-')) {
        eprintln!("keel github: `{a}` is not a sub-verb.");
    }
    eprintln!("usage: keel github pull|ingest|decider|gesture|decision-id ...   (D0453: the five channel verbs under one router, each keeping its arguments - `keel github <sub-verb>` with none is its own usage)");
    2
}

/// Refuse an argument that LOOKS like a flag where a path or a name is expected (GH#14).
///
/// A mistyped or unsupported `--flag` used to be accepted as the ROOT: `keel gate guard --read` gated a
/// directory named `--read`, found nothing, and reported every guard PASS with 0 scanned. Silent
/// mis-parsing plus pass-at-zero produces a GREEN RUN OVER NOTHING, which is worse than an error
/// because it is indistinguishable from a clean tree. The same shape was hit again while building
/// `github ingest`, where a trailing `--at` value was read as the root.
///
/// Returns the exit code to use, or `None` when the argument is fine. `cmd_activation` already did
/// this for process names (issue179); this generalises it to every path-taking entry point.
fn refuse_flag_as_path(arg: Option<&String>, cmd: &str) -> Option<i32> {
    let a = arg?;
    if !a.starts_with("--") {
        return None;
    }
    eprintln!("keel {cmd}: `{a}` looks like a flag, not a path.");
    eprintln!("  It would otherwise be taken as the ROOT — and a root that does not exist scans");
    eprintln!("  NOTHING, so every check would report PASS over an empty tree (GH#14). Refusing");
    eprintln!("  rather than answering green about a directory that is not there.");
    Some(2)
}

/// A string that names a runnable guard (an enforced one, or a runnable-only diagnostic).
fn is_guard_name(s: &str) -> bool {
    keel_cli::guards::GUARD_NAMES.contains(&s) || matches!(s, "assured" | "critique" | "critique-rigor" | "defect-guard-coverage")
}

/// Classify `keel gate guard` args into `(guard name to run, root arg)`. A first arg that is a known guard
/// name runs THAT guard; `all`, no arg, or a non-name first arg (a ROOT path like `.` or a dir) runs
/// ALL guards on that root. This is what lets `keel gate guard <ROOT>` work like `keel gate validate <ROOT>`.
fn classify_guard_args(args: &[String]) -> (Option<&str>, Option<&str>) {
    match args.first().map(String::as_str) {
        None => (None, None),
        Some("all") => (None, args.get(1).map(String::as_str)),
        Some(a) if is_guard_name(a) => (Some(a), args.get(1).map(String::as_str)),
        Some(a) => (None, Some(a)), // a bare ROOT, not a guard name
    }
}

fn cmd_guard(args: &[String]) -> i32 {
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
        let receipt_key = if keel_cli::receipt::forced(args) { None } else { keel_cli::receipt::key(&root) };
        let receipt = receipt_key
            .as_ref()
            .and_then(|k| keel_cli::receipt::read(&root, k))
            .filter(|r| r.covers_all(&[keel_cli::receipt::GUARDS]));
        let (reports, durations, from_receipt) = if let Some(r) = receipt {
            let line = r.line("[guard]");
            (r.guards, Vec::new(), Some(line))
        } else {
            let (reports, durations) = keel_cli::guards::run_all_timed(&root);
            (reports, durations, None)
        };
        // D0278: a control with a KNOWN defect says so beside its own verdict. Printed here rather
        // than inside `GuardReport::print` because the runner is what holds the root — and because
        // the note belongs to the reading, not to the report: the moment someone needs to know a
        // green is unreliable is the moment they are looking at it.
        let defects = keel_cli::control_defects::load(&root);
        let mut all_ok = true;
        for r in &reports {
            r.print();
            if let Some(d) = defects.get(r.name) {
                println!("{}", keel_cli::color::defect(&d.note(r.name)));
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
        let tail = keel_cli::guards::warning_population(&reports);
        println!("[guard] {}{tail}", if all_ok { keel_cli::color::pass("ALL PASS") } else { keel_cli::color::fail("FAILED") });
        let failing: Vec<String> = reports.iter().filter(|r| !r.ok()).map(|r| r.name.to_string()).collect();
        ledger_gate(&root, "guard", &failing, guard_started.elapsed().as_millis());
        // D0414 / issue429: the set's wall clock is bounded below by its longest guard; name it.
        if keel_cli::perf::enabled() {
            if let Some((name, ms)) = keel_cli::guards::critical_path() {
                println!("[guard] critical path: {name} {ms} ms - the parallel set's wall clock is bounded below by it");
            }
        }
        if let Some(line) = from_receipt {
            println!("{line}");
        } else if let Some(k) = &receipt_key {
            if all_ok {
                let _ = keel_cli::receipt::record_green(&root, k, &[keel_cli::receipt::GUARDS], &reports, &durations);
            } else {
                keel_cli::receipt::delete(&root);
            }
        }
        return i32::from(!all_ok);
    };
    let Some(report) = keel_cli::guards::run_one(name, &root) else {
        eprintln!(
            "unknown guard '{name}' (enforced: {} | runnable diagnostics: assured, critique, critique-rigor, defect-guard-coverage)",
            keel_cli::guards::GUARD_NAMES.join(", ")
        );
        return 2;
    };
    report.print();
    // Asking for ONE guard by name is a diagnostic, so the check still RUNS and its findings are still
    // shown — but the exit code must agree with the enforced gate (D0138). Without this, `keel gate guard
    // issues` exits 1 on a project that never adopted issue-resolution while `keel gate guard` exits 0, and a
    // script wired to the single-guard form would block on a control the project deliberately does not
    // enforce.
    if let keel_cli::activation::GuardState::Inactive(p) =
        keel_cli::activation::Activation::load(&root).guard_state(name)
    {
        println!(
            "[guard:{name}] NOT ACTIVE — process `{p}` is not in this project's active set, so the findings above are informational and do NOT block (`keel activate {p}` to enforce them)"
        );
        return 0;
    }
    i32::from(!report.ok())
}

// Root-only query: `keel <name> [ROOT]`.
fn cmd_query0(args: &[String], usage: &str, f: fn(&std::path::Path) -> String) -> i32 {
    let root = match root_arg(args, &format!("keel {usage} [ROOT]"), &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    println!("{}", f(&root));
    0
}

// Name + optional root: `keel <name> <arg> [ROOT]`.
fn cmd_query1(args: &[String], usage: &str, f: fn(&std::path::Path, &str) -> String) -> i32 {
    let arg = match positional_arg(args, &format!("keel {usage} <name> [ROOT]"), "an item name") {
        Ok(a) => a,
        Err(code) => return code,
    };
    let root = match root_arg(args, &format!("keel {usage} <name> [ROOT]"), &[], 1) {
        Ok(r) => r,
        Err(code) => return code,
    };
    // AN UNRESOLVABLE NAME IS A FAILURE, NOT AN EMPTY RESULT (issue177). Every command routed through
    // here used to exit 0 for a name that does not exist, answering `{upstream: [], downstream: []}` -
    // which a script reads as "no relations", the reassuring wrong answer. `report`, `render` and `arch`
    // already exit nonzero on an unknown argument; these six did not, so the CLI was inconsistent with
    // itself on the one interface D0093 makes the automation substrate.
    if !keel_cli::queries::is_declared(&root, arg) {
        eprintln!(
            "keel {usage}: no item named `{arg}` is declared in this model.
               An unknown name exits nonzero rather than answering with an empty result, because an empty              result reads as `this item has no relations` (issue177)."
        );
        return 1;
    }
    println!("{}", f(&root, arg));
    0
}

/// `keel record reverify [--all-drift | --task NAME | --demos] [--by ACTOR] [ROOT]` (D0101) — re-run the configured
/// gate at HEAD and stamp a fresh `TestResult` on each drift-suspect task on green; `--demos` (D0444)
/// re-runs every replayable demo receipt instead and records each replay's own verdict.
fn cmd_reverify(args: &[String]) -> i32 {
    let mut task: Option<String> = None;
    let mut by: Option<String> = None;
    let mut root: Option<PathBuf> = None;
    let mut demos = false;
    let mut i = 0;
    while let Some(a) = args.get(i) {
        match a.as_str() {
            "--all-drift" => {}
            "--demos" => demos = true,
            "--task" => {
                i += 1;
                task = args.get(i).cloned();
            }
            // The actor error names `--judged-by`, `--author` and `--by` as equivalents, so all three
            // are accepted here. They were not: `--judged-by claudeOpus5` fell through to the ROOT
            // arm, made the root `claudeOpus5`, and the command then refused with "no acting actor"
            // — an error about provenance for what was really an unknown flag, pointing at the one
            // fix that could not work.
            "--by" | "--judged-by" | "--author" => {
                i += 1;
                by = args.get(i).cloned();
            }
            // An unrecognised FLAG is a mistake, not a path. Swallowing it as a root turned a typo
            // into a confident wrong answer somewhere further downstream, which is this session's
            // most-repeated defect shape.
            other if other.starts_with("--") => {
                eprintln!("error: unknown flag `{other}`");
                eprintln!("usage: keel record reverify [--all-drift | --task NAME | --demos] [--by ACTOR] [ROOT]");
                return 2;
            }
            other => root = Some(PathBuf::from(other)),
        }
        i += 1;
    }
    let root = root.or_else(find_repo_root).unwrap_or_else(|| PathBuf::from("."));
    // reverify STAMPS a fresh TestResult, so it needs a true attributable actor (D0129/issue072).
    let by = match keel_cli::actor::resolve(&root, by.as_deref()) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("{msg}");
            return 2;
        }
    };
    if demos {
        return keel_cli::reverify::replay_demos(&root, &by);
    }
    keel_cli::reverify::run(&root, task.as_deref(), &by)
}

/// `keel intake [ROOT]` (D0166) — what was said, what it became, what nobody acted on.
fn cmd_intake(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel intake [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match keel_cli::view::intake(&root) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(e) => {
            eprintln!("error: {e}");
            1
        }
    }
}

/// `keel show priority [ROOT]` (D0311): the priority metric made visible.
// `keel show commit-delta [ROOT] [--range A..B]` - the model delta over a git range (dcCommitDeltaView, D0282).
fn cmd_commit_delta(args: &[String]) -> i32 {
    let usage = "keel show commit-delta [ROOT] [--range A..B]";
    let range = args.iter().position(|a| a == "--range").map_or_else(|| "HEAD~1..HEAD".to_string(), |i| args.get(i + 1).cloned().unwrap_or_default());
    if range.is_empty() || range.starts_with("--") {
        eprintln!("error: --range takes a value of the form A..B");
        eprintln!("usage: {usage}");
        return 2;
    }
    // the range value is consumed here, so it is not a positional for root_arg
    let rest: Vec<String> = args.iter().filter(|a| *a != "--range" && **a != range).cloned().collect();
    let root = match root_arg(&rest, usage, &["range"], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match keel_cli::view::delta::commit_delta(&root, &range) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(e) => {
            eprintln!("commit-delta error: {e}");
            1
        }
    }
}

fn cmd_priority(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel show priority [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match keel_cli::priority::priority(&root) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(e) => {
            eprintln!("priority error: {e}");
            1
        }
    }
}

fn cmd_open_issues(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel open-issues [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match keel_cli::view::open_issues(&root) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(e) => {
            eprintln!("open-issues error: {e}");
            1
        }
    }
}

fn cmd_dispositions(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel dispositions [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match keel_cli::view::dispositions(&root) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(e) => {
            eprintln!("dispositions error: {e}");
            1
        }
    }
}

fn cmd_sitting_coverage(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel sitting-coverage [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match keel_cli::view::sitting_coverage(&root) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(e) => {
            eprintln!("sitting-coverage error: {e}");
            1
        }
    }
}

fn cmd_deck(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel deck [ROOT] [--out FILE]", &["out"], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match keel_cli::deck::html(&root) {
        Ok(h) => {
            if let Some(out) = flag(args, "out") {
                if let Err(e) = keel_cli::write::write_atomic(std::path::Path::new(&out), &h) {
                    eprintln!("keel deck: writing {out}: {e}");
                    return 1;
                }
                println!("deck -> {out}");
            } else {
                println!("{h}");
            }
            0
        }
        Err(e) => {
            eprintln!("keel deck: {e}");
            1
        }
    }
}

/// `keel record mint [N]` (us019/issue170) — engine-minted v4 UUIDs, one per line, nothing else on stdout,
/// composable into any authoring script.
///
/// Exists so no authoring path depends on an AI generating identity by hand: two hand-minted ids
/// were mangled before guard 38 existed, and manual diligence is not a control (D0047). What this
/// prints is tested against guard 38's OWN shape predicate, so mint and guard stay one truth.
fn cmd_mint(args: &[String]) -> i32 {
    const USAGE: &str = "keel record mint [N]   (N >= 1, default 1)";
    let n: u64 = match args {
        [] => 1,
        [a] => {
            if a.starts_with('-') {
                // the positional_arg convention (issue179): a leading dash is never a count
                eprintln!("error: `{a}` looks like a flag, not a count.");
                eprintln!("usage: {USAGE}");
                return 2;
            }
            match a.parse::<u64>() {
                Ok(n) if n >= 1 => n,
                _ => {
                    eprintln!("error: `{a}` is not a count of at least 1.");
                    eprintln!("usage: {USAGE}");
                    return 2;
                }
            }
        }
        _ => {
            eprintln!("usage: {USAGE}");
            return 2;
        }
    };
    let mut out = String::new();
    for _ in 0..n {
        out.push_str(&keel_cli::ident::gen_uuid());
        out.push('\n');
    }
    print!("{out}");
    0
}

/// `keel record sprint <N> <slug> --charter <decision> [--points P]` (dcSprintScaffold/us019) — the
/// engine scaffolds the ceremony record: ids minted, provenance from the bound actor (refused when
/// absent), placeholders the fast gate rejects. See [`keel_cli::scaffold`].
fn cmd_new(args: &[String]) -> i32 {
    const USAGE: &str = "keel record sprint <NUMBER> <slug> --charter <decision> [--points P] [--fill FILE]";
    if args.first().map(String::as_str) != Some("sprint") {
        eprintln!("usage: {USAGE}");
        return 2;
    }
    let rest = args.get(1..).unwrap_or(&[]);
    let positionals: Vec<&String> = {
        let mut out = Vec::new();
        let mut skip = false;
        for a in rest {
            if skip {
                skip = false;
                continue;
            }
            if a.starts_with("--") {
                skip = true; // every flag here takes a value
                continue;
            }
            out.push(a);
        }
        out
    };
    let [number_arg, slug] = positionals.as_slice() else {
        eprintln!("usage: {USAGE}");
        return 2;
    };
    let Ok(number) = number_arg.parse::<u32>() else {
        eprintln!("error: `{number_arg}` is not a sprint number.");
        eprintln!("usage: {USAGE}");
        return 2;
    };
    let Some(charter) = flag(rest, "charter") else {
        eprintln!("error: --charter <decision> is required — a sprint's story is chartered, never orphaned.");
        eprintln!("usage: {USAGE}");
        return 2;
    };
    let points: u32 = match flag(rest, "points") {
        None => 1,
        Some(p) => match p.parse() {
            Ok(n) if n >= 1 => n,
            _ => {
                eprintln!("error: --points takes a count of at least 1.");
                return 2;
            }
        },
    };
    let root = find_repo_root().unwrap_or_else(|| PathBuf::from("."));
    // The provenance rule: the author is the bound actor, REFUSED when absent — never defaulted.
    let actor = match keel_cli::actor::resolve(&root, None) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("keel record sprint: {msg}");
            return 1;
        }
    };
    // issue267/D0301: `--fill FILE` writes the record's PROSE from a `--- key` draft (purpose, dod,
    // refine, standup, implement, review, closeOut, retro) - the sanctioned path for what a scratchpad
    // script used to emit, TestResult lines included. The fill writes no result; append-result and
    // append-gate-result are the only writers of verdicts.
    if let Some(fill_path) = flag(rest, "fill") {
        let fill = match decision_fields_from_file(&fill_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("keel record sprint: {e}");
                return 2;
            }
        };
        return match keel_cli::scaffold::sprint_filled(&root, number, slug, &charter, points, &actor, &fill) {
            Ok(path) => {
                println!("scaffolded and filled -> {}", path.display());
                println!("no result was written: record the DoD with `keel record result --file {} --task story<Slug> ...` and gates with `keel record gate-result`", path.display());
                0
            }
            Err(e) => {
                eprintln!("keel record sprint: {e}");
                1
            }
        };
    }
    match keel_cli::scaffold::sprint(&root, number, slug, &charter, points, &actor) {
        Ok(path) => {
            println!("scaffolded -> {}", path.display());
            println!("fill every {} before judging any gate - `keel gate --fast` rejects it until then", keel_cli::scaffold::PLACEHOLDER);
            0
        }
        Err(e) => {
            eprintln!("keel record sprint: {e}");
            1
        }
    }
}

/// `keel decision-follow-through [ROOT] [--table]` (dcDecisionFollowThroughView/us020) — JSON is the
/// authority; `--table` renders the same data for eyes: one line per accepted Decision with
/// downstream work, then the gaps.
fn cmd_decision_follow_through(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel decision-follow-through [ROOT] [--table]", &["table"], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    let json = match keel_cli::view::decision_follow_through(&root) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("keel decision-follow-through: {e}");
            return 1;
        }
    };
    if !args.iter().any(|a| a == "--table") {
        println!("{json}");
        return 0;
    }
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&json) else {
        eprintln!("keel decision-follow-through: internal: view emitted unparsable JSON");
        return 1;
    };
    let empty = Vec::new();
    println!(
        "accepted {}   with-downstream {}   gaps {}",
        v.get("acceptedDecisions").and_then(serde_json::Value::as_i64).unwrap_or(0),
        v.get("withDownstream").and_then(serde_json::Value::as_i64).unwrap_or(0),
        v.get("gapCount").and_then(serde_json::Value::as_i64).unwrap_or(0),
    );
    for d in v.get("decisions").and_then(|x| x.as_array()).unwrap_or(&empty) {
        let items = d.get("items").and_then(|x| x.as_array()).cloned().unwrap_or_default();
        let summary: Vec<String> = items
            .iter()
            .map(|i| {
                format!(
                    "{} ({})",
                    i.get("item").and_then(|x| x.as_str()).unwrap_or("?"),
                    i.get("evidence").and_then(|x| x.as_str()).unwrap_or("?"),
                )
            })
            .collect();
        println!("  {}  <-  {}", d.get("decision").and_then(|x| x.as_str()).unwrap_or("?"), summary.join(", "));
    }
    let gaps: Vec<&str> =
        v.get("gaps").and_then(|x| x.as_array()).unwrap_or(&empty).iter().filter_map(|g| g.as_str()).collect();
    if !gaps.is_empty() {
        println!("  GAPS (no downstream tracked item): {}", gaps.join(", "));
    }
    0
}

/// `keel sync-claude [ROOT] [--check]` (D0174/P0.2) — regenerate the keel-owned subset of the
/// `.claude/` surface in place (foreign entries survive), or with `--check` report drift and
/// version skew without writing. `--check` IS the `claude-surface-drift` guard's implementation.
/// `keel claude [claude args...]` - launch an interactive (or `-p`) Claude Code session with the keel
/// hooks pinned ON (D0296 layer 2): `--plugin-dir` at the plugin rendering and `--settings` with
/// `disableAllHooks: false` above project scope, `KEEL_BIN` pointing at this binary so every hook
/// resolves the engine that launched it. Everything after `claude` is passed through. Runs from
/// inside a keel project; the exit code is claude's own.
fn cmd_claude(args: &[String]) -> i32 {
    let Some(root) = find_repo_root() else {
        eprintln!("keel claude: not inside a keel project (no .engine/ up to the repository boundary) - run it from the project you want gated");
        return 2;
    };
    let pin = match keel_cli::launcher::hook_pin_args(&root) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("keel claude: cannot write the hook pin ({e}) - refusing to launch unpinned; a launch without the pin is `claude` itself");
            return 2;
        }
    };
    let mut command = if cfg!(windows) {
        let mut c = std::process::Command::new("cmd");
        c.arg("/C").arg("claude");
        c
    } else {
        std::process::Command::new("claude")
    };
    if let Ok(exe) = std::env::current_exe() {
        command.env("KEEL_BIN", exe);
    }
    // A keel-launched session is not a nested one: the harness marks its own shells, and a launch
    // from inside one would be refused by claude itself.
    command.env_remove("CLAUDECODE");
    match command.args(&pin).args(args).current_dir(&root).status() {
        Ok(st) => st.code().unwrap_or(1),
        Err(e) => {
            eprintln!("keel claude: cannot launch the `claude` CLI ({e}) - install Claude Code and put it on PATH");
            2
        }
    }
}

fn cmd_sync_claude(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel sync-claude [ROOT] [--check]", &["check"], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    let check = args.iter().any(|a| a == "--check");
    match keel_cli::claude_surface::sync_claude(&root, check) {
        Ok(r) => {
            if check {
                for d in &r.drift {
                    println!("DRIFT: {d}");
                }
                if let Some((old, new)) = &r.version_skew {
                    println!("REGENERATE: surface stamped by generator {old}, this binary is {new} — run `keel sync-claude` (an obligation, not a violation)");
                }
                if r.drift.is_empty() {
                    println!("claude-surface: keel-owned subset matches this binary's generator ({} registry skill(s))", r.registry_count);
                    0
                } else {
                    1
                }
            } else {
                println!(
                    "claude-surface synced: settings.json merged (keel-owned entries only), output style, {}/{} skill(s), stamped {} (version and build).",
                    r.skills_written,
                    r.registry_count,
                    keel_cli::claude_surface::surface_stamp()
                );
                // D0391/issue408: the surface that writes the hook commands also says - and on a self-build
                // tree places - the binary those commands will resolve to.
                match keel_cli::hook_binary::refresh(&root) {
                    Ok(Some(p)) => println!("hook binary: placed {} from {} (self-build stable copy, D0391)", p.copy.display(), p.from.display()),
                    Ok(None) => {}
                    Err(e) => println!("hook binary: could not place the self-build stable copy ({e})"),
                }
                if let Some(line) = keel_cli::hook_binary::describe(&root) {
                    println!("{line}");
                }
                0
            }
        }
        Err(e) => {
            eprintln!("keel sync-claude: {e}");
            1
        }
    }
}


/// `keel override <path> --reason "<text>"` (D0176 tier 3) — the sanctioned unlock for a direct
/// write the API cannot express. Single-use, target-path-bound, expiring; consumption records an
/// orient-visible obligation naming the path actually written (K7). Never a silent env var.
fn cmd_override(args: &[String]) -> i32 {
    const USAGE: &str = "keel override <path> (--reason-from FILE | --reason \"why the API cannot express this write\")";
    let args = &match prose_args(args, &["reason"], "override") {
        Ok(a) => a,
        Err(code) => return code,
    };
    let given = match positional_arg(args, USAGE, "a file path") {
        Ok(a) => a.replace('\\', "/"),
        Err(code) => return code,
    };
    let Some(reason) = flag(args, "reason").filter(|r| r.trim().len() >= 10) else {
        eprintln!("error: --reason is required (at least 10 characters) - the reason IS the record (D0176).");
        eprintln!("usage: {USAGE}");
        return 2;
    };
    let root = find_repo_root().unwrap_or_else(|| PathBuf::from("."));
    let target = match override_target(&root, &given) {
        Ok(key) => key,
        Err(msg) => {
            eprintln!("keel override: {msg}");
            return 2;
        }
    };
    let actor = match keel_cli::actor::resolve(&root, None) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("keel override: {msg}");
            return 1;
        }
    };
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let _ = std::fs::create_dir_all(root.join(".keel"));
    let unlock = serde_json::json!({"path": target, "reason": reason, "actor": actor, "ts": ts});
    if let Err(e) = keel_cli::write::write_atomic(&override_path(&root), unlock.to_string()) {
        eprintln!("keel override: cannot write the unlock: {e}");
        return 1;
    }
    println!("override armed for `{target}` - SINGLE USE, expires in {OVERRIDE_TTL_SECS}s; consumption records an obligation (D0176/K7).");
    0
}


/// `keel show enforcement-report [ROOT]` (D0180/K14) — fires, blocks, overrides, red-yields, and the
/// adherence trend, computed from the machine-local fire-ledger. Promotion decisions cite this.
fn cmd_enforcement_report(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel show enforcement-report [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match keel_cli::pm::enforcement_report(&root) {
        Ok(j) => {
            println!("{j}");
            0
        }
        Err(e) => {
            eprintln!("keel show enforcement-report: {e}");
            1
        }
    }
}

fn cmd_hardening(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel hardening [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match keel_cli::hardening::hardening(&root) {
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

fn cmd_concern_coverage(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel concern-coverage [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match keel_cli::view::concern_coverage(&root) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(e) => {
            eprintln!("concern-coverage error: {e}");
            1
        }
    }
}

// `keel gate rules [ROOT]` (D0105 EXPAND step 2): evaluate the DECLARED rules (`keel gate check` is taken by the
// spec-compat file checker; the D0105 name reconciliation is a tracked follow-up). Runs ALONGSIDE
// `keel gate guard` until parity retires each guard (guardsToRulesMigration).
fn cmd_rules(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel gate rules [ROOT] [--enforce]", &["enforce"], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match keel_cli::view::check(&root) {
        Ok(json) => {
            if !args.iter().any(|a| a == "--enforce") {
                println!("{json}");
                return 0;
            }
            // The gate form (D0177/P1.5): blocking rules FAIL the caller; warnings print.
            let Ok(v) = serde_json::from_str::<serde_json::Value>(&json) else {
                eprintln!("rules --enforce: internal: unparsable rule report");
                return 1;
            };
            let empty = Vec::new();
            let mut blocked = 0usize;
            for r in v.get("rules").and_then(|x| x.as_array()).unwrap_or(&empty) {
                let viols = r.get("violations").and_then(|x| x.as_array()).cloned().unwrap_or_default();
                if viols.is_empty() {
                    continue;
                }
                let name = r.get("rule").and_then(|s| s.as_str()).unwrap_or("?");
                let sev = r.get("severity").and_then(|s| s.as_str()).unwrap_or("?");
                for viol in &viols {
                    println!("[rule {name} - {sev}] {viol}");
                }
                if sev == "blocking" {
                    blocked += viols.len();
                }
            }
            if blocked > 0 {
                println!("rules: {blocked} blocking violation(s)");
                1
            } else {
                println!("rules: enforced clean");
                0
            }
        }
        Err(e) => {
            eprintln!("rules error: {e}");
            1
        }
    }
}

// `keel launchables [ROOT]` (srServeModelDrivenRegistry, Tier 1a): the model-declared launchable set.
fn cmd_launchables(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel launchables [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match keel_cli::view::launchables(&root) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(e) => {
            eprintln!("launchables error: {e}");
            1
        }
    }
}

// `keel business [ROOT]` (serveBusinessNeedsView): the Business layer (Brief/Personas/Needs/UseCases).
fn cmd_business(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel business [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match keel_cli::view::business(&root) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(e) => {
            eprintln!("business error: {e}");
            1
        }
    }
}

fn cmd_coverage(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel coverage [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match keel_cli::view::coverage(&root) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(e) => {
            eprintln!("coverage error: {e}");
            1
        }
    }
}

fn cmd_decisions(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel decisions [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match keel_cli::view::decisions_report(&root) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(e) => {
            eprintln!("decisions error: {e}");
            1
        }
    }
}

fn cmd_assured(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel gate assured [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match keel_cli::guards::assured_report(&root) {
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

fn cmd_critique_coverage(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel critique-coverage [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match keel_cli::view::critique_coverage(&root) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(e) => {
            eprintln!("critique-coverage error: {e}");
            1
        }
    }
}

fn cmd_critique_policy(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel critique-policy [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match keel_cli::view::critique_policy(&root) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(e) => {
            eprintln!("critique-policy error: {e}");
            1
        }
    }
}

fn cmd_governing_version(args: &[String]) -> i32 {
    let item = match positional_arg(
        args,
        "keel show governing-version <delivery Story name> [ROOT]",
        "an item name",
    ) {
        Ok(a) => a,
        Err(code) => return code,
    };
    let root = match root_arg(args, "keel show governing-version <delivery Story name> [ROOT]", &[], 1) {
        Ok(r) => r,
        Err(code) => return code,
    };
    // Same rule as `cmd_query1` (issue177). This command has its own wrapper, which is exactly how it
    // escaped the first fix: `keel show governing-version .` reported a process AND a process definition for
    // a name that does not exist, which is the most confidently wrong answer of the six.
    if !keel_cli::queries::is_declared(&root, item) {
        eprintln!("keel show governing-version: no item named `{item}` is declared in this model.");
        return 1;
    }
    println!("{}", keel_cli::govern::governing_version(&root, item));
    0
}

fn cmd_reprocess_candidates(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel show reprocess-candidates [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    println!("{}", keel_cli::govern::reprocess_candidates(&root));
    0
}

fn cmd_suspect(args: &[String]) -> i32 {
    let explain = args.iter().any(|a| a == "--explain");
    let root = match root_arg(args, "keel suspect [--explain] [ROOT]", &["explain"], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    println!("{}", keel_cli::govern::suspect(&root, explain));
    0
}

fn cmd_view(args: &[String]) -> i32 {
    // issue179: a view name resolves to a file under `.engine/views`, so a flag here is a path lookup.
    if let Some(f) = args.first().filter(|a| a.starts_with('-')) {
        eprintln!("error: `{f}` looks like a flag, not a view name (issue179).");
        return 2;
    }
    let Some(name) = args.first() else {
        eprintln!("usage: keel show view <name> [ROOT]");
        return 2;
    };
    let root = match root_arg(args, "keel show view <name> [ROOT]", &[], 1) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match keel_cli::view::run(&root, name) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(e) => {
            eprintln!("view error: {e}");
            1
        }
    }
}

fn cmd_whats_next(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel show whats-next [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    // issue239/issue247: whats-next used to print nothing and exit 0 whether the frontier was
    // genuinely empty or a filter had failed to compute — identical output for COMPUTED-EMPTY and
    // COULD-NOT-COMPUTE, on the one answer the AI auto-follows (D0052). Now the two are distinct:
    // a failed computation REFUSES rather than answering with silence.
    // The frontier half only (issue439): the suspect walk and the burndown are orient's, not this list's.
    let (ready, compute_failures, outstanding) = orient::ready(&root);
    if !compute_failures.is_empty() {
        eprintln!("whats-next: COULD-NOT-COMPUTE — refusing to print a frontier that may be wrong:");
        for r in &compute_failures {
            eprintln!("  {r}");
        }
        eprintln!("  This is NOT an empty frontier. Fix the model read, then re-run.");
        return 1;
    }
    if ready.is_empty() {
        eprintln!("whats-next: COMPUTED-EMPTY — no task is ready (computed over {outstanding} outstanding item(s)). This is an answer, not a failure.");
    }
    for task in ready {
        println!("{task}");
    }
    0
}

fn cmd_ls(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel ls [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    let dir = root.join(".tracking");
    for p in collect_sysml(&dir) {
        println!("{}", p.display());
    }
    0
}

/// Parse simple `--key value` flag pairs from a flat args slice.
fn flag(args: &[String], name: &str) -> Option<String> {
    let key = format!("--{name}");
    args.windows(2).find_map(|w| match w {
        [k, v] if *k == key => Some(v.clone()),
        _ => None,
    })
}

/// A PROSE flag: `--<name>-from FILE` read verbatim (trimmed), else `--<name> TEXT`. One reader for
/// every prose input of `record` (D0224, issue543). The trap this closes: a backtick inside a
/// double-quoted shell argument is command substitution, so the shell RUNS the command the prose
/// merely names - it fired into `decision` twice, `issue` once, `task` once, and on 2026-09-14 a
/// FIFTH time into `result`'s evidence, because the file form had been added verb by verb, each
/// after its own occurrence. Refuses both flags at once by name (the caller learns which would
/// have won) and a file it cannot read. `Ok(None)` is neither flag given.
fn prose_flag(args: &[String], name: &str, verb: &str) -> Result<Option<String>, String> {
    let from = flag(args, &format!("{name}-from"));
    let inline = flag(args, name);
    match (from, inline) {
        (Some(_), Some(_)) => Err(format!("{verb}: --{name}-from and --{name} were both given; pass one")),
        (Some(f), None) => match std::fs::read_to_string(&f) {
            Ok(t) => Ok(Some(t.trim().to_string())),
            Err(e) => Err(format!("{verb}: cannot read {f}: {e}")),
        },
        (None, inline) => Ok(inline),
    }
}

/// The argument list with every `--<name>-from FILE` of `names` rewritten into `--<name> TEXT`
/// through `prose_flag`, for a command whose prose is read DOWNSTREAM of its entry - `accept`'s
/// `--note` is read by `fold_words_into_note`, `fold_warnings_into_note` and the channel layer
/// before the command itself reads it (issue546, sprint 713). Normalising the arguments once at
/// the entry means every reader sees one form and none learns about files; the alternative,
/// teaching each of seven reads about `-from`, is the verb-by-verb path D0224 refused. Both forms
/// of one name is refused by name (the error is `prose_flag`'s) and nothing downstream runs.
fn prose_args(args: &[String], names: &[&str], verb: &str) -> Result<Vec<String>, i32> {
    let mut out = args.to_vec();
    for name in names {
        let from_key = format!("--{name}-from");
        if !out.contains(&from_key) {
            continue;
        }
        let text = match prose_flag(&out, name, verb) {
            Ok(Some(t)) => t,
            Ok(None) => continue,
            Err(msg) => {
                eprintln!("error: {msg}");
                return Err(2);
            }
        };
        let mut next = Vec::with_capacity(out.len());
        let mut skip_value = false;
        for a in &out {
            if skip_value {
                skip_value = false;
            } else if *a == from_key {
                skip_value = true;
            } else {
                next.push(a.clone());
            }
        }
        next.push(format!("--{name}"));
        next.push(text);
        out = next;
    }
    Ok(out)
}

/// A provenance DATE, refused rather than defaulted (issue182).
///
/// Five write paths read this as `flag(args, ..).unwrap_or_else(|| "2026-01-01".to_owned())`. CLAUDE.md
/// says provenance is never defaulted; that rule was implemented for the ACTOR, where a missing actor
/// makes the write refuse, and the DATE fell back to a false constant. A result written without a date
/// claimed it happened on 2026-01-01, which corrupts any series and feeds guard 36 - whose whole job is
/// catching evidence that cites a date it could not have had.
///
/// REFUSED, not defaulted to today: an AI has no clock it can honestly attest to, and guessing is what
/// produced the constant in the first place. The caller states the date or the write does not happen.
fn provenance_date(args: &[String], flag_name: &str, usage: &str) -> Result<String, i32> {
    flag(args, flag_name).ok_or_else(|| {
        eprintln!("error: --{flag_name} YYYY-MM-DD is required.");
        eprintln!(
"  A provenance date is never defaulted: it used to fall back to 2026-01-01."
        );
        eprintln!("usage: {usage}");
        2
    })
}

fn cmd_append_result(args: &[String]) -> i32 {
    let Some(file_str) = flag(args, "file") else {
        eprintln!("usage: keel record result --file FILE --task TASK --sha SHA [--verdict pass|fail] [--judged-by ACTOR] [--judged-at DATE]");
        eprintln!("       --evidence-from FILE   what ran, read from a file (PREFER THIS for anything quoting a command)");
        eprintln!("       --evidence TEXT        one-line receipt - a shell EXECUTES backticks in it (D0224)");
        return 2;
    };
    let Some(task) = flag(args, "task") else {
        eprintln!("error: --task required");
        return 2;
    };
    let Some(sha) = flag(args, "sha") else {
        eprintln!("error: --sha required");
        return 2;
    };
    let file = PathBuf::from(file_str);
    let verdict = flag(args, "verdict").unwrap_or_else(|| "pass".to_owned());
    // Provenance is never defaulted (D0129/issue072): refuse rather than attribute falsely.
    let judged_by = match keel_cli::actor::resolve(&keel_cli::actor::root_for(&file), flag(args, "judged-by").as_deref()) {
        Ok(a) => a,
        Err(msg) => { eprintln!("{msg}"); return 2; }
    };
    // Callers should pass --judged-at for determinism; this is a safe fallback.
    let judged_at = match provenance_date(args, "judged-at", "keel <write> --judged-at YYYY-MM-DD ...") {
        Ok(d) => d,
        Err(c) => return c,
    };

    let evidence = match prose_flag(args, "evidence", "record result") {
        Ok(e) => e,
        Err(msg) => { eprintln!("error: {msg}"); return 2; }
    };
    match w::append_result(&file, &task, &sha, &verdict, &judged_at, &judged_by, evidence.as_deref()) {
        Ok(uuid) => { println!("{uuid}"); proposed_note(&file, &uuid); binding_note(&file, &sha, &verdict); 0 }
        Err(e @ w::WriteError::ReceiptOwed(..)) => {
            // issue448/D0424: the refusal is a ledger fact - `append-result:ran-receipt` is the census row.
            ledger_refused(&keel_cli::actor::root_for(&file), "append-result", "ran-receipt");
            eprintln!("error: {e}");
            1
        }
        Err(e) => { eprintln!("error: {e}"); 1 }
    }
}

/// Say when the result just written landed as a PROPOSAL (D0312 B): read the outcome back from the
/// line carrying `uuid`, never from the write's own report, so the note is the computed view. stderr,
/// so a caller parsing the uuid is unaffected.
fn proposed_note(file: &std::path::Path, uuid: &str) {
    let Ok(text) = std::fs::read_to_string(file) else { return };
    let recorded = text.lines().find(|l| l.contains(uuid)).is_some_and(|l| l.contains(&format!("VerdictKind::{}", w::PROPOSED)));
    if recorded {
        eprintln!(
            "note: recorded {} - an AI-judged pass on an examined method (demo/analyze/inspect) with no replayable receipt is a proposal \
             until a human judges it, and counts as done for nothing meanwhile (D0312 B)",
            w::PROPOSED
        );
    }
}

/// Say where a pass recorded on a dirty tree will bind (dcResultBindsToItsLandingCommit): the
/// `--sha` names the tree BEFORE the work lands, and every reader of `judgedAgainst` will read the
/// landing commit instead once it exists. stderr, so a caller parsing the uuid is unaffected.
fn binding_note(file: &std::path::Path, sha: &str, verdict: &str) {
    if verdict != "pass" {
        return;
    }
    let dir = file.parent().unwrap_or_else(|| std::path::Path::new("."));
    let Ok(out) = keel_cli::gitx::git().arg("-C").arg(dir).args(["status", "--porcelain"]).output() else { return };
    if !out.status.success() {
        return;
    }
    let dirty = String::from_utf8_lossy(&out.stdout).lines().filter(|l| !l.trim().is_empty()).count();
    if dirty > 0 {
        eprintln!(
            "note: {dirty} uncommitted path(s) - this pass names {sha} but binds to the commit that lands it, \
             once {sha} is that commit's ancestor (dcResultBindsToItsLandingCommit)"
        );
    }
}

fn cmd_append_gate_result(args: &[String]) -> i32 {
    let Some(file_str) = flag(args, "file") else {
        eprintln!("usage: keel record gate-result --file FILE --gate GATE --sha SHA [--verdict pass|fail] [--judged-by ACTOR] [--judged-at DATE]");
        eprintln!("       --evidence-from FILE   what ran, read from a file (PREFER THIS for anything quoting a command)");
        eprintln!("       --evidence TEXT        one-line receipt - a shell EXECUTES backticks in it (D0224)");
        eprintln!("       --notes-from FILE | --notes TEXT   the same choice for notes");
        return 2;
    };
    let Some(gate) = flag(args, "gate") else {
        eprintln!("error: --gate required");
        return 2;
    };
    let Some(sha) = flag(args, "sha") else {
        eprintln!("error: --sha required");
        return 2;
    };
    let file = PathBuf::from(file_str);
    let verdict = flag(args, "verdict").unwrap_or_else(|| "pass".to_owned());
    // Provenance is never defaulted (D0129/issue072): refuse rather than attribute falsely.
    let judged_by = match keel_cli::actor::resolve(&keel_cli::actor::root_for(&file), flag(args, "judged-by").as_deref()) {
        Ok(a) => a,
        Err(msg) => { eprintln!("{msg}"); return 2; }
    };
    // Callers should pass --judged-at for determinism; this is a safe fallback.
    let judged_at = match provenance_date(args, "judged-at", "keel <write> --judged-at YYYY-MM-DD ...") {
        Ok(d) => d,
        Err(c) => return c,
    };

    let (notes, evidence) = match (prose_flag(args, "notes", "record gate-result"), prose_flag(args, "evidence", "record gate-result")) {
        (Ok(n), Ok(e)) => (n, e),
        (Err(msg), _) | (_, Err(msg)) => { eprintln!("error: {msg}"); return 2; }
    };
    match w::append_gate_result(&file, &gate, &sha, &verdict, &judged_at, &judged_by, notes.as_deref(), evidence.as_deref()) {
        Ok(uuid) => { println!("{uuid}"); proposed_note(&file, &uuid); binding_note(&file, &sha, &verdict); 0 }
        Err(e @ w::WriteError::ReceiptOwed(..)) => {
            // issue448/D0424: the refusal is a ledger fact - `append-gate-result:ran-receipt` is the census row.
            ledger_refused(&keel_cli::actor::root_for(&file), "append-gate-result", "ran-receipt");
            eprintln!("error: {e}");
            1
        }
        Err(e @ w::WriteError::RetroScanMissing(..)) => {
            // issue566: the retro scan wording is refused at the write, by the Test - the same ledger fact.
            ledger_refused(&keel_cli::actor::root_for(&file), "append-gate-result", "retro-scan");
            eprintln!("error: {e}");
            1
        }
        Err(e) => { eprintln!("error: {e}"); 1 }
    }
}

/// `keel record <type> ...` — the closed RMWX `record` verb (D0105/D0106; issue054 C1). Currently
/// records a Decision: `keel record decision --slug S --title T --context C --decision D --rationale R
/// --consequences Q --date YYYY-MM-DD --author A [--root ROOT]` → writes a proposed Decision file
/// (auto NNNN + UUID), killing point-of-decision friction (D0054). Acceptance stays a separate human gate.
/// Parse a `--from FILE` decision draft (issue255): `key: value` headers, then `--- key` sections
/// whose body runs to the next `---` marker. Deliberately NOT TOML/JSON — a Decision's fields are
/// paragraphs, and a format that needs the author to escape quotes reintroduces the very problem
/// the file is here to remove.
///
/// ```text
/// slug: decision-authoring-is-core
/// date: 2026-08-24
/// marker: process-change
/// --- title
/// One line or many; every field takes prose verbatim.
/// --- context
/// Quotes, "double quotes", backticks and $VARIABLES are all literal here.
/// ```
fn decision_fields_from_file(path: &str) -> Result<std::collections::BTreeMap<String, String>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read --from {path}: {e}"))?;
    let mut out: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    let mut key: Option<String> = None;
    let mut body: Vec<&str> = Vec::new();
    let flush = |out: &mut std::collections::BTreeMap<String, String>, key: &Option<String>, body: &[&str]| {
        if let Some(k) = key {
            out.insert(k.clone(), body.join(" ").split_whitespace().collect::<Vec<_>>().join(" "));
        }
    };
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("--- ") {
            flush(&mut out, &key, &body);
            body.clear();
            key = Some(rest.trim().to_string());
        } else if key.is_some() {
            body.push(line);
        } else if let Some((k, v)) = line.split_once(':') {
            let (k, v) = (k.trim(), v.trim());
            if !k.is_empty() && !v.is_empty() && !k.starts_with('#') {
                out.insert(k.to_string(), v.to_string());
            }
        }
    }
    flush(&mut out, &key, &body);
    if out.is_empty() {
        return Err(format!("--from {path} declared no fields (expected `key: value` lines and `--- key` sections)"));
    }
    Ok(out)
}

/// D0396 / D0375 option C: accept a marker Decision at record time when a plan the human signed
/// THEMSELVES names its step, or leave it proposed naming the clause that failed. Returns the process
/// exit code; the caller returns it directly.
#[allow(clippy::too_many_arguments)] // the record context this runs inside
fn try_plan_cover(root: &Path, path: &str, dname: &str, nnnn: &str, date: &str, author: &str, plan_id: &str, step: &str) -> i32 {
    match keel_cli::plan_cover::assess(root, dname, plan_id, step) {
        keel_cli::plan_cover::Cover::Covered { judge } => {
            let sha = keel_cli::gitx::git().arg("-C").arg(root).args(["rev-parse", "--short", "HEAD"]).output().ok().and_then(|o| String::from_utf8(o.stdout).ok()).map(|s| s.trim().to_owned()).unwrap_or_default();
            let cover_note = keel_cli::plan_cover::note(plan_id, step, &judge);
            match keel_cli::write::accept_decision(std::path::Path::new(path), dname, &sha, date, &judge, author, &cover_note) {
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
        keel_cli::plan_cover::Cover::Refused { clause, detail } => {
            println!("D{nnnn} stays proposed - plan cover clause ({clause}) does not hold: {detail}. Accept with the human's own word (D0289), the console, or their terminal; or fix the plan / step name and re-record.");
            0
        }
    }
}

/// D0337 (the human's answer to D0324, 2026-09-05: 'standing consent is scoped only to the existing
/// processes under which it was promulgated'): a Decision that CHANGES the process or enforcement
/// surface - a process-change or safety-change marker - is outside the ground the consent stands on and
/// stays proposed for the human. Consent covers work WITHIN the existing processes; it does not cover
/// rewriting them. Exit 0: the record itself succeeded.
fn outside_standing_consent(marker: &str) -> i32 {
    println!("OUTSIDE standing consent (D0337): this Decision carries a {marker} marker - it changes the process or enforcement surface, and standing consent is scoped to the existing processes it was promulgated under. Stays proposed; the human accepts it with their quoted word (D0289), the console, or their terminal.");
    0
}

/// The consent-scope gate at record time (D0337, and issue460 / D0439). A MARKED Decision is outside
/// standing consent (`outside_standing_consent`). An UNMARKED one whose own text names the marker
/// vocabulary is held proposed exactly the same way - the text declared itself a process change, the
/// draft did not, and the consent covers neither; the mismatch is written into the record's header line,
/// the one acceptance rewrites, so the file says why it waited and the hold dies with the human's word.
/// One classifier (`deck::marker_words`) serves this hold and guard `consent-scope`, so a hand-edited
/// file cannot pass what the write path holds. `Some(0)` when the record stays proposed for the human (the
/// record itself succeeded); `None` when the text names nothing or no consent is declared - an unmarked
/// text then gets an advisory and the caller continues.
fn consent_scope_gate(root: &Path, path: &Path, nnnn: &str, marker: Option<&str>, fields: &[&str]) -> Option<i32> {
    let consent = keel_cli::activation::standing_consent(root);
    let Some(words) = keel_cli::deck::marker_text_without_marker(fields, marker.is_some()) else {
        return marker.filter(|_| consent.is_some()).map(outside_standing_consent);
    };
    let list = words.join(", ");
    if consent.is_none() {
        println!(
            "note: the text names {list} and the draft carries no marker line; with no standing consent declared it is proposed either way, but add `marker: process-change` (or `safety-change`) if it changes the process so the process-change guard can see it, or state `{}: <why>` (issue460)",
            keel_cli::deck::NOT_A_PROCESS_CHANGE
        );
        return None;
    }
    if let Err(e) = keel_cli::write::note_marker_hold(path, &list) {
        eprintln!("the hold could not be written into D{nnnn}'s header: {e} - it is proposed regardless");
    }
    println!(
        "HELD proposed (D0337/issue460): the text names {list} and the draft carries no marker line, so standing consent did not accept it - a Decision that says it changes the process is outside the consent whether or not it says so with a marker. Add `marker: process-change` (or `safety-change`) and re-record so the process-change guard sees it, state `{}: <why it changes no process>` in the text, or a human accepts it with their quoted word (D0289).",
        keel_cli::deck::NOT_A_PROCESS_CHANGE
    );
    Some(0)
}

/// The record-time acceptance under standing consent (D0291), for a NON-FORK with one declared
/// decider. issue376 / GH#57: the words quoted are the PROJECT's declared `standingWords`, never a
/// literal in the engine - with consent declared and no words the Decision stays proposed and says so.
#[allow(clippy::too_many_arguments)]
fn auto_accept_under_consent(root: &Path, path: &str, dname: &str, nnnn: &str, consent: &str, date: &str, author: &str) {
    let deciders: Vec<String> = keel_cli::github::deciders(root).into_values().collect();
    let Some(words) = keel_cli::activation::standing_words(root) else {
        println!("standing consent {consent} is declared but attestation-policy.toml records no standingWords - the human's consent is quoted from the project's own policy, never from the engine (issue376); D{nnnn} stays proposed. Record their words verbatim as standingWords beside standingConsent, or accept with their quoted word (D0289).");
        return;
    };
    if let [judge] = deciders.as_slice() {
        let sha = keel_cli::gitx::git().arg("-C").arg(root).args(["rev-parse", "--short", "HEAD"]).output().ok()
            .filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
        let note = format!("AUTO-ACCEPTED under standing consent ({}). Their standing words, verbatim: '{words}' Not individually reviewed; override by a superseding Decision (D0290) or the human's quoted word in chat (D0289).", consent.to_uppercase());
        // Recorded by the actor running `record decision`, judged by the standing decider:
        // a DELEGATED record by construction, so the substance rule reads its quote.
        match keel_cli::write::accept_decision(Path::new(path), dname, &sha, date, judge, author, &note) {
            Ok(_) => println!("accepted D{nnnn} at record time under standing consent {consent} (non-fork; judge {judge}; override by a superseding Decision or your quoted word)"),
            Err(e) => eprintln!("standing consent {consent} declared but the acceptance could not be recorded: {e} - D{nnnn} stays proposed"),
        }
    } else {
        println!("standing consent {consent} declared but github-actors.toml names {} decider(s), not one - D{nnnn} stays proposed; accept with your quoted word (D0289)", deciders.len());
    }
}

/// D0451: the authoring family's nine verbs are sub-verbs of `record`, each named for the fact it
/// writes and each keeping its flags. `sprint` keeps its own word: `cmd_new` reads it.
fn record_authoring_subverb(args: &[String]) -> Option<i32> {
    let rest = args.get(1..).unwrap_or(&[]);
    Some(match args.first().map(String::as_str)? {
        "task" => cmd_add_task(rest),
        "sprint" => cmd_new(args),
        "result" => cmd_append_result(rest),
        "gate-result" => cmd_append_gate_result(rest),
        "review" => cmd_apply_review(rest),
        "measurement" => cmd_record_measurement(rest),
        "indicator-snapshot" => cmd_snapshot_indicators(rest),
        "reverify" => cmd_reverify(rest),
        "mint" => cmd_mint(rest),
        _ => return None,
    })
}

/// The `keel record` usage, one line per fact the router writes.
fn print_record_usage() {
    eprintln!("usage: keel record decision --slug S --title T --context C --decision D --rationale R --consequences Q --date YYYY-MM-DD --author A [--root ROOT]");
    eprintln!("       keel record decision --from DRAFT.md   (prose in a file - the sanctioned path, issue255; flags override)");
    eprintln!("           [--supersedes dNNNN[,..]] retires each target whole | [--supersedes-clause dNNNN[,..]] reverses one clause, target stays in force (D0398); draft lines `supersedes:` / `supersedes-clause:`");
    eprintln!("       keel record issue --title T --description D --severity Critical|High|Medium|Low --resolver R --date YYYY-MM-DD [--related-task T] [--marker M] [--in-field] [--by A] [--root ROOT]");
    eprintln!("       keel record statement --text \"<their exact words>\" | --from FILE --said-by A --said-at D --title T [--channel C]   (VERBATIM, D0216/D0236)");
    eprintln!("       keel record story --from-statement stNNN --title T --as-a R --i-want C --implication K [--so-that O] [--triage-note W] --at D");
    eprintln!("       keel record task|sprint|result|gate-result|review|measurement|indicator-snapshot|reverify|mint ...   (D0451: the nine authoring verbs under one router, each named for the fact it writes; flags unchanged - `keel record <sub-verb>` alone prints its usage)");
}

fn cmd_record(args: &[String]) -> i32 {
    // D0236: intake had NO write path. Every Statement in this repo was hand-edited into a file,
    // which is the one record type where that matters most - D0216 requires the human's words
    // VERBATIM before any Need exists, and a hand-typed "verbatim" field is a paraphrase waiting
    // to happen.
    match args.first().map(String::as_str) {
        Some("issue") => return cmd_record_issue(args),
        Some("statement") => return cmd_record_statement(args),
        Some("story") => return cmd_record_story(args),
        _ => {}
    }
    if let Some(code) = record_authoring_subverb(args) {
        return code;
    }
    if args.first().map(String::as_str) != Some("decision") {
        print_record_usage();
        return 2;
    }
    let root = flag(args, "root").map_or_else(
        || find_repo_root().unwrap_or_else(|| PathBuf::from(".")),
        PathBuf::from,
    );
    // issue255: `--from FILE` is the SANCTIONED authoring path for a Decision's prose. Passing five
    // paragraphs as double-quoted shell arguments is how ~2000 characters of `keel hardening` output
    // ended up inside D0223's `decision` field: a backtick inside double quotes is command
    // substitution, so the shell RAN the command the prose merely named. A file has no such layer.
    // The write path refuses tool-output-shaped prose either way (`reject_injected_output`); this is
    // the road that makes the refusal easy to obey rather than a rule to remember (D0054).
    let from_file = match flag(args, "from").map(|f| decision_fields_from_file(&f)) {
        Some(Ok(fields)) => Some(fields),
        Some(Err(msg)) => { eprintln!("error: {msg}"); return 2; }
        None => None,
    };
    let req = |name: &str| {
        flag(args, name).or_else(|| from_file.as_ref().and_then(|m| m.get(name).cloned()))
    };
    let (Some(slug), Some(title), Some(context), Some(decision), Some(rationale), Some(consequences)) =
        (req("slug"), req("title"), req("context"), req("decision"), req("rationale"), req("consequences"))
    else {
        eprintln!("error: --slug --title --context --decision --rationale --consequences are all required (a substantive why — D0103)");
        return 2;
    };
    let date = req("date").unwrap_or_default();
    // NEVER default to a named human (D0129/issue072): that silently forges a human attestation.
    let author = match keel_cli::actor::resolve(&root, req("author").as_deref()) {
        Ok(a) => a,
        Err(msg) => { eprintln!("{msg}"); return 2; }
    };
    if date.is_empty() {
        eprintln!("error: --date YYYY-MM-DD required (the attestation time is its own irreducible fact)");
        return 2;
    }
    // issue213: the D0070 marker as a first-class flag - forgetting it cost two landing commits.
    let from_marker = from_file.as_ref().and_then(|m| m.get("marker")).map(String::as_str);
    // An unknown `marker:` used to be accepted SILENTLY, producing an unmarked Decision - which is how
    // `marker: prospective-change` (a plausible spelling) yielded a Decision the process-change guard
    // could not see (issue346). The vocabulary is two words; anything else is refused by name.
    if let Some(m) = from_marker {
        if m != "process-change" && m != "safety-change" {
            eprintln!("error: unknown marker `{m}` - the marker vocabulary is `process-change` (emits #ProspectiveChange) or `safety-change` (emits #SafetyChange); an unrecognised value would have produced an UNMARKED Decision silently");
            return 2;
        }
    }
    let marker = if args.iter().any(|a| a == "--process-change") || from_marker == Some("process-change") {
        Some("ProspectiveChange")
    } else if args.iter().any(|a| a == "--safety-change") || from_marker == Some("safety-change") {
        Some("SafetyChange")
    } else {
        None
    };
    let research = req("research");
    // D0352: the edges a Decision is recorded WITH - `--supersedes d0001,d0002` / a `supersedes:` line,
    // `--derived-from st001` / a `derived-from:` line - so a reversal cannot land edgeless.
    let list = |name: &str| -> Vec<String> { req(name).map(|v| v.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()).unwrap_or_default() };
    let links = w::DecisionLinks { supersedes: list("supersedes"), supersedes_clause: list("supersedes-clause"), derived_from: list("derived-from") };
    match w::record_decision_with_links(&root, &slug, &title, &date, &author, &context, &decision, &rationale, &consequences, marker, research.as_deref(), &links) {
        Err(e @ w::WriteError::InjectedToolOutput(..)) => refused_injected_prose(&root, &e),
        Ok((nnnn, path)) => {
            println!("recorded D{nnnn} (proposed) -> {path}");
            for d in &links.supersedes {
                println!("  #Supersede d{nnnn} -> {d} authored with it (D0352): {d} is RETIRED whole");
            }
            for d in &links.supersedes_clause {
                println!("  #SupersedeClause d{nnnn} -> {d} authored with it (D0398): one clause reversed, {d} stays in force");
            }
            for t in &links.derived_from {
                println!("  #DerivedFrom d{nnnn} -> {t} authored with it (D0352)");
            }
            // D0291: standing consent (D0207) is applied HERE, at record time, for a NON-FORK - the
            // GitHub channel that used to do it at issue creation (and notify the human every time) is
            // disconnected. A fork (a Decision carrying OPTIONS) stays proposed for the human; the
            // recorder is the single declared decider, and if the deciders table does not name exactly
            // one Person the Decision stays proposed and says why rather than guessing a judge.
            let dname = format!("d{nnnn}");
            let rel = path.replace('\\', "/");
            let rel = rel.strip_prefix(&format!("{}/", root.to_string_lossy().replace('\\', "/"))).unwrap_or(&rel).to_string();
            // D0322 / issue373 (stpa-self UCA-R1): a Decision that WEIGHS alternatives in prose without
            // the OPTION marker is a fork in substance; standing consent must not accept it on the spot.
            // Two distinct signals in the decision text hold it proposed and say which words; the
            // author writes it as a fork or states `NOT A FORK` in the text.
            let disguised = keel_cli::deck::disguised_fork(&decision);
            // D0396 / D0375 option C: a marker Decision naming a STEP of a plan the human signed
            // themselves is covered by that signature - the human signed once, on the plan, and its
            // enumerated steps do not re-ask. Checked BEFORE standing consent, because the cover flows
            // through a human's own signature on the plan, not through consent (which never reaches the
            // enforcement surface, D0337).
            if let (Some(_m), Some(plan_id), Some(step)) = (marker, req("plan"), req("step")) {
                return try_plan_cover(&root, &path, &dname, &nnnn, &date, &author, &plan_id, &step);
            }
            // issue460 / D0439: the TEXT says what the draft did not - `process-change`, `#ProspectiveChange`
            // - and no marker line was given. D0432's consequences read 'Process-change (D0337)' and it
            // AUTO-ACCEPTED, because nothing read the text.
            // D0337: a marker Decision is outside standing consent either way (consent_scope_gate).
            if let Some(code) = consent_scope_gate(&root, Path::new(&path), &nnnn, marker, &[&context, &decision, &rationale, &consequences]) {
                return code;
            }
            match (keel_cli::activation::standing_consent(&root), keel_cli::deck::fork_options(&root, &rel).is_empty(), disguised) {
                (Some(consent), true, Some(signals)) => {
                    println!(
                        "HELD as a fork in substance: the decision text weighs alternatives ({}) without the `OPTION X (label)` marker, so standing consent {consent} does not apply (D0322/issue373). Write it as a fork (OPTION A (label) ... COST ...; OPTION B ...), or state `{}: <why it chooses one course>` in the text and re-record; a human may still accept it with their quoted word (D0289).",
                        signals.join(", "),
                        keel_cli::deck::NOT_A_FORK
                    );
                }
                (Some(consent), true, None) => auto_accept_under_consent(&root, &path, &dname, &nnnn, &consent, &date, &author),
                (Some(consent), false, _) => println!("a FORK (carries options): stays proposed under standing consent {consent} - the human chooses; surface it (decision-surfacing) and accept with their quoted word (D0289)."),
                (None, _, _) => println!("accept later via an explicit human sign-off (a quoted word in chat under the D0192 delegation, the console, or your terminal)."),
            }
            0
        }
        Err(e) => {
            eprintln!("error: {e}");
            1
        }
    }
}

/// `keel record statement --text "<their exact words>" --said-by A --said-at D --channel C --title T`
///
/// `--text` is passed through VERBATIM (escaped for the literal, otherwise untouched), so it must be
/// their words and not a summary; `--title` is the AI's label and is sanitised like any other field.
/// `--from FILE` reads the text from a file, which is the sanctioned path for anything containing
/// quotes, newlines or backticks - the same reason `record decision --from` exists (D0224/issue255).
fn cmd_record_statement(args: &[String]) -> i32 {
    let root = flag(args, "root").map_or_else(
        || find_repo_root().unwrap_or_else(|| PathBuf::from(".")),
        PathBuf::from,
    );
    let from_file = flag(args, "from").map(std::fs::read_to_string);
    let text = match from_file {
        Some(Ok(t)) => Some(t.trim_end_matches(['\n', '\r']).to_string()),
        Some(Err(e)) => {
            eprintln!("error: cannot read --from: {e}");
            return 2;
        }
        None => flag(args, "text"),
    };
    let (Some(text), Some(said_by), Some(said_at), Some(title)) =
        (text, flag(args, "said-by"), flag(args, "said-at"), flag(args, "title"))
    else {
        eprintln!("usage: keel record statement --text \"<their exact words>\" | --from FILE");
        // Derived, not restated (issue300): the usage line a user reads is the same vocabulary
        // surface as the check, so it must come from the same source or it will drift from it.
        eprintln!(
"       --said-by ACTOR --said-at YYYY-MM-DD --title T [--channel {}]",
            keel_cli::schema::enum_members_union(&root, "StatementChannel").join("|")
        );
        eprintln!("       [--by RECORDER] [--at YYYY-MM-DD] [--root ROOT]");
        eprintln!("  --text is VERBATIM (D0216): their words, not a summary. --title is your label for it.");
        return 2;
    };
    let channel = flag(args, "channel").unwrap_or_else(|| "chat".to_string());
    let author = match keel_cli::actor::resolve(&root, flag(args, "by").as_deref()) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("{msg}");
            return 2;
        }
    };
    // Provenance is never defaulted (D0129/issue182): the RECORD's date is its own fact, separate
    // from when they said it.
    let created_at = flag(args, "at").unwrap_or_else(|| said_at.clone());
    match keel_cli::intake_write::record_statement(
        &root,
        &keel_cli::intake_write::NewStatement {
            text: &text,
            said_by: &said_by,
            said_at: &said_at,
            channel: &channel,
            source_url: flag(args, "source-url").as_deref(),
            // A statement typed by the operator carries no external source, so no tier applies.
            source_trust: None,
            title: &title,
            author: &author,
            created_at: &created_at,
        },
    ) {
        Ok((name, path)) => {
            println!("recorded {name} -> {path}");
            println!("  their words are stored VERBATIM. Next: `keel record story --from-statement {name} ...`");
            0
        }
        Err(e) => {
            eprintln!("error: {e}");
            1
        }
    }
}

/// `keel record story --from-statement stNNN --title T --as-a R --i-want C --implication K`
///
/// The `#DerivedFrom` edge to the cited `Statement` is authored WITH the story, and a story citing a
/// `Statement` that does not exist is REFUSED with nothing written: a `UserStory` with no source is an
/// invention wearing a story's clothes (D0216).
fn cmd_record_story(args: &[String]) -> i32 {
    let args = &match prose_args(args, &["i-want", "so-that", "triage-note"], "record story") {
        Ok(a) => a,
        Err(code) => return code,
    };
    let root = flag(args, "root").map_or_else(
        || find_repo_root().unwrap_or_else(|| PathBuf::from(".")),
        PathBuf::from,
    );
    let (Some(from), Some(title), Some(as_a), Some(i_want), Some(implication)) = (
        flag(args, "from-statement"),
        flag(args, "title"),
        flag(args, "as-a"),
        flag(args, "i-want"),
        flag(args, "implication"),
    ) else {
        eprintln!("usage: keel record story --from-statement stNNN --title T --as-a ROLE (--i-want-from FILE | --i-want CAPABILITY)");
        // Derived, not restated (issue300) — see the note at the `record statement` usage.
        eprintln!(
"       --implication {}",
            keel_cli::schema::enum_members_union(&root, "ImplicationKind").join("|")
        );
        eprintln!("       [--so-that-from FILE | --so-that OUTCOME] [--triage-note-from FILE | --triage-note WHY] [--by RECORDER] [--at YYYY-MM-DD] [--root ROOT]");
        eprintln!("  Prose goes through a FILE (D0224): a backtick in a double-quoted shell argument is a command the shell runs into the record.");
        eprintln!("  --from-statement is REQUIRED: a UserStory with no cited source is an invention (D0216).");
        return 2;
    };
    let author = match keel_cli::actor::resolve(&root, flag(args, "by").as_deref()) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("{msg}");
            return 2;
        }
    };
    let Some(created_at) = flag(args, "at") else {
        eprintln!("error: --at YYYY-MM-DD required (the record's date is its own irreducible fact, issue182)");
        return 2;
    };
    let so_that = flag(args, "so-that");
    let triage = flag(args, "triage-note");
    match keel_cli::intake_write::record_story(
        &root,
        &keel_cli::intake_write::NewStory {
            from_statement: &from,
            title: &title,
            as_a: &as_a,
            i_want: &i_want,
            so_that: so_that.as_deref(),
            implication: &implication,
            triage_note: triage.as_deref(),
            author: &author,
            created_at: &created_at,
        },
    ) {
        Ok((name, path)) => {
            println!("recorded {name} -> {path}  (#DerivedFrom {from} authored with it)");
            0
        }
        Err(e) => {
            eprintln!("error: {e}");
            1
        }
    }
}

fn cmd_add_task(args: &[String]) -> i32 {
    let Some(file_str) = flag(args, "file") else {
        eprintln!("usage: keel record task --file FILE --def DEF --task TASK --method METHOD");
        eprintln!("       --dod-from FILE   the criterion, read from a file (PREFER THIS)");
        eprintln!("       --dod TEXT        the criterion inline — a shell EXECUTES backticks in it");
        return 2;
    };
    let Some(def_name) = flag(args, "def") else {
        eprintln!("error: --def required");
        return 2;
    };
    let Some(task) = flag(args, "task") else {
        eprintln!("error: --task required");
        return 2;
    };
    // PROSE COMES FROM A FILE (D0224, now extended to the third and last write path that took it by
    // argument). `record decision` got `--from` after shell backticks EXECUTED into a governance
    // record twice; `record issue` got `--description-from` after a third occurrence. `add-task`
    // was left on the argument path and the trap fired a FOURTH time, eating `keel show <lens>` out
    // of a DoD and leaving the sentence "one  routing to the existing implementations". Fixing two
    // of three call sites is what let this recur - and it recurred a FIFTH time into `result`'s
    // evidence (issue543), which is why every prose input now goes through `prose_flag`.
    let dod = match prose_flag(args, "dod", "record task") {
        Ok(d) => d,
        Err(msg) => { eprintln!("error: {msg}"); return 2; }
    };
    let Some(dod) = dod else {
        eprintln!("error: --dod required");
        return 2;
    };
    let file = PathBuf::from(file_str);
    let method = flag(args, "method").unwrap_or_else(|| "test".to_owned());

    match w::add_task(&file, &def_name, &task, &dod, &method) {
        Ok(uuid) => { println!("{uuid}"); 0 }
        Err(e) => { eprintln!("error: {e}"); 1 }
    }
}

/// `render <view> [--mode graph|table|review] [--root ROOT]` — modular interactive-artifact
/// renderer over the view layer (D0086). Emits self-contained HTML to stdout (redirect to a file).
/// `keel render <what>` — one verb for everything that draws (D0449). The first positional resolves
/// in a FIXED order, held by `cli_surface::RENDER_RESERVED` and its test: the reserved words `model`
/// | `all` | `whole` (the whole-model graph, once `keel diagram`), `report <kind>` (once `keel report`),
/// `decision-card [NAME]` (once `keel decision-card`), then `control-structure`, then a declared
/// `.view.toml`. The order is fixed because a declared view named `report` shadowing the sub-verb — or
/// the reverse — is a silent wrong answer, the class sprint 512 met in the console's view binder.
fn cmd_render(args: &[String]) -> i32 {
    let Some(view) = args.first().filter(|v| !v.starts_with('-')) else {
        eprintln!("usage: keel render <view>|model [--mode graph|table|review] [--root ROOT]");
        eprintln!("       keel render report <assurance|traceability|quality-debt|flow|governance|friction> [--html] [--trend] [--root ROOT]");
        eprintln!("       keel render decision-card [NAME] [--proposed]");
        eprintln!("  <view> = a declared view name (e.g. decisions, issues), or 'model' for the whole-model graph");
        return 2;
    };
    // The two sub-verbs with their own argument shapes: the arms are MOVED, not rewritten, so the
    // output is byte-equal to the pre-fold verbs (D0449's criterion). `args[1..]` is what each saw.
    let rest = args.get(1..).unwrap_or_default();
    match view.as_str() {
        "report" => return cmd_report(rest),
        "decision-card" => return cmd_decision_card(rest),
        _ => {}
    }
    let mode = flag(args, "mode").unwrap_or_else(|| "graph".to_owned());
    let root = match flag(args, "root") {
        Some(p) => PathBuf::from(p),
        None => {
            if let Some(r) = find_repo_root() {
                r
            } else {
                eprintln!("error: no .engine/ found from cwd upward; pass --root ROOT");
                return 2;
            }
        }
    };
    match keel_cli::view::render_html(&root, view, &mode) {
        Ok(html) => {
            println!("{html}");
            0
        }
        Err(e) => {
            eprintln!("render error: {e}");
            1
        }
    }
}

/// `render report <name> [--html] [--root ROOT]` — computed aggregate scorecard (D0087): assurance |
/// traceability | quality-debt | flow. JSON by default; `--html` emits a human-digestible scorecard.
/// Reached through `cmd_render` since D0449; the arm itself is unchanged.
fn cmd_report(args: &[String]) -> i32 {
    let Some(name) = args.first().filter(|v| !v.starts_with('-')) else {
        eprintln!("usage: keel render report <assurance|traceability|quality-debt|flow|governance|friction> [--html] [--trend] [--root ROOT]");
        return 2;
    };
    let root = match flag(args, "root") {
        Some(p) => PathBuf::from(p),
        None => {
            if let Some(r) = find_repo_root() {
                r
            } else {
                eprintln!("error: no .engine/ found from cwd upward; pass --root ROOT");
                return 2;
            }
        }
    };
    let html = args.iter().any(|a| a == "--html");
    let trend = args.iter().any(|a| a == "--trend");
    let result = if html { keel_cli::reports::report_html(&root, name, trend) } else { keel_cli::reports::report(&root, name, trend) };
    match result {
        Ok(out) => {
            println!("{out}");
            0
        }
        Err(e) => {
            eprintln!("report error: {e}");
            1
        }
    }
}

/// `indicators [--trend] [--root ROOT]` — monitored measures (D0089) with direction-aware status.
/// Computed indicators show current value (full series with `--trend`); pulled/manual show their
/// recorded-Measurement series + status.
fn cmd_indicators(args: &[String]) -> i32 {
    // issue281: this accepted `--root` ONLY, so a POSITIONAL root was silently ignored and the view
    // was computed against `find_repo_root()` — whatever project the process happened to be standing
    // in. `keel indicators <someWorkspace>` therefore reported on a DIFFERENT project and exited 0,
    // which is worse than the zeroed-green this issue is about: the numbers are real, just about
    // something else. Found by a test sweeping every model reader, not by reading the code. Going
    // through `root_arg` gives it the positional, the unknown-flag refusal, and the project
    // precondition that the other model readers already have.
    let root = match root_arg(args, "keel indicators [ROOT] [--trend] [--root ROOT]", &["trend", "root"], 0) {
        Ok(r) => flag(args, "root").map_or(r, PathBuf::from),
        Err(code) => return code,
    };
    if let Err(code) = keel_cli::workspace::require_project(&root, "keel indicators [ROOT] [--trend]") {
        return code;
    }
    let trend = args.iter().any(|a| a == "--trend");
    match keel_cli::view::indicators(&root, trend) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(e) => {
            eprintln!("indicators error: {e}");
            1
        }
    }
}

/// `record-measurement --indicator I --value V [--at DATE] [--source S] [--by ACTOR] [--file F]` —
/// record a Measurement datapoint (D0089) for a pulled/manual indicator (write path).
fn cmd_record_measurement(args: &[String]) -> i32 {
    let Some(indicator) = flag(args, "indicator") else {
        eprintln!("usage: keel record measurement --indicator I --value V [--at DATE] [--source S] [--by ACTOR] [--file F]");
        return 2;
    };
    let Some(value) = flag(args, "value") else {
        eprintln!("error: --value required");
        return 2;
    };
    let file = flag(args, "file").map_or_else(
        || find_repo_root().map_or_else(|| PathBuf::from(".tracking/indicators.sysml"), |r| r.join(".tracking").join("indicators.sysml")),
        PathBuf::from,
    );
    let at = match provenance_date(args, "at", "keel <write> --at YYYY-MM-DD ...") {
        Ok(d) => d,
        Err(c) => return c,
    };
    let source = flag(args, "source").unwrap_or_default();
    let by = match keel_cli::actor::resolve(&keel_cli::actor::root_for(&file), flag(args, "by").as_deref()) {
        Ok(a) => a,
        Err(msg) => { eprintln!("{msg}"); return 2; }
    };
    match w::append_measurement(&file, &indicator, &value, &at, &source, &by) {
        Ok(name) => {
            println!("{name}");
            0
        }
        Err(e) => {
            eprintln!("error: {e}");
            1
        }
    }
}

/// `snapshot-indicators [--at DATE] [--by ACTOR] [--file F] [--root ROOT]` — take a reading of every
/// COMPUTED indicator (its current `metric_value`) and bank it as a `Measurement` (D0091). Run per
/// sprint/quarter to build a durable, fast series alongside the pulled/manual observations.
fn cmd_snapshot_indicators(args: &[String]) -> i32 {
    let root = match flag(args, "root") {
        Some(p) => PathBuf::from(p),
        None => {
            if let Some(r) = find_repo_root() {
                r
            } else {
                eprintln!("error: no .engine/ found from cwd upward; pass --root ROOT");
                return 2;
            }
        }
    };
    let file = flag(args, "file").map_or_else(|| root.join(".tracking").join("indicators.sysml"), PathBuf::from);
    let at = match provenance_date(args, "at", "keel <write> --at YYYY-MM-DD ...") {
        Ok(d) => d,
        Err(c) => return c,
    };
    let by = match keel_cli::actor::resolve(&root, flag(args, "by").as_deref()) {
        Ok(a) => a,
        Err(msg) => { eprintln!("{msg}"); return 2; }
    };
    let keys = match keel_cli::view::computed_indicator_keys(&root) {
        Ok(k) => k,
        Err(e) => {
            eprintln!("error: {e}");
            return 1;
        }
    };
    let mut count = 0u32;
    for (indicator, key) in &keys {
        let Some(v) = keel_cli::view::metric_value(&root, key) else {
            eprintln!("skip {indicator}: metric '{key}' not computable");
            continue;
        };
        match w::append_measurement(&file, indicator, &format!("{v:.6}"), &at, "snapshot (computed reading)", &by) {
            Ok(name) => {
                println!("{name}  ({indicator} = {v:.2})");
                count += 1;
            }
            Err(e) => {
                eprintln!("error on {indicator}: {e}");
                return 1;
            }
        }
    }
    println!("banked {count} computed-indicator snapshot(s) @ {at} into {}", file.display());
    0
}

#[derive(serde::Deserialize)]
struct ReviewBatch {
    #[serde(default)]
    dispositions: Vec<ReviewDisp>,
    #[serde(default, rename = "judgedBy")]
    judged_by: String,
    #[serde(default, rename = "judgedAgainst")]
    judged_against: String,
}

#[derive(serde::Deserialize)]
struct ReviewDisp {
    element: String,
    verdict: String,
    #[serde(default)]
    lens: String,
    #[serde(default)]
    severity: String,
    #[serde(default)]
    rationale: String,
    #[serde(default)]
    actionable: bool,
}

/// `record review --batch FILE [--sha SHA] [--judged-by ACTOR] [--judged-at DATE] [--root ROOT]` —
/// ingest a review batch exported by `render --mode review` and write each disposition back as a new
/// linked critique (D0086) via the write path. `accept`->pass, `finding`/`reject`->fail (a finding,
/// which induces computed suspicion). Writes into `.tracking/critiques.sysml`.
fn cmd_apply_review(args: &[String]) -> i32 {
    let Some(batch_str) = flag(args, "batch") else {
        eprintln!("usage: keel record review --batch FILE [--sha SHA] [--judged-by ACTOR] [--judged-at DATE] [--root ROOT]");
        return 2;
    };
    let root = match flag(args, "root") {
        Some(p) => PathBuf::from(p),
        None => {
            if let Some(r) = find_repo_root() {
                r
            } else {
                eprintln!("error: no .engine/ found from cwd upward; pass --root ROOT");
                return 2;
            }
        }
    };
    let text = match std::fs::read_to_string(&batch_str) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error reading batch {batch_str}: {e}");
            return 2;
        }
    };
    let batch: ReviewBatch = match serde_json::from_str(&text) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: invalid review batch JSON: {e}");
            return 2;
        }
    };
    let judged_by = flag(args, "judged-by").filter(|s| !s.is_empty()).or_else(|| Some(batch.judged_by.clone()).filter(|s| !s.is_empty())).unwrap_or_else(|| "human".to_owned());
    let sha = flag(args, "sha").filter(|s| !s.is_empty()).or_else(|| Some(batch.judged_against.clone()).filter(|s| !s.is_empty())).unwrap_or_else(|| "uncommitted".to_owned());
    let judged_at = match provenance_date(args, "judged-at", "keel <write> --judged-at YYYY-MM-DD ...") {
        Ok(d) => d,
        Err(c) => return c,
    };
    // issue210: apply-review's records land in the JUDGE's per-actor file (forward-only routing).
    let Ok(critiques) = w::per_actor_file(&root, "critiques", &judged_by) else {
        eprintln!("error: cannot open the per-actor critiques file");
        return 1;
    };

    let mut count = 0u32;
    for d in &batch.dispositions {
        // Finding disposition (D0092): act / accept-risk / dismiss target a finding ISSUE, written as a
        // method=confirmation disposition (#Dispositions-linked), not a critique.
        if let Some(verdict) = match d.verdict.as_str() {
            "act" => Some("act"),
            "accept-risk" | "acceptRisk" => Some("acceptRisk"),
            "dismiss" => Some("dismiss"),
            _ => None,
        } {
            let disp = w::Disposition { finding: &d.element, verdict, rationale: &d.rationale, sha: &sha, judged_at: &judged_at, judged_by: &judged_by };
            match w::append_disposition(&critiques, &disp) {
                Ok(name) => {
                    println!("{name}  ({} disposition:{verdict})", d.element);
                    count += 1;
                }
                Err(e) => {
                    eprintln!("error on {}: {e}", d.element);
                    return 1;
                }
            }
            continue;
        }
        let outcome = match d.verdict.as_str() {
            "accept" => "pass",
            "finding" | "reject" => "fail",
            other => {
                eprintln!("skip {}: unknown verdict '{other}'", d.element);
                continue;
            }
        };
        let severity = (outcome == "fail" && !d.severity.is_empty()).then_some(d.severity.as_str());
        let lens = if d.lens.is_empty() { "correctness" } else { d.lens.as_str() };
        let mut rationale = d.rationale.clone();
        if d.actionable {
            rationale.push_str(" [actionable: warrants new implementation]");
        }
        let c = w::Critique {
            element: &d.element,
            method: "critique",
            lens,
            critiqued_by: "human",
            severity,
            rationale: &rationale,
            outcome,
            sha: &sha,
            judged_at: &judged_at,
            judged_by: &judged_by,
        };
        match w::append_critique(&critiques, &c) {
            Ok(name) => {
                println!("{name}  ({} {})", d.element, outcome);
                count += 1;
            }
            Err(e) => {
                eprintln!("error on {}: {e}", d.element);
                return 1;
            }
        }
    }
    println!("applied {count} disposition(s) to {}", critiques.display());
    0
}

/// Write one embedded engine file into `dst_engine`, remapping `decisions/*` -> `reference/decisions/*`
/// (read-only reference, NOT instance — the engine's architecture decisions must not enter the new
/// project's computed views, which scan `.engine/decisions`; D0093 engine/instance boundary).
// The scaffold path rules live in `migrate` — `keel migrate` resyncs a downstream `.engine/` from
// this same embedded tree, so it must map and exclude paths identically to `keel init` or a migrated
// project would differ from a freshly inited one. One definition, both callers.
use keel_cli::migrate::{is_engine_dev_only, remap_engine_content, remap_engine_path};

/// The `.engine/contracts/` files that are THIS project's instance data rather than engine definition
/// (issue243). `keel init` must reset these, not copy them: an adoption declaration and a set of
/// exchange identities belong to the project that made them.
fn is_instance_contract(rel: &Path) -> bool {
    matches!(
        rel.to_string_lossy().replace('\\', "/").as_str(),
        "contracts/activation.toml"
            | "contracts/unit-ids.toml"
            | "contracts/installed-units.toml"
            // D0219: WHO MAY DECIDE is the most consequential instance fact in the tree, and init
            // was shipping it verbatim — so every new project silently inherited THIS project's
            // decider and would have recorded acceptances in their name. Reset to an empty template.
            | "contracts/github-actors.toml"
    )
}

/// The starter content for a reset instance contract. `activation.toml` gets a commented-out template
/// so the honest default (absent section = everything active, D0138) is what a fresh project HAS,
/// while still showing how to declare a subset. The id registries start genuinely empty: an identity
/// is minted on first export, never inherited.
fn starter_for(rel: &Path) -> &'static str {
    if rel.to_string_lossy().replace('\\', "/") == "contracts/github-actors.toml" {
        return "# github-actors - GitHub login -> keel actor mapping (D0205 githubChannel).\n\
#\n\
# THIS TABLE IS WHO MAY DECIDE ON THIS PROJECT, and it starts EMPTY on purpose (D0219): inheriting\n\
# another project's decider would let their login record acceptances in your tree. An unmapped login\n\
# is REFUSED, never defaulted (issue182: provenance is never defaulted).\n\
#\n\
# Add one line per human who may decide here. Logins are matched exactly and case-sensitively as\n\
# GitHub reports them. Only HUMANS belong here: this table exists to attribute human judgment, and\n\
# mapping a bot login would recreate the AI-recorded-as-human class (issue072/073) at the channel\n\
# layer. The repo OWNER is not automatically a decider, and an ORG can never be one - an org is not\n\
# a person and cannot hold judgment.\n\
#\n\
# Check yours with `keel github decider <login>`.\n\
\n\
[logins]\n\
# yourGithubLogin = \"yourKeelActor\"\n";
    }
    if rel.to_string_lossy().replace('\\', "/") == "contracts/activation.toml" {
        return "# Process activation (D0138) - which processes THIS project has adopted.
#
# NO SECTION BELOW MEANS EVERYTHING IS ACTIVE, which is the honest default for a new project: a
# project that never adopted a control has not violated it (issue090). Declare a subset only when
# you have actually chosen one - `keel activate <process>` / `keel deactivate <process>` write it
# for you, and `keel process list` shows what there is to choose from.
#
# CORE guards (identity, provenance, vocabulary, well-formedness) are in no unit and CANNOT be
# deactivated. Activation stops enforcing procedures you have not adopted; it does not make
# truthfulness optional.
#
# [processes]
# active = [\"agile-workflow\"]
#
# [viewpoints]
# active = [\"orientVP\"]
";
    }
    "# Process-unit identity registry (D0183): a unit's id is its EXCHANGE identity - stable across
# exports, matched by `import --update`. Minted on THIS project's first export; never inherited,
# because an inherited id claims a lineage this project does not have (issue243).
"
}

fn write_engine_file(f: &include_dir::File, dst_engine: &Path, count: &mut u32) -> std::io::Result<()> {
    let rel = f.path();
    if is_engine_dev_only(rel) {
        return Ok(()); // engine-dev-only (kernel/python toolchain) — not shipped to downstream projects
    }
    let dst = dst_engine.join(remap_engine_path(rel));
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // The deliverable-suspicion manifest is instance-specific (lists the ENGINE's own tasks) — reset it
    // to a starter so a fresh project passes manifest-coverage (D0093 engine/instance boundary).
    if rel == Path::new("deliverable-manifest.txt") {
        std::fs::write(&dst, STARTER_MANIFEST)?;
    } else if rel == Path::new("contracts/attestation-policy.toml") {
        // issue376 / GH#57: the policy ships, its GRANT lines do not - a standing consent or a recording
        // delegation is the receiving project's human's to give, and an inherited one had every fresh
        // project auto-accepting in its decider's name quoting words they never said.
        std::fs::write(&dst, keel_cli::activation::without_grants(std::str::from_utf8(f.contents()).unwrap_or_default()))?;
    } else if is_instance_contract(rel) {
        // issue243: these three contracts are THIS project's instance data, not engine definition, and
        // shipping them verbatim made every new project inherit the self-build's choices. A fresh tree
        // reported `declared manifest: yes` for an adoption declaration it never made — falsifying
        // CLAUDE.md's own guarantee that an absent file means everything is active, so a project that
        // never adopted a control has not violated it. The file was not absent; it was someone else's.
        // unit-ids/installed-units are worse than misleading: they are the self-build's EXCHANGE
        // IDENTITIES for units the new project never exported, so an `import --update` could match
        // against a lineage that is not its own. Reset to a starter, exactly as the manifest already is.
        std::fs::write(&dst, starter_for(rel))?;
    } else if let Some(text) = std::str::from_utf8(f.contents())
        .ok()
        .and_then(|s| remap_engine_content(rel, s))
    {
        // issue291: the reference copy's package is renamed so the project's own first decision
        // cannot collide with it. See `remap_engine_content`.
        std::fs::write(&dst, text)?;
    } else {
        std::fs::write(&dst, f.contents())?;
    }
    *count += 1;
    Ok(())
}

/// Recursively scaffold the embedded engine tree into `dst_engine`. `include_dir`'s `File::path()` is
/// root-relative, so the remap in `write_engine_file` sees the full path regardless of nesting.
fn scaffold_engine(dir: &include_dir::Dir, dst_engine: &Path, count: &mut u32) -> std::io::Result<()> {
    for f in dir.files() {
        write_engine_file(f, dst_engine, count)?;
    }
    for d in dir.dirs() {
        scaffold_engine(d, dst_engine, count)?;
    }
    Ok(())
}

/// `keel init DIR` (D0093) — scaffold a fresh project: the embedded engine (`.engine/`, with the
/// architecture decisions remapped to read-only `reference/`), `CLAUDE.md`, and a starter `.tracking/`.
/// Self-contained cold start; refuses to overwrite an existing `.engine/`.
/// The first positional argument, REFUSING anything that looks like a flag (issue179).
///
/// `keel init --help` created a directory named `--help` and scaffolded a complete engine into it; 277
/// files reached this repository and were committed before the CRLF warnings gave it away. `cmd_init`
/// read `args.first()` directly and so never passed through `root_arg`, which has rejected unknown
/// flags all along - the bypass was the bug, not the parsing.
///
/// A leading `-` is refused wherever a PATH or a NAME is expected. For `init` the stakes are highest,
/// because its whole job is writing a tree to disk, so any string it accepts is a filesystem mutation.
fn positional_arg<'a>(args: &'a [String], usage: &str, what: &str) -> Result<&'a String, i32> {
    let Some(first) = args.first() else {
        eprintln!("usage: {usage}");
        return Err(2);
    };
    if first.starts_with('-') {
        eprintln!("error: `{first}` looks like a flag, not {what}.");
        eprintln!("  Refused rather than used: `keel init --help` once created a directory named");
        eprintln!("  `--help` and scaffolded an engine into it (issue179).");
        eprintln!("usage: {usage}");
        return Err(2);
    }
    Ok(first)
}

/// Write the scaffolded pre-commit gate at `repo_root` and arm `core.hooksPath` there (issue278).
///
/// `repo_root` is the git repository root, which is the only place a hook can be invoked from, and
/// `project` is the project just scaffolded — used only to say which one armed the gate.
///
/// An EXISTING hook is never overwritten. In a workspace the repo-root hook is shared, so a second
/// `keel init` would be silently replacing a file the first project (or a human) owns — D0108: a
/// non-owner may add, never overwrite in place. The scaffolded body is workspace-scoped and needs no
/// per-project edit, so an existing keel hook already covers the new project; anything else is the
/// author's own gate and is reported rather than clobbered.
fn install_commit_gate(repo_root: &Path, project: &Path) -> Result<(), i32> {
    let hooks = repo_root.join(".githooks");
    if let Err(e) = std::fs::create_dir_all(&hooks) {
        eprintln!("error creating {}: {e}", hooks.display());
        return Err(1);
    }
    let hook_path = hooks.join("pre-commit");
    let existed = hook_path.exists();
    if existed {
        let body = std::fs::read_to_string(&hook_path).unwrap_or_default();
        if body.contains("gate --workspace") {
            println!("commit gate already installed at {} — it is workspace-scoped and covers this project too.", hook_path.display());
        } else {
            println!("NOTE: {} exists and is not the scaffolded keel gate — left untouched.", hook_path.display());
            println!("  Add `keel gate --workspace .` to it, or this project is not gated at commit.");
        }
    } else {
        if let Err(e) = std::fs::write(&hook_path, precommit_hook()) {
            eprintln!("error writing {}: {e}", hook_path.display());
            return Err(1);
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let _ = std::fs::set_permissions(&hook_path, std::fs::Permissions::from_mode(0o755));
        }
        println!("commit gate written to {} (workspace-scoped).", hook_path.display());
    }
    // Arm it. Without this the hook is a file nothing runs — the K2 failure the drift warning exists
    // for. Armed HERE rather than left to the printed next-steps, because the step a newcomer is most
    // likely to skip is the one that turns the gate on.
    if repo_root.join(".git").exists() {
        let _ = keel_cli::gitx::git()
            .arg("-C")
            .arg(repo_root)
            .args(["config", "core.hooksPath", ".githooks"])
            .status();
        println!("core.hooksPath set to .githooks in {} (the gate is live).", repo_root.display());
    } else {
        println!("NOTE: {} is not a git repository yet — the gate is written but NOT ARMED.", repo_root.display());
        println!("  Run `git init` HERE (not inside the project) and then:");
        println!("    git -C {} config core.hooksPath .githooks", repo_root.display());
    }
    if project != repo_root {
        println!("  (the gate lives at the repository root because git allows one hooks path per repo)");
    }
    Ok(())
}

/// The D0174/P0 slice of `init`: the declared adoption-profile fact, the `.claude/` enforcement
/// surface (five hook events, output style, per-registry skills), the optional CI template, and
/// wiring `core.hooksPath` when a `.git` exists.
fn init_enforcement_surface(dir: &Path, engine_dst: &Path, profile: &str) -> Result<(), i32> {
    let contracts = engine_dst.join("contracts");
    let _ = std::fs::create_dir_all(&contracts);
    let profile_fact = format!(
        "# Adoption profile — DECLARED at init, never inferred (D0174/P0.4).
         # strict: blocking in-loop gates from day one. guided: advisory-first; promote to blocking
         # with `keel sync-claude` after the D0180 evidence window, citing the fire-ledger.
         profile = \"{profile}\"
declaredAt = \"{}\"
",
        keel_cli::scaffold::today()
    );
    if let Err(e) = std::fs::write(contracts.join("adoption-profile.toml"), profile_fact) {
        eprintln!("error writing adoption-profile.toml: {e}");
        return Err(1);
    }
    match keel_cli::claude_surface::sync_claude(dir, false) {
        Ok(r) => println!(
            ".claude/ scaffolded: settings.json (5 hook events), output style, {} skill(s) (= registry count {}).",
            r.skills_written, r.registry_count
        ),
        Err(e) => {
            eprintln!("error scaffolding .claude/: {e}");
            return Err(1);
        }
    }
    let wf = dir.join(".github").join("workflows");
    let _ = std::fs::create_dir_all(&wf);
    if let Err(e) = std::fs::write(wf.join("keel-gate.yml"), keel_cli::claude_surface::CI_TEMPLATE) {
        eprintln!("error writing CI template: {e}");
        return Err(1);
    }
    // `core.hooksPath` is armed by `install_commit_gate` against the REPOSITORY root (issue278).
    // It used to be armed here against the project directory, which is the wrong repository whenever
    // the project is a workspace peer.
    Ok(())
}

/// The scaffold's closing narration: what was written, and the two onboarding steps in order.
///
/// Lifted out of `cmd_init` when adding the D0225 step pushed it past the line limit - it is
/// narration, not logic, and it is the FIRST thing a newcomer reads.
/// Warn when the scaffold's own deepest path leaves too little room under the host's path limit.
///
/// issue313, found by federation reconnaissance: at a 150-character parent directory, the shipped
/// `.engine/reference/decisions/0137-a-release-is-verified-by-running-the-published-asset.sysml`
/// reaches 249 characters and `git add -A` FAILS — so the project cannot make its FIRST COMMIT, and
/// the failure surfaces much later as an opaque `unable to index file`. keel does not own the git
/// repository and cannot set `core.longpaths` for it, but it can refuse to let this be discovered
/// the hard way. The warning names the remedy; it does not block, because a project that never uses
/// git is unaffected and a lockout here would be worse than the defect.
fn warn_if_paths_are_near_the_limit(dir: &Path) {
    const LIMIT: usize = 260;
    const HEADROOM: usize = 20;
    if !cfg!(windows) {
        return;
    }
    let longest = keel_cli::walk_longest(dir);
    if longest + HEADROOM < LIMIT {
        return;
    }
    println!();
    println!("  WARNING — this project's deepest file path is {longest} characters, and Windows");
    println!("  refuses paths at {LIMIT}. `git add` will FAIL here with `unable to index file`, so the");
    println!("  project cannot make its first commit (issue313). Either move it to a shorter parent");
    println!("  directory, or enable long paths before committing:");
    println!("      git config --global core.longpaths true");
}

fn print_init_next_steps(dir: &Path, count: u32, profile: &str) {
    println!("Scaffolded the engine into {} ({count} engine file(s)). Adoption profile: {profile} (declared).", dir.display());
    warn_if_paths_are_near_the_limit(dir);
    println!();
    println!("Next:");
    println!("  1. cd {}", dir.display());
    // issue278: this step used to read `git init && git config core.hooksPath .githooks`. Followed
    // literally from inside a workspace peer — which step 1 has just put the reader in — `git init`
    // creates a NESTED repository and destroys the workspace: the peer stops being part of the repo
    // whose hook and push cover it. `install_commit_gate` now arms the gate against the repository
    // root at init time, so the step is a check rather than an instruction to run blind.
    if std::path::Path::new(".git").exists() || dir.join(".git").exists() {
        println!("  2. The commit gate is already armed (see above). Confirm: git config core.hooksPath");
    } else {
        println!("  2. `git init` AT THE REPOSITORY ROOT — not inside this project if it is one of");
        println!("     several in a shared repo, where a nested repo would take it out of the workspace.");
        println!("     Then re-run `keel init` here, or arm it by hand: git config core.hooksPath .githooks");
    }
    println!("  3. Read CLAUDE.md — how to work here (text is truth; the AI drives the CLI, you supervise).");
    // D0225: the two are sequenced, not alternatives. `project-onboarding` decides WHICH disciplines
    // this project runs; `introduction` walks the author through running one. Naming only the second
    // is how a project ends up with 25 active processes nobody chose (D0054 - friction is the top risk).
    println!("  4. Run the `project-onboarding` skill — it asks what you are building and charters the");
    println!("     process set on that basis. `keel onboard` reports NOT CHARTERED until you do.");
    println!("  5. Then the `introduction` skill — capture your first need + run your first sprint.");
    println!("     Or: keel show orient .   (where things stand)");
    println!();
    println!("The pre-commit gate at the REPOSITORY ROOT runs `keel gate --workspace` — validate + guard +");
    println!("declared rules, for every project the commit touches (Rust-only, no kernel).");
    println!("Engine design rationale is read-only reference in .engine/reference/decisions/;");
    println!("your project authors its OWN decisions fresh in .engine/decisions/.");
}

/// Refuse an `init` target INSIDE an existing project, reporting why (issue275). `true` = refused.
///
/// `keel init sub` under a project used to succeed, and the resulting nested project was invisible to
/// workspace discovery, so it rode out UNGATED. Discovery now finds it, but the layout still leaves
/// overlapping paths with two claimants and is not what any author means. Peers, not tenants.
fn refuse_nested_target(dir: &Path) -> bool {
    let Some(host) = enclosing_project(dir) else { return false };
    eprintln!(
        "error: {} is inside the keel project at {} — refusing to nest one project inside another.",
        dir.display(),
        host.display()
    );
    eprintln!("  A workspace holds projects as PEERS: create the directory beside the existing");
    eprintln!("  project, not under it. `keel projects` lists what this repository already holds.");
    // The host may BE the repository root, in which case there is no "beside" inside this repo at
    // all. Say so rather than leaving the author to discover it by trying: a workspace is peers in
    // subdirectories with no project at the root, which is the layout the request behind D0234
    // described — "a parent folder repo, then separate keel projects in subfolders".
    if host.join(".git").exists() {
        eprintln!();
        eprintln!("  {} is the REPOSITORY ROOT, so this repo has no room for a peer:", host.display());
        eprintln!("  a workspace is projects in subdirectories with none at the root. Either");
        eprintln!("    - give this project its own repository (usually the right answer), or");
        eprintln!("    - move the existing project into a subdirectory first, then init peers beside it.");
    }
    true
}

/// The nearest ANCESTOR of `dir` that is already a keel project, if any (issue275).
///
/// Strictly an ancestor: `dir` itself is handled by the `.engine/` refusal above, which reports the
/// overwrite case with its own message. Walks up rather than consulting `workspace::discover`,
/// because the target may not exist yet and may sit outside any git repository — neither of which
/// stops it from being inside a project.
fn enclosing_project(dir: &Path) -> Option<PathBuf> {
    // ABSOLUTISE FIRST. `init`'s target normally does not exist yet — that is the point of init — so
    // `canonicalize` fails and returns the argument unchanged. For a relative target like `sub`, its
    // parent is then the EMPTY path, and `is_project("")` silently tests the PROCESS's current
    // directory rather than the target's parent: the refusal printed the host project as an empty
    // string, and from another cwd it would have answered about a different directory entirely.
    //
    // `workspace::canon` rather than a bare canonicalize because the raw form carries the `\?\`
    // extended-length prefix on Windows, and this path is printed to the author.
    let abs = if dir.is_absolute() {
        dir.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(dir)
    };
    let start = keel_cli::workspace::canon(&abs);
    let mut cur = start.parent();
    while let Some(p) = cur {
        if keel_cli::workspace::is_project(p) {
            return Some(p.to_path_buf());
        }
        cur = p.parent();
    }
    None
}

/// Is a build commit one another machine can obtain - a clean SHA, not `+dirty` and not `unknown`?
/// (issue379 / GH#54: the wrapper cache may be seeded only from such a build.)
fn is_reproducible_build(commit: &str) -> bool {
    !commit.is_empty() && commit != "unknown" && !commit.ends_with("+dirty")
}

/// D0251 clause B: ship the wrapper into a fresh project. `keelw` resolves the project's pin;
/// `keel-wrapper.toml` is the committed checksum contract (entries come from the release page — the
/// starter names the duty rather than inventing hashes). The cache is SEEDED with the running binary
/// ONLY when that binary is a reproducible build (issue379 / GH#54): a `+dirty` build seeded under the
/// pin's name became "the pinned release" for a project's whole life, and keelw never noticed because
/// it verified downloads, not hits. Either way init says what it did; it never writes a checksum it
/// made up.
///
/// # Errors
/// The CLI exit code when either committed file cannot be written. A failed cache SEED is a note,
/// not an error — the wrapper still works via a checksum entry or a manual install.
fn init_wrapper(dir: &Path) -> Result<(), i32> {
    if let Err(e) = std::fs::write(dir.join("keelw"), include_str!("../../keelw")) {
        eprintln!("error writing keelw: {e}");
        return Err(1);
    }
    let wrapper_toml = format!(
        "# keel-wrapper — per-version, per-platform release-asset SHA-256s the keelw wrapper verifies\n# against. NEVER trust-on-first-use: a version with no entry here refuses to\n# download. Entries come from the release page's published checksums. A cache HIT is verified against the\n# entry for its version too (issue379): a binary seeded here by `keel init` runs UNVERIFIED until the\n# entry exists, and is refused if it then does not match.\n[\"{}\"]\n",
        env!("CARGO_PKG_VERSION")
    );
    if let Err(e) = std::fs::write(dir.join("keel-wrapper.toml"), wrapper_toml) {
        eprintln!("error writing keel-wrapper.toml: {e}");
        return Err(1);
    }
    let commit = env!("KEEL_BUILD_COMMIT");
    let version = env!("CARGO_PKG_VERSION");
    if !is_reproducible_build(commit) {
        println!("wrapper cache NOT seeded: this binary is build `{commit}`, which no other machine can obtain, so it must not stand in for keel {version} (issue379) - keelw fetches the release asset for v{version} on first use, verified against keel-wrapper.toml");
        return Ok(());
    }
    if let Ok(me) = std::env::current_exe() {
        let asset = if cfg!(windows) {
            "keel-windows-x86_64.exe"
        } else if cfg!(target_os = "macos") {
            "keel-macos-aarch64"
        } else {
            "keel-linux-x86_64"
        };
        let cache = dir.join(".keel").join("bin").join(version);
        let _ = std::fs::create_dir_all(&cache);
        match std::fs::copy(&me, cache.join(asset)) {
            Ok(_) => println!("wrapper cache seeded from this binary (build {commit}) at .keel/bin/{version}/{asset} - UNVERIFIED until keel-wrapper.toml carries the release checksum for {version}, which keelw then checks on every hit"),
            Err(e) => eprintln!("note: could not seed the wrapper cache ({e}) — keelw will need a checksum entry or a manual install"),
        }
    }
    Ok(())
}

fn cmd_init(args: &[String]) -> i32 {
    const USAGE: &str = "keel init DIR [--profile strict|guided]";
    let target = match positional_arg(args, USAGE, "a directory") {
        Ok(a) => a,
        Err(code) => return code,
    };
    let dir = PathBuf::from(target);
    let engine_dst = dir.join(".engine");
    if engine_dst.exists() {
        eprintln!("error: {} already contains a .engine/ — refusing to overwrite", dir.display());
        return 2;
    }
    if refuse_nested_target(&dir) {
        return 2;
    }
    // Adoption profile: DECLARED, never inferred (D0174/P0.4 — the issue089/129 lockout class died
    // of inference). Default applies only to a PROVABLY EMPTY directory (strict: a fresh scaffold
    // starts green, init_smoke proves it); any directory with existing content requires the flag.
    let profile = match flag(args, "profile") {
        Some(p) if p == "strict" || p == "guided" => p,
        Some(p) => {
            eprintln!("error: --profile takes `strict` or `guided`, not `{p}`.");
            eprintln!("usage: {USAGE}");
            return 2;
        }
        None => {
            let has_content =
                std::fs::read_dir(&dir).is_ok_and(|rd| rd.flatten().any(|e| e.file_name() != ".git"));
            if has_content {
                eprintln!("error: {} has existing content — an adoption profile must be DECLARED, never inferred (D0174).", dir.display());
                eprintln!("  --profile strict   blocking in-loop gates from day one (fresh projects)");
                eprintln!("  --profile guided   advisory-first; promote to blocking later, citing measured evidence (D0180)");
                eprintln!("usage: {USAGE}");
                return 2;
            }
            "strict".to_string()
        }
    };
    let mut count = 0u32;
    if let Err(e) = scaffold_engine(&ENGINE_DIR, &engine_dst, &mut count) {
        eprintln!("error scaffolding engine: {e}");
        return 1;
    }
    // Empty .engine/decisions/ — where the NEW project authors its own decisions (the engine's ship
    // as read-only reference under .engine/reference/decisions/).
    if let Err(e) = std::fs::create_dir_all(engine_dst.join("decisions")) {
        eprintln!("error creating .engine/decisions: {e}");
        return 1;
    }
    if let Err(e) = std::fs::write(dir.join("CLAUDE.md"), CLAUDE_MD) {
        eprintln!("error writing CLAUDE.md: {e}");
        return 1;
    }
    // A .gitignore, because the MACHINE-LOCAL files must never be committed and nothing was
    // stopping them. Found by a two-clone test (sprint 300): both contributors' `.keel/actor`
    // bindings landed in git and then CONFLICTED on merge — each clone claiming to be the other's
    // identity. `actor.rs` has always said the binding is per-machine and "committing it would
    // re-create the shared-default defect it exists to remove"; nothing enforced it downstream,
    // because `keel init` scaffolded no ignore file at all.
    if let Err(e) = std::fs::write(dir.join(".gitignore"), GITIGNORE) {
        eprintln!("error writing .gitignore: {e}");
        return 1;
    }
    let tracking = dir.join(".tracking");
    if let Err(e) = std::fs::create_dir_all(&tracking) {
        eprintln!("error creating .tracking: {e}");
        return 1;
    }
    if let Err(e) = std::fs::write(tracking.join("README.md"), TRACKING_STARTER) {
        eprintln!("error writing .tracking/README.md: {e}");
        return 1;
    }
    // A starter actor registry so the newcomer's first recorded fact (createdBy/judgedBy) passes the
    // actors guard (D0037) — they edit it to their real identities.
    if let Err(e) = std::fs::write(tracking.join("actors.sysml"), STARTER_ACTORS) {
        eprintln!("error writing .tracking/actors.sysml: {e}");
        return 1;
    }
    // Scaffold a RUST-ONLY commit gate so the project has an automated gate from day one — no
    // conda/kernel (D0048).
    //
    // The hook belongs to the REPOSITORY, not the project (issue278). git allows one
    // `core.hooksPath` per repository, so a hook written inside a project directory can never be
    // invoked for a sibling — and `git init` followed by two `keel init`s left no repo-root hook and
    // no hooksPath at all, so the whole workspace was UNGATED and the subsequent commit ran with zero
    // pre-commit lines. Verified before the fix.
    let repo_root = keel_cli::gitx::git()
        .arg("-C")
        .arg(&dir)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()))
        .filter(|p| p.is_dir())
        .map_or_else(|| dir.clone(), |p| keel_cli::workspace::canon(&p));
    if let Err(code) = install_commit_gate(&repo_root, &dir) {
        return code;
    }
    if let Err(code) = init_enforcement_surface(&dir, &engine_dst, &profile) {
        return code;
    }
    // D0190: stamp the declared engine version - which binary's checks this engine is defined
    // against. Re-stamped by `keel migrate`; read only by the parity warning, never by migrate.
    let version_toml = format!(
        "# engine-version - the BINDING engine pin: the version whose writes and gates this project accepts\n#. Written by `keel init`, re-stamped by `keel migrate`; a\n# mismatched binary refuses writes and gates, warns on reads. keelw resolves this pin.\nengine = \"{}\"\n",
        env!("CARGO_PKG_VERSION")
    );
    if let Err(code) = init_wrapper(&dir) {
        return code;
    }
    if let Err(e) = std::fs::write(engine_dst.join("contracts").join("engine-version.toml"), version_toml) {
        eprintln!("error writing engine-version.toml: {e}");
        return 1;
    }
    print_init_next_steps(&dir, count, &profile);
    0
}

/// Header kept at the top of a generated `activation.toml` — the file must explain itself, because the
/// consequence of editing it wrongly (a control silently off) is not obvious from its contents.
const ACTIVATION_HEADER: &str = "\
# Process activation (D0138) — which processes THIS project has adopted.
#
# What activating a process does: turns on its whole unit (skill + declared rules + guards), as defined
# by the engine from each process's own `assert constraint` declarations. Deactivating one stops its guards running,
# and `keel gate guard` then REPORTS each as NOT ACTIVE rather than skipping it silently.
#
# DELETE THIS FILE to return to \"everything is active\", which is also the behaviour when no file
# exists — so an existing project that never declares one is unaffected.
#
# CORE guards (identity, provenance, vocabulary, rootedness, well-formedness) are in no unit and CANNOT
# be deactivated here. Activation exists to stop enforcing procedures you have not adopted; it is not a
# switch that makes truthfulness optional.
#
# Edit by hand, or use `keel activate <process>` / `keel deactivate <process>`.
";

/// Write `activation.toml` with both active sets stated exactly.
///
/// BOTH sections are always written, even when one is unchanged (D0164). Writing only the section being
/// edited would leave the other absent, and absent means EVERYTHING ACTIVE — so deactivating one
/// viewpoint would silently re-activate every process the project had turned off. A partial write of a
/// contract whose absence has meaning is a data-loss bug, not a convenience.
/// Rewrite ONLY the `active = [...]` line inside `[section]`, leaving every other byte of the
/// manifest untouched — comments, `charteredBy`, blank lines, ordering, line endings.
///
/// Regenerating the file from a template instead was issue293: one `deactivate`+`activate` round
/// trip deleted `charteredBy = "d0226"` with the comment explaining it. Returns `None` when the
/// section or its `active` line is absent, so the caller can fall back to generating a fresh file
/// rather than silently writing a manifest with a section missing (absence has meaning here).
fn replace_active_line(src: &str, section: &str, items: &[String]) -> Option<String> {
    let want = format!("[{section}]");
    let rendered = items.iter().map(|p| format!("\"{p}\"")).collect::<Vec<_>>().join(", ");
    let (mut in_section, mut replaced) = (false, false);
    let mut out = String::with_capacity(src.len() + rendered.len());
    for line in src.split_inclusive('\n') {
        let trimmed = line.trim_end();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_section = trimmed == want;
        } else if in_section && !replaced && trimmed.starts_with("active") && trimmed.contains('=') {
            let eol = &line[trimmed.len()..]; // preserve LF vs CRLF vs no trailing newline
            out.push_str("active = [");
            out.push_str(&rendered);
            out.push(']');
            out.push_str(eol);
            replaced = true;
            continue;
        }
        out.push_str(line);
    }
    replaced.then_some(out)
}

/// The `active` list the manifest currently RECORDS for a section, read from the file rather than
/// recomputed from the engine's view of it.
///
/// The second half of issue293: the process list was rebuilt from `unit_names()`, which only knows
/// SWITCHABLE processes, so every `[always]` one (`decision-authoring`) vanished on any write. An
/// absent section means everything is active, but a PRESENT list naming 10 of 11 marks the 11th
/// INACTIVE — so omitting a process from a list that exists deactivates it by silence.
fn recorded_active(root: &Path, section: &str) -> Option<Vec<String>> {
    let src = std::fs::read_to_string(root.join(".engine/contracts/activation.toml")).ok()?;
    let want = format!("[{section}]");
    let mut in_section = false;
    for line in src.lines() {
        let trimmed = line.trim_end();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_section = trimmed == want;
        } else if in_section && trimmed.starts_with("active") {
            let list = trimmed.split_once('[')?.1.rsplit_once(']')?.0;
            return Some(
                list.split(',')
                    .map(|s| s.trim().trim_matches('"').to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
            );
        }
    }
    None
}

fn write_activation(root: &Path, processes: &[String], viewpoints: &[String]) -> std::io::Result<()> {
    // JOIN `.engine/contracts` HERE, as the original did. Taking a pre-joined directory instead was my
    // regression and it wrote the manifest to the repo root, where `Activation::load` never looks - so
    // `keel deactivate` reported success and changed nothing. Caught by round-tripping the command
    // against the surfaces it is supposed to affect rather than by reading its output, which said
    // "deactivated" either way.
    let dir = root.join(".engine/contracts");
    std::fs::create_dir_all(&dir)?;
    // PRESERVE FIRST (issue293): an existing manifest is EDITED, never regenerated. Only a missing
    // or unparseable one falls through to the template below.
    let path = dir.join("activation.toml");
    if let Ok(existing) = std::fs::read_to_string(&path) {
        if let Some(edited) = replace_active_line(&existing, "processes", processes)
            .and_then(|s| replace_active_line(&s, "viewpoints", viewpoints))
        {
            return std::fs::write(&path, edited);
        }
    }
    std::fs::write(
        dir.join("activation.toml"),
        format!(
            "{ACTIVATION_HEADER}
[processes]
active = [{}]

[viewpoints]
active = [{}]
",
            processes.iter().map(|p| format!("\"{p}\"")).collect::<Vec<_>>().join(", "),
            viewpoints.iter().map(|v| format!("\"{v}\"")).collect::<Vec<_>>().join(", "),
        ),
    )
}

/// Activate or deactivate one VIEWPOINT, writing both manifest sections (D0164).
///
/// Split out of `cmd_activation` to keep that function within the line budget, and because the two
/// namespaces genuinely differ: a process switch turns guards on and off, a viewpoint switch turns a LENS
/// on and off. Bundling them into one branchy function hid that.
fn switch_viewpoint(
    root: &Path,
    act: &keel_cli::activation::Activation,
    mode: &str,
    target: &str,
    all: &[String],
    mut set: Vec<String>,
) -> i32 {
    let materialising = act.active_viewpoints.is_none();
    if mode == "activate" {
        if !set.iter().any(|v| v == target) {
            set.push(target.to_string());
        }
    } else {
        set.retain(|v| v != target);
    }
    set.sort();
    // The PROCESS section must be rewritten too, unchanged: absence means everything active, so omitting
    // it would silently re-activate every process the project had turned off.
    let procs: Vec<String> = recorded_active(root, "processes")
        .unwrap_or_else(|| act.unit_names().into_iter().filter(|p| act.is_process_active(p)).collect());
    if let Err(e) = write_activation(root, &procs, &set) {
        eprintln!("error writing .engine/contracts/activation.toml: {e}");
        return 1;
    }
    if materialising {
        println!("No viewpoint manifest existed (all were active), so one was written with the current");
        println!("effective state before applying this change.");
    }
    println!("{mode}d viewpoint `{target}`. Active viewpoints: {} of {}", set.len(), all.len());
    println!("Read it back: keel activation | keel gate guard");
    0
}

/// `keel activate <process>` / `keel deactivate <process>` / `keel activation` (D0138).
///
/// The subtle case is activating when NO manifest exists. Absence means "everything is active", so
/// writing a manifest containing only the named process would silently DEACTIVATE every other control —
/// the opposite of what the caller asked for. So a first write MATERIALISES the current effective state
/// (all declared units) and then applies the change, and says that it did.
fn cmd_activation(mode: &str, args: &[String]) -> i32 {
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
    let act = keel_cli::activation::Activation::load(&root);

    if mode == "activation" {
        println!("declared manifest: {}", if act.is_declared() { "yes" } else { "no — everything present is active" });
        // EVERY declared process, not only the switchable ones (issue149): listing 6 of 18 with no note
        // that the rest exist reads as "this project has 6 processes".
        for p in keel_cli::activation::declared_processes(&root) {
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
        let vps = keel_cli::view::declared_viewpoints(&root).unwrap_or_default();
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
        for g in keel_cli::guards::GUARD_NAMES {
            if act.guard_state(g) == keel_cli::activation::GuardState::Core {
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
        keel_cli::view::declared_viewpoints(&root).unwrap_or_default().into_iter().map(|v| v.name).collect();
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
        if keel_cli::activation::declared_processes(&root).iter().any(|p| p == target) {
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
                keel_cli::activation::declared_processes(&root).join(", ")
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
        match keel_cli::claude_surface::sync_claude(&root, false) {
            Ok(r) => println!("claude surface regenerated: {}/{} skill(s) - a deactivated process's skill now opens with its INACTIVE state (D0348)", r.skills_written, r.registry_count),
            Err(e) => eprintln!("claude surface NOT regenerated ({e}) - run `keel sync-claude` so the deployed skills follow the active set"),
        }
    }
    println!("Read it back: keel activation | keel gate guard");
    0
}

/// `keel version` (also `--version` / `-V`) — report which build this is.
///
/// Exists because a downstream project could not answer "am I running the fix?": three versioned
/// releases shipped with no version identification in the artifact, so a project still on the blocked
/// version was INDISTINGUISHABLE from one that had upgraded. Reports the release version, the build
/// commit (baked by `build.rs`; `unknown` off-git, `+dirty` from a modified tree — never guessed), and
/// the CONTROL INVENTORY this binary carries, computed from `GUARD_NAMES` rather than restated, so it
/// cannot drift from the guards that actually run.
/// `keel migrate [ROOT] [--dry-run]` — bring a DOWNSTREAM project up to this binary's engine vintage.
///
/// Deliberately NOT defaulted to the discovered repo root: `find_repo_root` walks upward looking for
/// `.engine/`, which from inside a downstream project's subdirectory is right, but from anywhere in
/// the self-build repo would point this at the engine SOURCE. `migrate` refuses the self-build repo
/// anyway, but a command that rewrites authored facts should take its target explicitly.
/// `keel currency [ROOT] ...` (D0338): a trailing ROOT is honoured only when it is a keel project.
/// `keel show status [ROOT]`: a mistyped flag is never a root (issue133).
fn cmd_status(rest: &[String]) -> i32 {
    refuse_flag_as_path(rest.first(), "status").unwrap_or_else(|| {
        resolve_guard_root(rest.first()).map_or_else(
            || {
                eprintln!("error: no .engine/ directory found. usage: keel show status [ROOT]");
                2
            },
            |root| keel_cli::status::cmd(&root),
        )
    })
}

/// `keel suite [ROOT] [-- <cargo test args>]` (D0353): everything after `--` is cargo's, so the ROOT
/// is looked for only before it.
fn cmd_suite(rest: &[String]) -> i32 {
    let own: Vec<String> = rest.iter().take_while(|a| *a != "--").cloned().collect();
    keel_cli::suite::cmd(rest, &repo_arg(&own))
}

fn cmd_verify(rest: &[String]) -> i32 {
    // The probe value is a command line, never a root: the root is found among what the ladder
    // does not consume.
    let own = keel_cli::verify::own_args(rest);
    keel_cli::verify::cmd(rest, &repo_arg(&own))
}

fn cmd_currency(rest: &[String]) -> i32 {
    let root = rest.iter().find(|a| !a.starts_with("--") && Path::new(a.as_str()).join(".tracking").is_dir()).map_or_else(|| find_repo_root().unwrap_or_else(|| PathBuf::from(".")), PathBuf::from);
    keel_cli::currency::cmd(rest, &root, &keel_cli::github_ingest::pull_cmd)
}

fn cmd_migrate(args: &[String]) -> i32 {
    let dry_run = args.iter().any(|a| a == "--dry-run");
    // D0336: verified-or-reverted is the default; `--no-verify` writes an UNVERIFIED tree and says so.
    let verify = !args.iter().any(|a| a == "--no-verify");
    let root = match root_arg(args, "keel migrate [--dry-run] [--no-verify] [ROOT]", &["dry-run", "no-verify"], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    keel_cli::migrate::cmd_with(&root, &ENGINE_DIR, dry_run, verify)
}

/// `keel record issue` — the sanctioned path for D0108 clause 5, which mandates that conflicting
/// conclusions be recorded as an Issue for human adjudication and had no implementation.
///
/// REFUSES on an unknown `--resolver` rather than authoring a dangling edge: the `issues` guard is
/// satisfied by the PRESENCE of a `#Resolves` edge, so an edge pointing at nothing would read as
/// triaged while resolving nothing — exactly the phantom issue109 found twice in this repo.
fn cmd_record_issue(args: &[String]) -> i32 {
    let root = flag(args, "root").map_or_else(|| find_repo_root().unwrap_or_else(|| PathBuf::from(".")), PathBuf::from);
    let req = |n: &str| flag(args, n);
    // PROSE COMES FROM A FILE (D0224, extended here to `issue`). The decision path got this after
    // shell backticks inside a double-quoted `--description` EXECUTED and injected tool output into
    // a governance record. `issue` was left on the argument path and the same trap fired a third
    // time on 2026-08-30 — corrupting issue314's own description and running `keel deactivate`
    // against this repository. A trap that recurs after its fix is a fix that was applied to the
    // instance instead of the class (issue315).
    let description = match prose_flag(args, "description", "record issue") {
        Ok(d) => d,
        Err(msg) => { eprintln!("error: {msg}"); return 2; }
    };
    let (Some(title), Some(description), Some(severity), Some(resolver)) =
        (req("title"), description, req("severity"), req("resolver"))
    else {
        eprintln!("error: --title --description --severity --resolver are all required.");
        eprintln!("  PREFER --description-from FILE for anything long: prose passed as a shell argument");
        eprintln!("  has had its backticks EXECUTED into the record three times now (D0224/issue315).");
        eprintln!("  --resolver names the EXISTING item that resolves this issue. It is required because the");
        eprintln!("  `issues` guard fails on an untriaged Issue, so recording one without triage would hand you");
        eprintln!("  a red gate as the command's output (D0077).");
        return 2;
    };
    // The item process's own checks - the severity and the resolver-kind predicate the commit gate
    // applies (issue558) - are keel-issues' since sprint 737 (D0480); the refusal text is the member's.
    if let Err(e) = keel_cli::issues::issue_write::triage_holds(&root, &severity, &resolver) {
        eprintln!("error: {e}");
        return 2;
    }
    let Some(date) = flag(args, "date").filter(|d| !d.is_empty()) else {
        eprintln!("error: --date YYYY-MM-DD required (when it was found is its own irreducible fact)");
        return 2;
    };
    // NEVER default the actor (D0129/issue072): an unattributable fact looks like evidence.
    let author = match keel_cli::actor::resolve(&root, flag(args, "by").as_deref()) {
        Ok(a) => a,
        Err(msg) => { eprintln!("{msg}"); return 2; }
    };
    // Bound to locals so the borrows outlive the struct; the inline form silently collapsed both to
    // None (they type-checked and were wrong — a flag the caller passed would have been dropped).
    let related_task = flag(args, "related-task");
    let marker = flag(args, "marker");
    let n = keel_cli::write::NewIssue {
        title: &title,
        description: &description,
        severity: &severity,
        resolver: &resolver,
        related_task: related_task.as_deref(),
        date: &date,
        author: &author,
        marker: marker.as_deref(),
        in_field: args.iter().any(|a| a == "--in-field"),
    };
    match keel_cli::write::record_issue(&root, &n) {
        Ok((name, path)) => {
            println!("recorded {name} -> {path}");
            println!("  triaged on arrival: `#Resolves dependency from {resolver} to {name};`");
            println!("  run `keel gate validate . && keel gate guard .` to confirm; nothing was committed.");
            0
        }
        Err(e @ keel_cli::write::WriteError::InjectedToolOutput(..)) => refused_injected_prose(&root, &e),
        Err(e) => { eprintln!("error: {e}"); 1 }
    }
}

/// `keel accept <decision> --note "<the human's words>" --by <humanActor> --date YYYY-MM-DD`
///
/// THE SINGLE HUMAN GATE HAD NO CLI. `write::accept_decision` existed and was reachable only through
/// `keel serve`'s HTTP API, so recording the one attestation this engine treats as irreducibly human
/// required either running a web server or hand-editing the decision file. That is the same defect
/// `record issue` had (sprint 291): a mandated path with no implementation is the friction that
/// guarantees non-compliance (D0054), and here it applies to the gate the whole autonomous loop
/// pauses for.
///
/// # This command records a human's word; it cannot create one
///
/// `--note` must carry what the human actually said, and `--by` must name a `Person`. The
/// `confirmation-authenticity` guard independently checks that the acceptance result is judged by a
/// Person-typed actor, so an AI accepting its own proposal fails the gate rather than passing it —
/// this command makes the honest path easy without making the dishonest one possible.
/// D0315/issue359: is the human at THEIR terminal? A real TTY, or the test's declared stand-in
/// (`KEEL_TTY_GESTURE=1`) - which the record names as asserted rather than observed, so a reader
/// can tell the two apart.
fn tty_gesture() -> Option<&'static str> {
    use std::io::IsTerminal as _;
    if std::io::stdin().is_terminal() {
        Some("TTY gesture: typed at an interactive terminal")
    } else if std::env::var("KEEL_TTY_GESTURE").is_ok_and(|v| v == "1") {
        Some("TTY gesture (asserted by KEEL_TTY_GESTURE, not observed): typed at a terminal")
    } else {
        None
    }
}

/// Channel layer (D0178/P1.3, best-effort by recorded design): in a session bearing
/// agent-environment markers, `keel accept` requires a TTY-interactive human or the console
/// approve queue - the actor binding alone is agent-mutable state. The write layer (AI-kind
/// refusal) and the tree-derived audit are the real controls; this is the friction layer.
/// `Some(exit)` refuses; `None` lets the accept proceed.
fn accept_channel_refusal(args: &[String], tty_gesture: Option<&str>) -> Result<Vec<String>, i32> {
    verdict_channel_refusal("accept", "accepting", args, tty_gesture, "decisionAcceptance", true)
}

/// D0423: the checks that used to refuse a delegated record are written INTO it. Each WARN line names
/// the check, so a reader of the acceptance sees what kind of receipt it is; the write proceeds.
fn fold_warnings_into_note(args: &[String], warnings: &[String]) -> Vec<String> {
    if warnings.is_empty() {
        return args.to_vec();
    }
    let note = flag(args, "note").unwrap_or_default();
    let mut out: Vec<String> = Vec::with_capacity(args.len());
    let mut skip_value = false;
    for a in args {
        if skip_value {
            skip_value = false;
        } else if a == "--note" {
            skip_value = true;
        } else {
            out.push(a.clone());
        }
    }
    out.push("--note".to_string());
    out.push(format!("{note} {}", warnings.join(" ")));
    out
}

/// The channel rules shared by `keel accept` and `keel reject` (D0393/issue414): a human's verdict on a
/// proposed Decision, recorded from an agent session only under the declared delegation and only with
/// their words quoted. `verb` names the command in every message; `doing` is its participle for the
/// read-back line.
///
/// `Err(exit)` refuses and writes a `refused` ledger line naming the check (issue445); `Ok(warnings)`
/// proceeds, and since D0423 the warnings are what two of the checks used to refuse on: words shorter
/// than ten characters (D0375) and words that do not read the decision back (D0289 / D0201 B). Both are
/// still computed; they land in the record as `WARN: <check>` instead of costing the human their
/// channel. What still refuses: no delegation declared, no quote at all (a paraphrase is the
/// fabrication D0198 names), and a gesture word typed as the only evidence - that one binds the
/// AGENT's act, not the human's, and D0427 keeps it out of D0423's dissolution.
/// `delegation_class` names the attestation-policy.toml section whose `delegatedRecording` lets an
/// agent session record the human's verdict (`decisionAcceptance` for accept/reject, `confirmationRecord`
/// for judge-set, D0443); `read_back` runs the D0201 B read-back against the first positional argument as a
/// Decision id - false when the subject is not a Decision (judge-set's subject is a file).
fn verdict_channel_refusal(verb: &str, doing: &str, args: &[String], tty_gesture: Option<&str>, delegation_class: &str, read_back: bool) -> Result<Vec<String>, i32> {
    let mut warnings: Vec<String> = Vec::new();
    {
        let agent_marked = ["CLAUDECODE", "CLAUDE_CODE_ENTRYPOINT", "CLAUDE_CODE_SESSION_ID", "CLAUDE_CODE_BRIDGE_SESSION_ID"]
            .iter()
            .any(|k| std::env::var(k).is_ok_and(|v| !v.is_empty()));
        if agent_marked && tty_gesture.is_none() {
            // D0289: the channel layer HONOURS the declared recording delegation (D0192 option A). When
            // attestation-policy.toml delegates the RECORDING of acceptance to the agent, the human's
            // quoted words ARE the channel - the same quote receipt the substance rule demands after
            // the fact is demanded here before the write. The human's words, 2026-09-03: "i want an
            // exception for user text that was quoted to be authoritative ... until we have a better
            // non-local authoritative channel". Withdraw by deleting the delegation line; this arm
            // then refuses exactly as before.
            let root = find_repo_root().unwrap_or_else(|| PathBuf::from("."));
            let delegation = keel_cli::activation::recording_delegation(&root, delegation_class);
            // D0411 / issue426: the receipt this session can record is the human's QUOTED WORDS. A
            // gesture citation - console, deck, TTY, GitHub - is written by the surface that observed
            // the gesture (the console appends a device receipt; the terminal path above cites its own
            // TTY), never typed into a note; typed, it is the free text the substance rule used to
            // accept, and the agent-marked session with no terminal is exactly the writer that cannot
            // have observed any of them.
            let receipt = flag(args, "note").map_or(keel_cli::view::NoteReceipt::Nothing, |n| keel_cli::view::note_receipt(&n));
            match (delegation, receipt) {
                (Some(d), keel_cli::view::NoteReceipt::QuotedWords) => {
                    // D0201 B, the chat half: READ-BACK RATIFICATION. The quoted words must name THIS
                    // decision (its id, one of its option letters, or three words of its title), or a
                    // bare 'yes' could be attached to any Decision the agent picks. Forward-only by
                    // construction: it binds new records, never re-reads old ones. The note has a
                    // quoted span here (the arm above), so the read-back reads the span, never a
                    // gesture word.
                    if let (true, Some(dec), Some(note)) = (read_back, args.first().filter(|a| !a.starts_with('-')), flag(args, "note")) {
                        let (letters, title) = decision_options_and_title(&root, dec);
                        // D0423: computed the same way, written into the record instead of refusing.
                        if !keel_cli::view::read_back_names(&note, dec, &letters, &title) {
                            eprintln!("keel {verb}: WARN - the quoted words do not name {dec}, the decision they are {doing} (read-back, D0201 B: its id, an option letter, or three words of its title); recorded as given with the WARN in the note (D0423).");
                            warnings.push(format!("WARN: read-back - the quoted words do not name {dec} (D0201 B); recorded as given (D0423)."));
                        }
                        let short = keel_cli::view::short_quoted_spans(&note);
                        if !short.is_empty() {
                            eprintln!("keel {verb}: WARN - the quoted words are shorter than ten characters (D0375); recorded as given with the WARN in the note (D0423).");
                            warnings.push("WARN: short words - fewer than ten characters quoted (D0375); recorded as given (D0423).".to_string());
                        }
                    }
                    eprintln!("keel {verb}: recording the human's verdict under delegation {d} - the note quotes their words (D0289).");
                }
                (Some(d), keel_cli::view::NoteReceipt::GestureWordOnly) => {
                    eprintln!("keel {verb}: the note names a gesture (console, deck, TTY, GitHub) but no gesture reached this command - a gesture citation is written by the surface that observed it (the console appends a device receipt it can re-verify; a terminal cites its own TTY), never typed into a note (D0411/issue426). This session has no terminal and is not the console; under delegation {d} the receipt it can record is the human's words, verbatim: --words \"<what they said>\". Nothing written.");
                    ledger_refused(&root, verb, "gesture-word-typed");
                    return Err(1);
                }
                (Some(d), keel_cli::view::NoteReceipt::Nothing) => {
                    eprintln!("keel {verb}: delegation {d} lets this session RECORD the human's verdict, but it must QUOTE their words verbatim - pass them as their own argument, --words \"<what they said>\", or quote them in the note inside a declared pair (D0192/D0289). A gesture is cited by the surface that observed it, not by this note (D0411).");
                    ledger_refused(&root, verb, "no-quote");
                    return Err(1);
                }
                (None, _) => {
                    eprintln!("keel {verb}: this session carries agent-environment markers and no interactive terminal (D0178/K6), and attestation-policy.toml declares no recording delegation for {delegation_class}.");
                    eprintln!("  The verdict is the human's own act: run `keel {verb}` from YOUR terminal, or give it from the console approve queue / the deck.");
                    ledger_refused(&root, verb, "no-delegation");
                    return Err(1);
                }
            }
        }
    }
    Ok(warnings)
}

/// A decision's OPTION letters (a fork) and its title plus decision text, for read-back ratification (D0201 B).
fn decision_options_and_title(root: &Path, dec: &str) -> (Vec<String>, String) {
    let mut letters = Vec::new();
    let mut title = String::new();
    for p in keel_cli::collect_sysml(&root.join(".engine").join("decisions")) {
        let Ok(text) = std::fs::read_to_string(&p) else { continue };
        if !text.contains(&format!("part {dec} : Decision")) {
            continue;
        }
        let rel = p.strip_prefix(root).unwrap_or(&p).to_string_lossy().replace('\\', "/");
        letters = keel_cli::deck::fork_options(root, &rel).into_iter().map(|(l, _)| l).collect();
        // the title AND the decision text: quoting the decision's own words is a read-back of it
        for key in [":>> title = \"", ":>> decision = \""] {
            if let Some(i) = text.find(key) {
                title.push_str(text.get(i + key.len()..).and_then(|r| r.split('"').next()).unwrap_or(""));
                title.push(' ');
            }
        }
        break;
    }
    (letters, title)
}

/// `--words TEXT` (D0375/issue397): the human's verbatim words as their OWN argument, folded into the
/// note inside a typographic quote pair so the boundary is declared, never inferred from an apostrophe
/// in the recorder's framing. Returns the args with `--words` gone and `--note` carrying the pair.
fn fold_words_into_note(args: &[String]) -> Vec<String> {
    let Some(words) = flag(args, "words") else { return args.to_vec() };
    let framing = flag(args, "note").unwrap_or_else(|| "recorded from chat".to_string());
    let note = format!("{framing} - their words, verbatim: \u{201C}{}\u{201D}", words.trim());
    let mut out = Vec::with_capacity(args.len());
    let mut skip_value = false;
    for a in args {
        if skip_value {
            skip_value = false;
        } else if a == "--words" || a == "--note" {
            skip_value = true;
        } else {
            out.push(a.clone());
        }
    }
    out.push("--note".to_string());
    out.push(note);
    out
}

fn cmd_accept(args: &[String]) -> i32 {
    let args = &match prose_args(args, &["note"], "accept") {
        Ok(a) => a,
        Err(code) => return code,
    };
    let args = &fold_words_into_note(args);
    let tty_gesture = tty_gesture();
    let args = &match accept_channel_refusal(args, tty_gesture) {
        Ok(warnings) => fold_warnings_into_note(args, &warnings),
        Err(exit) => return exit,
    };
    let root = find_repo_root().unwrap_or_else(|| PathBuf::from("."));
    let Some(decision) = args.first().filter(|a| !a.starts_with('-')) else {
        eprintln!("usage: keel accept <decision> (--note-from FILE | --note \"<what the human said>\") --by <humanActor> --date YYYY-MM-DD");
        eprintln!("       keel accept <decision> --words \"<their words, verbatim>\" [--note-from FILE | --note \"<framing>\"] --by <humanActor> --date YYYY-MM-DD");
        eprintln!("         --note is prose and goes through a FILE (D0224); --words stays a shell argument on purpose - it is the human's");
        eprintln!("         short quote, read back and recorded as given (D0192), not AI-typed prose.");
        eprintln!("         --words records the words inside a typographic quote pair, so an apostrophe in the framing can never shift the span (D0375/issue397);");
        eprintln!("         they are recorded as given - short words or words that do not name the decision land as a WARN line in the note (D0423).");
        eprintln!();
        eprintln!("Records a HUMAN's acceptance of a proposed Decision (D0106). The note must be what they");
        eprintln!("actually said — it IS the attestation, and `confirmation-authenticity` independently checks");
        eprintln!("that `--by` names a Person, so this cannot be used to self-accept an AI's own proposal.");
        return 2;
    };
    let (Some(note), Some(date)) = (flag(args, "note"), flag(args, "date")) else {
        eprintln!("error: --note and --date are both required. The note is the attestation; the date is when it was given.");
        return 2;
    };
    let judged_by = match keel_cli::actor::resolve(&root, flag(args, "by").as_deref()) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("{msg}");
            return 2;
        }
    };
    // WHO IS RECORDING (issue287): the session's own actor, never defaulted. With no `--by` the
    // judge and the recorder are the same person at their own terminal; with `--by <human>` an
    // agent is recording on the human's behalf and the record says so.
    let recorded_by = if flag(args, "by").is_some() {
        match keel_cli::actor::resolve(&root, None) {
            Ok(a) => a,
            Err(msg) => {
                eprintln!("keel accept: --by names the judge, but WHO IS RECORDING is unbound - {msg}");
                return 2;
            }
        }
    } else {
        judged_by.clone()
    };
    // Find the decision's file rather than making the caller supply it: a path argument here is a
    // chance to accept the wrong file, and the name is unambiguous.
    let mut found = None;
    for p in keel_cli::collect_sysml(&root.join(".engine").join("decisions")) {
        if std::fs::read_to_string(&p).is_ok_and(|t| t.contains(&format!("part {decision} : Decision"))) {
            found = Some(p);
            break;
        }
    }
    let Some(path) = found else {
        eprintln!("error: no Decision '{decision}' under .engine/decisions/.");
        return 2;
    };
    let sha = keel_cli::gitx::git()
        .arg("-C")
        .arg(&root)
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_owned())
        .unwrap_or_default();
    // D0315/issue359: the human at THEIR terminal is the gesture. Six acceptances typed at a TTY
    // with a plain sentence went red at the gate for want of a single-quoted span (GH#53) - the
    // rule accepts a cited gesture, and the command had not cited the one it could see. With stdin
    // a terminal (or KEEL_TTY_GESTURE=1, the test's stand-in), the note cites it; an agent session
    // has no TTY and is unchanged - its receipt is the human's quoted words (D0289).
    let note = match tty_gesture {
        Some(gesture) => keel_cli::view::note_with_tty_gesture(&note, gesture, &judged_by, &date),
        None => note,
    };
    // --rebind (D0308): the Decision is already accepted and its text moved since; record a new
    // acceptance result against the SHA whose text is current, with the note saying what changed.
    if args.iter().any(|a| a == "--rebind") {
        return match keel_cli::write::rebind_acceptance(&path, decision, &sha, &date, &judged_by, &recorded_by, &note) {
            Ok(_) => {
                println!("re-bound {decision}'s acceptance to {sha} (judged by {judged_by} at {date}; the first acceptance stands as when it took effect)");
                0
            }
            Err(e) => {
                eprintln!("keel accept --rebind: {e}");
                1
            }
        };
    }
    match keel_cli::write::accept_decision(&path, decision, &sha, &date, &judged_by, &recorded_by, &note) {
        Ok(_) => {
            println!("accepted {decision} (judged by {judged_by} at {date}, against {sha})");
            println!("  -> {}", path.strip_prefix(&root).unwrap_or(&path).display().to_string().replace('\\', "/"));
            println!("  run `keel gate validate . && keel gate guard .` — confirmation-authenticity checks that {judged_by} is a Person.");
            0
        }
        Err(e) => {
            eprintln!("error: {e}");
            1
        }
    }
}

/// `keel reject <d> --words "<verbatim>" --by <human> --date YYYY-MM-DD` (D0393/issue414): a human's
/// REJECTION of a proposed Decision, recorded through the write API with the same channel rules as
/// `keel accept` - delegation, quote receipt, read-back, TTY gesture, unbound-recorder refusal - and
/// `createdBy` stamped on the result (D0299). Before this the first rejection in the project's
/// history was a hand edit mirroring the writer's output, which no guard distinguishes from a
/// fabricated one; `write::reject_decision` existed only behind the console.
fn cmd_reject(args: &[String]) -> i32 {
    let args = &match prose_args(args, &["note"], "reject") {
        Ok(a) => a,
        Err(code) => return code,
    };
    let args = &fold_words_into_note(args);
    let tty_gesture = tty_gesture();
    let args = &match verdict_channel_refusal("reject", "rejecting", args, tty_gesture, "decisionAcceptance", true) {
        Ok(warnings) => fold_warnings_into_note(args, &warnings),
        Err(exit) => return exit,
    };
    let root = find_repo_root().unwrap_or_else(|| PathBuf::from("."));
    let Some(decision) = args.first().filter(|a| !a.starts_with('-')) else {
        eprintln!("usage: keel reject <decision> --words \"<their words, verbatim>\" [--note-from FILE | --note \"<framing>\"] --by <humanActor> --date YYYY-MM-DD");
        eprintln!("       keel reject <decision> (--note-from FILE | --note \"<what the human said>\") --by <humanActor> --date YYYY-MM-DD");
        eprintln!();
        eprintln!("Records a HUMAN's rejection of a proposed Decision (D0106): status becomes rejected and a");
        eprintln!("confirmation Test with a FAIL result carries their words. Same channel rules as `keel accept`.");
        return 2;
    };
    let (Some(note), Some(date)) = (flag(args, "note"), flag(args, "date")) else {
        eprintln!("error: --note (or --words) and --date are both required. The note is the attestation; the date is when it was given.");
        return 2;
    };
    let judged_by = match keel_cli::actor::resolve(&root, flag(args, "by").as_deref()) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("{msg}");
            return 2;
        }
    };
    let recorded_by = if flag(args, "by").is_some() {
        match keel_cli::actor::resolve(&root, None) {
            Ok(a) => a,
            Err(msg) => {
                eprintln!("keel reject: --by names the judge, but WHO IS RECORDING is unbound - {msg}");
                return 2;
            }
        }
    } else {
        judged_by.clone()
    };
    let mut found = None;
    for p in keel_cli::collect_sysml(&root.join(".engine").join("decisions")) {
        if std::fs::read_to_string(&p).is_ok_and(|t| t.contains(&format!("part {decision} : Decision"))) {
            found = Some(p);
            break;
        }
    }
    let Some(path) = found else {
        eprintln!("error: no Decision '{decision}' under .engine/decisions/.");
        return 2;
    };
    let sha = keel_cli::gitx::git()
        .arg("-C")
        .arg(&root)
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_owned())
        .unwrap_or_default();
    let note = match tty_gesture {
        Some(gesture) => keel_cli::view::note_with_tty_gesture(&note, gesture, &judged_by, &date),
        None => note,
    };
    match keel_cli::write::reject_decision(&path, decision, &sha, &date, &judged_by, &recorded_by, &note) {
        Ok(_) => {
            println!("rejected {decision} (judged by {judged_by} at {date}, against {sha})");
            println!("  -> {}", path.strip_prefix(&root).unwrap_or(&path).display().to_string().replace('\\', "/"));
            println!("  run `keel gate validate . && keel gate guard .` — confirmation-authenticity checks that {judged_by} is a Person.");
            0
        }
        Err(e) => {
            eprintln!("error: {e}");
            1
        }
    }
}

/// `keel judge-set <file> --words "<verbatim>" --by <human> --date YYYY-MM-DD [--verdict pass|fail]
/// [--fail <test>,..] [--all]` (D0443): a human's verdict on the SAMPLED proposed results of one
/// `.tracking` file, in one sitting - every item recorded on its own line with its own quote receipt
/// (D0312 B; issue158 is why a count is never one card). The channel rules are `keel accept`'s with
/// the `confirmationRecord` delegation and no Decision read-back: the subject is a file, not a Decision.
/// `judge-set` is in `HUMAN_ONLY_WRITE_COMMANDS`; the write layer refuses an AI-kind judge.
/// THE SET IS COMPUTED, NEVER CHOSEN (D0443): the sample from the policy's rule over the uuid order, or
/// with `--all` every proposal no human has judged; `--fail a,b` names the items that fail while the rest
/// take `verdict`. Returns the items and the file's proposal total, or the exit code when nothing awaits
/// or a `--fail` name is outside the set.
fn judge_set_items(root: &Path, path: &Path, rel: &str, all: bool, verdict: &str, fail: Option<&str>) -> Result<(Vec<keel_cli::write::SetJudgment>, usize), i32> {
    let fails: Vec<String> = fail.map(|f| f.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()).unwrap_or_default();
    let proposals = keel_cli::attestation::proposals_in(root, path);
    let rule = keel_cli::attestation::sampling_rule(root);
    let pending: Vec<&keel_cli::attestation::Proposal> = if all {
        proposals.iter().filter(|p| !p.judged).collect()
    } else {
        keel_cli::attestation::sample(&proposals, rule).into_iter().filter(|p| !p.judged).collect()
    };
    if pending.is_empty() {
        eprintln!(
            "keel judge-set: nothing awaits judgment in {rel} - {} proposed, {} judged{}. Nothing written.",
            proposals.len(),
            proposals.iter().filter(|p| p.judged).count(),
            if all || rule.is_none() { "" } else { ", the sample is judged (--all judges the rest)" }
        );
        return Err(2);
    }
    for f in &fails {
        if !pending.iter().any(|p| &p.test == f) {
            eprintln!("error: --fail names '{f}', which is not in the set awaiting judgment: {}", pending.iter().map(|p| p.test.as_str()).collect::<Vec<_>>().join(", "));
            return Err(2);
        }
    }
    let items = pending
        .iter()
        .map(|p| keel_cli::write::SetJudgment {
            test: p.test.clone(),
            verdict: if fails.contains(&p.test) { "fail".to_string() } else { verdict.to_string() },
        })
        .collect();
    Ok((items, proposals.len()))
}

fn cmd_judge_set(args: &[String]) -> i32 {
    let args = &match prose_args(args, &["note"], "judge-set") {
        Ok(a) => a,
        Err(code) => return code,
    };
    let args = &fold_words_into_note(args);
    let tty_gesture = tty_gesture();
    let args = &match verdict_channel_refusal("judge-set", "judging", args, tty_gesture, "confirmationRecord", false) {
        Ok(warnings) => fold_warnings_into_note(args, &warnings),
        Err(exit) => return exit,
    };
    let root = find_repo_root().unwrap_or_else(|| PathBuf::from("."));
    let Some(file) = args.first().filter(|a| !a.starts_with('-')) else {
        eprintln!("usage: keel judge-set <.tracking file> --words \"<their words, verbatim>\" [--note-from FILE | --note \"<framing>\"] --by <humanActor> --date YYYY-MM-DD [--verdict pass|fail] [--fail <test>,..] [--all]");
        eprintln!();
        eprintln!("Records a HUMAN's judgment of the SAMPLED proposed results in one file (D0443 on D0312 B): one TestResult");
        eprintln!("and one <test>Attest<N> quote receipt PER ITEM, never a count. The sample is computed from attestation-policy.toml");
        eprintln!("[proposedJudgment] sampling over the results' uuid order (`keel show attestation` shows proposed / sampled / judged);");
        eprintln!("--all judges every unjudged proposal in the file instead. --verdict applies to every item; --fail names the items");
        eprintln!("that fail while the rest pass.");
        return 2;
    };
    let (Some(note), Some(date)) = (flag(args, "note"), flag(args, "date")) else {
        eprintln!("error: --words (or --note) and --date are both required. The words are the attestation; the date is when they were given.");
        return 2;
    };
    let judged_by = match keel_cli::actor::resolve(&root, flag(args, "by").as_deref()) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("{msg}");
            return 2;
        }
    };
    let recorded_by = if flag(args, "by").is_some() {
        match keel_cli::actor::resolve(&root, None) {
            Ok(a) => a,
            Err(msg) => {
                eprintln!("keel judge-set: --by names the judge, but WHO IS RECORDING is unbound - {msg}");
                return 2;
            }
        }
    } else {
        judged_by.clone()
    };
    let path = root.join(file.replace('/', std::path::MAIN_SEPARATOR_STR));
    let rel = path.strip_prefix(&root).unwrap_or(&path).to_string_lossy().replace('\\', "/");
    if !rel.starts_with(".tracking/") || !path.is_file() {
        eprintln!("error: judge-set records into one existing file under .tracking/ - got '{file}'.");
        return 2;
    }
    let verdict = flag(args, "verdict").unwrap_or_else(|| "pass".to_string());
    if verdict != "pass" && verdict != "fail" {
        eprintln!("error: --verdict must be pass or fail (got '{verdict}').");
        return 2;
    }
    let all = args.iter().any(|a| a == "--all");
    let (items, total) = match judge_set_items(&root, &path, &rel, all, &verdict, flag(args, "fail").as_deref()) {
        Ok(v) => v,
        Err(exit) => return exit,
    };
    let sha = keel_cli::gitx::git()
        .arg("-C")
        .arg(&root)
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_owned())
        .unwrap_or_default();
    let note = match tty_gesture {
        Some(gesture) => keel_cli::view::note_with_tty_gesture(&note, gesture, &judged_by, &date),
        None => note,
    };
    match keel_cli::write::judge_set(&path, &items, &sha, &date, &judged_by, &recorded_by, &note) {
        Ok(written) => {
            println!("judged {} item(s) in {rel} (judged by {judged_by} at {date}, against {sha}; sample {} of {total} proposed{}):", written.len(), items.len(), if all { ", --all" } else { "" });
            for (w, i) in written.iter().zip(&items) {
                println!("  {w}: {} (+ quote receipt {}Attest)", i.verdict, i.test);
            }
            0
        }
        Err(e) => {
            eprintln!("error: {e}");
            1
        }
    }
}

/// `keel enroll` — the trailing ROOT is only honoured when it actually looks like a keel project,
/// so a stray argument cannot silently redirect an enrollment into the wrong tree.
fn cmd_enroll(rest: &[String]) -> i32 {
    let root = rest
        .iter()
        .rev()
        .find(|a| !a.starts_with("--"))
        .filter(|a| Path::new(a.as_str()).join(".tracking").is_dir())
        .map_or_else(|| find_repo_root().unwrap_or_else(|| PathBuf::from(".")), PathBuf::from);
    keel_cli::enroll::cmd(rest, &root)
}

/// The subcommand catalogue, printed when no arm matches.
/// Rendered from the CLI FACTS (`cli_facts::CLI_FACTS`, the mirror of `.engine/cli/commands.sysml`,
/// D0271/issue344) so a synopsis has one home. The hand-written CATALOGUE it replaces had already gone
/// stale against D0273 - it listed `hardening`, `suspect` and `outstanding` as verbs a month after they
/// moved under `show` - which is the drift a table-plus-test cannot see and a fact-plus-guard can.
fn print_usage() -> i32 {
    eprint!("{}", keel_cli::cli_facts::render_help());
    2
}
/// A read-only view subcommand: run `f` against the repo root and print its JSON, or the error as
/// JSON so a consumer parsing stdout gets a parseable answer either way.
/// `keel render decision-card [NAME] [--proposed]` (D0205; under `render` since D0449): machine-readable deciding context.
fn cmd_decision_card(rest: &[String]) -> i32 {
    let name = rest.first().filter(|a| !a.starts_with('-')).map(String::as_str);
    let proposed = rest.iter().any(|a| a == "--proposed");
    let root = find_repo_root().unwrap_or_else(|| PathBuf::from("."));
    match keel_cli::deck::decision_cards(&root, name, proposed) {
        Ok(s) => {
            println!("{s}");
            0
        }
        Err(e) => {
            eprintln!("error: {e}");
            1
        }
    }
}

fn budget_arg(args: &[String]) -> usize {
    flag(args, "budget").and_then(|v| v.parse().ok()).unwrap_or(RECALL_BUDGET)
}

/// `keel recall --prompt - [--budget N] [ROOT]` — seed from a PROMPT and print a budgeted brief.
///
/// The prompt arrives on STDIN, never as an argument: it is free-form text that may contain quotes,
/// newlines and backticks, and the one thing this path must never do is let that text reach a shell.
fn cmd_recall(rest: &[String]) -> i32 {
    let usage = "keel recall --prompt - [--budget N] [ROOT]   (the prompt arrives on STDIN)";
    if flag(rest, "prompt").as_deref() != Some("-") {
        eprintln!("usage: {usage}");
        eprintln!("  `--prompt -` is required and means: read the prompt from stdin.");
        return 2;
    }
    let positional = without_flag_values(rest, &["prompt", "budget"]);
    let root = match root_arg(&positional, usage, &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    let mut prompt = String::new();
    if std::io::Read::read_to_string(&mut std::io::stdin(), &mut prompt).is_err() {
        eprintln!("recall: could not read the prompt from stdin");
        return 2;
    }
    match keel_cli::view::recall_for_prompt(&root, &prompt, budget_arg(rest)) {
        Ok(s) => {
            print!("{s}");
            0
        }
        Err(e) => {
            eprintln!("recall error: {e}");
            1
        }
    }
}

/// `keel why <term> [ROOT]` (D0161): seed on names/titles/aliases, traverse, answer with provenance.
fn cmd_why(rest: &[String]) -> i32 {
    // `--budget N` consumes N, so the positionals have to be taken from the STRIPPED list or the
    // number becomes the root — measured: `keel why keystone --brief --budget 1500` recalled 0 seeds
    // because it built a model at `./1500`.
    let positional = without_flag_values(rest, &["budget"]);
    let bare: Vec<&String> = positional.iter().filter(|a| !a.starts_with("--")).collect();
    let Some(term) = bare.first() else {
        eprintln!("usage: keel why <term> [ROOT] [--brief] [--budget N]");
        return 2;
    };
    let Some(root) = bare.get(1).map(|p| PathBuf::from(p.as_str())).or_else(find_repo_root) else {
        eprintln!("usage: keel why <term> [ROOT] [--brief] [--budget N]");
        return 2;
    };
    // `--brief` is the INJECTABLE form: budgeted, content-bearing text rather than JSON whose seeds
    // are bare identifiers. The JSON form is unchanged for every existing caller.
    if rest.iter().any(|a| a == "--brief") {
        return match keel_cli::view::why_brief(&root, term, budget_arg(rest)) {
            Ok(s) => {
                print!("{s}");
                0
            }
            Err(e) => {
                eprintln!("why error: {e}");
                1
            }
        };
    }
    match keel_cli::view::why(&root, term) {
        Ok(s) => {
            println!("{s}");
            0
        }
        Err(e) => {
            eprintln!("error: {e}");
            1
        }
    }
}

/// `keel knowledge question-coverage [ROOT]` (D0161): per declared Question, does seeding find an
/// entity and traversal reach an answer. Computed, never authored.
fn cmd_knowledge(rest: &[String]) -> i32 {
    if rest.first().map(String::as_str) != Some("question-coverage") {
        eprintln!("usage: keel knowledge question-coverage [ROOT]");
        return 2;
    }
    cmd_view0(rest.get(1..).unwrap_or(&[]), "knowledge question-coverage", keel_cli::view::question_coverage)
}

fn cmd_view0(
    rest: &[String],
    name: &'static str,
    f: fn(&Path) -> Result<String, keel_cli::view::ViewError>,
) -> i32 {
    // `cmd_query0` takes a fn POINTER, and a closure capturing `f` is not one — so the root is
    // resolved here and the view invoked directly, rather than threading a capture through it.
    let Some(root) = rest.first().map(PathBuf::from).or_else(find_repo_root) else {
        eprintln!("usage: keel {name} [ROOT]");
        return 2;
    };
    // issue281: a zero-argument VIEW must not answer over nothing either. `cmd_view0` resolves
    // its own root rather than going through `root_arg`, so the shared precondition missed it -
    // found by sweeping every command at a workspace root, where `controls` still exited 0.
    if let Err(code) = keel_cli::workspace::require_project(&root, &format!("keel {name} [ROOT]")) {
        return code;
    }
    println!("{}", f(&root).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")));
    0
}

/// The repo a git-touching subcommand acts on: the first non-flag argument, else the discovered root.
fn repo_arg(rest: &[String]) -> PathBuf {
    rest.iter()
        .find(|a| !a.starts_with("--"))
        .map_or_else(|| find_repo_root().unwrap_or_else(|| PathBuf::from(".")), PathBuf::from)
}

fn cmd_version(args: &[String]) -> i32 {
    let hard = keel_cli::guards::GUARD_NAMES.len() - WARNING_ONLY_GUARDS.len();
    if args.iter().any(|a| a == "--json") {
        println!(
            "{{\"version\":\"{}\",\"buildCommit\":\"{}\",\"guards\":{},\"guardsHardBlocking\":{},\"guardsWarningOnly\":{}}}",
            env!("CARGO_PKG_VERSION"),
            env!("KEEL_BUILD_COMMIT"),
            keel_cli::guards::GUARD_NAMES.len(),
            hard,
            WARNING_ONLY_GUARDS.len(),
        );
        return 0;
    }
    println!("keel {}", env!("CARGO_PKG_VERSION"));
    println!("build commit: {}", env!("KEEL_BUILD_COMMIT"));
    // D0190: the binary version is the ONE declared semver; the others are derived facts reported
    // beside it (a breaking API change is recorded in the release Decision, not versioned apart).
    println!("api contract: {} (derived; breaking changes recorded in release Decisions)", keel_cli::serve::KEEL_API_VERSION);
    println!("claude surface: {} (generated from this binary)", keel_cli::claude_surface::SURFACE_VERSION);
    match find_repo_root().map(|r| engine_version_skew(&r)) {
        Some(Some(w)) => println!("engine declared: SKEW - {w}"),
        Some(None) => println!("engine declared: matches (engine-version.toml or pre-D0190 absent)"),
        None => {}
    }
    println!(
        "guards: {} ({hard} hard-blocking, {} warning-only)",
        keel_cli::guards::GUARD_NAMES.len(),
        WARNING_ONLY_GUARDS.len(),
    );
    0
}

/// The warning-only members of `GUARD_NAMES` — they RUN on every commit and are visible, but never
/// block (the D0102 promote-once-low-noise pattern). Named here so `keel version` can report the
/// hard-vs-warning split without a hand-maintained count.
const WARNING_ONLY_GUARDS: [&str; 9] =
    ["decision-requirement-link", "verification-trace", "priority-inversion", "retro-backlog", "doc-sync", "hook-config-integrity", "sequence-multiplicity", "parser-coverage", "base-first-justification"];

/// `keel show flow [ROOT] [--json]` - the git-derived flow series (dcCycleTimeReadsFromGit). The lens
/// answers in JSON like every lens; `--json` is accepted as the explicit spelling the Definition of Done names.
fn cmd_flow(args: &[String]) -> i32 {
    let rest: Vec<String> = args.iter().filter(|a| *a != "--json").cloned().collect();
    cmd_view0(&rest, "show flow", keel_cli::view::flow::flow)
}

#[allow(clippy::too_many_lines)] // one dispatch table = one place a subcommand can be reached from;
// splitting it by arbitrary length would hide half the surface from anyone reading for what exists
/// `keel show <lens> [ROOT] ...` — the ONE read-only lens surface (D0273).
///
/// # What replaced what
///
/// 35 top-level verbs each computed one view of the model and printed it. The human's words for that
/// were "a lot of these seem to be variations of an idea cemented as separate cli hooks", and they
/// chose the CLEAN BREAK over an alias window: the old spellings are GONE, not deprecated. Every arm
/// below is the arm that used to sit in `main`, moved verbatim — a consolidation that changed an
/// answer would be a rewrite wearing a rename's clothes, so the calls are untouched and byte-equality
/// against the pre-collapse binary is asserted per lens.
///
/// # Why removal, and why it is safe to remove
///
/// An alias list that shrinks "as call sites move" has no forcing function. The cost the human
/// accepted is that every skill naming an old verb must move in the SAME commit, which is why this
/// lands with its 118 call sites rewritten and guard 39 green throughout — no tree ever exists in
/// which a skill names a command the binary lacks. Downstream, D0252 clause A's capability refusal
/// makes the break a NAMED error at import rather than a silent no-op.
fn cmd_show(args: &[String]) -> i32 {
    let rest: &[String] = args.get(1..).unwrap_or(&[]);
    match args.first().map(String::as_str) {
            Some("actor-trace") => cmd_query1(rest, "actor-trace", |r, a| keel_cli::view::actor_trace(r, a).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))),
            // `repo_arg(rest)` would take the SUBCOMMAND as the path — `arch elements .` resolved the
            // root to `./elements`, whose empty model then printed "no CodeElement instances authored".
            Some("arch") => keel_cli::arch::cmd(rest, &repo_arg(rest.get(1..).unwrap_or(&[]))),
            Some("assumptions") => cmd_view0(rest, "assumptions", keel_cli::view::assumptions),
            Some("attestation") => keel_cli::attestation::cmd(rest),
            Some("attestation-coverage") => cmd_attestation_coverage(rest),
            Some("authority-queue") => cmd_view0(rest, "authority-queue", keel_cli::view::authority_queue),
            Some("boundary") => cmd_query1(rest, "boundary", |r, need| keel_cli::view::boundary_json(r, need).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))),
            Some("boundary-sweep") => cmd_query0(rest, "keel boundary-sweep [ROOT]", |r| keel_cli::view::boundary_sweep_json(r).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))),
            Some("business") => cmd_business(rest),
            Some("commit-delta") => cmd_commit_delta(rest),
            Some("concern-coverage") => cmd_concern_coverage(rest),
            Some("contentions") => cmd_view0(rest, "contentions", keel_cli::view::contentions),
            Some("controls") => cmd_view0(rest, "controls", keel_cli::view::controls),
            Some("control-census") => cmd_view0(rest, "control-census", keel_cli::view::census::control_census),
            Some("control-structure") => {
                // `--svg` draws the STPA diagram in the binary (D0285, the Python interim retired):
                // same computed structure, serialised as the picture instead of the JSON.
                let svg = rest.iter().any(|a| a == "--svg");
                let rest: Vec<String> = rest.iter().filter(|a| *a != "--svg").cloned().collect();
                if svg {
                    cmd_view0(&rest, "control-structure --svg", keel_cli::view::stpa_diagram::control_structure_svg)
                } else {
                    cmd_view0(&rest, "control-structure", keel_cli::view::control_structure::control_structure)
                }
            }
            Some("coverage") => cmd_coverage(rest),
            Some("critique-coverage") => cmd_critique_coverage(rest),
            Some("critique-policy") => cmd_critique_policy(rest),
            Some("decision-follow-through") => cmd_decision_follow_through(rest),
            Some("decisions") => cmd_decisions(rest),
            Some("dispositions") => cmd_dispositions(rest),
            Some("enforcement-report") => cmd_enforcement_report(rest),
            Some("flow") => cmd_flow(rest),
            Some("governing-version") => cmd_governing_version(rest),
            Some("hardening") => cmd_hardening(rest),
            Some("indicators") => cmd_indicators(rest),
            Some("intake") => cmd_intake(rest),
            Some("item") => cmd_query1(rest, "item", keel_cli::queries::item),
            Some("knowledge") => cmd_knowledge(rest),
            Some("launchables") => cmd_launchables(rest),
            Some("ls") => cmd_ls(rest),
            Some("marker-census") => cmd_view0(rest, "marker-census", keel_cli::view::marker_census),
            Some("open-issues") => cmd_open_issues(rest),
            Some("orient") => cmd_orient(rest),
            Some("priority") => cmd_priority(rest),
            Some("orphans") => cmd_orphans(rest),
            Some("outstanding") => cmd_query0(rest, "outstanding", keel_cli::queries::outstanding),
            Some("recent") => cmd_query0(rest, "keel recent [ROOT]", |r| keel_cli::view::recent(r).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))),
            Some("reprocess-candidates") => cmd_reprocess_candidates(rest),
            Some("rootedness") => cmd_query0(rest, "keel rootedness [ROOT]", |r| keel_cli::view::rootedness(r).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))),
            Some("sitting-coverage") => cmd_sitting_coverage(rest),
            Some("status") => cmd_status(rest),
            Some("suspect") => cmd_suspect(rest),
            Some("tier-satisfaction") => cmd_query0(rest, "keel tier-satisfaction [ROOT]", |r| keel_cli::view::tier_satisfaction(r).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))),
            Some("trace") => cmd_query1(rest, "trace", keel_cli::queries::trace),
            Some("trace-need") => cmd_query1(rest, "trace-need", keel_cli::queries::trace_need),
            Some("verification") => {
                let root = repo_arg(rest);
                keel_cli::workspace::require_project(&root, "keel verification [ROOT] [--pending]")
                    .map_or_else(|code| code, |()| keel_cli::verification::cmd(rest, &root))
            }
            Some("view") => cmd_view(rest),
            Some("whats-next") => cmd_whats_next(rest),
            Some("why") => cmd_why(rest),
            Some("workflows") => cmd_query0(rest, "workflows", keel_cli::queries::workflows),
        Some(other) => {
            eprintln!("keel show: unknown lens `{other}`.");
            eprintln!("  Lenses: {}", keel_cli::cli_surface::LENS_NAMES.join(", "));
            2
        }
        None => {
            eprintln!("usage: keel show <lens> [ROOT] [flags]");
            eprintln!("  Lenses: {}", keel_cli::cli_surface::LENS_NAMES.join(", "));
            2
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let rest: &[String] = args.get(2..).unwrap_or(&[]);
    let code = match args.get(1).map(String::as_str) {
        // Version must answer BEFORE any root resolution — a caller establishing which binary they
        // have may be standing anywhere, including outside a keel project.
        Some("version" | "--version" | "-V") => cmd_version(rest),
        Some("init") => cmd_init(rest),
        Some("sync") => keel_cli::sync::cmd_sync(&repo_arg(rest)),
        Some("land") => keel_cli::sync::cmd_land(&repo_arg(rest), 3),
        Some("migrate") => cmd_migrate(rest),
        // D0138: what has this project ADOPTED — declared, not inferred from file presence.
        Some("process") => keel_cli::process_cmd::cmd(rest, &find_repo_root().unwrap_or_else(|| PathBuf::from("."))),
        Some("onboard") => keel_cli::onboard::cmd(rest),
        Some("projects") => keel_cli::workspace::cmd(rest),
        Some(v @ ("activation" | "activate" | "deactivate")) => cmd_activation(v, rest),
        Some("serve") => cmd_serve(rest),
        Some("hook") => cmd_hook(rest), // D0134: in-loop gates in the BINARY, no python runtime
        // D0128 Tier-2: the fast per-edit in-loop gate; D0452: the seven gating verbs route under it.
        Some("gate") => cmd_gate(rest),
        Some("library") => keel_cli::library::run(rest),
        Some("show") => cmd_show(rest),
        Some("audit") => cmd_audit(rest), // D0452: history, adherence and ci-runs route under it
        Some("deck") => cmd_deck(rest),
        Some("sync-claude") => cmd_sync_claude(rest),
        Some("claude") => cmd_claude(rest),
        Some("override") => cmd_override(rest),
        Some("recall") => cmd_recall(rest),
        // D0129/issue072: inspect or bind this machine's acting identity (never defaulted).
        Some("actor") => keel_cli::actor::cmd(rest, &find_repo_root().unwrap_or_else(|| PathBuf::from("."))),
        Some("claim") => keel_cli::claim::cmd(rest, &find_repo_root().unwrap_or_else(|| PathBuf::from("."))),
        Some("github") => cmd_github(rest), // D0453: pull, ingest, decider, gesture and decision-id route under it
        Some("currency") => cmd_currency(rest), // D0338: the unattended pass - pull, library, drift
        Some("suite") => cmd_suite(rest), // D0353: the full suite, with the receipt land demands
        Some("verify") => cmd_verify(rest), // D0476: the pre-commit ladder, stopping at the first red
        Some("advance") => keel_cli::cursor::advance_cmd(rest, &find_repo_root().unwrap_or_else(|| PathBuf::from("."))),
        Some("enroll") => cmd_enroll(rest),
        Some("render") => cmd_render(rest),
        Some("accept") => cmd_accept(rest),
        Some("reject") => cmd_reject(rest), // D0393/issue414: the human's rejection through the write API
        Some("judge-set") => cmd_judge_set(rest), // D0443: the human's judgment of a sampled set of proposed results
        Some("record") => cmd_record(rest),
        _ => print_usage(),
    };
    // STDERR, always, and after the command's own output: a computed view's stdout is JSON that
    // automation parses, so a perf line on stdout would corrupt every caller that asked for numbers.
    if let Some(r) = keel_cli::perf::report() {
        eprintln!("{r}");
    }
    process::exit(code);
}

#[cfg(test)]
mod tests {
    use super::{
        classify_guard_args, remap_engine_content, remap_engine_path, root_arg, Path,
    };

    #[test]
    fn an_unknown_flag_is_a_mistake_and_never_a_root() {
        // issue133: the whole class. A mistyped flag used to BECOME the root path (or, in the
        // skip-flags variant, be silently ignored), so a typo produced a confident wrong answer
        // somewhere downstream instead of an error where the mistake was made.
        let a = |v: &[&str]| v.iter().map(|s| (*s).to_string()).collect::<Vec<_>>();
        assert_eq!(root_arg(&a(&["--explan"]), "u", &[], 0), Err(2));
        assert_eq!(root_arg(&a(&["--explan"]), "u", &["explain"], 0), Err(2), "a NEAR-MISS of a known flag is still unknown");
        // A REAL project path, because `root_arg` now VALIDATES as well as parses (issue281): it
        // refuses a root that is not a keel project, so a synthetic `/r` no longer reaches the caller.
        // The assertions below still test what they always did — that the positional is FOUND around a
        // declared flag — they just use a root that a caller could really pass.
        let repo = keel_fs::test_support::repo_root();
        let r = repo.to_string_lossy().to_string();
        let found = |v: &[&str], pos: usize| {
            root_arg(&a(v), "u", &["explain"], pos).ok().map(|p| p.to_string_lossy().to_string())
        };
        assert_eq!(found(&["--explain", &r], 0), Some(r.clone()));
        assert_eq!(found(&[&r, "--explain"], 0), Some(r.clone()));
        // `positionals` skips the subcommand's own leading argument (`keel show view <name> [ROOT]`)
        assert_eq!(found(&["decisions", &r], 1), Some(r.clone()));
        // and a leading positional alone leaves ROOT to repo discovery, not to the positional
        assert_ne!(root_arg(&a(&["decisions"]), "u", &[], 1).map(|p| p.to_string_lossy().to_string()), Ok("decisions".to_string()));
        // The new half of the contract: a path that exists but is NOT a project is refused, never
        // answered over. This is the false green issue281 closed.
        let tmp = std::env::temp_dir();
        assert_eq!(root_arg(&a(&[&tmp.to_string_lossy()]), "u", &[], 0), Err(2), "a non-project root is refused");
    }

    #[test]
    fn guard_args_distinguish_name_from_root() {
        // Regression (v0.1.0 release smoke): `keel gate guard <ROOT>` must run all guards on ROOT, not read
        // ROOT as a guard name. A known name runs that one guard; "all"/no-arg/a path runs all.
        let s = |v: &[&str]| v.iter().map(|x| (*x).to_string()).collect::<Vec<_>>();
        assert_eq!(classify_guard_args(&s(&[])), (None, None)); // run all, default root
        assert_eq!(classify_guard_args(&s(&["all"])), (None, None)); // run all
        assert_eq!(classify_guard_args(&s(&["myproj"])), (None, Some("myproj"))); // bare ROOT -> run all on it
        assert_eq!(classify_guard_args(&s(&["."])), (None, Some("."))); // "." is a ROOT, not a guard
        assert_eq!(classify_guard_args(&s(&["ceremony"])), (Some("ceremony"), None)); // a known guard name
        assert_eq!(classify_guard_args(&s(&["ceremony", "myproj"])), (Some("ceremony"), Some("myproj"))); // name + root
        assert_eq!(classify_guard_args(&s(&["all", "myproj"])), (None, Some("myproj"))); // all on root
    }

    #[test]
    fn engine_path_remap_isolates_decisions() {
        // D0093 boundary: decisions ship as read-only reference, never as the new project's instance.
        assert_eq!(remap_engine_path(Path::new("decisions/0001-x.sysml")), Path::new("reference/decisions/0001-x.sysml"));

        // Everything else is scaffolded unchanged.
        assert_eq!(remap_engine_path(Path::new("schema/core/element.sysml")), Path::new("schema/core/element.sysml"));
        assert_eq!(remap_engine_path(Path::new("processes/introduction.sysml")), Path::new("processes/introduction.sysml"));
    }

    /// issue291: the reference copy's package must be renamed, or the project's own first recorded
    /// decision collides with it. Exercises the real corpus shape - a `// D0001` prose comment above
    /// the declaration, and a `procedureText` mentioning `d0001` - because the rename must touch the
    /// declaration ONLY (189 of 514 `dNNNN` occurrences downstream sit inside prose strings).
    #[test]
    fn reference_decision_package_is_renamed_declaration_only() {
        let src = concat!(
            "// D0001 - text files are truth\n",
            "package Decision0001 {\n",
"    part d0001 : Decision {\n",
"        :>> procedureText = \"ww confirmed d0001 on the call\";\n",
"    }\n",
            "}\n",
        );
        // `unwrap_or_default` rather than `expect`: the fail-loud lints deny panic/expect/unwrap
        // even here, and an empty string fails every assert below with the content in the message.
        let out = remap_engine_content(Path::new("decisions/0001-text-files-are-truth.sysml"), src)
            .unwrap_or_default();
        assert!(out.contains("package ReferenceDecision0001 {"), "package renamed: {out}");
        assert!(out.contains("part d0001 : Decision"), "part name untouched (global resolution): {out}");
        assert!(out.contains("confirmed d0001 on the call"), "prose untouched: {out}");
        assert!(out.contains("// D0001 - text files are truth"), "comment untouched: {out}");
        // Idempotent: the transform's own output declares no `package DecisionNNNN` to rename, which
        // is what keeps `step_engine_resync` from planning an edit every run.
        assert!(remap_engine_content(Path::new("decisions/0001-x.sysml"), &out).is_none());
    }

    /// Only files remapped INTO `reference/decisions/` are transformed - a schema or process file
    /// that happens to mention a decision is copied byte-for-byte.
    #[test]
    fn non_decision_engine_files_are_never_content_transformed() {
        let src = concat!(
            "package EngineRules {\n",
"    #JustifiedBy dependency from r to d0099;\n",
            "}\n",
        );
        assert!(remap_engine_content(Path::new("rules/rules.sysml"), src).is_none());
        assert!(remap_engine_content(Path::new("schema/core/element.sysml"), src).is_none());
    }

    #[test]
    fn version_guard_split_cannot_drift_from_the_guards_that_run() {
        // `keel version` reports the hard-vs-warning split by SUBTRACTING the warning list from
        // GUARD_NAMES. If a name in the warning list is not an actual enforced guard the reported hard
        // count silently overstates enforcement — and a longer warning list would underflow the
        // subtraction outright. Both are the same defect class as the version gap this command fixes:
        // a number a reader would trust that nothing checks.
        for w in super::WARNING_ONLY_GUARDS {
            assert!(
                keel_cli::guards::GUARD_NAMES.contains(&w),
                "warning-only guard `{w}` is not in GUARD_NAMES — the reported hard count would be wrong"
            );
        }
        assert!(
            super::WARNING_ONLY_GUARDS.len() < keel_cli::guards::GUARD_NAMES.len(),
            "warning list must be a strict subset — otherwise the hard count underflows"
        );
    }

    #[test]
    fn init_ships_downstream_claude_md_not_self_build() {
        // issue057 (field defect): `keel init` must ship a DOWNSTREAM "tracked by keel" CLAUDE.md,
        // NEVER the self-build's ("This repo is a work-tracking engine"). D0047 permanent control.
        assert!(super::CLAUDE_MD.contains("tracked by keel"), "init CLAUDE.md must frame the project as tracked BY keel");
        assert!(!super::CLAUDE_MD.contains("is a work-tracking engine"), "init must NOT ship the self-build CLAUDE.md");
        assert!(super::CLAUDE_MD.contains("Parsed:"), "downstream CLAUDE.md must carry the D0106 parse-first discipline");
    }

    /// issue379 / GH#54: only a build another machine can obtain may seed the wrapper cache.
    #[test]
    fn only_a_reproducible_build_may_seed_the_wrapper_cache() {
        assert!(super::is_reproducible_build("c4d5dbf"));
        assert!(!super::is_reproducible_build("4674965+dirty"), "a modified tree is nobody else's binary");
        assert!(!super::is_reproducible_build("unknown"), "off-git is not a version");
        assert!(!super::is_reproducible_build(""));
    }

    /// THE CONTROL for issue243: `keel init` must not ship THIS project's instance data as if it were
    /// engine definition. A fresh tree inherited a byte-identical activation.toml, so it reported
    /// `declared manifest: yes` for an adoption declaration it never made - falsifying the guarantee
    /// that an absent manifest means everything is active - and inherited the self-build's EXCHANGE
    /// IDENTITIES for units it never exported.
    #[test]
    fn init_resets_instance_contracts_and_keeps_engine_definition() {
        use std::path::Path;
        for p in ["contracts/activation.toml", "contracts/unit-ids.toml", "contracts/installed-units.toml",
                  "contracts/github-actors.toml"] {
            assert!(super::is_instance_contract(Path::new(p)), "{p} is instance data and must be reset");
        }
        // Engine DEFINITION must still be copied verbatim.
        for p in ["contracts/process-enforcement.toml", "processes/intake.sysml", "rules/rules.sysml"] {
            assert!(!super::is_instance_contract(Path::new(p)), "{p} is engine definition and must be shipped as-is");
        }
        // The activation starter must leave the honest default IN FORCE, i.e. no live [processes]
        // section - a commented template is guidance, an uncommented one is a declaration.
        let starter = super::starter_for(Path::new("contracts/activation.toml"));
        for line in starter.lines() {
            let l = line.trim();
            assert!(
                l.is_empty() || l.starts_with('#'),
                "the activation starter must declare NOTHING; found a live line: {l}"
            );
        }
        // The id registries start genuinely empty: no `name = "uuid"` entry may be inherited.
        let ids = super::starter_for(Path::new("contracts/unit-ids.toml"));
        assert!(!ids.lines().any(|l| !l.trim_start().starts_with('#') && l.contains('=')),
            "an inherited unit id claims a lineage this project does not have");
        // D0219: the decider table must start EMPTY - an inherited decider could record
        // acceptances in another project's tree under someone else's name.
        let gh = super::starter_for(Path::new("contracts/github-actors.toml"));
        assert!(gh.contains("[logins]"), "the starter must still show the section shape");
        assert!(!gh.lines().any(|l| !l.trim_start().starts_with('#') && l.contains('=')),
            "no login may be inherited: who may decide is per-project");
    }
}
