//! The `keel` verbs this member owns (D0479, sprint 750): `cmd_validate`, `cmd_gate`, `cmd_sync_claude`, `cmd_indicators`, `cmd_init`, `cmd_status`, `cmd_migrate`, `cmd_enroll`, `cmd_knowledge`, `cmd_view0`, `cmd_flow`.
//!
//! Moved out of `keel-cli/src/main.rs` by `scripts/extract_verbs.py`: the binary is argument dispatch, and a
//! verb body lives with the member whose reach it needs (`python scripts/verb_homes.py --by-member`).
//! Each `cmd_*` takes the arguments after its verb and returns the process exit code.
//
// These were the binary's private items, written under the same lint set; only the lints that fire on
// PUBLIC visibility are new here, and they ask for ceremony a verb body does not owe: its exit code goes
// straight back to `main`, and its `Err` and panics are the ones the body already documents in place.
#![allow(clippy::must_use_candidate, clippy::missing_errors_doc, clippy::missing_panics_doc)]

use crate::migrate::{is_engine_dev_only, remap_engine_content, remap_engine_path};
use keel_args::{flag, positional_arg, refuse_flag_as_path, root_arg};
use keel_fs::fs_verbs::print_init_next_steps;
use keel_git::git_verbs::resolve_guard_root;
use keel_git::projects::{engine_version_skew, find_repo_root};
use keel_guards::guards_verbs::{cmd_assured, cmd_guard};
use keel_model::model_verbs::cmd_check;
use keel_model::validate::validate_root;
use keel_schema::embedded::ENGINE_DIR;
use keel_view::view_verbs::cmd_rules;
use keel_write::ledger::ledger_gate;
use keel_write::write_verbs::{cmd_check_engine, init_enforcement_surface, install_commit_gate};
use std::path::{Path, PathBuf};

// A DOWNSTREAM CLAUDE.md template (issue057): a fresh project is TRACKED BY keel, not keel itself.
// The self-build repo's own CLAUDE.md (about building the engine) is NEVER shipped to init'd projects.
pub const CLAUDE_MD: &str = include_str!("../assets/claude-md-template.md");


pub const TRACKING_STARTER: &str = "# .tracking/ — your project's instance data\n\nThis directory holds THIS project's authored facts (needs, requirements, work items, issues,\ndecisions, test results) — the per-project INSTANCE. The reusable engine lives in `.engine/`.\n\nGetting started: run the `introduction` skill (guided onboarding), or author your first `Need`\nfollowing `.engine/docs/tracking-template.sysml`. State is COMPUTED — run `keel show orient .` to\nsee where things stand. The engine's design rationale is read-only in `.engine/reference/decisions/`;\nyour project authors its OWN decisions fresh in `.engine/decisions/`.\n";


/// A fresh project's deliverable-suspicion manifest is EMPTY — the shipped one lists the ENGINE's own
/// deliverable tasks (instance-specific), which would fail manifest-coverage on a new project (D0093
/// engine/instance boundary). The new project adds entries as it builds source-dependent verifications.
pub const STARTER_MANIFEST: &str = "# deliverable-manifest.txt — declares which verification tasks depend on which DELIVERABLE SOURCE\n# files (D0050), so `keel suspect` flags a task suspect when its source changed since it was\n# verified. One entry per line:  task: <taskName> | <relpath> <relpath> ...\n# Empty for a new project — add an entry when you have a deliverable-source-dependent verification.\n";


/// A starter actor registry scaffolded into a fresh project (`.tracking/actors.sysml`). Without it
/// the newcomer's FIRST recorded fact (any `createdBy`/`judgedBy`) fails the actors guard (D0037) —
/// there'd be no `ProjectActors` to reference. Ships placeholder actors (a human + the AI) the newcomer
/// edits to their real identities; the declared part name is the id that `createdBy`/`judgedBy` reference.
pub const STARTER_ACTORS: &str = "// ProjectActors — this project's actor registry (INSTANCE data). EDIT to your real actors.\n// The declared part name is the id that createdBy/judgedBy reference (enforced by `keel gate guard actors`).\npackage ProjectActors {\n    private import EngineElement::*;\n\n    part you : Person { :>> name = \"Your Name\"; :>> email = \"you@example.com\"; }\n    part ai : Actor { :>> name = \"AI assistant\"; :>> kind = ActorKind::ai; }\n}\n";


/// Scaffolded `.gitignore`. Machine-local state only — nothing here is a build artifact of the
/// project, it is state that is TRUE OF ONE CLONE and false of every other.
pub const GITIGNORE: &str = "# keel machine-local state — never commit these.
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

// ── subcommands ───────────────────────────────────────────────────────────────


pub fn cmd_validate(args: &[String]) -> i32 {
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
    if !crate::workspace::is_project(&root) {
        // NAME THE DIRECTORY THAT IS MISSING (issue283).
        eprintln!("error: {} is not a keel project — it has no {} directory.", root.display(), crate::workspace::missing_project_dirs(&root));
        let ws = crate::workspace::discover(&root);
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
        println!("{}", keel_json::color::pass(&format!("{} tracking file(s) validated clean.", report.validated)));
        0
    } else {
        eprintln!(
            "{}",
            keel_json::color::fail(&format!(
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
pub fn gate_subverb(args: &[String]) -> Option<i32> {
    let rest = args.get(1..).unwrap_or(&[]);
    Some(match args.first().map(String::as_str)? {
        "validate" => refuse_flag_as_path(rest.first(), "gate validate").unwrap_or_else(|| cmd_validate(rest)),
        "check" => cmd_check(rest),
        "check-engine" => refuse_flag_as_path(rest.first(), "gate check-engine").unwrap_or_else(|| cmd_check_engine(rest)),
        "guard" => cmd_guard(rest),
        "rules" => cmd_rules(rest),
        "assured" => cmd_assured(rest),
        "adoption-check" => crate::adoption_check::cmd(rest),
        _ => return None,
    })
}


/// The `keel gate` usage: the two tiers, then one line per sub-verb the router resolves.
pub fn print_gate_usage() {
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
pub fn cmd_gate(args: &[String]) -> i32 {
    if let Some(code) = gate_subverb(args) {
        return code;
    }
    // D0234: `--workspace` is a SCOPE (every project in this git repo), `--fast` is a TIER (the
    // per-edit subset). A repo holding several projects can only have one core.hooksPath, so its
    // pre-commit hook calls this rather than a per-project gate that could cover just one of them.
    if args.iter().any(|a| a == "--workspace") {
        return crate::workspace::gate_cmd(args);
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
    if !keel_guards::receipt::forced(args) {
        if let Some(r) = keel_guards::receipt::key(&root).and_then(|k| keel_guards::receipt::read(&root, &k)) {
            if r.covers_all(&[keel_guards::receipt::VALIDATE, keel_guards::receipt::GUARDS]) {
                println!("{}", r.line("gate: fast gate clean -"));
                return 0;
            }
        }
    }
    let report = keel_model::validate::validate_root(&root);
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
        if let Some(r) = keel_guards::run_one(name, &root) {
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


pub fn cmd_sync_claude(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel sync-claude [ROOT] [--check]", &["check"], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    let check = args.iter().any(|a| a == "--check");
    match keel_write::claude_surface::sync_claude(&root, check) {
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
                    keel_write::claude_surface::surface_stamp()
                );
                // D0391/issue408: the surface that writes the hook commands also says - and on a self-build
                // tree places - the binary those commands will resolve to.
                match keel_suite::hook_binary::refresh(&root) {
                    Ok(Some(p)) => println!("hook binary: placed {} from {} (self-build stable copy, D0391)", p.copy.display(), p.from.display()),
                    Ok(None) => {}
                    Err(e) => println!("hook binary: could not place the self-build stable copy ({e})"),
                }
                if let Some(line) = keel_suite::hook_binary::describe(&root) {
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


/// `indicators [--trend] [--root ROOT]` — monitored measures (D0089) with direction-aware status.
/// Computed indicators show current value (full series with `--trend`); pulled/manual show their
/// recorded-Measurement series + status.
pub fn cmd_indicators(args: &[String]) -> i32 {
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
    if let Err(code) = crate::workspace::require_project(&root, "keel indicators [ROOT] [--trend]") {
        return code;
    }
    let trend = args.iter().any(|a| a == "--trend");
    match keel_view::view::indicators(&root, trend) {
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


/// The `.engine/contracts/` files that are THIS project's instance data rather than engine definition
/// (issue243). `keel init` must reset these, not copy them: an adoption declaration and a set of
/// exchange identities belong to the project that made them.
pub fn is_instance_contract(rel: &Path) -> bool {
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
pub fn starter_for(rel: &Path) -> &'static str {
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


/// Write one embedded engine file into `dst_engine`, remapping `decisions/*` -> `reference/decisions/*`
/// (read-only reference, NOT instance — the engine's architecture decisions must not enter the new
/// project's computed views, which scan `.engine/decisions`; D0093 engine/instance boundary).
// The scaffold path rules live in `migrate` — `keel migrate` resyncs a downstream `.engine/` from
// this same embedded tree, so it must map and exclude paths identically to `keel init` or a migrated
// project would differ from a freshly inited one. One definition, both callers.
pub fn write_engine_file(f: &include_dir::File, dst_engine: &Path, count: &mut u32) -> std::io::Result<()> {
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
        std::fs::write(&dst, keel_model::activation::without_grants(std::str::from_utf8(f.contents()).unwrap_or_default()))?;
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
pub fn scaffold_engine(dir: &include_dir::Dir, dst_engine: &Path, count: &mut u32) -> std::io::Result<()> {
    for f in dir.files() {
        write_engine_file(f, dst_engine, count)?;
    }
    for d in dir.dirs() {
        scaffold_engine(d, dst_engine, count)?;
    }
    Ok(())
}


/// Refuse an `init` target INSIDE an existing project, reporting why (issue275). `true` = refused.
///
/// `keel init sub` under a project used to succeed, and the resulting nested project was invisible to
/// workspace discovery, so it rode out UNGATED. Discovery now finds it, but the layout still leaves
/// overlapping paths with two claimants and is not what any author means. Peers, not tenants.
pub fn refuse_nested_target(dir: &Path) -> bool {
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
pub fn enclosing_project(dir: &Path) -> Option<PathBuf> {
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
    let start = crate::workspace::canon(&abs);
    let mut cur = start.parent();
    while let Some(p) = cur {
        if crate::workspace::is_project(p) {
            return Some(p.to_path_buf());
        }
        cur = p.parent();
    }
    None
}


/// Is a build commit one another machine can obtain - a clean SHA, not `+dirty` and not `unknown`?
/// (issue379 / GH#54: the wrapper cache may be seeded only from such a build.)
pub fn is_reproducible_build(commit: &str) -> bool {
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
pub fn init_wrapper(dir: &Path) -> Result<(), i32> {
    if let Err(e) = std::fs::write(dir.join("keelw"), include_str!("../../../keelw")) {
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


/// `keel init DIR` (D0093) — scaffold a fresh project: the embedded engine (`.engine/`, with the
/// architecture decisions remapped to read-only `reference/`), `CLAUDE.md`, and a starter `.tracking/`.
/// Self-contained cold start; refuses to overwrite an existing `.engine/`.
pub fn cmd_init(args: &[String]) -> i32 {
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
    let repo_root = keel_git::gitx::git()
        .arg("-C")
        .arg(&dir)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()))
        .filter(|p| p.is_dir())
        .map_or_else(|| dir.clone(), |p| crate::workspace::canon(&p));
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
pub fn cmd_status(rest: &[String]) -> i32 {
    refuse_flag_as_path(rest.first(), "status").unwrap_or_else(|| {
        resolve_guard_root(rest.first()).map_or_else(
            || {
                eprintln!("error: no .engine/ directory found. usage: keel show status [ROOT]");
                2
            },
            |root| crate::status::cmd(&root),
        )
    })
}


pub fn cmd_migrate(args: &[String]) -> i32 {
    let dry_run = args.iter().any(|a| a == "--dry-run");
    // D0336: verified-or-reverted is the default; `--no-verify` writes an UNVERIFIED tree and says so.
    let verify = !args.iter().any(|a| a == "--no-verify");
    let root = match root_arg(args, "keel migrate [--dry-run] [--no-verify] [ROOT]", &["dry-run", "no-verify"], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    crate::migrate::cmd_with(&root, &ENGINE_DIR, dry_run, verify)
}


/// `keel enroll` — the trailing ROOT is only honoured when it actually looks like a keel project,
/// so a stray argument cannot silently redirect an enrollment into the wrong tree.
pub fn cmd_enroll(rest: &[String]) -> i32 {
    let root = rest
        .iter()
        .rev()
        .find(|a| !a.starts_with("--"))
        .filter(|a| Path::new(a.as_str()).join(".tracking").is_dir())
        .map_or_else(|| find_repo_root().unwrap_or_else(|| PathBuf::from(".")), PathBuf::from);
    crate::enroll::cmd(rest, &root)
}


/// `keel knowledge question-coverage [ROOT]` (D0161): per declared Question, does seeding find an
/// entity and traversal reach an answer. Computed, never authored.
pub fn cmd_knowledge(rest: &[String]) -> i32 {
    if rest.first().map(String::as_str) != Some("question-coverage") {
        eprintln!("usage: keel knowledge question-coverage [ROOT]");
        return 2;
    }
    cmd_view0(rest.get(1..).unwrap_or(&[]), "knowledge question-coverage", keel_view::view::question_coverage)
}


pub fn cmd_view0(
    rest: &[String],
    name: &'static str,
    f: fn(&Path) -> Result<String, keel_view::view::ViewError>,
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
    if let Err(code) = crate::workspace::require_project(&root, &format!("keel {name} [ROOT]")) {
        return code;
    }
    println!("{}", f(&root).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")));
    0
}


/// `keel show flow [ROOT] [--json]` - the git-derived flow series (dcCycleTimeReadsFromGit). The lens
/// answers in JSON like every lens; `--json` is accepted as the explicit spelling the Definition of Done names.
pub fn cmd_flow(args: &[String]) -> i32 {
    let rest: Vec<String> = args.iter().filter(|a| *a != "--json").cloned().collect();
    cmd_view0(&rest, "show flow", keel_view::view::flow::flow)
}
