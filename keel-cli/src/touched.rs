//! THE TOUCHED TEST SET (D0421, issue416): the integration tests that NAME a module changed since the
//! base of the push, run before the first push - and nothing else.
//!
//! WHY THIS EXISTS. D0356 withdrew the full-suite push gate on the owner's measurement: ~11 wall
//! minutes every time code moved, against roughly one catchable bad push in twenty-five. The next
//! CI red after that (issue416) was a test whose text named the very module the commit had changed -
//! the one file a text match would have picked out in seconds. This is NOT the D0356 gate returning:
//! the whole suite is still `keel suite`, still measured, still gating nothing. This runs the subset a
//! commit can be expected to have touched, and says when that subset is empty.
//!
//! HOW THE SET IS COMPUTED - by text, deliberately. A changed path under `keel-cli/src/` or under a
//! workspace member's `members/<crate>/src/` (D0479, sprint 714) contributes its MODULE STEM
//! (`sync.rs` -> `sync`, `view/mod.rs` -> `view`, `members/keel-git/src/gitx.rs` -> `gitx`);
//! `main.rs` and `lib.rs` name no module and are reported as unattributed rather than matched against
//! every test that says `main`. The members' own unit tests run with the lib (`member_libs`), until
//! D0481 computes the set over the workspace graph.
//! An integration test is touched when its text carries a stem as a whole word (`keel_cli::sync::`,
//! `keel sync`, `sync.rs` all count; `synced` does not), or when the test file itself changed.
//! A changed path under `.engine/` contributes the stem `init` (`embedded_stem`): that tree is
//! embedded in the binary and `keel init` ships it, so every test that scaffolds a project reads
//! it - 58 of 71 on 2026-09-13, when a skill's check landed under `.engine/skills/`, the receipt
//! read 570/0 over an empty stem set, and CI went red on `init_smoke` (issue524). Over-inclusion
//! costs a test run; under-inclusion is the CI red this replaces, so the match is generous and pure
//! (`names_stem`, `touched_tests` - unit-tested below).
//!
//! WHAT IS RUN of the set (D0474, issue537). One touched run costs ~936 s on this host, and the same
//! one-line fix ran three times in sprint 705 - the third time cargo rebuilt in 1.08 s and the tests
//! ran 736 s, because nothing held a memo of the RUN. The receipt now records, per binary, the
//! content its outcome depends on (`contentkey`): `code`, the bytes compiled into or run by the
//! binaries, and `tree`, every path outside `.keel/` plus HEAD. A binary whose text names
//! `CARGO_MANIFEST_DIR` is SELF-READING - it reads this repository's own tree (eighteen of them on
//! 2026-09-14; the lib's unit tests always are) - and is keyed on both; every other binary is keyed on
//! `code` alone. A binary with a green observation at the current key is SKIPPED; the rest run, and
//! the run's greens join the table. The invariant is that every binary in the set was observed green
//! against the content its outcome depends on - not that this run produced the observation. Skipping
//! is a memo of a run, never a substitute for one: a red drops the entry, `--no-receipt` or
//! `KEEL_NO_RECEIPT=1` runs everything, and a tree that moved during the run records nothing.
//!
//! HOW IT RUNS (D0475, issue536). `cargo nextest run --release` over the same `--test` selection:
//! the binaries execute in parallel and every test is its own process, and nextest's per-test
//! lines are read into the receipt as `[[timing]]` rows, slowest first. Measured on the 63-binary
//! set of 2026-09-14: `cargo test` 1246 s wall (per-binary sum 913 s); nextest 723 s at 20 test
//! threads, 653 s at 8, 701 s at 4 - bounded below by ONE test, `view::tests::
//! report_produces_cards_and_rejects_unknown`, 347-690 s under contention against a whole lib
//! binary of 145 s serially (issue540). keel-cli's three `harness = false` cucumber binaries cannot answer
//! nextest's `--list` and run under `cargo test` in a second invocation; a host with no nextest
//! runs everything that way and the receipt's `runner` says so. `KEEL_PERF` is scrubbed from both
//! children (issue539): the operator's timing lines would land after the JSON the tests parse.
//!
//! THE BASE is `origin/<branch>` when it resolves (what the push will land on), else the head the
//! last suite receipt recorded, else `HEAD~1`; a tree with none of those reads every tracked module
//! as changed and says so. THE CHANGED SET is measured from the merge-base to the WORKING TREE -
//! tracked edits and untracked files alike - not to HEAD: the D0425 verifier runs this BEFORE the
//! commit, and on 2026-09-10 (sprint 659, HEAD equal to origin/main, twenty files edited under
//! `keel-cli/`) `base...HEAD` read an empty set and the verifier ran nothing (issue463). Inside the
//! post-commit land the working tree IS HEAD, so nothing changes there.
//!
//! WHERE IT IS INERT (D0337). A refusal on `land` is a change to the integration path, which is outside
//! standing consent - so `land` computes and PRINTS the set on every push, but RUNS it and refuses only
//! once D0421 carries the human's acceptance (`d0421AcceptR1` in its decision file, the D0338 pattern).
//! `keel suite --touched` runs the set on demand regardless: an explicit command is the caller's word.
//!
//! THE RECEIPT is machine-local, beside the suite's (`.keel/metrics/touched-receipt.toml`): the base,
//! the stems, the tests, what they cost, the outcome, and the `[[observed]]` table - an empty set is a
//! receipt too, and carries the table forward. The receipt files are pre-write protected surfaces
//! (`claude_surface::PROTECTED_PATHS`): a memo a tool could Write would be an honour system.

use std::path::{Path, PathBuf};

pub const RECEIPT: &str = ".keel/metrics/touched-receipt.toml";

/// The name the lib's own unit tests carry in the set, the receipt and cargo's rerun hint.
pub const LIB: &str = "lib";

/// The env var a test reads to reach this repository's own tree; a test whose text names it is
/// self-reading (D0474).
const SELF_READING_MARK: &str = "CARGO_MANIFEST_DIR";

/// The path of a Rust source file relative to its crate's `src/`, for the crates whose modules the
/// set is keyed on: `keel-cli/src/` and every `members/<crate>/src/` (D0479). `None` elsewhere.
fn source_rel(p: &str) -> Option<&str> {
    if let Some(rel) = p.strip_prefix("keel-cli/src/") {
        return Some(rel);
    }
    let rest = p.strip_prefix("members/")?;
    let (crate_dir, rel) = rest.split_once("/src/")?;
    (!crate_dir.is_empty() && !crate_dir.contains('/')).then_some(rel)
}

/// The module stem a changed path contributes, or `None` for a path that names no module.
///
/// Pure: `keel-cli/src/sync.rs` -> `sync`; `keel-cli/src/view/mod.rs` -> `view`;
/// `members/keel-git/src/gitx.rs` -> `gitx`; `keel-cli/src/main.rs`, any `lib.rs`, anything outside
/// a keyed crate's `src/` -> `None`.
#[must_use]
pub fn module_stem(path: &str) -> Option<String> {
    let p = path.replace('\\', "/");
    let rel = source_rel(&p)?;
    let file = rel.strip_suffix(".rs")?;
    let mut parts: Vec<&str> = file.split('/').collect();
    let last = parts.pop()?;
    let stem = if last == "mod" { *parts.last()? } else { last };
    if stem == "main" || stem == "lib" || stem.is_empty() {
        return None;
    }
    Some(stem.to_string())
}

/// The stem a changed path under the EMBEDDED tree contributes: `init`, the command that ships it.
///
/// `.engine/**` is compiled into the binary (`embedded::ENGINE_DIR`) and every test that runs `keel init`
/// reads it, so a change there touches those tests as surely as a change to `init`'s own source would.
/// Pure: `.engine/skills/x/references/check.py` -> `init`; `.engine/` alone, `.tracking/x.sysml`,
/// `keel-cli/src/init.rs` -> `None` (the last is `module_stem`'s to name).
#[must_use]
pub fn embedded_stem(path: &str) -> Option<String> {
    let p = path.replace('\\', "/");
    let rel = p.strip_prefix(".engine/")?;
    if rel.is_empty() {
        return None;
    }
    Some("init".to_string())
}

/// Is this a changed path that names no module but is still deliverable code (`main.rs`, `lib.rs`)?
#[must_use]
pub fn is_unattributed_source(path: &str) -> bool {
    let p = path.replace('\\', "/");
    source_rel(&p).is_some() && std::path::Path::new(&p).extension().is_some_and(|x| x.eq_ignore_ascii_case("rs")) && module_stem(&p).is_none()
}

/// The test name an integration-test path contributes (`keel-cli/tests/land_gate.rs` -> `land_gate`).
#[must_use]
pub fn test_name(path: &str) -> Option<String> {
    let p = path.replace('\\', "/");
    let rel = p.strip_prefix("keel-cli/tests/")?;
    if rel.contains('/') {
        return None; // a helper under tests/<dir>/ is not a test binary
    }
    rel.strip_suffix(".rs").map(str::to_string)
}

/// Does `text` carry `stem` as a whole word? A word character is `[A-Za-z0-9_]`, so `keel_cli::sync::`
/// and `keel sync` and `sync.rs` all name `sync`, and `synced` / `resync` do not.
#[must_use]
pub fn names_stem(text: &str, stem: &str) -> bool {
    if stem.is_empty() {
        return false;
    }
    let bytes = text.as_bytes();
    let is_word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let mut from = 0;
    while let Some(i) = text[from..].find(stem) {
        let start = from + i;
        let end = start + stem.len();
        let before_ok = start == 0 || bytes.get(start - 1).is_none_or(|b| !is_word(*b));
        let after_ok = bytes.get(end).is_none_or(|b| !is_word(*b));
        if before_ok && after_ok {
            return true;
        }
        from = start + 1;
    }
    false
}

/// The touched set, pure: every test whose text names a changed stem, plus every test that changed
/// itself. Sorted, deduplicated. `tests` is `(name, text)`.
#[must_use]
pub fn touched_tests(tests: &[(String, String)], stems: &[String], changed_tests: &[String]) -> Vec<String> {
    let mut out: Vec<String> = tests
        .iter()
        .filter(|(name, text)| changed_tests.contains(name) || stems.iter().any(|s| names_stem(text, s)))
        .map(|(name, _)| name.clone())
        .collect();
    out.sort();
    out.dedup();
    out
}

/// The self-reading tests, pure (D0474).
///
/// Every test whose text names `CARGO_MANIFEST_DIR` - the one way the eighteen that read this
/// repository's own tree reach it - plus `lib`, whose unit tests read live facts by construction
/// (`adherence.rs`, `cli_surface_declared_tests` among them). Sorted. A second idiom, when one
/// appears, is added HERE, not to a reminder.
#[must_use]
pub fn self_reading_tests(tests: &[(String, String)]) -> Vec<String> {
    let mut out: Vec<String> = tests.iter().filter(|(_, text)| text.contains(SELF_READING_MARK)).map(|(name, _)| name.clone()).collect();
    out.push(LIB.to_string());
    out.sort();
    out.dedup();
    out
}

/// The test binaries cargo reported as failed (pure, over cargo's captured output).
///
/// Read from cargo's OWN attribution on stderr - `error: test failed, to rerun pass \`--test <name>\``
/// and the `--no-fail-fast` summary `error: N targets failed:` followed by one \`--test <name>\` per
/// line. The first cut paired each `test result: FAILED` line with the `Running tests/<name>.rs`
/// header before it; that pairing holds on a terminal, where the two streams interleave, and NOT in
/// a capture, where stdout (the results) is read whole before stderr (the headers) - the first live
/// run recorded `failed = 1` and `failing = []`. Cargo's rerun hint is the one line that carries the
/// name and the verdict together.
#[must_use]
pub fn failing_binaries(cargo_output: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in cargo_output.lines() {
        // The lib's hint is `--lib`, with no name: it is reported as `lib`.
        if line.contains("`--lib`") && !out.iter().any(|n| n == LIB) {
            out.push(LIB.to_string());
        }
        let mut rest = line;
        while let Some(i) = rest.find("`--test ") {
            let after = &rest[i + "`--test ".len()..];
            if let Some(end) = after.find('`') {
                let name = after[..end].trim();
                if !name.is_empty() && !out.iter().any(|n| n == name) {
                    out.push(name.to_string());
                }
                rest = &after[end + 1..];
            } else {
                break;
            }
        }
    }
    out.sort();
    out
}

/// One green observation (D0474): `binary` passed at content `code` (and `tree`, which matters only
/// when the binary is self-reading), `at` seconds since the epoch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observed {
    pub binary: String,
    pub code: String,
    pub tree: String,
    pub at: u64,
}

/// The skip computation, pure (D0474): which binaries of `set` have a green observation at `keys`.
///
/// A binary is skipped when an observation names it at an equal `code` key and - if it is in
/// `self_reading` - an equal `tree` key. Both vectors come back sorted; together they are `set`.
#[must_use]
pub fn split(set: &[String], self_reading: &[String], observed: &[Observed], keys: &crate::contentkey::ContentKeys) -> (Vec<String>, Vec<String>) {
    let mut run = Vec::new();
    let mut skipped = Vec::new();
    for b in set {
        let green = observed.iter().any(|o| o.binary == *b && o.code == keys.code && (!self_reading.contains(b) || o.tree == keys.tree));
        if green {
            skipped.push(b.clone());
        } else {
            run.push(b.clone());
        }
    }
    run.sort();
    skipped.sort();
    (run, skipped)
}

/// The observation table after a run, pure.
///
/// Every binary that RAN loses its old entry; those that passed gain one at `keys` (when `keys` is
/// `Some` - the tree held still through the run); a binary that was not run keeps whatever it had. A
/// run cargo did not finish (`cargo_ok` false with no attributed failure - a build error, a killed
/// process) is nobody's green.
#[must_use]
pub fn merge_observed(prior: &[Observed], ran: &[String], failing: &[String], cargo_ok: bool, keys: Option<&crate::contentkey::ContentKeys>, at: u64) -> Vec<Observed> {
    let mut out: Vec<Observed> = prior.iter().filter(|o| !ran.contains(&o.binary)).cloned().collect();
    let attributed = cargo_ok || !failing.is_empty();
    if let Some(k) = keys {
        if attributed {
            for b in ran.iter().filter(|b| !failing.contains(b)) {
                out.push(Observed { binary: b.clone(), code: k.code.clone(), tree: k.tree.clone(), at });
            }
        }
    }
    out.sort_by(|a, b| a.binary.cmp(&b.binary));
    out
}

/// The `[[observed]]` table of the receipt at `repo`, empty when there is none or it does not parse -
/// no observation is never wrong, only slow.
#[must_use]
pub fn read_observed(repo: &Path) -> Vec<Observed> {
    let Ok(text) = std::fs::read_to_string(repo.join(RECEIPT)) else { return Vec::new() };
    parse_observed(&text)
}

/// Pure over the receipt's text.
#[must_use]
pub fn parse_observed(text: &str) -> Vec<Observed> {
    let Ok(v) = toml::from_str::<toml::Value>(text) else { return Vec::new() };
    let Some(rows) = v.get("observed").and_then(|o| o.as_array()) else { return Vec::new() };
    let s = |row: &toml::Value, k: &str| row.get(k).and_then(|x| x.as_str()).map(str::to_string);
    rows.iter()
        .filter_map(|row| {
            Some(Observed {
                binary: s(row, "binary")?,
                code: s(row, "code")?,
                tree: s(row, "tree")?,
                at: row.get("at").and_then(toml::Value::as_integer).and_then(|i| u64::try_from(i).ok()).unwrap_or(0),
            })
        })
        .collect()
}

/// What was computed for one push: the base compared against, the stems, and the set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Touched {
    pub base: String,
    pub stems: Vec<String>,
    pub unattributed: Vec<String>,
    pub tests: Vec<String>,
    /// The lib's own unit tests (`cargo test --lib`) are in the set whenever ANY `keel-cli/src` or
    /// `members/*/src` path changed - and `lib` then runs every workspace member's lib tests too
    /// (`member_libs`), because a moved module's `#[cfg(test)]` block moved with it (sprint 714):
    /// a module's `#[cfg(test)]` block names it by construction, and a unit test elsewhere
    /// can read the changed module's live facts - `cli_surface_declared_tests` read the suite
    /// synopsis and hardcoded which Decisions it may cite, and CI went red on cab7cac when the
    /// synopsis gained a citation the set never ran (issue438). ~15 s on this host.
    pub lib: bool,
    /// The self-reading binaries among ALL integration tests plus `lib` (D0474) - keyed on the tree
    /// as well as the code.
    pub self_reading: Vec<String>,
    /// Every path the working tree changed since the base (tracked edits and untracked crate files),
    /// repo-relative - what the eol refusal names first (issue478).
    pub changed: Vec<String>,
    /// The working tree's line endings against the attribute (issue478): every tracked path whose
    /// `.gitattributes` entry declares an ending and whose working copy holds another. Read ONCE, with
    /// the set, so `land` and `suite --touched` refuse on the same census before cargo compiles the
    /// bytes; `eol_scanned` / `eol_millis` are the census's population and cost.
    pub eol: Vec<crate::eol::Mismatch>,
    pub eol_scanned: usize,
    pub eol_millis: u64,
}

impl Touched {
    /// Nothing to run: no integration test names a changed module AND no source path changed.
    #[must_use]
    pub const fn nothing_to_run(&self) -> bool {
        self.tests.is_empty() && !self.lib
    }

    /// The whole set as binary names: the integration tests plus `lib` when it is in.
    #[must_use]
    pub fn binaries(&self) -> Vec<String> {
        let mut v = self.tests.clone();
        if self.lib {
            v.push(LIB.to_string());
        }
        v.sort();
        v
    }

    /// The eol refusal line, or `None` when every declared path holds its ending: the changed paths by
    /// name first, then the count of the rest (`eol::describe`).
    #[must_use]
    pub fn eol_refusal(&self) -> Option<String> {
        if self.eol.is_empty() {
            return None;
        }
        Some(crate::eol::describe(&self.eol, &self.changed))
    }

    /// The one line that says the census ran and what it cost.
    #[must_use]
    pub fn eol_line(&self) -> String {
        format!("working-tree eol: {} declared paths hold their ending ({} ms, git ls-files --eol)", self.eol_scanned, self.eol_millis)
    }
}

fn git_out(repo: &Path, args: &[&str]) -> Option<String> {
    let o = crate::gitx::git().arg("-C").arg(repo).args(args).output().ok()?;
    o.status.success().then(|| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// The base the changed set is measured from: `origin/<branch>`, else the last suite receipt's head,
/// else `HEAD~1`. `None` when nothing resolves (a one-commit repository with no remote).
fn base_ref(repo: &Path) -> Option<String> {
    let branch = git_out(repo, &["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_else(|| "HEAD".to_string());
    let remote = format!("origin/{branch}");
    if git_out(repo, &["rev-parse", "--verify", "--quiet", &format!("{remote}^{{commit}}")]).is_some() {
        return Some(remote);
    }
    if let Some(r) = crate::suite::receipt(repo) {
        if !r.head.is_empty() && git_out(repo, &["rev-parse", "--verify", "--quiet", &format!("{}^{{commit}}", r.head)]).is_some() {
            return Some(r.head);
        }
    }
    git_out(repo, &["rev-parse", "--verify", "--quiet", "HEAD~1^{commit}"]).map(|_| "HEAD~1".to_string())
}

/// Compute the touched set for `repo`: what the working tree changed since the base. Self-build only
/// (the caller checks).
///
/// # Errors
/// When git cannot list the changed paths (`diff --name-only` against the base, or `ls-files` when
/// no base resolves).
pub fn compute(repo: &Path) -> Result<Touched, String> {
    let (base, changed): (String, Vec<String>) = if let Some(b) = base_ref(repo) {
        // The merge-base, so a remote that moved ahead does not read as our change; then the diff
        // from it to the WORKING TREE (no second revision), plus the untracked files under the crate
        // AND under the embedded tree - a new module, test file or engine file is a change the diff of
        // tracked paths cannot see (sprint 699's new skill under `.engine/skills/` was untracked when
        // the verifier ran, so it was in neither list; issue524).
        let from = git_out(repo, &["merge-base", &b, "HEAD"]).unwrap_or_else(|| b.clone());
        let out = git_out(repo, &["diff", "--name-only", &from]).ok_or_else(|| format!("git diff --name-only {from} failed"))?;
        let untracked = git_out(repo, &["ls-files", "--others", "--exclude-standard", "--", "keel-cli", "members", ".engine"]).unwrap_or_default();
        let mut paths: Vec<String> = out.lines().chain(untracked.lines()).filter(|l| !l.is_empty()).map(str::to_string).collect();
        paths.sort();
        paths.dedup();
        (b, paths)
    } else {
        let out = git_out(repo, &["ls-files", "keel-cli/src", "keel-cli/tests", "members"]).ok_or_else(|| "git ls-files failed".to_string())?;
        ("(no base: every tracked module)".to_string(), out.lines().map(str::to_string).collect())
    };
    let mut stems: Vec<String> = changed.iter().filter_map(|p| module_stem(p)).collect();
    let embedded: Vec<String> = changed.iter().filter_map(|p| embedded_stem(p)).collect();
    stems.extend(embedded.iter().cloned());
    stems.sort();
    stems.dedup();
    let unattributed: Vec<String> = changed.iter().filter(|p| is_unattributed_source(p)).cloned().collect();
    let changed_tests: Vec<String> = changed.iter().filter_map(|p| test_name(p)).collect();
    let tests_dir = repo.join("keel-cli").join("tests");
    let mut tests: Vec<(String, String)> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&tests_dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("rs")) && p.is_file() {
                if let Some(name) = p.file_stem().map(|s| s.to_string_lossy().to_string()) {
                    let text = std::fs::read_to_string(&p).unwrap_or_default();
                    tests.push((name, text));
                }
            }
        }
    }
    let self_reading = self_reading_tests(&tests);
    let tests = touched_tests(&tests, &stems, &changed_tests);
    // The lib is in the set when any source changed - and the embedded tree IS source: its bytes are
    // in the binary and the lib's own tests read `ENGINE_DIR` (issue524).
    let lib = !stems.is_empty() || !unattributed.is_empty();
    // issue478: the endings are read with the set, before any decision to run - the receipt this
    // computation writes must be able to say `eol-mismatch` in place of a verdict cargo never reached.
    let census = crate::eol::census(repo)?;
    Ok(Touched { base, stems, unattributed, tests, lib, self_reading, changed, eol: census.mismatches, eol_scanned: census.scanned, eol_millis: census.millis })
}

/// Is the land refusal ARMED - has the human accepted D0421? The D0338 pattern: the decision file
/// carries its first acceptance result once `keel accept` has run.
#[must_use]
pub fn gate_accepted(repo: &Path) -> bool {
    let Ok(rd) = std::fs::read_dir(repo.join(".engine").join("decisions")) else { return false };
    rd.flatten().any(|e| {
        let name = e.file_name().to_string_lossy().to_string();
        name.starts_with("0421-") && std::fs::read_to_string(e.path()).is_ok_and(|t| text_carries_acceptance(&t, "d0421"))
    })
}

/// Does a Decision file's text carry a PASSING first acceptance - the `part <dec>AcceptR1 : TestResult`
/// declaration itself, with `outcome = VerdictKind::pass` in its body (pure).
///
/// Not a substring search for the token: D0421's own decision text names `d0421AcceptR1` as the thing
/// that arms it, and the first cut matched that prose at record time - the gate armed itself on the
/// sentence describing how it would be armed, and the post-commit hook's `land` then refused a push
/// while the Decision was PROPOSED (issue435). Only the declaration is the acceptance.
#[must_use]
pub fn text_carries_acceptance(text: &str, dec: &str) -> bool {
    let needle = format!("part {dec}AcceptR1 ");
    let Some(pos) = text.find(&needle) else { return false };
    let after = &text[pos..];
    // `keel accept` writes the part on one line with no nested braces; its own `}` ends the body.
    let body_end = after.find('}').unwrap_or(after.len());
    let body = &after[..body_end];
    body.contains(": TestResult") && body.contains("outcome = VerdictKind::pass")
}

/// One run of the set.
#[derive(Debug, Clone)]
pub struct Run {
    pub passed: u64,
    pub failed: u64,
    pub failing: Vec<String>,
    pub seconds: u64,
    pub cargo_ok: bool,
    pub log: PathBuf,
    /// The binaries this run executed (D0474) - the set minus `skipped`.
    pub ran: Vec<String>,
    /// The binaries observed green at this content by an earlier run, and not executed (D0474).
    pub skipped: Vec<String>,
    /// What executed the set: `cargo-nextest <version>`, or `cargo-test (nextest not installed)` (D0475).
    pub runner: String,
    /// Every test nextest ran, with its verdict and duration (D0475) - empty under the cargo-test fallback.
    pub timings: Vec<TestTiming>,
}

impl Run {
    /// Green: cargo finished and nothing failed. An all-skipped run is green with nothing counted.
    #[must_use]
    pub const fn green(&self) -> bool {
        self.cargo_ok && self.failed == 0
    }

    /// `N passed in Ns; M skipped (green at this content)` - the line `land` and `suite --touched` print.
    #[must_use]
    pub fn summary(&self) -> String {
        use std::fmt::Write as _;
        let mut s = if self.ran.is_empty() {
            "nothing to execute".to_string()
        } else {
            format!("{} passed in {}s over [{}]", self.passed, self.seconds, self.ran.join(", "))
        };
        if !self.skipped.is_empty() {
            let _ = write!(s, "; {} skipped, observed green at this content [{}]", self.skipped.len(), self.skipped.join(", "));
        }
        if let Some(t) = self.slowest() {
            let _ = write!(s, "; slowest {} {} {}.{}s", t.binary, t.test, t.millis / 1000, (t.millis % 1000) / 100);
        }
        s
    }

    /// The longest test of the run (D0475, issue536): the critical path a parallel run is bounded by.
    #[must_use]
    pub fn slowest(&self) -> Option<&TestTiming> {
        self.timings.iter().max_by_key(|t| t.millis)
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// What the receipt records about the run.
///
/// Nothing to record (`NotRun` - an empty or not-run set), a run in progress (`Running` - the
/// stub written BEFORE cargo starts, D0387's sibling for this receipt), or a finished run.
#[derive(Debug, Clone, Copy)]
pub enum Phase<'a> {
    NotRun,
    /// `pid` is the process writing the stub (issue569): a second launch reads it and refuses while
    /// that process is alive, so two runs never race on this receipt or on the tests' scratch dirs.
    Running { started: u64, log: &'a Path, skipped: &'a [String], pid: u32 },
    Done(&'a Run),
    /// The working tree's line endings disagree with the attribute (issue478): cargo never started,
    /// and the receipt names the paths in place of a verdict.
    EolMismatch,
}

/// The D0474 memo that rides every receipt: the keys the run was judged at (absent when git could
/// not name the tree) and the observation table as it stands after the run.
#[derive(Debug, Clone, Default)]
pub struct Memo {
    pub keys: Option<crate::contentkey::ContentKeys>,
    pub observed: Vec<Observed>,
}

fn render_receipt(t: &Touched, head: &str, at: u64, phase: Phase<'_>, memo: &Memo) -> String {
    use std::fmt::Write as _;
    let list = |v: &[String]| v.iter().map(|s| format!("\"{s}\"")).collect::<Vec<_>>().join(", ");
    let mut s = format!(
        "# touched receipt (D0421): the integration tests that NAME a module changed since the base, and what\n# running exactly those cost. An empty set is a receipt too. Beside the suite's receipt, never in it.\n# `eol_scanned` / `eol_ms`: the working tree's line endings against .gitattributes, read with the set\n# (issue478); `outcome = \"eol-mismatch\"` names the paths that broke it and means cargo never started.\n# `code_key` / `tree_key` (D0474): the content the run was judged at; `skipped` the binaries observed\n# green at that content by an earlier run; `[[observed]]` one green observation per binary, kept until\n# the binary runs again. A self-reading binary (`self_reading`) is skipped only when both keys hold.\nhead = \"{}\"\nat = {}\nbase = \"{}\"\nstems = [{}]\nunattributed = [{}]\ntests = [{}]\nlib = {}\nself_reading = [{}]\neol_scanned = {}\neol_ms = {}\n",
        head,
        at,
        t.base,
        list(&t.stems),
        list(&t.unattributed),
        list(&t.tests),
        t.lib,
        list(&t.self_reading),
        t.eol_scanned,
        t.eol_millis
    );
    if let Some(k) = &memo.keys {
        let _ = write!(s, "code_key = \"{}\"\ntree_key = \"{}\"\n", k.code, k.tree);
    }
    match phase {
        Phase::EolMismatch => {
            let paths: Vec<String> = t.eol.iter().map(|m| m.path.clone()).collect();
            let _ = write!(s, "outcome = \"eol-mismatch\"\npassed = 0\nfailed = 0\nfailing = []\neol_mismatch = [{}]\nseconds = 0\n", list(&paths));
        }
        // The previous receipt is REPLACED before cargo starts (issue468, the D0387/issue399 class on
        // this receipt): a reader during the run - or after a killed one - sees `running` with THIS
        // run's set and log, never the last run's pass over a different change set. The verifier of
        // sprint 661 read the prior run's 546/0 as its own while its own run was failing two lib tests.
        Phase::Running { started, log, skipped, pid } => {
            let _ = write!(
                s,
                "outcome = \"running\"\npid = {}\npassed = 0\nfailed = 0\nfailing = []\nskipped = [{}]\nseconds = {}\nlog = \"{}\"\n",
                pid,
                list(skipped),
                at.saturating_sub(started),
                log.to_string_lossy().replace('\\', "/")
            );
        }
        Phase::Done(r) => {
            let outcome = if r.green() { "pass" } else { "fail" };
            let _ = write!(
                s,
                "outcome = \"{}\"\npassed = {}\nfailed = {}\nfailing = [{}]\nran = [{}]\nskipped = [{}]\nseconds = {}\nrunner = \"{}\"\nlog = \"{}\"\n",
                outcome,
                r.passed,
                r.failed,
                list(&r.failing),
                list(&r.ran),
                list(&r.skipped),
                r.seconds,
                r.runner,
                r.log.to_string_lossy().replace('\\', "/")
            );
        }
        // `empty`: nothing to run. `not-run`: a set exists and was not run (the refusal is inert, D0337).
        Phase::NotRun => {
            let _ = write!(s, "outcome = \"{}\"\npassed = 0\nfailed = 0\nseconds = 0\n", if t.nothing_to_run() { "empty" } else { "not-run" });
        }
    }
    for o in &memo.observed {
        let _ = write!(s, "\n[[observed]]\nbinary = \"{}\"\ncode = \"{}\"\ntree = \"{}\"\nat = {}\n", o.binary, o.code, o.tree, o.at);
    }
    // D0475: one row per test nextest ran, slowest first - the answer to "which test is the critical
    // path" (issue536) is the first row, and the whole population is here for the next question.
    if let Phase::Done(r) = phase {
        let mut rows: Vec<&TestTiming> = r.timings.iter().collect();
        rows.sort_by(|a, b| b.millis.cmp(&a.millis).then_with(|| a.binary.cmp(&b.binary)).then_with(|| a.test.cmp(&b.test)));
        for t in rows {
            let _ = write!(
                s,
                "\n[[timing]]\nbinary = \"{}\"\ntest = \"{}\"\nmillis = {}\nverdict = \"{}\"\n",
                t.binary,
                t.test,
                t.millis,
                if t.passed { "pass" } else { "fail" }
            );
        }
    }
    s
}

/// Is `pid` a process alive on this host right now? `0` is never alive (an absent or unparsed pid).
///
/// Windows asks `tasklist` for exactly that pid; a Unix host reads `/proc/<pid>` where there is a
/// `/proc`, else `kill -0`. The caller's own pid is alive - the known-positive of issue569's pair.
#[must_use]
pub fn pid_alive(pid: u32) -> bool {
    pid != 0 && pid_alive_host(pid)
}

#[cfg(windows)]
fn pid_alive_host(pid: u32) -> bool {
    let filter = format!("PID eq {pid}");
    std::process::Command::new("tasklist")
        .args(["/FI", &filter, "/NH", "/FO", "CSV"])
        .output()
        .is_ok_and(|o| String::from_utf8_lossy(&o.stdout).contains(&format!("\",\"{pid}\",\"")))
}

#[cfg(not(windows))]
fn pid_alive_host(pid: u32) -> bool {
    let proc_dir = Path::new("/proc");
    if proc_dir.is_dir() {
        return proc_dir.join(pid.to_string()).is_dir();
    }
    std::process::Command::new("kill").args(["-0", &pid.to_string()]).status().is_ok_and(|s| s.success())
}

/// A run in flight as the receipt on disk states it.
///
/// The D0387 `running` stub with the `pid` that wrote it, its `at` and its log. `None` for a
/// finished receipt, an absent one, or a stub from before the pid rode it (which names no writer
/// to wait for and is replaced as before).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveRun {
    pub pid: u32,
    pub at: u64,
    pub log: String,
}

#[must_use]
pub fn live_run_in(text: &str) -> Option<LiveRun> {
    let value = |key: &str| text.lines().find_map(|l| l.strip_prefix(key).and_then(|r| r.trim_start().strip_prefix('=')).map(|v| v.trim().trim_matches('"')));
    if value("outcome")? != "running" {
        return None;
    }
    Some(LiveRun { pid: value("pid")?.parse().ok()?, at: value("at")?.parse().ok()?, log: value("log").unwrap_or_default().to_owned() })
}

/// One touched run at a time per tree (issue569).
///
/// The refusal line when `text` - the receipt on disk - says a run is in flight and `alive(pid)`
/// holds for the process that wrote it. `None` when nothing is in flight or the writer is gone: a
/// killed run's stub is replaced, as D0387 intended.
///
/// Sprint 724's verifier launched the ladder twice two seconds apart; the two runs raced on this
/// receipt's rename and on the lib tests' fixed scratch dirs, and the receipt reported five reds no
/// single run produces. The launch is where that is refused, before anything is written or started.
#[must_use]
pub fn exclusive_refusal(text: &str, alive: impl Fn(u32) -> bool) -> Option<String> {
    let live = live_run_in(text)?;
    alive(live.pid).then(|| {
        format!(
            "a touched run is already in flight - {RECEIPT} says running, written by pid {} at {} (log {}). REFUSING to launch a second: two runs race on the receipt and on the tests' scratch dirs (issue569). Nothing was written and cargo was not started; wait for that process to exit. A stub whose pid is gone is a killed run and is replaced.",
            live.pid, live.at, live.log
        )
    })
}

fn write_receipt(repo: &Path, t: &Touched, phase: Phase<'_>, memo: &Memo) {
    let metrics = repo.join(".keel").join("metrics");
    let _ = std::fs::create_dir_all(&metrics);
    let head = git_out(repo, &["rev-parse", "--short", "HEAD"]).unwrap_or_default();
    if let Err(e) = crate::write::write_atomic(&repo.join(RECEIPT), render_receipt(t, &head, now_secs(), phase, memo)) {
        eprintln!("touched: receipt could not be written: {e}");
    }
}

/// A receipt that records no run carries the observation table forward: the table is a fact about
/// earlier runs, and an empty set is not a reason to forget it.
fn carry(repo: &Path) -> Memo {
    Memo { keys: None, observed: read_observed(repo) }
}

/// One test's verdict and duration as nextest reported it (D0475).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestTiming {
    pub binary: String,
    pub test: String,
    pub millis: u64,
    pub passed: bool,
}

/// The per-test rows of a nextest run (pure, over the captured output).
///
/// A row is `PASS [   0.075s] (  1/797) keel-cli activation::tests::x` - the verdict, the duration,
/// a `(n/N)` counter, the binary id, the test. The lib's id is the package name alone (`keel-cli`);
/// an integration binary's is `keel-cli::<name>`. Only lines carrying the counter are rows: a
/// `SLOW [>120.000s] (───────)` notice has none, and the failures nextest re-lists after `Summary`
/// have none either, so a red is counted once. `LEAK` is a pass that left a child alive; a retry
/// verdict (`TRY 2 PASS`) is judged by its last word.
#[must_use]
pub fn parse_nextest(output: &str) -> Vec<TestTiming> {
    output.lines().filter_map(parse_nextest_row).collect()
}

/// One row of [`parse_nextest`], or `None` for any other line.
fn parse_nextest_row(line: &str) -> Option<TestTiming> {
    let (status, rest) = line.split_once('[')?;
    let (duration, rest) = rest.split_once(']')?;
    let (_, rest) = rest.split_once('(')?;
    let (counter, rest) = rest.split_once(')')?;
    if !counter.contains('/') || !counter.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }
    let mut words = rest.split_whitespace();
    let (id, test) = (words.next()?, words.next()?);
    let binary = id.split_once("::").map_or(LIB, |(_, name)| name).to_string();
    let status = status.trim();
    let passed = status.split_whitespace().last() == Some("PASS") || status == "LEAK";
    Some(TestTiming { binary, test: test.to_string(), millis: millis_of(duration.trim().trim_end_matches('s')), passed })
}

/// `12.345` -> 12345, integer arithmetic only; an unreadable duration is 0, which the receipt shows
/// as a row without a time rather than no row.
fn millis_of(seconds: &str) -> u64 {
    let (whole, frac) = seconds.split_once('.').unwrap_or((seconds, ""));
    let whole: u64 = whole.trim().parse().unwrap_or(0);
    let frac: String = frac.chars().chain(std::iter::repeat('0')).take(3).collect();
    whole.saturating_mul(1000).saturating_add(frac.parse().unwrap_or(0))
}

/// The `[workspace] members` reader lives with the corpus walk (sprint 732: the proof census in
/// keel-view reads it too); re-exported so `crate::touched::workspace_members` keeps resolving.
pub use crate::corpus::workspace_members;

/// The package names of every workspace member other than keel-cli (sprint 714).
///
/// These are the libs that run with `lib`. Read from the root manifest and each member's own; a
/// member whose manifest cannot be read is named by its directory's last component.
#[must_use]
pub fn member_libs(repo: &Path) -> Vec<String> {
    let Ok(ws) = std::fs::read_to_string(repo.join("Cargo.toml")) else { return Vec::new() };
    let mut out = Vec::new();
    for member in workspace_members(&ws) {
        let manifest = std::fs::read_to_string(repo.join(&member).join("Cargo.toml")).unwrap_or_default();
        let name = manifest
            .lines()
            .map(str::trim)
            .find_map(|l| l.strip_prefix("name = ").map(|v| v.trim_matches('"').to_string()))
            .unwrap_or_else(|| member.rsplit('/').next().unwrap_or(&member).to_string());
        if name != "keel-cli" {
            out.push(name);
        }
    }
    out.sort();
    out.dedup();
    out
}

/// The `[[test]]` targets of a Cargo manifest declared `harness = false` (pure over its text).
///
/// nextest lists a binary with `--list --format terse` before it runs it, and a custom harness -
/// the three cucumber binaries here - does not answer that flag (`unexpected argument '--list'`,
/// exit 104 before a single test ran, 2026-09-14). Those binaries run under `cargo test` in a second
/// invocation; everything else runs under nextest.
#[must_use]
pub fn custom_harness_tests(manifest: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_test = false;
    let mut name: Option<String> = None;
    let mut custom = false;
    for raw in manifest.lines() {
        let line = raw.trim();
        if line.starts_with('[') {
            if custom {
                out.extend(name.take());
            }
            name = None;
            custom = false;
            in_test = line == "[[test]]";
            continue;
        }
        if !in_test {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            match k.trim() {
                "name" => name = Some(v.trim().trim_matches('"').to_string()),
                "harness" => custom = v.trim().starts_with("false"),
                _ => {}
            }
        }
    }
    if custom {
        out.extend(name.take());
    }
    out.sort();
    out
}

/// Which runner executes the set, and what the receipt says it was.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Runner {
    /// `cargo nextest run` at this version string (D0475).
    Nextest(String),
    /// `cargo test`, because nextest is not installed on this host: the run is serial and carries no
    /// per-test timing, and the receipt says so. Not a refusal - a missing tool is not a red tree.
    CargoTest,
}

impl Runner {
    fn label(&self) -> String {
        match self {
            Self::Nextest(v) => format!("cargo-nextest {v}"),
            Self::CargoTest => "cargo-test (nextest not installed)".to_string(),
        }
    }
}

/// The line printed when nextest is absent - the pinned build CI installs, so the two agree.
pub const NEXTEST_INSTALL: &str = "touched: cargo nextest is not installed - the set runs serially under cargo test with no per-test timing. Install the pin CI uses: https://get.nexte.st/0.9.144/<platform> (D0475).";

/// `cargo nextest --version` once per run; `None` when the subcommand is missing.
fn nextest_version(repo: &Path) -> Option<String> {
    let out = std::process::Command::new("cargo").arg("nextest").arg("--version").current_dir(repo).output().ok()?;
    if !out.status.success() {
        return None;
    }
    // `cargo-nextest 0.9.144` (then a `cargo-nextest-` build line): the version is the second word.
    String::from_utf8_lossy(&out.stdout).lines().next()?.split_whitespace().nth(1).map(str::to_string)
}

/// What one cargo invocation left behind: whether it exited 0, and both streams in order.
struct Captured {
    ok: bool,
    text: String,
}

/// The operator's profiling request is for the suite process, never for the code under test
/// (issue539): `KEEL_PERF` in the environment reaches every keel the tests spawn, whose report -
/// on stderr, which the tests read together with stdout - then trails the JSON they parse. Three
/// binaries went red for that reason alone on 2026-09-14.
fn scrub_perf(cmd: &mut std::process::Command) {
    cmd.env_remove("KEEL_PERF");
}

/// Start cargo in `repo` with the profiling variable scrubbed, and capture both streams in order.
fn capture(mut cmd: std::process::Command, repo: &Path) -> Result<Captured, String> {
    scrub_perf(&mut cmd);
    let out = cmd.current_dir(repo).output().map_err(|e| format!("cargo could not be run: {e}"))?;
    Ok(Captured { ok: out.status.success(), text: format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)) })
}

/// `--test <name>` per integration binary, `--lib` when the lib is in - the selection both runners share.
fn select_targets(cmd: &mut std::process::Command, set: &[String]) {
    for name in set.iter().filter(|n| *n != LIB) {
        cmd.arg("--test").arg(name);
    }
    if set.iter().any(|n| n == LIB) {
        cmd.arg("--lib");
    }
}

/// What one cargo invocation over the member libs reported (sprint 714).
struct MemberLibs {
    ok: bool,
    passed: u64,
    failed: u64,
    timings: Vec<TestTiming>,
    text: String,
}

/// One invocation from the ROOT manifest with `-p` per member, `--lib` only. Its rows fold into
/// `lib` (a lib row's id is the package name, which `parse_nextest_row` reads as `lib`), and a red
/// is `lib`'s red; under the cargo-test fallback the counts come from the summary lines instead.
fn run_member_libs(repo: &Path, runner: &Runner, members: &[String]) -> Result<MemberLibs, String> {
    let mut cmd = std::process::Command::new("cargo");
    let root_manifest = repo.join("Cargo.toml");
    match runner {
        Runner::Nextest(_) => {
            cmd.arg("nextest").arg("run").arg("--release").arg("--manifest-path").arg(&root_manifest).arg("--no-fail-fast");
            cmd.arg("--no-tests").arg("pass").arg("--color").arg("never").arg("--status-level").arg("all").arg("--final-status-level").arg("none");
        }
        Runner::CargoTest => {
            cmd.arg("test").arg("--release").arg("--manifest-path").arg(&root_manifest).arg("--no-fail-fast");
        }
    }
    cmd.arg("--lib");
    for m in members {
        cmd.arg("-p").arg(m);
    }
    let c = capture(cmd, repo)?;
    let (passed, failed, timings) = match runner {
        Runner::Nextest(_) => {
            let rows = parse_nextest(&c.text);
            let count = |want: bool| u64::try_from(rows.iter().filter(|r| r.passed == want).count()).unwrap_or(u64::MAX);
            (count(true), count(false), rows)
        }
        Runner::CargoTest => {
            let (p, f) = crate::suite::count_results(&c.text);
            (p, f, Vec::new())
        }
    };
    Ok(MemberLibs { ok: c.ok, passed, failed, timings, text: c.text })
}

/// Run the set, minus the binaries observed green at this content (D0474), unless `force`.
///
/// The runner is cargo-nextest (D0475): the binaries execute in parallel, every test in its own
/// process, and each test's duration is read from nextest's output into the receipt. The
/// `harness = false` binaries of keel-cli/Cargo.toml cannot be listed by nextest and run under
/// `cargo test` in a second invocation; on a host without nextest the whole set does, and the
/// receipt's `runner` says so. Both invocations are `--release` (the binaries CI links are the ones
/// exercised) and `--no-fail-fast` (every named test reports). Writes the log and the receipt.
///
/// # Errors
/// When the metrics directory cannot be created or cargo cannot be started at all.
pub fn run(repo: &Path, t: &Touched, force: bool) -> Result<Run, String> {
    let none = || Run { passed: 0, failed: 0, failing: vec![], seconds: 0, cargo_ok: true, log: PathBuf::new(), ran: vec![], skipped: vec![], runner: String::new(), timings: vec![] };
    // issue569: the receipt on disk is READ before this run writes anything. A live run's stub
    // refuses this launch; nothing below runs, so neither receipt nor scratch dir is raced.
    if let Some(line) = exclusive_refusal(&std::fs::read_to_string(repo.join(RECEIPT)).unwrap_or_default(), pid_alive) {
        return Err(line);
    }
    if t.nothing_to_run() {
        write_receipt(repo, t, Phase::NotRun, &carry(repo));
        return Ok(none());
    }
    let metrics = repo.join(".keel").join("metrics");
    std::fs::create_dir_all(&metrics).map_err(|e| format!("cannot create {}: {e}", metrics.display()))?;
    let set = t.binaries();
    let prior = read_observed(repo);
    let keys = crate::contentkey::compute(repo);
    let (to_run, skipped) = match (&keys, force) {
        (Some(k), false) => split(&set, &t.self_reading, &prior, k),
        _ => (set, Vec::new()),
    };
    if to_run.is_empty() {
        // Every binary was observed green at exactly this content: the invariant holds and there is
        // nothing left to execute. The receipt says so, and keeps the table that says why.
        let r = Run { skipped, ..none() };
        write_receipt(repo, t, Phase::Done(&r), &Memo { keys, observed: prior });
        return Ok(r);
    }
    let started = now_secs();
    let log = metrics.join(format!("touched-{started}.log"));
    write_receipt(repo, t, Phase::Running { started, log: &log, skipped: &skipped, pid: std::process::id() }, &Memo { keys: keys.clone(), observed: prior.clone() });
    let manifest = repo.join("keel-cli").join("Cargo.toml");
    let runner = nextest_version(repo).map_or_else(
        || {
            eprintln!("{NEXTEST_INSTALL}");
            Runner::CargoTest
        },
        Runner::Nextest,
    );
    let custom = custom_harness_tests(&std::fs::read_to_string(&manifest).unwrap_or_default());
    // Under nextest the custom harnesses go to cargo test; under the fallback everything does.
    let (under_nextest, under_cargo_test): (Vec<String>, Vec<String>) = match runner {
        Runner::Nextest(_) => to_run.iter().cloned().partition(|n| !custom.contains(n)),
        Runner::CargoTest => (Vec::new(), to_run.clone()),
    };
    let (mut text, mut ok, mut passed, mut failed, mut failing, mut timings) = (String::new(), true, 0u64, 0u64, Vec::new(), Vec::new());
    if !under_nextest.is_empty() {
        let mut cmd = std::process::Command::new("cargo");
        cmd.arg("nextest").arg("run").arg("--release").arg("--manifest-path").arg(&manifest).arg("--no-fail-fast");
        // A selected binary with no tests (a lib whose tests all live in integration files) is not a
        // red: nextest alone exits `error: no tests to run`, and every named binary would be reported failing.
        cmd.arg("--no-tests").arg("pass");
        cmd.arg("--color").arg("never").arg("--status-level").arg("all").arg("--final-status-level").arg("none");
        select_targets(&mut cmd, &under_nextest);
        let c = capture(cmd, repo)?;
        let rows = parse_nextest(&c.text);
        let count = |want: bool| u64::try_from(rows.iter().filter(|r| r.passed == want).count()).unwrap_or(u64::MAX);
        let (p, f) = (count(true), count(false));
        // A list or build failure is not a verdict about the tests - but it IS a reason not to push:
        // the binaries CI will link do not link here either. Every named test is reported as not run.
        if crate::suite::never_ran(c.ok, p, f) {
            failing.extend(under_nextest.iter().cloned());
        } else {
            failing.extend(rows.iter().filter(|r| !r.passed).map(|r| r.binary.clone()));
        }
        passed += p;
        failed += f;
        timings = rows;
        ok &= c.ok;
        text.push_str(&c.text);
    }
    if !under_cargo_test.is_empty() {
        let mut cmd = std::process::Command::new("cargo");
        cmd.arg("test").arg("--release").arg("--manifest-path").arg(&manifest).arg("--no-fail-fast");
        select_targets(&mut cmd, &under_cargo_test);
        let c = capture(cmd, repo)?;
        let (p, f) = crate::suite::count_results(&c.text);
        if crate::suite::never_ran(c.ok, p, f) {
            failing.extend(under_cargo_test.iter().cloned());
        } else {
            failing.extend(failing_binaries(&c.text));
        }
        passed += p;
        failed += f;
        ok &= c.ok;
        text.push_str(&c.text);
    }
    // `lib` in the set means every workspace member's lib tests, not keel-cli's alone (sprint 714):
    // the leaf members carry the unit tests of the modules that moved into them.
    let members = member_libs(repo);
    if to_run.iter().any(|n| n == LIB) && !members.is_empty() {
        let m = run_member_libs(repo, &runner, &members)?;
        if crate::suite::never_ran(m.ok, m.passed, m.failed) || m.failed > 0 {
            failing.push(LIB.to_string());
        }
        passed += m.passed;
        failed += m.failed;
        ok &= m.ok;
        timings.extend(m.timings);
        text.push_str(&m.text);
    }
    failing.sort();
    failing.dedup();
    let _ = std::fs::write(&log, &text);
    let at = now_secs();
    // The keys again: a tree that moved while cargo ran is not one tree, and its greens are nobody's.
    let held = keys.as_ref().filter(|k| crate::contentkey::compute(repo).as_ref() == Some(*k));
    let observed = merge_observed(&prior, &to_run, &failing, ok, held, at);
    let r = Run { passed, failed, failing, seconds: at.saturating_sub(started), cargo_ok: ok, log, ran: to_run, skipped, runner: runner.label(), timings };
    write_receipt(repo, t, Phase::Done(&r), &Memo { keys, observed });
    Ok(r)
}

/// One line per fact, for `land` and `suite --touched` alike.
fn describe(t: &Touched) -> String {
    use std::fmt::Write as _;
    let mut s = format!("touched tests: base {}; changed modules [{}]", t.base, t.stems.join(", "));
    if !t.unattributed.is_empty() {
        let _ = write!(s, "; unattributed (name no module): [{}]", t.unattributed.join(", "));
    }
    if t.tests.is_empty() {
        s.push_str("; set EMPTY - no integration test names a changed module");
    } else {
        let _ = write!(s, "; set [{}]", t.tests.join(", "));
    }
    if t.lib {
        s.push_str("; plus the lib unit tests (a source path changed)");
    } else if t.tests.is_empty() {
        s.push_str(", nothing to run");
    }
    s
}

/// `land`'s first call, BEFORE the tree gate: compute the set and judge the line endings (issue478).
///
/// Self-build only. `None` = a downstream tree, or one whose set could not be computed - `land` gates
/// and pushes without a touched run. `Some(Err(code))` = refuse with that exit code; `Some(Ok(t))` =
/// the set to hand to [`after_gate`] once the tree gate is green.
///
/// WHY BEFORE THE GATE: guard `working-tree-eol` reads the same census, so a mismatch would otherwise
/// surface as one of N gate problems and this line - the changed paths first, the count of the rest,
/// the receipt - would never be reached. The census is one `git ls-files --eol` per run either way.
#[must_use]
pub fn before_gate(repo: &Path) -> Option<Result<Touched, i32>> {
    if !crate::suite::is_self_build(repo) {
        return None;
    }
    let t = match compute(repo) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("keel land: touched tests could not be computed ({e}) - pushing without them");
            return None;
        }
    };
    println!("keel land: {}", describe(&t));
    // issue478: the bytes cargo would compile are judged BEFORE the run and before the empty-set
    // shortcut - a CRLF `.sysml` under `eol=lf` names no module, and it is still a tree this push
    // must not carry to a machine whose tests read it exactly. Not behind the D0421 arming: the
    // touched run's verdict depended on the tree's endings, and a verdict that depends on which tool
    // last wrote a file is not the verdict D0421 armed.
    if let Some(line) = t.eol_refusal() {
        write_receipt(repo, &t, Phase::EolMismatch, &carry(repo));
        eprintln!("keel land: {line}");
        eprintln!("  REFUSING to push: the touched tests would read these bytes, not the ones git normalises at the commit (issue478). Receipt {RECEIPT} says eol-mismatch. Nothing was pushed.");
        return Some(Err(1));
    }
    println!("keel land: {}", t.eol_line());
    Some(Ok(t))
}

/// `land`'s call after the tree gate and before the first push, with the set [`before_gate`]
/// computed. `None` = continue to push; `Some(code)` = refuse with that exit code.
#[must_use]
pub fn after_gate(repo: &Path, t: &Touched) -> Option<i32> {
    if t.nothing_to_run() {
        write_receipt(repo, t, Phase::NotRun, &carry(repo));
        return None;
    }
    if !gate_accepted(repo) {
        println!("keel land: not run - D0421 is proposed; the touched-test refusal is declared but INERT until the human's word (D0337); the set is in the receipt ({RECEIPT}).");
        write_receipt(repo, t, Phase::NotRun, &carry(repo));
        return None;
    }
    if let Some(reason) = crate::suite::own_image_refusal(repo, "keel land") {
        eprintln!("{reason}");
        return Some(2);
    }
    let n = t.binaries().len();
    println!("keel land: {n} touched test binar{} before the push (cargo nextest run over the selected binaries, D0475; those observed green at this content are skipped, D0474)", if n == 1 { "y" } else { "ies" });
    match run(repo, t, crate::receipt::forced(&[])) {
        Ok(r) if r.green() => {
            println!("keel land: touched tests pass - {} (receipt {RECEIPT})", r.summary());
            None
        }
        Ok(r) => {
            eprintln!("keel land: touched tests FAIL - REFUSING to push. Failing: [{}] ({} passed, {} failed, {}s; log {})", r.failing.join(", "), r.passed, r.failed, r.seconds, r.log.display());
            eprintln!("  These tests name a module this push changes. Fix them (or the module), commit, and re-run. Nothing was pushed.");
            Some(1)
        }
        Err(e) => {
            eprintln!("keel land: touched tests could not be run ({e}) - REFUSING to push: the set is non-empty and unverified.");
            Some(2)
        }
    }
}

/// `keel suite --touched [ROOT] [--no-receipt]`: compute and run the set now, on the caller's word.
/// `force` (`--no-receipt` / `KEEL_NO_RECEIPT=1`) runs every binary in the set, observed or not.
#[must_use]
pub fn cmd(repo: &Path, force: bool) -> i32 {
    if !crate::suite::is_self_build(repo) {
        eprintln!("keel suite --touched: {} holds no keel-cli/Cargo.toml - there is no test set to compute here", repo.display());
        return 2;
    }
    let t = match compute(repo) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("keel suite --touched: {e}");
            return 2;
        }
    };
    println!("keel suite --touched: {}", describe(&t));
    if let Some(line) = t.eol_refusal() {
        write_receipt(repo, &t, Phase::EolMismatch, &carry(repo));
        eprintln!("keel suite --touched: {line}");
        eprintln!("  REFUSING to run: cargo would compile and test these bytes, not the ones git normalises at the commit (issue478). Receipt {RECEIPT} says eol-mismatch; nothing was measured.");
        return 1;
    }
    println!("keel suite --touched: {}", t.eol_line());
    if t.nothing_to_run() {
        write_receipt(repo, &t, Phase::NotRun, &carry(repo));
        println!("keel suite --touched: empty set recorded in {RECEIPT}");
        return 0;
    }
    if let Some(reason) = crate::suite::own_image_refusal(repo, "keel suite --touched") {
        eprintln!("{reason}");
        return 2;
    }
    match run(repo, &t, force) {
        Ok(r) => {
            for l in std::fs::read_to_string(&r.log).unwrap_or_default().lines().filter(|l| l.contains("FAILED") || l.contains("panicked at")).take(20) {
                println!("  {l}");
            }
            let outcome = if r.green() { "pass" } else { "fail" };
            println!("keel suite --touched: {outcome} - {}; {} failed, failing [{}]; receipt {RECEIPT}", r.summary(), r.failed, r.failing.join(", "));
            if outcome == "pass" { 0 } else { 101 }
        }
        Err(e) => {
            eprintln!("keel suite --touched: {e}");
            2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compute, embedded_stem, exclusive_refusal, failing_binaries, live_run_in, merge_observed, module_stem, names_stem, parse_observed, pid_alive, render_receipt, self_reading_tests, split, test_name, text_carries_acceptance, touched_tests, LiveRun, Memo, Observed,
        custom_harness_tests, is_unattributed_source, millis_of, parse_nextest, workspace_members, Phase, Run, Touched, LIB,
    };
    use crate::contentkey::ContentKeys;

    fn fixture() -> Touched {
        Touched {
            base: "origin/main".into(),
            stems: vec!["scaffold".into()],
            unattributed: vec![],
            tests: vec!["orient_bdd".into()],
            lib: true,
            self_reading: vec![LIB.into()],
            changed: vec!["keel-cli/src/scaffold.rs".into()],
            eol: vec![],
            eol_scanned: 3,
            eol_millis: 7,
        }
    }

    fn keys(code: &str, tree: &str) -> ContentKeys {
        ContentKeys { code: code.into(), tree: tree.into() }
    }

    fn run_of(passed: u64, failed: u64, failing: &[&str], cargo_ok: bool, ran: &[&str], skipped: &[&str]) -> Run {
        let v = |xs: &[&str]| xs.iter().map(|s| (*s).to_string()).collect::<Vec<_>>();
        Run { passed, failed, failing: v(failing), seconds: 9, cargo_ok, log: std::path::PathBuf::from("x.log"), ran: v(ran), skipped: v(skipped), runner: "cargo-nextest 0.9.144".into(), timings: vec![] }
    }

    /// issue478 known-positive: a set whose tree holds a CRLF `eol=lf` path renders `eol-mismatch` with
    /// the paths, no verdict and no run; the refusal line names the changed path first and counts the
    /// rest. Known-negative: a clean census renders no refusal and the header carries its population.
    #[test]
    fn an_eol_mismatch_is_a_receipt_in_place_of_a_verdict() {
        let mut t = fixture();
        assert!(t.eol_refusal().is_none(), "a clean census refuses nothing");
        let clean = render_receipt(&t, "1234567", 100, Phase::NotRun, &Memo::default());
        assert!(clean.contains("eol_scanned = 3\n") && clean.contains("eol_ms = 7\n"), "{clean}");
        t.eol = vec![
            crate::eol::Mismatch { path: "keel-cli/src/scaffold.rs".into(), declared: "lf".into(), worktree: "crlf".into() },
            crate::eol::Mismatch { path: ".tracking/backlog.sysml".into(), declared: "lf".into(), worktree: "crlf".into() },
        ];
        let line = t.eol_refusal().expect("a mismatch refuses");
        assert!(line.contains("changed by this push: [keel-cli/src/scaffold.rs (w/crlf where eol=lf)]"), "{line}");
        assert!(line.contains("and 1 unchanged path (first: .tracking/backlog.sysml)"), "{line}");
        let text = render_receipt(&t, "1234567", 100, Phase::EolMismatch, &Memo::default());
        assert!(text.contains("outcome = \"eol-mismatch\""), "{text}");
        assert!(text.contains("eol_mismatch = [\"keel-cli/src/scaffold.rs\", \".tracking/backlog.sysml\"]"), "{text}");
        assert!(text.contains("passed = 0\nfailed = 0\n"), "{text}");
        assert!(!text.contains("\"pass\"") && !text.contains("\"running\""), "{text}");
    }

    /// issue468, known-negative: a finished run's receipt carries its verdict and counts.
    #[test]
    fn a_finished_run_renders_its_verdict() {
        let run = run_of(5, 1, &[LIB], false, &[LIB, "orient_bdd"], &[]);
        let text = render_receipt(&fixture(), "1234567", 100, Phase::Done(&run), &Memo::default());
        assert!(text.contains("outcome = \"fail\""), "{text}");
        assert!(text.contains("passed = 5") && text.contains("failed = 1"), "{text}");
        assert!(text.contains("ran = [\"lib\", \"orient_bdd\"]") && text.contains("skipped = []"), "{text}");
        assert!(!text.contains("outcome = \"running\""), "{text}");
    }

    /// issue468, known-positive: the stub written before cargo starts says `running`, counts nothing,
    /// names THIS run's log and set - so a reader during the run, or after a killed one, never sees
    /// the previous run's pass over a different change set.
    #[test]
    fn a_running_stub_is_not_a_verdict() {
        let log = std::path::PathBuf::from(".keel/metrics/touched-100.log");
        let skipped = vec!["orient_bdd".to_string()];
        let text = render_receipt(&fixture(), "1234567", 103, Phase::Running { started: 100, log: &log, skipped: &skipped, pid: 4242 }, &Memo::default());
        assert!(text.contains("outcome = \"running\""), "{text}");
        assert!(text.contains("pid = 4242"), "the stub names the process writing it (issue569): {text}");
        assert!(text.contains("passed = 0") && text.contains("failed = 0"), "{text}");
        assert!(text.contains("seconds = 3"), "{text}");
        assert!(text.contains("touched-100.log"), "{text}");
        assert!(text.contains("skipped = [\"orient_bdd\"]"), "the stub says what this run is not executing: {text}");
        assert!(text.contains("\"scaffold\"") && text.contains("lib = true"), "the stub carries the set it is running:\n{text}");
        assert!(!text.contains("\"pass\""), "{text}");
        let live = live_run_in(&text).expect("a running stub reads back as a live run");
        assert_eq!(live, LiveRun { pid: 4242, at: 103, log: ".keel/metrics/touched-100.log".into() });
    }

    fn running_stub_by(pid: u32) -> String {
        let log = std::path::PathBuf::from(".keel/metrics/touched-100.log");
        render_receipt(&fixture(), "1234567", 103, Phase::Running { started: 100, log: &log, skipped: &[], pid }, &Memo::default())
    }

    /// issue569 known-positive, chosen before the tree is read: the receipt says `running` with THIS
    /// process's own pid - the writer is alive by construction - and a second launch is refused with a
    /// line naming the pid and the stub's `at`. The liveness read is the real one (`pid_alive`).
    #[test]
    fn a_second_launch_is_refused_while_the_stub_writer_is_alive() {
        let me = std::process::id();
        let line = exclusive_refusal(&running_stub_by(me), pid_alive).expect("refused: the writer is alive");
        assert!(line.contains(&format!("pid {me} at 103")), "{line}");
        assert!(line.contains("REFUSING") && line.contains("issue569"), "{line}");
        assert!(pid_alive(me), "this process is alive");
        assert!(!pid_alive(0), "0 is never a live writer");
    }

    /// issue569 known-negative: a `running` stub whose pid no process holds is a killed run - the
    /// launch proceeds and the stub is replaced, as D0387 intended. The pid is a child spawned and
    /// waited on, so no process holds it when the stub is read; a finished receipt and a stub from
    /// before the pid rode it refuse nothing either.
    #[test]
    fn a_stub_whose_writer_is_gone_is_replaced() {
        let mut child = if cfg!(windows) { std::process::Command::new("cmd").args(["/C", "exit 0"]).spawn() } else { std::process::Command::new("true").spawn() }.expect("spawn a short-lived child");
        let gone = child.id();
        let _ = child.wait();
        assert!(!pid_alive(gone), "the child has exited: pid {gone}");
        assert_eq!(exclusive_refusal(&running_stub_by(gone), pid_alive), None, "a dead writer's stub is replaced");
        assert_eq!(exclusive_refusal(&running_stub_by(7), |_| false), None);
        assert!(live_run_in(&running_stub_by(7)).is_some(), "the stub is read; only liveness clears it");
        let finished = render_receipt(&fixture(), "1234567", 103, Phase::NotRun, &Memo::default());
        assert_eq!(exclusive_refusal(&finished, |_| true), None, "a finished receipt is not a run in flight");
        let before_pid = running_stub_by(7).replace("pid = 7\n", "");
        assert_eq!(live_run_in(&before_pid), None, "a stub from before the pid rode it names no writer");
        assert_eq!(exclusive_refusal(&before_pid, |_| true), None);
        assert_eq!(exclusive_refusal("", |_| true), None, "no receipt, no refusal");
    }

    /// D0474 round trip: the receipt renders its keys and its `[[observed]]` table, and `parse_observed`
    /// reads the table back exactly. A receipt with no table, or one that is not TOML, reads as no
    /// observation (known-negative) - never as a green.
    #[test]
    fn the_observation_table_round_trips_through_the_receipt() {
        let observed = vec![Observed { binary: "orient_bdd".into(), code: "c1".into(), tree: "t1".into(), at: 50 }, Observed { binary: LIB.into(), code: "c1".into(), tree: "t2".into(), at: 51 }];
        let run = run_of(3, 0, &[], true, &[LIB], &["orient_bdd"]);
        let text = render_receipt(&fixture(), "1234567", 100, Phase::Done(&run), &Memo { keys: Some(keys("c1", "t2")), observed: observed.clone() });
        assert!(text.contains("code_key = \"c1\"\ntree_key = \"t2\"\n"), "{text}");
        assert!(text.contains("self_reading = [\"lib\"]"), "{text}");
        assert!(text.contains("outcome = \"pass\"") && text.contains("skipped = [\"orient_bdd\"]") && text.contains("ran = [\"lib\"]"), "{text}");
        assert_eq!(parse_observed(&text), observed, "the table reads back as written:\n{text}");
        assert!(parse_observed("outcome = \"pass\"\n").is_empty(), "no table is no observation");
        assert!(parse_observed("this is not = = toml").is_empty(), "an unparseable receipt is no observation");
    }

    /// D0474 known-positive: a binary observed green at the current code key is skipped; a self-reading
    /// binary observed at the current code key but an older tree key is RUN. Known-negative: a binary
    /// with no observation, or one at an older code key, is run. The two halves are the whole set.
    #[test]
    fn a_binary_is_skipped_only_at_the_content_its_outcome_depends_on() {
        let set: Vec<String> = vec!["a_gate".into(), "b_reads_repo".into(), LIB.into(), "z_new".into()];
        let self_reading: Vec<String> = vec!["b_reads_repo".into(), LIB.into()];
        let observed = vec![
            Observed { binary: "a_gate".into(), code: "c2".into(), tree: "t1".into(), at: 1 },
            Observed { binary: "b_reads_repo".into(), code: "c2".into(), tree: "t1".into(), at: 1 },
            Observed { binary: LIB.into(), code: "c2".into(), tree: "t2".into(), at: 1 },
        ];
        // The unchanged tree: everything observed is skipped, the new binary runs.
        let (run, skipped) = split(&set, &self_reading, &observed, &keys("c2", "t1"));
        assert_eq!(skipped, vec!["a_gate".to_string(), "b_reads_repo".to_string()]);
        assert_eq!(run, vec![LIB.to_string(), "z_new".to_string()], "lib was observed at another tree and is self-reading");
        // A ceremony write: the tree moved, the code did not - exactly the self-reading binaries run.
        let (run, skipped) = split(&set, &self_reading, &observed, &keys("c2", "t3"));
        assert_eq!(skipped, vec!["a_gate".to_string()]);
        assert_eq!(run, vec!["b_reads_repo".to_string(), LIB.to_string(), "z_new".to_string()]);
        // A code edit: the code moved - everything runs.
        let (run, skipped) = split(&set, &self_reading, &observed, &keys("c3", "t1"));
        assert!(skipped.is_empty());
        assert_eq!(run, set);
    }

    /// D0474: after a run, the binaries that ran and passed are observed at the run's keys, the one that
    /// failed loses its entry, and a binary not run keeps its old entry (known-positive). A run whose
    /// tree moved (`keys` None) or that cargo never finished records no green (known-negative).
    #[test]
    fn the_table_gains_the_greens_that_ran_and_drops_the_red() {
        let prior = vec![
            Observed { binary: "kept".into(), code: "c1".into(), tree: "t1".into(), at: 1 },
            Observed { binary: "red".into(), code: "c1".into(), tree: "t1".into(), at: 1 },
            Observed { binary: "green".into(), code: "c1".into(), tree: "t1".into(), at: 1 },
        ];
        let ran: Vec<String> = vec!["green".into(), "red".into(), LIB.into()];
        let k = keys("c2", "t2");
        let got = merge_observed(&prior, &ran, &["red".to_string()], false, Some(&k), 9);
        let names: Vec<&str> = got.iter().map(|o| o.binary.as_str()).collect();
        assert_eq!(names, vec!["green", "kept", LIB]);
        assert!(got.iter().filter(|o| o.binary != "kept").all(|o| o.code == "c2" && o.tree == "t2" && o.at == 9), "{got:?}");
        assert_eq!(got.iter().find(|o| o.binary == "kept").map(|o| o.code.as_str()), Some("c1"), "a binary not run keeps its observation");
        // The tree moved mid-run: nothing new, the ran entries are gone.
        let moved = merge_observed(&prior, &ran, &[], true, None, 9);
        assert_eq!(moved.iter().map(|o| o.binary.as_str()).collect::<Vec<_>>(), vec!["kept"]);
        // Cargo did not finish and attributed nothing (a build error): nobody's green.
        let built_not = merge_observed(&prior, &ran, &[], false, Some(&k), 9);
        assert_eq!(built_not.iter().map(|o| o.binary.as_str()).collect::<Vec<_>>(), vec!["kept"]);
    }

    /// D0474: the scan names the one idiom the eighteen use; `lib` is always in.
    #[test]
    fn a_test_that_names_the_manifest_dir_is_self_reading_and_lib_always_is() {
        let tests = vec![
            ("reads_repo".to_string(), "let root = Path::new(env!(\"CARGO_MANIFEST_DIR\")).parent();".to_string()),
            ("scaffolds".to_string(), "keel init over a temp dir".to_string()),
        ];
        assert_eq!(self_reading_tests(&tests), vec![LIB.to_string(), "reads_repo".to_string()]);
        assert_eq!(self_reading_tests(&[]), vec![LIB.to_string()]);
    }

    #[test]
    fn a_stem_is_the_module_a_path_names() {
        assert_eq!(module_stem("keel-cli/src/sync.rs"), Some("sync".to_string()));
        assert_eq!(module_stem("keel-cli\\src\\view\\mod.rs"), Some("view".to_string()));
        assert_eq!(module_stem("keel-cli/src/view/table.rs"), Some("table".to_string()));
        assert_eq!(module_stem("keel-cli/src/main.rs"), None);
        assert_eq!(module_stem("keel-cli/src/lib.rs"), None);
        assert_eq!(module_stem("keel-cli/tests/sync.rs"), None);
        assert_eq!(module_stem(".engine/tools/x.rs"), None);
        // D0479 members (sprint 714): the known positive is the first module that moved; the known
        // negative is a path under members/ that is not a crate's src.
        assert_eq!(module_stem("members/keel-git/src/gitx.rs"), Some("gitx".to_string()));
        assert_eq!(module_stem("members\\keel-json\\src\\color.rs"), Some("color".to_string()));
        assert_eq!(module_stem("members/keel-git/src/lib.rs"), None);
        assert!(is_unattributed_source("members/keel-git/src/lib.rs"));
        assert_eq!(module_stem("members/keel-git/Cargo.toml"), None);
        assert!(!is_unattributed_source("members/keel-git/Cargo.toml"));
        assert_eq!(module_stem("members/src/x.rs"), None);
        assert_eq!(workspace_members("[workspace]\nmembers = [\n    \"keel-parser\",\n    \"members/keel-git\",\n    \"keel-cli\",\n]\nresolver = \"2\"\n"), vec!["keel-parser", "members/keel-git", "keel-cli"]);
        assert!(workspace_members("[package]\nname = \"x\"\n").is_empty());
        assert_eq!(test_name("keel-cli/tests/land_gate.rs"), Some("land_gate".to_string()));
        assert_eq!(test_name("keel-cli/tests/common/mod.rs"), None);
    }

    /// issue524: the path that went red on CI under a 570/0 receipt is the known positive; a path the
    /// binary does not embed is the known negative - chosen before the rule was written (D0388).
    #[test]
    fn a_change_under_the_embedded_tree_names_init() {
        assert_eq!(embedded_stem(".engine/skills/delegated-ceremony/references/check_report.py"), Some("init".to_string()));
        assert_eq!(embedded_stem(".engine\\processes\\delegated-ceremony.sysml"), Some("init".to_string()));
        assert_eq!(embedded_stem(".tracking/backlog.sysml"), None);
        assert_eq!(embedded_stem("keel-cli/src/init.rs"), None);
        assert_eq!(embedded_stem(".engine/"), None);
        assert_eq!(embedded_stem(".engineering/x"), None);
    }

    #[test]
    fn a_stem_is_named_as_a_whole_word_only() {
        assert!(names_stem("use keel_cli::sync::cmd_land;", "sync"));
        assert!(names_stem("runs `keel sync` twice", "sync"));
        assert!(names_stem("sync.rs owns it", "sync"));
        assert!(names_stem("sync", "sync"));
        assert!(!names_stem("the tree is synced and resync is off", "sync"));
        assert!(!names_stem("nothing here", "sync"));
        assert!(!names_stem("anything", ""));
    }

    #[test]
    fn the_set_is_the_tests_naming_a_changed_stem_plus_the_tests_that_changed() {
        let tests = vec![
            ("land_gate".to_string(), "keel_cli::sync::cmd_land".to_string()),
            ("suite_receipt".to_string(), "keel suite writes a receipt".to_string()),
            ("unrelated".to_string(), "nothing named".to_string()),
        ];
        // known positive: sync changed -> land_gate; the changed test itself joins
        let got = touched_tests(&tests, &["sync".to_string()], &["unrelated".to_string()]);
        assert_eq!(got, vec!["land_gate".to_string(), "unrelated".to_string()]);
        // known negative: a module nothing names -> empty
        assert!(touched_tests(&tests, &["orient".to_string()], &[]).is_empty());
    }

    /// Known-positive: the shape `keel accept` writes. Known-negatives: the token named in the decision
    /// prose (issue435, the live failure), a declared acceptance whose outcome is fail, and no token.
    #[test]
    fn arming_reads_the_acceptance_part_not_the_token_in_prose() {
        let accepted = "package D { part d0421 : Decision { :>> decision = \"armed by d0421AcceptR1\"; }\n    part d0421AcceptR1 : TestResult { :>> outcome = VerdictKind::pass; :>> judgedAgainst = \"abc\"; }\n}\n";
        assert!(text_carries_acceptance(accepted, "d0421"));
        let prose_only = "package D { part d0421 : Decision { :>> decision = \"once this Decision carries d0421AcceptR1 in its file\"; } }\n";
        assert!(!text_carries_acceptance(prose_only, "d0421"));
        let failed = "package D {\n    part d0421AcceptR1 : TestResult { :>> outcome = VerdictKind::fail; }\n}\n";
        assert!(!text_carries_acceptance(failed, "d0421"));
        assert!(!text_carries_acceptance("package D { }", "d0421"));
    }

    /// issue539 probe pair, chosen before the fix was written. Known-positive: the command the run
    /// builds carries an explicit removal of `KEEL_PERF`, so the child cannot inherit it whatever the
    /// parent's environment. Known-negative: a command nobody scrubbed carries no such entry - the
    /// inherited environment, which is exactly the shape that went red.
    #[test]
    fn the_cargo_child_never_inherits_the_operators_perf_variable() {
        let mut scrubbed = std::process::Command::new("cargo");
        super::scrub_perf(&mut scrubbed);
        let removed: Vec<&std::ffi::OsStr> = scrubbed.get_envs().filter(|(_, v)| v.is_none()).map(|(k, _)| k).collect();
        assert_eq!(removed, vec![std::ffi::OsStr::new("KEEL_PERF")], "the scrub is an explicit removal on the child");
        let inherited = std::process::Command::new("cargo");
        assert_eq!(inherited.get_envs().count(), 0, "an unscrubbed command inherits everything, KEEL_PERF included");
    }

    /// D0475 probe pair for the nextest parser. Known-positive: a run's rows - the lib (id is the
    /// package alone), an integration binary, a LEAK, a FAIL, a retry - are read with their
    /// durations. Known-negative: the SLOW notice, the final re-listing after `Summary` (no
    /// counter) and cargo's own lines are not rows, so a red is counted exactly once.
    #[test]
    fn nextest_rows_are_read_once_each_with_their_durations() {
        let out = "    Finished `release` profile [optimized] target(s) in 0.48s\n\
                   ────────────\n\
                    Nextest run ID ec1d with nextest profile: default\n\
                       Starting 797 tests across 62 binaries\n\
                           PASS [   0.075s] (  1/797) keel-cli activation::tests::a_typo_fails_loud\n\
                           LEAK [   2.500s] (  2/797) keel-cli::hooks a_child_outlives_the_test\n\
                           SLOW [>120.000s] (───────) keel-cli::touched_tests_run_before_land a_binary_observed_green\n\
                           FAIL [  12.100s] (  3/797) keel-cli::orient_x a_ready_item_is_listed\n\
                      TRY 2 PASS [   0.900s] (  4/797) keel-cli::flaky_bin second_time_lucky\n\
                           PASS [ 690.475s] (797/797) keel-cli view::tests::report_produces_cards_and_rejects_unknown\n\
                   ────────────\n\
                        Summary [ 722.834s] 797 tests run: 796 passed, 1 failed, 0 skipped\n\
                           FAIL [  12.100s] keel-cli::orient_x a_ready_item_is_listed\n";
        let rows = parse_nextest(out);
        let names: Vec<(&str, &str, u64, bool)> = rows.iter().map(|r| (r.binary.as_str(), r.test.as_str(), r.millis, r.passed)).collect();
        assert_eq!(
            names,
            vec![
                ("lib", "activation::tests::a_typo_fails_loud", 75, true),
                ("hooks", "a_child_outlives_the_test", 2500, true),
                ("orient_x", "a_ready_item_is_listed", 12100, false),
                ("flaky_bin", "second_time_lucky", 900, true),
                ("lib", "view::tests::report_produces_cards_and_rejects_unknown", 690_475, true),
            ]
        );
        assert_eq!(rows.iter().filter(|r| !r.passed).count(), 1, "the re-listed failure after Summary is not a second red");
        assert!(parse_nextest("error: creating test list failed\nexit=104\n").is_empty(), "a list failure has no rows");
        assert_eq!(millis_of("0.0"), 0);
        assert_eq!(millis_of("7"), 7000);
        assert_eq!(millis_of("1.5"), 1500);
        assert_eq!(millis_of("junk"), 0);
    }

    /// D0475: the `harness = false` targets are read from the manifest. Known-positive: keel-cli's three
    /// cucumber binaries, declared in any field order, with the flag anywhere in their block.
    /// Known-negative: a libtest `[[test]]`, a `[[bin]]` with `harness = false`, and the package
    /// table are not in the list.
    #[test]
    fn custom_harness_targets_are_read_from_the_manifest() {
        let manifest = "[package]\nname = \"keel-cli\"\n\n[[bin]]\nname = \"keel\"\nharness = false\n\n[[test]]\nname = \"cli_bdd\"\nharness = false\n\n[[test]]\nharness = false\nname = \"orient_bdd\"\n\n[[test]]\nname = \"plain\"\n\n[[test]]\nname = \"write_bdd\"\nharness = false  # cucumber\n";
        assert_eq!(custom_harness_tests(manifest), vec!["cli_bdd".to_string(), "orient_bdd".to_string(), "write_bdd".to_string()]);
        assert!(custom_harness_tests("[package]\nname = \"x\"\n").is_empty());
        // keel-cli's manifest declares exactly these three; keel-parser's four are CI's to route (guard custom-harness-routed, issue542).
        let real = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")).unwrap();
        assert_eq!(custom_harness_tests(&real), vec!["cli_bdd".to_string(), "orient_bdd".to_string(), "write_bdd".to_string()]);
    }

    /// Known-positive: a capture in cargo's real order - every stdout result first, then stderr with
    /// the headers, the rerun hint and the `--no-fail-fast` summary. Known-negative: a green run.
    #[test]
    fn failing_binaries_are_read_from_cargos_rerun_hint() {
        let out = "running 1 test\ntest x ... FAILED\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored\nrunning 3 tests\ntest result: ok. 3 passed; 0 failed; 0 ignored\n     Running tests\\land_gate.rs (target\\release\\deps\\land_gate-abc.exe)\nerror: test failed, to rerun pass `--test land_gate`\n     Running tests\\other.rs (target\\release\\deps\\other-def.exe)\nerror: 1 target failed:\n    `--test land_gate`\n";
        assert_eq!(failing_binaries(out), vec!["land_gate".to_string()]);
        let two = "error: 2 targets failed:\n    `--test b_gate`\n    `--test a_gate`\n";
        assert_eq!(failing_binaries(two), vec!["a_gate".to_string(), "b_gate".to_string()]);
        let lib = "error: test failed, to rerun pass `--lib`\nerror: 2 targets failed:\n    `--lib`\n    `--test a_gate`\n";
        assert_eq!(failing_binaries(lib), vec!["a_gate".to_string(), "lib".to_string()]);
        assert!(failing_binaries("test result: ok. 1 passed; 0 failed\n     Running tests/other.rs (x)\n").is_empty());
    }

    /// The set is what the WORKING TREE changed, not what HEAD did (issue463): with HEAD equal to the
    /// base and edits uncommitted, the verifier's pre-commit run read an empty set. Known-negative: a
    /// clean tree whose last commit touched no module reads no stem. Known-positive: an uncommitted edit
    /// to a tracked module and an untracked new module both read as changed.
    #[test]
    fn the_changed_set_is_the_working_trees_not_heads() {
        let dir = std::env::temp_dir().join(format!("keel-touched-{}", crate::ident::gen_uuid()));
        std::fs::create_dir_all(dir.join("keel-cli").join("src")).unwrap();
        std::fs::create_dir_all(dir.join("keel-cli").join("tests")).unwrap();
        let git = |args: &[&str]| {
            let o = crate::gitx::git().arg("-C").arg(&dir).args(args).output().unwrap();
            assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
            String::from_utf8_lossy(&o.stdout).trim().to_string()
        };
        git(&["init", "-q"]);
        std::fs::write(dir.join("keel-cli").join("src").join("foo.rs"), "pub fn foo() {}\n").unwrap();
        std::fs::write(dir.join("keel-cli").join("tests").join("foo_bites.rs"), "// names foo\n").unwrap();
        git(&["add", "-A"]);
        git(&["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "base"]);
        // A second commit touching no module, so the base (HEAD~1, no remote here) differs from HEAD by nothing under keel-cli.
        git(&["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "--allow-empty", "-m", "head"]);
        let clean = compute(&dir).unwrap();
        assert!(clean.stems.is_empty(), "known-negative: a clean tree reads no stem, got {:?}", clean.stems);
        assert!(!clean.lib, "known-negative: no source path changed");
        assert_eq!(clean.self_reading, vec![LIB.to_string()], "no test names the manifest dir; lib always is");

        std::fs::write(dir.join("keel-cli").join("src").join("foo.rs"), "pub fn foo() { /* edited, uncommitted */ }\n").unwrap();
        std::fs::write(dir.join("keel-cli").join("src").join("newmod.rs"), "pub fn newmod() {}\n").unwrap();
        let dirty = compute(&dir).unwrap();
        assert_eq!(dirty.stems, vec!["foo".to_string(), "newmod".to_string()], "known-positive: the uncommitted edit and the untracked module are the change");
        assert!(dirty.lib, "a source path changed, so the lib's own tests run");
        assert_eq!(dirty.tests, vec!["foo_bites".to_string()], "the test naming the changed stem is in the set");
        assert_eq!(dirty.binaries(), vec!["foo_bites".to_string(), LIB.to_string()]);

        // issue524: an UNTRACKED new file under the embedded tree is a change too, and it names `init`
        // - sprint 699's skill was exactly this when the verifier ran, and the set had no way to see it.
        std::fs::create_dir_all(dir.join(".engine").join("skills").join("x")).unwrap();
        std::fs::write(dir.join(".engine").join("skills").join("x").join("check.py"), "print(1)\n").unwrap();
        std::fs::write(dir.join("keel-cli").join("tests").join("scaffolds.rs"), "// runs keel init over the tree at CARGO_MANIFEST_DIR\n").unwrap();
        let embedded = compute(&dir).unwrap();
        assert_eq!(embedded.stems, vec!["foo".to_string(), "init".to_string(), "newmod".to_string()], "the untracked engine file contributes init");
        assert!(embedded.changed.iter().any(|c| c.ends_with(".engine/skills/x/check.py")), "the untracked engine file is in the changed set: {:?}", embedded.changed);
        assert!(embedded.tests.contains(&"scaffolds".to_string()), "the test that runs keel init is in the set: {:?}", embedded.tests);
        assert_eq!(embedded.self_reading, vec![LIB.to_string(), "scaffolds".to_string()], "the test naming the manifest dir is self-reading");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The anchors a unit test may not key on: each names this repository only while the crate sits
    /// one level below it, so the test breaks the moment its module moves into `members/`. Returned
    /// as (1-based line, the offending text). A `//` comment line is not an anchor.
    fn cwd_relative_anchors(src: &str) -> Vec<(usize, String)> {
        const ANCHORS: [&str; 4] = ["Path::new(\"..\")", "read_to_string(\"src/", "CARGO_MANIFEST_DIR\")).join(\"..\")", "read_to_string(\"../"];
        src.lines()
            .enumerate()
            .filter(|(_, l)| !l.trim_start().starts_with("//"))
            .filter_map(|(i, l)| ANCHORS.iter().find(|a| l.contains(**a)).map(|a| (i + 1, (*a).to_string())))
            .collect()
    }

    /// Every `.rs` under `members/*/src`, walked from the workspace root.
    fn member_sources(root: &std::path::Path) -> Vec<std::path::PathBuf> {
        fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
            for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if p.extension().is_some_and(|x| x == "rs") {
                    out.push(p);
                }
            }
        }
        let mut out = Vec::new();
        for m in std::fs::read_dir(root.join("members")).into_iter().flatten().flatten() {
            walk(&m.path().join("src"), &mut out);
        }
        out
    }

    /// Every `.rs` under `keel-cli/src`, `keel-cli/tests` and `members/*/src`: the population the
    /// scratch census (D0498) reads. `(path, always_test)` - a file under `tests/` is test code whole.
    fn test_bearing_sources(root: &std::path::Path) -> Vec<(std::path::PathBuf, bool)> {
        fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
            for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if p.extension().is_some_and(|x| x == "rs") {
                    out.push(p);
                }
            }
        }
        let mut src = Vec::new();
        walk(&root.join("keel-cli").join("src"), &mut src);
        let mut out: Vec<(std::path::PathBuf, bool)> = src.into_iter().chain(member_sources(root)).map(|p| (p, false)).collect();
        let mut tests = Vec::new();
        walk(&root.join("keel-cli").join("tests"), &mut tests);
        out.extend(tests.into_iter().map(|p| (p, true)));
        out
    }

    /// The `(line, join text)` pairs the D0498 scratch census names in one source: a
    /// `temp_dir().join(` in TEST CODE whose argument carries no per-process token. Test code is the
    /// region from the first bare `#[cfg(test)]` line, or the first `pub mod test_support` line (a
    /// fixture module shared across the crate boundary, scaffold.rs), to the end of the file - or the
    /// whole file when `always_test` (a `tests/` binary). `process::id()` and `gen_uuid()` are the
    /// tokens; `keel_fs::scratch` never matches because it is not a `temp_dir().join(`.
    fn fixed_scratch_joins(src: &str, always_test: bool) -> Vec<(usize, String)> {
        const NEEDLE: &str = "temp_dir().join(";
        let mut in_test = always_test;
        let mut out = Vec::new();
        for (i, line) in src.lines().enumerate() {
            let t = line.trim();
            if !in_test && (t == "#[cfg(test)]" || t.starts_with("pub mod test_support")) {
                in_test = true;
                continue;
            }
            if !in_test || t.starts_with("//") {
                continue;
            }
            let Some(at) = line.find(NEEDLE) else { continue };
            let call = line.get(at..).unwrap_or("");
            // A needle quoted whole, or a call carrying an escaped quote, is a string literal - this
            // census's own fixtures - not a scratch directory anyone opens.
            let quoted = line.get(..at).is_some_and(|before| before.ends_with('"')) || call.contains("\\\"");
            if quoted || call.contains("process::id()") || call.contains("gen_uuid()") {
                continue;
            }
            out.push((i + 1, call.trim_end().to_string()));
        }
        out
    }

    /// D0388 known-positive, stated before the tree is read: a fixed literal and a `format!` with no
    /// per-process token below the `#[cfg(test)]` line are both named; under `tests/` the fixed join is
    /// named with no `#[cfg(test)]` line at all; the fixture module's own line counts as test code.
    #[test]
    fn the_scratch_census_names_a_fixed_join_in_test_code() {
        let lib = "fn prod() { let _ = std::env::temp_dir().join(\"keel-audit\"); }\n#[cfg(test)]\nmod tests {\n    let a = std::env::temp_dir().join(\"keel-x\");\n    let b = std::env::temp_dir().join(format!(\"keel-k6-{tag}\"));\n}\n";
        let named = fixed_scratch_joins(lib, false);
        assert_eq!(named.iter().map(|(l, _)| *l).collect::<Vec<_>>(), vec![4, 5], "the two test-code joins, not the production one: {named:?}");
        assert!(named[0].1.starts_with("temp_dir().join(\"keel-x\")"), "the census quotes the call: {}", named[0].1);

        let bin = "fn unique_dir() -> PathBuf { std::env::temp_dir().join(format!(\"write_bdd_{n}\")) }\n";
        assert_eq!(fixed_scratch_joins(bin, true).len(), 1, "a tests/ binary is test code whole");

        let fixture = "pub mod test_support {\n    pub fn temp_root(tag: &str) -> PathBuf { std::env::temp_dir().join(format!(\"keel-scaffold-{tag}\")) }\n}\n";
        assert_eq!(fixed_scratch_joins(fixture, false).iter().map(|(l, _)| *l).collect::<Vec<_>>(), vec![2], "a shared fixture module is test code");
    }

    /// D0388 known-negative: the helper, a hand-interpolated process id, a per-call uuid and a fixed
    /// join ABOVE the `#[cfg(test)]` line (the production worktree sites) all pass; so do a comment, a
    /// call inside a string literal and a quoted needle (this census's own fixtures).
    #[test]
    fn the_scratch_census_passes_per_process_joins_and_production_sites() {
        let src = "let scratch = std::env::temp_dir().join(\"keel-audit\");\n#[cfg(test)]\nmod tests {\n    let a = keel_fs::scratch(\"keel-x\");\n    let b = std::env::temp_dir().join(format!(\"keel-x-{}\", std::process::id()));\n    let c = std::env::temp_dir().join(format!(\"keel-eol-{tag}-{}\", crate::ident::gen_uuid()));\n    // let d = std::env::temp_dir().join(\"keel-commented\");\n    let e = \"let x = std::env::temp_dir().join(\\\"keel-in-a-literal\\\");\";\n    const N: &str = \"temp_dir().join(\";\n}\n";
        let named = fixed_scratch_joins(src, false);
        assert!(named.is_empty(), "nothing to name: {named:?}");
    }

    /// Two overlapping lib runs during sprint 724's ceremony produced two different failing sets from
    /// unit tests sharing fixed-name scratch trees (issue570); the fix was thirty-six sites and this is
    /// the control (D0047, D0498): no test-code `temp_dir().join(` names a tree two processes could share.
    #[test]
    fn no_test_names_a_scratch_directory_two_processes_could_share() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .find(|a| a.join(".git").exists())
            .expect("keel-cli sits inside the keel repository")
            .to_path_buf();
        let files = test_bearing_sources(&root);
        assert!(files.len() > 100, "the three trees are the population, asserted non-empty: {}", files.len());
        let mut offenders = Vec::new();
        for (f, always_test) in &files {
            let src = std::fs::read_to_string(f).expect("a source is readable");
            for (line, call) in fixed_scratch_joins(&src, *always_test) {
                offenders.push(format!("{}:{line} {call}", f.strip_prefix(&root).unwrap_or(f).display()));
            }
        }
        assert!(
            offenders.is_empty(),
            "a test scratch directory two processes could share (issue570); name it through keel_fs::scratch(tag) instead:\n{}",
            offenders.join("\n")
        );
    }

    /// Sprint 714 broke four member tests and sprint 718 eight on the same anchor class - a test keyed
    /// on `..` or `src/...` from the crate directory, which names the repo only one level down. Each
    /// was fixed by hand twice; this is the control (D0047). Negative: the four anchors are found in a
    /// synthetic source and a commented one is not. Positive: no member source carries one.
    #[test]
    fn no_member_test_anchors_on_a_cwd_relative_path() {
        let synthetic = "let root = Path::new(\"..\");\n// let x = Path::new(\"..\");\nlet s = read_to_string(\"src/main.rs\");\nlet d = Path::new(env!(\"CARGO_MANIFEST_DIR\")).join(\"..\");\nlet t = read_to_string(\"../x\");\n";
        let found = cwd_relative_anchors(synthetic);
        assert_eq!(found.iter().map(|(l, _)| *l).collect::<Vec<_>>(), vec![1, 3, 4, 5], "the synthetic anchors are found and the comment is not: {found:?}");

        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .find(|a| a.join(".git").exists())
            .expect("keel-cli sits inside the keel repository")
            .to_path_buf();
        let files = member_sources(&root);
        assert!(files.len() > 20, "the member sources are the population, asserted non-empty: {}", files.len());
        let mut offenders = Vec::new();
        for f in &files {
            let src = std::fs::read_to_string(f).expect("a member source is readable");
            for (line, anchor) in cwd_relative_anchors(&src) {
                offenders.push(format!("{}:{line} {anchor}", f.strip_prefix(&root).unwrap_or(f).display()));
            }
        }
        assert!(
            offenders.is_empty(),
            "member tests keyed on a cwd-relative anchor break when the crate sits two levels down (sprints 714, 718); resolve the repo root from CARGO_MANIFEST_DIR's ancestors instead:\n{}",
            offenders.join("\n")
        );
    }
}
