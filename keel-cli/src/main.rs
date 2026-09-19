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

use keel_cli::args::repo_arg;
use keel_cli::hooks::cmd_hook;
use keel_cli::projects::find_repo_root;
use std::process;
use std::path::PathBuf;
// Every verb body is its member's (D0479, sprint 750); lib.rs lists them and this file dispatches.
use keel_cli::verbs::{
    cmd_accept, cmd_activation, cmd_attestation_coverage, cmd_audit, cmd_business, cmd_claude,
    cmd_commit_delta, cmd_concern_coverage, cmd_coverage, cmd_critique_coverage,
    cmd_critique_policy, cmd_currency, cmd_decision_follow_through, cmd_decisions, cmd_deck,
    cmd_dispositions, cmd_enforcement_report, cmd_enroll, cmd_flow, cmd_gate, cmd_github,
    cmd_governing_version, cmd_hardening, cmd_indicators, cmd_init, cmd_intake, cmd_judge_set,
    cmd_knowledge, cmd_launchables, cmd_ls, cmd_migrate, cmd_open_issues, cmd_orient, cmd_orphans,
    cmd_override, cmd_priority, cmd_query0, cmd_query1, cmd_recall, cmd_record, cmd_reject,
    cmd_render, cmd_reprocess_candidates, cmd_serve, cmd_sitting_coverage, cmd_status, cmd_suite,
    cmd_suspect, cmd_sync_claude, cmd_verify, cmd_version, cmd_view, cmd_view0, cmd_whats_next,
    cmd_why, print_usage,
};

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
    use keel_cli::verbs::classify_guard_args;
    use keel_cli::migrate::remap_engine_content;
    use keel_cli::migrate::remap_engine_path;
    use std::path::Path;

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
        for w in keel_cli::verbs::WARNING_ONLY_GUARDS {
            assert!(
                keel_cli::guards::GUARD_NAMES.contains(&w),
                "warning-only guard `{w}` is not in GUARD_NAMES — the reported hard count would be wrong"
            );
        }
        assert!(
            keel_cli::verbs::WARNING_ONLY_GUARDS.len() < keel_cli::guards::GUARD_NAMES.len(),
            "warning list must be a strict subset — otherwise the hard count underflows"
        );
    }

    #[test]
    fn init_ships_downstream_claude_md_not_self_build() {
        // issue057 (field defect): `keel init` must ship a DOWNSTREAM "tracked by keel" CLAUDE.md,
        // NEVER the self-build's ("This repo is a work-tracking engine"). D0047 permanent control.
        assert!(keel_cli::verbs::CLAUDE_MD.contains("tracked by keel"), "init CLAUDE.md must frame the project as tracked BY keel");
        assert!(!keel_cli::verbs::CLAUDE_MD.contains("is a work-tracking engine"), "init must NOT ship the self-build CLAUDE.md");
        assert!(keel_cli::verbs::CLAUDE_MD.contains("Parsed:"), "downstream CLAUDE.md must carry the D0106 parse-first discipline");
    }

    /// issue379 / GH#54: only a build another machine can obtain may seed the wrapper cache.
    #[test]
    fn only_a_reproducible_build_may_seed_the_wrapper_cache() {
        assert!(keel_cli::verbs::is_reproducible_build("c4d5dbf"));
        assert!(!keel_cli::verbs::is_reproducible_build("4674965+dirty"), "a modified tree is nobody else's binary");
        assert!(!keel_cli::verbs::is_reproducible_build("unknown"), "off-git is not a version");
        assert!(!keel_cli::verbs::is_reproducible_build(""));
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
            assert!(keel_cli::verbs::is_instance_contract(Path::new(p)), "{p} is instance data and must be reset");
        }
        // Engine DEFINITION must still be copied verbatim.
        for p in ["contracts/process-enforcement.toml", "processes/intake.sysml", "rules/rules.sysml"] {
            assert!(!keel_cli::verbs::is_instance_contract(Path::new(p)), "{p} is engine definition and must be shipped as-is");
        }
        // The activation starter must leave the honest default IN FORCE, i.e. no live [processes]
        // section - a commented template is guidance, an uncommented one is a declaration.
        let starter = keel_cli::verbs::starter_for(Path::new("contracts/activation.toml"));
        for line in starter.lines() {
            let l = line.trim();
            assert!(
                l.is_empty() || l.starts_with('#'),
                "the activation starter must declare NOTHING; found a live line: {l}"
            );
        }
        // The id registries start genuinely empty: no `name = "uuid"` entry may be inherited.
        let ids = keel_cli::verbs::starter_for(Path::new("contracts/unit-ids.toml"));
        assert!(!ids.lines().any(|l| !l.trim_start().starts_with('#') && l.contains('=')),
            "an inherited unit id claims a lineage this project does not have");
        // D0219: the decider table must start EMPTY - an inherited decider could record
        // acceptances in another project's tree under someone else's name.
        let gh = keel_cli::verbs::starter_for(Path::new("contracts/github-actors.toml"));
        assert!(gh.contains("[logins]"), "the starter must still show the section shape");
        assert!(!gh.lines().any(|l| !l.trim_start().starts_with('#') && l.contains('=')),
            "no login may be inherited: who may decide is per-project");
    }
}
