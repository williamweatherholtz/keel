//! dcSchemaLacksMemberRefusalIsLedgered (issue616): the D0521 refusal - a PROPOSED verdict into a
//! tree whose `VerdictKind` lacks `proposed` - is a registry row (`write::WRITE_PATH_REFUSALS`), a
//! census row, and a ledger line `append-result:schema-member` / `append-gate-result:schema-member`
//! in `.keel/metrics/hooks.jsonl`. Before this the refusal fired with no row anywhere, so the fire
//! ledger could not count it and the census claimed nothing about it. The test drives the built
//! binary, since the ledger arm lives in main.rs and a unit test on the write API never reaches it.
//!
//! The D0388 pair. KNOWN-POSITIVE: an AI demo pass (no receipt - a demo owes none, and it lands
//! PROPOSED for an unregistered judge) into a fixture whose `element.sysml` declares the pre-D0312
//! `VerdictKind` exits 1, writes nothing, and leaves exactly one `append-result:schema-member`
//! ledger line - never an `unregistered:` one. KNOWN-NEGATIVE: the same write into the same fixture
//! with `proposed` added to the enum exits 0, lands `VerdictKind::proposed`, and leaves NO refused
//! line.

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

const OLDER_ENUM: &str = "enum def VerdictKind { pass; fail; inconclusive; error; }";
const CURRENT_ENUM: &str = "enum def VerdictKind { pass; fail; inconclusive; error; proposed; }";

/// A project at a schema vintage: one task with a `method=demo` DoD, one ceremony gate, an actor
/// file, a git root, and the ONE schema file the write path reads (`schema/core/element.sysml`)
/// declaring `VerdictKind` as `enum_line` says.
fn fixture(tag: &str, enum_line: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("keel-schemamember-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join(".tracking").join("delivery")).expect("mkdir");
    std::fs::create_dir_all(root.join(".keel")).expect("mkdir");
    let schema = root.join(".engine").join("schema").join("core");
    std::fs::create_dir_all(&schema).expect("mkdir schema");
    std::fs::write(
        schema.join("element.sysml"),
        format!("package Core {{\n    enum def ActorKind {{ human; ai; }}\n    {enum_line}\n    enum def VerificationMethod {{ inspect; analyze; demo; test; confirmation; critique; }}\n}}\n"),
    )
    .expect("schema");
    std::fs::write(root.join(".keel").join("actor"), "claudeOpus5\n").expect("actor");
    // the judge is a REGISTERED ai actor: `proposed_tier` reads the registry, and a tree with no
    // actors.sysml (the write API's own fixtures) lands every pass native
    std::fs::write(
        root.join(".tracking").join("actors.sysml"),
        "package ProjectActors {\n    private import EngineElement::*;\n\n    part claudeOpus5 : Actor { :>> name = \"Claude Opus 5\"; :>> kind = ActorKind::ai; :>> role = \"contributor\"; }\n}\n",
    )
    .expect("actors");
    std::fs::write(
        root.join(".tracking").join("backlog.sysml"),
        "package Fx {\n    private import EngineElement::*;\n    private import EngineWork::*;\n    private import EngineVerification::*;\n\n    action def Build {\n        action dcThing;\n        verification dcThingDoD : Test { :>> id = \"00000000-0000-4000-8000-000000000001\"; :>> method = VerificationMethod::demo; :>> procedureText = \"the demo criterion\"; }\n    }\n}\n",
    )
    .expect("backlog");
    std::fs::write(
        root.join(".tracking").join("delivery").join("sprint1_x.sysml"),
        "package S1 {\n    private import EngineElement::*;\n    private import EngineVerification::*;\n\n    verification xImplementGate : Test { :>> id = \"00000000-0000-4000-8000-000000000002\"; :>> method = VerificationMethod::demo; :>> procedureText = \"the implement gate\"; }\n}\n",
    )
    .expect("sprint");
    let git = |args: &[&str]| {
        let out = Command::new("git").arg("-C").arg(&root).args(args).output().expect("git runs");
        assert!(out.status.success(), "git {args:?} failed: {}", String::from_utf8_lossy(&out.stderr));
    };
    git(&["init", "-q", "."]);
    git(&["-c", "user.email=p@x", "-c", "user.name=p", "add", "-A"]);
    git(&["-c", "user.email=p@x", "-c", "user.name=p", "commit", "-q", "-m", "seed"]);
    root
}

fn keel(root: &Path, args: &[&str]) -> Output {
    Command::new(keel_bin()).args(args).current_dir(root).env("KEEL_ACTOR", "claudeOpus5").output().expect("keel runs")
}

fn record_result(root: &Path) -> Output {
    keel(root, &["record", "result", "--file", ".tracking/backlog.sysml", "--task", "dcThing", "--sha", "abc1234", "--verdict", "pass", "--judged-by", "claudeOpus5", "--judged-at", "2026-09-18"])
}

fn record_gate_result(root: &Path) -> Output {
    keel(root, &["record", "gate-result", "--file", ".tracking/delivery/sprint1_x.sysml", "--gate", "xImplementGate", "--sha", "abc1234", "--verdict", "pass", "--judged-by", "claudeOpus5", "--judged-at", "2026-09-18"])
}

/// The `refused` lines of the fixture's fire ledger, or none when nothing has been ledgered.
fn refused_lines(root: &Path) -> Vec<String> {
    std::fs::read_to_string(root.join(".keel").join("metrics").join("hooks.jsonl"))
        .unwrap_or_default()
        .lines()
        .filter(|l| l.contains(r#""event":"refused""#))
        .map(str::to_owned)
        .collect()
}

#[test]
fn a_proposed_result_into_the_older_vintage_is_refused_and_ledgered_under_its_registered_name() {
    let root = fixture("result-older", OLDER_ENUM);
    let before = std::fs::read_to_string(root.join(".tracking").join("backlog.sysml")).expect("read");

    let out = record_result(&root);
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "the older vintage refuses the proposed line: {err}");
    assert!(err.contains("VerdictKind::proposed") && err.contains("keel migrate"), "the refusal names the member and the remedy: {err}");
    assert!(!err.contains("unregistered"), "the control is a registry row, so the ledger never marks it unregistered (issue449): {err}");
    assert_eq!(std::fs::read_to_string(root.join(".tracking").join("backlog.sysml")).expect("read"), before, "a refused write leaves the file byte-for-byte");

    let refused = refused_lines(&root);
    assert_eq!(refused.len(), 1, "exactly one refused line: {refused:?}");
    assert!(refused[0].contains(r#""control":"append-result:schema-member""#), "the line names the census row: {}", refused[0]);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_proposed_gate_result_into_the_older_vintage_is_ledgered_under_the_gate_verb() {
    let root = fixture("gate-older", OLDER_ENUM);
    let out = record_gate_result(&root);
    assert_eq!(out.status.code(), Some(1), "refused: {}", String::from_utf8_lossy(&out.stderr));
    let refused = refused_lines(&root);
    assert_eq!(refused.len(), 1, "exactly one refused line: {refused:?}");
    assert!(refused[0].contains(r#""control":"append-gate-result:schema-member""#), "{}", refused[0]);
    let _ = std::fs::remove_dir_all(&root);
}

/// Known negative: the tree that declares `proposed` takes the same write, lands it proposed, and
/// the ledger holds no refusal - the row fires on the vintage, not on the verb.
#[test]
fn the_same_write_into_a_tree_that_declares_proposed_lands_and_ledgers_no_refusal() {
    let root = fixture("current", CURRENT_ENUM);
    let out = record_result(&root);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let text = std::fs::read_to_string(root.join(".tracking").join("backlog.sysml")).expect("read");
    assert!(text.contains("part dcThingDoDR1 : TestResult") && text.contains("VerdictKind::proposed"), "the AI-examined pass lands proposed (D0312 B):\n{text}");
    let out = record_gate_result(&root);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(refused_lines(&root).is_empty(), "nothing refused, nothing ledgered as refused: {:?}", refused_lines(&root));
    let _ = std::fs::remove_dir_all(&root);
}
