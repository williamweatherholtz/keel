//! The `keel` verbs this member owns (D0479, sprint 750): `cmd_serve`, `cmd_orient`, `cmd_github`, `cmd_deck`, `cmd_claude`, `cmd_record`, `cmd_render`, `cmd_report`, `cmd_currency`, `cmd_judge_set`, `cmd_decision_card`, `cmd_version`.
//!
//! Moved out of `keel-cli/src/main.rs` by `scripts/extract_verbs.py`: the binary is argument dispatch, and a
//! verb body lives with the member whose reach it needs (`python scripts/verb_homes.py --by-member`).
//! Each `cmd_*` takes the arguments after its verb and returns the process exit code.
//
// These were the binary's private items, written under the same lint set; only the lints that fire on
// PUBLIC visibility are new here, and they ask for ceremony a verb body does not owe: its exit code goes
// straight back to `main`, and its `Err` and panics are the ones the body already documents in place.
#![allow(clippy::must_use_candidate, clippy::missing_errors_doc, clippy::missing_panics_doc)]

use keel_args::{flag, prose_args, root_arg};
use keel_args::args_verbs::{fold_warnings_into_note, fold_words_into_note};
use keel_git::git_verbs::resolve_guard_root;
use keel_git::projects::{engine_version_skew, find_repo_root};
use keel_guards::guards_verbs::try_plan_cover;
use keel_issues::issues_verbs::{cmd_record_issue, cmd_record_statement, cmd_record_story};
use keel_view::view_verbs::{auto_accept_under_consent, record_authoring_subverb, tty_gesture, verdict_channel_refusal};
use keel_write::ledger::refused_injected_prose;
use keel_write::write_verbs::decision_fields_from_file;
use std::path::{Path, PathBuf};

pub fn cmd_serve(args: &[String]) -> i32 {
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
    if !keel_process::workspace::is_project(&root) {
        eprintln!("serve: {} is not a keel project (needs .engine/ and .tracking/).", root.display());
        return 2;
    }

    // `--forget`: stop listing this project in the console selector, without touching the project.
    if args.iter().any(|a| a == "--forget") {
        return match crate::console_registry::deregister(&root) {
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
    let today = keel_write::scaffold::today();
    if crate::console_registry::console_on(port) {
        if let Err(e) = crate::console_registry::register(&root, Some(port), &today) {
            eprintln!("console: registered nothing ({e}) — the console is running but this project");
            eprintln!("  will not appear in its selector until the registry is writable.");
            return 1;
        }
        println!("Keel console is ALREADY RUNNING on http://127.0.0.1:{port} — attached, did not start a second.");
        println!("  registered: {}", keel_process::workspace::canon(&root).display());
        println!("  open:       http://127.0.0.1:{port}/  then pick it from the project selector");
        println!("  forget it:  keel serve --forget {}", root.display());
        return 0;
    }
    // No console of ours. Register BEFORE binding, so the project is in the selector the moment the
    // surface comes up rather than one restart later.
    if let Err(e) = crate::console_registry::register(&root, Some(port), &today) {
        eprintln!("console: could not record this project in the registry: {e}");
        eprintln!("  starting anyway — the selector will show only the active project.");
    }
    crate::serve::run(root, port)
}


pub fn cmd_orient(args: &[String]) -> i32 {
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
        return match crate::reports::orient_html(&root) {
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
    if let Err(why) = keel_git::gitx::commit_gate_armed(&root) {
        if root.join(".githooks").join("pre-commit").exists() {
            eprintln!("[keel] WARNING: the commit gate is NOT ARMED — {why}. Fix: git config core.hooksPath .githooks (D0174/K2).");
        }
    }
    println!("{}", crate::reports::orient(&root).to_json());
    0
}


/// D0453: the five channel verbs route under `keel github`, each arm MOVED verbatim from the top-level
/// dispatch - its function and its trust classification untouched, its arguments the same. `None`
/// when the first argument is not one of the five, so the router can print its own usage.
pub fn github_subverb(args: &[String]) -> Option<i32> {
    let rest = args.get(1..).unwrap_or(&[]);
    Some(match args.first().map(String::as_str)? {
        "gesture" => keel_github::github::gesture_cmd(),
        "pull" => {
            let root = resolve_guard_root(
                rest.iter().position(|a| a == "--root").and_then(|i| rest.get(i + 1)),
            )
            .unwrap_or_else(|| std::path::PathBuf::from("."));
            keel_issues::github_ingest::pull_cmd(rest, &root)
        }
        "ingest" => {
            // ROOT is an explicit --root, never a trailing positional: the trailing argument here is
            // the value of --at, and guessing it as a path made the command fail with an opaque
            // filesystem error on its very first live run.
            let root = resolve_guard_root(
                rest.iter().position(|a| a == "--root").and_then(|i| rest.get(i + 1)),
            )
            .unwrap_or_else(|| std::path::PathBuf::from("."));
            keel_issues::github_ingest::cmd(rest, &root)
        }
        "decision-id" => keel_github::github::decision_id_cmd(rest),
        "decider" => keel_github::github::decider_cmd(rest, &find_repo_root().unwrap_or_else(|| PathBuf::from("."))),
        _ => return None,
    })
}


/// `keel github <sub-verb> ...` - the channel family under one router (D0453). Bare, or with a word
/// that is not a sub-verb, it prints the five and refuses: there is no bare meaning to default to.
pub fn cmd_github(args: &[String]) -> i32 {
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


pub fn cmd_deck(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel deck [ROOT] [--out FILE]", &["out"], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match crate::deck::html(&root) {
        Ok(h) => {
            if let Some(out) = flag(args, "out") {
                if let Err(e) = keel_write::write::write_atomic(std::path::Path::new(&out), &h) {
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


/// `keel sync-claude [ROOT] [--check]` (D0174/P0.2) — regenerate the keel-owned subset of the
/// `.claude/` surface in place (foreign entries survive), or with `--check` report drift and
/// version skew without writing. `--check` IS the `claude-surface-drift` guard's implementation.
/// `keel claude [claude args...]` - launch an interactive (or `-p`) Claude Code session with the keel
/// hooks pinned ON (D0296 layer 2): `--plugin-dir` at the plugin rendering and `--settings` with
/// `disableAllHooks: false` above project scope, `KEEL_BIN` pointing at this binary so every hook
/// resolves the engine that launched it. Everything after `claude` is passed through. Runs from
/// inside a keel project; the exit code is claude's own.
pub fn cmd_claude(args: &[String]) -> i32 {
    let Some(root) = find_repo_root() else {
        eprintln!("keel claude: not inside a keel project (no .engine/ up to the repository boundary) - run it from the project you want gated");
        return 2;
    };
    let pin = match crate::launcher::hook_pin_args(&root) {
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


/// D0337 (the human's answer to D0324, 2026-09-05: 'standing consent is scoped only to the existing
/// processes under which it was promulgated'): a Decision that CHANGES the process or enforcement
/// surface - a process-change or safety-change marker - is outside the ground the consent stands on and
/// stays proposed for the human. Consent covers work WITHIN the existing processes; it does not cover
/// rewriting them. Exit 0: the record itself succeeded.
pub fn outside_standing_consent(marker: &str) -> i32 {
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
pub fn consent_scope_gate(root: &Path, path: &Path, nnnn: &str, marker: Option<&str>, fields: &[&str]) -> Option<i32> {
    let consent = keel_model::activation::standing_consent(root);
    let Some(words) = crate::deck::marker_text_without_marker(fields, marker.is_some()) else {
        return marker.filter(|_| consent.is_some()).map(outside_standing_consent);
    };
    let list = words.join(", ");
    if consent.is_none() {
        println!(
            "note: the text names {list} and the draft carries no marker line; with no standing consent declared it is proposed either way, but add `marker: process-change` (or `safety-change`) if it changes the process so the process-change guard can see it, or state `{}: <why>` (issue460)",
            crate::deck::NOT_A_PROCESS_CHANGE
        );
        return None;
    }
    if let Err(e) = keel_write::write::note_marker_hold(path, &list) {
        eprintln!("the hold could not be written into D{nnnn}'s header: {e} - it is proposed regardless");
    }
    println!(
        "HELD proposed (D0337/issue460): the text names {list} and the draft carries no marker line, so standing consent did not accept it - a Decision that says it changes the process is outside the consent whether or not it says so with a marker. Add `marker: process-change` (or `safety-change`) and re-record so the process-change guard sees it, state `{}: <why it changes no process>` in the text, or a human accepts it with their quoted word (D0289).",
        crate::deck::NOT_A_PROCESS_CHANGE
    );
    Some(0)
}


/// The `keel record` usage, one line per fact the router writes.
pub fn print_record_usage() {
    eprintln!("usage: keel record decision --slug S --title T --context C --decision D --rationale R --consequences Q --date YYYY-MM-DD --author A [--root ROOT]");
    eprintln!("       keel record decision --from DRAFT.md   (prose in a file - the sanctioned path, issue255; flags override)");
    eprintln!("           [--supersedes dNNNN[,..]] retires each target whole | [--supersedes-clause dNNNN[,..]] reverses one clause, target stays in force (D0398); draft lines `supersedes:` / `supersedes-clause:`");
    eprintln!("       keel record issue --title T --description D --severity Critical|High|Medium|Low --resolver R --date YYYY-MM-DD [--related-task T] [--marker M] [--in-field] [--by A] [--root ROOT]");
    eprintln!("       keel record statement --text \"<their exact words>\" | --from FILE --said-by A --said-at D --title T [--channel C]   (VERBATIM, D0216/D0236)");
    eprintln!("       keel record story --from-statement stNNN --title T --as-a R --i-want C --implication K [--so-that O] [--triage-note W] --at D");
    eprintln!("       keel record task|sprint|result|gate-result|review|measurement|indicator-snapshot|reverify|mint ...   (D0451: the nine authoring verbs under one router, each named for the fact it writes; flags unchanged - `keel record <sub-verb>` alone prints its usage)");
}


pub fn cmd_record(args: &[String]) -> i32 {
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
    let author = match keel_actor::actor::resolve(&root, req("author").as_deref()) {
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
    let links = keel_write::write::DecisionLinks { supersedes: list("supersedes"), supersedes_clause: list("supersedes-clause"), derived_from: list("derived-from") };
    match keel_write::write::record_decision_with_links(&root, &slug, &title, &date, &author, &context, &decision, &rationale, &consequences, marker, research.as_deref(), &links) {
        Err(e @ keel_write::write::WriteError::InjectedToolOutput(..)) => refused_injected_prose(&root, &e),
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
            let disguised = keel_model::textscan::disguised_fork(&decision);
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
            match (keel_model::activation::standing_consent(&root), keel_model::textscan::fork_options(&root, &rel).is_empty(), disguised) {
                (Some(consent), true, Some(signals)) => {
                    println!(
                        "HELD as a fork in substance: the decision text weighs alternatives ({}) without the `OPTION X (label)` marker, so standing consent {consent} does not apply (D0322/issue373). Write it as a fork (OPTION A (label) ... COST ...; OPTION B ...), or state `{}: <why it chooses one course>` in the text and re-record; a human may still accept it with their quoted word (D0289).",
                        signals.join(", "),
                        keel_model::textscan::NOT_A_FORK
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


/// `render <view> [--mode graph|table|review] [--root ROOT]` — modular interactive-artifact
/// renderer over the view layer (D0086). Emits self-contained HTML to stdout (redirect to a file).
/// `keel render <what>` — one verb for everything that draws (D0449). The first positional resolves
/// in a FIXED order, held by `cli_surface::RENDER_RESERVED` and its test: the reserved words `model`
/// | `all` | `whole` (the whole-model graph, once `keel diagram`), `report <kind>` (once `keel report`),
/// `decision-card [NAME]` (once `keel decision-card`), then `control-structure`, then a declared
/// `.view.toml`. The order is fixed because a declared view named `report` shadowing the sub-verb — or
/// the reverse — is a silent wrong answer, the class sprint 512 met in the console's view binder.
pub fn cmd_render(args: &[String]) -> i32 {
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
    match keel_view::view::render_html(&root, view, &mode) {
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
pub fn cmd_report(args: &[String]) -> i32 {
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
    let result = if html { crate::reports::report_html(&root, name, trend) } else { crate::reports::report(&root, name, trend) };
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


pub fn cmd_currency(rest: &[String]) -> i32 {
    let root = rest.iter().find(|a| !a.starts_with("--") && Path::new(a.as_str()).join(".tracking").is_dir()).map_or_else(|| find_repo_root().unwrap_or_else(|| PathBuf::from(".")), PathBuf::from);
    keel_process::currency::cmd(rest, &root, &keel_issues::github_ingest::pull_cmd)
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
pub fn judge_set_items(root: &Path, path: &Path, rel: &str, all: bool, verdict: &str, fail: Option<&str>) -> Result<(Vec<keel_write::write::SetJudgment>, usize), i32> {
    let fails: Vec<String> = fail.map(|f| f.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()).unwrap_or_default();
    let proposals = crate::attestation::proposals_in(root, path);
    let rule = crate::attestation::sampling_rule(root);
    let pending: Vec<&crate::attestation::Proposal> = if all {
        proposals.iter().filter(|p| !p.judged).collect()
    } else {
        crate::attestation::sample(&proposals, rule).into_iter().filter(|p| !p.judged).collect()
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
        .map(|p| keel_write::write::SetJudgment {
            test: p.test.clone(),
            verdict: if fails.contains(&p.test) { "fail".to_string() } else { verdict.to_string() },
        })
        .collect();
    Ok((items, proposals.len()))
}


pub fn cmd_judge_set(args: &[String]) -> i32 {
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
        Some(gesture) => keel_view::view::note_with_tty_gesture(&note, gesture, &judged_by, &date),
        None => note,
    };
    match keel_write::write::judge_set(&path, &items, &sha, &date, &judged_by, &recorded_by, &note) {
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


/// A read-only view subcommand: run `f` against the repo root and print its JSON, or the error as
/// JSON so a consumer parsing stdout gets a parseable answer either way.
/// `keel render decision-card [NAME] [--proposed]` (D0205; under `render` since D0449): machine-readable deciding context.
pub fn cmd_decision_card(rest: &[String]) -> i32 {
    let name = rest.first().filter(|a| !a.starts_with('-')).map(String::as_str);
    let proposed = rest.iter().any(|a| a == "--proposed");
    let root = find_repo_root().unwrap_or_else(|| PathBuf::from("."));
    match crate::deck::decision_cards(&root, name, proposed) {
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


pub fn cmd_version(args: &[String]) -> i32 {
    let hard = keel_guards::GUARD_NAMES.len() - WARNING_ONLY_GUARDS.len();
    if args.iter().any(|a| a == "--json") {
        println!(
            "{{\"version\":\"{}\",\"buildCommit\":\"{}\",\"guards\":{},\"guardsHardBlocking\":{},\"guardsWarningOnly\":{}}}",
            env!("CARGO_PKG_VERSION"),
            env!("KEEL_BUILD_COMMIT"),
            keel_guards::GUARD_NAMES.len(),
            hard,
            WARNING_ONLY_GUARDS.len(),
        );
        return 0;
    }
    println!("keel {}", env!("CARGO_PKG_VERSION"));
    println!("build commit: {}", env!("KEEL_BUILD_COMMIT"));
    // D0190: the binary version is the ONE declared semver; the others are derived facts reported
    // beside it (a breaking API change is recorded in the release Decision, not versioned apart).
    println!("api contract: {} (derived; breaking changes recorded in release Decisions)", crate::serve::KEEL_API_VERSION);
    println!("claude surface: {} (generated from this binary)", keel_write::claude_surface::SURFACE_VERSION);
    match find_repo_root().map(|r| engine_version_skew(&r)) {
        Some(Some(w)) => println!("engine declared: SKEW - {w}"),
        Some(None) => println!("engine declared: matches (engine-version.toml or pre-D0190 absent)"),
        None => {}
    }
    println!(
        "guards: {} ({hard} hard-blocking, {} warning-only)",
        keel_guards::GUARD_NAMES.len(),
        WARNING_ONLY_GUARDS.len(),
    );
    0
}


/// The warning-only members of `GUARD_NAMES` — they RUN on every commit and are visible, but never
/// block (the D0102 promote-once-low-noise pattern). Named here so `keel version` can report the
/// hard-vs-warning split without a hand-maintained count.
pub const WARNING_ONLY_GUARDS: [&str; 9] =
    ["decision-requirement-link", "verification-trace", "priority-inversion", "retro-backlog", "doc-sync", "hook-config-integrity", "sequence-multiplicity", "parser-coverage", "base-first-justification"];
