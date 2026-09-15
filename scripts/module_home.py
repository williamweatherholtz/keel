#!/usr/bin/env python3
# ci-probe: --probe
"""One answer to "where does Rust module X live?" across the workspace.

Sprint 718 moved twenty modules out of keel-cli/src into members/; four scripts each carried its own
`keel-cli/src/<module>.rs` anchor, and the one that opened reverify.rs turned CI red at d110d0a
(issue559). Every D0479 extraction moves more. This module reads the crate list from Cargo.toml's
`[workspace] members` - the one list (D0486) - so a new member needs no edit here; a module found in
two crates is REFUSED, never guessed; a module found nowhere is None, so the caller decides.

    sys.path.insert(0, "<repo>/scripts"); from module_home import module_home, rust_sources
    module_home("reverify")   -> <repo>/members/keel-write/src/reverify.rs
    module_home("view/mod")   -> <repo>/keel-cli/src/view/mod.rs
    module_home("view")       -> <repo>/keel-cli/src/view/mod.rs   (a directory module)
    module_home("no_such")    -> None
    rust_sources()            -> every .rs under every crate's src/ (the census walkers' corpus)

    python scripts/module_home.py reverify guards        # print each home, or MISSING
    python scripts/module_home.py --probe                # the known cases + the anchor scan; exit 1 on any failure

The probe also scans scripts/**/*.py and .engine/tools/*.py for a path EXPRESSION that anchors a .rs
file under keel-cli/src (`join(..., "keel-cli", "src", "x.rs")` or `/ "keel-cli/src/x.rs"`) - the
shape that broke - so the next such anchor fails in CI before the next move breaks it (D0047).
"""
from __future__ import annotations

import glob
import os
import re
import sys
import tempfile
import tomllib

# The first ancestor of this file that holds .git - never the cwd (issue557's class, Python side).
def repo_root(start: str | None = None) -> str:
    p = os.path.abspath(start or __file__)
    while True:
        if os.path.isdir(os.path.join(p, ".git")) or os.path.isfile(os.path.join(p, ".git")):
            return p
        parent = os.path.dirname(p)
        if parent == p:
            sys.exit("module_home: no .git ancestor - run from inside the repository")
        p = parent


class AmbiguousModule(Exception):
    """The same module path exists in more than one crate; naming one would be a guess."""


def crate_dirs(root: str | None = None) -> list[str]:
    """Every workspace member directory as Cargo.toml [workspace] members lists it, globs expanded, in manifest order."""
    root = root or repo_root()
    with open(os.path.join(root, "Cargo.toml"), "rb") as f:
        manifest = tomllib.load(f)
    out = []
    for m in manifest.get("workspace", {}).get("members", []):
        hits = sorted(glob.glob(os.path.join(root, m))) if any(c in m for c in "*?[") else [os.path.join(root, m)]
        out.extend(os.path.normpath(h) for h in hits if os.path.isfile(os.path.join(h, "Cargo.toml")))
    return out


def src_dirs(root: str | None = None) -> list[str]:
    return [os.path.join(c, "src") for c in crate_dirs(root) if os.path.isdir(os.path.join(c, "src"))]


def module_home(name: str, root: str | None = None) -> str | None:
    """The file for module `name` ("guards", "view/mod", "view"): `<src>/<name>.rs`, else `<src>/<name>/mod.rs`.

    Raises AmbiguousModule when two crates both hold it; returns None when none does.
    """
    parts = name.split("/")
    hits = []
    for src in src_dirs(root):
        for cand in (os.path.join(src, *parts) + ".rs", os.path.join(src, *parts, "mod.rs")):
            if os.path.isfile(cand):
                hits.append(cand)
                break
    if len(hits) > 1:
        raise AmbiguousModule(f"module {name!r} lives in {len(hits)} crates: {hits}")
    return hits[0] if hits else None


def rust_sources(root: str | None = None) -> list[str]:
    """Every .rs under every crate's src/, manifest order then path order."""
    out = []
    for src in src_dirs(root):
        for dirpath, _dirs, files in os.walk(src):
            out.extend(os.path.join(dirpath, f) for f in sorted(files) if f.endswith(".rs"))
    return out


# --- the anchor scan: a path expression naming a .rs file under keel-cli/src -----------------------------------
ANCHOR = re.compile(r'''join\([^)\n]*["']keel-cli["'],\s*["']src["'],\s*["'][\w/]+\.rs["']|/\s*["']keel-cli/src/[\w/]+\.rs["']''')
SCAN_GLOBS = ("scripts/**/*.py", ".engine/tools/*.py")


def anchored_lines(root: str | None = None) -> list[tuple[str, int, str]]:
    root = root or repo_root()
    me = os.path.abspath(__file__)
    out = []
    for pattern in SCAN_GLOBS:
        for path in sorted(glob.glob(os.path.join(root, pattern), recursive=True)):
            if os.path.abspath(path) == me:
                continue
            for i, line in enumerate(open(path, encoding="utf-8", errors="replace"), 1):
                if ANCHOR.search(line):
                    out.append((os.path.relpath(path, root), i, line.rstrip()))
    return out


# --- probe --------------------------------------------------------------------------------------------------------
def probe() -> int:
    fails = 0

    def check(name: str, ok: bool, detail: str) -> None:
        nonlocal fails
        print(f"  {'pass' if ok else 'FAIL'}  {name} - {detail}")
        fails += 0 if ok else 1

    # Known cases chosen before the tree is read (D0388): a fixture workspace with two crates.
    with tempfile.TemporaryDirectory() as tmp:
        os.makedirs(os.path.join(tmp, ".git"))
        for crate in ("a", "members/b"):
            os.makedirs(os.path.join(tmp, crate, "src", "deep"))
            open(os.path.join(tmp, crate, "Cargo.toml"), "w").write(f'[package]\nname = "{crate.split("/")[-1]}"\n')
        open(os.path.join(tmp, "Cargo.toml"), "w").write('[workspace]\nmembers = ["a", "members/*"]\n')
        open(os.path.join(tmp, "a", "src", "only_a.rs"), "w").write("")
        open(os.path.join(tmp, "members", "b", "src", "deep", "mod.rs"), "w").write("")
        for crate in ("a", "members/b"):
            open(os.path.join(tmp, crate, "src", "both.rs"), "w").write("")
        check("known-positive: a module in one crate resolves to that file",
              module_home("only_a", tmp) == os.path.join(tmp, "a", "src", "only_a.rs"), "only_a -> a/src/only_a.rs")
        check("known-positive: a directory module resolves to its mod.rs through a glob member",
              module_home("deep", tmp) == os.path.join(tmp, "members", "b", "src", "deep", "mod.rs"), "deep -> members/b/src/deep/mod.rs")
        check("known-negative: a module in no crate is None", module_home("absent", tmp) is None, "absent -> None")
        try:
            module_home("both", tmp)
            check("known-negative: a module in two crates is refused", False, "no AmbiguousModule raised")
        except AmbiguousModule as e:
            check("known-negative: a module in two crates is refused", "2 crates" in str(e), str(e)[:80])
        check("rust_sources walks every crate's src", len(rust_sources(tmp)) == 4, f"{len(rust_sources(tmp))} files, expected 4")

    # The scan's own pair, chosen before the tree is read: the line that broke, and its replacement.
    broke = 'REVERIFY_RS = os.path.join(ROOT, "keel-cli", "src", "reverify.rs")'
    pathlib_shape = '(REPO / "keel-cli/src/guards.rs").read_text(encoding="utf-8"),'
    fixed = 'REVERIFY_RS = module_home("reverify")'
    prose = 'Every top-level module under keel-cli/src (a file `X.rs` or a directory `X/`) is a node'
    check("known-positive: the os.path.join anchor that broke CI is caught", bool(ANCHOR.search(broke)), broke)
    check("known-positive: the pathlib anchor shape is caught", bool(ANCHOR.search(pathlib_shape)), pathlib_shape)
    check("known-negative: the resolver call is not", not ANCHOR.search(fixed), fixed)
    check("known-negative: prose naming the directory is not", not ANCHOR.search(prose), prose[:60])

    # The real tree.
    root = repo_root()
    for name, tail in (("reverify", os.path.join("members", "keel-write", "src", "reverify.rs")),
                       ("guards", os.path.join("keel-cli", "src", "guards.rs")),
                       ("view", os.path.join("keel-cli", "src", "view", "mod.rs"))):
        home = module_home(name, root)
        check(f"this tree: {name} resolves", home is not None and home.endswith(tail), home or "MISSING")
    hits = anchored_lines(root)
    check("this tree: no script anchors a .rs file under keel-cli/src", not hits,
          "; ".join(f"{p}:{i}" for p, i, _ in hits) or "none")
    print(f"module_home probe: {fails} failure(s)")
    return 1 if fails else 0


def main(argv: list[str]) -> int:
    if argv == ["--probe"]:
        return probe()
    if not argv or argv[0].startswith("-"):
        sys.exit(__doc__)
    rc = 0
    for name in argv:
        try:
            home = module_home(name)
        except AmbiguousModule as e:
            print(f"{name}: REFUSED - {e}")
            rc = 1
            continue
        print(f"{name}: {home or 'MISSING'}")
        rc |= 0 if home else 1
    return rc


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
