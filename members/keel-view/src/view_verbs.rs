//! The `keel` verbs this member owns (D0479, sprint 750): `cmd_attestation_coverage`, `cmd_commit_delta`, `cmd_priority`, `cmd_sitting_coverage`, `cmd_decision_follow_through`, `cmd_enforcement_report`, `cmd_concern_coverage`, `cmd_rules`, `cmd_launchables`, `cmd_business`, `cmd_coverage`, `cmd_decisions`, `cmd_critique_coverage`, `cmd_critique_policy`, `cmd_governing_version`, `cmd_reprocess_candidates`, `cmd_suspect`, `cmd_view`, `cmd_snapshot_indicators`, `cmd_accept`, `cmd_reject`.
//!
//! Moved out of `keel-cli/src/main.rs` by `scripts/extract_verbs.py`: the binary is argument dispatch, and a
//! verb body lives with the member whose reach it needs (`python scripts/verb_homes.py --by-member`).
//! Each `cmd_*` takes the arguments after its verb and returns the process exit code.
//
// These were the binary's private items, written under the same lint set; only the lints that fire on
// PUBLIC visibility are new here, and they ask for ceremony a verb body does not owe: its exit code goes
// straight back to `main`, and its `Err` and panics are the ones the body already documents in place.
#![allow(clippy::must_use_candidate, clippy::missing_errors_doc, clippy::missing_panics_doc)]

use keel_args::{flag, positional_arg, prose_args, provenance_date, root_arg};
use keel_args::args_verbs::{fold_warnings_into_note, fold_words_into_note};
use keel_git::projects::find_repo_root;
use keel_model::model_verbs::{cmd_mint, decision_options_and_title};
use keel_write::ledger::ledger_refused;
use keel_write::write_verbs::{cmd_add_task, cmd_append_gate_result, cmd_append_result, cmd_apply_review, cmd_new, cmd_record_measurement, cmd_reverify};
use std::path::{Path, PathBuf};

pub fn cmd_attestation_coverage(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel attestation-coverage [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match crate::view::attestation_coverage(&root) {
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


/// `keel show priority [ROOT]` (D0311): the priority metric made visible.
// `keel show commit-delta [ROOT] [--range A..B]` - the model delta over a git range (dcCommitDeltaView, D0282).
pub fn cmd_commit_delta(args: &[String]) -> i32 {
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
    match crate::view::delta::commit_delta(&root, &range) {
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


pub fn cmd_priority(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel show priority [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match crate::priority::priority(&root) {
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


pub fn cmd_sitting_coverage(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel sitting-coverage [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match crate::view::sitting_coverage(&root) {
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


/// `keel decision-follow-through [ROOT] [--table]` (dcDecisionFollowThroughView/us020) — JSON is the
/// authority; `--table` renders the same data for eyes: one line per accepted Decision with
/// downstream work, then the gaps.
pub fn cmd_decision_follow_through(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel decision-follow-through [ROOT] [--table]", &["table"], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    let json = match crate::view::decision_follow_through(&root) {
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


/// `keel show enforcement-report [ROOT]` (D0180/K14) — fires, blocks, overrides, red-yields, and the
/// adherence trend, computed from the machine-local fire-ledger. Promotion decisions cite this.
pub fn cmd_enforcement_report(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel show enforcement-report [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match crate::pm::enforcement_report(&root) {
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


pub fn cmd_concern_coverage(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel concern-coverage [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match crate::view::concern_coverage(&root) {
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
pub fn cmd_rules(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel gate rules [ROOT] [--enforce]", &["enforce"], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match crate::view::check(&root) {
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
pub fn cmd_launchables(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel launchables [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match crate::view::launchables(&root) {
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
pub fn cmd_business(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel business [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match crate::view::business(&root) {
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


pub fn cmd_coverage(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel coverage [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match crate::view::coverage(&root) {
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


pub fn cmd_decisions(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel decisions [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match crate::view::decisions_report(&root) {
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


pub fn cmd_critique_coverage(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel critique-coverage [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match crate::view::critique_coverage(&root) {
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


pub fn cmd_critique_policy(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel critique-policy [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match crate::view::critique_policy(&root) {
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


pub fn cmd_governing_version(args: &[String]) -> i32 {
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
    if !keel_model::queries::is_declared(&root, item) {
        eprintln!("keel show governing-version: no item named `{item}` is declared in this model.");
        return 1;
    }
    println!("{}", crate::govern::governing_version(&root, item));
    0
}


pub fn cmd_reprocess_candidates(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel show reprocess-candidates [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    println!("{}", crate::govern::reprocess_candidates(&root));
    0
}


pub fn cmd_suspect(args: &[String]) -> i32 {
    let explain = args.iter().any(|a| a == "--explain");
    let root = match root_arg(args, "keel suspect [--explain] [ROOT]", &["explain"], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    println!("{}", crate::govern::suspect(&root, explain));
    0
}


pub fn cmd_view(args: &[String]) -> i32 {
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
    match crate::view::run(&root, name) {
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


/// The record-time acceptance under standing consent (D0291), for a NON-FORK with one declared
/// decider. issue376 / GH#57: the words quoted are the PROJECT's declared `standingWords`, never a
/// literal in the engine - with consent declared and no words the Decision stays proposed and says so.
#[allow(clippy::too_many_arguments)]
pub fn auto_accept_under_consent(root: &Path, path: &str, dname: &str, nnnn: &str, consent: &str, date: &str, author: &str) {
    let deciders: Vec<String> = keel_github::github::deciders(root).into_values().collect();
    let Some(words) = keel_model::activation::standing_words(root) else {
        println!("standing consent {consent} is declared but attestation-policy.toml records no standingWords - the human's consent is quoted from the project's own policy, never from the engine (issue376); D{nnnn} stays proposed. Record their words verbatim as standingWords beside standingConsent, or accept with their quoted word (D0289).");
        return;
    };
    if let [judge] = deciders.as_slice() {
        let sha = keel_git::gitx::git().arg("-C").arg(root).args(["rev-parse", "--short", "HEAD"]).output().ok()
            .filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
        let note = format!("AUTO-ACCEPTED under standing consent ({}). Their standing words, verbatim: '{words}' Not individually reviewed; override by a superseding Decision (D0290) or the human's quoted word in chat (D0289).", consent.to_uppercase());
        // Recorded by the actor running `record decision`, judged by the standing decider:
        // a DELEGATED record by construction, so the substance rule reads its quote.
        match keel_write::write::accept_decision(Path::new(path), dname, &sha, date, judge, author, &note) {
            Ok(_) => println!("accepted D{nnnn} at record time under standing consent {consent} (non-fork; judge {judge}; override by a superseding Decision or your quoted word)"),
            Err(e) => eprintln!("standing consent {consent} declared but the acceptance could not be recorded: {e} - D{nnnn} stays proposed"),
        }
    } else {
        println!("standing consent {consent} declared but github-actors.toml names {} decider(s), not one - D{nnnn} stays proposed; accept with your quoted word (D0289)", deciders.len());
    }
}


/// D0451: the authoring family's nine verbs are sub-verbs of `record`, each named for the fact it
/// writes and each keeping its flags. `sprint` keeps its own word: `cmd_new` reads it.
pub fn record_authoring_subverb(args: &[String]) -> Option<i32> {
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


/// `snapshot-indicators [--at DATE] [--by ACTOR] [--file F] [--root ROOT]` — take a reading of every
/// COMPUTED indicator (its current `metric_value`) and bank it as a `Measurement` (D0091). Run per
/// sprint/quarter to build a durable, fast series alongside the pulled/manual observations.
pub fn cmd_snapshot_indicators(args: &[String]) -> i32 {
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
    let by = match keel_actor::actor::resolve(&root, flag(args, "by").as_deref()) {
        Ok(a) => a,
        Err(msg) => { eprintln!("{msg}"); return 2; }
    };
    let keys = match crate::view::computed_indicator_keys(&root) {
        Ok(k) => k,
        Err(e) => {
            eprintln!("error: {e}");
            return 1;
        }
    };
    let mut count = 0u32;
    for (indicator, key) in &keys {
        let Some(v) = crate::view::metric_value(&root, key) else {
            eprintln!("skip {indicator}: metric '{key}' not computable");
            continue;
        };
        match keel_write::write::append_measurement(&file, indicator, &format!("{v:.6}"), &at, "snapshot (computed reading)", &by) {
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
pub fn tty_gesture() -> Option<&'static str> {
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
pub fn accept_channel_refusal(args: &[String], tty_gesture: Option<&str>) -> Result<Vec<String>, i32> {
    verdict_channel_refusal("accept", "accepting", args, tty_gesture, "decisionAcceptance", true)
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
pub fn verdict_channel_refusal(verb: &str, doing: &str, args: &[String], tty_gesture: Option<&str>, delegation_class: &str, read_back: bool) -> Result<Vec<String>, i32> {
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
            let delegation = keel_model::activation::recording_delegation(&root, delegation_class);
            // D0411 / issue426: the receipt this session can record is the human's QUOTED WORDS. A
            // gesture citation - console, deck, TTY, GitHub - is written by the surface that observed
            // the gesture (the console appends a device receipt; the terminal path above cites its own
            // TTY), never typed into a note; typed, it is the free text the substance rule used to
            // accept, and the agent-marked session with no terminal is exactly the writer that cannot
            // have observed any of them.
            let receipt = flag(args, "note").map_or(crate::view::NoteReceipt::Nothing, |n| crate::view::note_receipt(&n));
            match (delegation, receipt) {
                (Some(d), crate::view::NoteReceipt::QuotedWords) => {
                    // D0201 B, the chat half: READ-BACK RATIFICATION. The quoted words must name THIS
                    // decision (its id, one of its option letters, or three words of its title), or a
                    // bare 'yes' could be attached to any Decision the agent picks. Forward-only by
                    // construction: it binds new records, never re-reads old ones. The note has a
                    // quoted span here (the arm above), so the read-back reads the span, never a
                    // gesture word.
                    if let (true, Some(dec), Some(note)) = (read_back, args.first().filter(|a| !a.starts_with('-')), flag(args, "note")) {
                        let (letters, title) = decision_options_and_title(&root, dec);
                        // D0423: computed the same way, written into the record instead of refusing.
                        if !crate::view::read_back_names(&note, dec, &letters, &title) {
                            eprintln!("keel {verb}: WARN - the quoted words do not name {dec}, the decision they are {doing} (read-back, D0201 B: its id, an option letter, or three words of its title); recorded as given with the WARN in the note (D0423).");
                            warnings.push(format!("WARN: read-back - the quoted words do not name {dec} (D0201 B); recorded as given (D0423)."));
                        }
                        let short = crate::view::short_quoted_spans(&note);
                        if !short.is_empty() {
                            eprintln!("keel {verb}: WARN - the quoted words are shorter than ten characters (D0375); recorded as given with the WARN in the note (D0423).");
                            warnings.push("WARN: short words - fewer than ten characters quoted (D0375); recorded as given (D0423).".to_string());
                        }
                    }
                    eprintln!("keel {verb}: recording the human's verdict under delegation {d} - the note quotes their words (D0289).");
                }
                (Some(d), crate::view::NoteReceipt::GestureWordOnly) => {
                    eprintln!("keel {verb}: the note names a gesture (console, deck, TTY, GitHub) but no gesture reached this command - a gesture citation is written by the surface that observed it (the console appends a device receipt it can re-verify; a terminal cites its own TTY), never typed into a note (D0411/issue426). This session has no terminal and is not the console; under delegation {d} the receipt it can record is the human's words, verbatim: --words \"<what they said>\". Nothing written.");
                    ledger_refused(&root, verb, "gesture-word-typed");
                    return Err(1);
                }
                (Some(d), crate::view::NoteReceipt::Nothing) => {
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


pub fn cmd_accept(args: &[String]) -> i32 {
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
    let judged_by = match keel_actor::actor::resolve(&root, flag(args, "by").as_deref()) {
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
        match keel_actor::actor::resolve(&root, None) {
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
    for p in keel_model::corpus::collect_sysml(&root.join(".engine").join("decisions")) {
        if std::fs::read_to_string(&p).is_ok_and(|t| t.contains(&format!("part {decision} : Decision"))) {
            found = Some(p);
            break;
        }
    }
    let Some(path) = found else {
        eprintln!("error: no Decision '{decision}' under .engine/decisions/.");
        return 2;
    };
    let sha = keel_git::gitx::git()
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
        Some(gesture) => crate::view::note_with_tty_gesture(&note, gesture, &judged_by, &date),
        None => note,
    };
    // --rebind (D0308): the Decision is already accepted and its text moved since; record a new
    // acceptance result against the SHA whose text is current, with the note saying what changed.
    if args.iter().any(|a| a == "--rebind") {
        return match keel_write::write::rebind_acceptance(&path, decision, &sha, &date, &judged_by, &recorded_by, &note) {
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
    match keel_write::write::accept_decision(&path, decision, &sha, &date, &judged_by, &recorded_by, &note) {
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
pub fn cmd_reject(args: &[String]) -> i32 {
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
    let judged_by = match keel_actor::actor::resolve(&root, flag(args, "by").as_deref()) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("{msg}");
            return 2;
        }
    };
    let recorded_by = if flag(args, "by").is_some() {
        match keel_actor::actor::resolve(&root, None) {
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
    for p in keel_model::corpus::collect_sysml(&root.join(".engine").join("decisions")) {
        if std::fs::read_to_string(&p).is_ok_and(|t| t.contains(&format!("part {decision} : Decision"))) {
            found = Some(p);
            break;
        }
    }
    let Some(path) = found else {
        eprintln!("error: no Decision '{decision}' under .engine/decisions/.");
        return 2;
    };
    let sha = keel_git::gitx::git()
        .arg("-C")
        .arg(&root)
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_owned())
        .unwrap_or_default();
    let note = match tty_gesture {
        Some(gesture) => crate::view::note_with_tty_gesture(&note, gesture, &judged_by, &date),
        None => note,
    };
    match keel_write::write::reject_decision(&path, decision, &sha, &date, &judged_by, &recorded_by, &note) {
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
