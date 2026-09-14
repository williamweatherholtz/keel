//! Every flag-shaped prose input reads a FILE (D0224, issue546, sprint 713).
//!
//! Sprint 711 gave `record result` and `record gate-result` their `--evidence-from` / `--notes-from` forms
//! through one reader, `prose_flag`; its retro found six more prose flags still riding the shell argument
//! path - `override --reason`, `record story --i-want / --so-that / --triage-note`, and the `--note` of
//! `accept`, `reject` and `judge-set`. A backtick inside a double-quoted shell argument is command
//! substitution: the shell RUNS the command the prose merely names, into the record. These tests drive
//! the built binary with a file carrying a backtick span and a parenthesised refusal into each new form
//! and read the record back. Known-positive: the text lands verbatim (modulo the field sanitiser, which
//! collapses whitespace and never touches a backtick). Known-negative: the inline forms still land, and
//! both forms of one flag at once is refused naming both, with nothing written.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const SPAN: &str = "run `keel gate validate .` first (refused otherwise)";

fn keel_bin() -> PathBuf {
    let mut p = std::env::current_exe().expect("test exe path");
    p.pop();
    if p.ends_with("deps") {
        p.pop();
    }
    p.join(if cfg!(windows) { "keel.exe" } else { "keel" })
}

/// Run as an AGENT session would: the marker set, stdin not a terminal, the actor bound.
fn agent(dir: &Path, args: &[&str]) -> (bool, String) {
    let out = Command::new(keel_bin())
        .args(args)
        .current_dir(dir)
        .env("CLAUDE_CODE_SESSION_ID", "00000000-0000-4000-8000-000000000001")
        .env("KEEL_ACTOR", "ai")
        .stdin(Stdio::null())
        .output()
        .expect("keel runs");
    (out.status.success(), format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)))
}

fn shallow_root(tag: &str) -> PathBuf {
    let base = if cfg!(windows) { PathBuf::from("C:\\kt") } else { std::env::temp_dir() };
    let root = base.join(format!("pf{tag}{}", std::process::id() % 10_000));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("mkdir");
    root
}

fn scaffold(tag: &str) -> PathBuf {
    let root = shallow_root(tag);
    assert!(agent(&root, &["init", "."]).0, "scaffold");
    root
}

/// A prose file: the span the shell would have executed as a double-quoted argument.
fn prose_file(root: &Path, name: &str, text: &str) -> String {
    let p = root.join(name);
    std::fs::write(&p, format!("{text}\n")).expect("prose file");
    p.to_string_lossy().replace('\\', "/")
}

fn tracking_text(root: &Path) -> String {
    let mut out = String::new();
    fn walk(dir: &Path, out: &mut String) {
        for e in std::fs::read_dir(dir).expect("dir").flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "sysml") {
                out.push_str(&std::fs::read_to_string(&p).expect("read"));
            }
        }
    }
    walk(&root.join(".tracking"), &mut out);
    out
}

fn decision_text(root: &Path, slug: &str) -> String {
    let dir = root.join(".engine").join("decisions");
    let f = std::fs::read_dir(&dir).expect("decisions").flatten().map(|e| e.path()).find(|p| p.to_string_lossy().contains(slug)).expect("the decision file");
    std::fs::read_to_string(f).expect("read")
}

/// issue376: a fresh scaffold ships the policy with its GRANT lines commented out; this human delegates
/// the RECORDING of their verdicts (D0192) so an agent session may write them.
fn grant_recording_delegation(root: &Path) {
    let p = root.join(".engine/contracts/attestation-policy.toml");
    let text = std::fs::read_to_string(&p).expect("policy");
    let granted = text.replace("# delegatedRecording = \"d0192\"", "delegatedRecording = \"d0192\"");
    assert_ne!(text, granted, "the commented delegation line ships");
    std::fs::write(&p, granted).expect("grant");
}

fn proposed_decision(root: &Path, slug: &str) {
    let (ok, text) = agent(
        root,
        &["record", "decision", "--slug", slug, "--title", "t", "--context", "c", "--decision", "d", "--rationale", "r", "--consequences", "q", "--author", "ai", "--date", "2026-09-14"],
    );
    assert!(ok, "a proposed decision exists: {text}");
}

#[test]
fn override_reason_reads_a_file_and_the_inline_form_still_lands() {
    let root = scaffold("ov");
    let reason = prose_file(&root, "reason.txt", SPAN);
    let (ok, text) = agent(&root, &["override", ".engine/contracts/attestation-policy.toml", "--reason-from", &reason]);
    assert!(ok, "--reason-from arms the override: {text}");
    let unlock = std::fs::read_to_string(root.join(".keel").join("override.json")).expect("the unlock");
    assert!(unlock.contains(SPAN), "the reason landed verbatim, backtick span included:\n{unlock}");

    // known-negative: the inline form is unchanged
    let (ok, text) = agent(&root, &["override", ".engine/contracts/attestation-policy.toml", "--reason", "inline reason, ten chars and more"]);
    assert!(ok, "--reason still arms: {text}");
    assert!(std::fs::read_to_string(root.join(".keel").join("override.json")).expect("the unlock").contains("inline reason"));

    // both at once: refused by name, and the armed unlock is not replaced
    let (ok, text) = agent(&root, &["override", ".engine/contracts/attestation-policy.toml", "--reason-from", &reason, "--reason", "and inline too, long enough"]);
    assert!(!ok, "both forms is a refusal: {text}");
    assert!(text.contains("--reason-from") && text.contains("--reason "), "the refusal names both flags: {text}");
    assert!(std::fs::read_to_string(root.join(".keel").join("override.json")).expect("the unlock").contains("inline reason"), "nothing was written by the refused call");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn story_prose_reads_files_field_by_field() {
    let root = scaffold("st");
    let (ok, text) = agent(&root, &["record", "statement", "--text", "it keeps running my prose as a command", "--said-by", "you", "--said-at", "2026-09-14", "--title", "prose ran"]);
    assert!(ok, "a statement to derive from: {text}");
    let i_want = prose_file(&root, "iwant.txt", &format!("to {SPAN}"));
    let so_that = prose_file(&root, "sothat.txt", "the record holds what I wrote (and not what the shell ran)");
    let triage = prose_file(&root, "triage.txt", "maps to `prose_flag` (D0224); not a new verb");
    let (ok, text) = agent(
        &root,
        &["record", "story", "--from-statement", "st001", "--title", "prose lands", "--as-a", "recorder", "--i-want-from", &i_want, "--implication", "need", "--so-that-from", &so_that, "--triage-note-from", &triage, "--at", "2026-09-14"],
    );
    assert!(ok, "the story records from three files: {text}");
    let t = tracking_text(&root);
    assert!(t.contains(&format!("to {SPAN}")), "iWant landed verbatim:\n{t}");
    assert!(t.contains("the record holds what I wrote (and not what the shell ran)"), "soThat landed:\n{t}");
    assert!(t.contains("maps to `prose_flag` (D0224); not a new verb"), "triageNote landed:\n{t}");

    // known-negative: inline --i-want still lands; both forms of one field is refused
    let (ok, text) = agent(&root, &["record", "story", "--from-statement", "st001", "--title", "inline", "--as-a", "recorder", "--i-want", "an inline capability", "--implication", "need", "--at", "2026-09-14"]);
    assert!(ok, "inline --i-want unchanged: {text}");
    assert!(tracking_text(&root).contains("an inline capability"));
    let before = tracking_text(&root);
    let (ok, text) = agent(&root, &["record", "story", "--from-statement", "st001", "--title", "both", "--as-a", "recorder", "--i-want-from", &i_want, "--i-want", "inline too", "--implication", "need", "--at", "2026-09-14"]);
    assert!(!ok, "both forms of --i-want is a refusal: {text}");
    assert!(text.contains("--i-want-from") && text.contains("--i-want "), "the refusal names both: {text}");
    assert_eq!(before, tracking_text(&root), "nothing was written by the refused call");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn accept_and_reject_notes_read_a_file_before_the_channel_layer_reads_them() {
    let root = scaffold("ar");
    grant_recording_delegation(&root);
    proposed_decision(&root, "probe");
    proposed_decision(&root, "second");
    // The note framing goes through a file; the human's short quote stays a shell argument (D0192).
    let framing = prose_file(&root, "framing.txt", &format!("their words in chat after I said {SPAN}"));
    let (ok, text) = agent(&root, &["accept", "d0001", "--words", "yes, accept d0001 and keep going", "--note-from", &framing, "--by", "you", "--date", "2026-09-14"]);
    assert!(ok, "--note-from folds with --words exactly as --note does: {text}");
    let d = decision_text(&root, "probe");
    assert!(d.contains("DecisionStatus::accepted") && d.contains(SPAN) && d.contains("yes, accept d0001 and keep going"), "framing and quote both landed:\n{d}");

    // A note that IS the quote, from a file: the channel layer's QUOTE check reads the normalised text.
    let quote = prose_file(&root, "quote.txt", "their words in chat: 'reject d0002, we will not do it (see `keel show why`)'");
    let (ok, text) = agent(&root, &["reject", "d0002", "--note-from", &quote, "--by", "you", "--date", "2026-09-14"]);
    assert!(ok, "--note-from satisfies the channel layer's quote check: {text}");
    let d2 = decision_text(&root, "second");
    assert!(d2.contains("DecisionStatus::rejected") && d2.contains("see `keel show why`"), "the rejection carries the file's words:\n{d2}");

    // both forms: refused by name, nothing written
    proposed_decision(&root, "third");
    let (ok, text) = agent(&root, &["accept", "d0003", "--note-from", &quote, "--note", "inline framing", "--by", "you", "--date", "2026-09-14"]);
    assert!(!ok, "both forms of --note is a refusal: {text}");
    assert!(text.contains("--note-from") && text.contains("--note "), "the refusal names both: {text}");
    assert!(!decision_text(&root, "third").contains("DecisionStatus::accepted"), "nothing was written");
    let _ = std::fs::remove_dir_all(&root);
}
