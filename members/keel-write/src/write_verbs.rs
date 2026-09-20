//! The `keel` verbs this member owns (D0479, sprint 750): `cmd_check_engine`, `cmd_reverify`, `cmd_new`, `cmd_append_result`, `cmd_append_gate_result`, `cmd_add_task`, `cmd_record_measurement`, `cmd_apply_review`.
//!
//! Moved out of `keel-cli/src/main.rs` by `scripts/extract_verbs.py`: the binary is argument dispatch, and a
//! verb body lives with the member whose reach it needs (`python scripts/verb_homes.py --by-member`).
//! Each `cmd_*` takes the arguments after its verb and returns the process exit code.
//
// These were the binary's private items, written under the same lint set; only the lints that fire on
// PUBLIC visibility are new here, and they ask for ceremony a verb body does not owe: its exit code goes
// straight back to `main`, and its `Err` and panics are the ones the body already documents in place.
#![allow(clippy::must_use_candidate, clippy::missing_errors_doc, clippy::missing_panics_doc)]

use crate::claude_surface::precommit_hook;
use crate::ledger::{ledger_gate, ledger_refused};
use keel_args::{flag, prose_flag, provenance_date, root_arg};
use keel_git::git_verbs::binding_note;
use keel_git::projects::find_repo_root;
use std::path::{Path, PathBuf};

/// `keel gate check-engine [ROOT]` (D0112 phase 2, issue067) — semantically validate the `.engine` INSTANCE
/// files (decisions/processes/views + registry + template) against the schema, KERNEL-FREE — the Rust
/// backstop for the `unresolved` reference class the JVM `validate_instances.py` used to be the sole
/// source of.
pub fn cmd_check_engine(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel gate check-engine [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    let diags = keel_model::validate::validate_engine_instances(&root);
    for (path, d) in &diags {
        println!("ERROR: {}:{} — {}", path.display(), d.line, d.message);
        if let Some(hint) = &d.suggestion {
            println!("       hint: {hint}");
        }
    }
    let failing: Vec<String> = if diags.is_empty() { Vec::new() } else { vec!["check-engine".to_string()] };
    ledger_gate(&root, "check-engine", &failing, 0);
    if diags.is_empty() {
        println!("{}", keel_json::color::pass(".engine instance files validated clean (kernel-free; D0112 phase 2)."));
        0
    } else {
        eprintln!("{}", keel_json::color::fail(&format!("{} .engine semantic diagnostic(s).", diags.len())));
        1
    }
}


/// `keel record reverify [--all-drift | --task NAME | --demos] [--by ACTOR] [ROOT]` (D0101) — re-run the configured
/// gate at HEAD and stamp a fresh `TestResult` on each drift-suspect task on green; `--demos` (D0444)
/// re-runs every replayable demo receipt instead and records each replay's own verdict.
pub fn cmd_reverify(args: &[String]) -> i32 {
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
    let by = match keel_actor::actor::resolve(&root, by.as_deref()) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("{msg}");
            return 2;
        }
    };
    if demos {
        return crate::reverify::replay_demos(&root, &by);
    }
    crate::reverify::run(&root, task.as_deref(), &by)
}


/// `keel record sprint <N> <slug> --charter <decision> [--points P]` (dcSprintScaffold/us019) — the
/// engine scaffolds the ceremony record: ids minted, provenance from the bound actor (refused when
/// absent), placeholders the fast gate rejects. See [`crate::scaffold`].
pub fn cmd_new(args: &[String]) -> i32 {
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
    let actor = match keel_actor::actor::resolve(&root, None) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("keel record sprint: {msg}");
            return 1;
        }
    };
    // issue267/D0301: `--fill FILE` writes the record's PROSE from a `--- key` draft (purpose, dod,
    // delivers, refine, standup, implement, review, closeOut, retro) - the sanctioned path for what a
    // scratchpad script used to emit, TestResult lines included. `delivers` names the backlog actions the
    // Story delivers, comma-separated, each authored as a #Delivers edge beside the charter edge (D0515).
    // The fill writes no result; append-result and append-gate-result are the only writers of verdicts.
    if let Some(fill_path) = flag(rest, "fill") {
        let fill = match decision_fields_from_file(&fill_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("keel record sprint: {e}");
                return 2;
            }
        };
        return match crate::scaffold::sprint_filled(&root, number, slug, &charter, points, &actor, &fill) {
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
    match crate::scaffold::sprint(&root, number, slug, &charter, points, &actor) {
        Ok(path) => {
            println!("scaffolded -> {}", path.display());
            println!("fill every {} before judging any gate - `keel gate --fast` rejects it until then", crate::scaffold::PLACEHOLDER);
            0
        }
        Err(e) => {
            eprintln!("keel record sprint: {e}");
            1
        }
    }
}


pub fn cmd_append_result(args: &[String]) -> i32 {
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
    let judged_by = match keel_actor::actor::resolve(&keel_actor::actor::root_for(&file), flag(args, "judged-by").as_deref()) {
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
    match crate::write::append_result(&file, &task, &sha, &verdict, &judged_at, &judged_by, evidence.as_deref()) {
        Ok(uuid) => { println!("{uuid}"); proposed_note(&file, &uuid); binding_note(&file, &sha, &verdict); 0 }
        Err(e @ crate::write::WriteError::ReceiptOwed(..)) => {
            // issue448/D0424: the refusal is a ledger fact - `append-result:ran-receipt` is the census row.
            ledger_refused(&keel_actor::actor::root_for(&file), "append-result", "ran-receipt");
            eprintln!("error: {e}");
            1
        }
        Err(e @ crate::write::WriteError::SchemaLacksMember(..)) => {
            // D0521/issue616: a proposed verdict the tree's schema cannot read is the same ledger fact.
            ledger_refused(&keel_actor::actor::root_for(&file), "append-result", "schema-member");
            eprintln!("error: {e}");
            1
        }
        Err(e) => { eprintln!("error: {e}"); 1 }
    }
}


/// Say when the result just written landed as a PROPOSAL (D0312 B): read the outcome back from the
/// line carrying `uuid`, never from the write's own report, so the note is the computed view. stderr,
/// so a caller parsing the uuid is unaffected.
pub fn proposed_note(file: &std::path::Path, uuid: &str) {
    let Ok(text) = std::fs::read_to_string(file) else { return };
    let recorded = text.lines().find(|l| l.contains(uuid)).is_some_and(|l| l.contains(&format!("VerdictKind::{}", crate::write::PROPOSED)));
    if recorded {
        eprintln!(
            "note: recorded {} - an AI-judged pass on an examined method (demo/analyze/inspect) with no replayable receipt is a proposal \
             until a human judges it, and counts as done for nothing meanwhile (D0312 B)",
            crate::write::PROPOSED
        );
    }
}


pub fn cmd_append_gate_result(args: &[String]) -> i32 {
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
    let judged_by = match keel_actor::actor::resolve(&keel_actor::actor::root_for(&file), flag(args, "judged-by").as_deref()) {
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
    match crate::write::append_gate_result(&file, &gate, &sha, &verdict, &judged_at, &judged_by, notes.as_deref(), evidence.as_deref()) {
        Ok(uuid) => { println!("{uuid}"); proposed_note(&file, &uuid); binding_note(&file, &sha, &verdict); 0 }
        Err(e @ crate::write::WriteError::ReceiptOwed(..)) => {
            // issue448/D0424: the refusal is a ledger fact - `append-gate-result:ran-receipt` is the census row.
            ledger_refused(&keel_actor::actor::root_for(&file), "append-gate-result", "ran-receipt");
            eprintln!("error: {e}");
            1
        }
        Err(e @ crate::write::WriteError::RetroScanMissing(..)) => {
            // issue566: the retro scan wording is refused at the write, by the Test - the same ledger fact.
            ledger_refused(&keel_actor::actor::root_for(&file), "append-gate-result", "retro-scan");
            eprintln!("error: {e}");
            1
        }
        Err(e @ crate::write::WriteError::SchemaLacksMember(..)) => {
            // D0521/issue616: a proposed gate verdict the tree's schema cannot read is the same ledger fact.
            ledger_refused(&keel_actor::actor::root_for(&file), "append-gate-result", "schema-member");
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
pub fn decision_fields_from_file(path: &str) -> Result<std::collections::BTreeMap<String, String>, String> {
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


pub fn cmd_add_task(args: &[String]) -> i32 {
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

    match crate::write::add_task(&file, &def_name, &task, &dod, &method) {
        Ok(uuid) => { println!("{uuid}"); 0 }
        Err(e) => { eprintln!("error: {e}"); 1 }
    }
}


/// `record-measurement --indicator I --value V [--at DATE] [--source S] [--by ACTOR] [--file F]` —
/// record a Measurement datapoint (D0089) for a pulled/manual indicator (write path).
pub fn cmd_record_measurement(args: &[String]) -> i32 {
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
    let by = match keel_actor::actor::resolve(&keel_actor::actor::root_for(&file), flag(args, "by").as_deref()) {
        Ok(a) => a,
        Err(msg) => { eprintln!("{msg}"); return 2; }
    };
    match crate::write::append_measurement(&file, &indicator, &value, &at, &source, &by) {
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


#[derive(serde::Deserialize)]
pub struct ReviewBatch {
    #[serde(default)]
    dispositions: Vec<ReviewDisp>,
    #[serde(default, rename = "judgedBy")]
    judged_by: String,
    #[serde(default, rename = "judgedAgainst")]
    judged_against: String,
}


#[derive(serde::Deserialize)]
pub struct ReviewDisp {
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
pub fn cmd_apply_review(args: &[String]) -> i32 {
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
    let Ok(critiques) = crate::write::per_actor_file(&root, "critiques", &judged_by) else {
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
            let disp = crate::write::Disposition { finding: &d.element, verdict, rationale: &d.rationale, sha: &sha, judged_at: &judged_at, judged_by: &judged_by };
            match crate::write::append_disposition(&critiques, &disp) {
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
        let c = crate::write::Critique {
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
        match crate::write::append_critique(&critiques, &c) {
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
pub fn install_commit_gate(repo_root: &Path, project: &Path) -> Result<(), i32> {
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
        let _ = keel_git::gitx::git()
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
pub fn init_enforcement_surface(dir: &Path, engine_dst: &Path, profile: &str) -> Result<(), i32> {
    let contracts = engine_dst.join("contracts");
    let _ = std::fs::create_dir_all(&contracts);
    let profile_fact = format!(
        "# Adoption profile — DECLARED at init, never inferred (D0174/P0.4).
         # strict: blocking in-loop gates from day one. guided: advisory-first; promote to blocking
         # with `keel sync-claude` after the D0180 evidence window, citing the fire-ledger.
         profile = \"{profile}\"
declaredAt = \"{}\"
",
        crate::scaffold::today()
    );
    if let Err(e) = std::fs::write(contracts.join("adoption-profile.toml"), profile_fact) {
        eprintln!("error writing adoption-profile.toml: {e}");
        return Err(1);
    }
    match crate::claude_surface::sync_claude(dir, false) {
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
    if let Err(e) = std::fs::write(wf.join("keel-gate.yml"), crate::claude_surface::CI_TEMPLATE) {
        eprintln!("error writing CI template: {e}");
        return Err(1);
    }
    // `core.hooksPath` is armed by `install_commit_gate` against the REPOSITORY root (issue278).
    // It used to be armed here against the project directory, which is the wrong repository whenever
    // the project is a workspace peer.
    Ok(())
}
