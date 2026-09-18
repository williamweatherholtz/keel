//! `keel suite [ROOT] [-- <cargo test args>]` — run the full suite and record what that run cost.
//!
//! IT NO LONGER GATES ANYTHING (D0356). For one day a green receipt was required before any push.
//! Measured honestly the run costs about eleven wall minutes every time the code moves, against
//! roughly one bad push in twenty-five it could catch — and two of the three most recent failures
//! were platform faults this machine cannot reproduce. The owner withdrew it. What stays is the
//! measurement: the receipt is how anyone knows what a full run costs, which is the number that
//! decided the question.
//!
//! THE RECEIPT is machine-local (`.keel/metrics/suite-receipt.toml`, beside the hook fire-ledger):
//! the fingerprint of the deliverable as it was tested, the HEAD it was tested near, when, and the
//! counts. It is evidence about THIS machine's run and never travels — CI reruns the suite itself.
//!
//! THE FINGERPRINT is over the deliverable's CONTENT ON DISK, tracked or not: every workspace member
//! the root `Cargo.toml` lists (read, never typed - issue588 was the typed list stopping at `keel-cli`
//! while the members held their own tests), the embedded `.engine/`, `keelw`, and the two Cargo
//! manifests. The receipt carries one digest PER ROOT, so the staleness message names the member
//! whose edit moved it. It still answers "was this exact tree tested", which is worth knowing even
//! when nothing refuses on the answer. The run is `cargo test --release --workspace`: the receipt's
//! counts are the workspace's, the members' lib tests and the `harness = false` binaries included.
//!
//! THE RECEIPT SAYS WHAT WAS MEASURED (issue386). Two ways a run can end without measuring the code
//! are told apart from a red: launched from the very image `cargo test --release` relinks, the
//! command refuses BEFORE the running stub (on Windows the file is locked; the copy remedy is named);
//! and a cargo exit with no `test result:` line restores the previous receipt rather than writing
//! `fail - 0 passed, 0 failed` over a tree the tests never saw.

use sha2::{Digest as _, Sha256};
use std::path::{Path, PathBuf};

/// The deliverable paths NO workspace member owns, repo-relative: the embedded engine, the wrapper
/// script and the two manifests. The members themselves come from the root manifest
/// ([`deliverable_paths_of`]), never from a list typed here (issue588).
pub const UNOWNED_DELIVERABLE_PATHS: [&str; 4] = [".engine", "keelw", "Cargo.toml", "Cargo.lock"];

/// The deliverable's roots from a root manifest text: each `[workspace] members` entry in manifest
/// order, then [`UNOWNED_DELIVERABLE_PATHS`]. Pure.
#[must_use]
pub fn deliverable_paths_of(root_manifest: &str) -> Vec<String> {
    let mut paths = keel_model::corpus::workspace_members(root_manifest);
    paths.extend(UNOWNED_DELIVERABLE_PATHS.iter().map(|p| (*p).to_owned()));
    paths
}

/// The deliverable's roots of the repository at `repo`.
///
/// # Errors
/// When the root `Cargo.toml` cannot be read.
pub fn deliverable_paths(repo: &Path) -> Result<Vec<String>, String> {
    let root = std::fs::read_to_string(repo.join("Cargo.toml")).map_err(|e| format!("cannot read {}: {e}", repo.join("Cargo.toml").display()))?;
    Ok(deliverable_paths_of(&root))
}

/// One digest per deliverable root: `(root, SHA-256 over (path, content) of every file git knows or
/// would add under it, sorted by path)`, roots in [`deliverable_paths`] order, content read from DISK
/// so an uncommitted edit counts. A root with no file on disk digests the empty set (present, empty).
///
/// # Errors
/// When the root manifest cannot be read or git cannot list the tree.
pub fn fingerprint_parts(repo: &Path) -> Result<Vec<(String, String)>, String> {
    let roots = deliverable_paths(repo)?;
    let mut files: Vec<String> = Vec::new();
    for args in [vec!["ls-files", "-z", "--"], vec!["ls-files", "-z", "-o", "--exclude-standard", "--"]] {
        let mut a: Vec<&str> = args;
        a.extend(roots.iter().map(String::as_str));
        let out = keel_git::gitx::git().arg("-C").arg(repo).args(&a).output().map_err(|e| format!("git ls-files: {e}"))?;
        if !out.status.success() {
            return Err(format!("git ls-files failed: {}", String::from_utf8_lossy(&out.stderr).trim()));
        }
        files.extend(String::from_utf8_lossy(&out.stdout).split('\0').filter(|p| !p.is_empty()).map(str::to_owned));
    }
    files.sort();
    files.dedup();
    Ok(roots
        .iter()
        .map(|root| {
            let mut h = Sha256::new();
            for rel in files.iter().filter(|f| *f == root || f.starts_with(&format!("{root}/"))) {
                let Ok(bytes) = std::fs::read(repo.join(rel)) else { continue }; // deleted on disk: absent from the hash
                h.update(rel.as_bytes());
                h.update([0u8]);
                h.update(&bytes);
                h.update([0u8]);
            }
            (root.clone(), keel_actor::device::hex(&h.finalize()))
        })
        .collect())
}

/// The deliverable fingerprint over its parts: SHA-256 over `(root, digest)` in order. Pure.
#[must_use]
pub fn fingerprint_of(parts: &[(String, String)]) -> String {
    let mut h = Sha256::new();
    for (root, digest) in parts {
        h.update(root.as_bytes());
        h.update([0u8]);
        h.update(digest.as_bytes());
        h.update([0u8]);
    }
    keel_actor::device::hex(&h.finalize())
}

/// The deliverable fingerprint of the repository at `repo` ([`fingerprint_of`] its
/// [`fingerprint_parts`]).
///
/// # Errors
/// As [`fingerprint_parts`].
pub fn fingerprint(repo: &Path) -> Result<String, String> {
    Ok(fingerprint_of(&fingerprint_parts(repo)?))
}

/// The roots whose digest differs between two part lists, or that only one side carries - the
/// members a staleness message names. Pure; empty when both agree.
#[must_use]
pub fn moved_parts(before: &[(String, String)], now: &[(String, String)]) -> Vec<String> {
    let mut moved: Vec<String> = now
        .iter()
        .filter(|(root, digest)| before.iter().find(|(r, _)| r == root).is_none_or(|(_, d)| d != digest))
        .map(|(root, _)| root.clone())
        .collect();
    moved.extend(before.iter().filter(|(root, _)| !now.iter().any(|(r, _)| r == root)).map(|(root, _)| root.clone()));
    moved
}

/// Where the receipt lives, repo-relative.
pub const RECEIPT: &str = ".keel/metrics/suite-receipt.toml";

// The self-build predicate descended to the read model (sprint 733); re-exported so `crate::suite::is_self_build`
// keeps resolving.
pub use keel_model::corpus::is_self_build;

/// What the last suite run on this machine recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    pub fingerprint: String,
    pub head: String,
    pub at: u64,
    pub passed: u64,
    pub failed: u64,
    pub outcome: String,
    /// The run's wall clock; 0 on a running stub (issue472 - the touched receipt's word).
    pub seconds: u64,
    /// One `(root, digest)` per deliverable root as the run saw it (`[[part]]` rows); empty on a
    /// receipt written before the rows existed, and then no member can be named as moved.
    pub parts: Vec<(String, String)>,
}

impl Receipt {
    #[must_use]
    pub fn green(&self) -> bool {
        self.outcome == "pass" && self.failed == 0
    }
}

/// Parse a receipt's text (pure, tested).
#[must_use]
pub fn parse_receipt(text: &str) -> Option<Receipt> {
    let v = text.parse::<toml::Value>().ok()?;
    let s = |k: &str| v.get(k).and_then(toml::Value::as_str).map(str::to_owned);
    let n = |k: &str| v.get(k).and_then(toml::Value::as_integer).and_then(|i| u64::try_from(i).ok());
    let parts = v
        .get("part")
        .and_then(toml::Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter_map(|row| Some((row.get("path")?.as_str()?.to_owned(), row.get("digest")?.as_str()?.to_owned())))
                .collect()
        })
        .unwrap_or_default();
    Some(Receipt { fingerprint: s("fingerprint")?, head: s("head").unwrap_or_default(), at: n("at").unwrap_or(0), passed: n("passed").unwrap_or(0), failed: n("failed").unwrap_or(0), outcome: s("outcome").unwrap_or_else(|| "fail".into()), seconds: n("seconds").unwrap_or(0), parts })
}

fn render_receipt(r: &Receipt, log: &Path) -> String {
    use std::fmt::Write as _;
    let mut text = format!(
        "# suite receipt: the deliverable as the full suite last saw it ON THIS MACHINE, and what that run\n# cost. Nothing refuses on it (D0356) - it is a measurement, not a gate. `outcome = \"running\"` is the\n# stub written before cargo starts (D0387): a run in progress, or one that was killed - not an answer.\n# `at` is when this file was WRITTEN - the end of a done run, the start of a stub - and `seconds` the\n# run's wall clock, the same two words the touched receipt uses (issue472). `[[part]]` is one digest per\n# deliverable root - every workspace member the root manifest lists, then the paths no member owns -\n# so a later reader can name the member whose edit moved the fingerprint (issue588).\nfingerprint = \"{}\"\nhead = \"{}\"\nat = {}\npassed = {}\nfailed = {}\noutcome = \"{}\"\nseconds = {}\nlog = \"{}\"\n",
        r.fingerprint, r.head, r.at, r.passed, r.failed, r.outcome, r.seconds, log.to_string_lossy().replace('\\', "/")
    );
    for (root, digest) in &r.parts {
        let _ = write!(text, "\n[[part]]\npath = \"{root}\"\ndigest = \"{digest}\"\n");
    }
    text
}

/// Read this machine's receipt, if any.
#[must_use]
pub fn receipt(repo: &Path) -> Option<Receipt> {
    parse_receipt(&std::fs::read_to_string(repo.join(RECEIPT)).ok()?)
}

/// Sum `test result:` lines of a cargo test run (pure, tested).
#[must_use]
pub fn count_results(output: &str) -> (u64, u64) {
    let mut passed = 0u64;
    let mut failed = 0u64;
    for l in output.lines().filter(|l| l.starts_with("test result:")) {
        let words: Vec<&str> = l.split_whitespace().collect();
        for (i, w) in words.iter().enumerate() {
            if *w == "passed;" || *w == "passed" {
                passed += words.get(i.wrapping_sub(1)).and_then(|n| n.parse::<u64>().ok()).unwrap_or(0);
            }
            if *w == "failed;" || *w == "failed" {
                failed += words.get(i.wrapping_sub(1)).and_then(|n| n.parse::<u64>().ok()).unwrap_or(0);
            }
        }
    }
    (passed, failed)
}

/// Whether the last suite run covers the tree as it stands — `None` when it does (or there is no
/// suite), otherwise the reason it does not.
///
/// NOT WIRED TO ANYTHING SINCE D0356: `land` no longer consults it. Kept because "is this tree
/// tested" is a real question a human or a later control may want answered, and the answer is
/// cheap; deleting it would also delete the only place that knows what staleness looks like.
#[must_use]
pub fn land_refusal(repo: &Path) -> Option<String> {
    if !is_self_build(repo) {
        return None;
    }
    let now = match fingerprint_parts(repo) {
        Ok(p) => p,
        Err(e) => return Some(format!("the deliverable could not be fingerprinted ({e})")),
    };
    staleness(receipt(repo).as_ref(), &now)
}

/// The reason `receipt` does not cover a tree whose parts are `now`, or `None` when it does. Pure.
/// A CHANGED message names the roots that moved ([`moved_parts`]) - the member whose edit it was -
/// or says the receipt predates the rows when it carries none.
#[must_use]
pub fn staleness(receipt: Option<&Receipt>, now: &[(String, String)]) -> Option<String> {
    let now_fp = fingerprint_of(now);
    match receipt {
        None => Some(format!("no suite receipt at {RECEIPT} - the full suite has not run on this machine since the receipt existed. Run `keel suite` (it writes the receipt), then land.")),
        Some(r) if r.outcome == "running" => Some(format!("a suite run started at {} on this machine and has not completed (or was killed) - its receipt is a running stub, not a verdict. Wait for it or run `keel suite` again.", r.at)),
        Some(r) if !r.green() => Some(format!("the last suite run on this machine was RED ({} passed, {} failed; head {}). Fix, run `keel suite` to green, then land.", r.passed, r.failed, r.head)),
        Some(r) if r.fingerprint != now_fp => {
            let moved = if r.parts.is_empty() { "the receipt predates the per-root digests, so the moved member is not named".to_owned() } else { format!("moved: {}", moved_parts(&r.parts, now).join(", ")) };
            Some(format!("the deliverable CHANGED since the last green suite run (receipt {} at head {}, {} passed; the tree now fingerprints {}; {moved}). Run `keel suite`, then land.", &r.fingerprint[..12], r.head, r.passed, &now_fp[..12]))
        }
        Some(_) => None,
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// The directory `cargo test --release` writes for this repository: `CARGO_TARGET_DIR` when set,
/// else `<repo>/target`.
#[must_use]
pub fn target_dir(repo: &Path) -> PathBuf {
    std::env::var_os("CARGO_TARGET_DIR").map_or_else(|| repo.join("target"), PathBuf::from)
}

/// Is `exe` a file the release build REWRITES - `<target>/release/keel(.exe)` or anything under
/// `<target>/release/deps/`? Pure, so the collision is testable on a host that would not lock it.
///
/// issue386: on Windows a running image cannot be replaced, so a suite launched from the binary
/// cargo is about to relink fails with `Access is denied (os error 5)` before any test runs, and
/// used to record `fail - 0 passed, 0 failed` as if the code had been measured. Deliberately NARROW:
/// a copy at `<target>/release/keel-serve.exe` (the documented remedy, issue150) sits in the same
/// directory and does not collide, because cargo never writes it.
#[must_use]
pub fn image_collides(exe: &Path, target: &Path) -> bool {
    let release = target.join("release");
    let Ok(rel) = exe.strip_prefix(&release) else { return false };
    let mut parts = rel.components();
    let Some(first) = parts.next() else { return false };
    let first = first.as_os_str().to_string_lossy();
    if parts.next().is_none() {
        return first == format!("keel{}", std::env::consts::EXE_SUFFIX);
    }
    first == "deps"
}

/// The reason `who` (`keel suite`, `keel land`, ...) will not run cargo from this image, or `None`.
///
/// Refuses only where the lock is real (Windows); elsewhere the build replaces the file under a
/// running process without harm. Shared with the touched set (D0421), which links the same binaries.
#[must_use]
pub fn own_image_refusal(repo: &Path, who: &str) -> Option<String> {
    if !cfg!(windows) {
        return None;
    }
    let exe = std::env::current_exe().ok()?;
    let target = target_dir(repo);
    let exe_c = exe.canonicalize().unwrap_or_else(|_| exe.clone());
    let target_c = target.canonicalize().unwrap_or_else(|_| target.clone());
    if !(image_collides(&exe, &target) || image_collides(&exe_c, &target_c)) {
        return None;
    }
    let release = target.join("release");
    let copy = release.join(format!("keel-serve{}", std::env::consts::EXE_SUFFIX));
    Some(format!(
        "{who}: this command is running from {} - the very file `cargo test --release` relinks. On this host a running image cannot be replaced, so the build would fail with `Access is denied` before any test ran (issue386). Run it from a copy cargo does not write:\n  cp {} {}\n  {} {}\nNo receipt was written: nothing was measured.",
        exe.display(),
        release.join(format!("keel{}", std::env::consts::EXE_SUFFIX)).display(),
        copy.display(),
        copy.display(),
        who.strip_prefix("keel ").unwrap_or(who)
    ))
}

/// `keel suite [-- <cargo test args>]`: run the full suite, write the log and the receipt, exit as
/// cargo did. Always `--no-fail-fast`, so the receipt's counts are the whole population.
#[must_use]
pub fn cmd(args: &[String], repo: &Path) -> i32 {
    // --help must not RUN the suite. It did, once, and cost 185 seconds to discover.
    if args.iter().take_while(|a| *a != "--").any(|a| a == "--help" || a == "-h") {
        println!("usage: keel suite [ROOT] [--touched] [-- <cargo test args>]");
        println!("  runs the full suite (--release --workspace --no-fail-fast: every member's tests), logs under");
        println!("  .keel/metrics/, and writes {RECEIPT}: the deliverable fingerprint - one [[part]] digest per");
        println!("  workspace member the root manifest lists plus .engine, keelw and the manifests - counts and outcome.");
        println!("  --touched: instead run ONLY the integration tests that name a module changed since the base");
        println!("  of the push (origin/<branch>, else the last suite receipt's head, else HEAD~1) and write");
        println!("  {}: the base, stems, set and cost - an empty set is recorded too (D0421).", crate::touched::RECEIPT);
        println!("  A binary of the set observed green at the current content (the code, and the tree too when the");
        println!("  test reads this repository) is SKIPPED; --no-receipt or KEEL_NO_RECEIPT=1 runs every one (D0474).");
        println!("  A workspace member's unit tests are in the set (row lib:<member>) when a changed source is that");
        println!("  member's or one it depends on, keyed on that scope's code; a test that reads the repository only");
        println!("  through keel_fs::test_support is keyed on the paths it recorded reading (reads, reads_key) (D0481).");
        println!("  The set runs under cargo-nextest, binaries in parallel, and the receipt carries every test's");
        println!("  duration ([[timing]], slowest first); the harness = false cucumber binaries run under cargo test");
        println!("  in a second invocation. Without nextest the whole set runs that way and `runner` says so (D0475).");
        println!("  One touched run at a time per tree: the running stub names its writer (pid) and a second launch is");
        println!("  refused (exit 2, nothing written) while that process is alive; a dead writer's stub is replaced (issue569).");
        return 0;
    }
    if args.iter().take_while(|a| *a != "--").any(|a| a == "--touched") {
        return crate::touched::cmd(repo, keel_fs::fsx::no_receipt_forced(args));
    }
    if !is_self_build(repo) {
        eprintln!("keel suite: {} holds no keel-cli/Cargo.toml - there is no suite to run here (a downstream project's gate is `keel gate`)", repo.display());
        return 2;
    }
    let extra: Vec<&String> = args.iter().skip_while(|a| *a != "--").skip(1).collect();
    let metrics = repo.join(".keel").join("metrics");
    if let Err(e) = std::fs::create_dir_all(&metrics) {
        eprintln!("keel suite: cannot create {}: {e}", metrics.display());
        return 1;
    }
    if let Some(reason) = own_image_refusal(repo, "keel suite") {
        eprintln!("{reason}");
        return 2;
    }
    let started = now_secs();
    let log = metrics.join(format!("suite-{started}.log"));
    // Kept so a run that never reaches a test can put it back: a receipt says what was MEASURED, and
    // a build failure measured nothing (issue386).
    let previous = std::fs::read_to_string(repo.join(RECEIPT)).ok();
    // D0387/issue399: the previous receipt is REPLACED by a running stub before cargo starts, so a run
    // that is killed leaves `outcome = "running"` - not green, not counted - rather than the last
    // completed run's verdict standing over a tree it never saw. Same fingerprint as the final receipt
    // will carry, so a reader comparing fingerprints is told the run is in progress, not stale.
    let head = keel_git::gitx::git().arg("-C").arg(repo).args(["rev-parse", "--short", "HEAD"]).output().ok().filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    if let Ok(parts) = fingerprint_parts(repo) {
        let stub = Receipt { fingerprint: fingerprint_of(&parts), head: head.clone(), at: started, passed: 0, failed: 0, outcome: "running".to_string(), seconds: 0, parts };
        if let Err(e) = keel_fs::fsx::write_atomic(&repo.join(RECEIPT), render_receipt(&stub, &log)) {
            eprintln!("keel suite: running stub could not be written: {e}");
        }
    }
    // --workspace: every member's tests, the harness = false binaries included - the receipt's counts
    // are the workspace's, not keel-cli's alone (issue588).
    println!("keel suite: cargo test --release --workspace --no-fail-fast (log -> {})", log.display());
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("test").arg("--release").arg("--workspace").arg("--manifest-path").arg(repo.join("Cargo.toml")).arg("--no-fail-fast");
    for a in extra {
        cmd.arg(a);
    }
    let out = match cmd.current_dir(repo).output() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("keel suite: cargo could not be run: {e}");
            return 2;
        }
    };
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    let _ = std::fs::write(&log, &text);
    let (passed, failed) = count_results(&text);
    if never_ran(out.status.success(), passed, failed) {
        // The build (or cargo itself) failed before a single test binary reported: no verdict about
        // the code exists, so none is recorded. The previous receipt stands as what was last measured.
        match previous {
            Some(p) => {
                if let Err(e) = keel_fs::fsx::write_atomic(&repo.join(RECEIPT), p) {
                    eprintln!("keel suite: previous receipt could not be restored: {e}");
                }
            }
            None => {
                let _ = std::fs::remove_file(repo.join(RECEIPT));
            }
        }
        for l in text.lines().filter(|l| l.starts_with("error")).take(5) {
            eprintln!("  {l}");
        }
        eprintln!("keel suite: cargo exited {} before any test ran - the deliverable was NOT measured, no receipt written (log -> {})", out.status.code().map_or_else(|| "by signal".to_string(), |c| c.to_string()), log.display());
        return 2;
    }
    let outcome = if out.status.success() && failed == 0 { "pass" } else { "fail" };
    let parts = match fingerprint_parts(repo) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("keel suite: ran ({passed} passed, {failed} failed) but the deliverable could not be fingerprinted: {e} - no receipt written");
            return if outcome == "pass" { 1 } else { 101 };
        }
    };
    let r = done_receipt(parts, head, started, now_secs(), passed, failed, outcome);
    if let Err(e) = keel_fs::fsx::write_atomic(&repo.join(RECEIPT), render_receipt(&r, &log)) {
        eprintln!("keel suite: receipt could not be written: {e}");
    }
    for l in text.lines().filter(|l| l.contains("FAILED") || l.contains("panicked at")).take(20) {
        println!("  {l}");
    }
    println!("keel suite: {outcome} - {passed} passed, {failed} failed; receipt {} (fingerprint {}...)", RECEIPT, &r.fingerprint[..12]);
    if outcome == "pass" { 0 } else { 101 }
}

/// The receipt a finished run writes: `at` is `finished` - the moment of the write, the touched
/// receipt's meaning - and `seconds` the wall clock since `started` (issue472). Pure, tested.
#[must_use]
pub fn done_receipt(parts: Vec<(String, String)>, head: String, started: u64, finished: u64, passed: u64, failed: u64, outcome: &str) -> Receipt {
    Receipt { fingerprint: fingerprint_of(&parts), head, at: finished, passed, failed, outcome: outcome.to_string(), seconds: finished.saturating_sub(started), parts }
}

/// Did cargo fail without a single `test result:` line - a build or tool failure, not a verdict?
/// Pure: `(cargo succeeded, passed, failed)`.
#[must_use]
pub const fn never_ran(cargo_ok: bool, passed: u64, failed: u64) -> bool {
    !cargo_ok && passed == 0 && failed == 0
}

/// A path for tests to plant a receipt.
#[must_use]
pub fn receipt_path(repo: &Path) -> PathBuf {
    repo.join(RECEIPT)
}

#[cfg(test)]
mod tests {
    use super::{count_results, deliverable_paths, deliverable_paths_of, done_receipt, fingerprint_of, fingerprint_parts, image_collides, moved_parts, never_ran, parse_receipt, render_receipt, staleness, Receipt, UNOWNED_DELIVERABLE_PATHS};
    use std::path::Path;

    /// THE CONTROL for issue386, meaningful on any host: the image cargo relinks collides, the
    /// documented copy beside it does not, and a run with no `test result:` line is not a verdict.
    #[test]
    fn the_suite_knows_its_own_image_and_a_run_that_never_ran() {
        let target = Path::new("repo").join("target");
        let release = target.join("release");
        let keel = format!("keel{}", std::env::consts::EXE_SUFFIX);
        assert!(image_collides(&release.join(&keel), &target), "the binary cargo writes");
        assert!(image_collides(&release.join("deps").join("keel-0123abcd.exe"), &target), "a test binary under deps");
        assert!(!image_collides(&release.join(format!("keel-serve{}", std::env::consts::EXE_SUFFIX)), &target), "the issue150 copy is what the remedy names");
        assert!(!image_collides(&target.join("debug").join(&keel), &target), "the debug build is not relinked by --release");
        assert!(!image_collides(Path::new("elsewhere").join(&keel).as_path(), &target), "an installed keel");
        assert!(never_ran(false, 0, 0), "cargo failed and nothing reported: not a verdict");
        assert!(!never_ran(false, 3, 1), "a real red is a verdict");
        assert!(!never_ran(true, 0, 0), "cargo succeeded with an empty filter: measured, trivially");
    }

    #[test]
    fn results_are_summed_across_every_test_binary() {
        let out = "test result: ok. 12 passed; 0 failed; 0 ignored\nnoise\ntest result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured\n";
        assert_eq!(count_results(out), (15, 1));
        assert_eq!(count_results("no results"), (0, 0));
    }

    #[test]
    fn a_receipt_round_trips_and_a_red_one_is_not_green() {
        let r = parse_receipt("fingerprint = \"abc\"\nhead = \"1234567\"\nat = 5\npassed = 10\nfailed = 0\noutcome = \"pass\"\n").expect("parses");
        assert!(r.green() && r.fingerprint == "abc" && r.passed == 10);
        let red = parse_receipt("fingerprint = \"abc\"\npassed = 9\nfailed = 1\noutcome = \"fail\"\n").expect("parses");
        assert!(!red.green());
        assert!(parse_receipt("nonsense = ").is_none());
    }

    #[test]
    fn a_running_stub_is_not_green_and_a_killed_run_leaves_it() {
        // D0387/issue399: the stub `cmd` writes before cargo starts is what a killed run leaves behind;
        // it must read as no verdict, never as the previous run's pass.
        let stub = Receipt { fingerprint: "abc".into(), head: "1234567".into(), at: 7, passed: 0, failed: 0, outcome: "running".into(), seconds: 0, parts: Vec::new() };
        let text = render_receipt(&stub, Path::new("x.log"));
        let back = parse_receipt(&text).expect("parses");
        assert_eq!(back, stub);
        assert!(!back.green(), "a run in progress has no verdict");
        assert!(text.contains("running"), "the file says so in its own text");
    }

    #[test]
    fn a_done_receipt_is_stamped_when_written_and_carries_the_run_seconds() {
        // issue472 known-positive: started=100, finished=700 -> at=700 (the write), seconds=600. Before
        // this `at` was `started`, one second after launch on every run, so a reader told to check `at`
        // against the launch epoch saw the run's identity and never its completion.
        let r = done_receipt(vec![("keel-cli".into(), "abc".into())], "1234567".into(), 100, 700, 5, 0, "pass");
        assert_eq!((r.at, r.seconds), (700, 600));
        let text = render_receipt(&r, Path::new("x.log"));
        assert!(text.contains("at = 700\n") && text.contains("seconds = 600\n"), "{text}");
        assert_eq!(parse_receipt(&text).expect("parses"), r, "round-trips with the field");
    }

    #[test]
    fn a_running_stub_is_stamped_at_its_start_with_no_seconds() {
        // issue472 known-negative: the stub is written BEFORE cargo starts, so its `at` IS the start
        // and it has run for no time - the only receipt whose `at` equals the launch.
        let stub = Receipt { fingerprint: "abc".into(), head: "1234567".into(), at: 100, passed: 0, failed: 0, outcome: "running".into(), seconds: 0, parts: Vec::new() };
        let text = render_receipt(&stub, Path::new("x.log"));
        assert!(text.contains("at = 100\n") && text.contains("seconds = 0\n"), "{text}");
        // and a receipt from before the field existed still parses, reading 0
        let old = parse_receipt("fingerprint = \"f\"\nhead = \"h\"\nat = 5\npassed = 1\nfailed = 0\noutcome = \"pass\"\n").expect("parses");
        assert_eq!(old.seconds, 0);
    }

    /// issue588: the deliverable's roots are READ from the root manifest - every member it lists, in
    /// its order, then the four paths no member owns - never a list typed beside the code that stopped
    /// at keel-cli while eighteen members held their own tests.
    #[test]
    fn the_deliverable_is_every_member_the_root_manifest_lists_plus_what_no_member_owns() {
        let manifest = "[workspace]\nmembers = [\n    \"keel-parser\",\n    \"members/keel-fs\",\n    \"keel-cli\",\n]\nresolver = \"2\"\n";
        assert_eq!(
            deliverable_paths_of(manifest),
            vec!["keel-parser", "members/keel-fs", "keel-cli", ".engine", "keelw", "Cargo.toml", "Cargo.lock"],
            "members first in manifest order, then the unowned paths"
        );
        assert_eq!(deliverable_paths_of("[package]\nname = \"x\"\n"), UNOWNED_DELIVERABLE_PATHS.iter().map(|p| (*p).to_owned()).collect::<Vec<_>>(), "no members list: the unowned paths alone");
        // The tree this test runs in: every listed member is a root, and keel-cli is no longer the only one.
        let here = deliverable_paths(&keel_fs::test_support::repo_root()).expect("this repository's manifest");
        assert!(here.iter().any(|p| p == "members/keel-fs") && here.iter().any(|p| p == "keel-parser") && here.iter().any(|p| p == "keel-cli"), "{here:?}");
        assert!(here.len() > 10, "the workspace's members, not a typed handful: {}", here.len());
    }

    /// issue588 known-positive, chosen before the tree is read: an edit under a member's `src/`
    /// moves that member's part alone, the fingerprint with it, and the CHANGED message names the
    /// member. Known-negative: files under `target/` and `.keel/` - outside every root - move nothing.
    #[test]
    fn a_member_edit_moves_its_own_part_and_the_staleness_names_it() {
        let dir = std::env::temp_dir().join(format!("keel-suite-fp-{}", keel_model::ident::gen_uuid()));
        for sub in ["members/a/src", "members/b/src", "keel-cli/src", ".engine", "target/release", ".keel/metrics"] {
            std::fs::create_dir_all(dir.join(sub)).unwrap();
        }
        let git = |args: &[&str]| {
            let o = keel_git::gitx::git().arg("-C").arg(&dir).args(args).output().unwrap();
            assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
        };
        git(&["init", "-q"]);
        std::fs::write(dir.join("Cargo.toml"), "[workspace]\nmembers = [\n    \"members/a\",\n    \"members/b\",\n    \"keel-cli\",\n]\n").unwrap();
        std::fs::write(dir.join("Cargo.lock"), "# lock\n").unwrap();
        std::fs::write(dir.join("keelw"), "#!/bin/sh\n").unwrap();
        std::fs::write(dir.join("members/a/src/lib.rs"), "pub fn a() {}\n").unwrap();
        std::fs::write(dir.join("members/b/src/lib.rs"), "pub fn b() {}\n").unwrap();
        std::fs::write(dir.join("keel-cli/src/main.rs"), "fn main() {}\n").unwrap();
        std::fs::write(dir.join(".engine/x.sysml"), "package X;\n").unwrap();
        std::fs::write(dir.join(".gitignore"), "target/\n.keel/\n").unwrap();
        git(&["add", "-A"]);
        let before = fingerprint_parts(&dir).expect("parts");
        assert_eq!(before.iter().map(|(r, _)| r.as_str()).collect::<Vec<_>>(), vec!["members/a", "members/b", "keel-cli", ".engine", "keelw", "Cargo.toml", "Cargo.lock"]);
        let green = done_receipt(before.clone(), "1234567".into(), 1, 2, 9, 0, "pass");
        assert_eq!(staleness(Some(&green), &before), None, "the receipt covers the tree it was written over");

        // known-negative: outside every root, uncommitted and ignored - the deliverable did not move
        std::fs::write(dir.join("target/release/keel.exe"), "binary\n").unwrap();
        std::fs::write(dir.join(".keel/metrics/x-receipt.toml"), "at = 1\n").unwrap();
        assert_eq!(fingerprint_parts(&dir).expect("parts"), before, "target/ and .keel/ are outside the deliverable");

        // known-positive: one member's source, uncommitted - that part moves, the others stay, the message names it
        std::fs::write(dir.join("members/a/src/lib.rs"), "pub fn a() { /* edited */ }\n").unwrap();
        let after = fingerprint_parts(&dir).expect("parts");
        assert_eq!(moved_parts(&before, &after), vec!["members/a".to_owned()]);
        assert_ne!(fingerprint_of(&before), fingerprint_of(&after));
        let why = staleness(Some(&green), &after).expect("the deliverable changed");
        assert!(why.contains("CHANGED") && why.contains("moved: members/a") && !why.contains("members/b"), "{why}");

        // a receipt from before the rows existed cannot name the member, and says so
        let old = Receipt { parts: Vec::new(), ..green };
        let why = staleness(Some(&old), &after).expect("still changed");
        assert!(why.contains("predates the per-root digests"), "{why}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_receipt_carries_one_row_per_root_and_round_trips_them() {
        let parts = vec![("members/keel-fs".to_owned(), "d1".to_owned()), ("keel-cli".to_owned(), "d2".to_owned())];
        let r = done_receipt(parts.clone(), "1234567".into(), 100, 700, 5, 0, "pass");
        let text = render_receipt(&r, Path::new("x.log"));
        assert_eq!(text.matches("\n[[part]]\npath = ").count(), 2, "two rows (the header comment names the table once more): {text}");
        assert!(text.contains("path = \"members/keel-fs\"\ndigest = \"d1\"\n"), "{text}");
        let back = parse_receipt(&text).expect("parses");
        assert_eq!(back, r);
        assert_eq!(back.parts, parts);
        assert_eq!(back.fingerprint, fingerprint_of(&parts), "the fingerprint is the rows' digest, so a reader can recompute it");
        assert_eq!(moved_parts(&parts, &[("keel-cli".to_owned(), "d2".to_owned())]), vec!["members/keel-fs".to_owned()], "a root only one side carries is moved");
    }
}
