"""extract_verbs.py - the sprint 750 transform: every verb body leaves main.rs for the member that owns it (D0479, dcKeelCliIsThinDispatch).

not-an-instrument: it is a one-shot codemod (the migration skill's gate 1); its reconciliation totals are the
transform checking itself before it writes, not a measure of the project - the release build under
deny(warnings), the byte-identical --help, verb_homes --check on the applied tree and the touched test set
are the sensors the move answers to.

One committed script, dry-run by default (D0479 one extraction per sprint; the migration skill's gate 1).
The placement table is `python scripts/verb_homes.py --by-member`, read here through verb_homes.analyse():
each top-level item of keel-cli/src/main.rs goes to its LEAST member - the minimal member whose reach covers
everything the body names - as one `pub` item in `members/<m>/src/<short>_verbs.rs` (`view_verbs.rs` in
keel-view). An item that reaches nothing but std (`any`) goes where its callers went: their one member, or
the nearest member every caller member depends on. An item with two least members is placed by PLACEMENT,
never guessed. `main`, `cmd_show` (the dispatch) and `mod tests` stay: main.rs becomes the header, the
imports the dispatch still uses, `use keel_cli::verbs::*;`, and those three.

Every path in a moved body is rewritten for its destination: `crate::X::` and `keel_cli::X::` through the
binary's lib.rs re-exports (the `view`/`write` wrapper modules resolve per name), the `w::` alias likewise,
`keel_<m>::` inside member m to `crate::`; the bare names main.rs imported (`flag`, `root_arg`, `ENGINE_DIR`,
`Path`) become `use` lines in the member file - only the ones the file's text names, so deny(warnings) holds -
and a call into a verb that landed in another member becomes `use keel_<n>::<n>_verbs::name;`. A
`(+keel-args)` in the table adds that dependency to the member's manifest; an external crate a moved body
names (`serde_json::`, `toml::`) is added the same way, its spec copied from keel-cli's manifest; a body that
reads `env!("KEEL_BUILD_COMMIT")` needs the shared build script, so the manifest gains keel-cli's `build =`
line (as keel-process and keel-guards already carry it). `include_str!` paths are rebased on the new file,
and the one asset under keel-cli/assets moves with its owner as a git rename.

The slice of an item is its head line expanded upward over the contiguous `///`, `#[` and `//` lines that
document it, so no doc comment is stranded on the item before it (issue601: sprints 739 and 740 each
reattached docs by hand). The plan then asserts, for every slice, that the nearest non-blank line above its
start is neither a doc line nor an attribute - the expander stops at a blank line, the check does not, so a
`/// doc` separated from its item by a blank fails the dry run naming that line. The count checked is on
the reconciliation line.

    python scripts/extract_verbs.py                 # plan + reconciliation, nothing written
    python scripts/extract_verbs.py --apply         # the member files, the rewrites, the manifests, main.rs
    python scripts/extract_verbs.py --slice-check F # only the slice / doc check on a main.rs-shaped file F

Idempotent: a tree where members/keel-view/src/view_verbs.rs exists is reported as already applied.
"""
from __future__ import annotations

import os
import re
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parent
sys.path.insert(0, str(HERE))
import verb_homes as vh  # noqa: E402

CLI_DIR = REPO / "keel-cli"
CLI_SRC = CLI_DIR / "src"
STAY = {"main", "cmd_show", "tests"}
# Items with two least members (verb_homes prints both): the home is chosen here, once, with its reason.
PLACEMENT = {
    # reaches keel-args, keel-suite and keel-write; keel-process holds `sync` (keel sync) and the claude
    # surface it re-syncs is the process's adoption surface, so the verb sits beside `keel sync`.
    "cmd_sync_claude": "keel-process",
}
DOC_LINE = re.compile(r"^\s*(///|//!|/\*\*)")
ATTR_LINE = re.compile(r"^\s*#!?\[")
COMMENT_LINE = re.compile(r"^\s*//")
SECTION_RULE = re.compile(r"^// ─")
USE_LINE = re.compile(r"^use .*;$")
CLI_PATH = re.compile(r"\b(?:crate|keel_cli)::([A-Za-z_]\w*)(?:::([A-Za-z_]\w*))?")
MEMBER_PATH = re.compile(r"\b(keel_[a-z_]+)::")
INCLUDE = re.compile(r'(include_str!|include_bytes!)\("([^"]+)"\)')
EXTERNAL_CRATES = ("serde_json", "serde", "toml", "sha2", "getrandom", "include_dir", "thiserror")
HEAD_PUB = re.compile(r"^(pub(?:\([a-z]+\))?\s+)?(fn|struct|enum|const|static|type|trait)\b")


def read(p: Path) -> str:
    return p.read_text(encoding="utf-8")


def write(p: Path, text: str) -> None:
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(text, encoding="utf-8", newline="\n")


def short(member: str) -> str:
    return member.removeprefix("keel-")


def crate_ident(member: str) -> str:
    return member.replace("-", "_")


def refuse(msg: str) -> None:
    raise SystemExit(f"extract_verbs: {msg}")


# ── the binary's lib.rs: every re-export as a path table ─────────────────────────────────────────────
class LibPaths:
    """`crate::A[::B]` (or `keel_cli::A[::B]`) -> the member path that resolves it, read from lib.rs."""

    def __init__(self, lib_text: str):
        self.mods: dict[str, str] = {}      # module or alias name -> `keel_x::mod` / `keel_x`
        self.roots: dict[str, str] = {}     # crate-root item name -> `keel_x::mod::name`
        self.wrappers: dict[str, tuple[dict[str, str], str | None]] = {}  # wrapper -> ({name: path}, star module)
        for m in re.finditer(r"^pub mod ([a-z_]+) \{(.*?)^\}", lib_text, re.S | re.M):
            names: dict[str, str] = {}
            star = None
            for u in re.finditer(r"pub use (keel_[a-z_]+(?:::[a-z_]+)*)::(\{[^}]*\}|\*|[A-Za-z_]\w*);", m.group(2)):
                base, what = u.group(1), u.group(2)
                if what == "*":
                    star = base
                else:
                    for n in what.strip("{}").split(","):
                        n = n.strip()
                        if n:
                            names[n] = f"{base}::{n}"
            self.wrappers[m.group(1)] = (names, star)
        plain = re.sub(r"^pub mod [a-z_]+ \{.*?^\}", "", lib_text, flags=re.S | re.M)
        for m in re.finditer(r"^pub use (keel_[a-z_]+) as ([a-z_]+);", plain, re.M):
            self.mods[m.group(2)] = m.group(1)
        for m in re.finditer(r"^pub use (keel_[a-z_]+(?:::[a-z_]+)*)::(\{[^}]*\}|[A-Za-z_]\w*);", plain, re.M):
            base, what = m.group(1), m.group(2)
            if what.startswith("{"):
                for n in what.strip("{}").split(","):
                    n = n.strip()
                    if n:
                        self.roots[n] = f"{base}::{n}"
            elif base.count("::") == 0:
                self.mods[what] = f"{base}::{what}"      # pub use keel_model::orient;
            else:
                self.roots[what] = f"{base}::{what}"     # pub use keel_write::claude_surface::precommit_hook;

    def resolve(self, first: str, second: str | None) -> tuple[str, bool]:
        """(path, consumed_second): the member path for `crate::first[::second]`."""
        if first in self.wrappers:
            names, star = self.wrappers[first]
            if second is None:
                refuse(f"crate::{first} names a wrapper module without an item")
            if second in names:
                return names[second], True
            if star is None:
                refuse(f"crate::{first}::{second} is not re-exported by the wrapper")
            return f"{star}::{second}", True
        if first in self.mods:
            return (f"{self.mods[first]}::{second}", True) if second else (self.mods[first], False)
        if first in self.roots:
            return (f"{self.roots[first]}::{second}", True) if second else (self.roots[first], False)
        refuse(f"crate::{first} is not re-exported by keel-cli/src/lib.rs")
        return "", False


def localize(path: str, dest: str) -> str:
    """`keel_view::view::x` inside keel-view is `crate::view::x`."""
    own = crate_ident(dest)
    if path == own:
        return "crate"
    return "crate" + path[len(own):] if path.startswith(own + "::") else path


# ── main.rs imports: a small use-tree parser ────────────────────────────────────────────────────────
def use_bindings(line: str) -> list[tuple[str, str]]:
    """`use a::{b, c::{d, e as f}};` -> [(a::b, b), (a::c::d, d), (a::c::e, f)]."""
    body = line.removeprefix("use ").rstrip(";").strip()
    out: list[tuple[str, str]] = []

    def walk(prefix: str, text: str) -> None:
        depth = 0
        part = ""
        parts: list[str] = []
        for ch in text:
            if ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
            if ch == "," and depth == 0:
                parts.append(part)
                part = ""
            else:
                part += ch
        parts.append(part)
        for p in parts:
            p = p.strip()
            if not p:
                continue
            if "{" in p:
                head, inner = p.split("{", 1)
                inner = inner.rsplit("}", 1)[0]
                walk(prefix + head.strip(), inner)
            else:
                alias = None
                if " as " in p:
                    p, alias = (s.strip() for s in p.split(" as "))
                full = prefix + p
                out.append((full, alias or full.rsplit("::", 1)[-1]))

    walk("", body)
    return out


# ── slices ───────────────────────────────────────────────────────────────────────────────────────────
def slice_table(text: str) -> tuple[list[str], dict[str, tuple[int, int]], list[int], int, set[int]]:
    """(lines, name -> [start, end) expanded slice, top-level use line indexes, header end).

    The header is everything before the first top-level `use`. Boundaries are item heads and `use`
    lines; an item's slice runs from its expanded start to the next boundary, and every boundary
    before the first item head belongs to the header.
    """
    lines = text.split("\n")
    heads = [(i, vh.ITEM_HEAD.match(l).group(2)) for i, l in enumerate(lines) if vh.ITEM_HEAD.match(l)]
    uses = [i for i, l in enumerate(lines) if l.startswith("use ")]
    # a `// ──` section rule and its indented continuation lines belong to no item (they are dropped)
    section: set[int] = set()
    for i, l in enumerate(lines):
        if SECTION_RULE.match(l):
            section.add(i)
            j = i + 1
            while j < len(lines) and re.match(r"^//\s{3,}", lines[j]):
                section.add(j)
                j += 1
    # a `//` note directly above a `use` line annotates the import, which is re-derived: dropped too
    for i in uses:
        j = i - 1
        while j >= 0 and COMMENT_LINE.match(lines[j]) and not DOC_LINE.match(lines[j]):
            section.add(j)
            j -= 1
    # a `use` that wraps (`use a::{\n    b,\n};`) covers every line to its `;`; those lines join into one
    for i in uses:
        j = i
        while not lines[j].rstrip().endswith(";"):
            j += 1
            if j >= len(lines) or vh.ITEM_HEAD.match(lines[j]):
                refuse(f"main.rs:{i + 1}: a use with no closing `;`")
            section.add(j)
    if len({n for _, n in heads}) != len(heads):
        refuse("two top-level items share a name")
    starts: dict[str, int] = {}
    for i, name in heads:
        s = i
        while s > 0 and s - 1 not in section and (DOC_LINE.match(lines[s - 1]) or ATTR_LINE.match(lines[s - 1]) or COMMENT_LINE.match(lines[s - 1])):
            s -= 1
        starts[name] = s
    boundaries = sorted(set(starts.values()) | set(uses))
    table: dict[str, tuple[int, int]] = {}
    for name, s in starts.items():
        nxt = [b for b in boundaries if b > s]
        table[name] = (s, nxt[0] if nxt else len(lines))
    header_end = min(boundaries) if boundaries else len(lines)
    return lines, table, uses, header_end, section


def use_text(lines: list[str], i: int) -> str:
    """The `use` statement starting at line `i`, joined into one line through its closing `;`."""
    j = i
    while not lines[j].rstrip().endswith(";"):
        j += 1
    return " ".join(l.strip() for l in lines[i:j + 1])


def check_slices(lines: list[str], table: dict[str, tuple[int, int]], section: set[int] = frozenset()) -> int:
    """Refuse when a doc line or attribute would be stranded at either edge of a slice (issue601); the count checked.

    Above a start: the nearest non-blank line before the slice documents the item and the expander did not
    reach it (a blank between them). At an end: the slice's own last non-blank line is a doc line or
    attribute, so it documents whatever came next in main.rs - a `use` line, in the one case found
    (`write_engine_file`'s doc sat above the `migrate` import, six lines and two items from its fn).
    """
    for name, (s, e) in sorted(table.items(), key=lambda kv: kv[1][0]):
        j = s - 1
        while j >= 0 and not lines[j].strip():
            j -= 1
        if j >= 0 and (DOC_LINE.match(lines[j]) or ATTR_LINE.match(lines[j])):
            refuse(f"main.rs:{j + 1}: the slice for {name} (starting line {s + 1}) would strand this doc line or attribute: {lines[j].strip()[:60]!r}")
        k = e - 1
        while k > s and (not lines[k].strip() or k in section):
            k -= 1
        if DOC_LINE.match(lines[k]) or ATTR_LINE.match(lines[k]):
            refuse(f"main.rs:{k + 1}: the slice for {name} ends in a doc line or attribute that documents nothing in it: {lines[k].strip()[:60]!r}")
    return len(table)


# ── placement ────────────────────────────────────────────────────────────────────────────────────────
def place(ws: vh.Workspace, rows: list[dict], stripped: dict[str, str]) -> dict[str, str]:
    """name -> member for every item that moves; `any` items follow their callers."""
    home: dict[str, str] = {}
    pending: list[str] = []
    for r in rows:
        name = r["name"]
        if name in STAY:
            continue
        least = r["least"]
        if not least:
            refuse(f"{name}: unreachable ({','.join(r['unresolved']) or 'no member covers its reach'})")
        if least == ["any"]:
            pending.append(name)
        elif len(least) == 1:
            if least[0] == ws.bin:
                refuse(f"{name}: only the binary can hold it (verb_homes --check names it)")
            home[name] = least[0]
        else:
            if PLACEMENT.get(name) not in least:
                refuse(f"{name}: least members {least}; PLACEMENT must name one")
            home[name] = PLACEMENT[name]
    names = [r["name"] for r in rows]
    callers: dict[str, set[str]] = {n: set() for n in names}
    for n in names:
        for other in names:
            if other != n and re.search(r"(?<![\w.:])%s\b" % re.escape(n), stripped[other]):
                callers[n].add(other)
    while pending:
        progressed = False
        for name in list(pending):
            cs = callers[name] - {"tests"}
            if not cs:
                refuse(f"{name}: reaches nothing and nothing outside the tests calls it - it has no home")
            if any(c not in home and c not in STAY for c in cs):
                continue
            mems = {home[c] for c in cs if c in home}
            if not mems:
                refuse(f"{name}: called only by the dispatch that stays; place it by hand")
            if len(mems) == 1:
                home[name] = next(iter(mems))
            else:
                cands = [m for m in ws.deps if m != ws.bin and all(m in ws.reach[c] for c in mems)]
                if not cands:
                    refuse(f"{name}: callers in {sorted(mems)} share no member below them all")
                home[name] = max(cands, key=lambda m: (len(ws.reach[m]), m))
            pending.remove(name)
            progressed = True
        if not progressed:
            refuse(f"placement does not converge: {pending}")
    return home


# ── the rewrite of one slice for its destination ─────────────────────────────────────────────────────
class Rewriter:
    def __init__(self, ws: vh.Workspace, lib: LibPaths, pool: dict[str, str], home: dict[str, str]):
        self.ws, self.lib, self.pool, self.home = ws, lib, pool, home
        # a binding whose target is a wrapper module (`use keel_cli::write as w;`) resolves per name
        self.wrapper_alias = {b: full.split("::")[1] for b, full in pool.items()
                              if full.startswith("keel_cli::") and full.count("::") == 1 and full.split("::")[1] in lib.wrappers}
        self.counts: dict[str, int] = defaultdict(int)

    def resolve_binding(self, full: str) -> str:
        """The real path behind a main.rs import target (`keel_cli::args::flag` -> `keel_args::flag`)."""
        if full.startswith("std::"):
            return full
        m = re.match(r"keel_cli::([A-Za-z_]\w*)(?:::([A-Za-z_]\w*))?(.*)", full)
        if not m:
            refuse(f"import {full} is neither std nor keel_cli")
        path, _used = self.lib.resolve(m.group(1), m.group(2))
        if m.group(2) is None:
            return path + m.group(3)
        return path + m.group(3)

    def rewrite(self, text: str, dest: str, own_names: set[str]) -> tuple[str, tuple[set[str], dict[str, set[str]]], set[str], set[str]]:
        """(text, (module uses, {module: names}), member crates named, external crates named) for a slice landing in `dest`."""
        crates: set[str] = set()

        def cli(m: re.Match[str]) -> str:
            path, used = self.lib.resolve(m.group(1), m.group(2))
            self.counts["crate:: paths"] += 1
            out = localize(path, dest)
            if m.group(2) and not used:
                out += "::" + m.group(2)
            return out

        text = CLI_PATH.sub(cli, text)
        for alias, wrapper in self.wrapper_alias.items():
            def wrap(m: re.Match[str], wrapper: str = wrapper) -> str:
                path, _ = self.lib.resolve(wrapper, m.group(1))
                self.counts[f"{alias}:: paths"] += 1
                return localize(path, dest)
            text = re.sub(r"(?<![\w.:])%s::([A-Za-z_]\w*)" % re.escape(alias), wrap, text)

        def member(m: re.Match[str]) -> str:
            ident = m.group(1)
            if ident == crate_ident(dest):
                self.counts["own crate paths"] += 1
                return "crate::"
            mem = self.ws.crate_of(ident)
            if mem:
                crates.add(mem)
            return m.group(0)

        text = MEMBER_PATH.sub(member, text)
        stripped = vh.strip(text)
        uses: dict[str, set[str]] = defaultdict(set)
        module_uses: set[str] = set()
        for binding, full in self.pool.items():
            if binding in self.wrapper_alias:
                continue
            if not re.search(r"(?<![\w.:])%s\b" % re.escape(binding), stripped):
                continue
            path = localize(self.resolve_binding(full), dest)
            if path.startswith("keel_"):
                crates.add(self.ws.crate_of(path.split("::")[0]) or "")
            last = path.rsplit("::", 1)[-1]
            if last == binding:
                uses[path.rsplit("::", 1)[0]].add(last)
            else:
                module_uses.add(f"{path} as {binding}")
        for other, mem in self.home.items():
            if other in own_names or mem == dest:
                continue
            if re.search(r"(?<![\w.:])%s\b" % re.escape(other), stripped):
                uses[f"{crate_ident(mem)}::{short(mem)}_verbs"].add(other)
                crates.add(mem)
        for stay in STAY - {"tests"}:
            if re.search(r"(?<![\w.:])%s\b" % re.escape(stay), stripped):
                refuse(f"a body landing in {dest} names {stay}, which stays in the binary")
        externals = {c for c in EXTERNAL_CRATES if re.search(r"(?<![\w.:])%s::" % c, stripped)}
        crates.discard("")
        return text, (module_uses, uses), crates, externals


def use_lines(module_uses: set[str], uses: dict[str, set[str]]) -> list[str]:
    """One `use` per module, names merged across every slice in the file."""
    out = [f"use {p};" for p in sorted(module_uses)]
    for mod, names in sorted(uses.items()):
        ns = sorted(names)
        out.append(f"use {mod}::{ns[0]};" if len(ns) == 1 else f"use {mod}::{{{', '.join(ns)}}};")
    return out


def make_pub(slice_text: str, head_name: str) -> str:
    out = []
    done = False
    for l in slice_text.split("\n"):
        m = vh.ITEM_HEAD.match(l)
        if not done and m and m.group(2) == head_name:
            hp = HEAD_PUB.match(l)
            if hp and hp.group(1) is None:
                l = "pub " + l
            elif hp and hp.group(1) and hp.group(1).strip() != "pub":
                l = "pub " + l[len(hp.group(1)):]
            done = True
        out.append(l)
    if not done:
        refuse(f"{head_name}: head line not found in its slice")
    return "\n".join(out)


def rebase_includes(text: str, src_dir: Path, dest_dir: Path, asset_moves: dict[Path, Path]) -> tuple[str, int]:
    n = 0

    def sub(m: re.Match[str]) -> str:
        nonlocal n
        target = (src_dir / m.group(2)).resolve()
        if target in asset_moves:
            target = asset_moves[target]
        rel = os.path.relpath(target, dest_dir).replace("\\", "/")
        n += 1
        return f'{m.group(1)}("{rel}")'

    return INCLUDE.sub(sub, text), n


# ── manifests and lib.rs ─────────────────────────────────────────────────────────────────────────────
def add_dependencies(man: str, new: list[str]) -> str:
    """Each `name = spec` line goes at the end of the keel- block of [dependencies]."""
    m = re.search(r"^\[dependencies\]\n", man, re.M)
    if not m:
        refuse("manifest without [dependencies]")
    start = m.end()
    end = man.find("\n[", start)
    end = len(man) if end < 0 else end + 1
    block = man[start:end]
    block_lines = block.split("\n")
    last_keel = max((i for i, l in enumerate(block_lines) if l.startswith("keel-")), default=-1)
    for line in new:
        if line.startswith("keel-"):
            block_lines.insert(last_keel + 1, line)
            last_keel += 1
        else:
            tail = len(block_lines) - 1
            while tail > 0 and not block_lines[tail].strip():
                tail -= 1
            block_lines.insert(tail + 1, line)
    return man[:start] + "\n".join(block_lines) + man[end:]


def add_build_script(man: str, crate_dir: Path) -> str:
    rel = os.path.relpath(CLI_DIR / "build.rs", crate_dir).replace("\\", "/")
    note = ("# A verb body reports the binary's engine build (KEEL_BUILD_COMMIT, sprint 750); the script that bakes\n"
            "# the commit is keel-cli's, shared, not copied (as keel-process, keel-guards and keel-hooks).\n"
            f'build = "{rel}"\n')
    return man.replace("\n[dependencies]\n", "\n" + note + "\n[dependencies]\n", 1)


def declare_module(lib: str, module: str, note: str) -> str:
    mods = list(re.finditer(r"^pub mod [a-z_]+;\n", lib, re.M))
    if mods:
        at = mods[-1].end()
        return lib[:at] + f"{note}pub mod {module};\n" + lib[at:]
    attrs = list(re.finditer(r"^#!\[.*\]\n", lib, re.M))
    if not attrs:
        refuse("lib.rs with neither pub mod lines nor inner attributes")
    at = attrs[-1].end()
    return lib[:at] + f"\n{note}pub mod {module};\n" + lib[at:]


# ── the plan ─────────────────────────────────────────────────────────────────────────────────────────
def plan(root: Path):
    ws, rows = vh.analyse(str(root))
    main_path = Path(ws.dirs[ws.bin]) / "src" / "main.rs"
    lib_path = Path(ws.dirs[ws.bin]) / "src" / "lib.rs"
    text = read(main_path)
    lines, table, uses, header_end, section = slice_table(text)
    checked = check_slices(lines, table, section)
    its, order = vh.items(text)
    if set(its) != set(table):
        refuse(f"item tables differ: {sorted(set(its) ^ set(table))}")
    stripped = {n: its[n][2] for n in order}
    home = place(ws, rows, stripped)
    lib = LibPaths(read(lib_path))
    pool: dict[str, str] = {}
    for i in uses:
        for full, binding in use_bindings(use_text(lines, i)):
            if binding in pool:
                refuse(f"main.rs imports {binding} twice")
            pool[binding] = full
    rw = Rewriter(ws, lib, pool, home)

    # accounted lines: header, use lines, slices; whatever is left must be blank or a section rule
    covered = [False] * len(lines)
    for i in range(header_end):
        covered[i] = True
    for i in uses:
        covered[i] = True
    for s, e in table.values():
        for i in range(s, e):
            covered[i] = True
    orphans = [i for i, c in enumerate(covered) if not c and lines[i].strip() and i not in section]
    if orphans:
        refuse("lines in no slice: " + ", ".join(f"main.rs:{i + 1} {lines[i].strip()[:40]!r}" for i in orphans[:5]))

    asset_moves: dict[Path, Path] = {}
    for name, (s, e) in table.items():
        for m in INCLUDE.finditer("\n".join(lines[s:e])):
            target = (main_path.parent / m.group(2)).resolve()
            if target.is_relative_to(CLI_DIR / "assets") and name in home:
                asset_moves[target] = Path(ws.dirs[home[name]]) / "assets" / target.name

    by_member: dict[str, list[str]] = defaultdict(list)
    for name in order:
        if name in home:
            by_member[home[name]].append(name)

    files: dict[Path, str] = {}
    manifests: dict[Path, str] = {}
    libs: dict[Path, str] = {}
    dep_adds: dict[str, list[str]] = {}
    build_adds: list[str] = []
    include_rewrites = 0
    moved_lines = 0
    cli_man = read(CLI_DIR / "Cargo.toml")
    for mem, names in sorted(by_member.items()):
        dest_dir = Path(ws.dirs[mem]) / "src"
        own = set(names)
        bodies: list[str] = []
        module_uses: set[str] = set()
        named_uses: dict[str, set[str]] = defaultdict(set)
        crates: set[str] = set()
        externals: set[str] = set()
        needs_build = False
        for name in names:
            s, e = table[name]
            raw = "\n".join(lines[s:e]).rstrip("\n")
            moved_lines += e - s
            new, us, cs, ex = rw.rewrite(raw, mem, own)
            new, n_inc = rebase_includes(new, main_path.parent, dest_dir, asset_moves)
            include_rewrites += n_inc
            new = make_pub(new, name)
            if 'env!("KEEL_BUILD_COMMIT")' in new:
                needs_build = True
            bodies.append(new)
            module_uses |= us[0]
            for mod, ns in us[1].items():
                named_uses[mod] |= ns
            crates |= cs
            externals |= ex
        crates.discard(mem)
        adds = []
        for c in sorted(crates):
            # a body names a crate by path only when the crate is a DIRECT dependency: one already in the
            # member's reach is added without widening it, one outside it only when could_add says so
            if c in ws.deps[mem]:
                continue
            if c not in ws.reach[mem] and not ws.could_add(mem, c):
                refuse(f"{mem} would need {c} for {names} and cannot add it without widening its reach")
            adds.append(f'{c} = {{ path = "{os.path.relpath(ws.dirs[c], ws.dirs[mem]).replace(chr(92), "/")}" }}')
        man_path = Path(ws.dirs[mem]) / "Cargo.toml"
        man = read(man_path)
        for ext in sorted(externals):
            if not re.search(r"^%s\s*=" % re.escape(ext), man, re.M):
                spec = re.search(r"^%s\s*=.*$" % re.escape(ext), cli_man, re.M)
                if not spec:
                    refuse(f"{ext} is named by a body landing in {mem} and keel-cli's manifest has no spec for it")
                adds.append(spec.group(0))
        new_man = add_dependencies(man, adds) if adds else man
        if needs_build and "\nbuild = " not in new_man:
            new_man = add_build_script(new_man, Path(ws.dirs[mem]))
            build_adds.append(mem)
        if new_man != man:
            manifests[man_path] = new_man
        if adds:
            dep_adds[mem] = adds
        module = f"{short(mem)}_verbs"
        verbs = ", ".join(f"`{n}`" for n in names if n.startswith("cmd_")) or "helpers only"
        header = (
            f"//! The `keel` verbs this member owns (D0479, sprint 750): {verbs}.\n"
            "//!\n"
            "//! Moved out of `keel-cli/src/main.rs` by `scripts/extract_verbs.py`: the binary is argument dispatch, and a\n"
            "//! verb body lives with the member whose reach it needs (`python scripts/verb_homes.py --by-member`).\n"
            "//! Each `cmd_*` takes the arguments after its verb and returns the process exit code.\n"
            "//\n"
            "// These were the binary's private items, written under the same lint set; only the lints that fire on\n"
            "// PUBLIC visibility are new here, and they ask for ceremony a verb body does not owe: its exit code goes\n"
            "// straight back to `main`, and its `Err` and panics are the ones the body already documents in place.\n"
            "#![allow(clippy::must_use_candidate, clippy::missing_errors_doc, clippy::missing_panics_doc)]\n"
        )
        use_block = "\n".join(use_lines(module_uses, named_uses))
        content = header + "\n" + (use_block + "\n\n" if use_block else "") + "\n\n".join(b.strip("\n") + "\n" for b in bodies)
        files[dest_dir / f"{module}.rs"] = content
        lib_p = dest_dir / "lib.rs"
        libs[lib_p] = declare_module(read(lib_p), module, f"// The verbs this member owns, out of main.rs (D0479, sprint 750).\n")

    # the binary: lib.rs gains the re-export list, main.rs keeps the header, the dispatch, the tests
    cli_lib = read(lib_path)
    if "pub mod verbs" in cli_lib:
        refuse("keel-cli/src/lib.rs already declares verbs")
    reexports = "".join(f"    pub use {crate_ident(m)}::{short(m)}_verbs::*;\n" for m in sorted(by_member))
    cli_lib_new = cli_lib.rstrip("\n") + (
        "\n\n// Every verb body is its member's (D0479, sprint 750): main.rs is argument dispatch, and this is the one\n"
        "// list it reads. A verb's home is `python scripts/verb_homes.py --by-member`; the transform was scripts/extract_verbs.py.\n"
        "pub mod verbs {\n" + reexports + "}\n"
    )
    stay_names = [n for n in order if n in STAY]
    stay_text = {n: "\n".join(lines[table[n][0]:table[n][1]]).rstrip("\n") for n in stay_names}
    dispatch_stripped = "\n".join(stripped[n] for n in stay_names if n != "tests")
    kept: dict[str, set[str]] = defaultdict(set)
    kept_modules: set[str] = set()
    for binding, full in pool.items():
        if re.search(r"(?<![\w.:])%s\b" % re.escape(binding), dispatch_stripped):
            last = full.rsplit("::", 1)[-1]
            if last == binding:
                kept[full.rsplit("::", 1)[0]].add(last)
            else:
                kept_modules.add(f"{full} as {binding}")
    import_lines = [f"use {p};" for p in sorted(kept_modules)]
    for mod, names in sorted(kept.items()):
        ns = sorted(names)
        import_lines.append(f"use {mod}::{ns[0]};" if len(ns) == 1 else f"use {mod}::{{{', '.join(ns)}}};")
    tests = stay_text.get("tests", "")
    if tests:
        def super_use(m: re.Match[str]) -> str:
            out = []
            for n in m.group(1).replace("\n", " ").split(","):
                n = n.strip()
                if not n:
                    continue
                if n in home:
                    out.append(f"    use keel_cli::verbs::{n};")
                elif n in pool:
                    out.append(f"    use {pool[n]};")
                else:
                    refuse(f"tests import super::{n}, which is neither moved nor imported")
            return "\n".join(out)
        tests = re.sub(r"    use super::\{([^}]*)\};", super_use, tests)
        tests = re.sub(r"\bsuper::([A-Za-z_]\w*)", lambda m: f"keel_cli::verbs::{m.group(1)}" if m.group(1) in home else m.group(0), tests)
    header = "\n".join(lines[:header_end]).rstrip("\n")
    dispatched = sorted(n for n in home if re.search(r"(?<![\w.:])%s\b" % re.escape(n), dispatch_stripped))
    verb_use, row = ["use keel_cli::verbs::{"], "   "
    for n in dispatched:
        if len(row) + len(n) + 2 > 100:
            verb_use.append(row.rstrip())
            row = "   "
        row += f" {n},"
    verb_use += [row.rstrip(), "};"]
    parts = [header, "",
             "\n".join(import_lines),
             "// Every verb body is its member's (D0479, sprint 750); lib.rs lists them and this file dispatches.",
             "\n".join(verb_use), ""]
    for n in stay_names:
        parts.append((tests if n == "tests" else stay_text[n]) + "\n")
    main_new = "\n".join(parts).rstrip("\n") + "\n"
    files[main_path] = main_new
    files[lib_path] = cli_lib_new
    if len(home) + len(stay_names) != len(order):
        refuse(f"conservation: {len(home)} moved + {len(stay_names)} stay != {len(order)} items")
    summary = {
        "items": len(order), "moved": len(home), "stay": stay_names, "members": {m: len(v) for m, v in sorted(by_member.items())},
        "moved_lines": moved_lines, "checked": checked, "rewrites": dict(rw.counts), "dep_adds": dep_adds, "build_adds": build_adds,
        "include_rewrites": include_rewrites, "asset_moves": asset_moves, "main_lines": main_new.count("\n"),
    }
    return files, manifests, libs, asset_moves, summary


def main(argv: list[str]) -> int:
    if "--slice-check" in argv:
        path = Path(argv[argv.index("--slice-check") + 1])
        lines, table, _uses, _h, section = slice_table(read(path))
        n = check_slices(lines, table, section)
        print(f"slice check: {n} slice(s), none strands a doc line or attribute")
        return 0
    apply = "--apply" in argv
    if (REPO / "members" / "keel-view" / "src" / "view_verbs.rs").exists():
        print("already applied: members/keel-view/src/view_verbs.rs exists")
        return 0
    files, manifests, libs, asset_moves, s = plan(REPO)
    for m, n in s["members"].items():
        print(f"{m}: {n} item(s) -> members/{m}/src/{short(m)}_verbs.rs" if m != "keel-parser" else f"{m}: {n} item(s) -> keel-parser/src/parser_verbs.rs")
    for m, adds in s["dep_adds"].items():
        print(f"manifest {m}: + {', '.join(a.split(' =')[0] for a in adds)}")
    for m in s["build_adds"]:
        print(f"manifest {m}: + build = keel-cli/build.rs")
    for src, dst in asset_moves.items():
        print(f"asset {src.relative_to(REPO).as_posix()} -> {dst.relative_to(REPO).as_posix()}")
    print(f"reconciliation: {s['items']} items = {s['moved']} moved + {len(s['stay'])} stay ({', '.join(s['stay'])}); "
          f"{s['moved_lines']} lines moved into {len(s['members'])} member files; slice starts checked against a preceding doc line or attribute: {s['checked']}; "
          f"rewrites {s['rewrites']}; include paths rebased: {s['include_rewrites']}; main.rs after: {s['main_lines']} lines; "
          f"{len(manifests)} manifests, {len(libs)} member lib.rs edited")
    if not apply:
        print("dry run; --apply to write")
        return 0
    for src, dst in asset_moves.items():
        dst.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run(["git", "mv", str(src), str(dst)], cwd=REPO, check=True)
    for p, t in list(files.items()) + list(manifests.items()) + list(libs.items()):
        write(p, t)
        print("wrote", p.relative_to(REPO).as_posix())
    print("applied")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
