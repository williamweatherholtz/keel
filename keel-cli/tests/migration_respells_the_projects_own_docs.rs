//! A verb fold travels with migrate (D0523, issue620, GH#86-90).
//!
//! The downloaded 0.5.0, run over a tree `keel init` 0.4.1 had scaffolded, applied its resync and
//! then failed its own `cli-reference` guard on sixteen references in files the run never writes -
//! the adopter's CLAUDE.md as the 0.4.1 template wrote it, and the comments of the two contracts the
//! resync rightly keeps as project-owned - and rolled itself back, on words the engine itself had
//! written. Five verb folds each carried a committed transform over THIS repository's call sites;
//! none carried the fold downstream.
//!
//! These cases run the whole command, binary to tree, so the claim "migrate lands and the docs are
//! true" is observed rather than composed from unit parts (D0253).

use std::path::{Path, PathBuf};
use std::process::Command;

fn keel_bin() -> PathBuf {
    let mut p = std::env::current_exe().expect("test exe");
    p.pop();
    if p.ends_with("deps") {
        p.pop();
    }
    p.join(if cfg!(windows) { "keel.exe" } else { "keel" })
}

fn git(dir: &Path, args: &[&str]) -> bool {
    Command::new("git").arg("-C").arg(dir).args(args).output().is_ok_and(|o| o.status.success())
}

fn run(dir: &Path, args: &[&str]) -> (bool, String) {
    let out = Command::new(keel_bin()).args(args).current_dir(dir).output().expect("keel");
    (out.status.success(), format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)))
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// The lines of the 0.4.1 `keel init` CLAUDE.md template that the 0.5.0 guard failed, verbatim from
/// the adopter tree that reported GH#86-90. Everything a fold retired, in every position the guard
/// reads: a fenced block, backtick spans, a span carrying a root argument.
const CLAUDE_MD_0_4_1: &str = "# CLAUDE.md - how to work here

- **The `keel` CLI is the authority.** State is never read from prose - it is **computed**:
  `keel orient .` (where things stand / what's ready), `keel whats-next .`, `keel validate .`,
  `keel guard .`. Author facts via the write API (`keel add-task`, `keel append-result`, ...).
1. **Text is truth; everything derivable is a view.** **Never author a document, matrix,
   baseline, or report** - those are *computed views* (`keel report`, `keel render`).
5. **Validate before done.** A change is not done until `keel validate .` is clean and
   `keel guard .` passes.
  *ORIENT* -> `keel orient .`. When no process fits, define one - don't free-form.
- **Use the write API** (`keel add-task` / `append-result` / ...) - it enforces UUIDs.

```
keel validate .    # your .tracking facts parse clean (no ERROR)
keel guard .       # the honest-state guards pass
keel orient .      # where things stand + what's ready + the burndown
```
";

/// A committed project shaped like the one that reported the defect: scaffolded by `keel init`, its
/// living docs at the 0.4.1 spellings, a Cargo workspace at the root, and a stale pin so the
/// migration has real work to do.
fn adopter_at_0_4_1(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("keel-respell-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let out = Command::new(keel_bin()).args(["init"]).arg(&root).args(["--profile", "guided"]).output().expect("init");
    assert!(out.status.success(), "init: {}", String::from_utf8_lossy(&out.stderr));
    std::fs::write(root.join("CLAUDE.md"), CLAUDE_MD_0_4_1).expect("CLAUDE.md");
    std::fs::write(root.join("Cargo.toml"), "[workspace]\nresolver = \"2\"\nmembers = []\n").expect("Cargo.toml");
    // The two project-owned contracts, their comments as 0.4.1 wrote them.
    let policy = root.join(".engine/contracts/attestation-policy.toml");
    std::fs::write(&policy, format!("# This file is the policy; `keel guard attestation-authority` reads it.\n{}", read(&policy))).expect("policy");
    let actors = root.join(".engine/contracts/github-actors.toml");
    std::fs::write(&actors, format!("# Check yours with `keel github-decider <login>`.\n{}", read(&actors))).expect("actors");
    let pin = root.join(".engine/contracts/engine-version.toml");
    let stale = read(&pin)
        .lines()
        .map(|l| if l.trim_start().starts_with("engine") { "engine = \"0.4.1\"".to_string() } else { l.to_string() })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&pin, stale).expect("stale pin");
    assert!(git(&root, &["init", "-q"]), "git init");
    assert!(git(&root, &["config", "user.email", "p@e.invalid"]), "config");
    assert!(git(&root, &["config", "user.name", "probe"]), "config");
    assert!(git(&root, &["add", "-A"]), "stage");
    assert!(git(&root, &["-c", "commit.gpgsign=false", "commit", "-q", "-m", "scaffolded at 0.4.1"]), "commit");
    root
}

/// KNOWN-POSITIVE (D0388): the adopter's tree lands - no rollback - and every retired spelling in its
/// own docs now names the verb this binary dispatches. KNOWN-NEGATIVE: a second run plans no
/// `verb-respell` edit and leaves CLAUDE.md byte-identical.
#[test]
fn a_tree_scaffolded_at_0_4_1_lands_and_its_own_docs_name_todays_verbs() {
    let root = adopter_at_0_4_1("lands");
    let (ok, text) = run(&root, &["migrate", "."]);
    assert!(ok, "migrate must LAND on the adopter's tree, not roll back on its own 0.4.1 words: {text}");
    assert!(!text.contains("ROLLED BACK") && !text.contains("REVERTING"), "{text}");
    assert!(text.contains("[verb-respell]"), "the step is in the plan the human reads: {text}");
    assert!(text.contains("CLAUDE.md"), "and it names the file it rewrote: {text}");

    let claude = read(&root.join("CLAUDE.md"));
    for want in ["`keel show orient .`", "`keel show whats-next .`", "`keel gate validate .`", "`keel gate guard .`", "`keel record task`", "`keel record result`", "`keel render report`", "\nkeel show orient .", "\nkeel gate validate .", "\nkeel gate guard ."] {
        assert!(claude.contains(want), "expected {want} in the respelled CLAUDE.md:\n{claude}");
    }
    for gone in ["`keel orient", "`keel validate", "`keel guard .", "`keel add-task", "`keel append-result", "`keel report`", "\nkeel orient", "\nkeel validate", "\nkeel guard "] {
        assert!(!claude.contains(gone), "{gone} survived:\n{claude}");
    }
    assert!(claude.contains("`keel render`"), "a verb the binary still dispatches is left as written:\n{claude}");
    assert!(read(&root.join(".engine/contracts/attestation-policy.toml")).contains("`keel gate guard attestation-authority`"));
    assert!(read(&root.join(".engine/contracts/github-actors.toml")).contains("`keel github decider <login>`"));

    // The guard the 0.5.0 run failed is green on the tree the run left - that is what "lands" means.
    let (guard_ok, guard) = run(&root, &["gate", "guard", "cli-reference", "--no-receipt"]);
    assert!(guard_ok, "cli-reference must be green after the respell:\n{guard}");

    // Commit what landed - migrate refuses a dirty tree - then a second run has nothing to respell.
    assert!(git(&root, &["add", "-A"]) && git(&root, &["-c", "commit.gpgsign=false", "commit", "-q", "-m", "migrated"]), "commit the landed tree");
    let before = read(&root.join("CLAUDE.md"));
    let (ok2, again) = run(&root, &["migrate", "."]);
    assert!(ok2, "{again}");
    assert!(again.contains("Nothing to do"), "a second run plans no respell: {again}");
    assert_eq!(read(&root.join("CLAUDE.md")), before, "idempotent, byte for byte");
    let _ = std::fs::remove_dir_all(&root);
}

/// CLAUDE.md joined the run's scope for what it may now write, so the rollback restores it too. A
/// killed run leaves the marker and a half-rewritten CLAUDE.md; the next run recovers both.
#[test]
fn an_interrupted_run_restores_claude_md_with_the_rest() {
    let root = adopter_at_0_4_1("restore");
    let head = {
        let out = Command::new("git").arg("-C").arg(&root).args(["rev-parse", "HEAD"]).output().expect("git");
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };
    std::fs::create_dir_all(root.join(".keel")).expect("mkdir");
    std::fs::write(root.join(".keel").join("migrate-in-progress"), &head).expect("marker");
    std::fs::write(root.join("CLAUDE.md"), "# half-written by a killed run\n").expect("write");
    std::fs::write(root.join(".engine/contracts/unit-ids.toml"), "# half-written by a killed run\n").expect("write");

    let (_, text) = run(&root, &["migrate", "."]);
    assert!(text.contains("recovered"), "{text}");
    // The recovery restored CLAUDE.md to the pre-run commit, and the run that followed then respelled
    // it - so it is neither the killed run's stub nor the 0.4.1 text.
    let claude = read(&root.join("CLAUDE.md"));
    assert!(!claude.contains("half-written"), "CLAUDE.md is restored from the commit, not left as the killed run wrote it:\n{claude}");
    assert!(claude.contains("`keel show orient .`"), "and the run that recovered it then migrated it:\n{claude}");
    assert!(!read(&root.join(".engine/contracts/unit-ids.toml")).contains("half-written"));
    let _ = std::fs::remove_dir_all(&root);
}

/// The door: an uncommitted CLAUDE.md now refuses the run, as an uncommitted `.engine/` file always
/// has - a file the run rewrites and restores has to be clean going in, or the restore would discard
/// an edit the run never made.
#[test]
fn an_uncommitted_claude_md_is_refused_at_the_door() {
    let root = adopter_at_0_4_1("door");
    std::fs::write(root.join("CLAUDE.md"), format!("{CLAUDE_MD_0_4_1}\n- my own uncommitted line\n")).expect("write");
    let (ok, text) = run(&root, &["migrate", "."]);
    assert!(!ok, "a dirty CLAUDE.md must refuse: {text}");
    assert!(text.contains("CLAUDE.md"), "and the refusal names it: {text}");
    assert!(read(&root.join("CLAUDE.md")).contains("my own uncommitted line"), "the refusal wrote nothing");
    let _ = std::fs::remove_dir_all(&root);
}
