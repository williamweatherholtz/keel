//! Test fixtures shared across the crate boundary (a `#[cfg(test)]` module is not reachable from
//! another crate). Nothing here runs outside a test.
//!
//! THE READ SET (D0481, sprint 741). A test that reaches this repository through [`repo_root`] or
//! [`repo_path`] RECORDS what it reached: when the runner has set [`READSET_DIR_VAR`], every call
//! appends the repo-relative path it returned - `.` for the root, `keel-cli/src/main.rs` for a file,
//! `.githooks` for a directory - to `<dir>/<binary>.reads`, the binary named by its own executable
//! with cargo's `-<hash>` suffix removed ([`binary_stem`]). The touched runner reads those files after
//! the run and keys the binary on the bytes under exactly those paths (`contentkey::reads_key`) rather
//! than on one hash of the whole tree; a binary that recorded nothing keeps D0474's tree key. An
//! observed set is a receipt where a declared one would drift: a test that starts reading a new path
//! records it on the run that reads it, and is keyed on it from then on. Recording `.` says "the
//! whole tree", and that key is the tree key - a root reader saves nothing and claims nothing.

/// The environment variable naming the directory read sets are appended under; unset = no recording.
pub const READSET_DIR_VAR: &str = "KEEL_READSET_DIR";

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
/// root a walk from the caller's would. Records `.` (D0481): the caller may read anything under it.
///
/// # Panics
/// Test fixture: panics when no ancestor holds `.git` - the crate is not inside a git checkout.
#[doc(hidden)]
#[must_use]
// A fixture panics where a test would: the same allowance lib.rs grants under cfg(test).
#[allow(clippy::expect_used)]
pub fn repo_root() -> std::path::PathBuf {
    record(".");
    root_unrecorded()
}

/// One path under the repository root, repo-relative with either separator, recorded as read (D0481):
/// `repo_path("keel-cli/src/main.rs")`, `repo_path(".githooks")`. A path that climbs out (`..`) is
/// recorded as `.` - the whole tree - because that is what it can reach.
///
/// # Panics
/// Test fixture: panics when no ancestor holds `.git`.
#[doc(hidden)]
#[must_use]
pub fn repo_path(rel: &str) -> std::path::PathBuf {
    let norm = rel.replace('\\', "/");
    let norm = norm.trim_matches('/');
    let entry = if norm.is_empty() || norm.split('/').any(|seg| seg == "..") { ".".to_owned() } else { norm.to_owned() };
    record(&entry);
    root_unrecorded().join(norm)
}

/// The verb sources - where a `cmd_*` body or a `*_subverb` router is defined - as
/// `(repo-relative path, text)`, each read recorded (D0481): `keel-cli/src/main.rs`,
/// `keel-parser/src/parser_verbs.rs`, and every `members/<m>/src/<short>_verbs.rs` the tree holds
/// (the D0479 layout `scripts/extract_verbs.py` writes, sprint 750).
///
/// A source-reading test that named `keel-cli/src/main.rs` by path checked a file, not its subject:
/// sprint 750 moved 126 bodies out of it and three such tests went red while a fourth passed over an
/// empty scan. A test asks this list for the source that holds its subject ([`source_holding`]) and
/// follows the next move for free. The set is listed from the tree, not re-parsed from `Cargo.toml`: a
/// member whose verbs file exists is in it whether or not a manifest names it.
///
/// # Panics
/// Test fixture: panics when `keel-cli/src/main.rs` or `members/` cannot be read.
#[doc(hidden)]
#[must_use]
#[allow(clippy::expect_used)]
pub fn verb_sources() -> Vec<(String, String)> {
    let mut rels = vec!["keel-cli/src/main.rs".to_owned(), "keel-parser/src/parser_verbs.rs".to_owned()];
    let members = std::fs::read_dir(root_unrecorded().join("members")).expect("members/ is listable");
    for entry in members.flatten() {
        let Ok(name) = entry.file_name().into_string() else { continue };
        let Ok(src) = std::fs::read_dir(entry.path().join("src")) else { continue };
        for file in src.flatten() {
            let Ok(file_name) = file.file_name().into_string() else { continue };
            if file_name.ends_with("_verbs.rs") {
                rels.push(format!("members/{name}/src/{file_name}"));
            }
        }
    }
    rels.sort();
    let mut out = Vec::new();
    for rel in rels {
        let path = repo_path(&rel);
        if let Ok(text) = std::fs::read_to_string(&path) {
            out.push((rel, text));
        }
    }
    assert!(out.iter().any(|(p, _)| p == "keel-cli/src/main.rs"), "keel-cli/src/main.rs is a verb source");
    out
}

/// The one verb source whose text contains `needle` (a `fn name(` head, a string literal), as
/// `(repo-relative path, text)`; panics naming the needle when none does, so a test that lost its
/// subject fails saying what it looked for rather than `the router exists`.
///
/// # Panics
/// Test fixture: panics when no verb source contains `needle`.
#[doc(hidden)]
#[must_use]
pub fn source_holding<'a>(sources: &'a [(String, String)], needle: &str) -> &'a (String, String) {
    sources
        .iter()
        .find(|(_, text)| text.contains(needle))
        .unwrap_or_else(|| panic_no_source(needle))
}

#[allow(clippy::panic)]
fn panic_no_source(needle: &str) -> ! {
    panic!("no verb source holds `{needle}`")
}

#[allow(clippy::expect_used)]
fn root_unrecorded() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|a| a.join(".git").exists())
        .expect("keel-fs sits inside the keel repository")
        .to_path_buf()
}

/// The binary a test executable's file stem names.
///
/// Cargo appends `-<16 hex>` to every test target (`orient_bdd-1a2b3c4d5e6f7a8b` -> `orient_bdd`,
/// `keel_fs-0123456789abcdef` -> `keel_fs`); a stem with no such suffix is itself. Pure.
#[must_use]
pub fn binary_stem(file_stem: &str) -> String {
    match file_stem.rsplit_once('-') {
        Some((name, hash)) if hash.len() == 16 && hash.chars().all(|c| c.is_ascii_hexdigit()) => name.to_owned(),
        _ => file_stem.to_owned(),
    }
}

/// Append `entry` to this binary's read-set file when the runner asked for one. Silent otherwise, and
/// silent on any failure: a recording that cannot be written is a binary with no recorded set, which
/// the runner keys on the whole tree - never a false narrowing.
fn record(entry: &str) {
    use std::io::Write as _;
    let Ok(dir) = std::env::var(READSET_DIR_VAR) else { return };
    if dir.is_empty() {
        return;
    }
    let Some(stem) = std::env::current_exe().ok().and_then(|p| p.file_stem().map(|s| binary_stem(&s.to_string_lossy()))) else { return };
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(std::path::Path::new(&dir).join(format!("{stem}.reads"))) {
        let _ = writeln!(f, "{entry}");
    }
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
        assert_eq!(super::repo_path("members\\keel-fs/"), root.join("members/keel-fs"), "either separator, the trailing slash dropped");
    }

    /// D0481 pair, chosen before the code was read. Known-positive: a cargo test stem loses its hash
    /// and nothing else. Known-negative: a stem whose last dash-part is not sixteen hex digits is kept
    /// whole - `keel-fs` is a crate name, not a hash.
    #[test]
    fn a_test_binary_is_named_without_cargos_hash() {
        assert_eq!(super::binary_stem("orient_bdd-1a2b3c4d5e6f7a8b"), "orient_bdd");
        assert_eq!(super::binary_stem("keel_fs-0123456789abcdef"), "keel_fs");
        assert_eq!(super::binary_stem("hook_and_push_controls_actually_fire-ffffffffffffffff"), "hook_and_push_controls_actually_fire");
        assert_eq!(super::binary_stem("keel-fs"), "keel-fs", "a crate name is not a hash");
        assert_eq!(super::binary_stem("x-1a2b"), "x-1a2b", "too short to be cargo's hash");
        assert_eq!(super::binary_stem("plain"), "plain");
    }

    /// D0481: with the variable set, a root read records `.` and a path read records the path (an
    /// escaping path records `.`); with it unset nothing is written. The directory is this process's
    /// own scratch, so two test processes never share a file.
    #[test]
    fn reads_are_recorded_under_the_runners_directory_and_nowhere_else() {
        let dir = std::env::temp_dir().join(format!("keel-readset-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch");
        // Known-negative first: unset (or empty) records nothing.
        std::env::set_var(super::READSET_DIR_VAR, "");
        let _ = super::repo_root();
        assert!(std::fs::read_dir(&dir).expect("dir").next().is_none(), "an empty variable records nothing");
        std::env::set_var(super::READSET_DIR_VAR, &dir);
        let _ = super::repo_root();
        let _ = super::repo_path(".githooks/");
        let _ = super::repo_path("keel-cli/../.tracking");
        std::env::remove_var(super::READSET_DIR_VAR);
        let _ = super::repo_path("not-recorded");
        let files: Vec<_> = std::fs::read_dir(&dir).expect("dir").flatten().map(|e| e.file_name().to_string_lossy().to_string()).collect();
        assert_eq!(files.len(), 1, "one file per binary: {files:?}");
        let expected = format!("{}.reads", super::binary_stem(&std::env::current_exe().expect("exe").file_stem().expect("stem").to_string_lossy()));
        assert_eq!(files[0], expected, "named by this binary without cargo's hash");
        assert!(!expected.contains('-') || expected.starts_with("keel"), "the hash is gone: {expected}");
        let text = std::fs::read_to_string(dir.join(&files[0])).expect("reads");
        assert_eq!(text, ".\n.githooks\n.\n", "root, the path, the escape as the root; nothing after the variable was removed:\n{text}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
