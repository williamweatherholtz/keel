//! Test fixtures shared across the crate boundary (a `#[cfg(test)]` module is not reachable from
//! another crate). Nothing here runs outside a test.

/// The repository root: the first ancestor of this crate's manifest directory that holds `.git`.
///
/// A unit test that keys on `..` or `src/...` names the repository only while its crate sits directly
/// under it; the moment the module moves into `members/` (two levels down) the anchor points at
/// `members/` and the test breaks. Sprint 714 fixed four such tests by hand, sprint 718 eight more, and
/// nine files then carried their own copy of this walk (issue557). One helper, one walk, and the
/// touched.rs scan `no_member_test_anchors_on_a_cwd_relative_path` refuses the anchor class over every
/// crate's source - so the next D0479 move inherits no path fix.
///
/// Every member sits inside the one checkout, so the walk from THIS crate's manifest lands on the same
/// root a walk from the caller's would.
///
/// # Panics
/// Test fixture: panics when no ancestor holds `.git` - the crate is not inside a git checkout.
#[doc(hidden)]
#[must_use]
// A fixture panics where a test would: the same allowance lib.rs grants under cfg(test).
#[allow(clippy::expect_used)]
pub fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|a| a.join(".git").exists())
        .expect("keel-fs sits inside the keel repository")
        .to_path_buf()
}

#[cfg(test)]
mod tests {
    #[test]
    fn repo_root_holds_the_workspace_manifest_and_the_engine() {
        let root = super::repo_root();
        assert!(root.join(".git").exists(), "the root holds .git: {}", root.display());
        assert!(root.join("Cargo.toml").exists(), "the root holds the workspace manifest");
        assert!(root.join(".engine").is_dir(), "the root holds the engine");
        assert_eq!(
            root.join("members").join("keel-fs"),
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")),
            "this crate sits two levels under the root it names"
        );
    }
}
