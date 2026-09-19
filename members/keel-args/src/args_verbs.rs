//! The `keel` verbs this member owns (D0479, sprint 750): `cmd_query0`.
//!
//! Moved out of `keel-cli/src/main.rs` by `scripts/extract_verbs.py`: the binary is argument dispatch, and a
//! verb body lives with the member whose reach it needs (`python scripts/verb_homes.py --by-member`).
//! Each `cmd_*` takes the arguments after its verb and returns the process exit code.
//
// These were the binary's private items, written under the same lint set; only the lints that fire on
// PUBLIC visibility are new here, and they ask for ceremony a verb body does not owe: its exit code goes
// straight back to `main`, and its `Err` and panics are the ones the body already documents in place.
#![allow(clippy::must_use_candidate, clippy::missing_errors_doc, clippy::missing_panics_doc)]

use crate::{flag, root_arg};

// Root-only query: `keel <name> [ROOT]`.
pub fn cmd_query0(args: &[String], usage: &str, f: fn(&std::path::Path) -> String) -> i32 {
    let root = match root_arg(args, &format!("keel {usage} [ROOT]"), &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    println!("{}", f(&root));
    0
}


/// D0423: the checks that used to refuse a delegated record are written INTO it. Each WARN line names
/// the check, so a reader of the acceptance sees what kind of receipt it is; the write proceeds.
pub fn fold_warnings_into_note(args: &[String], warnings: &[String]) -> Vec<String> {
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


/// `--words TEXT` (D0375/issue397): the human's verbatim words as their OWN argument, folded into the
/// note inside a typographic quote pair so the boundary is declared, never inferred from an apostrophe
/// in the recorder's framing. Returns the args with `--words` gone and `--note` carrying the pair.
pub fn fold_words_into_note(args: &[String]) -> Vec<String> {
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
