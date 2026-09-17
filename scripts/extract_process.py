"""extract_process.py - the sprint 736 transform: the governance processes become member keel-process.

not-an-instrument: it is a one-shot codemod (the migration skill's gate 1); its reconciliation totals are the
transform checking itself before it writes, not a measure of the project - the release build under
deny(warnings), the integration tests of advance/activate/status and the Fresh probe are the sensors the
move answers to.

One committed script, dry-run by default (D0479 one extraction per sprint; the migration skill's gate 1).
Eight modules move as git renames out of keel-cli/src - process_cmd, library, adoption_check, migrate,
workspace, sync, status and currency (activation and onboard, the DoD's other two, are keel-model's since
sprint 718) - and every `crate::<module>::` path in them is rewritten per MODULE_MAP to the crate that
owns the module now. Nothing else in a moved file changes, with one counted exception: currency.rs took
its GitHub pull from `crate::github_ingest`, a module that stays in keel-cli until D0480 makes it
keel-issues', so the pull becomes a parameter the binary passes (`main.rs` hands it
`keel_cli::github_ingest::pull_cmd`). The reconciliation holds each file's text equal to its source modulo
the counted rewrites and that one anchored edit.

Two descents make the member possible. The validate authority - `Report`, `check_files`, `validate_root`,
`validate_engine_instances` and their tests - leaves keel-cli/src/lib.rs for members/keel-model/src/validate.rs
(workspace.rs is a caller, and the read model already owns the corpus walk it reads); lib.rs re-exports
the four at their old paths. The member's Cargo.toml and lib.rs are written; keel-cli/src/lib.rs re-exports
the eight at their old paths; the workspace and keel-cli manifests gain the member.

    python scripts/extract_process.py            # plan + reconciliation, nothing written
    python scripts/extract_process.py --apply    # the renames, the rewrites, the descent, the manifests, the re-exports

Idempotent: a tree where members/keel-process/src/lib.rs exists is reported as already applied.
"""
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
CLI_SRC = REPO / "keel-cli" / "src"
MODEL_SRC = REPO / "members" / "keel-model" / "src"
DST = REPO / "members" / "keel-process"
DST_SRC = DST / "src"

# (source path, destination file name)
MOVES = [
    (CLI_SRC / "process_cmd.rs", "process_cmd.rs"),
    (CLI_SRC / "library.rs", "library.rs"),
    (CLI_SRC / "adoption_check.rs", "adoption_check.rs"),
    (CLI_SRC / "migrate.rs", "migrate.rs"),
    (CLI_SRC / "workspace.rs", "workspace.rs"),
    (CLI_SRC / "sync.rs", "sync.rs"),
    (CLI_SRC / "status.rs", "status.rs"),
    (CLI_SRC / "currency.rs", "currency.rs"),
]

# `crate::<module>` in a moved file -> the path that resolves from keel-process. A module the member
# owns stays `crate::`. `validate_root` descends to keel-model in this sprint.
MODULE_MAP = {
    "collect_sysml": "keel_model::corpus::collect_sysml",
    "validate_root": "keel_model::validate::validate_root",
    "gitx": "keel_git::gitx",
    "write": "keel_write::write",
    "cli_surface": "keel_schema::cli_surface",
    "actor": "keel_actor::actor",
    "activation": "keel_model::activation",
    "scaffold": "keel_write::scaffold",
    "ident": "keel_model::ident",
    "guards": "keel_guards",
    "arch": "keel_view::arch",
    "embedded": "keel_schema::embedded",
    "receipt": "keel_guards::receipt",
    "view": "keel_view::view",
    "pin_skew": "keel_write::pin_skew",
    "touched": "keel_suite::touched",
    "orient": "keel_model::orient",
    "gitfacts": "keel_model::gitfacts",
    "claim": "keel_write::claim",
    "claude_surface": "keel_write::claude_surface",
    "hook_binary": "keel_suite::hook_binary",
    "color": "keel_json::color",
}
OWN = {"process_cmd", "library", "adoption_check", "migrate", "workspace", "sync", "status", "currency"}

# Anchored edits applied to a moved file BEFORE the crate:: rewrite; each old text occurs exactly once.
PRE_EDITS: dict[str, list[tuple[str, str]]] = {
    "currency.rs": [
        (
            "/// Run the pass. Prints each stage under its own header, then the summary.\n#[must_use]\npub fn cmd(args: &[String], root: &Path) -> i32 {\n",
            "/// Run the pass. Prints each stage under its own header, then the summary.\n///\n/// `pull` is the GitHub pull the binary hands in (`keel_cli::github_ingest::pull_cmd`): the intake stays\n/// in keel-cli until D0480 makes it keel-issues', and the process member depends on neither (sprint 736).\n#[must_use]\npub fn cmd(args: &[String], root: &Path, pull: &dyn Fn(&[String], &Path) -> i32) -> i32 {\n",
        ),
        ("let code = crate::github_ingest::pull_cmd(&pull_args, root);", "let code = pull(&pull_args, root);"),
    ],
}

REF = re.compile(r"\bcrate::([A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)?)")

CARGO = '''[package]
name = "keel-process"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "The governance processes: the process cursor (advance), activation and adoption, the unit library, the two migrations, the workspace gate, sync/land, status and the unattended currency pass."
# `keel migrate` reports the binary's engine build (migrate.rs: KEEL_BUILD_COMMIT); the script that bakes
# the commit is keel-cli's, shared, not copied (sprint 736, as keel-guards and keel-write).
build = "../../keel-cli/build.rs"

[dependencies]
keel-fs = { path = "../keel-fs" }
keel-git = { path = "../keel-git" }
keel-actor = { path = "../keel-actor" }
keel-json = { path = "../keel-json" }
keel-schema = { path = "../keel-schema" }
keel-model = { path = "../keel-model" }
keel-write = { path = "../keel-write" }
keel-view = { path = "../keel-view" }
keel-guards = { path = "../keel-guards" }
keel-suite = { path = "../keel-suite" }
serde_json = "1"
toml = "0.8"
include_dir = "0.7"
'''

LIB = '''//! keel-process: the governance processes - the process cursor, activation and adoption, the unit library,
//! the migrations, the workspace gate, sync and land, status and the unattended currency pass.
//!
//! The sixth D0479 extraction (sprint 736): `process_cmd`, `library`, `adoption_check`, `migrate`,
//! `workspace`, `sync`, `status` and `currency` out of keel-cli, moved by `scripts/extract_process.py`.
//! keel-cli re-exports every module at its old path, so no caller moved. The crate sits above the guards
//! and the build-and-test tooling because these processes RUN them (`keel gate --workspace`, `keel show
//! status`, the touched set before a land); it depends on nothing that serves or ingests, so an edit to
//! serve.rs or the GitHub intake leaves it Fresh.
#![forbid(unsafe_code)]
#![deny(warnings, clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, clippy::todo, clippy::unimplemented)]
#![allow(clippy::implicit_hasher, clippy::too_long_first_doc_paragraph, clippy::module_name_repetitions)]
// Tests may use unwrap/expect/panic/indexing/asserts freely.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing))]

pub mod adoption_check;
pub mod currency;
pub mod library;
pub mod migrate;
pub mod process_cmd;
pub mod status;
pub mod sync;
pub mod workspace;
'''

VALIDATE_HEAD = '''//! The validate authority: parse-check files, register the schema and semantically validate every
//! `.tracking`/`.knowledge` file (the `validate` gate) and the `.engine` INSTANCE files (`check-engine`).
//!
//! Out of keel-cli/src/lib.rs in sprint 736 (D0479): the workspace gate is a caller and it became a member,
//! and the read model already owns the corpus walk this reads. keel-cli re-exports `Report`, `check_files`,
//! `validate_root` and `validate_engine_instances` at their old paths.

use std::path::{Path, PathBuf};

use keel_parser::ast::Package;
use keel_parser::{Diagnostic, PackageRegistry};

use crate::corpus::{collect_sysml, parse_pkg, CheckError};

'''

# The three slices of keel-cli/src/lib.rs that descend, each bounded by anchors that occur once.
SLICES = [
    ("// ── report types ─", "// ── orient types ─"),
    ("// ── public commands ─", "/// The character length of the longest path under `dir`"),
    ("#[cfg(test)]\nmod engine_instance_tests {", None),
]
LIB_REEXPORT = "// The validate authority is the read model's (sprint 736, D0479); the four keep resolving at the root.\npub use keel_model::validate::{check_files, validate_engine_instances, validate_root, Report};\n"
LIB_IMPORT_OLD = "use std::path::{Path, PathBuf};\n\nuse keel_parser::ast::{ActionDef, Item, Package, Part, Value};\nuse keel_parser::{Diagnostic, PackageRegistry};\n"
LIB_IMPORT_NEW = "use std::path::Path;\n\nuse keel_parser::ast::{ActionDef, Item, Package, Part, Value};\n"
LIB_DOC_OLD = "//! - [`validate_root`]: register schema packages, semantic-validate all `.tracking/` files.\n//! - [`check_files`]: parse-only check for one or more files.\n"
LIB_DOC_NEW = "//! - [`validate_root`] / [`check_files`]: the validate authority, keel-model's since sprint 736, re-exported here.\n"


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
        if path in MODULE_MAP:
            counts[path] = counts.get(path, 0) + 1
            return MODULE_MAP[path]
        if head in MODULE_MAP:
            counts[head] = counts.get(head, 0) + 1
            return MODULE_MAP[head] + path[len(head):]
        raise SystemExit(f"unmapped crate path in a moved file: crate::{path}")

    return REF.sub(sub, text), counts


def plan() -> list[tuple[Path, Path, str, dict[str, int], int]]:
    out = []
    for src, name in MOVES:
        text = read(src)
        pre = text
        pre_delta = 0
        for old, new in PRE_EDITS.get(name, []):
            if pre.count(old) != 1:
                raise SystemExit(f"{name}: pre-edit anchor occurs {pre.count(old)} times")
            pre = pre.replace(old, new)
            pre_delta += len(new) - len(old)
        new, counts = rewrite(pre)
        before = len(REF.findall(pre))
        after_foreign = sum(counts.values())
        after_own = len(REF.findall(new))
        # reconciliation: every crate:: reference is either rewritten (counted) or kept (own module)
        if before != after_foreign + after_own:
            raise SystemExit(f"{src.name}: {before} crate:: refs before, {after_foreign} rewritten + {after_own} kept")
        # content: the only differences are the counted substitutions and the anchored pre-edits - the length
        # moves by exactly their sum (a moved file may already name `keel_fs::` itself, so reversing the map is
        # not the check)
        delta = sum(n * (len(MODULE_MAP[k]) - len("crate::" + k)) for k, n in counts.items()) + pre_delta
        if len(new) - len(text) != delta or (not counts and not pre_delta and new != text):
            raise SystemExit(f"{src.name}: the rewrite is not the only difference")
        out.append((src, DST_SRC / name, new, counts, before))
    return out


def plan_descent() -> tuple[str, str, int]:
    """keel-cli/src/lib.rs minus the three validate slices, and members/keel-model/src/validate.rs = the slices."""
    lib = read(CLI_SRC / "lib.rs")
    remaining = lib
    moved = []
    for start, end in SLICES:
        if remaining.count(start) != 1:
            raise SystemExit(f"lib.rs: slice anchor {start!r} occurs {remaining.count(start)} times")
        i = remaining.index(start)
        if end is None:
            j = len(remaining)
        else:
            if remaining.count(end) != 1:
                raise SystemExit(f"lib.rs: slice end {end!r} occurs {remaining.count(end)} times")
            j = remaining.index(end)
            if j < i:
                raise SystemExit(f"lib.rs: slice {start!r} ends before it starts")
        moved.append(remaining[i:j])
        remaining = remaining[:i] + remaining[j:]
    for old, new in ((LIB_IMPORT_OLD, LIB_IMPORT_NEW), (LIB_DOC_OLD, LIB_DOC_NEW)):
        if remaining.count(old) != 1:
            raise SystemExit(f"lib.rs: anchor occurs {remaining.count(old)} times: {old[:40]!r}")
        remaining = remaining.replace(old, new)
    anchor = "pub use keel_model::corpus::{supersede_edges, supersede_targets};\n"
    if remaining.count(anchor) != 1:
        raise SystemExit("lib.rs: re-export anchor")
    remaining = remaining.replace(anchor, anchor + LIB_REEXPORT)
    # the moved test module used `super::is_engine_instance_file` etc.; in validate.rs `super` is still the module
    validate = VALIDATE_HEAD + "".join(s.rstrip("\n") + "\n\n" for s in moved).rstrip("\n") + "\n"
    # reconciliation: nothing of lib.rs is lost - what left it is in validate.rs, what stayed is unchanged text
    moved_len = sum(len(s) for s in moved)
    edits = (len(LIB_IMPORT_NEW) - len(LIB_IMPORT_OLD)) + (len(LIB_DOC_NEW) - len(LIB_DOC_OLD)) + len(LIB_REEXPORT)
    if len(remaining) != len(lib) - moved_len + edits:
        raise SystemExit("lib.rs: the descent is not the only difference")
    for s in moved:
        if s.strip() not in validate:
            raise SystemExit("validate.rs: a slice did not carry over whole")
    return remaining, validate, moved_len


def edit_manifests_and_reexports() -> list[str]:
    done = []
    ws = REPO / "Cargo.toml"
    t = read(ws)
    if '"members/keel-process"' not in t:
        t = t.replace('    "members/keel-suite",\n    "keel-cli",', '    "members/keel-suite",\n    "members/keel-process",\n    "keel-cli",')
        assert '"members/keel-process"' in t, "workspace members anchor"
        write(ws, t)
        done.append("Cargo.toml: members/keel-process")
    cli = REPO / "keel-cli" / "Cargo.toml"
    t = read(cli)
    if "keel-process" not in t:
        anchor = 'keel-suite = { path = "../members/keel-suite" }\n'
        assert t.count(anchor) == 1, "keel-cli manifest anchor"
        t = t.replace(anchor, anchor + '# D0479 governance processes (sprint 736): process_cmd, library, adoption_check, migrate, workspace, sync, status, currency; re-exported under their old module paths in lib.rs.\nkeel-process = { path = "../members/keel-process" }\n')
        write(cli, t)
        done.append("keel-cli/Cargo.toml: keel-process")
    lib = CLI_SRC / "lib.rs"
    t = read(lib)
    if "keel_process" not in t:
        first = True
        for name in ("currency", "adoption_check", "workspace", "library", "migrate", "process_cmd", "status", "sync"):
            old = f"pub mod {name};\n"
            assert t.count(old) == 1, old
            new = f"pub use keel_process::{name};\n"
            if first:
                new = "// The governance processes are member keel-process (D0479, sprint 736); `crate::workspace::` etc. keep resolving.\n" + new
                first = False
            t = t.replace(old, new)
        write(lib, t)
        done.append("keel-cli/src/lib.rs: eight re-exports")
    main_rs = CLI_SRC / "main.rs"
    t = read(main_rs)
    old = "keel_cli::currency::cmd(rest, &root)\n"
    if old in t:
        assert t.count(old) == 1, "main.rs currency anchor"
        write(main_rs, t.replace(old, "keel_cli::currency::cmd(rest, &root, &keel_cli::github_ingest::pull_cmd)\n"))
        done.append("keel-cli/src/main.rs: currency hands in the GitHub pull")
    # two integration tests read adoption_check.rs by `include_str!`; the path follows the file
    for test in ("adoption_check_can_fail.rs", "tests_bind_to_properties.rs"):
        p = REPO / "keel-cli" / "tests" / test
        t = read(p)
        old = 'include_str!("../src/adoption_check.rs")'
        if old in t:
            assert t.count(old) == 1, test
            write(p, t.replace(old, 'include_str!("../../members/keel-process/src/adoption_check.rs")'))
            done.append(f"keel-cli/tests/{test}: include_str! follows adoption_check.rs")
    mlib = MODEL_SRC / "lib.rs"
    t = read(mlib)
    if "pub mod validate;" not in t:
        anchor = "pub mod onboard;\n"
        assert t.count(anchor) == 1, "keel-model lib.rs anchor"
        write(mlib, t.replace(anchor, anchor + "pub mod validate;\n"))
        done.append("keel-model/src/lib.rs: validate")
    return done


def main(argv: list[str]) -> int:
    apply = "--apply" in argv
    if (DST_SRC / "lib.rs").exists():
        print("already applied: members/keel-process/src/lib.rs exists")
        return 0
    moves = plan()
    lib_new, validate_new, moved_len = plan_descent()
    total_refs = 0
    for src, dst, _new, counts, before in moves:
        total_refs += before
        print(f"{src.relative_to(REPO).as_posix()} -> {dst.relative_to(REPO).as_posix()}: {before} crate:: refs, rewritten {dict(sorted(counts.items()))}")
    print(f"keel-cli/src/lib.rs -> members/keel-model/src/validate.rs: {moved_len} chars in {len(SLICES)} slices; lib.rs keeps the orient API and walk_longest")
    print(f"reconciliation: {len(moves)} files, {total_refs} crate:: refs each kept or rewritten; text equal modulo the rewrites and {sum(len(v) for v in PRE_EDITS.values())} anchored edit(s)")
    if not apply:
        print("dry run; --apply to write")
        return 0
    DST_SRC.mkdir(parents=True, exist_ok=True)
    for src, dst, new, _counts, _before in moves:
        subprocess.run(["git", "mv", str(src), str(dst)], cwd=REPO, check=True)
        write(dst, new)
    write(DST / "Cargo.toml", CARGO)
    write(DST_SRC / "lib.rs", LIB)
    write(CLI_SRC / "lib.rs", lib_new)
    write(MODEL_SRC / "validate.rs", validate_new)
    print("edited keel-cli/src/lib.rs: validate authority descended")
    for line in edit_manifests_and_reexports():
        print("edited", line)
    print("applied")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
