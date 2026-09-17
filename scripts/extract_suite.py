"""extract_suite.py - the sprint 735 transform: the build-and-test tooling becomes member keel-suite.

not-an-instrument: it is a one-shot codemod (the migration skill's gate 1); its reconciliation totals are the
transform checking itself before it writes, not a measure of the project - the release build under
deny(warnings), the receipt tests and the Fresh probe are the sensors the move answers to.

One committed script, dry-run by default (D0479 one extraction per sprint; the migration skill's gate 1).
Five modules move as git renames - suite, touched, verify and hook_binary out of keel-cli/src, contentkey
out of members/keel-guards/src (only touched.rs reads it) - and every `crate::<module>::` path in them is
rewritten per MODULE_MAP to the crate that owns the module now. Nothing else in a moved file changes: the
reconciliation holds each file's text equal to its source modulo the counted rewrites. The member's
Cargo.toml and lib.rs are written; keel-cli/src/lib.rs re-exports the five at their old paths; the
workspace and keel-cli manifests gain the member. The `--no-receipt` predicate descends from the guard
member's receipt.rs to keel-fs so the suite reads the flag without depending on the guards (the DoD's
known-positive: an edit to a view leaves keel-suite Fresh) - the two guard-member edits are the ones a
marked Decision co-commits (process-change guard, D0209 cl.2).

    python scripts/extract_suite.py            # plan + reconciliation, nothing written
    python scripts/extract_suite.py --apply    # the renames, the rewrites, the manifests, the re-exports

Idempotent: a tree where members/keel-suite/src/lib.rs exists is reported as already applied.
"""
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
CLI_SRC = REPO / "keel-cli" / "src"
GUARDS_SRC = REPO / "members" / "keel-guards" / "src"
DST = REPO / "members" / "keel-suite"
DST_SRC = DST / "src"

# (source path, destination file name)
MOVES = [
    (CLI_SRC / "suite.rs", "suite.rs"),
    (CLI_SRC / "touched.rs", "touched.rs"),
    (CLI_SRC / "verify.rs", "verify.rs"),
    (CLI_SRC / "hook_binary.rs", "hook_binary.rs"),
    (GUARDS_SRC / "contentkey.rs", "contentkey.rs"),
]

# `crate::<module>` in a moved file -> the path that resolves from keel-suite. A module the member
# owns stays `crate::`. `write::write_atomic` is keel-fs's fact (keel-write re-exports it); `receipt::forced`
# descends to keel-fs in this sprint under the name the predicate states.
MODULE_MAP = {
    "write::write_atomic": "keel_fs::fsx::write_atomic",
    "receipt::forced": "keel_fs::fsx::no_receipt_forced",
    "gitx": "keel_git::gitx",
    "eol": "keel_git::eol",
    "device": "keel_actor::device",
    "corpus": "keel_model::corpus",
    "ident": "keel_model::ident",
}
OWN = {"suite", "touched", "verify", "hook_binary", "contentkey"}

REF = re.compile(r"\bcrate::([A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)?)")

CARGO = '''[package]
name = "keel-suite"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "The build-and-test tooling: the full suite and its receipt, the touched set (D0421/D0474) with its content keys, the pre-commit ladder (D0476) and the hook-binary refresh."

[dependencies]
keel-fs = { path = "../keel-fs" }
keel-git = { path = "../keel-git" }
keel-actor = { path = "../keel-actor" }
keel-model = { path = "../keel-model" }
keel-perf = { path = "../keel-perf" }
sha2 = "0.10"
toml = "0.8"
'''

LIB = '''//! keel-suite: the build-and-test tooling - the full suite, the touched set, the pre-commit ladder and the
//! hook-binary refresh, each writing the receipt its command is judged by.
//!
//! The fifth D0479 extraction (sprint 735): `suite`, `touched`, `verify` and `hook_binary` out of keel-cli
//! and `contentkey` out of the guard member, moved by `scripts/extract_suite.py`. keel-cli re-exports every
//! module at its old path, so no caller moved. The crate depends on the leaves and the read model only -
//! not on the views or the guards - so an edit to either leaves it Fresh and the ladder binary stable.
#![forbid(unsafe_code)]
#![deny(warnings, clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, clippy::todo, clippy::unimplemented)]
#![allow(clippy::implicit_hasher, clippy::too_long_first_doc_paragraph, clippy::module_name_repetitions)]
// Tests may use unwrap/expect/panic/indexing/asserts freely.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing))]

pub mod contentkey;
pub mod hook_binary;
pub mod suite;
pub mod touched;
pub mod verify;
'''

FORCED_DOC = '''
/// `--no-receipt` on the command line, or `KEEL_NO_RECEIPT=1` in the environment.
///
/// Hooks and CI have no argv of their own, hence the variable. A caller that reads `true` runs its check
/// and writes no receipt. Out of the guard member's `receipt.rs` in sprint 735 (D0479) so the
/// build-and-test tooling reads the flag without depending on the guards; `keel_guards::receipt::forced`
/// re-exports it under the name its callers use.
#[must_use]
pub fn no_receipt_forced(args: &[String]) -> bool {
    args.iter().any(|a| a == "--no-receipt") || std::env::var("KEEL_NO_RECEIPT").is_ok_and(|v| v == "1")
}
'''

RECEIPT_OLD = '''/// `--no-receipt` on the command line, or `KEEL_NO_RECEIPT=1` in the environment (hooks and CI have no
/// argv of their own).
#[must_use]
pub fn forced(args: &[String]) -> bool {
    args.iter().any(|a| a == "--no-receipt") || std::env::var("KEEL_NO_RECEIPT").is_ok_and(|v| v == "1")
}
'''
RECEIPT_NEW = '''// The `--no-receipt` predicate descended to keel-fs (sprint 735, D0479) so the suite member reads it
// without depending on the guards; re-exported so `receipt::forced` keeps resolving for every caller.
pub use keel_fs::fsx::no_receipt_forced as forced;
'''


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
        new, counts = rewrite(text)
        before = len(REF.findall(text))
        after_foreign = sum(counts.values())
        after_own = len(REF.findall(new))
        # reconciliation: every crate:: reference is either rewritten (counted) or kept (own module)
        if before != after_foreign + after_own:
            raise SystemExit(f"{src.name}: {before} crate:: refs before, {after_foreign} rewritten + {after_own} kept")
        # content: the only difference is the counted substitutions - the length moves by exactly their sum
        # (a moved file may already name `keel_git::gitx` itself, so reversing the map is not the check)
        delta = sum(n * (len(MODULE_MAP[k]) - len("crate::" + k)) for k, n in counts.items())
        if len(new) - len(text) != delta or (not counts and new != text):
            raise SystemExit(f"{src.name}: the rewrite is not the only difference")
        out.append((src, DST_SRC / name, new, counts, before))
    return out


def edit_manifests_and_reexports() -> list[str]:
    done = []
    ws = REPO / "Cargo.toml"
    t = read(ws)
    if '"members/keel-suite"' not in t:
        t = t.replace('    "members/keel-guards",\n    "keel-cli",', '    "members/keel-guards",\n    "members/keel-suite",\n    "keel-cli",')
        assert '"members/keel-suite"' in t, "workspace members anchor"
        write(ws, t)
        done.append("Cargo.toml: members/keel-suite")
    cli = REPO / "keel-cli" / "Cargo.toml"
    t = read(cli)
    if "keel-suite" not in t:
        anchor = 'keel-guards = { path = "../members/keel-guards" }\n'
        assert t.count(anchor) == 1, "keel-cli manifest anchor"
        t = t.replace(anchor, anchor + '# D0479 build-and-test tooling (sprint 735): suite, touched, verify, hook_binary, contentkey; re-exported under their old module paths in lib.rs.\nkeel-suite = { path = "../members/keel-suite" }\n')
        write(cli, t)
        done.append("keel-cli/Cargo.toml: keel-suite")
    lib = CLI_SRC / "lib.rs"
    t = read(lib)
    if "keel_suite" not in t:
        old = "pub mod suite;\npub mod touched;\npub mod verify;\n"
        assert t.count(old) == 1, "lib.rs suite/touched/verify anchor"
        t = t.replace(old, "// The build-and-test tooling is member keel-suite (D0479, sprint 735); `crate::suite::` etc. keep resolving.\npub use keel_suite::suite;\npub use keel_suite::touched;\npub use keel_suite::verify;\n")
        for old, new in (("pub mod hook_binary;\n", "pub use keel_suite::hook_binary;\n"), ("pub use keel_guards::contentkey;\n", "pub use keel_suite::contentkey;\n")):
            assert t.count(old) == 1, old
            t = t.replace(old, new)
        write(lib, t)
        done.append("keel-cli/src/lib.rs: five re-exports")
    glib = GUARDS_SRC / "lib.rs"
    t = read(glib)
    if "pub mod contentkey;\n" in t:
        write(glib, t.replace("pub mod contentkey;\n", ""))
        done.append("keel-guards/src/lib.rs: contentkey left")
    gcargo = REPO / "members" / "keel-guards" / "Cargo.toml"
    t = read(gcargo)
    if "the content key, " in t:
        write(gcargo, t.replace("the content key, ", ""))
        done.append("keel-guards/Cargo.toml: description")
    receipt = GUARDS_SRC / "receipt.rs"
    t = read(receipt)
    if RECEIPT_OLD in t:
        write(receipt, t.replace(RECEIPT_OLD, RECEIPT_NEW))
        done.append("keel-guards/src/receipt.rs: forced re-exported from keel-fs")
    fsx = REPO / "members" / "keel-fs" / "src" / "fsx.rs"
    t = read(fsx)
    if "pub fn no_receipt_forced" not in t:
        anchor = "#[cfg(test)]\nmod tests {"
        assert t.count(anchor) == 1, "fsx.rs test module anchor"
        write(fsx, t.replace(anchor, FORCED_DOC.strip("\n") + "\n\n" + anchor))
        done.append("keel-fs/src/fsx.rs: no_receipt_forced")
    return done


def main(argv: list[str]) -> int:
    apply = "--apply" in argv
    if (DST_SRC / "lib.rs").exists():
        print("already applied: members/keel-suite/src/lib.rs exists")
        return 0
    moves = plan()
    total_refs = 0
    for src, dst, _new, counts, before in moves:
        total_refs += before
        print(f"{src.relative_to(REPO).as_posix()} -> {dst.relative_to(REPO).as_posix()}: {before} crate:: refs, rewritten {dict(sorted(counts.items()))}")
    print(f"reconciliation: {len(moves)} files, {total_refs} crate:: refs each kept or rewritten; text equal modulo the rewrites")
    if not apply:
        print("dry run; --apply to write")
        return 0
    DST_SRC.mkdir(parents=True, exist_ok=True)
    for src, dst, new, _counts, _before in moves:
        subprocess.run(["git", "mv", str(src), str(dst)], cwd=REPO, check=True)
        write(dst, new)
    write(DST / "Cargo.toml", CARGO)
    write(DST_SRC / "lib.rs", LIB)
    for line in edit_manifests_and_reexports():
        print("edited", line)
    print("applied")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
