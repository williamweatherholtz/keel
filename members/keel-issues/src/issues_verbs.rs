//! The `keel` verbs this member owns (D0479, sprint 750): `cmd_intake`, `cmd_open_issues`, `cmd_dispositions`, `cmd_record_statement`, `cmd_record_story`, `cmd_record_issue`.
//!
//! Moved out of `keel-cli/src/main.rs` by `scripts/extract_verbs.py`: the binary is argument dispatch, and a
//! verb body lives with the member whose reach it needs (`python scripts/verb_homes.py --by-member`).
//! Each `cmd_*` takes the arguments after its verb and returns the process exit code.
//
// These were the binary's private items, written under the same lint set; only the lints that fire on
// PUBLIC visibility are new here, and they ask for ceremony a verb body does not owe: its exit code goes
// straight back to `main`, and its `Err` and panics are the ones the body already documents in place.
#![allow(clippy::must_use_candidate, clippy::missing_errors_doc, clippy::missing_panics_doc)]

use keel_args::{flag, prose_args, prose_flag, root_arg};
use keel_git::projects::find_repo_root;
use keel_write::ledger::refused_injected_prose;
use std::path::PathBuf;

/// `keel intake [ROOT]` (D0166) — what was said, what it became, what nobody acted on.
pub fn cmd_intake(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel intake [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match crate::views::intake(&root) {
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


pub fn cmd_open_issues(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel open-issues [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match crate::views::open_issues(&root) {
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


pub fn cmd_dispositions(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel dispositions [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match crate::views::dispositions(&root) {
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


/// `keel record statement --text "<their exact words>" --said-by A --said-at D --channel C --title T`
///
/// `--text` is passed through VERBATIM (escaped for the literal, otherwise untouched), so it must be
/// their words and not a summary; `--title` is the AI's label and is sanitised like any other field.
/// `--from FILE` reads the text from a file, which is the sanctioned path for anything containing
/// quotes, newlines or backticks - the same reason `record decision --from` exists (D0224/issue255).
pub fn cmd_record_statement(args: &[String]) -> i32 {
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
            keel_schema::schema::enum_members_union(&root, "StatementChannel").join("|")
        );
        eprintln!("       [--by RECORDER] [--at YYYY-MM-DD] [--root ROOT]");
        eprintln!("  --text is VERBATIM (D0216): their words, not a summary. --title is your label for it.");
        return 2;
    };
    let channel = flag(args, "channel").unwrap_or_else(|| "chat".to_string());
    let author = match keel_actor::actor::resolve(&root, flag(args, "by").as_deref()) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("{msg}");
            return 2;
        }
    };
    // Provenance is never defaulted (D0129/issue182): the RECORD's date is its own fact, separate
    // from when they said it.
    let created_at = flag(args, "at").unwrap_or_else(|| said_at.clone());
    match crate::intake_write::record_statement(
        &root,
        &crate::intake_write::NewStatement {
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
pub fn cmd_record_story(args: &[String]) -> i32 {
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
            keel_schema::schema::enum_members_union(&root, "ImplicationKind").join("|")
        );
        eprintln!("       [--so-that-from FILE | --so-that OUTCOME] [--triage-note-from FILE | --triage-note WHY] [--by RECORDER] [--at YYYY-MM-DD] [--root ROOT]");
        eprintln!("  Prose goes through a FILE (D0224): a backtick in a double-quoted shell argument is a command the shell runs into the record.");
        eprintln!("  --from-statement is REQUIRED: a UserStory with no cited source is an invention (D0216).");
        return 2;
    };
    let author = match keel_actor::actor::resolve(&root, flag(args, "by").as_deref()) {
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
    match crate::intake_write::record_story(
        &root,
        &crate::intake_write::NewStory {
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


/// `keel record issue` — the sanctioned path for D0108 clause 5, which mandates that conflicting
/// conclusions be recorded as an Issue for human adjudication and had no implementation.
///
/// REFUSES on an unknown `--resolver` rather than authoring a dangling edge: the `issues` guard is
/// satisfied by the PRESENCE of a `#Resolves` edge, so an edge pointing at nothing would read as
/// triaged while resolving nothing — exactly the phantom issue109 found twice in this repo.
pub fn cmd_record_issue(args: &[String]) -> i32 {
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
    if let Err(e) = crate::issue_write::triage_holds(&root, &severity, &resolver) {
        eprintln!("error: {e}");
        return 2;
    }
    let Some(date) = flag(args, "date").filter(|d| !d.is_empty()) else {
        eprintln!("error: --date YYYY-MM-DD required (when it was found is its own irreducible fact)");
        return 2;
    };
    // NEVER default the actor (D0129/issue072): an unattributable fact looks like evidence.
    let author = match keel_actor::actor::resolve(&root, flag(args, "by").as_deref()) {
        Ok(a) => a,
        Err(msg) => { eprintln!("{msg}"); return 2; }
    };
    // Bound to locals so the borrows outlive the struct; the inline form silently collapsed both to
    // None (they type-checked and were wrong — a flag the caller passed would have been dropped).
    let related_task = flag(args, "related-task");
    let marker = flag(args, "marker");
    let n = crate::issue_write::NewIssue {
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
    match crate::issue_write::record_issue(&root, &n) {
        Ok((name, path)) => {
            println!("recorded {name} -> {path}");
            println!("  triaged on arrival: `#Resolves dependency from {resolver} to {name};`");
            println!("  run `keel gate validate . && keel gate guard .` to confirm; nothing was committed.");
            0
        }
        Err(e @ keel_write::write::WriteError::InjectedToolOutput(..)) => refused_injected_prose(&root, &e),
        Err(e) => { eprintln!("error: {e}"); 1 }
    }
}
