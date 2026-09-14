//! dcEvidenceGoesThroughAFile (issue543, D0224's fifth occurrence): `record result` and
//! `record gate-result` read their receipt from `--evidence-from FILE`, verbatim, so a backtick in
//! the prose is a character in the record and never a command the shell ran. The shell argument
//! form `--evidence TEXT` stays for one-line receipts; both at once is refused by name and nothing is
//! written. The test drives the built binary exactly as a shell would - the point is the argument
//! path, and a unit test on `prose_flag` alone would not show the receipt landing in the file.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn keel_bin() -> PathBuf {
    let mut p = std::env::current_exe().expect("test binary path");
    p.pop();
    if p.ends_with("deps") {
        p.pop();
    }
    p.join(if cfg!(windows) { "keel.exe" } else { "keel" })
}

/// A minimal project: one task with a `method=test` DoD, an actor file, and a git root so the write
/// path's provenance checks have what they read.
fn fixture(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("keel-evfile-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join(".tracking")).expect("mkdir");
    std::fs::create_dir_all(root.join(".keel")).expect("mkdir");
    std::fs::write(root.join(".keel").join("actor"), "claudeOpus5\n").expect("actor");
    std::fs::write(
        root.join(".tracking").join("backlog.sysml"),
        "package Fx {\n    private import EngineElement::*;\n    private import EngineWork::*;\n    private import EngineVerification::*;\n    private import EngineRelationships::*;\n\n    action def Build {\n        action dcThing;\n        verification dcThingDoD : Test { :>> id = \"00000000-0000-4000-8000-000000000001\"; :>> method = VerificationMethod::test; :>> procedureText = \"the agreed criterion\"; }\n    }\n}\n",
    )
    .expect("seed");
    let git = |args: &[&str]| {
        let out = Command::new("git").arg("-C").arg(&root).args(args).output().expect("git runs");
        assert!(out.status.success(), "git {args:?} failed: {}", String::from_utf8_lossy(&out.stderr));
    };
    git(&["init", "-q", "."]);
    git(&["-c", "user.email=p@x", "-c", "user.name=p", "add", "-A"]);
    git(&["-c", "user.email=p@x", "-c", "user.name=p", "commit", "-q", "-m", "seed"]);
    root
}

fn record(root: &Path, extra: &[&str]) -> Output {
    Command::new(keel_bin())
        .args(["record", "result", "--file", ".tracking/backlog.sysml", "--task", "dcThing", "--sha", "abc1234", "--verdict", "pass", "--judged-by", "claudeOpus5", "--judged-at", "2026-09-14"])
        .args(extra)
        .current_dir(root)
        .env("KEEL_ACTOR", "claudeOpus5")
        .output()
        .expect("record result")
}

fn backlog(root: &Path) -> String {
    std::fs::read_to_string(root.join(".tracking").join("backlog.sysml")).expect("read")
}

/// Known positive: the exact text a double-quoted shell argument would have SUBSTITUTED - a backtick
/// span naming a command, and a parenthesised refusal - lands in the `// RAN:` line byte for byte.
#[test]
fn a_receipt_read_from_a_file_lands_verbatim_backticks_included() {
    let root = fixture("verbatim");
    let receipt = "ran `cargo test --release --lib -- write::` (821 passed) and `keel gate guard --no-receipt .` (ALL PASS); refused: `record result` with no receipt";
    let f = root.join("receipt.txt");
    std::fs::write(&f, format!("{receipt}\n")).expect("receipt file");
    let out = record(&root, &["--evidence-from", f.to_str().expect("utf8 path")]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let text = backlog(&root);
    assert!(text.contains(&format!("// RAN: {receipt}")), "the file's text is the receipt, byte for byte:\n{text}");
    assert!(text.contains("part dcThingDoDR1 : TestResult"), "and the result landed under it");
    let _ = std::fs::remove_dir_all(&root);
}

/// Known negative for the shape: the one-line shell form is unchanged - it still lands.
#[test]
fn the_one_line_shell_form_still_lands() {
    let root = fixture("inline");
    let out = record(&root, &["--evidence", "cargo test: ok"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(backlog(&root).contains("// RAN: cargo test: ok"), "the inline receipt is written as before");
    let _ = std::fs::remove_dir_all(&root);
}

/// Both flags at once is refused BY NAME - the caller learns which two collided, not which one would
/// silently have won - and the file is untouched.
#[test]
fn both_flags_at_once_are_refused_by_name_and_nothing_is_written() {
    let root = fixture("both");
    let before = backlog(&root);
    let f = root.join("receipt.txt");
    std::fs::write(&f, "from the file\n").expect("receipt file");
    let out = record(&root, &["--evidence-from", f.to_str().expect("utf8 path"), "--evidence", "from the argument"]);
    assert!(!out.status.success(), "two receipts is a refusal");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("--evidence-from") && err.contains("--evidence"), "the refusal names both flags:\n{err}");
    assert_eq!(backlog(&root), before, "a refused write writes nothing");

    // An unreadable file is the same class of refusal: named, and nothing written.
    let out = record(&root, &["--evidence-from", root.join("no-such-file.txt").to_str().expect("utf8 path")]);
    assert!(!out.status.success(), "a file that cannot be read is a refusal");
    assert!(String::from_utf8_lossy(&out.stderr).contains("cannot read"), "and says so");
    assert_eq!(backlog(&root), before);
    let _ = std::fs::remove_dir_all(&root);
}
