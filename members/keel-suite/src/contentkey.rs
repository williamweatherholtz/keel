//! CONTENT KEYS (D0474): what a test binary's outcome depends on, named by the bytes themselves.
//!
//! WHY BYTES. The guard receipt keyed every path on `(len, mtime)` and needed a two-second RACY window
//! to trust a stamp; the window raced the wall clock twice under load (issue481, issue534). Cargo
//! memoises compilation by content fingerprint and, on the third run of one fix, rebuilt in 1.08 s -
//! then ran 736 s of tests nothing had a memo for. A digest of the bytes is settled the moment it is
//! computed: equal bytes are equal inputs, whatever the clock said. The whole tracked corpus is 30 MB
//! in 1893 files and hashes in about 0.2 s warm on this host (walkcost.py, 2026-09-14).
//!
//! TWO KEYS, because the ceremony writes to `.tracking/` between the touched run and the land, and a
//! single whole-tree key would rerun every binary for a write no binary reads. `code` covers the paths
//! compiled into or run by the binaries: `keel-cli/**`, `.engine/**` (embedded, `embedded::ENGINE_DIR`),
//! `Cargo.toml`, `Cargo.lock`, `.githooks/**`, `.claude/**` (`OUTPUT_STYLE` is an `include_str!` of
//! `.claude/output-styles/keel.md`) and `keelw` (written out by `keel init`). `tree` covers every
//! tracked and untracked path outside `.keel/` - the repository as the eighteen SELF-READING tests
//! (those whose text names `CARGO_MANIFEST_DIR`) see it - plus HEAD's id, because four of them read
//! git state (`status_distinguishes_unknown.rs` reads drift against HEAD) and a commit that changes no
//! byte still changes that answer. Over-inclusion reruns a binary; under-inclusion skips one whose
//! outcome moved, and that is the CI red D0421 exists to prevent - so `is_code` is generous.
//!
//! `.keel/` is outside both keys: `metrics/` is written by the runs themselves, `bin/` holds binaries
//! no test opens, and the guard receipt (`receipt.rs`) walks the rest for its own key.
//!
//! TWO NARROWER KEYS (D0481, sprint 741), each a digest over the same listing. [`scoped_code`] is the
//! code key restricted to what one member's unit tests are built from - its directory and its
//! dependencies' (`wsgraph::scope`) plus the code no member owns - so a member's `lib` row is skipped
//! on the code it is built from, not the whole workspace's. [`reads_key`] is the bytes under the paths
//! a self-reading binary RECORDED reading (`keel_fs::test_support`), so the ceremony's write under
//! `.tracking/` no longer reruns a test that reads `keel-cli/src/main.rs` and `.githooks/`. A recorded
//! `.` is the whole tree, and its key IS the tree key - head included, because a root reader can read
//! git state; a narrower set carries no head, because a commit that changes none of its bytes changes
//! nothing it can see.

use std::hash::{Hash, Hasher};
use std::path::Path;

/// One listing of the tree, read once and digested several ways: HEAD's id and every path outside
/// `.keel/` ([`paths`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing {
    pub head: String,
    pub paths: Vec<String>,
}

impl Listing {
    /// The listing of `root`; `None` when git cannot name the tree.
    #[must_use]
    pub fn read(root: &Path) -> Option<Self> {
        Some(Self { head: head(root)?, paths: paths(root)? })
    }

    /// The digest of `head` (when `with_head`) and of every listed path `keep` admits, in listing order.
    fn digest(&self, root: &Path, with_head: bool, keep: impl Fn(&str) -> bool) -> String {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        if with_head {
            h.write(self.head.as_bytes());
        }
        for rel in self.paths.iter().filter(|p| keep(p)) {
            hash_file(root, rel, &mut h);
        }
        format!("{:016x}", h.finish())
    }

    /// Both D0474 keys over this listing.
    #[must_use]
    pub fn keys(&self, root: &Path) -> ContentKeys {
        ContentKeys { code: self.digest(root, false, is_code), tree: self.digest(root, true, |_| true) }
    }
}

/// The code key narrowed to the paths `keep` admits - the caller's "built from" predicate over
/// `is_code` paths (D0481: a member's scope directories plus the code no member owns).
#[must_use]
pub fn scoped_code(root: &Path, listing: &Listing, keep: impl Fn(&str) -> bool) -> String {
    listing.digest(root, false, |p| is_code(p) && keep(p))
}

/// The key of a recorded read set: `/`-normalised repo-relative entries, each a file or a directory.
/// Any entry equal to `.` (or empty, or climbing out) makes the key the TREE key - head and all; else
/// the digest of every listed path equal to or under an entry, without head. `None` for an empty set:
/// a binary that recorded nothing has no key of its own and falls back to the tree key.
#[must_use]
pub fn reads_key(root: &Path, listing: &Listing, reads: &[String]) -> Option<String> {
    if reads.is_empty() {
        return None;
    }
    let entries: Vec<String> = reads.iter().map(|r| r.replace('\\', "/").trim_matches('/').to_owned()).collect();
    if entries.iter().any(|e| e.is_empty() || e == "." || e.split('/').any(|s| s == "..")) {
        return Some(listing.digest(root, true, |_| true));
    }
    Some(listing.digest(root, false, |p| entries.iter().any(|e| p == e || p.starts_with(&format!("{e}/")))))
}

/// The two digests of one tree, as hex.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentKeys {
    /// Every path compiled into or run by the test binaries.
    pub code: String,
    /// Every path outside `.keel/`, plus HEAD - what a self-reading test can see.
    pub tree: String,
}

/// Is `rel` (repo-relative, either separator) a path the test binaries are built from or run against?
///
/// Pure. `keel-cli/src/x.rs`, `members/keel-git/src/gitx.rs`, `keel-parser/src/lexer.rs`,
/// `.engine/skills/y/SKILL.md`, `Cargo.lock`, `.githooks/post-commit`, `.claude/output-styles/keel.md`,
/// `keelw` -> true; `.tracking/backlog.sysml`, `docs/x.md`, `fonts/a.ttf` -> false. Every workspace
/// member is code (sprint 714): the binaries link all of them.
#[must_use]
pub fn is_code(rel: &str) -> bool {
    let p = rel.replace('\\', "/");
    p.starts_with("keel-cli/")
        || p.starts_with("members/")
        || p.starts_with("keel-parser/")
        || p.starts_with(".engine/")
        || p.starts_with(".githooks/")
        || p.starts_with(".claude/")
        || p == "Cargo.toml"
        || p == "Cargo.lock"
        || p == "keelw"
}

/// Is `rel` under the one directory neither key reads?
#[must_use]
pub fn is_keel_local(rel: &str) -> bool {
    let p = rel.replace('\\', "/");
    p == ".keel" || p.starts_with(".keel/")
}

/// Hash one path's name and bytes - or its absence, for a listed path that is gone.
fn hash_file(root: &Path, rel: &str, h: &mut impl Hasher) {
    rel.hash(h);
    match std::fs::read(root.join(rel)) {
        Ok(bytes) => {
            bytes.len().hash(h);
            bytes.hash(h);
        }
        Err(_) => "absent".hash(h),
    }
}

fn git_z(root: &Path, args: &[&str]) -> Option<Vec<String>> {
    let o = keel_git::gitx::git().arg("-C").arg(root).args(args).output().ok()?;
    if !o.status.success() {
        return None;
    }
    Some(o.stdout.split(|b| *b == 0).filter(|f| !f.is_empty()).map(|f| String::from_utf8_lossy(f).replace('\\', "/")).collect())
}

/// The paths both keys are computed over: every tracked path and every untracked path git does not
/// ignore, sorted and deduplicated, `.keel/` removed. `None` when git cannot list them.
#[must_use]
pub fn paths(root: &Path) -> Option<Vec<String>> {
    let mut all = git_z(root, &["ls-files", "-z"])?;
    all.extend(git_z(root, &["ls-files", "-z", "--others", "--exclude-standard"])?);
    all.retain(|p| !is_keel_local(p));
    all.sort();
    all.dedup();
    Some(all)
}

/// HEAD's id, read live: `gitfacts::head_sha` memoises per process, and the key is asked for again
/// after a run that may sit across a commit.
fn head(root: &Path) -> Option<String> {
    let o = keel_git::gitx::git().arg("-C").arg(root).args(["rev-parse", "HEAD"]).output().ok()?;
    o.status.success().then(|| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// Compute both keys for `root`. `None` when git cannot name the tree - a tree whose inputs cannot be
/// listed is never skipped against.
#[must_use]
pub fn compute(root: &Path) -> Option<ContentKeys> {
    keel_perf::perf::phase("contentkey:compute", || Some(Listing::read(root)?.keys(root)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// D0388 pair, chosen before the rule was read: the module a test compiles against and the
    /// embedded engine file are code; the ceremony's own file and a font are not.
    #[test]
    fn code_is_what_the_binaries_are_built_from_or_run_against() {
        assert!(is_code("keel-cli/src/touched.rs"));
        assert!(is_code(".engine\\skills\\x\\SKILL.md"));
        assert!(is_code("Cargo.lock"));
        assert!(is_code(".githooks/post-commit"));
        assert!(is_code(".claude/output-styles/keel.md"));
        assert!(is_code("keelw"));
        assert!(is_code("members/keel-git/src/gitx.rs"));
        assert!(is_code("keel-parser/src/lexer.rs"));
        assert!(!is_code(".tracking/backlog.sysml"));
        assert!(!is_code(".tracking/delivery/sprint706_x.sysml"));
        assert!(!is_code("fonts/NotoEmoji-var.ttf"));
        assert!(!is_code("docs/usage.md"));
        assert!(is_keel_local(".keel/metrics/touched-receipt.toml") && is_keel_local(".keel") && !is_keel_local(".keelx/y"));
    }

    fn git(dir: &Path, args: &[&str]) {
        let out = keel_git::gitx::git().arg("-C").arg(dir).args(args).output().expect("git");
        assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    }

    /// Known-positive: a byte changed under `keel-cli/` moves BOTH keys; a byte changed under
    /// `.tracking/` moves `tree` alone; a commit that changes no byte moves `tree` alone (HEAD is in
    /// it). Known-negative: the same bytes rewritten with a new mtime move neither - the clock is not
    /// an input any more. `.keel/metrics/` writes move neither.
    #[test]
    fn a_code_edit_moves_both_keys_a_tracking_edit_moves_the_tree_key_and_an_mtime_moves_neither() {
        let d = std::env::temp_dir().join(format!("keel-contentkey-{}", keel_model::ident::gen_uuid()));
        std::fs::create_dir_all(d.join("keel-cli").join("src")).expect("mk");
        std::fs::create_dir_all(d.join(".tracking")).expect("mk");
        std::fs::write(d.join("keel-cli").join("src").join("lib.rs"), "pub fn a() {}\n").expect("w");
        std::fs::write(d.join(".tracking").join("backlog.sysml"), "package B {}\n").expect("w");
        git(&d, &["init", "-q"]);
        git(&d, &["add", "-A"]);
        git(&d, &["-c", "user.email=t@t", "-c", "user.name=t", "commit", "-q", "-m", "root"]);
        let k0 = compute(&d).expect("keys");

        // Known-negative first: the same bytes, a new mtime.
        std::fs::write(d.join("keel-cli").join("src").join("lib.rs"), "pub fn a() {}\n").expect("w");
        std::fs::OpenOptions::new().write(true).open(d.join("keel-cli").join("src").join("lib.rs")).expect("open").set_modified(std::time::SystemTime::now()).expect("mtime");
        assert_eq!(compute(&d).expect("keys"), k0, "an mtime is not an input");
        std::fs::create_dir_all(d.join(".keel").join("metrics")).expect("mk");
        std::fs::write(d.join(".keel").join("metrics").join("touched-receipt.toml"), "outcome = \"pass\"\n").expect("w");
        assert_eq!(compute(&d).expect("keys"), k0, ".keel/ is outside both keys");

        // A ceremony write: untracked, under .tracking - tree moves, code holds.
        std::fs::write(d.join(".tracking").join("sprint.sysml"), "package S {}\n").expect("w");
        let k1 = compute(&d).expect("keys");
        assert_eq!(k1.code, k0.code, "a .tracking write leaves the code key");
        assert_ne!(k1.tree, k0.tree, "and moves the tree key");

        // A commit of it: no byte changes, HEAD does - tree moves again, code holds.
        git(&d, &["add", "-A"]);
        git(&d, &["-c", "user.email=t@t", "-c", "user.name=t", "commit", "-q", "-m", "ceremony"]);
        let k2 = compute(&d).expect("keys");
        assert_eq!(k2.code, k1.code);
        assert_ne!(k2.tree, k1.tree, "HEAD is in the tree key: a self-reading test can read git state");

        // A code edit: both move.
        std::fs::write(d.join("keel-cli").join("src").join("lib.rs"), "pub fn a() -> u8 { 1 }\n").expect("w");
        let k3 = compute(&d).expect("keys");
        assert_ne!(k3.code, k2.code, "a source edit moves the code key");
        assert_ne!(k3.tree, k2.tree, "and the tree key");
        let _ = std::fs::remove_dir_all(&d);
    }

    /// D0481 pair (the definition of done's second known-positive), chosen before the code was written: a recorded
    /// read set `[keel-cli/src/main.rs, .githooks]` keeps its key across an edit to a path OUTSIDE the
    /// set (`.tracking/`, a sibling source file, a commit) and loses it on an edit INSIDE it (a byte
    /// under `.githooks/`). Known-negative: a set holding `.` is the tree key itself, an empty set has
    /// no key, and the member-scoped code key ignores a member outside the scope.
    #[test]
    fn a_read_set_keeps_its_key_across_an_edit_outside_it_and_loses_it_on_one_inside() {
        let d = std::env::temp_dir().join(format!("keel-readskey-{}", keel_model::ident::gen_uuid()));
        for sub in ["keel-cli/src", ".githooks", ".tracking", "members/leaf/src", "members/other/src"] {
            std::fs::create_dir_all(d.join(sub)).expect("mk");
        }
        std::fs::write(d.join("keel-cli/src/main.rs"), "fn main() {}\n").expect("w");
        std::fs::write(d.join("keel-cli/src/lib.rs"), "pub fn a() {}\n").expect("w");
        std::fs::write(d.join(".githooks/post-commit"), "#!/bin/sh\n").expect("w");
        std::fs::write(d.join(".tracking/backlog.sysml"), "package B {}\n").expect("w");
        std::fs::write(d.join("members/leaf/src/lib.rs"), "pub fn l() {}\n").expect("w");
        std::fs::write(d.join("members/other/src/lib.rs"), "pub fn o() {}\n").expect("w");
        std::fs::write(d.join("Cargo.toml"), "[workspace]\n").expect("w");
        git(&d, &["init", "-q"]);
        git(&d, &["add", "-A"]);
        git(&d, &["-c", "user.email=t@t", "-c", "user.name=t", "commit", "-q", "-m", "root"]);
        let reads = vec!["keel-cli/src/main.rs".to_owned(), ".githooks/".to_owned()];
        // The leaf's scope: its own directory plus the code no member owns (keel-cli and `other` are members here).
        let leaf_scope = |p: &str| p.starts_with("members/leaf/") || (is_code(p) && !p.starts_with("members/") && !p.starts_with("keel-cli/"));
        let l0 = Listing::read(&d).expect("listing");
        let k0 = l0.keys(&d);
        let r0 = reads_key(&d, &l0, &reads).expect("key");
        let s0 = scoped_code(&d, &l0, leaf_scope);
        assert_eq!(reads_key(&d, &l0, &[".".to_owned()]).as_deref(), Some(k0.tree.as_str()), "a root reader is keyed on the tree");
        assert_eq!(reads_key(&d, &l0, &["a/../b".to_owned()]).as_deref(), Some(k0.tree.as_str()), "an escaping entry is the tree");
        assert_eq!(reads_key(&d, &l0, &[]), None, "no recorded set, no key of its own");

        // Outside the set: a ceremony write, a sibling source edit, a commit - the read-set key holds.
        std::fs::write(d.join(".tracking/sprint.sysml"), "package S {}\n").expect("w");
        std::fs::write(d.join("keel-cli/src/lib.rs"), "pub fn a() -> u8 { 1 }\n").expect("w");
        std::fs::write(d.join("members/other/src/lib.rs"), "pub fn o() -> u8 { 2 }\n").expect("w");
        git(&d, &["add", "-A"]);
        git(&d, &["-c", "user.email=t@t", "-c", "user.name=t", "commit", "-q", "-m", "outside"]);
        let l1 = Listing::read(&d).expect("listing");
        let k1 = l1.keys(&d);
        assert_ne!(k1.tree, k0.tree, "the tree key moved (the D0474 fallback would rerun)");
        assert_ne!(k1.code, k0.code, "the whole-workspace code key moved");
        assert_eq!(reads_key(&d, &l1, &reads).expect("key"), r0, "the read-set key did not: nothing it reads changed");
        assert_eq!(scoped_code(&d, &l1, leaf_scope), s0, "the leaf's scoped code key did not: `other` is outside its scope");

        // Inside the set: one byte under .githooks/ - the read-set key moves.
        std::fs::write(d.join(".githooks/post-commit"), "#!/bin/sh\nexit 0\n").expect("w");
        let l2 = Listing::read(&d).expect("listing");
        assert_ne!(reads_key(&d, &l2, &reads).expect("key"), r0, "a byte under a recorded directory moves the key");
        std::fs::write(d.join("members/leaf/src/lib.rs"), "pub fn l() -> u8 { 3 }\n").expect("w");
        assert_ne!(scoped_code(&d, &Listing::read(&d).expect("listing"), leaf_scope), s0, "a leaf edit moves the leaf's scoped key");
        let _ = std::fs::remove_dir_all(&d);
    }
}
