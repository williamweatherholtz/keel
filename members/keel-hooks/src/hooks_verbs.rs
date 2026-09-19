//! The `keel` verbs this member owns (D0479, sprint 750): `cmd_override`, `cmd_recall`, `cmd_why`.
//!
//! Moved out of `keel-cli/src/main.rs` by `scripts/extract_verbs.py`: the binary is argument dispatch, and a
//! verb body lives with the member whose reach it needs (`python scripts/verb_homes.py --by-member`).
//! Each `cmd_*` takes the arguments after its verb and returns the process exit code.
//
// These were the binary's private items, written under the same lint set; only the lints that fire on
// PUBLIC visibility are new here, and they ask for ceremony a verb body does not owe: its exit code goes
// straight back to `main`, and its `Err` and panics are the ones the body already documents in place.
#![allow(clippy::must_use_candidate, clippy::missing_errors_doc, clippy::missing_panics_doc)]

use crate::{OVERRIDE_TTL_SECS, RECALL_BUDGET, override_path, override_target};
use keel_args::{flag, positional_arg, prose_args, root_arg, without_flag_values};
use keel_git::projects::find_repo_root;
use std::path::PathBuf;

/// `keel override <path> --reason "<text>"` (D0176 tier 3) — the sanctioned unlock for a direct
/// write the API cannot express. Single-use, target-path-bound, expiring; consumption records an
/// orient-visible obligation naming the path actually written (K7). Never a silent env var.
pub fn cmd_override(args: &[String]) -> i32 {
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
    let actor = match keel_actor::actor::resolve(&root, None) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("keel override: {msg}");
            return 1;
        }
    };
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let _ = std::fs::create_dir_all(root.join(".keel"));
    let unlock = serde_json::json!({"path": target, "reason": reason, "actor": actor, "ts": ts});
    if let Err(e) = keel_write::write::write_atomic(&override_path(&root), unlock.to_string()) {
        eprintln!("keel override: cannot write the unlock: {e}");
        return 1;
    }
    println!("override armed for `{target}` - SINGLE USE, expires in {OVERRIDE_TTL_SECS}s; consumption records an obligation (D0176/K7).");
    0
}


pub fn budget_arg(args: &[String]) -> usize {
    flag(args, "budget").and_then(|v| v.parse().ok()).unwrap_or(RECALL_BUDGET)
}


/// `keel recall --prompt - [--budget N] [ROOT]` — seed from a PROMPT and print a budgeted brief.
///
/// The prompt arrives on STDIN, never as an argument: it is free-form text that may contain quotes,
/// newlines and backticks, and the one thing this path must never do is let that text reach a shell.
pub fn cmd_recall(rest: &[String]) -> i32 {
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
    match keel_view::view::recall_for_prompt(&root, &prompt, budget_arg(rest)) {
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
pub fn cmd_why(rest: &[String]) -> i32 {
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
        return match keel_view::view::why_brief(&root, term, budget_arg(rest)) {
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
    match keel_view::view::why(&root, term) {
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
