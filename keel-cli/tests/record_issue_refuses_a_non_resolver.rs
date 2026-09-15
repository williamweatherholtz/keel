//! `record issue --resolver` refuses what the `resolver-kind` guard refuses (issue558).
//!
//! Sprint 718's first triage of issue556 named the sprint Story as its resolver. The write checked
//! only that the item EXISTED, printed "triaged on arrival", and the pre-commit guard was the first
//! thing to say a Story is not a resolver. A write that reports a triage the commit gate rejects is
//! the write-time/gate-time gap this probe holds shut: the command now reads the guard's own
//! predicate (`guards::resolver_kind_holds`), so the two cannot disagree.
//!
//! The D0388 pair: the Story is refused with nothing written (known-positive); an action and a
//! Decision are both accepted (known-negatives, one per kind the predicate admits).

use std::path::{Path, PathBuf};
use std::process::Command;

fn keel_bin() -> PathBuf {
    let mut p = std::env::current_exe().expect("test exe path");
    p.pop();
    if p.ends_with("deps") {
        p.pop();
    }
    p.join(if cfg!(windows) { "keel.exe" } else { "keel" })
}

/// A throwaway project holding one item of each kind the predicate must tell apart.
fn fixture(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("keel-resolverkind-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join(".tracking")).expect("mkdir tracking");
    std::fs::write(root.join(".tracking").join("issues.sysml"), "package Seed {\n}\n").expect("seed issues");
    std::fs::write(
        root.join(".tracking").join("backlog.sysml"),
        "package Seed {\n    action doTheWork;\n    part aStory : Story { :>> title = \"a story\"; }\n    part d0001 : Decision { :>> title = \"a decision\"; }\n}\n",
    )
    .expect("seed backlog");
    std::fs::write(root.join("desc.md"), "a defect found while probing\n").expect("description file");
    root
}

fn record_issue(root: &Path, resolver: &str) -> std::process::Output {
    Command::new(keel_bin())
        .args([
            "record", "issue",
            "--title", "probe issue",
            "--description-from", &root.join("desc.md").to_string_lossy(),
            "--severity", "Low",
            "--resolver", resolver,
            "--date", "2026-09-14",
            "--by", "claudeFable5",
            "--root", &root.to_string_lossy(),
        ])
        .output()
        .expect("keel ran")
}

fn issues_written(root: &Path) -> usize {
    std::fs::read_dir(root.join(".tracking"))
        .map(|d| {
            d.filter_map(Result::ok)
                .filter_map(|e| std::fs::read_to_string(e.path()).ok())
                .map(|t| t.matches(": Issue").count())
                .sum()
        })
        .unwrap_or(0)
}

#[test]
fn a_story_resolver_is_refused_before_anything_is_written() {
    assert!(keel_bin().exists(), "keel binary not found at {} — build before running this probe", keel_bin().display());
    let root = fixture("story");
    let out = record_issue(&root, "aStory");
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "a Story resolver must be refused at the write; stderr: {err}");
    assert!(err.contains("is a Story, not a declared action or a Decision"), "the refusal names the kind it saw: {err}");
    assert!(err.contains("resolver-kind"), "the refusal names the guard whose predicate it applied: {err}");
    assert_eq!(issues_written(&root), 0, "a refused write leaves nothing in the tree");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn an_action_and_a_decision_are_both_accepted() {
    assert!(keel_bin().exists(), "keel binary not found at {} — build before running this probe", keel_bin().display());
    let root = fixture("kinds");
    for resolver in ["doTheWork", "d0001"] {
        let out = record_issue(&root, resolver);
        assert!(
            out.status.success(),
            "resolver {resolver} is a kind the predicate admits; stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    assert_eq!(issues_written(&root), 2, "both accepted writes landed");
    let _ = std::fs::remove_dir_all(&root);
}
