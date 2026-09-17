"""extract_issues.py - the sprint 737 transform: the item and issue processes become member keel-issues (D0480).

not-an-instrument: it is a one-shot codemod (the migration skill's gate 1); its reconciliation totals are the
transform checking itself before it writes, not a measure of the project - the release build under
deny(warnings), the record-issue / record-story / github-intake integration tests and the Fresh probe are
the sensors the move answers to.

One committed script, dry-run by default (D0479 one extraction per sprint; the migration skill's gate 1).
Two modules move as git renames - github_ingest.rs out of keel-cli/src and intake_write.rs out of
members/keel-write/src - and every `crate::<module>::` path in them is rewritten per MODULE_MAP to the
crate that owns the module now. Four slices leave files that stay: the issue write (NewIssue,
next_issue_number, record_issue and its test) out of write.rs into issue_write.rs; the open-issues and
dispositions views out of view/mod.rs and the intake view out of view/staleness.rs into views.rs. The
reconciliation holds each moved file's text equal to its source modulo the counted rewrites, and each
sliced file equal to its source minus the slices plus the anchored edits named below.

Two descents make the member possible without a guard or a view in its tree. The resolver predicate -
`declared_task_names` (keel-guards lib.rs) and `resolver_kind_holds` (keel-guards issues.rs), the check
`record issue` applies before it writes (issue558) - descends to members/keel-model/src/resolvers.rs and
keel-guards re-exports both at their old paths; `at_least_medium` (view/mod.rs) descends to
keel_model::queries beside `issue_disposition`. The severity and resolver checks that sat inline in
main.rs's cmd_record_issue become `issue_write::triage_holds`, and main.rs calls it. keel-cli's lib.rs
keeps every old path: `view` and `write` become wrapper modules that glob re-export the member's module
and add the sliced names, so `crate::view::open_issues` and `crate::write::record_issue` still resolve.

    python scripts/extract_issues.py            # plan + reconciliation, nothing written
    python scripts/extract_issues.py --apply    # the renames, the rewrites, the slices, the descents, the manifests, the re-exports

Idempotent: a tree where members/keel-issues/src/lib.rs exists is reported as already applied.
"""
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
CLI_SRC = REPO / "keel-cli" / "src"
WRITE_SRC = REPO / "members" / "keel-write" / "src"
VIEW_SRC = REPO / "members" / "keel-view" / "src" / "view"
GUARDS_SRC = REPO / "members" / "keel-guards" / "src"
MODEL_SRC = REPO / "members" / "keel-model" / "src"
DST = REPO / "members" / "keel-issues"
DST_SRC = DST / "src"

# (source path, destination file name)
MOVES = [
    (CLI_SRC / "github_ingest.rs", "github_ingest.rs"),
    (WRITE_SRC / "intake_write.rs", "intake_write.rs"),
]

# `crate::<module>` in a moved file -> the path that resolves from keel-issues. A module the member owns
# stays `crate::`.
MODULE_MAP = {
    "write": "keel_write::write",
}
OWN = {"github_ingest", "intake_write", "issue_write", "views"}
REF = re.compile(r"\bcrate::([A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*)")

LINTS = '''#![forbid(unsafe_code)]
#![deny(warnings, clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, clippy::todo, clippy::unimplemented)]
#![allow(clippy::implicit_hasher, clippy::too_long_first_doc_paragraph, clippy::module_name_repetitions)]
// Tests may use unwrap/expect/panic/indexing/asserts freely.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing))]
'''

CARGO = '''[package]
name = "keel-issues"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "The item and issue processes (D0480): recording an issue, a statement or a story, the GitHub intake, and the open-issues, dispositions and intake views - apart from the processes that govern keel itself."

[dependencies]
keel-json = { path = "../keel-json" }
keel-schema = { path = "../keel-schema" }
keel-model = { path = "../keel-model" }
keel-write = { path = "../keel-write" }
serde_json = "1"
'''

LIB = '''//! keel-issues: the item and issue processes - recording an issue, a statement or a story, the GitHub
//! intake, and the open-issues, dispositions and intake views (D0480).
//!
//! The seventh D0479 extraction (sprint 737): `github_ingest` out of keel-cli, `intake_write` out of
//! keel-write, the issue write sliced from write.rs and the three item views sliced from keel-view, moved
//! by `scripts/extract_issues.py`. keel-cli re-exports every name at its old path (`crate::view::open_issues`,
//! `crate::write::record_issue`, `crate::intake_write`, `crate::github_ingest`), so no caller moved. The
//! crate depends on the read model, the write API, the schema and the JSON leaf - on no view, no guard,
//! no process and nothing that serves - so a project that only tracks items can depend on it without the
//! engine-governance stack, and an edit to `process_cmd.rs` or `serve.rs` leaves it Fresh (D0508).
''' + LINTS + '''
pub mod github_ingest;
pub mod intake_write;
pub mod issue_write;
pub mod views;

/// `keel record task` adds a `DoD` `Test` to a declared action - an item verb, reachable here by name. The
/// write itself stays keel-write's: it reads eight of write.rs's private insertion helpers, which
/// `append_result` shares, and a fact lives where its readers are (D0508).
pub use keel_write::write::add_task;
'''

VIEWS_HEAD = '''//! The item views (D0480, sprint 737): open-issues (D0077), dispositions (D0092) and intake (D0166),
//! sliced whole out of keel-view's view/mod.rs and view/staleness.rs by `scripts/extract_issues.py`.
//! keel-cli's `view` wrapper module re-exports the three, so `keel show open-issues`, `dispositions` and
//! `intake` and the server's lens table resolve them at their old paths.

use std::collections::BTreeMap;
use std::path::Path;

use keel_json::json::Json;
use keel_model::model::{ItemInfo, Model, ViewError};
use keel_model::queries::{at_least_medium, compute_issue_resolution, issue_disposition};

'''

ISSUE_HEAD = '''//! `keel record issue` (D0480, sprint 737): the triaged Issue and its `#Resolves` edge, sliced whole out of
//! keel-write's write.rs by `scripts/extract_issues.py`, plus `triage_holds` - the severity and
//! resolver-kind checks that sat inline in main.rs's `cmd_record_issue` (issue558) and are the item
//! process's own. keel-cli's `write` wrapper module re-exports `NewIssue` and `record_issue`, so
//! `crate::write::record_issue` still resolves.

use std::path::Path;

use keel_model::ident::gen_uuid;
use keel_model::model::Model;
use keel_model::resolvers::{declared_task_names, resolver_kind_holds};
use keel_write::write::{per_actor_file, reject_injected_output, sanitize_field, with_file_lock, write_atomic, WriteError};

'''

TRIAGE = '''
/// Why `record issue` refuses a triage before anything is written (issue558, D0077).
#[derive(Debug)]
pub enum TriageRefusal {
    /// `--severity` outside `Critical | High | Medium | Low`.
    Severity(String),
    /// The resolver is declared nowhere in the model.
    UnknownResolver(String),
    /// The resolver exists but is neither a declared action nor a `Decision`; carries the kind it is.
    ResolverKind { resolver: String, kind: String },
    /// The model could not be read to check the resolver.
    Model(String),
}

impl std::fmt::Display for TriageRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Severity(s) => write!(f, "--severity must be Critical | High | Medium | Low (got '{s}')"),
            Self::UnknownResolver(r) => write!(
                f,
                "resolver '{r}' is declared nowhere in the model.\\n  Authoring the edge anyway would make this Issue read as TRIAGED by something that does not\\n  exist (issue109). Declare the resolving action or Decision first, then re-run."
            ),
            Self::ResolverKind { resolver, kind } => write!(
                f,
                "resolver '{resolver}' is a {kind}, not a declared action or a Decision.\\n  A resolver is the work that closes the issue or the Decision that moots it; the\\n  `resolver-kind` guard refuses any other #Resolves source at commit (issue136/issue558),\\n  so writing the edge here would only move that refusal to the gate. Name the action\\n  (a backlog item or sprint task) or the Decision, then re-run."
            ),
            Self::Model(e) => write!(f, "cannot read the model to check the resolver: {e}"),
        }
    }
}

/// The item process's own checks on a triage, in the order the command applied them: the severity is one
/// of the four, and the resolver is what the `resolver-kind` guard accepts, read through the guard's own
/// predicate (`keel_model::resolvers::resolver_kind_holds`) so the write refuses exactly what the commit
/// gate would (issue558): the first triage of issue556 pointed at a Story, the write printed "triaged on
/// arrival", and the pre-commit guard was the first thing to say otherwise.
///
/// # Errors
/// [`TriageRefusal`] naming what failed; nothing is written.
pub fn triage_holds(root: &Path, severity: &str, resolver: &str) -> Result<(), TriageRefusal> {
    if !["Critical", "High", "Medium", "Low"].contains(&severity) {
        return Err(TriageRefusal::Severity(severity.to_string()));
    }
    let actions = declared_task_names(root);
    if actions.contains(resolver) {
        return Ok(());
    }
    let model = Model::build(root).map_err(|e| TriageRefusal::Model(e.to_string()))?;
    match model.items.get(resolver) {
        None => Err(TriageRefusal::UnknownResolver(resolver.to_string())),
        Some(item) if !resolver_kind_holds(&actions, resolver, &item.type_name) => {
            Err(TriageRefusal::ResolverKind { resolver: resolver.to_string(), kind: item.type_name.clone() })
        }
        Some(_) => Ok(()),
    }
}

#[cfg(test)]
mod triage_tests {
    use super::{triage_holds, TriageRefusal};
    use std::path::Path;

    #[test]
    fn a_severity_outside_the_four_is_refused_before_the_model_is_read() {
        // A root that does not exist: the severity check must fire first, or this reads a model.
        let err = triage_holds(Path::new("Z:/no/such/root"), "medium", "anything").unwrap_err();
        assert!(matches!(err, TriageRefusal::Severity(ref s) if s == "medium"), "{err}");
        assert!(err.to_string().contains("Critical | High | Medium | Low"));
    }

    #[test]
    fn the_refusal_text_names_the_guard_whose_predicate_it_applied() {
        let e = TriageRefusal::ResolverKind { resolver: "st1".into(), kind: "Story".into() }.to_string();
        assert!(e.contains("is a Story, not a declared action or a Decision"), "{e}");
        assert!(e.contains("resolver-kind"), "{e}");
    }
}
'''

RESOLVERS = '''//! The resolver predicate (issue136/issue558): which item may be the source of a `#Resolves` edge. The
//! `resolver-kind` guard applies it at commit and `record issue --resolver` applies it before it writes,
//! so the two cannot disagree. Descended from keel-guards in sprint 737 (D0508) so the item member reads
//! it without depending on the guards; keel-guards re-exports both names at their old paths.

use std::collections::HashSet;
use std::path::Path;

'''

# ── slices: (file, start anchor (inclusive), end anchor (exclusive)) ─────────────────────────────────────
ISSUE_SLICE = (WRITE_SRC / "write.rs", "/// A new `Issue`, with the triage that makes it well-formed on arrival.\n", "// ── record claim (D0147 / D0129 srDcWorkClaim)")
OPEN_ISSUES_SLICE = (VIEW_SRC / "mod.rs", "/// Open-issues view (D0077) as JSON", "/// One declared `Viewpoint`, as every consumer of the registry needs it.")
DISPOSITIONS_SLICE = (VIEW_SRC / "mod.rs", "/// Dispositions view (D0092): every >= Medium finding + its typed disposition verdict.", "/// The set of sprint Story names covered by a `#Covers` edge (review -> sprint). Pure (for self-test).")
MEDIUM_SLICE = (VIEW_SRC / "mod.rs", "/// `true` if a severity string is >= Medium (the human-disposition tier, D0079).\n", "/// `true` if a severity string is >= High")
INTAKE_SLICE = (VIEW_SRC / "staleness.rs", "/// The INTAKE view (D0166): what was said, what it became, and what nobody acted on.\n", "/// `(outcome, judgedAgainst)` of the HIGHEST-numbered")
TASK_NAMES_SLICE = (GUARDS_SRC / "lib.rs", "/// All `action <name>;` task names declared in .tracking/{backlog,delivery} (not `action def`).\n", "/// Readiness, composed (D0079 c)")
KIND_SLICE = (GUARDS_SRC / "issues.rs", "/// THE ONE PREDICATE behind `resolver-kind`", "#[cfg(test)]\nmod taint_tests")

# anchored edits in files that stay: (file, old, new) - each anchor occurs exactly once
EDITS = [
    (WRITE_SRC / "write.rs",
     "/// Refuse every prose field that carries captured tool output (issue255).\nfn reject_injected_output(",
     "/// Refuse every prose field that carries captured tool output (issue255). `pub` since sprint 737: the\n/// issue write is keel-issues' and reads this same refusal (D0224 - one predicate, never a copy).\n///\n/// # Errors\n/// `WriteError::InjectedToolOutput` naming the field and an excerpt of what it saw.\npub fn reject_injected_output("),
    (WRITE_SRC / "lib.rs", "pub mod intake_write;\n", ""),
    (VIEW_SRC / "mod.rs",
     "use keel_model::queries::{days_between, issue_disposition, repo_today};\n",
     "use keel_model::queries::{at_least_medium, days_between, issue_disposition, repo_today};\n"),
    (VIEW_SRC / "staleness.rs", "use std::collections::{BTreeMap, HashMap, HashSet};\n", "use std::collections::{HashMap, HashSet};\n"),
    (GUARDS_SRC / "lib.rs", TASK_NAMES_SLICE[2], "// The resolver predicate is the read model's (sprint 737, D0508); the guards and `record issue` read one function.\npub use keel_model::resolvers::declared_task_names;\n\n" + TASK_NAMES_SLICE[2]),
    (GUARDS_SRC / "issues.rs", KIND_SLICE[2], "// THE ONE PREDICATE behind `resolver-kind` is keel_model::resolvers' (sprint 737, D0508); re-exported so `guards::issues::resolver_kind_holds` resolves.\npub use keel_model::resolvers::resolver_kind_holds;\n\n" + KIND_SLICE[2]),
    (MODEL_SRC / "lib.rs", "pub mod validate;\n", "pub mod validate;\npub mod resolvers;\n"),
    (MODEL_SRC / "queries.rs",
     "/// The latest recorded disposition verdict on a finding Issue (D0092)",
     "<<MEDIUM>>\n/// The latest recorded disposition verdict on a finding Issue (D0092)"),
    (CLI_SRC / "lib.rs", "pub mod github_ingest;\n", "pub use keel_issues::github_ingest;\n"),
    (CLI_SRC / "lib.rs", "pub use keel_write::intake_write;\n", "pub use keel_issues::intake_write;\n"),
    (CLI_SRC / "lib.rs", "pub use keel_view::view;\n",
     "// The item views and the issue write are member keel-issues' (D0480, sprint 737): `view` and `write` are\n// wrapper modules so `crate::view::open_issues` and `crate::write::record_issue` keep resolving.\npub mod view {\n    pub use keel_issues::views::{dispositions, intake, open_issues};\n    pub use keel_view::view::*;\n}\n"),
    (CLI_SRC / "lib.rs", "pub use keel_write::write;\n",
     "pub mod write {\n    pub use keel_issues::issue_write::{record_issue, NewIssue};\n    pub use keel_write::write::*;\n}\npub use keel_issues as issues;\n"),
    (CLI_SRC / "main.rs",
     "    if ![\"Critical\", \"High\", \"Medium\", \"Low\"].contains(&severity.as_str()) {\n        eprintln!(\"error: --severity must be Critical | High | Medium | Low (got '{severity}')\");\n        return 2;\n    }\n",
     "    // The item process's own checks - the severity and the resolver-kind predicate the commit gate\n    // applies (issue558) - are keel-issues' since sprint 737 (D0480); the refusal text is the member's.\n    if let Err(e) = keel_cli::issues::issue_write::triage_holds(&root, &severity, &resolver) {\n        eprintln!(\"error: {e}\");\n        return 2;\n    }\n"),
    (REPO / "Cargo.toml", '    "members/keel-process",\n    "keel-cli",', '    "members/keel-process",\n    "members/keel-issues",\n    "keel-cli",'),
    (REPO / "keel-cli" / "Cargo.toml", 'keel-process = { path = "../members/keel-process" }\n',
     'keel-process = { path = "../members/keel-process" }\n# D0480 item processes (sprint 737): github_ingest, intake_write, the issue write and the item views; re-exported under their old paths in lib.rs.\nkeel-issues = { path = "../members/keel-issues" }\n'),
]
# main.rs: the inline resolver check leaves for triage_holds - a slice with no destination (its text is the
# member's, re-authored as the predicate above); anchors as every other slice
MAIN_RESOLVER_SLICE = (CLI_SRC / "main.rs", "    // The resolver must be what the `resolver-kind` guard accepts, checked HERE through the guard's\n", "    // Bound to locals so the borrows outlive the struct")


def read(p: Path) -> str:
    return p.read_text(encoding="utf-8")


def write(p: Path, text: str) -> None:
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(text, encoding="utf-8", newline="\n")


def rewrite(text: str) -> tuple[str, dict[str, int]]:
    """Every `crate::X[::Y]` per MODULE_MAP; the counts per key are the reconciliation."""
    counts: dict[str, int] = {}

    def sub(m: re.Match[str]) -> str:
        path = m.group(1)
        head = path.split("::")[0]
        if head in OWN:
            return m.group(0)
        if head in MODULE_MAP:
            counts[head] = counts.get(head, 0) + 1
            return MODULE_MAP[head] + path[len(head):]
        raise SystemExit(f"unmapped crate path in a moved file: crate::{path}")

    return REF.sub(sub, text), counts


def plan_moves() -> list[tuple[Path, Path, str, dict[str, int], int]]:
    out = []
    for src, name in MOVES:
        text = read(src)
        new, counts = rewrite(text)
        before = len(REF.findall(text))
        after_foreign = sum(counts.values())
        after_own = len(REF.findall(new))
        if before != after_foreign + after_own:
            raise SystemExit(f"{src.name}: {before} crate:: refs before, {after_foreign} rewritten + {after_own} kept")
        delta = sum(n * (len(MODULE_MAP[k]) - len("crate::" + k)) for k, n in counts.items())
        if len(new) - len(text) != delta or (not counts and new != text):
            raise SystemExit(f"{src.name}: the rewrite is not the only difference")
        out.append((src, DST_SRC / name, new, counts, before))
    return out


def cut(text: str, start: str, end: str, label: str) -> tuple[str, str]:
    """(text without the slice, the slice). Both anchors occur exactly once; the end is exclusive."""
    if text.count(start) != 1:
        raise SystemExit(f"{label}: start anchor occurs {text.count(start)} times: {start[:50]!r}")
    if text.count(end) != 1:
        raise SystemExit(f"{label}: end anchor occurs {text.count(end)} times: {end[:50]!r}")
    i, j = text.index(start), text.index(end)
    if j <= i:
        raise SystemExit(f"{label}: slice ends before it starts")
    return text[:i] + text[j:], text[i:j]


class Plan:
    def __init__(self) -> None:
        self.files: dict[Path, str] = {}
        self.slices: dict[str, str] = {}
        self.log: list[str] = []

    def text(self, p: Path) -> str:
        if p not in self.files:
            self.files[p] = read(p)
        return self.files[p]

    def slice(self, spec: tuple[Path, str, str], label: str) -> str:
        p, start, end = spec
        before = self.text(p)
        remaining, piece = cut(before, start, end, label)
        if len(remaining) + len(piece) != len(before):
            raise SystemExit(f"{label}: the slice is not conserved")
        self.files[p] = remaining
        self.slices[label] = piece
        self.log.append(f"{p.relative_to(REPO).as_posix()}: {label} out, {len(piece)} chars")
        return piece

    def edit(self, p: Path, old: str, new: str) -> None:
        t = self.text(p)
        if t.count(old) != 1:
            raise SystemExit(f"{p.relative_to(REPO).as_posix()}: anchor occurs {t.count(old)} times: {old[:60]!r}")
        self.files[p] = t.replace(old, new)


def plan() -> tuple[Plan, dict[Path, str]]:
    pl = Plan()
    issue = pl.slice(ISSUE_SLICE, "issue write")
    open_issues = pl.slice(OPEN_ISSUES_SLICE, "open_issues")
    dispositions = pl.slice(DISPOSITIONS_SLICE, "dispositions")
    medium = pl.slice(MEDIUM_SLICE, "at_least_medium")
    intake = pl.slice(INTAKE_SLICE, "intake")
    task_names = pl.slice(TASK_NAMES_SLICE, "declared_task_names")
    kind = pl.slice(KIND_SLICE, "resolver_kind_holds")
    pl.slice(MAIN_RESOLVER_SLICE, "main.rs inline resolver check")
    # the descended scan reads its own crate's corpus walk
    n = task_names.count("keel_model::corpus::")
    if n != 2:
        raise SystemExit(f"declared_task_names: expected 2 keel_model::corpus:: refs, found {n}")
    task_names = task_names.replace("keel_model::corpus::", "crate::corpus::")
    for p, old, new in EDITS:
        if "<<MEDIUM>>" in new:
            # the descended predicate is read across the crate boundary: pub, and #[must_use] as every pub predicate here
            if medium.count("\nfn at_least_medium(") != 1:
                raise SystemExit("at_least_medium: expected one private fn to publish")
            public = medium.replace("\nfn at_least_medium(", "\n#[must_use]\npub fn at_least_medium(")
            new = new.replace("<<MEDIUM>>\n", public.rstrip("\n") + "\n\n")
        pl.edit(p, old, new)
    new_files = {
        DST / "Cargo.toml": CARGO,
        DST_SRC / "lib.rs": LIB,
        DST_SRC / "views.rs": VIEWS_HEAD + open_issues.rstrip("\n") + "\n\n" + dispositions.rstrip("\n") + "\n\n" + intake.rstrip("\n") + "\n",
        DST_SRC / "issue_write.rs": ISSUE_HEAD + issue.rstrip("\n") + "\n" + TRIAGE,
        MODEL_SRC / "resolvers.rs": RESOLVERS + task_names.rstrip("\n") + "\n\n" + kind.rstrip("\n") + "\n",
    }
    # reconciliation: every slice with a destination is in its destination whole
    for label, dest in (("issue write", DST_SRC / "issue_write.rs"), ("open_issues", DST_SRC / "views.rs"), ("dispositions", DST_SRC / "views.rs"), ("intake", DST_SRC / "views.rs"), ("resolver_kind_holds", MODEL_SRC / "resolvers.rs")):
        if pl.slices[label].strip() not in new_files[dest]:
            raise SystemExit(f"{label}: did not carry over whole")
    if task_names.strip() not in new_files[MODEL_SRC / "resolvers.rs"]:
        raise SystemExit("declared_task_names: did not carry over whole")
    if medium.replace("\nfn at_least_medium(", "\n#[must_use]\npub fn at_least_medium(").strip() not in pl.files[MODEL_SRC / "queries.rs"]:
        raise SystemExit("at_least_medium: did not carry over whole")
    return pl, new_files


def main(argv: list[str]) -> int:
    apply = "--apply" in argv
    if (DST_SRC / "lib.rs").exists():
        print("already applied: members/keel-issues/src/lib.rs exists")
        return 0
    moves = plan_moves()
    pl, new_files = plan()
    total_refs = 0
    for src, dst, _new, counts, before in moves:
        total_refs += before
        print(f"{src.relative_to(REPO).as_posix()} -> {dst.relative_to(REPO).as_posix()}: {before} crate:: refs, rewritten {dict(sorted(counts.items()))}")
    for line in pl.log:
        print(line)
    print(f"reconciliation: {len(moves)} files moved, {total_refs} crate:: refs each kept or rewritten; {len(pl.slices)} slices conserved; {len(EDITS)} anchored edits in {len(pl.files)} staying files; {len(new_files)} files written")
    if not apply:
        print("dry run; --apply to write")
        return 0
    DST_SRC.mkdir(parents=True, exist_ok=True)
    for src, dst, new, _counts, _before in moves:
        subprocess.run(["git", "mv", str(src), str(dst)], cwd=REPO, check=True)
        write(dst, new)
    for p, text in pl.files.items():
        write(p, text)
        print("edited", p.relative_to(REPO).as_posix())
    for p, text in new_files.items():
        write(p, text)
        print("wrote", p.relative_to(REPO).as_posix())
    print("applied")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
