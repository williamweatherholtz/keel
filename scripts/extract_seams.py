"""extract_seams.py - the sprint 740 transform: the five helper seams descend below the verbs (D0479).

not-an-instrument: a one-shot codemod (the migration skill's gate 1); its reconciliation totals are the
transform checking itself before it writes, not a measure of the project - the release build under
deny(warnings), the moved unit tests in their new crates, `keel --help` byte-identical and
`scripts/verb_homes.py --check` are the sensors the move answers to.

`scripts/verb_homes.py` showed why dcKeelCliIsThinDispatch could not move a single `cmd_` body: each one
reaches a helper that only the binary, or a member above the verb's owner, holds. Five seams, one script:

  (1) project discovery - `Workspace`, `is_project`, `missing_project_dirs`, `require_project`, `canon`,
      `discover`, `find_repo_root`, `engine_version_skew` and the private walkers - leaves
      members/keel-process/src/workspace.rs for members/keel-git/src/projects.rs (keel-git reaches gitx and
      nothing above it; a second workspace.rs would make `module_home('workspace')` ambiguous). The workspace
      GATE stays, and `keel_process::workspace` re-exports the moved names so every old path resolves.
  (2) the hook ledger writers `ledger_gate`, `ledger_refused`, `refused_injected_prose`, `ledger_line` and the
      private `ledger_actor_kind` leave members/keel-hooks/src/lib.rs for members/keel-write/src/ledger.rs, and
      the slow-fire threshold `SLOW_FIRE_MS` + `slow_fire_phases` leave members/keel-view/src/pm.rs for the same
      file, so the writer and the reader share one constant; `ledger_fire` stays beside the `EMITTED_VERDICT`
      it reads; keel-hooks stops depending on keel-process.
  (3) fork detection - `fork_options`, `NOT_A_FORK`, `fork_signals`, `disguised_fork` and their tests - leaves
      members/keel-serve/src/deck.rs for members/keel-model/src/textscan.rs; deck re-exports them.
  (4) `walk_longest` leaves keel-cli/src/lib.rs for members/keel-fs/src/lib.rs; lib.rs re-exports it.
  (5) the argument helpers - `without_flag_values`, `root_arg`, `refuse_flag_as_path`, `flag`, `prose_flag`,
      `prose_args`, `provenance_date`, `positional_arg`, `repo_arg` and the root_arg test - become member
      members/keel-args (depends on keel-git alone); main.rs imports them by name; lib.rs re-exports `args`.

Two doc comments that earlier moves left orphaned are reattached: `root_arg`'s (stranded in workspace.rs
when the fn moved without it) goes back above `root_arg`; `cmd_init`'s (stranded above `positional_arg`)
goes back above `fn cmd_init`. Every `keel_git::gitx::` in text landing in keel-git becomes `crate::gitx::`,
every `keel_write::write::` landing in keel-write becomes `crate::write::`; the counts are the reconciliation,
with each slice conserved out of its source and present whole in its destination.

    python scripts/extract_seams.py            # plan + reconciliation, nothing written
    python scripts/extract_seams.py --apply    # the slices, the rewrites, the manifests, the re-exports

Idempotent: a tree where members/keel-args/src/lib.rs exists is reported as already applied.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
CLI = REPO / "keel-cli"
CLI_SRC = CLI / "src"
MAIN = CLI_SRC / "main.rs"
CLI_LIB = CLI_SRC / "lib.rs"
MEMBERS = REPO / "members"
WORKSPACE = MEMBERS / "keel-process" / "src" / "workspace.rs"
PROJECTS = MEMBERS / "keel-git" / "src" / "projects.rs"
GIT_LIB = MEMBERS / "keel-git" / "src" / "lib.rs"
GIT_CARGO = MEMBERS / "keel-git" / "Cargo.toml"
HOOKS_LIB = MEMBERS / "keel-hooks" / "src" / "lib.rs"
HOOKS_CARGO = MEMBERS / "keel-hooks" / "Cargo.toml"
PM = MEMBERS / "keel-view" / "src" / "pm.rs"
PERF = MEMBERS / "keel-perf" / "src" / "perf.rs"
LEDGER = MEMBERS / "keel-write" / "src" / "ledger.rs"
WRITE_LIB = MEMBERS / "keel-write" / "src" / "lib.rs"
WRITE_CARGO = MEMBERS / "keel-write" / "Cargo.toml"
DECK = MEMBERS / "keel-serve" / "src" / "deck.rs"
TEXTSCAN = MEMBERS / "keel-model" / "src" / "textscan.rs"
FS_LIB = MEMBERS / "keel-fs" / "src" / "lib.rs"
ARGS = MEMBERS / "keel-args"
ARGS_LIB = ARGS / "src" / "lib.rs"
ARGS_CARGO = ARGS / "Cargo.toml"

# ── the slices: (start anchor, end anchor exclusive), each anchor occurring exactly once in its file ──
# (1) workspace.rs
WS_DISCOVERY = ("/// One git repository and every keel project inside it.", "/// Resolve a subcommand's optional `[ROOT]` positional, REFUSING an unrecognised flag (issue133).")
WS_ROOT_ARG_DOC = ("/// Resolve a subcommand's optional `[ROOT]` positional, REFUSING an unrecognised flag (issue133).", "/// One spelling for a path, so two that name the same directory compare equal.")
WS_WALKERS = ("/// One spelling for a path, so two that name the same directory compare equal.", "/// Staged paths that are KEYSTONE events and that no project gate covers (issue276).")
WS_ROOT_SKEW = ("// ── project discovery from the current directory (sprint 739, D0479)", "#[cfg(test)]\nmod tests {")
WS_TESTS = ("    #[test]\n    fn a_single_project_repo_is_unchanged_by_any_of_this() {", "    /// issue276: the two rules that make the unowned surface a verdict rather than a printed note.")
# (2) hooks lib.rs + pm.rs
HOOKS_LEDGER = ("/// A write-path refusal is a ledger fact (issue445)", "/// Minimal raw-HTTP call to the LOCAL console")
PM_SLOW = ("/// A hook fire at or past this many ms is SLOW (issue429 / D0414).", "/// One `slowFires` row (D0414 / issue429)")
# (3) deck.rs
DECK_FORK = ("/// Extract `OPTION X (short label)` enumerations from a Decision file's text.", "// The consent-marker classifier descended to the read model's textscan (sprint 733)")
DECK_FORK_TEST = ("    /// issue223: a Decision enumerating `OPTION X (label)` choices is a FORK", "    /// PASS ZERO (issue191, now enforced at the source)")
# (4) keel-cli lib.rs: the banner to EOF
CLI_WALK_BANNER = "// ── internal parse helper ─────────────────────────────────────────────────────\n"
# (5) main.rs
MAIN_ROOT_ARG = ("/// Drop each named flag AND ITS VALUE, leaving only true positionals for [`root_arg`].", "// ── subcommands ──")
MAIN_REFUSE_FLAG = ("/// Refuse an argument that LOOKS like a flag where a path or a name is expected (GH#14).", "/// A string that names a runnable guard (an enforced one, or a runnable-only diagnostic).")
MAIN_FLAGS = ("/// Parse simple `--key value` flag pairs from a flat args slice.", "fn cmd_append_result(args: &[String]) -> i32 {")
MAIN_INIT_DOC = ("/// `keel init DIR` (D0093) — scaffold a fresh project", "/// The first positional argument, REFUSING anything that looks like a flag (issue179).")
MAIN_POSITIONAL = ("/// The first positional argument, REFUSING anything that looks like a flag (issue179).", "/// Write the scaffolded pre-commit gate at `repo_root` and arm `core.hooksPath` there (issue278).")
MAIN_REPO_ARG = ("/// The repo a git-touching subcommand acts on: the first non-flag argument, else the discovered root.", "fn cmd_version(args: &[String]) -> i32 {")
MAIN_ROOT_ARG_TEST = ("    #[test]\n    fn an_unknown_flag_is_a_mistake_and_never_a_root() {", "    #[test]\n    fn guard_args_distinguish_name_from_root() {")

ARG_FNS = ["without_flag_values", "root_arg", "refuse_flag_as_path", "flag", "prose_flag", "prose_args", "provenance_date", "positional_arg", "repo_arg"]
MUST_USE = ["without_flag_values", "flag", "refuse_flag_as_path", "repo_arg"]
ERRORS_DOC = {
    "root_arg": "An unknown flag, no project found from the current directory, or a root that is not a keel project: the usage line is printed and the code is `2`.",
    "prose_flag": "Both forms of one name at once, or a `-from` file that cannot be read; the message names which.",
    "prose_args": "The exit code `2` after `prose_flag`'s message, for any name given in both forms or whose file cannot be read.",
    "provenance_date": "`2` after the usage line when the flag is absent - the date is stated or the write does not happen.",
    "positional_arg": "`2` after the usage line when there is no first argument or it looks like a flag.",
}
PROJECT_NAMES = ["canon", "discover", "engine_version_skew", "find_repo_root", "is_project", "missing_project_dirs", "require_project", "Workspace"]
LEDGER_NAMES = ["ledger_gate", "ledger_refused", "refused_injected_prose"]
FORK_NAMES = ["disguised_fork", "fork_options", "fork_signals", "NOT_A_FORK"]

GITX_REF = re.compile(r"\bkeel_git::gitx::")
WRITE_REF = re.compile(r"\bkeel_write::write::")
DECK_FORK_REF = re.compile(r"\bkeel_cli::deck::(fork_options|disguised_fork|NOT_A_FORK)\b")
FOREIGN = re.compile(r"\bkeel_[a-z_]+::")

LINTS = '''#![forbid(unsafe_code)]
#![deny(warnings, clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, clippy::todo, clippy::unimplemented)]
#![allow(clippy::implicit_hasher, clippy::too_long_first_doc_paragraph, clippy::module_name_repetitions)]
// Tests may use unwrap/expect/panic/indexing/asserts freely.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing))]
'''

ARGS_CARGO_TEXT = '''[package]
name = "keel-args"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "The argument helpers every keel verb shares: the [ROOT] positional that refuses unknown flags and non-projects, the flag-looks-like-a-path refusal, `--key value`, the prose flag with its `-from FILE` form, the never-defaulted provenance date."

[dependencies]
# root_arg / repo_arg resolve the discovered project root; keel-git is the one member below every verb (sprint 740, D0479).
keel-git = { path = "../keel-git" }

[dev-dependencies]
# keel_fs::test_support::repo_root, the real project the root_arg test parses around.
keel-fs = { path = "../keel-fs" }
'''

ARGS_LIB_HEAD = '''//! keel-args: the argument helpers every keel verb shares (D0479, sprint 740).
//!
//! `scripts/verb_homes.py` showed the shape: every `cmd_` body in keel-cli/src/main.rs parsed its arguments
//! through nine private fns of main.rs, so no body could leave the binary before they did. They moved here
//! whole by `scripts/extract_seams.py`: [`root_arg`] (the `[ROOT]` positional that refuses an unknown flag,
//! issue133, and a non-project, issue281), [`without_flag_values`], [`refuse_flag_as_path`] (GH#14),
//! [`flag`], [`prose_flag`] and [`prose_args`] (the `-from FILE` form, D0224), [`provenance_date`]
//! (never defaulted, issue182), [`positional_arg`] (issue179) and [`repo_arg`]. The crate depends on
//! keel-git alone, for project discovery; keel-cli re-exports it as `args` and main.rs imports the names.
''' + LINTS + '''
use std::path::PathBuf;

use keel_git::projects::{find_repo_root, require_project};

'''

PROJECTS_HEAD = '''//! Project discovery: one git repository and every keel project inside it (D0234), and the walk from
//! the current directory to the project root (issue281) with the D0190 engine-version pin warning.
//!
//! Moved whole out of members/keel-process/src/workspace.rs by `scripts/extract_seams.py` (sprint 740,
//! D0479): the workspace GATE stays there and re-exports these names, so `keel_process::workspace::` and
//! `keel_cli::workspace::` paths resolve unchanged. It lives in keel-git because every verb's argument
//! parsing (member keel-args) needs the discovered root, and keel-git is the one member below them all
//! that already spawns git. Not `workspace.rs`: two crates holding one module file would make
//! `scripts/module_home.py` ambiguous (issue559).
use std::path::{Path, PathBuf};

'''

LEDGER_HEAD = '''//! The hook fire ledger (`.keel/metrics/hooks.jsonl`): one line per fire, refusal and commit-gate tier.
//!
//! Moved out of members/keel-hooks/src/lib.rs by `scripts/extract_seams.py` (sprint 740, D0479) so the
//! write API's refusals (`keel accept` / `reject` / `record`) can be ledgered from the member that refuses
//! them, without the binary in between; keel-hooks re-exports the three writers it still calls and keeps
//! `ledger_fire` beside the emitted-verdict cell it reads. The slow-fire threshold came here from
//! keel-view's pm.rs with it: the writer that stamps `phases` and the report that reads them share one
//! constant, and pm re-exports it.
use std::path::Path;

'''

WS_REEXPORT = '''// Project discovery is keel-git's (`keel_git::projects`, sprint 740, D0479); the gate below keeps using
// the names, and every `keel_process::workspace::` / `keel_cli::workspace::` path resolves unchanged.
pub use keel_git::projects::{canon, discover, engine_version_skew, find_repo_root, is_project, missing_project_dirs, require_project, Workspace};

'''

# anchored edits in files that stay: (file, old, new) - each anchor occurs exactly once
EDITS = [
    # (1) workspace.rs keeps the gate; the tests import what stays
    (WORKSPACE, "//! positive if one existed. Discovery now descends (`discover`), so the claim is a measurement.\nuse std::path::{Path, PathBuf};\n\n",
     "//! positive if one existed. Discovery now descends (`discover`), so the claim is a measurement.\n//!\n//! Discovery itself is `keel_git::projects` since sprint 740; this module is the workspace GATE.\nuse std::path::{Path, PathBuf};\n\n" + WS_REEXPORT),
    (WORKSPACE, "    use super::{discover, is_project, unowned_keystone_events, Workspace};\n    use std::path::PathBuf;\n", "    use super::unowned_keystone_events;\n"),
    (GIT_LIB, "pub mod gitx;\npub mod eol;\n", "pub mod gitx;\npub mod eol;\n// Project discovery (D0234 / issue281), below every verb's argument parsing (sprint 740, D0479).\npub mod projects;\n"),
    (GIT_CARGO, 'keel-perf = { path = "../keel-perf" }\n', 'keel-perf = { path = "../keel-perf" }\n\n[dev-dependencies]\n# keel_fs::test_support::repo_root, the real project the discovery tests read.\nkeel-fs = { path = "../keel-fs" }\n'),
    # (2) hooks: project discovery from keel-git, the ledger writers from keel-write; no keel-process
    (HOOKS_LIB, "use keel_process::workspace::{engine_version_skew, find_repo_root};\n",
     "use keel_git::projects::{engine_version_skew, find_repo_root};\n"
     "// The ledger writers are the write API's (`keel_write::ledger`, sprint 740, D0479); the three the binary\n"
     "// calls keep resolving as `keel_hooks::`, and `ledger_fire` below writes through the same line writer.\n"
     "pub use keel_write::ledger::{ledger_gate, ledger_refused, refused_injected_prose};\nuse keel_write::ledger::ledger_line;\n"),
    (HOOKS_LIB, "//! refresh and the process layer's project discovery, and nothing reads it but the binary - so an edit to\n//! a hook rebuilds this crate and keel-cli only.\n",
     "//! refresh and keel-git's project discovery, and nothing reads it but the binary - so an edit to\n//! a hook rebuilds this crate and keel-cli only. The ledger writers descended to `keel_write::ledger`\n//! in sprint 740 (`scripts/extract_seams.py`); this crate re-exports the three the binary calls.\n"),
    (HOOKS_CARGO, 'keel-process = { path = "../keel-process" }\n', ""),
    (WRITE_LIB, "pub mod claude_surface;\n", "pub mod claude_surface;\npub mod ledger;\n"),
    (WRITE_CARGO, 'keel-schema = { path = "../keel-schema" }\n', 'keel-schema = { path = "../keel-schema" }\n# The slow-fire attribution the ledger line carries (D0414), with the threshold, since sprint 740.\nkeel-perf = { path = "../keel-perf" }\n'),
    (PM, "/// One `slowFires` row (D0414 / issue429)",
     "// The threshold and the `phases` builder are the ledger writer's (`keel_write::ledger`, sprint 740, D0479):\n// the writer that stamps a slow fire and this reader share one constant. Both keep resolving as `pm::`.\npub use keel_write::ledger::{slow_fire_phases, SLOW_FIRE_MS};\n\n/// One `slowFires` row (D0414 / issue429)"),
    (PERF, "a fire past `pm::SLOW_FIRE_MS`", "a fire past `ledger::SLOW_FIRE_MS`"),
    # (3) deck re-exports fork detection from textscan; textscan opens one file now and says so
    (DECK, "pub use keel_model::textscan::{marker_text_without_marker, marker_words, MARKER_WORDS, NOT_A_PROCESS_CHANGE};\n",
     "pub use keel_model::textscan::{marker_text_without_marker, marker_words, MARKER_WORDS, NOT_A_PROCESS_CHANGE};\n"
     "// Fork detection descended there too (sprint 740, D0479), so the record verbs reach it without the console.\n"
     "pub use keel_model::textscan::{disguised_fork, fork_options, fork_signals, NOT_A_FORK};\n"),
    (TEXTSCAN, "//! Pure scanners over `.sysml` TEXT - no model, no git, no filesystem.\n",
     "//! Pure scanners over `.sysml` TEXT - no model, no git; the one file read is [`fork_options`], which opens\n//! the Decision it is asked about (fork detection descended from the console's deck in sprint 740).\n"),
    (TEXTSCAN, "use std::collections::HashSet;\n", "use std::collections::HashSet;\nuse std::path::Path;\n"),
    # (4)/(5) keel-cli lib.rs re-exports
    (CLI_LIB, "pub use keel_git::gitx;\n",
     "pub use keel_git::gitx;\n// Project discovery and the shared argument helpers descended below the verbs (D0479, sprint 740).\npub use keel_git::projects;\npub use keel_args as args;\n"),
    (CLI_LIB, "pub use keel_hooks::shellcheck;\n", "pub use keel_hooks::shellcheck;\n// The hook ledger writers are the write API's (D0479, sprint 740).\npub use keel_write::ledger;\n"),
    # manifests
    (REPO / "Cargo.toml", '    "members/keel-git",\n', '    "members/keel-git",\n    "members/keel-args",\n'),
    (CLI / "Cargo.toml", 'keel-git = { path = "../members/keel-git" }\n',
     'keel-git = { path = "../members/keel-git" }\n# D0479 shared argument helpers (sprint 740): root_arg, flag, prose_flag and the rest; re-exported as `args` in lib.rs.\nkeel-args = { path = "../members/keel-args" }\n'),
    # main.rs: the tests import only what stays
    (MAIN, "        classify_guard_args, remap_engine_content, remap_engine_path, root_arg, Path,\n", "        classify_guard_args, remap_engine_content, remap_engine_path, Path,\n"),
]


def read(p: Path) -> str:
    return p.read_text(encoding="utf-8")


def write(p: Path, text: str) -> None:
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(text, encoding="utf-8", newline="\n")


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

    def slice(self, p: Path, spec: tuple[str, str], label: str) -> str:
        start, end = spec
        before = self.text(p)
        remaining, piece = cut(before, start, end, label)
        if len(remaining) + len(piece) != len(before):
            raise SystemExit(f"{label}: the slice is not conserved")
        self.files[p] = remaining
        self.slices[label] = piece
        self.log.append(f"{p.relative_to(REPO).as_posix()}: {label} out, {len(piece)} chars, {piece.count(chr(10))} lines")
        return piece

    def edit(self, p: Path, old: str, new: str) -> None:
        t = self.text(p)
        if t.count(old) != 1:
            raise SystemExit(f"{p.relative_to(REPO).as_posix()}: anchor occurs {t.count(old)} times: {old[:60]!r}")
        self.files[p] = t.replace(old, new)


def publish(text: str, names: list[str], label: str) -> str:
    """`fn x(` / `fn x<` -> `pub fn x`, each exactly once."""
    for name in names:
        hits = [a for a in (f"\nfn {name}(", f"\nfn {name}<") if text.count(a) == 1]
        if len(hits) != 1 or any(text.count(f"\nfn {name}{c}") > 1 for c in "(<"):
            raise SystemExit(f"{label}: expected one `fn {name}`")
        text = text.replace(hits[0], "\npub " + hits[0][1:])
    return text


def expect_once(text: str, names: list[str], label: str) -> None:
    for name in names:
        if text.count(name) != 1:
            raise SystemExit(f"{label}: expected one {name!r}, found {text.count(name)}")


def counted(label: str, text: str, ref: re.Pattern[str], new: str, expect: int) -> str:
    n = len(ref.findall(text))
    if n != expect:
        raise SystemExit(f"{label}: {n} {ref.pattern!r} refs, expected {expect}")
    return ref.sub(new, text)


def no_foreign(label: str, text: str, allowed: set[str]) -> None:
    bad = sorted({m.group(0) for m in FOREIGN.finditer(text)} - allowed)
    if bad:
        raise SystemExit(f"{label}: paths a lower crate cannot reach: {bad}")


def plan() -> tuple[Plan, dict[Path, str], dict[str, int]]:
    pl = Plan()
    totals: dict[str, int] = {}

    # ── (1) project discovery -> keel-git/src/projects.rs ──
    discovery = pl.slice(WORKSPACE, WS_DISCOVERY, "discovery types")
    root_arg_doc = pl.slice(WORKSPACE, WS_ROOT_ARG_DOC, "orphaned root_arg doc")
    walkers = pl.slice(WORKSPACE, WS_WALKERS, "canon and the walkers")
    root_skew = pl.slice(WORKSPACE, WS_ROOT_SKEW, "find_repo_root + engine_version_skew")
    ws_tests = pl.slice(WORKSPACE, WS_TESTS, "discovery tests")
    expect_once(discovery, ["pub struct Workspace {", "pub fn is_project(", "pub fn missing_project_dirs(", "pub fn require_project("], "discovery types")
    expect_once(walkers, ["pub fn canon(", "fn git_root(", "const SKIP:", "fn walk(", "fn git_known_paths(", "pub fn discover("], "canon and the walkers")
    expect_once(root_skew, ["pub fn find_repo_root()", "pub fn engine_version_skew("], "find_repo_root + engine_version_skew")
    expect_once(ws_tests, ["fn a_single_project_repo_is_unchanged_by_any_of_this()", "fn labels_and_ownership_resolve_within_the_workspace()", "fn discovery_finds_nested_and_deep_projects()"], "discovery tests")
    if not root_arg_doc.startswith("/// Resolve a subcommand") or not root_arg_doc.rstrip("\n").endswith("the caller returns that code unchanged."):
        raise SystemExit("orphaned root_arg doc: not the twelve doc lines expected")
    moved = discovery + walkers + root_skew
    gitx_in_code = len(GITX_REF.findall(moved))
    gitx_in_tests = len(GITX_REF.findall(ws_tests))
    moved = GITX_REF.sub("crate::gitx::", moved)
    ws_tests_new = GITX_REF.sub("crate::gitx::", ws_tests)
    totals["keel_git::gitx:: -> crate::gitx::"] = gitx_in_code + gitx_in_tests
    no_foreign("projects.rs", moved, {"keel_fs::"})
    no_foreign("projects.rs tests", ws_tests_new, {"keel_fs::"})
    projects = (
        PROJECTS_HEAD + moved.rstrip("\n") + "\n\n"
        "#[cfg(test)]\nmod tests {\n    use super::{discover, is_project, Workspace};\n    use std::path::PathBuf;\n\n"
        + ws_tests_new.rstrip("\n") + "\n}\n"
    )

    # ── (2) the ledger writers + the slow-fire threshold -> keel-write/src/ledger.rs ──
    ledger = pl.slice(HOOKS_LIB, HOOKS_LEDGER, "ledger writers")
    slow = pl.slice(PM, PM_SLOW, "slow-fire threshold")
    expect_once(ledger, ["pub fn ledger_refused(", "pub fn refused_injected_prose(", "pub fn ledger_gate(", "fn ledger_actor_kind(", "fn ledger_line("], "ledger writers")
    expect_once(slow, ["pub const SLOW_FIRE_MS: u64 = 3000;", "pub fn slow_fire_phases("], "slow-fire threshold")
    ledger = counted("ledger writers", ledger, WRITE_REF, "crate::write::", 1)
    totals["keel_write::write:: -> crate::write::"] = 1
    ledger = counted("ledger writers", ledger, re.compile(r"\bkeel_view::pm::slow_fire_phases\("), "slow_fire_phases(", 1)
    totals["keel_view::pm::slow_fire_phases -> slow_fire_phases"] = 1
    pl_old = "A fast fire carries no field (pm.rs owns the\n        // threshold and the reader)."
    if ledger.count(pl_old) != 1:
        raise SystemExit("ledger writers: the threshold comment moved")
    ledger = ledger.replace(pl_old, "A fast fire carries no field (the threshold is\n        // [`SLOW_FIRE_MS`] above; pm.rs is the reader).")
    ledger = ledger.replace(
        "\nfn ledger_line(",
        "\n/// The one line writer: every fire, refusal and commit-gate tier appends one JSON object to\n"
        "/// `.keel/metrics/hooks.jsonl`. `verdict` is `(decision, control)` when the caller has one; otherwise the\n"
        "/// exit code decides `allow` / `block`. Unavailable or failed writes are reported on stderr, never fatal:\n"
        "/// the ledger observes the hook, it does not gate it.\n"
        "pub fn ledger_line(", 1)
    expect_once(ledger, ["pub fn ledger_line("], "ledger writers")
    no_foreign("ledger.rs", ledger + slow, {"keel_actor::", "keel_perf::"})
    ledger_rs = LEDGER_HEAD + slow.rstrip("\n") + "\n\n" + ledger.rstrip("\n") + "\n"

    # ── (3) fork detection -> keel-model/src/textscan.rs ──
    fork = pl.slice(DECK, DECK_FORK, "fork detection")
    fork_test = pl.slice(DECK, DECK_FORK_TEST, "fork_options test")
    expect_once(fork, ["pub fn fork_options(", "pub const NOT_A_FORK:", "pub fn fork_signals(", "pub fn disguised_fork(", "mod fork_shape_tests {"], "fork detection")
    expect_once(fork_test, ["fn fork_options_parse_the_d0192_shape()"], "fork_options test")
    no_foreign("textscan.rs additions", fork + fork_test, {"keel_fs::"})
    ts_anchor = "#[cfg(test)]\nmod marker_text_tests {"
    pl.edit(TEXTSCAN, ts_anchor, fork.rstrip("\n") + "\n\n" + ts_anchor)
    pl.files[TEXTSCAN] = pl.text(TEXTSCAN).rstrip("\n") + "\n\n#[cfg(test)]\nmod fork_options_tests {\n    use super::fork_options;\n\n" + fork_test.rstrip("\n") + "\n}\n"
    main_text = pl.text(MAIN)
    pl.files[MAIN] = counted("main.rs", main_text, DECK_FORK_REF, r"keel_cli::textscan::\1", 4)
    totals["keel_cli::deck::fork_* -> keel_cli::textscan::"] = 4

    # ── (4) walk_longest -> keel-fs ──
    lib = pl.text(CLI_LIB)
    if lib.count(CLI_WALK_BANNER) != 1:
        raise SystemExit("keel-cli lib.rs: the internal-parse-helper banner is not there once")
    head, tail = lib.split(CLI_WALK_BANNER)
    walk_doc_start = "/// The character length of the longest path under `dir`"
    if tail.count(walk_doc_start) != 1 or tail.count("\nfn ") + tail.count("\npub fn ") != 1:
        raise SystemExit("keel-cli lib.rs: the banner is not followed by walk_longest alone")
    walk = tail[tail.index(walk_doc_start):].rstrip("\n") + "\n"
    pl.slices["walk_longest"] = walk
    pl.log.append(f"keel-cli/src/lib.rs: walk_longest out, {len(walk)} chars, {walk.count(chr(10))} lines")
    pl.files[CLI_LIB] = head + "// The init path-length walk is keel-fs's (sprint 740, D0479); `keel_cli::walk_longest` keeps resolving.\npub use keel_fs::walk_longest;\n"
    pl.files[FS_LIB] = pl.text(FS_LIB).rstrip("\n") + "\n\n" + walk

    # ── (5) the argument helpers -> member keel-args ──
    a_root = pl.slice(MAIN, MAIN_ROOT_ARG, "without_flag_values + root_arg")
    a_refuse = pl.slice(MAIN, MAIN_REFUSE_FLAG, "refuse_flag_as_path")
    a_flags = pl.slice(MAIN, MAIN_FLAGS, "flag, prose_flag, prose_args, provenance_date")
    init_doc = pl.slice(MAIN, MAIN_INIT_DOC, "orphaned cmd_init doc")
    a_pos = pl.slice(MAIN, MAIN_POSITIONAL, "positional_arg")
    a_repo = pl.slice(MAIN, MAIN_REPO_ARG, "repo_arg")
    a_test = pl.slice(MAIN, MAIN_ROOT_ARG_TEST, "root_arg test")
    if init_doc.count("\n") != 3 or not init_doc.startswith("/// `keel init DIR` (D0093)"):
        raise SystemExit("orphaned cmd_init doc: not the three doc lines expected")
    pl.edit(MAIN, "\nfn cmd_init(args: &[String]) -> i32 {", "\n" + init_doc + "fn cmd_init(args: &[String]) -> i32 {")
    # root_arg's stranded doc goes back above it, with the `# Errors` section a pub fn under pedantic carries
    root_arg_doc_full = root_arg_doc.rstrip("\n") + "\n///\n/// # Errors\n///\n/// " + ERRORS_DOC["root_arg"] + "\n"
    if a_root.count("\nfn root_arg(") != 1:
        raise SystemExit("root_arg: not found once in its slice")
    a_root = a_root.replace("\nfn root_arg(", "\n" + root_arg_doc_full + "fn root_arg(", 1)
    body = "".join(s.rstrip("\n") + "\n\n" for s in (a_root, a_refuse, a_flags, a_pos, a_repo)).rstrip("\n") + "\n"
    body = counted("keel-args", body, re.compile(r"\bkeel_cli::workspace::require_project\("), "require_project(", 2)
    totals["keel_cli::workspace::require_project -> require_project"] = 2
    body = publish(body, ARG_FNS, "keel-args")
    for name in MUST_USE:
        body = body.replace(f"\npub fn {name}(", f"\n#[must_use]\npub fn {name}(", 1)
    for name, why in ERRORS_DOC.items():
        if name == "root_arg":
            continue
        anchor = f"\npub fn {name}(" if name != "positional_arg" else "\npub fn positional_arg<"
        if body.count(anchor) != 1:
            raise SystemExit(f"keel-args: {anchor.strip()!r} not once")
        body = body.replace(anchor, f"\n///\n/// # Errors\n///\n/// {why}" + anchor, 1)
    no_foreign("keel-args", body + a_test, {"keel_fs::"})
    args_lib = ARGS_LIB_HEAD + body + "\n#[cfg(test)]\nmod tests {\n    use super::root_arg;\n\n" + a_test.rstrip("\n") + "\n}\n"

    # main.rs imports only the names it still uses (deny(warnings) makes an unused import a red build)
    for p, old, new in EDITS:
        pl.edit(p, old, new)
    main_text = pl.text(MAIN)
    old_imports = (
        "// The hooks are member keel-hooks (D0479, sprint 739); project discovery is the process layer's.\n"
        "use keel_cli::hooks::{cmd_hook, ledger_gate, ledger_refused, override_path, override_target, refused_injected_prose, OVERRIDE_TTL_SECS, RECALL_BUDGET};\n"
        "use keel_cli::workspace::{engine_version_skew, find_repo_root};\n"
    )
    if main_text.count(old_imports) != 1:
        raise SystemExit("main.rs: the sprint-739 import block is not there once")
    rest = main_text.replace(old_imports, "")
    used = lambda names: [n for n in names if re.search(r"\b" + n + r"\b", rest)]  # noqa: E731
    args_used, ledger_used = used(ARG_FNS), used(LEDGER_NAMES)
    if len(args_used) != len(ARG_FNS):
        raise SystemExit(f"main.rs no longer names {sorted(set(ARG_FNS) - set(args_used))}: drop them from the import")
    new_imports = (
        "// The hooks are member keel-hooks (D0479, sprint 739).\n"
        "use keel_cli::hooks::{cmd_hook, override_path, override_target, OVERRIDE_TTL_SECS, RECALL_BUDGET};\n"
        "// Project discovery is keel-git's, the ledger writers the write API's, the shared argument helpers keel-args' (D0479, sprint 740).\n"
        "use keel_cli::projects::{engine_version_skew, find_repo_root};\n"
        f"use keel_cli::ledger::{{{', '.join(ledger_used)}}};\n"
        f"use keel_cli::args::{{{', '.join(sorted(args_used))}}};\n"
    )
    pl.files[MAIN] = main_text.replace(old_imports, new_imports)

    new_files = {PROJECTS: projects, LEDGER: ledger_rs, ARGS_CARGO: ARGS_CARGO_TEXT, ARGS_LIB: args_lib}

    # reconciliation: every slice is whole in its destination, modulo the counted rewrites and the pubs
    def carried(label: str, piece: str, dest: str, *subs: tuple[str, str]) -> None:
        p = piece.rstrip("\n")
        for old, new in subs:
            p = p.replace(old, new)
        if p not in dest:
            raise SystemExit(f"{label}: did not carry over whole")

    carried("discovery types", discovery, projects, ("keel_git::gitx::", "crate::gitx::"))
    carried("canon and the walkers", walkers, projects, ("keel_git::gitx::", "crate::gitx::"))
    carried("find_repo_root + engine_version_skew", root_skew, projects)
    carried("discovery tests", ws_tests, projects, ("keel_git::gitx::", "crate::gitx::"))
    carried("slow-fire threshold", slow, ledger_rs)
    carried("fork detection", fork, pl.files[TEXTSCAN])
    carried("fork_options test", fork_test, pl.files[TEXTSCAN])
    carried("walk_longest", walk, pl.files[FS_LIB])
    carried("orphaned cmd_init doc", init_doc, pl.files[MAIN])
    carried("orphaned root_arg doc", root_arg_doc, args_lib)
    carried("root_arg test", a_test, args_lib)
    for label, piece in (("refuse_flag_as_path", a_refuse), ("positional_arg", a_pos), ("repo_arg", a_repo)):
        stripped = re.sub(r"\n(#\[must_use\]\n)?pub fn ", "\nfn ", args_lib)
        stripped = re.sub(r"///\n/// # Errors\n///\n/// [^\n]*\n", "", stripped)
        if piece.rstrip("\n") not in stripped:
            raise SystemExit(f"{label}: did not carry over whole")
    for name in ARG_FNS:
        if len(re.findall(r"\bpub fn " + name + r"\b", args_lib)) != 1:
            raise SystemExit(f"keel-args: pub fn {name} not once")
    carried("ledger writers", ledger, ledger_rs)
    return pl, new_files, totals


def main(argv: list[str]) -> int:
    apply = "--apply" in argv
    if ARGS_LIB.exists():
        print("already applied: members/keel-args/src/lib.rs exists")
        return 0
    pl, new_files, totals = plan()
    for line in pl.log:
        print(line)
    print("rewrites: " + ", ".join(f"{k}={v}" for k, v in totals.items()))
    print(f"reconciliation: {len(pl.slices)} slices conserved out of 5 source files and present whole in their destinations; "
          f"{len(ARG_FNS)} fns published; 2 orphaned docs reattached; {len(EDITS)} anchored edits in {len(pl.files)} staying files; {len(new_files)} files written")
    if not apply:
        print("dry run; --apply to write")
        return 0
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
