//! Project discovery: one git repository and every keel project inside it (D0234), and the walk from
//! the current directory to the project root (issue281) with the D0190 engine-version pin warning.
//!
//! Moved whole out of members/keel-process/src/workspace.rs by `scripts/extract_seams.py` (sprint 740,
//! D0479): the workspace GATE stays there and re-exports these names, so `keel_process::workspace::` and
//! `keel_cli::workspace::` paths resolve unchanged. It lives in keel-git because every verb's argument
//! parsing (member keel-args) needs the discovered root, and keel-git is the one member below them all
//! that already spawns git. Not `workspace.rs`: two crates holding one module file would make
//! `scripts/module_home.py` ambiguous (issue559).
use std::path::{Path, PathBuf};

/// One git repository and every keel project inside it.
pub struct Workspace {
    /// The git repository root — the boundary that matters, because it is what a commit, a push and a
    /// `core.hooksPath` are all scoped to.
    pub root: PathBuf,
    /// Every project in the repo, sorted. A single-project repo yields exactly `[root]`.
    pub projects: Vec<PathBuf>,
}

impl Workspace {
    /// Does this repo hold more than one project? Everything that has to change changes ONLY here —
    /// a single-project repo must behave exactly as it did before this existed.
    #[must_use]
    pub const fn is_multi(&self) -> bool {
        self.projects.len() > 1
    }

    /// The project's label within the workspace: its path relative to the repo root, slash-separated.
    /// The root project itself labels as `.`, which is what a single-project repo has always been.
    #[must_use]
    pub fn label(&self, project: &Path) -> String {
        project
            .strip_prefix(&self.root)
            .ok()
            .map(|r| r.to_string_lossy().replace('\\', "/"))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| ".".to_string())
    }

    /// The project that owns `path`, by longest-prefix match. A file may sit under the repo root but
    /// inside no project (a README, a workspace-level script) — that is `None`, not an error.
    ///
    /// The argument is normalised first (`canon`), because the two sides reach here from different
    /// sources that spell the same directory differently — see `canon` for what that cost.
    #[must_use]
    pub fn owner_of(&self, path: &Path) -> Option<&PathBuf> {
        let needle = canon(path);
        self.projects
            .iter()
            .filter(|p| needle.starts_with(p))
            .max_by_key(|p| p.components().count())
    }
}

/// Is this directory a keel project?
#[must_use]
pub fn is_project(dir: &Path) -> bool {
    dir.join(".engine").is_dir() && dir.join(".tracking").is_dir()
}

/// Which of the two project directories `dir` lacks, named for a refusal message.
///
/// issue283: the message used to say `.engine/` whatever was absent, so a tree holding `.engine/`
/// and no `.tracking/` was told the wrong thing.
#[must_use]
pub fn missing_project_dirs(dir: &Path) -> String {
    let missing: Vec<&str> = [".engine/", ".tracking/"].into_iter().filter(|d| !dir.join(d.trim_end_matches('/')).is_dir()).collect();
    if missing.is_empty() { "(both present)".to_string() } else { missing.join(" or ") }
}

/// Refuse to answer over nothing (issue281): the resolved `root` must actually be a keel project.
///
/// # The false green this ends
///
/// The issue269 refusal was applied to `validate` alone, so at a WORKSPACE ROOT — a repository holding
/// projects with none at the root — every other model-reading command answered about an empty model
/// and called it an answer. Measured, in a two-project workspace full of work: `orient` exited 0 with
/// an empty ready list, zero outstanding and `answerStatus: COMPUTED`, and `orient` is the surface
/// CLAUDE.md makes the AI's ONLY legitimate state read, explicitly forbidding prose substitutes;
/// `whats-next` printed "COMPUTED-EMPTY — this is an answer, not a failure" over zero items; and
/// `check-engine` printed "validated clean" over zero files while being a BLOCKING step in this
/// repository's own commit gate.
///
/// Placed in `root_arg` rather than in each command, because `root_arg` is what every command that
/// takes a `[ROOT]` already calls and it already has an error channel — one precondition, 40-odd call
/// sites, no per-command edit to forget. `repo_arg` is deliberately NOT covered: `sync` and `land` are
/// repository-scoped by design, and `projects`/`gate --workspace` parse their own arguments.
/// # Errors
/// `Err(2)` when `root` is not a keel project — the CLI exit code, having already explained on stderr
/// which projects the repository does hold.
pub fn require_project(root: &Path, usage: &str) -> Result<(), i32> {
    if is_project(root) {
        return Ok(());
    }
    eprintln!("error: {} is not a keel project — it has no {} directory.", root.display(), missing_project_dirs(root));
    let ws = discover(root);
    if ws.projects.is_empty() {
        eprintln!("  No keel project was found in {} either.", ws.root.display());
        eprintln!("  An empty answer here would be a FALSE clean, not an answer (K2), so this refuses.");
    } else {
        eprintln!("  This repository holds {} project(s). Name one:", ws.projects.len());
        for p in &ws.projects {
            eprintln!("    {}", ws.label(p));
        }
        eprintln!("  Reporting zero-over-nothing as a computed answer is the false green this refuses.");
    }
    eprintln!("usage: {usage}");
    Err(2)
}

/// One spelling for a path, so two that name the same directory compare equal.
///
/// Every prefix comparison in this module has one side from git (`rev-parse --show-toplevel`, long
/// names, forward slashes) and the other from the process (a CLI argument, `env::temp_dir()`,
/// `canonicalize`). On Windows those differ in three ways at once: `canonicalize` returns the
/// extended-length `\\?\` prefix, git returns forward slashes, and a path may arrive in 8.3 short
/// form (`WILLIA~1`) that git has already expanded. Any one of them makes `starts_with` fail on
/// paths that are in fact the same, which is why `keel projects` reported `current: false` for every
/// project in all eight real keel repos on this machine and printed no you-are-here marker.
///
/// `canonicalize` resolves short names and symlinks; stripping `\\?\` puts the result back in the
/// form the rest of the codebase prints and joins. A path that does not exist yet cannot be
/// canonicalised, so it is returned unchanged — comparisons between two such literals still work.
#[must_use]
pub fn canon(p: &Path) -> PathBuf {
    p.canonicalize().map_or_else(
        |_| p.to_path_buf(),
        |c| {
            let s = c.to_string_lossy().to_string();
            PathBuf::from(s.strip_prefix(r"\\?\").map_or(s.as_str(), |t| t).to_string())
        },
    )
}

/// The git repository root containing `from`, or `from` itself when it is not in a repo.
fn git_root(from: &Path) -> PathBuf {
    crate::gitx::git()
        .arg("-C")
        .arg(from)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()))
        .filter(|p| p.is_dir())
        .map_or_else(|| canon(from), |p| canon(&p))
}

/// Directories that never contain a project and are expensive to walk.
const SKIP: [&str; 5] = [".git", "target", "node_modules", ".keel", ".claude"];

fn walk(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if is_project(dir) {
        out.push(dir.to_path_buf());
        // issue275: this used to `return` here, on the reasoning that a project does not nest inside
        // another project. `keel init` refuses neither position, so the reasoning described a
        // convention rather than the code — and a project nested inside another was invisible to
        // every workspace-scoped mechanism, which meant it rode out UNGATED. Keep descending; a
        // reference or vendored copy is not a false positive because it has no `.tracking/`.
    }
    if depth == 0 {
        return;
    }
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if !p.is_dir() {
            continue;
        }
        let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        if name.starts_with('.') && !is_project(&p) || SKIP.contains(&name.as_str()) {
            continue;
        }
        walk(&p, depth - 1, out);
    }
}

/// Every path git knows about under `root` — tracked plus untracked-and-not-ignored — repo-relative
/// with forward slashes. Empty when `root` is not a git repository.
///
/// Untracked-but-not-ignored matters: a project one minute old has no tracked files yet, and it is
/// exactly the project most likely to be missed.
fn git_known_paths(root: &Path) -> Vec<String> {
    crate::gitx::git()
        .arg("-C")
        .arg(root)
        .args(["-c", "core.quotePath=false", "ls-files", "-c", "-o", "--exclude-standard"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|l| l.trim().replace('\\', "/"))
                .filter(|l| !l.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Discover the workspace containing `from`.
///
/// # Why git, and not a directory walk (issue275)
///
/// This used to be a depth-3 recursive walk that RETURNED at the first project it found. Three
/// panelists found the same consequence independently: a project nested inside another, or deeper
/// than three, was invisible to every workspace-scoped mechanism and therefore rode out UNGATED.
/// The verified reproduction was `keel init .` then `keel init sub` in one repo reporting a single
/// project, after which a commit that added an unparseable tracking file AND downgraded every
/// blocking rule in the invisible project exited 0 while the gate printed clean.
///
/// Asking git is strictly better than a deeper walk. It is bounded by what the repository actually
/// contains rather than by a guessed depth, it costs one process instead of a recursive stat storm,
/// it already excludes `.git`, `target`, and anything else `.gitignore` covers, and — the property
/// that matters for a gate — it enumerates exactly the set a push can carry. A project git does not
/// know about cannot reach a commit, so not gating it is a definition rather than a hole.
///
/// The filesystem walk survives as the fallback for a directory that is not a git repository at all,
/// where there is no push and no shared hook for a workspace mechanism to be responsible for.
#[must_use]
pub fn discover(from: &Path) -> Workspace {
    let root = git_root(from);
    let mut projects = Vec::new();
    for rel in git_known_paths(&root) {
        // A project is the directory holding `.engine/` and `.tracking/`, so every path under either
        // marker names one. Taking the prefix before the marker finds it at ANY depth.
        for marker in [".engine/", ".tracking/"] {
            let Some(i) = rel.find(marker) else { continue };
            // The marker has to start a path component: `vendor/.engine/x` counts, `my.engine/x`
            // does not.
            if i != 0 && !rel[..i].ends_with('/') {
                continue;
            }
            let prefix = rel[..i].trim_end_matches('/');
            let dir = if prefix.is_empty() { root.clone() } else { root.join(prefix) };
            if is_project(&dir) {
                projects.push(canon(&dir));
            }
        }
    }
    if projects.is_empty() {
        walk(&root, 3, &mut projects);
    }
    projects.sort();
    projects.dedup();
    Workspace { root, projects }
}

// ── project discovery from the current directory (sprint 739, D0479) ─────────
// Moved whole out of keel-cli/src/main.rs by `scripts/extract_hooks.py`: the hooks call both, and a
// member cannot call into the binary. keel-cli imports them from here.

// ── repo-root discovery ───────────────────────────────────────────────────────

#[must_use]
pub fn find_repo_root() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        if dir.join(".engine").is_dir() {
            return Some(dir);
        }
        // STOP AT THE REPOSITORY BOUNDARY (issue281). This walk had none while workspace discovery
        // did, so standing in a directory nested under an unrelated keel project, `keel gate validate`
        // with no argument walked OUT of the repository and validated the OUTER repo's project —
        // reporting it clean. A command that answers about a repository the caller is not in is worse
        // than one that refuses: the answer looks right.
        if dir.join(".git").exists() {
            return None;
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// D0190: the engine-version parity warning. The DECLARED version (engine-version.toml, stamped by
/// init and re-stamped by migrate) answers one question only: which binary's checks is this on-disk
/// engine defined against? A mismatch WARNS and names `keel migrate` - never blocks, because skew is
/// not dishonest state (D0098), and never gates or skips anything (migrate derives its vintage from
/// the TREE, per its own no-stamp rule - this declaration exists for the warning, the two designs
/// answer different questions). Absent declaration = pre-D0190 project, silent (forward-only, issue068).
#[must_use]
pub fn engine_version_skew(root: &Path) -> Option<String> {
    let text = std::fs::read_to_string(root.join(".engine").join("contracts").join("engine-version.toml")).ok()?;
    let declared = text
        .lines()
        .find_map(|l| l.trim().strip_prefix("engine").map(|r| r.trim_start_matches(['=', ' ']).trim_matches('"').to_string()))
        .filter(|v| !v.is_empty())?;
    let binary = env!("CARGO_PKG_VERSION");
    if declared == binary {
        return None;
    }
    Some(format!(
        "[keel] engine-version SKEW: this binary is {binary} but this project PINS {declared} (engine-version.toml).          The pin is BINDING: writes and gates REFUSE under skew; reads warn and proceed. Run the pinned          version, or `keel migrate` to bring the tree to this one (it re-stamps the pin)."
    ))
}

#[cfg(test)]
mod tests {
    use super::{discover, is_project, Workspace};
    use std::path::PathBuf;

    #[test]
    fn a_single_project_repo_is_unchanged_by_any_of_this() {
        // The whole design rests on this: everything workspace-aware must be a no-op for the repos
        // that exist today, or the feature is a migration rather than an addition.
        let root = keel_fs::test_support::repo_root();
        let ws = discover(&root);
        assert!(is_project(&ws.root), "this repo's root is itself a project");
        assert_eq!(ws.projects.len(), 1, "discovery must find exactly one project here, got {:?}", ws.projects);
        assert!(!ws.is_multi(), "a single-project repo must not take any workspace branch");
        assert_eq!(ws.label(&ws.projects[0]), ".", "the root project labels as `.`");
    }

    #[test]
    fn labels_and_ownership_resolve_within_the_workspace() {
        let ws = Workspace {
            root: PathBuf::from("/repo"),
            projects: vec![PathBuf::from("/repo/alpha"), PathBuf::from("/repo/nested/beta")],
        };
        assert!(ws.is_multi());
        assert_eq!(ws.label(&PathBuf::from("/repo/alpha")), "alpha");
        assert_eq!(ws.label(&PathBuf::from("/repo/nested/beta")), "nested/beta");
        // A staged file resolves to the project that owns it - that is what lets the hook gate only
        // the projects a commit actually touches.
        assert_eq!(ws.owner_of(&PathBuf::from("/repo/alpha/.tracking/x.sysml")), Some(&PathBuf::from("/repo/alpha")));
        assert_eq!(ws.owner_of(&PathBuf::from("/repo/nested/beta/.engine/y.sysml")), Some(&PathBuf::from("/repo/nested/beta")));
        // A workspace-level file belongs to no project. That is an answer, not an error.
        assert_eq!(ws.owner_of(&PathBuf::from("/repo/README.md")), None);
    }
    /// issue275: discovery must find EVERY project git knows about — nested, and deeper than the old
    /// depth bound. This is the test the previous one lacked: the earlier unit test constructed a
    /// `Workspace` by hand from POSIX literals and never went through `discover`, so it could not
    /// have caught a discovery defect, and the one test that did call `discover` asserted a single
    /// project against this repo, where one is the correct answer.
    ///
    /// Builds a real git repo on disk, because the whole fix is "ask git" and a hand-built fixture
    /// would test the string arithmetic while skipping the part that was wrong.
    #[test]
    fn discovery_finds_nested_and_deep_projects() {
        let dir = std::env::temp_dir().join(format!("keel_ws_discover_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mk = |p: &std::path::Path| {
            let _ = std::fs::create_dir_all(p.join(".engine"));
            let _ = std::fs::create_dir_all(p.join(".tracking"));
            let _ = std::fs::write(p.join(".tracking").join("x.sysml"), "package X {}
");
        };
        let _ = std::fs::create_dir_all(&dir);
        let git = |args: &[&str]| {
            let _ = crate::gitx::git().arg("-C").arg(&dir).args(args).output();
        };
        git(&["init", "-q", "."]);

        mk(&dir); // a project AT the repo root
        let nested = dir.join("sub"); // a project INSIDE it — the layout the panel reproduced
        let _ = std::fs::create_dir_all(&nested);
        mk(&nested);
        let deep = dir.join("a").join("b").join("c").join("d"); // depth 4 — past the old bound
        let _ = std::fs::create_dir_all(&deep);
        mk(&deep);

        let ws = discover(&dir);
        let found: Vec<String> = ws.projects.iter().map(|p| ws.label(p)).collect();
        assert_eq!(ws.projects.len(), 3, "all three projects must be found, got {found:?}");
        assert!(found.iter().any(|l| l == "."), "the root project: {found:?}");
        assert!(found.iter().any(|l| l == "sub"), "the NESTED project: {found:?}");
        assert!(found.iter().any(|l| l == "a/b/c/d"), "the depth-4 project: {found:?}");
        assert!(ws.is_multi(), "three projects is a workspace");

        // Ownership resolves through real discovery, not a hand-built structure: the nested project
        // owns its own files even though its parent is also a project.
        // Compared by LABEL, not by path string: the point is which project owns the file, and the
        // two paths legitimately differ in spelling (`canon` resolves the 8.3 short name that
        // `env::temp_dir()` hands back on this platform, which is what dcOwnerOfMatchesOnWindows was).
        let owner = ws.owner_of(&nested.join(".tracking").join("x.sysml"));
        assert_eq!(
            owner.map(|p| ws.label(p)).as_deref(),
            Some("sub"),
            "the nested project owns its own files even though its parent is also a project"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
