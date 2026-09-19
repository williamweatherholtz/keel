"""extract_staying_modules.py - the sprint 748 transform: the four modules beside main.rs move to the members that own them (D0513).

not-an-instrument: it is a one-shot codemod (the migration skill's gate 1); its reconciliation totals are the
transform checking itself before it writes, not a measure of the project - the release build under
deny(warnings), the moved unit tests in their new crates, the enforcement-surface lock test and the
byte-identical --help are the sensors the move answers to.

One committed script, dry-run by default (D0479 one extraction per sprint; the migration skill's gate 1).
Four modules move as git renames out of keel-cli/src: adherence.rs and history.rs (the two audits that
re-derive the gate verdict from the git tree by running the guards) become members/keel-guards/src files,
so they sit under the directory the enforcement surface locks by prefix (GUARD_SOURCE_DIRS, D0504);
cursor.rs (keel advance) and enroll.rs (keel enroll) become members/keel-process/src files. Every
`crate::<module>::` path in a moved file is rewritten per the destination's MODULE_MAP to the crate that
owns the module now; a path the destination itself owns (`crate::guards::` inside keel-guards) becomes
`crate::`. Nothing else in a moved file changes: whole files move, so there is no slice and no slice
start to hold clear of a doc line or attribute (issue601) - the plan prints that count as 0.

The enforcement surface follows the file (D0513): the `keel-cli/src/adherence.rs` entry leaves
GUARD_SOURCE_FILES (its file is under GUARD_SOURCE_DIRS by construction, so nothing is unlocked) and the
lock test asserts the two new paths locked and the old path not. keel-cli's lib.rs re-exports the four at
their old paths; each member's lib.rs declares its two new modules; the code registry's element for
history.rs (claudeFable5's, so edited in place per D0108) follows the file.

    python scripts/extract_staying_modules.py            # plan + reconciliation, nothing written
    python scripts/extract_staying_modules.py --apply    # the renames, the rewrites, the lock, the re-exports

Idempotent: a tree where members/keel-guards/src/adherence.rs exists is reported as already applied.
"""
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
CLI_SRC = REPO / "keel-cli" / "src"
GUARDS_SRC = REPO / "members" / "keel-guards" / "src"
PROCESS_SRC = REPO / "members" / "keel-process" / "src"

# (source path, destination directory, destination crate name)
MOVES = [
    (CLI_SRC / "adherence.rs", GUARDS_SRC, "keel_guards"),
    (CLI_SRC / "history.rs", GUARDS_SRC, "keel_guards"),
    (CLI_SRC / "cursor.rs", PROCESS_SRC, "keel_process"),
    (CLI_SRC / "enroll.rs", PROCESS_SRC, "keel_process"),
]

# `crate::<module>` in a moved file -> the path that resolves from the destination crate. `guards` is the
# guards member itself, so inside keel-guards it is `crate`; `workspace::discover` is keel-git's since
# sprint 740 (keel_process::workspace re-exports it, but keel-guards sits below keel-process).
MODULE_MAP: dict[str, dict[str, str]] = {
    "keel_guards": {
        "gitx": "keel_git::gitx",
        "workspace": "keel_git::projects",
        "guards": "crate",
        "validate_root": "keel_model::validate::validate_root",
    },
    "keel_process": {
        "collect_sysml": "keel_model::corpus::collect_sysml",
        "corpus": "keel_model::corpus",
        "guards": "keel_guards",
        "orient": "keel_model::orient",
        "textscan": "keel_model::textscan",
        "view": "keel_view::view",
        "actor": "keel_actor::actor",
        "validate_root": "keel_model::validate::validate_root",
    },
}

REF = re.compile(r"\bcrate::([A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)?)")

# Anchored edits outside the moved files; each old text occurs exactly once in its file.
LOCK_OLD = (
    "// guard_names.rs holds GUARD_NAMES (sprint 732): removing a name there disarms a guard as surely as\n"
    "// deleting its arm here would, so the list is locked with the source that dispatches it.\n"
    'pub(crate) const GUARD_SOURCE_FILES: &[&str] = &["keel-cli/src/adherence.rs", "members/keel-schema/src/guard_names.rs"];\n'
)
LOCK_NEW = (
    "// guard_names.rs holds GUARD_NAMES (sprint 732): removing a name there disarms a guard as surely as\n"
    "// deleting its arm here would, so the list is locked with the source that dispatches it. The\n"
    "// audit-adherence gate (adherence.rs) and the audit-history re-derivation (history.rs) are this member's\n"
    "// since sprint 748 (D0513), locked by GUARD_SOURCE_DIRS below; the file entry that named the audit in\n"
    "// keel-cli/src is retired here because that path no longer exists.\n"
    'pub(crate) const GUARD_SOURCE_FILES: &[&str] = &["members/keel-schema/src/guard_names.rs"];\n'
)
LOCK_TEST_OLD = (
    '        assert!(is_enforcement_surface("members/keel-guards/src/receipt.rs"));\n'
    '        assert!(is_enforcement_surface("keel-cli/src/adherence.rs"));\n'
    '        assert!(is_enforcement_surface("members/keel-schema/src/guard_names.rs"));\n'
    "        // NOT locked: ordinary source, docs, a workflow-shaped path outside the dir, the old path.\n"
    '        assert!(!is_enforcement_surface("keel-cli/src/main.rs"));\n'
    '        assert!(!is_enforcement_surface("keel-cli/src/guards.rs"));\n'
)
LOCK_TEST_NEW = (
    '        assert!(is_enforcement_surface("members/keel-guards/src/receipt.rs"));\n'
    "        // The two audits are locked by the directory they entered in sprint 748 (D0513), not by name.\n"
    '        assert!(is_enforcement_surface("members/keel-guards/src/adherence.rs"));\n'
    '        assert!(is_enforcement_surface("members/keel-guards/src/history.rs"));\n'
    '        assert!(is_enforcement_surface("members/keel-schema/src/guard_names.rs"));\n'
    "        // NOT locked: ordinary source, docs, a workflow-shaped path outside the dir, the old paths.\n"
    '        assert!(!is_enforcement_surface("keel-cli/src/main.rs"));\n'
    '        assert!(!is_enforcement_surface("keel-cli/src/guards.rs"));\n'
    '        assert!(!is_enforcement_surface("keel-cli/src/adherence.rs"));\n'
    '        assert!(!is_enforcement_surface("keel-cli/src/history.rs"));\n'
)
# keel-cli/src/lib.rs: `pub mod X;` -> the re-export, in the order the four lines sit today.
CLI_REEXPORTS = [
    ("pub mod history;\n", "// The two audits are the guards member's and the cursor and enrollment the process member's (D0513, sprint 748); the old paths keep resolving.\npub use keel_guards::history;\n"),
    ("pub mod adherence;\n", "pub use keel_guards::adherence;\n"),
    ("pub mod cursor;\n", "pub use keel_process::cursor;\n"),
    ("pub mod enroll;\n", "pub use keel_process::enroll;\n"),
]
GUARDS_LIB_OLD = "pub mod hardening;\npub mod plan_cover;\npub mod receipt;\n"
GUARDS_LIB_NEW = (
    "// The two audits that re-derive the gate verdict from the git tree (sprint 748, D0513): keel audit adherence\n"
    "// and keel audit history. Under the enforcement lock by the directory they live in.\n"
    "pub mod adherence;\npub mod hardening;\npub mod history;\npub mod plan_cover;\npub mod receipt;\n"
)
PROCESS_LIB_OLD = "pub mod adoption_check;\npub mod currency;\npub mod library;\n"
PROCESS_LIB_NEW = "pub mod adoption_check;\npub mod currency;\n// keel advance and keel enroll (sprint 748, D0513): the process cursor and the actor-enrollment process.\npub mod cursor;\npub mod enroll;\npub mod library;\n"
REGISTRY_OLD = '        :>> filePath = "keel-cli/src/history.rs";\n'
REGISTRY_NEW = '        :>> filePath = "members/keel-guards/src/history.rs";\n'
GUARDS_DOC_OLD = "the runner, the nine family files, the receipt and its content key; `keel-cli/src/adherence.rs`; and since D0503"
GUARDS_DOC_NEW = "the runner, the nine family files, the receipt and its content key, and since D0513 the two audits `adherence.rs` and `history.rs`; and since D0503"

EDITS: list[tuple[Path, str, str]] = [
    (GUARDS_SRC / "enforcement.rs", LOCK_OLD, LOCK_NEW),
    (GUARDS_SRC / "lib.rs", LOCK_TEST_OLD, LOCK_TEST_NEW),
    (GUARDS_SRC / "lib.rs", GUARDS_LIB_OLD, GUARDS_LIB_NEW),
    (PROCESS_SRC / "lib.rs", PROCESS_LIB_OLD, PROCESS_LIB_NEW),
    (REPO / ".tracking" / "architecture" / "code-registry.sysml", REGISTRY_OLD, REGISTRY_NEW),
    (REPO / ".engine" / "docs" / "guards.md", GUARDS_DOC_OLD, GUARDS_DOC_NEW),
] + [(CLI_SRC / "lib.rs", old, new) for old, new in CLI_REEXPORTS]


def read(p: Path) -> str:
    return p.read_text(encoding="utf-8")


def write(p: Path, text: str) -> None:
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(text, encoding="utf-8", newline="\n")


def rewrite(text: str, table: dict[str, str]) -> tuple[str, dict[str, int]]:
    """Every `crate::X[::Y]` per the destination's table; the counts per key are the reconciliation."""
    counts: dict[str, int] = {}

    def sub(m: re.Match[str]) -> str:
        path = m.group(1)
        head = path.split("::")[0]
        if path in table:
            counts[path] = counts.get(path, 0) + 1
            return table[path]
        if head in table:
            counts[head] = counts.get(head, 0) + 1
            return table[head] + path[len(head):]
        raise SystemExit(f"unmapped crate path in a moved file: crate::{path}")

    return REF.sub(sub, text), counts


def plan() -> list[tuple[Path, Path, str, dict[str, int], int]]:
    out = []
    for src, dst_dir, crate in MOVES:
        text = read(src)
        table = MODULE_MAP[crate]
        new, counts = rewrite(text, table)
        before = len(REF.findall(text))
        rewritten = sum(counts.values())
        # reconciliation: every crate:: reference is rewritten and counted (a moved file owns nothing in
        # its destination, so none is kept as-is)
        if before != rewritten:
            raise SystemExit(f"{src.name}: {before} crate:: refs before, {rewritten} rewritten")
        # content: the only difference is the counted substitutions - the length moves by exactly their sum
        delta = sum(n * (len(table[k]) - len("crate::" + k)) for k, n in counts.items())
        if len(new) - len(text) != delta or (not counts and new != text):
            raise SystemExit(f"{src.name}: the rewrite is not the only difference")
        out.append((src, dst_dir / src.name, new, counts, before))
    return out


def plan_edits() -> list[tuple[Path, str]]:
    """Each anchored edit applied to the file's current text; every anchor occurs exactly once."""
    texts: dict[Path, str] = {}
    for path, old, new in EDITS:
        t = texts.get(path) or read(path)
        if t.count(old) != 1:
            raise SystemExit(f"{path.relative_to(REPO).as_posix()}: anchor occurs {t.count(old)} times: {old[:60]!r}")
        texts[path] = t.replace(old, new)
    return list(texts.items())


def main(argv: list[str]) -> int:
    apply = "--apply" in argv
    if (GUARDS_SRC / "adherence.rs").exists():
        print("already applied: members/keel-guards/src/adherence.rs exists")
        return 0
    moves = plan()
    edits = plan_edits()
    total_refs = 0
    for src, dst, _new, counts, before in moves:
        total_refs += before
        print(f"{src.relative_to(REPO).as_posix()} -> {dst.relative_to(REPO).as_posix()}: {before} crate:: refs, rewritten {dict(sorted(counts.items()))}")
    for path, _ in edits:
        n = sum(1 for p, _o, _n in EDITS if p == path)
        print(f"edit {path.relative_to(REPO).as_posix()}: {n} anchored edit(s)")
    print(f"reconciliation: {len(moves)} files, {total_refs} crate:: refs each rewritten; text equal modulo the rewrites; slice starts checked against a preceding doc line or attribute: 0 (whole files move); {len(EDITS)} anchored edits in {len(edits)} files")
    if not apply:
        print("dry run; --apply to write")
        return 0
    for src, dst, new, _counts, _before in moves:
        subprocess.run(["git", "mv", str(src), str(dst)], cwd=REPO, check=True)
        write(dst, new)
    for path, text in edits:
        write(path, text)
        print("edited", path.relative_to(REPO).as_posix())
    print("applied")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
