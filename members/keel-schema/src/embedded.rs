//! The engine tree embedded in this binary - ONE `include_dir!`, shared by the two things that need it.
//!
//! `keel init` scaffolds it and `keel migrate` resyncs a project from it (D0093 / D0275); since D0441
//! the `process-change` guard reads it too, to tell the engine's own published text arriving on a
//! project from a hand edit to a locked file. A second `include_dir!` of the same tree would embed
//! every byte twice, so the static lives here in the library and `main.rs` borrows it.

use include_dir::{include_dir, Dir};
use std::path::Path;

/// The reusable engine tree + operating manual, embedded at compile time so `keel init` is
/// self-contained (no external fetch - the cytoscape precedent).
pub static ENGINE_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../../.engine");

/// The embedded engine's text at `rel` (a path relative to `.engine/`, forward slashes), or `None`
/// when the engine ships no file there or the file is not UTF-8.
#[must_use]
pub fn engine_text(rel: &str) -> Option<&'static str> {
    ENGINE_DIR.get_file(rel).and_then(include_dir::File::contents_utf8)
}

/// Where the repository carries the keel PLUGIN - the enforcement copy of the hook set (D0296).
///
/// Declared beside the embedded tree it is rendered from (sprint 732) so the control-structure view
/// can read the plugin's hook file without reaching up into `claude_surface`; `claude_surface`
/// re-exports it under its old path.
pub const PLUGIN_DIR: &str = ".engine/claude-plugin";

/// The text an engine resync WRITES at `mapped` (relative to `.engine/`).
///
/// Onto a project whose file currently reads `current` that is the shipped text, or for a sectioned contract the shipped text with the
/// project's own sections merged in (issue349). `None` when the merge conflicts, which the resync
/// reports as a blocker and writes nothing for.
///
/// The `process-change` guard asks this question of every locked path it sees change (D0441): a file
/// whose new text is what the resync would have written is the engine arriving, not a hand edit. One
/// function answers both so the guard cannot drift from the writer.
#[must_use]
pub fn resync_text(mapped: &Path, shipped: &str, current: Option<&str>) -> Option<String> {
    if is_sectioned_contract(mapped) {
        if let Some(cur) = current {
            return merge_project_sections(shipped, cur).ok();
        }
    }
    Some(shipped.to_owned())
}

/// Engine-shipped contracts whose `[section]`s a project may EXTEND with its own (issue349): the
/// resync merges rather than overwrites them.
#[must_use]
pub fn is_sectioned_contract(mapped: &Path) -> bool {
    mapped.parent().is_some_and(|p| p.ends_with("contracts")) && mapped.file_name().and_then(|f| f.to_str()) == Some("unit-extras.toml")
}

/// The `[name]` sections of a TOML-shaped contract as (name, body) in file order; text before the
/// first header is the preamble and is not a section.
#[must_use]
pub fn project_sections(text: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if let Some(name) = t.strip_prefix('[').and_then(|r| r.strip_suffix(']')).filter(|_| !t.starts_with("[[")) {
            out.push((name.to_string(), String::new()));
        } else if let Some((_, body)) = out.last_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    out
}

/// Merge the engine's shipped copy with the project's current one: every section the engine does not
/// carry is appended verbatim after the engine's text. A section both carry with a different body
/// (comments and blank lines aside) is a conflict, returned by name; the caller blocks on it.
/// # Errors
/// The names of the sections both copies carry with different bodies.
pub fn merge_project_sections(shipped: &str, current: &str) -> Result<String, Vec<String>> {
    let engine = project_sections(shipped);
    let normal = |body: &str| body.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).collect::<Vec<_>>().join("\n");
    let mut conflicts = Vec::new();
    let mut extra = String::new();
    for (name, body) in project_sections(current) {
        match engine.iter().find(|(n, _)| *n == name) {
            Some((_, eb)) if normal(eb) != normal(&body) => conflicts.push(name),
            Some(_) => {}
            None => {
                extra.push('[');
                extra.push_str(&name);
                extra.push_str("]\n");
                extra.push_str(&body);
            }
        }
    }
    if !conflicts.is_empty() {
        return Err(conflicts);
    }
    if extra.is_empty() {
        return Ok(shipped.to_string());
    }
    let mut out = shipped.trim_end_matches('\n').to_string();
    out.push_str("\n\n# ── project-authored sections, preserved through `keel migrate` (issue349) ──\n");
    out.push_str(extra.trim_end_matches('\n'));
    out.push('\n');
    Ok(out)
}

/// Engine-DEV-only embedded paths EXCLUDED from the scaffold (D0093 boundary): the kernel/Python
/// toolchain and any compiled-Python cache. Downstream projects use the Rust path (D0048).
///
/// EXCEPT the tools a shipped process DEPLOYS BY PATH: the obligation-review process (D0171,
/// portable) names the deck e2e and the inbox recorder, and guard 39 (`tool-reference`) rightly
/// fails a fresh scaffold whose process references tools it never received — CI caught exactly that
/// on the guard's first landing. Both are self-contained (httpx + stdlib), kernel-free, and carry no
/// repo-specific state, so shipping them keeps D0048 intact.
/// The tools a scaffold RECEIVES from the engine: stdlib-only, kernel-free, referenced by shipped
/// processes. One home, because two lists of what the engine ships would drift and the drift would be
/// invisible until a follower found a dead path.
const PORTABLE_TOOLS: [&str; 2] = ["test_deck_e2e.py", "deck_inbox_record.py"];

/// Is this one of the tools the engine ships into every project?
///
/// Read by guard 65: a project must declare the instruments IT wrote, and accusing it of the ones the
/// engine handed it is a false refusal - the guard went red on a fresh adoption before this existed.
#[must_use]
pub fn is_portable_engine_tool(rel: &Path) -> bool {
    rel.parent().is_some_and(|p| p.ends_with("tools"))
        && rel.file_name().is_some_and(|f| PORTABLE_TOOLS.iter().any(|t| f == *t))
}

#[must_use]
pub fn is_engine_dev_only(rel: &Path) -> bool {
    // A tool a shipped process references by path must ship with it - a scaffold that ships the process
    // without the tool hands a follower a dead path (tool-reference went red on CI's foreign-tree check
    // the day stpa-diagram landed without its Python interim on this list; that renderer is in the
    // binary since D0285's port, `keel show control-structure --svg`).
    if is_portable_engine_tool(rel) {
        return false;
    }
    // HISTORICAL (D0361 -> D0363): the engine's instrument inventory was briefly a contract file, and
    // migrating it gave every adopter seventeen dead references - `tool-reference` went red across 18
    // scaffold and migration tests. The inventory is model items now, which are .tracking data and
    // therefore never shipped, so no exclusion is needed. The line is gone rather than kept as a
    // tombstone; this comment is here because the next person to see that test failure should know
    // what caused it.
    rel.components().any(|c| {
        let s = c.as_os_str().to_string_lossy();
        s == "tools" || s == "__pycache__"
    }) || rel.extension().is_some_and(|e| e == "pyc")
}

#[cfg(test)]
mod section_merge_tests {
    use super::{merge_project_sections, project_sections};

    const ENGINE: &str = "# header\n# more\n[alpha]\nfiles = [\"a.yml\"]\n";

    /// GH#44: the project's own [unit] section rides through the resync; the engine's own text is
    /// otherwise the engine's.
    #[test]
    fn a_project_authored_section_is_appended_and_the_engine_text_wins_elsewhere() {
        let current = "# stale header\n[alpha]\nfiles = [\"a.yml\"]\n[ours]\nfiles = [\"tools/ours.py\"]\nrequires = [\"x\"]\n";
        let merged = merge_project_sections(ENGINE, current).expect("no conflict");
        assert!(merged.starts_with("# header\n# more\n[alpha]"), "the engine's text leads: {merged}");
        assert!(merged.contains("[ours]\nfiles = [\"tools/ours.py\"]\nrequires = [\"x\"]"), "the project's section is intact: {merged}");
        assert!(merged.contains("issue349"), "and says why it is there");
        assert_eq!(project_sections(&merged).len(), 2);
    }

    /// A section both carry with a different body is nobody's to decide: named, so the run blocks.
    #[test]
    fn a_conflicting_section_is_named_not_resolved() {
        let current = "[alpha]\nfiles = [\"a.yml\", \"b.yml\"]\n";
        assert_eq!(merge_project_sections(ENGINE, current), Err(vec!["alpha".to_string()]));
        // the same body with a comment and spacing difference is NOT a conflict
        let same = "[alpha]\n# a note\n\nfiles = [\"a.yml\"]\n";
        assert_eq!(merge_project_sections(ENGINE, same).expect("same"), ENGINE);
    }
}
