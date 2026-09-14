//! dcReadyHonoursItemDependencies (issue549, issue552): the ready frontier honours a `#DependsOn` edge
//! from one work item to another, and a `#CharteredBy` edge to a Decision the human has not accepted.
//!
//! Before this the succession chain (`first A then B`) and a `#DependsOn` to a PROPOSED Decision were
//! the only edges the frontier read; nineteen item-to-item edges in this repository's backlog were read
//! by nothing and a six-member layering chain ranked ready at once. `keel show orient` now lists each
//! held-off item under `blocked` with the item it waits on, and nothing is stored: a pass on the
//! predecessor, or the human's acceptance, frees the dependant with no further edit.
//!
//! The known-positive pair (chosen before the real tree was read): A #DependsOn B with B undone lists
//! A blocked on B and B ready, and A is ready once B carries a pass; a Story chartered by a proposed
//! Decision is blocked, and ready once the Decision is accepted. The known-negative lives in the sprint
//! record: the real tree's frontier lost exactly the items whose predecessor is undone and nothing else.

use std::path::{Path, PathBuf};
use std::process::Command;

fn keel_bin() -> PathBuf {
    let mut p = std::env::current_exe().expect("test binary path");
    p.pop();
    if p.ends_with("deps") {
        p.pop();
    }
    p.join(if cfg!(windows) { "keel.exe" } else { "keel" })
}

fn git(root: &Path, args: &[&str]) -> String {
    let out = Command::new("git").arg("-C").arg(root).args(args).output().expect("git runs");
    assert!(out.status.success(), "git {args:?} failed: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn orient(root: &Path) -> serde_json::Value {
    let out = Command::new(keel_bin()).args(["show", "orient", "."]).current_dir(root).env("KEEL_OFFLINE", "1").env("KEEL_ACTOR", "claudeOpus5").output().expect("orient");
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("orient JSON: {e}\n{text}"))
}

fn names(v: &serde_json::Value) -> Vec<String> {
    v.as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect()).unwrap_or_default()
}

fn decision(status: &str) -> String {
    format!(
        "package Decision0001 {{\n    private import EngineElement::*;\n    part d0001 : Decision {{\n        :>> id = \"00000000-0000-4000-8000-000000000001\";\n        :>> title = \"probe\";\n        :>> createdAt = \"2026-09-01\";\n        :>> createdBy = \"hum\";\n        :>> status = DecisionStatus::{status};\n        :>> context = \"c\";\n        :>> decision = \"the charter under probe\";\n        :>> rationale = \"r\";\n        :>> consequences = \"q\";\n    }}\n}}\n"
    )
}

#[test]
fn an_item_waits_on_its_undone_predecessor_and_a_story_waits_on_its_held_charter() {
    let root = std::env::temp_dir().join(format!("keel-itemdeps-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join(".tracking")).expect("mkdir");
    std::fs::create_dir_all(root.join(".engine").join("decisions")).expect("mkdir");
    std::fs::create_dir_all(root.join(".keel")).expect("mkdir");
    std::fs::write(root.join(".keel").join("actor"), "claudeOpus5\n").expect("actor");
    std::fs::write(root.join(".tracking").join("actors.sysml"), "package Actors {\n    private import EngineElement::*;\n    part hum : Person { :>> id = \"00000000-0000-4000-8000-000000000101\"; :>> title = \"hum\"; }\n}\n").expect("actors");
    let dec = root.join(".engine").join("decisions").join("0001-probe.sysml");
    std::fs::write(&dec, decision("proposed")).expect("decision");
    let backlog = root.join(".tracking").join("backlog.sysml");
    std::fs::write(
        &backlog,
        "package Fx {\n    private import EngineElement::*;\n    private import EngineWork::*;\n    private import EngineVerification::*;\n    private import EngineRelationships::*;\n\n    action def Build {\n        action dcFirst;\n        verification dcFirstDoD : Test { :>> id = \"00000000-0000-4000-8000-000000000011\"; :>> method = VerificationMethod::test; :>> procedureText = \"the predecessor\"; }\n        action dcSecond;\n        verification dcSecondDoD : Test { :>> id = \"00000000-0000-4000-8000-000000000012\"; :>> method = VerificationMethod::test; :>> procedureText = \"the dependant\"; }\n        action storyChartered;\n        verification storyCharteredDoD : Test { :>> id = \"00000000-0000-4000-8000-000000000013\"; :>> method = VerificationMethod::test; :>> procedureText = \"the chartered story\"; }\n    }\n    #DependsOn dependency from dcSecond to dcFirst;\n    #CharteredBy dependency from storyChartered to d0001;\n}\n",
    )
    .expect("seed");
    git(&root, &["init", "-q", "."]);
    git(&root, &["-c", "user.email=p@x", "-c", "user.name=p", "add", "-A"]);
    git(&root, &["-c", "user.email=p@x", "-c", "user.name=p", "commit", "-q", "-m", "seed"]);
    let seed = git(&root, &["rev-parse", "--short", "HEAD"]);

    // Known-positive: the predecessor is ready, the dependant is blocked on it and SAYS so, the
    // chartered story is blocked on the held Decision.
    let o = orient(&root);
    let ready = names(&o["ready"]);
    assert!(ready.contains(&"dcFirst".to_string()), "the predecessor is ready: {ready:?}");
    assert!(!ready.contains(&"dcSecond".to_string()), "an item whose predecessor is undone is not ready: {ready:?}");
    assert!(!ready.contains(&"storyChartered".to_string()), "a story chartered by a PROPOSED Decision is not ready: {ready:?}");
    let blocked = o["blocked"].as_array().cloned().unwrap_or_default();
    assert_eq!(blocked.len(), 1, "one item waits on another item: {blocked:?}");
    assert_eq!(blocked[0]["item"], "dcSecond");
    assert_eq!(blocked[0]["waitsOn"], "dcFirst");
    assert_eq!(blocked[0]["why"], "not done");
    assert_eq!(o["answerStatus"], "COMPUTED", "every narrowing filter ran: {o}");

    // The predecessor passes: the dependant is ready with no other edit.
    let out = Command::new(keel_bin())
        .args(["record", "result", "--file", ".tracking/backlog.sysml", "--task", "dcFirst", "--sha", &seed, "--verdict", "pass", "--judged-by", "claudeOpus5", "--judged-at", "2026-09-14", "--evidence", "probe"])
        .current_dir(&root)
        .env("KEEL_ACTOR", "claudeOpus5")
        .output()
        .expect("record result");
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    git(&root, &["-c", "user.email=p@x", "-c", "user.name=p", "add", "-A"]);
    git(&root, &["-c", "user.email=p@x", "-c", "user.name=p", "commit", "-q", "-m", "the pass"]);
    let o = orient(&root);
    let ready = names(&o["ready"]);
    assert!(ready.contains(&"dcSecond".to_string()), "the dependant is ready once its predecessor passes: {ready:?}");
    assert!(o["blocked"].as_array().is_some_and(Vec::is_empty), "nothing waits on an item any more: {}", o["blocked"]);
    assert!(!ready.contains(&"storyChartered".to_string()), "the charter is still held: {ready:?}");

    // The human accepts the charter: the story is ready with no other edit.
    std::fs::write(&dec, decision("accepted")).expect("accept");
    let ready = names(&orient(&root)["ready"]);
    assert!(ready.contains(&"storyChartered".to_string()), "a story chartered by an ACCEPTED Decision is ready: {ready:?}");
    let _ = std::fs::remove_dir_all(&root);
}
