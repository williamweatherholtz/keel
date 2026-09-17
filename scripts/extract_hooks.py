"""extract_hooks.py - the sprint 739 transform: the Claude hooks become member keel-hooks (D0479).

not-an-instrument: it is a one-shot codemod (the migration skill's gate 1); its reconciliation totals are the
transform checking itself before it writes, not a measure of the project - the release build under
deny(warnings), the hook unit tests and `keel --help` byte-identical are the sensors the move answers to.

One committed script, dry-run by default (D0479 one extraction per sprint; the migration skill's gate 1).
This is the first extraction that slices main.rs itself: the hook block (`cmd_hook` through `hook_stop`,
1,248 lines - the harness's Stop / PostToolUse / PreToolUse / UserPromptSubmit / SubagentStop entry points,
the ledger writers, the override unlock, the bash and write classifiers, the console probe and the headless
ask) leaves keel-cli/src/main.rs whole and lands in members/keel-hooks/src/lib.rs, with its three test
modules. Two files move as git renames beside it, because the hooks are their only readers:

    keel-cli/src/shellcheck.rs  -> members/keel-hooks/src/shellcheck.rs   (the pre-bash advisories, D0309/D0491)
    keel-cli/src/proactive.rs   -> members/keel-hooks/src/proactive.rs    (the post-edit advisories)

Two dispatch-level helpers the block calls, `find_repo_root` (the cwd walk to `.engine`, issue281) and
`engine_version_skew` (the D0190 pin warning), are project discovery, so they land in
members/keel-process/src/workspace.rs beside `require_project` - a member cannot call into the binary. The
two recall constants the user-prompt hook shares with `keel recall` ride with the hooks and are re-exported.
Every `keel_cli::<module>::` path in the moved text is rewritten per the map of the crate it lands in; the
six fns main.rs still calls (`cmd_hook`, `ledger_gate`, `ledger_refused`, `refused_injected_prose`, the
override pair) and three consts (`RECALL_CAP_MS`, `RECALL_BUDGET`, `OVERRIDE_TTL_SECS`) become `pub`, the
eight of them main.rs names are imported at its top, and keel-cli's lib.rs re-exports
`keel_hooks` as `hooks` plus `shellcheck` and `proactive` at their old paths, so no other caller moved.
The reconciliation holds each moved file's text equal to its source modulo the counted rewrites, each
slice conserved out of main.rs and present whole in its destination, and each staying file equal to its
source minus its slices plus the anchored edits.

    python scripts/extract_hooks.py            # plan + reconciliation, nothing written
    python scripts/extract_hooks.py --apply    # the renames, the rewrites, the slices, the manifests, the re-exports

Idempotent: a tree where members/keel-hooks/src/lib.rs exists is reported as already applied.
"""
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
CLI = REPO / "keel-cli"
CLI_SRC = CLI / "src"
MAIN = CLI_SRC / "main.rs"
MEMBERS = REPO / "members"
HOOKS = MEMBERS / "keel-hooks"
HOOKS_SRC = HOOKS / "src"
PROCESS_SRC = MEMBERS / "keel-process" / "src"

# `keel_cli::<module>` in the hook block -> the path that resolves from keel-hooks. A module the destination
# owns becomes `crate::`.
HOOKS_MAP = {
    "write": "keel_write::write",
    "claude_surface": "keel_write::claude_surface",
    "perf": "keel_perf::perf",
    "receipt": "keel_guards::receipt",
    "guards": "keel_guards",
    "actor": "keel_actor::actor",
    "view": "keel_view::view",
    "pm": "keel_view::pm",
    "shellcheck": "crate::shellcheck",
    "proactive": "crate::proactive",
    "fingerprint": "keel_model::fingerprint",
    "validate_root": "keel_model::validate::validate_root",
    "activation": "keel_model::activation",
    "hook_binary": "keel_suite::hook_binary",
    "gitx": "keel_git::gitx",
}
# `crate::<module>` in a moved FILE -> the path from keel-hooks
PROACTIVE_MAP = {"gitx": "keel_git::gitx", "view": "keel_view::view"}
HOOKS_OWN = {"shellcheck", "proactive"}

# (source, destination, map, own)
MOVES = [
    (CLI_SRC / "shellcheck.rs", HOOKS_SRC / "shellcheck.rs", {}, HOOKS_OWN),
    (CLI_SRC / "proactive.rs", HOOKS_SRC / "proactive.rs", PROACTIVE_MAP, HOOKS_OWN),
]
CRATE_REF = re.compile(r"\bcrate::([A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*)")
CLI_REF = re.compile(r"\bkeel_cli::([A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*)")

# the slices out of main.rs: (start anchor, end anchor exclusive), each anchor occurring exactly once
REPO_ROOT_SLICE = ("// ── repo-root discovery", "/// Drop each named flag AND ITS VALUE")
SKEW_SLICE = ("/// D0190: the engine-version parity warning.", "fn cmd_validate(args: &[String]) -> i32 {")
HOOK_SLICE = ("/// `keel hook <stop|post-edit>` (D0134) — the in-loop gates, IN THE BINARY.", "/// `keel gate --fast [ROOT]` (D0128 Tier-2)")
RECALL_SLICE = ("/// Hard latency cap for prompt-path recall.", "fn budget_arg(args: &[String]) -> usize {")
TEST_MODS_SLICE = ("#[cfg(test)]\nmod subagent_block_control_tests {", "#[cfg(test)]\nmod tests {")
TEST_FNS_SLICE = ("    /// issue427 (stpa-self run 2, UCA-O1): an override unlock covers exactly one file.", "    #[test]\n    fn an_unknown_flag_is_a_mistake_and_never_a_root() {")

# the names main.rs and the tests still call: `fn x(` -> `pub fn x(` inside the hook slice, each once
PUBLISHED = ["cmd_hook", "ledger_refused", "refused_injected_prose", "ledger_gate", "override_path", "override_target"]
PUB_CONSTS = ["RECALL_CAP_MS", "RECALL_BUDGET", "OVERRIDE_TTL_SECS"]
PUB_DOC_EDITS = [
    ("\npub fn override_path(", "\n#[must_use]\npub fn override_path("),
    ("/// the file-not-directory rule. Returns the canonical key the unlock stores.\npub fn override_target(",
     "/// the file-not-directory rule. Returns the canonical key the unlock stores.\n///\n/// # Errors\n///\n/// A directory, a prefix, or a file nowhere: the message names the file-not-directory rule.\npub fn override_target("),
]

LINTS = '''#![forbid(unsafe_code)]
#![deny(warnings, clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, clippy::todo, clippy::unimplemented)]
#![allow(clippy::implicit_hasher, clippy::too_long_first_doc_paragraph, clippy::module_name_repetitions)]
// Tests may use unwrap/expect/panic/indexing/asserts freely.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing))]
'''

CARGO = '''[package]
name = "keel-hooks"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "The Claude hooks (D0134): the Stop, PostToolUse, PreToolUse, UserPromptSubmit and SubagentStop entry points, their ledger, the override unlock, the bash and write classifiers, the pre-bash and post-edit advisories - the in-loop gates, in the binary, above every member they read and below keel-cli."
# The console probe reports the binary's engine build (KEEL_BUILD_COMMIT); the script that bakes the commit is
# keel-cli's, shared, not copied (sprint 736, as keel-guards, keel-write and keel-process).
build = "../../keel-cli/build.rs"

[dependencies]
keel-perf = { path = "../keel-perf" }
keel-git = { path = "../keel-git" }
keel-actor = { path = "../keel-actor" }
keel-model = { path = "../keel-model" }
keel-write = { path = "../keel-write" }
keel-view = { path = "../keel-view" }
keel-guards = { path = "../keel-guards" }
keel-suite = { path = "../keel-suite" }
keel-process = { path = "../keel-process" }
serde_json = "1"

[dev-dependencies]
# keel_fs::scratch, the one scratch dir for tests (issue557).
keel-fs = { path = "../keel-fs" }
'''

LIB_HEAD = '''//! keel-hooks: the Claude hooks (D0134) - the in-loop gates, IN THE BINARY.
//!
//! The ninth D0479 extraction (sprint 739), and the first to slice `main.rs` itself: `cmd_hook` through
//! `hook_stop` (the `Stop` / `PostToolUse` / `PreToolUse` / `UserPromptSubmit` / `SubagentStop` entry points, the
//! ledger writers, the override unlock, the bash and write classifiers, the console probe and the headless
//! ask) moved here whole by `scripts/extract_hooks.py`, with `shellcheck` (the pre-bash advisories) and
//! `proactive` (the post-edit advisories), whose only readers they are. keel-cli re-exports the crate as
//! `hooks` and the two modules at their old paths, and `main.rs` dispatches `keel hook` to [`cmd_hook`]
//! exactly as before. The crate reads the write API, the guards, the view layer, the suite's hook-binary
//! refresh and the process layer's project discovery, and nothing reads it but the binary - so an edit to
//! a hook rebuilds this crate and keel-cli only.
''' + LINTS + '''
pub mod proactive;
pub mod shellcheck;

use std::path::{Path, PathBuf};

use keel_process::workspace::{engine_version_skew, find_repo_root};

'''

WORKSPACE_HEAD = '''
// ── project discovery from the current directory (sprint 739, D0479) ─────────
// Moved whole out of keel-cli/src/main.rs by `scripts/extract_hooks.py`: the hooks call both, and a
// member cannot call into the binary. keel-cli imports them from here.

'''

# anchored edits in files that stay: (file, old, new) - each anchor occurs exactly once
EDITS = [
    # main.rs: the names it still calls arrive as imports where the other keel_cli imports sit
    (MAIN, "use keel_cli::write as w;\n",
     "use keel_cli::write as w;\n"
     "// The hooks are member keel-hooks (D0479, sprint 739); project discovery is the process layer's.\n"
     "use keel_cli::hooks::{cmd_hook, ledger_gate, ledger_refused, override_path, override_target, refused_injected_prose, OVERRIDE_TTL_SECS, RECALL_BUDGET};\n"
     "use keel_cli::workspace::{engine_version_skew, find_repo_root};\n"),
    # main.rs: the remaining tests import only what stays
    (MAIN,
     "        bash_classify, bash_tokens, classify_guard_args, consume_override, override_path, override_target, remap_engine_content, remap_engine_path,\n        root_arg, BashVerdict, Path,\n",
     "        classify_guard_args, remap_engine_content, remap_engine_path, root_arg, Path,\n"),
    # keel-cli lib.rs: the two modules become re-exports; the crate is `hooks`
    (CLI_SRC / "lib.rs", "pub mod proactive;\n", "pub use keel_hooks::proactive;\n"),
    (CLI_SRC / "lib.rs", "pub mod shellcheck;\n",
     "// The Claude hooks are member keel-hooks (D0479, sprint 739): cmd_hook and its ledger, shellcheck, proactive; the old paths keep resolving.\npub use keel_hooks as hooks;\npub use keel_hooks::shellcheck;\n"),
    # the manifests
    (REPO / "Cargo.toml", '    "members/keel-issues",\n    "members/keel-serve",', '    "members/keel-issues",\n    "members/keel-hooks",\n    "members/keel-serve",'),
    (CLI / "Cargo.toml", 'keel-serve = { path = "../members/keel-serve" }\n',
     'keel-serve = { path = "../members/keel-serve" }\n# D0479 Claude hooks (sprint 739): cmd_hook and its ledger, shellcheck, proactive; re-exported as `hooks` and under the old module paths in lib.rs.\nkeel-hooks = { path = "../members/keel-hooks" }\n'),
]


def read(p: Path) -> str:
    return p.read_text(encoding="utf-8")


def write(p: Path, text: str) -> None:
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(text, encoding="utf-8", newline="\n")


def rewrite(text: str, ref: re.Pattern[str], module_map: dict[str, str], own: set[str], prefix: str) -> tuple[str, dict[str, int]]:
    """Every `<prefix>::X[::Y]` per the map; the counts per key are the reconciliation."""
    counts: dict[str, int] = {}

    def sub(m: re.Match[str]) -> str:
        path = m.group(1)
        segs = path.split("::")
        for key in ("::".join(segs[:2]), segs[0]):
            if key in module_map:
                counts[key] = counts.get(key, 0) + 1
                return module_map[key] + path[len(key):]
        if segs[0] in own:
            return m.group(0)
        raise SystemExit(f"unmapped {prefix} path in moved text: {prefix}::{path}")

    return ref.sub(sub, text), counts


def checked_rewrite(label: str, text: str, ref: re.Pattern[str], module_map: dict[str, str], own: set[str], prefix: str) -> tuple[str, dict[str, int], int]:
    new, counts = rewrite(text, ref, module_map, own, prefix)
    before = len(ref.findall(text))
    after_foreign = sum(counts.values())
    after_own = len(ref.findall(new))
    if before != after_foreign + after_own:
        raise SystemExit(f"{label}: {before} {prefix}:: refs before, {after_foreign} rewritten + {after_own} kept")
    delta = sum(n * (len(module_map[k]) - len(prefix + "::" + k)) for k, n in counts.items())
    if len(new) - len(text) != delta:
        raise SystemExit(f"{label}: the rewrite is not the only difference")
    return new, counts, before


def plan_moves() -> list[tuple[Path, Path, str, dict[str, int], int]]:
    out = []
    for src, dst, module_map, own in MOVES:
        text = read(src)
        new, counts, before = checked_rewrite(src.name, text, CRATE_REF, module_map, own, "crate")
        out.append((src, dst, new, counts, before))
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


def publish(text: str, names: list[str], kind: str, label: str) -> str:
    """`fn x(` / `const X:` -> `pub ...`, each exactly once; the count is the reconciliation."""
    for name in names:
        anchor = f"\n{kind} {name}" + ("(" if kind == "fn" else ":")
        if text.count(anchor) != 1:
            raise SystemExit(f"{label}: {anchor.strip()!r} occurs {text.count(anchor)} times")
        text = text.replace(anchor, f"\npub {kind} {name}" + ("(" if kind == "fn" else ":"))
    return text


def expect_once(text: str, names: list[str], label: str) -> None:
    for name in names:
        if text.count(name) != 1:
            raise SystemExit(f"{label}: expected one {name!r}, found {text.count(name)}")


def plan() -> tuple[Plan, dict[Path, str], dict[str, int]]:
    pl = Plan()
    repo_root = pl.slice(MAIN, REPO_ROOT_SLICE, "find_repo_root")
    skew = pl.slice(MAIN, SKEW_SLICE, "engine_version_skew")
    hooks = pl.slice(MAIN, HOOK_SLICE, "hook block")
    recall = pl.slice(MAIN, RECALL_SLICE, "recall constants")
    test_mods = pl.slice(MAIN, TEST_MODS_SLICE, "hook test modules")
    test_fns = pl.slice(MAIN, TEST_FNS_SLICE, "hook tests in mod tests")
    expect_once(repo_root, ["fn find_repo_root() -> Option<PathBuf> {"], "find_repo_root")
    expect_once(skew, ["fn engine_version_skew(root: &Path) -> Option<String> {"], "engine_version_skew")
    expect_once(hooks, ["fn cmd_hook(", "fn hook_stop(", "fn hook_pre_write(", "fn hook_pre_bash(", "fn hook_post_edit(", "fn hook_user_prompt(", "fn hook_subagent_stop(", "const CONSOLE_PORT:", "const HOOK_DEADLINE_SECS:"], "hook block")
    expect_once(recall, ["const RECALL_CAP_MS:", "const RECALL_BUDGET:"], "recall constants")
    expect_once(test_mods, ["mod subagent_block_control_tests {", "mod subagent_stop_route_tests {"], "hook test modules")
    expect_once(test_fns, ["fn override_unlocks_exactly_one_file()", "fn bash_matcher_is_argv_level_and_carves_out_human_judgment()"], "hook tests in mod tests")
    if CRATE_REF.search(hooks):
        raise SystemExit("hook block: a `crate::` path in the binary's own text - map it")
    for p, old, new in EDITS:
        pl.edit(p, old, new)

    # the hook block, the constants and the tests land in keel-hooks with their keel_cli:: paths rewritten
    moved = recall + "\n" + hooks
    body, counts, before = checked_rewrite("hook block", moved, CLI_REF, HOOKS_MAP, set(), "keel_cli")
    body = publish(body, PUBLISHED, "fn", "hook block")
    body = publish(body, PUB_CONSTS, "const", "recall constants")
    # a pub fn in a lib is held to pedantic: the path getter is must_use, the Result carries its `# Errors`
    for old, new in PUB_DOC_EDITS:
        if body.count(old) != 1:
            raise SystemExit(f"hook block: pub-doc anchor occurs {body.count(old)} times: {old[:60]!r}")
        body = body.replace(old, new)
    if CLI_REF.search(test_mods + test_fns):
        raise SystemExit("hook tests: a keel_cli:: path - map it")
    tests = (
        test_mods.rstrip("\n") + "\n\n"
        "#[cfg(test)]\nmod tests {\n"
        "    use super::{bash_classify, bash_tokens, consume_override, override_path, override_target, BashVerdict};\n\n"
        + test_fns.rstrip("\n") + "\n}\n"
    )
    lib = LIB_HEAD + body.rstrip("\n") + "\n\n" + tests

    # the two helpers land in workspace.rs as pub
    helpers = repo_root.rstrip("\n") + "\n\n" + skew.rstrip("\n") + "\n"
    helpers = helpers.replace("\nfn find_repo_root()", "\n#[must_use]\npub fn find_repo_root()", 1).replace("\nfn engine_version_skew(", "\n#[must_use]\npub fn engine_version_skew(", 1)
    expect_once(helpers, ["pub fn find_repo_root()", "pub fn engine_version_skew("], "helpers")
    ws = PROCESS_SRC / "workspace.rs"
    # before the file's test module: clippy's items_after_test_module
    pl.edit(ws, "#[cfg(test)]\nmod tests {", WORKSPACE_HEAD.lstrip("\n") + helpers + "\n#[cfg(test)]\nmod tests {")

    new_files = {HOOKS / "Cargo.toml": CARGO, HOOKS_SRC / "lib.rs": lib}
    # reconciliation: every slice is in its destination whole (modulo the counted path rewrites and the pubs)
    for label, piece, dest in (("hook test modules", test_mods.rstrip("\n"), lib), ("hook tests in mod tests", test_fns.rstrip("\n"), lib)):
        if piece not in dest:
            raise SystemExit(f"{label}: did not carry over whole")
    for label, piece in (("find_repo_root", repo_root.rstrip("\n")), ("engine_version_skew", skew.rstrip("\n"))):
        if piece.replace("\nfn ", "\n#[must_use]\npub fn ", 1) not in pl.files[ws]:
            raise SystemExit(f"{label}: did not carry over whole")
    if hooks.count("\n") + recall.count("\n") != moved.count("\n") - 1:
        raise SystemExit("hook block: line count not conserved into the moved text")
    return pl, new_files, {"keel_cli_refs": before, **{f"keel_cli::{k}": v for k, v in sorted(counts.items())}}


def main(argv: list[str]) -> int:
    apply = "--apply" in argv
    if (HOOKS_SRC / "lib.rs").exists():
        print("already applied: members/keel-hooks/src/lib.rs exists")
        return 0
    moves = plan_moves()
    pl, new_files, totals = plan()
    for src, dst, _new, counts, before in moves:
        print(f"{src.relative_to(REPO).as_posix()} -> {dst.relative_to(REPO).as_posix()}: {before} crate:: refs, rewritten {dict(sorted(counts.items()))}")
    for line in pl.log:
        print(line)
    print(f"hook block: {totals['keel_cli_refs']} keel_cli:: refs rewritten: " + ", ".join(f"{k}={v}" for k, v in totals.items() if k != "keel_cli_refs"))
    print(f"reconciliation: {len(moves)} files moved; {len(pl.slices)} slices conserved out of main.rs; {len(PUBLISHED)} fns + {len(PUB_CONSTS)} consts published; {len(EDITS)} anchored edits in {len(pl.files)} staying files; {len(new_files)} files written")
    if not apply:
        print("dry run; --apply to write")
        return 0
    HOOKS_SRC.mkdir(parents=True, exist_ok=True)
    for src, dst, new, _counts, _before in moves:
        dst.parent.mkdir(parents=True, exist_ok=True)
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
