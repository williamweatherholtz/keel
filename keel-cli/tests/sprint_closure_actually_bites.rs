//! Arming probe for guard 55, `sprint-closure` (D0260) — the sprint483 class.
//!
//! The miss it exists to prevent: a sprint's task is finished and verified, its TestResult is
//! never appended, and the ready frontier therefore serves FINISHED work as OPEN work for weeks.
//! One occurrence in 496 sprints, found only when D0258's priority-assessment step first read the
//! frontier's head item by item.
//!
//! A guard that has never been observed to fail is a claim, not a control. This probe fires it in
//! the refuse case AND pins the exemption that keeps it from becoming the issue272 lockout — an
//! IN-PROGRESS sprint has unstamped tasks by definition and must still commit.

use std::path::{Path, PathBuf};

fn delivery(root: &Path) -> PathBuf {
    let d = root.join(".tracking").join("delivery");
    std::fs::create_dir_all(&d).expect("mkdir");
    d
}

/// A sprint file with one task, stamped or not.
fn sprint(root: &Path, n: u32, task: &str, stamped: bool) {
    let result = if stamped {
        format!("        part {task}DoDR1 : TestResult {{ :>> id = \"r{n}\"; :>> outcome = VerdictKind::pass; }}\n")
    } else {
        String::new()
    };
    let body = format!(
        "package S{n} {{\n    action def Run{n} {{\n        action {task};\n\
         \x20       verification {task}DoD : Test {{ :>> id = \"t{n}\"; }}\n{result}    }}\n}}\n"
    );
    std::fs::write(delivery(root).join(format!("sprint{n:03}_probe.sysml")), body).expect("write");
}

fn fresh(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("keel-closure-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join(".engine")).expect("mkdir");
    root
}

fn violations(root: &Path) -> Vec<String> {
    keel_cli::guards::sprint_closure(root).violations
}

// ── THE REFUSE CASE: the sprint483 reproduction ───────────────────────────────────────────────

#[test]
fn a_sprint_left_unstamped_while_work_moved_on_is_a_violation() {
    let root = fresh("bites");
    sprint(&root, 483, "storyLeftUnstamped", false);
    sprint(&root, 484, "storyMovedOn", true); // work moved on — 483 is no longer in progress
    let v = violations(&root);
    assert_eq!(
        v.len(),
        1,
        "an unstamped task in a sprint the work has MOVED ON from must be a violation — this is \
         exactly how finished work was served as ready for three weeks (sprint483): {v:?}"
    );
    assert!(
        v[0].contains("storyLeftUnstamped") && v[0].contains("sprint483"),
        "the violation names the task AND its sprint, or it cannot be acted on: {v:?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

// ── THE ALLOW CASE: the issue272 lesson — never lock out legitimate work ──────────────────────

#[test]
fn the_newest_sprint_may_be_unstamped_because_it_is_in_progress() {
    let root = fresh("inprogress");
    sprint(&root, 483, "storyClosed", true);
    sprint(&root, 484, "storyStillRunning", false); // the sprint being worked right now
    assert!(
        violations(&root).is_empty(),
        "the highest-numbered sprint is IN PROGRESS and has unstamped tasks by definition — \
         gating it would make the guard a lockout, the issue272 failure repeated"
    );
    let _ = std::fs::remove_dir_all(&root);
}

// ── THE DISCRIMINATION: it is the STAMP that matters, not the file's existence ────────────────

#[test]
fn stamping_the_task_clears_the_violation() {
    let root = fresh("clears");
    sprint(&root, 483, "storyLeftUnstamped", false);
    sprint(&root, 484, "storyMovedOn", true);
    assert_eq!(violations(&root).len(), 1, "precondition");
    sprint(&root, 483, "storyLeftUnstamped", true); // append the owed result
    assert!(
        violations(&root).is_empty(),
        "appending the TestResult must clear it — otherwise the guard reports a state no action \
         can fix, which is how a control gets bypassed rather than obeyed"
    );
    let _ = std::fs::remove_dir_all(&root);
}

// ── D0515: THE ITEM THE SPRINT DELIVERS (issue563 / issue589) ────────────────────────────────

/// A sprint file whose Story is stamped, recorded on `created_at`, delivering `item` by edge (or not).
fn delivering_sprint(root: &Path, n: u32, created_at: &str, item: Option<&str>) {
    let edge = item.map_or(String::new(), |i| format!("    #Delivers dependency from s{n}Story to {i};\n"));
    let body = format!(
        "package S{n} {{\n    #CharteredBy dependency from s{n}Story to d0001;\n{edge}\n\
         \x20   part s{n}Story : Story {{ :>> id = \"s{n}\"; :>> createdAt = \"{created_at}\"; }}\n\
         \x20   action def Run{n} {{\n        action storyS{n};\n\
         \x20       verification storyS{n}DoD : Test {{ :>> id = \"t{n}\"; }}\n\
         \x20       part storyS{n}DoDR1 : TestResult {{ :>> id = \"r{n}\"; :>> outcome = VerdictKind::pass; }}\n    }}\n}}\n"
    );
    std::fs::write(delivery(root).join(format!("sprint{n:03}_probe.sysml")), body).expect("write");
}

/// The backlog declaring `item` with its DoD, stamped or not - the result lives HERE, beside the DoD,
/// never in the delivery file.
fn backlog(root: &Path, item: &str, stamped: bool) {
    let result = if stamped {
        format!("        part {item}DoDR1 : TestResult {{ :>> id = \"b1\"; :>> outcome = VerdictKind::pass; }}\n")
    } else {
        String::new()
    };
    let body = format!(
        "package B {{\n    action def Backlog {{\n        action {item};\n\
         \x20       verification {item}DoD : Test {{ :>> id = \"bt\"; }}\n{result}    }}\n}}\n"
    );
    std::fs::create_dir_all(root.join(".tracking")).expect("mkdir");
    std::fs::write(root.join(".tracking").join("backlog.sysml"), body).expect("write backlog");
}

/// D0388 known-positive: sprint N delivers X, X's DoD has no result, sprint N+1 exists - red naming X
/// and the sprint. This is sprint 720 (issue563) and sprint 735 (issue589): the story stamped, the
/// delivered item left open, the frontier serving finished work as ready.
#[test]
fn a_delivered_item_left_open_while_work_moved_on_is_a_violation_naming_the_item() {
    let root = fresh("delivers-open");
    delivering_sprint(&root, 720, "2026-09-19", Some("dcDeliveredThing"));
    delivering_sprint(&root, 721, "2026-09-19", Some("dcDeliveredThing"));
    backlog(&root, "dcDeliveredThing", false);
    let v = violations(&root);
    assert_eq!(v.len(), 1, "sprint 720 delivers an open item and work moved on: {v:?}");
    assert!(
        v[0].contains("dcDeliveredThing") && v[0].contains("sprint720") && v[0].contains("D0515"),
        "the violation names the item AND its sprint: {v:?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// D0388 known-negative: the same fixture with X's DoD result recorded in the backlog is green - the
/// stamp is what clears it, and the guard reads it where it lives.
#[test]
fn a_delivered_item_with_its_dod_result_is_clean() {
    let root = fresh("delivers-stamped");
    delivering_sprint(&root, 720, "2026-09-19", Some("dcDeliveredThing"));
    delivering_sprint(&root, 721, "2026-09-19", Some("dcDeliveredThing"));
    backlog(&root, "dcDeliveredThing", true);
    let v = violations(&root);
    assert!(v.is_empty(), "the delivered item's DoD result clears it: {v:?}");
    let _ = std::fs::remove_dir_all(&root);
}

/// D0515 clause 2, both arms: a Story recorded on or after 2026-09-18 with no #Delivers edge is red
/// (the newest sprint included - the edge is authored at prep); one recorded before that date is history.
#[test]
fn a_sprint_recorded_after_d0515_names_what_it_delivers_and_earlier_ones_are_history() {
    let root = fresh("delivers-edge");
    backlog(&root, "dcDeliveredThing", true);
    delivering_sprint(&root, 700, "2026-09-17", None);
    delivering_sprint(&root, 701, "2026-09-18", None);
    let v = violations(&root);
    assert_eq!(v.len(), 1, "only the sprint recorded on/after the date owes an edge: {v:?}");
    assert!(v[0].contains("sprint701") && v[0].contains("#Delivers"), "{v:?}");
    delivering_sprint(&root, 701, "2026-09-18", Some("dcDeliveredThing"));
    assert!(violations(&root).is_empty(), "authoring the edge clears it");
    let _ = std::fs::remove_dir_all(&root);
}

// ── THE LIVE TREE: the ratchet starts clean, and that is asserted, not assumed ────────────────

#[test]
fn this_repository_is_clean_under_the_new_guard() {
    let root = keel_fs::test_support::repo_root();
    let report = keel_cli::guards::sprint_closure(&root);
    assert!(
        report.violations.is_empty(),
        "the guard is a RATCHET on an already-perfect record (496/496 sprints stamp every task); \
         a violation here means it was mis-scoped, not that the tree is dirty: {:?}",
        report.violations
    );
    assert!(report.scanned > 400, "must actually scan the corpus, not silently see nothing: {}", report.scanned);
    let _ = std::fs::remove_dir_all(root.join("nonexistent"));
}
