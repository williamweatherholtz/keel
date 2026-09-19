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

use std::path::Path;

/// The scaffold's closing narration: what was written, and the two onboarding steps in order.
///
/// Lifted out of `cmd_init` when adding the D0225 step pushed it past the line limit - it is
/// narration, not logic, and it is the FIRST thing a newcomer reads.
/// Warn when the scaffold's own deepest path leaves too little room under the host's path limit.
///
/// issue313, found by federation reconnaissance: at a 150-character parent directory, the shipped
/// `.engine/reference/decisions/0137-a-release-is-verified-by-running-the-published-asset.sysml`
/// reaches 249 characters and `git add -A` FAILS — so the project cannot make its FIRST COMMIT, and
/// the failure surfaces much later as an opaque `unable to index file`. keel does not own the git
/// repository and cannot set `core.longpaths` for it, but it can refuse to let this be discovered
/// the hard way. The warning names the remedy; it does not block, because a project that never uses
/// git is unaffected and a lockout here would be worse than the defect.
pub fn warn_if_paths_are_near_the_limit(dir: &Path) {
    const LIMIT: usize = 260;
    const HEADROOM: usize = 20;
    if !cfg!(windows) {
        return;
    }
    let longest = crate::walk_longest(dir);
    if longest + HEADROOM < LIMIT {
        return;
    }
    println!();
    println!("  WARNING — this project's deepest file path is {longest} characters, and Windows");
    println!("  refuses paths at {LIMIT}. `git add` will FAIL here with `unable to index file`, so the");
    println!("  project cannot make its first commit (issue313). Either move it to a shorter parent");
    println!("  directory, or enable long paths before committing:");
    println!("      git config --global core.longpaths true");
}


pub fn print_init_next_steps(dir: &Path, count: u32, profile: &str) {
    println!("Scaffolded the engine into {} ({count} engine file(s)). Adoption profile: {profile} (declared).", dir.display());
    warn_if_paths_are_near_the_limit(dir);
    println!();
    println!("Next:");
    println!("  1. cd {}", dir.display());
    // issue278: this step used to read `git init && git config core.hooksPath .githooks`. Followed
    // literally from inside a workspace peer — which step 1 has just put the reader in — `git init`
    // creates a NESTED repository and destroys the workspace: the peer stops being part of the repo
    // whose hook and push cover it. `install_commit_gate` now arms the gate against the repository
    // root at init time, so the step is a check rather than an instruction to run blind.
    if std::path::Path::new(".git").exists() || dir.join(".git").exists() {
        println!("  2. The commit gate is already armed (see above). Confirm: git config core.hooksPath");
    } else {
        println!("  2. `git init` AT THE REPOSITORY ROOT — not inside this project if it is one of");
        println!("     several in a shared repo, where a nested repo would take it out of the workspace.");
        println!("     Then re-run `keel init` here, or arm it by hand: git config core.hooksPath .githooks");
    }
    println!("  3. Read CLAUDE.md — how to work here (text is truth; the AI drives the CLI, you supervise).");
    // D0225: the two are sequenced, not alternatives. `project-onboarding` decides WHICH disciplines
    // this project runs; `introduction` walks the author through running one. Naming only the second
    // is how a project ends up with 25 active processes nobody chose (D0054 - friction is the top risk).
    println!("  4. Run the `project-onboarding` skill — it asks what you are building and charters the");
    println!("     process set on that basis. `keel onboard` reports NOT CHARTERED until you do.");
    println!("  5. Then the `introduction` skill — capture your first need + run your first sprint.");
    println!("     Or: keel show orient .   (where things stand)");
    println!();
    println!("The pre-commit gate at the REPOSITORY ROOT runs `keel gate --workspace` — validate + guard +");
    println!("declared rules, for every project the commit touches (Rust-only, no kernel).");
    println!("Engine design rationale is read-only reference in .engine/reference/decisions/;");
    println!("your project authors its OWN decisions fresh in .engine/decisions/.");
}
