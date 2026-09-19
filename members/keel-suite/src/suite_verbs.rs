//! The `keel` verbs this member owns (D0479, sprint 750): `cmd_suite`, `cmd_verify`.
//!
//! Moved out of `keel-cli/src/main.rs` by `scripts/extract_verbs.py`: the binary is argument dispatch, and a
//! verb body lives with the member whose reach it needs (`python scripts/verb_homes.py --by-member`).
//! Each `cmd_*` takes the arguments after its verb and returns the process exit code.
//
// These were the binary's private items, written under the same lint set; only the lints that fire on
// PUBLIC visibility are new here, and they ask for ceremony a verb body does not owe: its exit code goes
// straight back to `main`, and its `Err` and panics are the ones the body already documents in place.
#![allow(clippy::must_use_candidate, clippy::missing_errors_doc, clippy::missing_panics_doc)]

use keel_args::repo_arg;

/// `keel suite [ROOT] [-- <cargo test args>]` (D0353): everything after `--` is cargo's, so the ROOT
/// is looked for only before it.
pub fn cmd_suite(rest: &[String]) -> i32 {
    let own: Vec<String> = rest.iter().take_while(|a| *a != "--").cloned().collect();
    crate::suite::cmd(rest, &repo_arg(&own))
}


pub fn cmd_verify(rest: &[String]) -> i32 {
    // The probe value is a command line, never a root: the root is found among what the ladder
    // does not consume.
    let own = crate::verify::own_args(rest);
    crate::verify::cmd(rest, &repo_arg(&own))
}
