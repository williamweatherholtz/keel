//! The workspace graph the touched set descends (D0481, sprint 741): every member the root manifest
//! lists, the directory it lives in, and the members it depends on by `path`. Pure over the manifest
//! texts; the one reader is [`read_graph`].
//!
//! The graph answers two questions the touched set used to answer with one hash. WHICH members a
//! change reaches: the owner of each changed path, closed over its dependents ([`candidates`]) - a
//! leaf edit reaches the leaf and everything built on it, not the sibling nobody built on. WHAT a
//! member's unit tests are built from: its own directory plus the directories of everything it
//! depends on, transitively ([`scope`]) - the code key a member's `lib` row is skipped on.
//!
//! A changed code path no member owns (the root `Cargo.toml`, `Cargo.lock`, `.engine/` - embedded by
//! keel-schema but read by keel-cli's tests as well - `.githooks/`, `.claude/`) is a seed for EVERY
//! member: the workspace cannot say who it reaches, so it reaches all of them. Never a false narrowing.

use std::collections::{BTreeMap, BTreeSet};

/// One workspace member: its package `name`, its directory relative to the repository root with
/// `/` separators and no trailing slash, and the package names of its `path` dependencies (dev
/// dependencies included - a test binary is built from them too).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub name: String,
    pub dir: String,
    pub deps: Vec<String>,
}

/// The members, in root-manifest order.
pub type Graph = Vec<Member>;

/// The `name` inside `[package]` and every `{ path = ".." }` dependency of one manifest text. The
/// path is resolved against `dir` and reduced to the member directory it names; the member's
/// package name is what the caller resolves later (`path = "../keel-fs"` names `members/keel-fs`,
/// whose package is `keel-fs`; the two agree everywhere here, and [`read_graph`] resolves by
/// directory, never by assuming they do).
fn parse_manifest(text: &str) -> (Option<String>, Vec<String>) {
    let mut name = None;
    let mut dep_dirs = Vec::new();
    let mut section = String::new();
    for raw in text.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.starts_with('[') {
            line.trim_matches(|c| c == '[' || c == ']').trim().clone_into(&mut section);
            continue;
        }
        let Some((key, value)) = line.split_once('=') else { continue };
        let (key, value) = (key.trim(), value.trim());
        if section == "package" && key == "name" {
            name = Some(value.trim_matches('"').to_owned());
        }
        if section.ends_with("dependencies") {
            if let Some(path) = value.split("path").nth(1).and_then(|rest| rest.split('"').nth(1)) {
                dep_dirs.push(path.to_owned());
            }
        }
    }
    (name, dep_dirs)
}

/// `dir` joined with a relative `path`, `/`-normalised, `.` and `..` segments resolved.
fn resolve(dir: &str, path: &str) -> String {
    let mut segs: Vec<&str> = if dir.is_empty() || dir == "." { Vec::new() } else { dir.split('/').collect() };
    let path = path.replace('\\', "/");
    for seg in path.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                segs.pop();
            }
            s => segs.push(s),
        }
    }
    segs.join("/")
}

/// The graph from the root manifest text and one `(dir, manifest text)` per listed member. Pure.
/// A dependency path that names no listed member is dropped: it is a crate outside the workspace,
/// and the workspace graph has nothing to say about it.
#[must_use]
pub fn build(root_manifest: &str, manifests: &[(String, String)]) -> Graph {
    let dirs = keel_model::corpus::workspace_members(root_manifest);
    let parsed: Vec<(String, Option<String>, Vec<String>)> =
        dirs.iter().filter_map(|dir| manifests.iter().find(|(d, _)| d == dir)).map(|(dir, text)| {
            let (name, dep_dirs) = parse_manifest(text);
            (dir.clone(), name, dep_dirs.iter().map(|p| resolve(dir, p)).collect())
        }).collect();
    let by_dir: BTreeMap<&str, &str> =
        parsed.iter().filter_map(|(dir, name, _)| name.as_deref().map(|n| (dir.as_str(), n))).collect();
    parsed
        .iter()
        .filter_map(|(dir, name, dep_dirs)| {
            let name = name.clone()?;
            let mut deps: Vec<String> =
                dep_dirs.iter().filter_map(|d| by_dir.get(d.as_str()).map(|n| (*n).to_owned())).collect();
            deps.sort();
            deps.dedup();
            Some(Member { name, dir: dir.clone(), deps })
        })
        .collect()
}

/// The graph of the repository at `repo`: the root `Cargo.toml` plus each listed member's manifest.
/// `None` when the root manifest cannot be read.
#[must_use]
pub fn read_graph(repo: &std::path::Path) -> Option<Graph> {
    let root = std::fs::read_to_string(repo.join("Cargo.toml")).ok()?;
    let manifests: Vec<(String, String)> = keel_model::corpus::workspace_members(&root)
        .into_iter()
        .filter_map(|dir| std::fs::read_to_string(repo.join(&dir).join("Cargo.toml")).ok().map(|t| (dir, t)))
        .collect();
    Some(build(&root, &manifests))
}

/// The member whose directory holds `rel` (a `/`-normalised repo-relative path), by name. The
/// longest matching directory wins, so a member nested under another's directory owns its own files.
#[must_use]
pub fn owner<'g>(graph: &'g Graph, rel: &str) -> Option<&'g Member> {
    let rel = rel.replace('\\', "/");
    graph
        .iter()
        .filter(|m| rel == m.dir || rel.starts_with(&format!("{}/", m.dir)))
        .max_by_key(|m| m.dir.len())
}

/// `seeds` closed over "depends on": every member that is a seed or depends, transitively, on one.
/// Sorted by name.
#[must_use]
pub fn dependents(graph: &Graph, seeds: &BTreeSet<String>) -> Vec<String> {
    let mut reached: BTreeSet<String> = seeds.clone();
    loop {
        let before = reached.len();
        for m in graph {
            if !reached.contains(&m.name) && m.deps.iter().any(|d| reached.contains(d)) {
                reached.insert(m.name.clone());
            }
        }
        if reached.len() == before {
            break;
        }
    }
    reached.into_iter().collect()
}

/// The candidate members for a set of changed `/`-normalised repo-relative paths: the owners of the
/// changed paths closed over their dependents. Any changed CODE path with no owner (root manifest,
/// lock file, `build/`, `.engine/`, `.githooks/`, `.claude/`, `keelw`) makes every member a candidate; a
/// changed path that is neither code nor owned (`.tracking/`, docs) contributes nothing. Sorted.
#[must_use]
pub fn candidates(graph: &Graph, changed: &[String]) -> Vec<String> {
    let mut seeds = BTreeSet::new();
    for rel in changed {
        let rel = rel.replace('\\', "/");
        match owner(graph, &rel) {
            Some(m) => {
                seeds.insert(m.name.clone());
            }
            None if crate::contentkey::is_code(&rel) => return graph.iter().map(|m| m.name.clone()).collect(),
            None => {}
        }
    }
    dependents(graph, &seeds)
}

/// The directories a member's tests are built from: its own and, transitively, its dependencies'.
/// Sorted; empty when `member` is not in the graph.
#[must_use]
pub fn scope(graph: &Graph, member: &str) -> Vec<String> {
    let by_name: BTreeMap<&str, &Member> = graph.iter().map(|m| (m.name.as_str(), m)).collect();
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut stack = vec![member];
    while let Some(name) = stack.pop() {
        if let Some(m) = by_name.get(name) {
            if seen.insert(name) {
                stack.extend(m.deps.iter().map(String::as_str));
            }
        }
    }
    seen.iter().filter_map(|n| by_name.get(n).map(|m| m.dir.clone())).collect::<BTreeSet<_>>().into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Three members: `leaf` (no deps), `mid` (depends on leaf), `other` (no deps, nobody depends on it).
    fn fixture() -> Graph {
        let root = "[workspace]\nmembers = [\n    \"members/leaf\",\n    \"members/mid\",\n    \"other\",\n]\n";
        let manifests = vec![
            ("members/leaf".to_owned(), "[package]\nname = \"leaf\"\n\n[dependencies]\nserde = \"1\"\n".to_owned()),
            (
                "members/mid".to_owned(),
                "[package]\nname = \"mid\" # the middle\n\n[dependencies]\nleaf = { path = \"../leaf\" }\ntoml = \"0.8\"\n\n[dev-dependencies]\nleaf = { path = \"../leaf\", features = [\"x\"] }\n".to_owned(),
            ),
            ("other".to_owned(), "[package]\nname = \"other\"\n\n[dependencies]\noutside = { path = \"../../elsewhere\" }\n".to_owned()),
        ];
        build(root, &manifests)
    }

    /// D0481 pair (the definition of done's known-positive), chosen before the code was written: only the leaf changes,
    /// the candidate set is the leaf and its dependent and NOT the third member.
    #[test]
    fn a_leaf_edit_reaches_the_leaf_and_its_dependent_and_not_the_third() {
        let g = fixture();
        assert_eq!(
            g,
            vec![
                Member { name: "leaf".into(), dir: "members/leaf".into(), deps: vec![] },
                Member { name: "mid".into(), dir: "members/mid".into(), deps: vec!["leaf".into()] },
                Member { name: "other".into(), dir: "other".into(), deps: vec![] },
            ],
            "the graph reads names, dirs and path deps; a dep outside the workspace is dropped, a dev-dep duplicate is one edge"
        );
        assert_eq!(candidates(&g, &["members/leaf/src/lib.rs".to_owned()]), vec!["leaf", "mid"]);
        assert_eq!(candidates(&g, &["members\\mid\\Cargo.toml".to_owned()]), vec!["mid"], "a mid edit reaches nothing below it");
        assert_eq!(candidates(&g, &["other/src/x.rs".to_owned()]), vec!["other"]);
        // Known-negative: a path no member owns that is not code contributes nothing; one that IS code reaches all.
        assert!(candidates(&g, &[".tracking/backlog.sysml".to_owned()]).is_empty());
        assert_eq!(candidates(&g, &["Cargo.lock".to_owned()]), vec!["leaf", "mid", "other"]);
        assert_eq!(candidates(&g, &[".engine/schema/core.sysml".to_owned(), "other/src/x.rs".to_owned()]), vec!["leaf", "mid", "other"]);
        assert!(candidates(&g, &[]).is_empty());
    }

    #[test]
    fn a_members_scope_is_its_directory_and_its_dependencies_directories() {
        let g = fixture();
        assert_eq!(scope(&g, "mid"), vec!["members/leaf", "members/mid"]);
        assert_eq!(scope(&g, "leaf"), vec!["members/leaf"]);
        assert_eq!(scope(&g, "other"), vec!["other"]);
        assert!(scope(&g, "nobody").is_empty());
        assert_eq!(owner(&g, "members/mid/src/a.rs").map(|m| m.name.as_str()), Some("mid"));
        assert_eq!(owner(&g, "members/middle/src/a.rs"), None, "a prefix is not a directory");
        assert_eq!(owner(&g, "other"), Some(&g[2]), "the directory itself is owned");
    }

    #[test]
    fn a_relative_dependency_path_resolves_against_the_members_directory() {
        assert_eq!(resolve("members/mid", "../leaf"), "members/leaf");
        assert_eq!(resolve("keel-cli", "../members/keel-fs"), "members/keel-fs");
        assert_eq!(resolve("keel-cli", "..\\keel-parser"), "keel-parser");
        assert_eq!(resolve("a/b", "./c/"), "a/b/c");
        assert_eq!(resolve(".", "x"), "x");
    }

    /// The real workspace: every member the root lists is in the graph, keel-cli depends on keel-suite,
    /// keel-suite's scope holds keel-fs, and a keel-fs edit reaches keel-suite and keel-cli.
    #[test]
    fn the_real_workspace_graph_holds_every_member_and_keel_cli_depends_on_the_rest() {
        let repo = keel_fs::test_support::repo_root();
        let g = read_graph(&repo).expect("root manifest");
        let root = std::fs::read_to_string(repo.join("Cargo.toml")).expect("root");
        assert_eq!(g.len(), keel_model::corpus::workspace_members(&root).len(), "one Member per listed dir");
        let cli = g.iter().find(|m| m.name == "keel-cli").expect("keel-cli");
        assert_eq!(cli.dir, "keel-cli");
        assert!(cli.deps.iter().any(|d| d == "keel-suite") && cli.deps.iter().any(|d| d == "keel-parser"), "{:?}", cli.deps);
        assert!(scope(&g, "keel-suite").contains(&"members/keel-fs".to_owned()));
        let reached = candidates(&g, &["members/keel-fs/src/fsx.rs".to_owned()]);
        assert!(reached.contains(&"keel-suite".to_owned()) && reached.contains(&"keel-cli".to_owned()), "{reached:?}");
        assert!(!reached.contains(&"keel-parser".to_owned()), "keel-parser does not depend on keel-fs: {reached:?}");
    }
}
