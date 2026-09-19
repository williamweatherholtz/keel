//! The `keel` verbs this member owns (D0479, sprint 750): helpers only.
//!
//! Moved out of `keel-cli/src/main.rs` by `scripts/extract_verbs.py`: the binary is argument dispatch, and a
//! verb body lives with the member whose reach it needs (`python scripts/verb_homes.py --by-member`).
//! Each `cmd_*` takes the arguments after its verb and returns the process exit code.
//
// These were the binary's private items, written under the same lint set; only the lints that fire on
// PUBLIC visibility are new here, and they ask for ceremony a verb body does not owe: its exit code goes
// straight back to `main`, and its `Err` and panics are the ones the body already documents in place.
#![allow(clippy::must_use_candidate, clippy::missing_errors_doc, clippy::missing_panics_doc)]

/// The subcommand catalogue, printed when no arm matches.
/// Rendered from the CLI FACTS (`cli_facts::CLI_FACTS`, the mirror of `.engine/cli/commands.sysml`,
/// D0271/issue344) so a synopsis has one home. The hand-written CATALOGUE it replaces had already gone
/// stale against D0273 - it listed `hardening`, `suspect` and `outstanding` as verbs a month after they
/// moved under `show` - which is the drift a table-plus-test cannot see and a fact-plus-guard can.
pub fn print_usage() -> i32 {
    eprint!("{}", crate::cli_facts::render_help());
    2
}
