//! Guard family `release` - split from `keel-cli/src/guards.rs` by `scripts/split_guards.py` (sprint 733).
//!
//! Each guard's dispatch arm, code and tests sit together; the shared scanners, the runner and the
//! lock predicates are the crate root's (`super`). Nothing here was retyped: the text is guards.rs's,
//! with `crate::` paths pointing at the members and private items opened to the crate.

use super::*;

/// The `release` family: every guard it dispatches, in `GUARD_NAMES` order, with the tier note each
/// arm carried in `run_one` (sprint 733). The root's union test holds these tables equal to `GUARD_NAMES`.
pub(crate) const FAMILY: Family = Family {
    name: "release",
    arms: &[
        ("gating-workflow-history", gating_workflow_history),
        ("release-recorded", release_recorded), // WARNING-tier (D0191, deploy unit) — a shipped tag with no authored Release item
        ("gate-environment-parity", gate_environment_parity),
        ("release-checksums-published", release_checksums_published), // hard (D0385/issue417) - a published binary with no published hash
        ("wrapper-pin-checksummed", wrapper_pin_checksummed), // WARNING-tier (D0385/issue418) - the pin moved, the wrapper table did not
        ("working-tree-eol", working_tree_eol), // hard (issue478) - a working copy holds the ending .gitattributes declares for it
        ("custom-harness-routed", custom_harness_routed), // hard (issue542/D0475) - every cucumber binary of every member is kept from nextest and run under cargo test
    ],
};

/// Guard 51: a workflow that RUNS the gate must check out full history (D0229/issue260).
///
/// `claim-ancestry` and `audit-history` derive their verdicts from git history and SKIP LOUDLY on a
/// shallow clone - correctly, because a depth-dependent verdict is the machine-dependence K15
/// forbids. The consequence is that a gating workflow which forgets `fetch-depth: 0` silently runs
/// with those guards disabled. That is exactly what happened: `ci.yml` carried the fix (issue229),
/// `release.yml` never got it, and the release gate had been RED since guard 48 landed - undetected
/// for weeks because releases are rare enough that nobody ran it.
///
/// The same-fix-in-N-places class, which is the most-repeated finding in this project's retros. The
/// control is cheap: the fix is a literal, so its absence is decidable.
///
/// STATED LIMITATION: the check is per-FILE, not per-JOB, because there is no YAML parser here. A
/// workflow with two jobs where only the non-gating one sets `fetch-depth: 0` would pass - which is
/// the shape `release.yml` itself has (its build job checks out shallow, legitimately). It catches
/// the real failure class, a gating workflow with no full-history checkout anywhere, and nothing
/// finer. Recorded rather than left for someone to discover.
pub(crate) fn gating_workflow_history(root: &Path) -> GuardReport {
    let dir = root.join(".github").join("workflows");
    let mut scanned = 0usize;
    let mut violations = Vec::new();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return GuardReport { name: "gating-workflow-history", scanned, warnings: Vec::new(), violations };
    };
    let mut files: Vec<std::path::PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()).is_some_and(|e| e == "yml" || e == "yaml"))
        .collect();
    files.sort();
    for path in &files {
        let Ok(text) = keel_model::corpus::read_to_string(path) else { continue };
        // "Runs the gate" is judged by what the workflow actually invokes, not by its name.
        // D0452: the gating verbs are sub-verbs of `gate` and `audit`; a needle reads the routed spelling.
        // D0475: the suite runs under nextest, so that spelling is a test step too.
        let gates = ["cargo test", "cargo nextest", "keel gate guard", "keel gate validate", "audit history", "audit adherence"]
            .iter()
            .any(|needle| text.contains(needle));
        if !gates || !text.contains("actions/checkout") {
            continue;
        }
        scanned += 1;
        if !text.contains("fetch-depth: 0") {
            violations.push(format!(
                "{}: runs the gate but checks out SHALLOW - every history-derived guard (claim-ancestry, audit-history) silently skips, so this workflow reports a pass it did not verify. Add `fetch-depth: 0` to the checkout (issue229/issue260)",
                relpath(root, path)
            ));
        }
    }
    GuardReport { name: "gating-workflow-history", scanned, warnings: Vec::new(), violations }
}

/// Guard 66: a workflow that PUBLISHES release assets hashes them and publishes the hash (D0385/issue417).
///
/// The wrapper contract (`keel-wrapper.toml`, `keelw`) promises verification against "the release
/// page's published checksums", and for three releases the release page carried none: `release.yml`
/// attached the binary and nothing else, so every checksum entry was computed by downloading the
/// asset and hashing it locally - a hash of the very download it was meant to verify, which proves
/// nothing about the bytes the build produced. The checksum has to come out of the run that built
/// the asset, beside it. The human's words, 2026-09-08: "keel release doesn't have a checksum? it
/// needs one" / "make a normal part of process". This guard is what makes it a part of the process
/// rather than a step someone remembers.
///
/// "Publishes release assets" is judged by what the workflow invokes: `softprops/action-gh-release`,
/// `gh release upload` or `gh release create`. Such a workflow must contain a SHA-256 computation
/// (`sha256sum`, `shasum -a 256`, `Get-FileHash`, `certutil -hashfile`) AND name a published hash
/// file (`.sha256` or `SHA256SUMS`). A project with no publishing workflow declares nothing.
///
/// STATED LIMITATION: file-level, like `gating-workflow-history` - there is no YAML parser here, so
/// a hash computed in one job and an upload in another that omits it would pass. It catches the
/// failure class this project had - a publishing workflow with no hashing anywhere - and nothing finer.
pub(crate) fn release_checksums_published(root: &Path) -> GuardReport {
    let dir = root.join(".github").join("workflows");
    let mut scanned = 0usize;
    let mut violations = Vec::new();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return GuardReport { name: "release-checksums-published", scanned, warnings: Vec::new(), violations };
    };
    let mut files: Vec<std::path::PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()).is_some_and(|e| e == "yml" || e == "yaml"))
        .collect();
    files.sort();
    for path in &files {
        let Ok(text) = keel_model::corpus::read_to_string(path) else { continue };
        let publishes = ["action-gh-release", "gh release upload", "gh release create"]
            .iter()
            .any(|needle| text.contains(needle));
        if !publishes {
            continue;
        }
        scanned += 1;
        let hashes = ["sha256sum", "shasum -a 256", "Get-FileHash", "certutil -hashfile"]
            .iter()
            .any(|needle| text.contains(needle));
        let names_hash_file = text.contains(".sha256") || text.contains("SHA256SUMS");
        if !hashes || !names_hash_file {
            violations.push(format!(
                "{}: publishes release assets but {} - a downstream project can only hash the download it is trying to verify. Hash each asset in the job that built it and attach the `.sha256` / `SHA256SUMS` beside it (D0385/issue417)",
                relpath(root, path),
                match (hashes, names_hash_file) {
                    (false, false) => "computes no SHA-256 and publishes no hash file",
                    (false, true) => "computes no SHA-256 (the hash file it names comes from nowhere)",
                    _ => "publishes no `.sha256` / `SHA256SUMS` for what it hashes",
                }
            ));
        }
    }
    GuardReport { name: "release-checksums-published", scanned, warnings: Vec::new(), violations }
}

/// Guard 67 (WARNING-tier): the pinned engine version has a wrapper checksum for every asset keelw
/// can download (D0385/issue418).
///
/// `keelw` reads the pin from `.engine/contracts/engine-version.toml` and refuses a download with
/// no entry in `keel-wrapper.toml` - correctly (never trust-on-first-use). The consequence is that a
/// pin can move without its entries and nothing says so until a fresh clone is refused: this
/// project's pin moved to 0.4.1 on 2026-09-06 while the table carried 0.3.1 only, and every gate
/// stayed green for two days. The asset names mirror `keelw`'s platform case, which is the one
/// place they are declared.
///
/// A WARNING, not a violation: a project fresh from `keel init` has a seeded cache and an empty
/// table for its version, and is not wrong until it copies the entries from the release's
/// `SHA256SUMS` (the release skill's step 3). Absent `keel-wrapper.toml`, nothing is claimed.
pub(crate) fn wrapper_pin_checksummed(root: &Path) -> GuardReport {
    const ASSETS: [&str; 3] = ["keel-linux-x86_64", "keel-macos-aarch64", "keel-windows-x86_64.exe"];
    let mut warnings = Vec::new();
    let Ok(table) = keel_model::corpus::read_to_string(root.join("keel-wrapper.toml")) else {
        return GuardReport { name: "wrapper-pin-checksummed", scanned: 0, warnings, violations: Vec::new() };
    };
    let pin = keel_model::corpus::read_to_string(root.join(".engine").join("contracts").join("engine-version.toml"))
        .ok()
        .and_then(|t| {
            t.lines().find_map(|l| {
                let l = l.trim();
                let rest = l.strip_prefix("engine")?.trim_start().strip_prefix('=')?.trim();
                let rest = rest.strip_prefix('"')?;
                rest.split('"').next().map(str::to_string)
            })
        });
    let Some(pin) = pin else {
        return GuardReport { name: "wrapper-pin-checksummed", scanned: 0, warnings, violations: Vec::new() };
    };
    // the pin's table: from its `["<pin>"]` header to the next `[` header
    let header = format!("[\"{pin}\"]");
    let section: &str = table.find(&header).map_or("", |i| {
        let body = &table[i + header.len()..];
        body.find("\n[").map_or(body, |j| &body[..j])
    });
    let missing: Vec<&str> = ASSETS
        .iter()
        .copied()
        .filter(|a| !section.lines().any(|l| l.trim_start().starts_with(&format!("\"{a}\""))))
        .collect();
    if !missing.is_empty() {
        warnings.push(format!(
            "keel-wrapper.toml: the pinned engine {pin} has no checksum for {} - keelw REFUSES to download it on any clone without a cache (never trust-on-first-use). Copy the entries from the release's SHA256SUMS (release skill, step 3; D0385/issue418)",
            missing.join(", ")
        ));
    }
    GuardReport { name: "wrapper-pin-checksummed", scanned: 1, warnings, violations: Vec::new() }
}

/// Guard 64: every workflow that runs the gate supplies the SAME environment (issue385).
///
/// release.yml ran `cargo test --workspace --release` with no env block; ci.yml ran the same command
/// with `GH_TOKEN`. The federation suite reaches the real repository through `gh`, which refuses
/// inside Actions without a token - so the v0.4.0 release gated RED on a test that is green on every
/// push, the build matrix was skipped, and the tag published nothing. Releases are rare, so the
/// divergence lived until it fired: the issue260 class, one workflow over.
///
/// The expectation is derived from the tree, never hardcoded. A variable supplied to the gate in ANY
/// workflow is one the gate is known to need; a sibling gating workflow that omits it is running the
/// same command in a different world. A project whose gate needs nothing declares nothing and is
/// never accused.
///
/// STATED LIMITATION: file-level, like `gating-workflow-history`, because there is no YAML parser
/// here - a variable set on a non-gating job counts as present. It catches the real class (a gating
/// workflow with the variable nowhere in it) and nothing finer.
pub(crate) fn gate_environment_parity(root: &Path) -> GuardReport {
    let dir = root.join(".github").join("workflows");
    let mut scanned = 0usize;
    let mut violations = Vec::new();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return GuardReport { name: "gate-environment-parity", scanned, warnings: Vec::new(), violations };
    };
    let mut files: Vec<std::path::PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()).is_some_and(|e| e == "yml" || e == "yaml"))
        .collect();
    files.sort();

    // the gating surfaces: those that actually run the test suite - under cargo test or nextest (D0475)
    let mut gating: Vec<(std::path::PathBuf, String)> = Vec::new();
    for path in &files {
        let Ok(text) = keel_model::corpus::read_to_string(path) else { continue };
        if text.contains("cargo test") || text.contains("cargo nextest") {
            gating.push((path.clone(), text));
        }
    }
    if gating.len() < 2 {
        return GuardReport { name: "gate-environment-parity", scanned, warnings: Vec::new(), violations };
    }

    // what any one of them supplies is what the gate is known to need
    let mut expected: Vec<String> = Vec::new();
    for (_, text) in &gating {
        for name in env_names(text) {
            if !expected.contains(&name) {
                expected.push(name);
            }
        }
    }
    expected.sort();
    for (path, text) in &gating {
        scanned += 1;
        let missing: Vec<&String> = expected.iter().filter(|n| !text.contains(n.as_str())).collect();
        if missing.is_empty() {
            continue;
        }
        let names: Vec<&str> = missing.iter().map(|n| n.as_str()).collect();
        violations.push(format!(
            "{}: runs `cargo test` without {} - a sibling gating workflow supplies it, so these two run the same suite in different environments and the rarer one fails on a test the other passes (issue385)",
            relpath(root, path),
            names.join(", ")
        ));
    }
    GuardReport { name: "gate-environment-parity", scanned, warnings: Vec::new(), violations }
}

/// The environment variable names a workflow supplies, read as `NAME: value` under an `env:` key.
///
/// Deliberately narrow: `SCREAMING_SNAKE` keys only, which is what an environment variable looks like
/// and what a YAML step key does not.
pub(crate) fn env_names(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_env = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "env:" {
            in_env = true;
            continue;
        }
        if !in_env {
            continue;
        }
        let Some((key, _)) = trimmed.split_once(':') else {
            in_env = false;
            continue;
        };
        let key = key.trim();
        let looks_like_a_variable = !key.is_empty()
            && key.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
            && key.chars().any(|c| c.is_ascii_uppercase());
        if looks_like_a_variable {
            if !out.contains(&key.to_string()) {
                out.push(key.to_string());
            }
        } else {
            in_env = false;
        }
    }
    out
}

/// An authored `Release` block as guard 43 reads it: `(name, tag, commit)`. `tag` is the record's
/// own `:>> tag` field (D0400) - empty for a milestone that shipped no tag.
pub type ReleaseRow = (String, String, String);

/// The Release whose `tag` field EQUALS `tag`, skipping retired records.
///
/// Pure, so the binding is testable against the two shapes issue387 named: a title that honestly
/// mentions another version, and a tag that is a prefix of a longer one (v0.4.1 / v0.4.11). Neither
/// can match here, because the title is never read and equality is not containment.
#[must_use]
pub fn release_for_tag<'a, S: std::hash::BuildHasher>(tag: &str, releases: &'a [ReleaseRow], retired: &std::collections::HashSet<String, S>) -> Option<&'a ReleaseRow> {
    if tag.is_empty() {
        return None;
    }
    releases.iter().filter(|(n, _, _)| !retired.contains(n)).find(|(_, t, _)| t == tag)
}

/// Guard 43 (D0191, WARNING tier, owned by the `deploy` unit).
///
/// Every local version tag has a `Release` item whose `tag` field names it (D0400) and whose recorded commit matches the tag's commit — "a version was
/// recorded" and "the reconciled version matches the tag" were process-enforcement.toml's own
/// admitted checkable claims, unguarded until now. Zero tags scans zero and passes, so a project
/// without releases (or without the deploy unit) is untouched.
///
/// issue387: the binding used to be `title.contains(tag)`, first hit. A v0.4.0 record whose title
/// said its payload shipped as v0.4.1 was reported as v0.4.1's record, and unanchored containment
/// would let a v0.4.1 record vouch for a v0.4.11 tag. The tag is now the record's own field.
#[must_use]
pub fn release_recorded(root: &Path) -> GuardReport {
    let tags: Vec<String> = keel_git::gitx::git()
        .arg("-C")
        .arg(root)
        .arg("tag")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with('v') && l[1..].starts_with(|c: char| c.is_ascii_digit()))
        .map(str::to_string)
        .collect();
    // issue244/D0214: a Release carrying an inbound `#Supersede` edge is RETIRED, exactly as the
    // sibling views already treat superseded Needs (issue088), requirements (issue127) and tasks
    // (issue100). Without this, a correction made through the engine's OWN sanctioned mechanism -
    // and the only one available to a non-owner (D0108) - could never clear the warning, so the
    // warning was unresolvable by construction and therefore permanent noise.
    let superseded = keel_model::corpus::supersede_targets(root);
    // Every authored Release block: (name, tag, commit).
    let mut releases: Vec<ReleaseRow> = Vec::new();
    for f in keel_model::corpus::collect_sysml(&root.join(".tracking")) {
        let Ok(text) = keel_model::corpus::read_to_string(&f) else { continue };
        let mut name = String::new();
        let mut tag = String::new();
        let mut commit = String::new();
        let mut in_release = false;
        for line in text.lines() {
            let l = line.trim_start();
            if l.starts_with("part ") && l.contains(": Release {") {
                in_release = true;
                name = l.trim_start_matches("part ").split_whitespace().next().unwrap_or("").to_string();
                tag.clear();
                commit.clear();
            }
            if in_release {
                if let Some(v) = l.strip_prefix(":>> tag = \"") {
                    tag = v.split('"').next().unwrap_or("").to_string();
                }
                if let Some(v) = l.strip_prefix(":>> commit = \"") {
                    commit = v.split('"').next().unwrap_or("").to_string();
                }
                if l.trim_end() == "}" {
                    in_release = false;
                    releases.push((name.clone(), tag.clone(), commit.clone()));
                }
            }
        }
    }
    let mut warnings = Vec::new();
    for tag in &tags {
        let Some((_, _, recorded)) = release_for_tag(tag, &releases, &superseded) else {
            warnings.push(format!(
                "tag `{tag}` has NO Release item whose `tag` field names it — what shipped is not an authored fact (D0191/D0400; record it in .tracking/baselines.sysml with `:>> tag = \"{tag}\"`)"
            ));
            continue;
        };
        let tag_commit = keel_git::gitx::git()
            .arg("-C")
            .arg(root)
            .args(["rev-parse", "--short", &format!("{tag}^{{commit}}")])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        if !tag_commit.is_empty() && !recorded.is_empty() && !tag_commit.starts_with(recorded.as_str()) && !recorded.starts_with(tag_commit.as_str()) {
            warnings.push(format!(
                "tag `{tag}` points at {tag_commit} but its Release item records commit {recorded} — the recorded release and the shipped tag disagree (D0191)"
            ));
        }
    }
    GuardReport { name: "release-recorded", scanned: tags.len(), warnings, violations: Vec::new() }
}

// ── custom-harness-routed guard (issue542: the CI nextest split mirrored one manifest of two) ─────

/// Guard: every `harness = false` test binary of every workspace member is ROUTED by every workflow
/// that runs nextest - kept out of `cargo nextest run` and handed to `cargo test` (issue542, D0475).
///
/// `cargo nextest run --workspace` asks each test binary for `--list`; a cucumber binary answers
/// `unexpected argument '--list'` and nextest exits 104 before one test runs. So ci.yml and
/// release.yml exclude those binaries with `-E 'not (binary(a) | ...)'` and run them under `cargo
/// test -p <crate> --test <name>`. At 1e2ed25 that split was mirrored by hand from keel-cli's manifest
/// alone: keel-parser declares four more (`lexer_bdd`, `parser_bdd`, `semantic_bdd`, `spec_compat_bdd`),
/// nextest listed the first of them, and CI concluded failure while the local touched run - scoped to
/// `-p keel-cli`, so it never meets them - was green. sprint707's retro had looked at exactly this
/// drift and dismissed it: "the manifest is read". One of two was.
///
/// The expectation is DERIVED from the tree: the `[workspace] members` of the root manifest, each
/// member's `[[test]]` targets with `harness = false` (the same reader `keel suite --touched` uses,
/// `touched::custom_harness_tests`). Per gating workflow, a declared binary the nextest line does not
/// exclude, or that no `cargo test -p <its crate> ... --test <name>` line runs, is a violation; so is
/// a name a workflow routes that no manifest declares (a renamed or deleted binary the split still
/// names). A workspace with no custom harness, or a project with no workflows, is never accused.
///
/// STATED LIMITATION: text-level, like `gate-environment-parity` - `binary(x)` anywhere on a
/// `cargo nextest run` line counts as excluded, and the crate is the `-p` on the `cargo test` line.
/// It catches the class that fired (a crate's cucumber binaries absent from the split) and nothing
/// finer; a `--test` spelled on a continuation line is not read.
pub(crate) fn custom_harness_routed(root: &Path) -> GuardReport {
    let name = "custom-harness-routed";
    let declared = workspace_custom_harnesses(root);
    let mut scanned = 0usize;
    let mut violations = Vec::new();
    let dir = root.join(".github").join("workflows");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return GuardReport { name, scanned, warnings: Vec::new(), violations };
    };
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()).is_some_and(|e| e == "yml" || e == "yaml"))
        .collect();
    files.sort();
    for path in &files {
        let Ok(text) = keel_model::corpus::read_to_string(path) else { continue };
        if !text.contains("cargo nextest run") {
            continue;
        }
        scanned += 1;
        let (missing, stale) = custom_harness_drift(&text, &declared);
        if !missing.is_empty() {
            violations.push(format!(
                "{}: `cargo nextest run` would be handed {} - a harness = false binary answers --list with an error and nextest exits 104 before one test runs (issue542, 1e2ed25). Exclude it with binary(<name>) and run it under `cargo test -p <crate> --test <name>`",
                relpath(root, path),
                missing.join(", ")
            ));
        }
        if !stale.is_empty() {
            violations.push(format!(
                "{}: routes {} that no workspace manifest declares as a harness = false test - a renamed or deleted binary the split still names",
                relpath(root, path),
                stale.join(", ")
            ));
        }
    }
    GuardReport { name, scanned, warnings: Vec::new(), violations }
}

/// `(crate, binary)` for every `harness = false` `[[test]]` target of every `[workspace] member`,
/// sorted by crate then binary so a violation line reads the same whatever order `members` lists.
pub(crate) fn workspace_custom_harnesses(root: &Path) -> Vec<(String, String)> {
    let Ok(ws) = std::fs::read_to_string(root.join("Cargo.toml")) else { return Vec::new() };
    let Some(members) = ws.split("members = [").nth(1).and_then(|s| s.split(']').next()) else { return Vec::new() };
    let mut out = Vec::new();
    for member in members.split(',').map(|m| m.trim().trim_matches('"')).filter(|m| !m.is_empty()) {
        let Ok(manifest) = std::fs::read_to_string(root.join(member).join("Cargo.toml")) else { continue };
        let crate_name = manifest
            .lines()
            .map(str::trim)
            .find_map(|l| l.strip_prefix("name = ").map(|v| v.trim_matches('"').to_string()))
            .unwrap_or_else(|| member.to_string());
        for test in keel_model::corpus::custom_harness_tests(&manifest) {
            out.push((crate_name.clone(), test));
        }
    }
    out.sort();
    out
}

/// What a workflow text keeps away from nextest and hands to `cargo test`, read line by line.
pub(crate) fn custom_harness_split(workflow: &str) -> (Vec<String>, Vec<(String, String)>) {
    let mut excluded = Vec::new();
    let mut cargo_test = Vec::new();
    for line in workflow.lines().map(str::trim) {
        if line.starts_with("cargo nextest run") {
            for piece in line.split("binary(").skip(1) {
                if let Some(n) = piece.split(')').next() {
                    excluded.push(n.trim().to_string());
                }
            }
        }
        if line.starts_with("cargo test") {
            let words: Vec<&str> = line.split_whitespace().collect();
            let pairs = || words.iter().zip(words.iter().skip(1));
            if let Some(crate_name) = pairs().find(|(flag, _)| **flag == "-p").map(|(_, v)| (*v).to_string()) {
                for (_, v) in pairs().filter(|(flag, _)| **flag == "--test") {
                    cargo_test.push((crate_name.clone(), (*v).to_string()));
                }
            }
        }
    }
    (excluded, cargo_test)
}

/// The declared binaries a workflow text does not route (`crate::binary`), and the names it routes
/// that nothing declares - empty on both counts is the only pass.
pub(crate) fn custom_harness_drift(workflow: &str, declared: &[(String, String)]) -> (Vec<String>, Vec<String>) {
    let (excluded, cargo_test) = custom_harness_split(workflow);
    let missing: Vec<String> = declared
        .iter()
        .filter(|(c, t)| !excluded.contains(t) || !cargo_test.iter().any(|(rc, rt)| rc == c && rt == t))
        .map(|(c, t)| format!("{c}::{t}"))
        .collect();
    let mut stale: Vec<String> = cargo_test
        .iter()
        .filter(|(c, t)| !declared.iter().any(|(dc, dt)| dc == c && dt == t))
        .map(|(c, t)| format!("{c}::{t}"))
        .collect();
    stale.extend(excluded.iter().filter(|t| !declared.iter().any(|(_, dt)| dt == *t)).map(|t| format!("?::{t}")));
    (missing, stale)
}

// ── working-tree-eol guard (issue478: the working tree's line endings decided the touched run's verdict) ──

/// Guard: a tracked path holds the line ending its `.gitattributes` declares (issue478).
///
/// Every path whose attribute declares an ending (`eol=lf` / `eol=crlf`) is judged in its WORKING
/// COPY - read from `git ls-files --eol`, one call, the `w/` column against the `attr/` column
/// (`keel_git::eol`).
///
/// Git normalises a declared file at the add, so a CRLF working copy of an `eol=lf` file is clean
/// to `git status` and invisible to every diff - and is exactly what `keel suite --touched` and `keel
/// land` compile and test (issue477: they test the working tree). On 2026-09-11 that turned one
/// session's tool choice (a Python `write_text`, the harness Write tool) into five test failures nothing
/// in the change could reach, and a census found 140 more silent CRLF paths in the clone. This guard
/// names every such path; the touched run and `land` refuse on the same census before cargo starts.
///
/// HARD. A path whose attribute declares no ending (`text=auto`, `-text`, none) is outside the check:
/// its ending is whatever `core.autocrlf` chose, which the repository chose not to pin. A tree git
/// cannot list (not a repository) has nothing to judge and reports zero scanned.
#[must_use]
pub fn working_tree_eol(root: &Path) -> GuardReport {
    let Ok(c) = keel_git::eol::census(root) else {
        return GuardReport { name: "working-tree-eol", scanned: 0, warnings: Vec::new(), violations: Vec::new() };
    };
    let violations: Vec<String> = c
        .mismatches
        .iter()
        .map(|m| format!("{}: the working copy is {} while .gitattributes declares eol={} (issue478) - a writer that translates newlines produced it; git would normalise it at the commit, but every run over the working tree reads these bytes. {}", m.path, m.worktree, m.declared, keel_git::eol::REMEDY))
        .collect();
    GuardReport { name: "working-tree-eol", scanned: c.scanned, warnings: Vec::new(), violations }
}

#[cfg(test)]
mod release_tag_tests {
    use super::{release_for_tag, ReleaseRow};
    use std::collections::HashSet;

    fn row(name: &str, tag: &str, commit: &str) -> ReleaseRow {
        (name.to_string(), tag.to_string(), commit.to_string())
    }

    /// THE CONTROL for issue387 (D0400): the binding is the `tag` field, exact - a title that
    /// mentions another version is never consulted, a prefix tag never matches a longer one, and a
    /// retired record does not vouch.
    #[test]
    fn a_release_is_bound_to_its_tag_by_the_field_and_nothing_else() {
        // release040's title used to end "the payload shipped as v0.4.1" (reworded at 3dd4c2c) - honest
        // prose that, under containment, made it the first hit for the v0.4.1 tag.
        let rows = vec![row("release040", "v0.4.0", "2c288d8"), row("release041", "v0.4.1", "b171cd7"), row("release0411", "v0.4.11", "aaaaaaa"), row("milestone", "", "61850f4")];
        let none = HashSet::new();
        assert_eq!(release_for_tag("v0.4.1", &rows, &none).map(|r| r.0.as_str()), Some("release041"), "v0.4.1 binds to its own record, not the one whose prose mentions it");
        assert_eq!(release_for_tag("v0.4.0", &rows, &none).map(|r| r.0.as_str()), Some("release040"));
        assert_eq!(release_for_tag("v0.4.11", &rows, &none).map(|r| r.0.as_str()), Some("release0411"), "v0.4.11 is not vouched for by v0.4.1");
        assert_eq!(release_for_tag("v0.4.12", &rows, &none), None, "a tag with no record is the D0191 violation, not a near match");
        assert_eq!(release_for_tag("", &rows, &none), None, "an untagged milestone never binds to anything");
        let retired: HashSet<String> = std::iter::once("release041".to_string()).collect();
        assert_eq!(release_for_tag("v0.4.1", &rows, &retired), None, "a retired record does not vouch (issue244/D0214)");
    }
}

#[cfg(test)]
mod working_tree_eol_tests {
    use std::path::{Path, PathBuf};

    fn git(dir: &Path, args: &[&str]) {
        let o = keel_git::gitx::git().arg("-C").arg(dir).args(args).output().unwrap();
        assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
    }

    /// A repository declaring `*.sysml text eol=lf` and `*.bat text eol=crlf`, autocrlf OFF so the
    /// fixture's bytes are what git sees on every host (CI is Linux; this host is Windows).
    fn fixture(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("keel-eol-{tag}-{}", keel_model::ident::gen_uuid()));
        std::fs::create_dir_all(dir.join(".tracking")).unwrap();
        git(&dir, &["init", "-q"]);
        git(&dir, &["config", "core.autocrlf", "false"]);
        std::fs::write(dir.join(".gitattributes"), "* text=auto\n*.sysml text eol=lf\n*.bat text eol=crlf\n").unwrap();
        std::fs::write(dir.join(".tracking").join("a.sysml"), "package A {\n}\n").unwrap();
        std::fs::write(dir.join("run.bat"), "@echo off\r\necho hi\r\n").unwrap();
        std::fs::write(dir.join("notes"), "no attribute here\r\n").unwrap();
        git(&dir, &["add", "-A"]);
        git(&dir, &["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "seed"]);
        dir
    }

    /// D0388 known-positive: one `eol=lf` file rewritten CRLF - the guard fails naming that one path.
    #[test]
    fn a_crlf_copy_of_an_eol_lf_file_fails_naming_the_path() {
        let dir = fixture("pos");
        std::fs::write(dir.join(".tracking").join("a.sysml"), "package A {\r\n}\r\n").unwrap();
        let r = super::working_tree_eol(&dir);
        assert_eq!(r.violations.len(), 1, "{:?}", r.violations);
        assert!(r.violations[0].starts_with(".tracking/a.sysml: the working copy is crlf while .gitattributes declares eol=lf"), "{}", r.violations[0]);
        assert_eq!(r.scanned, 2, "a.sysml and run.bat declare an ending; `notes` does not");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// D0388 known-negative: the same repository with the file LF passes; the `eol=crlf` file that IS
    /// CRLF is not named; the file with no declared ending is not judged whatever it holds.
    #[test]
    fn a_conforming_tree_passes_and_an_undeclared_ending_is_not_judged() {
        let dir = fixture("neg");
        let r = super::working_tree_eol(&dir);
        assert!(r.violations.is_empty(), "{:?}", r.violations);
        assert_eq!(r.scanned, 2);
        // and a tree git cannot list has nothing to judge
        let plain = std::env::temp_dir().join(format!("keel-eol-plain-{}", keel_model::ident::gen_uuid()));
        std::fs::create_dir_all(&plain).unwrap();
        let none = super::working_tree_eol(&plain);
        assert_eq!((none.scanned, none.violations.len()), (0, 0));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&plain);
    }
}

#[cfg(test)]
mod custom_harness_routed_tests {
    use super::{custom_harness_drift, custom_harness_routed, workspace_custom_harnesses};

    fn declared() -> Vec<(String, String)> {
        [("keel-cli", "cli_bdd"), ("keel-cli", "orient_bdd"), ("keel-cli", "write_bdd"), ("keel-parser", "lexer_bdd"), ("keel-parser", "parser_bdd"), ("keel-parser", "semantic_bdd"), ("keel-parser", "spec_compat_bdd")]
            .iter()
            .map(|(c, t)| ((*c).to_string(), (*t).to_string()))
            .collect()
    }

    /// D0388 known-positive: the split as committed at 1e2ed25 - keel-cli's trio alone - against the
    /// seven the workspace declares names keel-parser's four as missing. This is the CI failure of
    /// 2026-09-14 (`gh run view 34825100902`, nextest exit 104), constructed rather than described.
    #[test]
    fn the_split_of_1e2ed25_names_keel_parsers_four_cucumber_binaries_as_missing() {
        let as_committed = "run: |\n          cargo nextest run --workspace --release --no-fail-fast -E 'not (binary(cli_bdd) | binary(orient_bdd) | binary(write_bdd))'\n          cargo test --release -p keel-cli --no-fail-fast --test cli_bdd --test orient_bdd --test write_bdd\n";
        let (missing, stale) = custom_harness_drift(as_committed, &declared());
        assert_eq!(missing, vec!["keel-parser::lexer_bdd", "keel-parser::parser_bdd", "keel-parser::semantic_bdd", "keel-parser::spec_compat_bdd"]);
        assert!(stale.is_empty(), "the trio it does name is declared: {stale:?}");
    }

    /// D0388 known-negative: a split naming all seven, each excluded and each run under its crate,
    /// drifts on neither count; and an exclusion with no `cargo test` line is a hole, not a route.
    #[test]
    fn a_split_naming_every_declared_binary_under_its_crate_is_clean() {
        let full = "cargo nextest run --workspace --release --no-fail-fast -E 'not (binary(cli_bdd) | binary(orient_bdd) | binary(write_bdd) | binary(lexer_bdd) | binary(parser_bdd) | binary(semantic_bdd) | binary(spec_compat_bdd))'\ncargo test --release -p keel-cli --no-fail-fast --test cli_bdd --test orient_bdd --test write_bdd\ncargo test --release -p keel-parser --no-fail-fast --test lexer_bdd --test parser_bdd --test semantic_bdd --test spec_compat_bdd\n";
        assert_eq!(custom_harness_drift(full, &declared()), (Vec::<String>::new(), Vec::<String>::new()));
        let hole = "cargo nextest run --workspace -E 'not (binary(cli_bdd))'\n";
        let one = vec![("keel-cli".to_string(), "cli_bdd".to_string())];
        assert_eq!(custom_harness_drift(hole, &one).0, vec!["keel-cli::cli_bdd"]);
        // a name the manifests no longer declare is stale, under the crate it was routed for
        let renamed = "cargo nextest run --workspace -E 'not (binary(cli_bdd) | binary(old_bdd))'\ncargo test -p keel-cli --test cli_bdd --test old_bdd\n";
        let (missing, stale) = custom_harness_drift(renamed, &one);
        assert!(missing.is_empty());
        assert_eq!(stale, vec!["keel-cli::old_bdd", "?::old_bdd"]);
    }

    /// The real tree: both members' manifests declare the seven, and the workflows as committed route
    /// every one of them - the guard's own answer over this repository is a pass with two scanned.
    #[test]
    fn this_workspace_declares_seven_and_both_workflows_route_them() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap();
        assert_eq!(workspace_custom_harnesses(root), declared());
        let report = custom_harness_routed(root);
        assert_eq!(report.scanned, 2, "ci.yml and release.yml run nextest");
        assert!(report.violations.is_empty(), "{:?}", report.violations);
    }
}
