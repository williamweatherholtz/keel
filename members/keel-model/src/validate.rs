//! The validate authority: parse-check files, register the schema and semantically validate every
//! `.tracking`/`.knowledge` file (the `validate` gate) and the `.engine` INSTANCE files (`check-engine`).
//!
//! Out of keel-cli/src/lib.rs in sprint 736 (D0479): the workspace gate is a caller and it became a member,
//! and the read model already owns the corpus walk this reads. keel-cli re-exports `Report`, `check_files`,
//! `validate_root` and `validate_engine_instances` at their old paths.

use std::path::{Path, PathBuf};

use keel_parser::ast::Package;
use keel_parser::{Diagnostic, PackageRegistry};

use crate::corpus::{collect_sysml, parse_pkg, CheckError};

// ── report types ─────────────────────────────────────────────────────────────

/// Accumulated results from a [`check_files`] or [`validate_root`] run.
#[derive(Debug, Default)]
pub struct Report {
    /// Files that could not be read or parsed.
    pub errors: Vec<CheckError>,
    /// Semantic diagnostics produced by [`PackageRegistry::validate`].
    pub diagnostics: Vec<(PathBuf, Diagnostic)>,
    /// Number of `.tracking/` files that were semantically validated.
    pub validated: usize,
}

impl Report {
    /// `true` when there are no errors and no diagnostics.
    #[must_use]
    pub const fn is_clean(&self) -> bool {
        self.errors.is_empty() && self.diagnostics.is_empty()
    }
}

// ── public commands ───────────────────────────────────────────────────────────

/// Parse-check each file in `files` without semantic validation.
///
/// Reads and tokenizes each file; adds a [`CheckError`] for any file that
/// cannot be read or that produces a lex/parse error.
#[must_use]
pub fn check_files(files: &[PathBuf]) -> Report {
    let mut report = Report::default();
    for path in files {
        if let Err(e) = parse_pkg(path) {
            report.errors.push(e);
        }
    }
    report
}

/// Register all schema packages under `root/.engine/` then semantically
/// validate every `.sysml` file under `root/.tracking/`.
///
/// Schema files are registered as ground truth but are not themselves
/// validated (they may reference `ScalarValues::*` which the registry
/// treats as a system namespace).  Tracking files are both registered and
/// validated so they may import each other.
#[must_use]
pub fn validate_root(root: &Path) -> Report {
    let mut report = Report::default();
    let mut registry = PackageRegistry::new();

    // Phase 1 — register all schema packages.
    let engine_dir = root.join(".engine");
    if engine_dir.is_dir() {
        for path in collect_sysml(&engine_dir) {
            match parse_pkg(&path) {
                Ok(pkg) => registry.register(&pkg),
                Err(e) => report.errors.push(e),
            }
        }
    }

    // Phase 2 — TWO-PASS (issue079): register ALL tracking packages first, THEN validate. A one-pass
    // register+validate was filesystem-order-dependent — a tracking file importing another package
    // (e.g. requirements.sysml importing ProjectBusiness) failed when validated before that package was
    // registered. Two-pass makes cross-package imports resolve regardless of iteration order.
    // `.knowledge` (D0161) validates exactly like `.tracking`: declared Questions/Aliases are
    // instance facts under the same identity/reference/provenance authority. Absent dir = nothing.
    let mut parsed: Vec<(std::path::PathBuf, Package)> = Vec::new();
    for base in [".tracking", ".knowledge"] {
        let dir = root.join(base);
        if !dir.is_dir() {
            continue;
        }
        for path in collect_sysml(&dir) {
            match parse_pkg(&path) {
                Ok(pkg) => parsed.push((path, pkg)),
                Err(e) => report.errors.push(e),
            }
        }
    }
    {
        for (_, pkg) in &parsed {
            registry.register(pkg);
        }
        for (path, pkg) in &parsed {
            let diags = registry.validate(pkg, &path.to_string_lossy());
            report.validated += 1;
            for d in diags {
                report.diagnostics.push((path.clone(), d));
            }
        }
    }

    report
}

/// True for the `.engine` files that are INSTANCES (validated like `.tracking`), not schema/workflow
/// definitions (registered as ground truth only): `decisions/`, `processes/`, `views/`, plus
/// `skills-registry.sysml` and `tracking-template.sysml`.
fn is_engine_instance_file(path: &Path) -> bool {
    let s = path.to_string_lossy().replace('\\', "/");
    s.contains("/decisions/")
        || s.contains("/processes/")
        || s.contains("/views/")
        || s.ends_with("skills-registry.sysml")
        // D0222: a skill's own declaration lives beside it now (`.engine/skills/<s>/registry.sysml`),
        // so those 36 files are instances too. Missed on the first pass, which meant 36 newly
        // authored files were NOT being validated by `check-engine` while it reported clean — a
        // coverage gap that reads exactly like coverage.
        || (s.contains("/skills/") && s.ends_with("registry.sysml"))
        || s.ends_with("tracking-template.sysml")
        // D0271/issue344: the CLI command facts. Added the day the file was created, so it never sits
        // unvalidated the way the 36 skill registries did.
        || s.contains("/cli/")
}

/// Semantically validate the `.engine` INSTANCE files against the schema, KERNEL-FREE (D0112 phase 2,
/// issue067).
///
/// The Rust backstop for the `unresolved` reference class that the JVM `validate_instances.py` was the
/// only source of. Registers ALL `.engine` packages first (schema + instances) so cross-file references
/// resolve (the Rust advantage over the per-file kernel, issue021/024), then runs the SAME
/// [`PackageRegistry::validate`] used for `.tracking`. Schema/workflow files are registered but NOT
/// validated (they reference the `ScalarValues` system namespace). Returns `(path, diagnostic)` pairs —
/// empty when clean.
///
/// A file that does not PARSE is a diagnostic, not a skip (issue467): for one day a Decision file
/// carrying an extra closing brace was reported "validated clean" here while `keel gate check` rejected it
/// at 25:1, because both passes read `if let Ok(pkg)` and dropped the `Err` — an enforcement point
/// that reports a pass it did not compute (EHZ5). Every `.engine` file's parse failure is reported,
/// instance or schema: a schema file that fails to parse is silently absent from the registry, and
/// every reference into it would then be misreported as unresolved.
#[must_use]
pub fn validate_engine_instances(root: &Path) -> Vec<(PathBuf, Diagnostic)> {
    let engine_dir = root.join(".engine");
    if !engine_dir.is_dir() {
        return Vec::new();
    }
    let all = collect_sysml(&engine_dir);
    let mut registry = PackageRegistry::new();
    let mut parsed = Vec::with_capacity(all.len());
    let mut out = Vec::new();
    for path in &all {
        match parse_pkg(path) {
            Ok(pkg) => {
                registry.register(&pkg);
                parsed.push((path, pkg));
            }
            Err(e) => out.push((path.clone(), parse_failure_diagnostic(path, &e))),
        }
    }
    for (path, pkg) in &parsed {
        if !is_engine_instance_file(path) {
            continue;
        }
        for d in registry.validate(pkg, &path.to_string_lossy()) {
            out.push(((*path).clone(), d));
        }
    }
    out
}

/// A [`CheckError`] from [`parse_pkg`] as the [`Diagnostic`] `check-engine` prints. The lexer and
/// parser messages are `<file>:<line>:<col>: <what>`; the line is lifted from that prefix so the
/// diagnostic points at the line, and is 0 when the message carries none (an unreadable file).
fn parse_failure_diagnostic(path: &Path, e: &CheckError) -> Diagnostic {
    let name = path.to_string_lossy();
    let line = e
        .message
        .strip_prefix(name.as_ref())
        .and_then(|rest| rest.strip_prefix(':'))
        .and_then(|rest| rest.split(':').next())
        .and_then(|n| n.parse::<u32>().ok())
        .unwrap_or(0);
    Diagnostic {
        file: name.as_ref().into(),
        line,
        message: format!("does not parse — {}", e.message).into_boxed_str(),
        suggestion: Some(
            "the file was skipped by every reference check, not validated; `keel gate check <file>` names the construct (issue467)"
                .into(),
        ),
    }
}

#[cfg(test)]
mod engine_instance_tests {
    use super::is_engine_instance_file;
    use std::path::Path;

    #[test]
    fn classifies_engine_instance_files() {
        // D0112 phase 2: decisions/processes/views + the two named files are instances (validated);
        // schema/workflows are NOT (registered as ground truth only).
        assert!(is_engine_instance_file(Path::new(".engine/decisions/0112-x.sysml")));
        assert!(is_engine_instance_file(Path::new(".engine/processes/doc-sync.sysml")));
        assert!(is_engine_instance_file(Path::new(".engine/views/viewpoint-registry.sysml")));
        assert!(is_engine_instance_file(Path::new(".engine/skills/skills-registry.sysml")));
        assert!(is_engine_instance_file(Path::new(".engine/docs/tracking-template.sysml")));
        // schema + workflow DEFS are not instances.
        assert!(!is_engine_instance_file(Path::new(".engine/schema/core.sysml")));
        assert!(!is_engine_instance_file(Path::new(".engine/workflows/delivery.sysml")));
        assert!(!is_engine_instance_file(Path::new(".engine/rules/rules.sysml")));
    }

    struct Tmp(std::path::PathBuf);
    impl Drop for Tmp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    const DECISION: &str = concat!(
        "package Fx467 {\n",
        "    part d9999 : Decision {\n",
        "        attribute :>> title = \"a well-formed decision\";\n",
        "    }\n",
        "}\n",
    );

    fn tree(tag: &str) -> Tmp {
        let root = std::env::temp_dir().join(format!("keel-issue467-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".engine").join("decisions")).unwrap();
        std::fs::create_dir_all(root.join(".engine").join("schema")).unwrap();
        std::fs::write(
            root.join(".engine").join("schema").join("core.sysml"),
            concat!(
                "package Core {\n",
                "    part def Decision {\n",
                "        attribute title : String;\n",
                "    }\n",
                "}\n",
            ),
        )
        .unwrap();
        std::fs::write(root.join(".engine").join("decisions").join("9999-fx.sysml"), DECISION).unwrap();
        Tmp(root)
    }

    /// issue467, known-negative: a well-formed instance file yields no parse diagnostic.
    #[test]
    fn a_well_formed_instance_file_is_not_a_parse_diagnostic() {
        let t = tree("neg");
        let diags = super::validate_engine_instances(&t.0);
        assert!(
            diags.iter().all(|(_, d)| !d.message.contains("does not parse")),
            "unexpected parse diagnostic: {diags:?}"
        );
    }

    /// issue467, known-positive: the same file with one extra closing brace is reported with its line,
    /// where both passes used to drop the `Err` and report the tree clean.
    #[test]
    fn an_unparseable_instance_file_is_a_diagnostic_naming_its_line() {
        let t = tree("pos");
        let f = t.0.join(".engine").join("decisions").join("9999-fx.sysml");
        std::fs::write(&f, format!("{DECISION}}}\n")).unwrap();
        let diags = super::validate_engine_instances(&t.0);
        let hit = diags
            .iter()
            .find(|(p, d)| p == &f && d.message.contains("does not parse"))
            .unwrap_or_else(|| panic!("no parse diagnostic for the broken file: {diags:?}"));
        assert_eq!(hit.1.line, 6, "the extra brace is on line 6: {}", hit.1.message);
        assert!(hit.1.message.contains("got RBrace"), "{}", hit.1.message);
        assert!(hit.1.suggestion.as_deref().is_some_and(|s| s.contains("keel gate check")));
    }
}
