//! The `keel` verbs this member owns (D0479, sprint 750): `cmd_spec_version`.
//!
//! Moved out of `keel-cli/src/main.rs` by `scripts/extract_verbs.py`: the binary is argument dispatch, and a
//! verb body lives with the member whose reach it needs (`python scripts/verb_homes.py --by-member`).
//! Each `cmd_*` takes the arguments after its verb and returns the process exit code.
//
// These were the binary's private items, written under the same lint set; only the lints that fire on
// PUBLIC visibility are new here, and they ask for ceremony a verb body does not owe: its exit code goes
// straight back to `main`, and its `Err` and panics are the ones the body already documents in place.
#![allow(clippy::must_use_candidate, clippy::missing_errors_doc, clippy::missing_panics_doc)]

pub fn cmd_spec_version(args: &[String]) -> i32 {
    use crate::spec_compat as sc;
    println!("grammar version (baked): {}", sc::SYSML_V2_GRAMMAR_VERSION);
    println!("pinned sha:              {}", sc::SYSML_V2_GRAMMAR_SHA);
    println!("spec url:                {}", sc::SYSML_V2_SPEC_URL);
    if sc::is_offline() || args.iter().any(|a| a == "--no-fetch") {
        println!("live check:              skipped (offline)");
        return 0;
    }
    let fetched = std::process::Command::new("curl")
        .args(["-sSL", sc::SYSML_V2_SPEC_URL])
        .output();
    let Ok(out) = fetched else {
        println!("live check:              unavailable (curl not found)");
        return 0;
    };
    if !out.status.success() || out.stdout.is_empty() {
        println!("live check:              unavailable (no network)");
        return 0;
    }
    let live = sc::sha256_hex(&out.stdout);
    println!("live sha:                {live}");
    let pinned = sc::SYSML_V2_GRAMMAR_SHA;
    if pinned.bytes().all(|b| b == b'0') {
        println!("status:                  not pinned — baked version is the reference; pin the live sha to enable drift detection");
        0
    } else if live == pinned {
        println!("status:                  CURRENT");
        0
    } else {
        println!("status:                  STALE — upstream changed since the pin");
        1
    }
}
