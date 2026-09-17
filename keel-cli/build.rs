//! Bake the BUILD PROVENANCE into the binary so `keel version` can answer the one question a
//! downstream project cannot otherwise answer: *which build am I actually running?*
//!
//! The version string alone is insufficient — it changes only at release, so a stale local build and
//! the published artifact both claim the same number. Two downstream blockers (issue089/issue090) were
//! each reported as "still broken" states where the FIRST diagnostic step is establishing which binary
//! is in play, and neither the reporter nor the maintainer could establish it. The commit is what makes
//! the claim checkable against the release record (`.tracking/baselines.sysml`).
//!
//! Honest degradation, never fabrication: with no `git` and no `.git/` (a crates.io/tarball build) the
//! commit is reported as `unknown` rather than guessed, and a build from a DIRTY tree is marked
//! `+dirty` — a modified working tree is exactly the case where the SHA would otherwise overstate what
//! the binary contains.

use std::path::Path;
use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?.trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn main() {
    let commit = git(&["rev-parse", "--short", "HEAD"]).map_or_else(
        || "unknown".to_string(),
        |sha| {
            // `--porcelain` is empty iff the tree is clean. A failed status call must not silently
            // produce a clean-looking claim, so treat only a definite empty result as clean.
            match git(&["status", "--porcelain"]) {
                Some(_) => format!("{sha}+dirty"), // non-empty output = uncommitted changes
                None => sha,                       // empty output (clean) or git unavailable
            }
        },
    );
    println!("cargo:rustc-env=KEEL_BUILD_COMMIT={commit}");

    // Rebuild when HEAD moves, so the baked commit cannot go stale. Only emit for paths that exist —
    // naming an absent path makes cargo rebuild on EVERY invocation. The repository root is found by
    // walking up from the package's own directory: this script is shared by keel-cli and by keel-write
    // (`build = "../../keel-cli/build.rs"`, sprint 733 - the Claude surface stamps the build it was
    // written by), and the two packages sit at different depths.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let mut dir = Some(Path::new(&manifest_dir));
    while let Some(d) = dir {
        let git_dir = d.join(".git");
        if git_dir.is_dir() {
            for p in ["HEAD", "index"] {
                let p = git_dir.join(p);
                if p.exists() {
                    println!("cargo:rerun-if-changed={}", p.display());
                }
            }
            break;
        }
        dir = d.parent();
    }
}
