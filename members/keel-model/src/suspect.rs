//! The suspect walk: which DONE work can no longer be trusted, and why.
//!
//! Step 3 of an orient run reads each done task's dependencies' `DoD` criterion as it stood at the
//! commit the task was verified against (one batched blob fetch, cached in `gitfacts`) and marks the
//! task suspect when that text changed; step 4 propagates suspicion up the dependency graph; step 5
//! marks manifest deliverables whose own source drifted (D0050). Below the views (D0479,
//! dcGuardsViewOrientCycleIsBroken): coverage, readiness and the reports read [`suspect`], not orient.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::gitfacts::batch_cat_blobs;
use crate::indexer::TaskData;

/// Read criterion text for `task`'s `DoD` at git commit `sha`. The per-task FALLBACK for when the
/// batched blob fetch (`batch_cat_blobs`) missed — e.g. the `DoD` moved files since `sha`.
///
/// orientPerf/issue063: ONE `git grep -h` for the `DoD`'s (single-line) declaration at `sha`, instead
/// of `git ls-tree` + a `git show` per `.sysml` file — many fewer subprocess spawns (the Windows
/// process-creation/AV tail-latency cost). `-h` yields the raw matched line(s), so the extraction below
/// is IDENTICAL to the pre-change ls-tree+show path (a `DoD` is authored on one line in this model).
fn git_criterion_at(sha: &str, task: &str, repo: &Path) -> Option<String> {
    // A fact about an immutable commit: served from the content-addressed cache when `sha` is a full
    // id (dcGitFactsAreContentAddressed); a short id runs the grep every time.
    if let Some(cached) = crate::gitfacts::grep(repo, sha, task) {
        return cached;
    }
    let found = git_criterion_at_uncached(sha, task, repo);
    crate::gitfacts::remember_grep(repo, sha, task, found.as_deref());
    found
}

fn git_criterion_at_uncached(sha: &str, task: &str, repo: &Path) -> Option<String> {
    let dod_pfx = format!("verification {task}DoD");
    let grep = keel_git::gitx::git()
        .arg("-C")
        .arg(repo)
        .args(["grep", "-h", "-F", "--no-color", "-e", &dod_pfx, sha, "--", ".tracking"])
        .output()
        .ok()?;
    if !grep.status.success() {
        return None; // no match (or git error) — conservative, matches the old "not found" path
    }
    let content = String::from_utf8(grep.stdout).ok()?;
    for line in content.lines() {
        if line.trim().starts_with(&dod_pfx) {
            // Extract procedureText from the line (unchanged from the ls-tree+show path).
            let pat = "procedureText = \"";
            let start = line.find(pat)? + pat.len();
            return string_literal_body(&line[start..]);
        }
    }
    None
}

/// Extract a `<task>DoD` verification's `procedureText` from a file's content (no git). The value is
/// UNESCAPED the way the parser reads it, so it compares equal to the model's text.
fn extract_dod_criterion(content: &str, task: &str) -> Option<String> {
    let pfx = format!("verification {task}DoD");
    for line in content.lines() {
        if line.trim_start().starts_with(&pfx) {
            let pat = "procedureText = \"";
            let start = line.find(pat)? + pat.len();
            return string_literal_body(&line[start..]);
        }
    }
    None
}

/// The body of a `SysML` string literal whose opening quote has been consumed, read with the lexer's
/// escape rules (`\n` `\t` `\"` `\\`) so a historical blob compares equal to the parsed model text.
/// Cutting at the first `"` and skipping the unescape is the issue044 false-stale class: a criterion
/// carrying `\s` or an escaped quote read as CHANGED against itself. `None` when the literal never
/// closes or carries an escape the lexer would refuse - then there is no text to judge.
#[must_use]
pub fn string_literal_body(rest: &str) -> Option<String> {
    let mut out = String::new();
    let mut chars = rest.chars();
    loop {
        match chars.next()? {
            '"' => return Some(out),
            '\\' => match chars.next()? {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                _ => return None,
            },
            c => out.push(c),
        }
    }
}

/// Map every CURRENT `<task>DoD` verification to its repo-relative file path (one working-tree
/// pass, no git). Lets criterion lookup fetch a single historical blob instead of scanning all
/// files at the commit — the dominant `orient` cost (orientPerf, sr11FastStart).
pub fn build_dod_files(repo: &Path) -> HashMap<String, String> {
    let mut out: HashMap<String, String> = HashMap::new();
    for path in crate::corpus::collect_sysml(&repo.join(".tracking")) {
        let Ok(text) = std::fs::read_to_string(&path) else { continue };
        let Some(rel) = path.strip_prefix(repo).ok().and_then(std::path::Path::to_str).map(|s| s.replace('\\', "/")) else {
            continue;
        };
        for line in text.lines() {
            if let Some(rest) = line.trim_start().strip_prefix("verification ") {
                let name = rest.split([' ', ':']).next().unwrap_or("");
                if let Some(task) = name.strip_suffix("DoD") {
                    out.entry(task.to_owned()).or_insert_with(|| rel.clone());
                }
            }
        }
    }
    out
}

/// Compute criterion-change suspects (step 3) with a SINGLE batched blob fetch (orientPerf): a done
/// task is suspect if a non-ordering dep's `DoD` criterion text changed since the task's verified
/// commit. Returns `(task, reason)` pairs. Falls back to a per-call scan only for blobs the batch
/// couldn't resolve (rare — a `DoD` file absent at that commit).
#[must_use]
pub fn criterion_suspects(
    repo: &Path,
    tasks: &HashMap<String, TaskData>,
    ordering_only: &HashSet<(String, String)>,
    done_map: &HashMap<String, bool>,
    verified_at: &HashMap<String, String>,
    dod_files: &HashMap<String, String>,
) -> Vec<(String, String)> {
    let is_done = |n: &str| done_map.get(n).copied().unwrap_or(false);
    // Pass 1: gather the distinct (ct, file, task) criteria every (task, dep) comparison will need.
    let mut wanted: HashSet<(String, String, String)> = HashSet::new();
    for (name, data) in tasks {
        if !is_done(name) {
            continue;
        }
        let Some(ct) = verified_at.get(name.as_str()) else { continue };
        // The task's OWN criterion too (dcOwnDoDDriftIsSuspect): the thing verified must be the thing
        // agreed, and the owner may otherwise rewrite it after the pass with the pass still standing.
        if let Some(file) = dod_files.get(name.as_str()) {
            wanted.insert((ct.clone(), file.clone(), name.clone()));
        }
        for dep in &data.deps {
            if !ordering_only.contains(&(dep.clone(), name.clone())) {
                if let Some(file) = dod_files.get(dep) {
                    wanted.insert((ct.clone(), file.clone(), dep.clone()));
                }
            }
        }
    }
    // The criterion of `task` as `file` held it at `ct` is a fact about an immutable commit: answered
    // from the content-addressed cache when `ct` is a full id (dcGitFactsAreContentAddressed), and only
    // the misses cost the one batched blob fetch. `None` means the file at `ct` holds no such criterion.
    let mut criteria: HashMap<(String, String, String), Option<String>> = HashMap::new();
    let mut keys: HashSet<String> = HashSet::new();
    let mut misses: Vec<(String, String, String)> = Vec::new();
    for w in wanted {
        if let Some(known) = crate::gitfacts::criterion(repo, &w.0, &w.1, &w.2) {
            criteria.insert(w, known);
        } else {
            keys.insert(format!("{}:{}", w.0, w.1));
            misses.push(w);
        }
    }
    let key_vec: Vec<String> = keys.into_iter().collect();
    let blobs = batch_cat_blobs(repo, &key_vec);
    for w in misses {
        let found = blobs
            .get(&format!("{}:{}", w.0, w.1))
            .cloned()
            .flatten()
            .and_then(|content| extract_dod_criterion(&content, &w.2));
        crate::gitfacts::remember_criterion(repo, &w.0, &w.1, &w.2, found.as_deref());
        criteria.insert(w, found);
    }
    let criterion_of = |ct: &str, task: &str| -> Option<String> {
        dod_files
            .get(task)
            .and_then(|file| criteria.get(&(ct.to_string(), file.clone(), task.to_string())))
            .cloned()
            .flatten()
    };
    let head = keel_git::gitx::git()
        .arg("-C")
        .arg(repo)
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .map_or_else(|_| "HEAD".to_string(), |o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    // Pass 2: compare each dep's historical criterion (from the batch) to its current text.
    let mut out: Vec<(String, String)> = Vec::new();
    for (name, data) in tasks {
        if !is_done(name) {
            continue;
        }
        let Some(ct) = verified_at.get(name.as_str()) else { continue };
        // OWN criterion first. Compared the same way a dependency's is: the text at the verified SHA
        // against the text at HEAD. A criterion that did not yet exist at that SHA (a DoD recorded in
        // the same commit as its result) has no historical text and is not judged - stated, not hidden:
        // that is the limit of comparing against the verified commit, and the sibling helper for
        // acceptances (issue341) shares it.
        let own_cur = data.dod_text.as_deref().unwrap_or("");
        let own_old = criterion_of(ct, name);
        if own_old.as_deref().is_some_and(|old| old != own_cur) {
            out.push((name.clone(), format!("OWN criterion of '{name}' changed: the text at {head} (HEAD) is not the text the pass judged at {ct} (dcOwnDoDDriftIsSuspect) - re-verify against the new criterion or restore it")));
            continue;
        }
        for dep in &data.deps {
            if ordering_only.contains(&(dep.clone(), name.clone())) {
                continue;
            }
            let Some(dep_data) = tasks.get(dep.as_str()) else { continue };
            let cur = dep_data.dod_text.as_deref().unwrap_or("");
            let old = criterion_of(ct, dep).or_else(|| git_criterion_at(ct, dep, repo));
            if let Some(old) = old {
                if old != cur {
                    out.push((name.clone(), format!("criterion of dependency '{dep}' changed since verified at {ct}")));
                    break;
                }
            }
        }
    }
    crate::gitfacts::flush(repo);
    out
}

/// Load the PER-TASK deliverable-suspicion manifest (D0050; suspectDiagnostics): a list of
/// `(task, its source paths)`. Missing manifest → empty (feature inert).
/// Format: `task: <name> | <space-separated paths>`; `#` = comment.
fn load_deliverable_manifest(repo: &Path) -> Vec<(String, Vec<String>)> {
    let mut out = Vec::new();
    let Ok(text) = std::fs::read_to_string(repo.join(".engine").join("deliverable-manifest.txt"))
    else {
        return out;
    };
    for line in text.lines() {
        let l = line.trim();
        if let Some(rest) = l.strip_prefix("task:") {
            let mut parts = rest.splitn(2, '|');
            let task = parts.next().unwrap_or("").trim().to_owned();
            let paths: Vec<String> = parts.next().unwrap_or("").split_whitespace().map(str::to_owned).collect();
            if !task.is_empty() {
                out.push((task, paths));
            }
        }
    }
    out
}

/// Repo-relative paths changed in `<sha>..HEAD` (one `git diff --name-only` spawn). Empty on git
/// failure (conservative: no drift). Forward slashes (git's native output).
fn changed_paths_since(repo: &Path, sha: &str) -> Vec<String> {
    if sha.is_empty() {
        return Vec::new();
    }
    // The diff between two commits is a fact about them (dcGitFactsAreContentAddressed): keyed by
    // `sha` and HEAD's FULL id, both of which must be full for the cache to answer or remember.
    // A git failure returns empty (conservative: no drift) and is never remembered.
    let head = crate::gitfacts::head_sha(repo);
    if let Some(h) = head.as_deref() {
        if let Some(cached) = crate::gitfacts::changed(repo, sha, h) {
            return cached;
        }
    }
    let diffed: Option<Vec<String>> = keel_git::gitx::git()
        .arg("-C").arg(repo)
        .args(["diff", "--name-only", &format!("{sha}..HEAD")])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().map(str::trim).filter(|l| !l.is_empty()).map(String::from).collect());
    match (diffed, head.as_deref()) {
        (Some(paths), Some(h)) => {
            crate::gitfacts::remember_changed(repo, sha, h, &paths);
            crate::gitfacts::flush(repo);
            paths
        }
        (Some(paths), None) => paths,
        (None, _) => Vec::new(),
    }
}

/// True if a manifest `path` (file or directory) contains any `changed` repo-relative path.
fn path_drifted(path: &str, changed: &[String]) -> bool {
    let dir_prefix = format!("{path}/");
    changed.iter().any(|c| c == path || c.starts_with(&dir_prefix))
}

/// Mark manifest deliverable tasks suspect when THEIR OWN source drifted since they were verified
/// (D0050, per-task). Perf (orientPerf/sr11): ONE `git diff` per DISTINCT verified-commit (memoized)
/// instead of one `git log` per task — then prefix-match paths in-process.
pub fn apply_deliverable_suspicion(
    repo: &Path,
    done_map: &HashMap<String, bool>,
    verified_at: &HashMap<String, String>,
    suspect_set: &mut HashSet<String>,
    reasons: &mut HashMap<String, String>,
) {
    let mut changed_by_ct: HashMap<String, Vec<String>> = HashMap::new();
    for (name, paths) in load_deliverable_manifest(repo) {
        if paths.is_empty() || !done_map.get(name.as_str()).copied().unwrap_or(false) {
            continue;
        }
        let Some(ct) = verified_at.get(name.as_str()) else { continue };
        let changed = changed_by_ct.entry(ct.clone()).or_insert_with(|| changed_paths_since(repo, ct));
        if let Some(hit) = paths.iter().find(|p| path_drifted(p, changed)) {
            suspect_set.insert(name.clone());
            let hit = hit.clone();
            reasons.entry(name.clone()).or_insert_with(|| {
                format!("deliverable source [{}] drifted since verified at {ct} (e.g. {hit})", paths.join(", "))
            });
        }
    }
}

/// Propagate suspicion up the dependency graph to fixpoint: a done task is suspect if any of its
/// (non-ordering-only) deps is suspect. Extracted from `compute_orient` (step 4).
pub fn propagate_transitive_suspect(
    tasks: &HashMap<String, TaskData>,
    ordering_only: &HashSet<(String, String)>,
    done_map: &HashMap<String, bool>,
    suspect_set: &mut HashSet<String>,
    suspect_reasons: &mut HashMap<String, String>,
) {
    let mut changed = true;
    while changed {
        changed = false;
        for (name, data) in tasks {
            if !done_map.get(name.as_str()).copied().unwrap_or(false) || suspect_set.contains(name.as_str()) {
                continue;
            }
            for dep in &data.deps {
                if ordering_only.contains(&(dep.clone(), name.clone())) {
                    continue;
                }
                if suspect_set.contains(dep.as_str()) {
                    suspect_set.insert(name.clone());
                    suspect_reasons.insert(name.clone(), format!("transitively suspect: dependency '{dep}' is suspect"));
                    changed = true;
                    break;
                }
            }
        }
    }
}

/// Steps 3-5 of an orient run over an already-classified evidence set: criterion-change suspects,
/// their transitive closure, then deliverable drift (D0050). Returns the sorted suspect list and the
/// per-task reason `suspect --explain` shows.
#[must_use]
pub fn walk(
    repo: &Path,
    tasks: &HashMap<String, TaskData>,
    ordering_only: &HashSet<(String, String)>,
    done_map: &HashMap<String, bool>,
    verified_at: &HashMap<String, String>,
) -> (Vec<String>, HashMap<String, String>) {
    // Perf (orientPerf/sr11): one batched `git cat-file` reads ALL needed historical DoD blobs.
    let dod_files = build_dod_files(repo);
    let mut suspect_set: HashSet<String> = HashSet::new();
    let mut suspect_reasons: HashMap<String, String> = HashMap::new();
    for (name, reason) in criterion_suspects(repo, tasks, ordering_only, done_map, verified_at, &dod_files) {
        suspect_set.insert(name.clone());
        suspect_reasons.entry(name).or_insert(reason);
    }
    propagate_transitive_suspect(tasks, ordering_only, done_map, &mut suspect_set, &mut suspect_reasons);
    apply_deliverable_suspicion(repo, done_map, verified_at, &mut suspect_set, &mut suspect_reasons);
    let mut suspect: Vec<String> = suspect_set.into_iter().collect();
    suspect.sort();
    (suspect, suspect_reasons)
}

/// The suspect set alone, sorted - what the coverage and readiness views read (they used to read it
/// off a whole `orient::compute`, which is how the views came to depend on orient).
#[must_use]
pub fn suspect(root: &Path) -> Vec<String> {
    let idx = crate::indexer::extract(&root.join(".tracking"));
    let ev = crate::evidence::classify(root, &idx.tasks, false, true);
    walk(root, &idx.tasks, &idx.ordering_only, &ev.done_map, &ev.verified_at).0
}

#[cfg(test)]
mod tests {
    use super::{extract_dod_criterion, string_literal_body};

    /// THE CONTROL for the issue044 class reaching the criterion reader (dcResultBindsToItsLandingCommit
    /// exposed it: twelve done tasks read as OWN-criterion drift against a byte-identical line). The
    /// historical extraction must read a literal exactly as the lexer does - escapes decoded, an escaped
    /// quote not taken as the close - so a criterion carrying `\\s` or `\"` compares equal to itself.
    #[test]
    fn the_historical_criterion_reads_a_literal_as_the_lexer_does() {
        let raw = r#"regex \\s and \\w; quoted \"inner\" then a tab\t end"#;
        let line = format!("        verification tDoD : Test {{ :>> procedureText = \"{raw}\"; :>> id = \"x\"; }}");
        let ours = extract_dod_criterion(&line, "t").expect("a criterion");
        let toks = keel_parser::tokenize(&format!("\"{raw}\""), "t").expect("lexes");
        let lexed = toks
            .iter()
            .find_map(|t| match &t.kind {
                keel_parser::token::TokenKind::Str(s) => Some(s.clone()),
                _ => None,
            })
            .expect("one string token");
        assert_eq!(ours, lexed);
        assert!(ours.contains("quoted \"inner\" then"), "the escaped quote did not close the literal: {ours}");
        // The lexer refuses `\q`; so do we - no text, rather than a wrong one.
        assert_eq!(string_literal_body(r#"bad \q escape""#), None);
        assert_eq!(string_literal_body("never closes"), None);
    }
}
