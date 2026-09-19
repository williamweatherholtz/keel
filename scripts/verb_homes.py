#!/usr/bin/env python3
# ci-probe: --probe
"""Where could each top-level item of keel-cli/src/main.rs live? (D0479, dcSharedHelpersDescendBelowTheVerbs)

The member DAG is read from every workspace manifest (Cargo.toml `[workspace] members`, then each
member's `[dependencies]` - dev-dependencies excluded, so a test-only reach never counts). For every
top-level item of the binary's main.rs (fn / const / static / struct / enum / mod / impl), the members
its body REACHES are collected: `crate::X::` and `keel_cli::X::` paths resolved through the binary's
lib.rs re-exports (`pub use keel_git::gitx;`, `pub use keel_hooks as hooks;`, the `view`/`write`
wrapper modules, crate-root names such as `validate_root`), `keel_<m>::` paths directly, bare names
the file imports (`use keel_cli::args::{flag, root_arg};`, `use keel_cli::write as w;`), and - through
the other main.rs items the body calls - everything THOSE reach. A `pub mod X;` the lib.rs declares
locally is the binary's own: reaching it is reaching keel-cli.

The LEAST members of an item are the minimal members of the DAG that could hold it: a member holds a
needed crate when the crate is in its transitive dependencies, or when it could add the crate as a
dependency without widening its reach - every dependency of the crate is already under the member, so
the new edge brings in that crate alone (a helper crate such as keel-args, below every verb, is held
by any member that reaches git; the table says `keel-write(+keel-args)`). Empty reach (std only) is
`any`; a reach no member covers even so (a body that names `keel_serve::` and `keel_hooks::` together,
each carrying dependencies the other lacks, so only the binary reaches both) is the binary's.

    python scripts/verb_homes.py                    # one row per item: line, size, least, direct, via
    python scripts/verb_homes.py --by-member        # items grouped by their least member, largest first
    python scripts/verb_homes.py --check            # exit 1 naming every item whose least is the binary or unreachable
    python scripts/verb_homes.py --probe            # the known-positive / known-negative fixtures first (D0388)
    python scripts/verb_homes.py --root DIR ...     # a fixture workspace instead of this one

`--check` is the control behind the seam sprint: a cmd_ body whose least member is keel-cli needs a
helper only the binary has, and the parent item (dcKeelCliIsThinDispatch) cannot move that body until
the helper descends. `main`, `cmd_show` (the dispatch that stays) and `mod tests` are excluded.
Comments and string literals are stripped before any name is read, so usage text naming `flag` or a
doc line naming `main` is not a call.
"""
from __future__ import annotations

import os
import re
import sys
import tempfile
import tomllib
from collections import defaultdict

# `{name}` or `{name:spec}` in a string literal: an implicit format argument, a reference to `name`
CAPTURE = re.compile(r"(?<!\{)\{([A-Za-z_]\w*)(?=[:}])")
ITEM_HEAD = re.compile(r"^(?:pub(?:\([a-z]+\))?\s+)?(fn|mod|struct|enum|const|static|impl|type|trait)\s+([A-Za-z_][A-Za-z0-9_]*)")
CLI_REF = re.compile(r"\b(?:crate|keel_cli)::([A-Za-z_][A-Za-z0-9_]*)(?:::([A-Za-z_][A-Za-z0-9_]*))?")
MEMBER_REF = re.compile(r"\b(keel_[a-z_]+)::")
EXCLUDED = {"main", "cmd_show", "tests"}


def repo_root(start: str | None = None) -> str:
    p = os.path.abspath(start or __file__)
    while True:
        if os.path.exists(os.path.join(p, ".git")):
            return p
        parent = os.path.dirname(p)
        if parent == p:
            sys.exit("verb_homes: no .git ancestor - run from inside the repository")
        p = parent


def strip(text: str) -> str:
    """Comments and string literals blanked (same width, newlines kept), so names inside them are not read.

    A character walk, not a line regex: a `const` whose string spans lines (`ACTIVATION_HEADER`) hid the
    word `process` from the old line-at-a-time pass, and the transform then imported `std::process`
    into a member that never calls it (sprint 750, deny(unused_imports)). Handles `"..."` with escapes,
    `r#"..."#` raw strings, `b"..."`, char literals (a lifetime `'a` is left alone), `//` and nested `/* */`.
    A format capture inside a string literal (`"{ACTIVATION_HEADER}"`, `"{n:>4}"`) IS a reference to a
    name in scope, so `{ident` survives the blanking of its string.
    """
    out: list[str] = []
    i, n = 0, len(text)

    def blank(j: int, k: int, string: bool = False) -> None:
        seg = text[j:k]
        keep = set()
        if string:
            for m in CAPTURE.finditer(seg):
                keep.update(range(m.start(), m.end()))
        out.append("".join("\n" if c == "\n" else c if p in keep else " " for p, c in enumerate(seg)))

    while i < n:
        c = text[i]
        two = text[i:i + 2]
        if two == "//":
            k = text.find("\n", i)
            k = n if k < 0 else k
            blank(i, k)
            i = k
        elif two == "/*":
            depth, k = 1, i + 2
            while k < n and depth:
                if text[k:k + 2] == "/*":
                    depth, k = depth + 1, k + 2
                elif text[k:k + 2] == "*/":
                    depth, k = depth - 1, k + 2
                else:
                    k += 1
            blank(i, k)
            i = k
        elif c == '"' or (c == "b" and two == 'b"'):
            k = i + (2 if c == "b" else 1)
            while k < n and text[k] != '"':
                k += 2 if text[k] == "\\" else 1
            k = min(k + 1, n)
            out.append(text[i:i + (2 if c == "b" else 1)])
            blank(i + (2 if c == "b" else 1), k, string=True)
            i = k
        elif (c == "r" or two == "br") and re.match(r"b?r#*\"", text[i:i + 10]):
            m = re.match(r"(b?r)(#*)\"", text[i:])
            close = '"' + m.group(2)
            k = text.find(close, i + m.end())
            k = n if k < 0 else k + len(close)
            out.append(text[i:i + m.end()])
            blank(i + m.end(), k, string=True)
            i = k
        elif c == "'" and re.match(r"'(?:\\.[^']*|[^'\\])'", text[i:i + 12]):
            m = re.match(r"'(?:\\.[^']*|[^'\\])'", text[i:i + 12])
            out.append("'" + " " * (m.end() - 2) + "'")
            i += m.end()
        else:
            out.append(c)
            i += 1
    return "".join(out)


class Workspace:
    def __init__(self, root: str):
        self.root = root
        with open(os.path.join(root, "Cargo.toml"), "rb") as f:
            ws = tomllib.load(f)
        self.deps: dict[str, set[str]] = {}
        self.dirs: dict[str, str] = {}
        for m in ws.get("workspace", {}).get("members", []):
            d = os.path.join(root, m)
            with open(os.path.join(d, "Cargo.toml"), "rb") as f:
                man = tomllib.load(f)
            name = man["package"]["name"]
            self.dirs[name] = d
            self.deps[name] = {k for k in man.get("dependencies", {}) if k in self._names(ws)}
        bins = [n for n, d in self.dirs.items() if os.path.isfile(os.path.join(d, "src", "main.rs"))]
        if len(bins) != 1:
            sys.exit(f"verb_homes: expected one member with src/main.rs, found {bins}")
        self.bin = bins[0]
        self.reach: dict[str, set[str]] = {}
        for n in self.deps:
            self.reach[n] = self._closure(n) | {n}

    @staticmethod
    def _names(ws: dict) -> set[str]:
        out = set()
        for m in ws.get("workspace", {}).get("members", []):
            out.add(os.path.basename(m))
        return out

    def _closure(self, n: str, seen: set[str] | None = None) -> set[str]:
        seen = set() if seen is None else seen
        for d in self.deps.get(n, ()):
            if d not in seen:
                seen.add(d)
                self._closure(d, seen)
        return seen

    def could_add(self, member: str, crate: str) -> bool:
        """`member` could depend on `crate` without widening its reach: the crate is not the binary, does not
        itself reach the member (the edge would be a cycle), and every dependency of the crate is already
        under the member, so the edge brings in that crate alone."""
        return crate != self.bin and member not in self.reach[crate] and (self.reach[crate] - {crate}) <= self.reach[member]

    def least(self, needs: set[str]) -> list[str]:
        """The minimal members that could hold `needs` - each need in the member's reach or addable to it
        ([`could_add`]): the members adding the fewest dependencies, and among those the lowest (one
        that another candidate reaches is dropped). The binary reaches everything and adds nothing, so
        it is the answer only when no member could hold the item; [] when nothing could."""
        cands = [m for m in self.deps if m != self.bin and all(c in self.reach[m] or self.could_add(m, c) for c in needs)]
        if not cands:
            return [self.bin] if needs <= self.reach[self.bin] else []
        fewest = min(len(self.adds(m, needs)) for m in cands)
        cands = [m for m in cands if len(self.adds(m, needs)) == fewest]
        return sorted(m for m in cands if not any(o != m and o in self.reach[m] for o in cands))

    def adds(self, member: str, needs: set[str]) -> list[str]:
        """The needs `member` would add as dependencies to hold the item."""
        return sorted(needs - self.reach[member])

    def crate_of(self, ident: str) -> str | None:
        """`keel_git` -> `keel-git` when that is a member."""
        name = ident.replace("_", "-")
        return name if name in self.deps else None


class Resolver:
    """lib.rs re-exports and main.rs imports -> the member a path or bare name reaches."""

    def __init__(self, ws: Workspace):
        self.ws = ws
        src = os.path.join(ws.dirs[ws.bin], "src")
        lib = open(os.path.join(src, "lib.rs"), encoding="utf-8").read()
        self.mod2mem: dict[str, str] = {}          # module or crate-root name -> member
        self.wrapper: dict[str, dict[str, str]] = {}  # wrapper module -> {name -> member, "*" -> member}
        self.unknown: set[str] = set()
        for m in re.finditer(r"^pub mod ([a-z_]+) \{(.*?)^\}", lib, re.S | re.M):
            table: dict[str, str] = {}
            for u in re.finditer(r"pub use (keel_[a-z_]+)::[a-z_]+::(\{[^}]*\}|\*|[A-Za-z_]+);", m.group(2)):
                mem = ws.crate_of(u.group(1))
                if mem is None:
                    continue
                if u.group(2) == "*":
                    table["*"] = mem
                else:
                    for n in u.group(2).strip("{}").split(","):
                        table[n.strip()] = mem
            self.wrapper[m.group(1)] = table
        plain = re.sub(r"^pub mod [a-z_]+ \{.*?^\}", "", lib, flags=re.S | re.M)
        for m in re.finditer(r"^pub mod ([a-z_]+);", plain, re.M):
            self.mod2mem[m.group(1)] = ws.bin
        for m in re.finditer(r"^pub use (keel_[a-z_]+) as ([a-z_]+);", plain, re.M):
            mem = ws.crate_of(m.group(1))
            if mem:
                self.mod2mem[m.group(2)] = mem
        for m in re.finditer(r"^pub use (keel_[a-z_]+)::([a-z_]+);", plain, re.M):
            mem = ws.crate_of(m.group(1))
            if mem:
                self.mod2mem[m.group(2)] = mem
        for m in re.finditer(r"^pub use (keel_[a-z_]+)::[a-z_]+::(\{[^}]*\}|[A-Za-z_]+);", plain, re.M):
            mem = ws.crate_of(m.group(1))
            if mem:
                for n in m.group(2).strip("{}").split(","):
                    self.mod2mem[n.strip()] = mem
        # main.rs imports: `use keel_cli::a::b::{x, y};`, `use keel_cli::a::{x};`, `use keel_cli::x;`, `use keel_cli::a as w;`
        main = open(os.path.join(src, "main.rs"), encoding="utf-8").read()
        self.bare: dict[str, str] = {}   # bare name -> member
        self.alias: dict[str, str] = {}  # alias -> module
        for m in re.finditer(r"^use (?:crate|keel_cli)::(\{[^}]*\});", main, re.M):
            for n in m.group(1).strip("{}").split(","):
                n = n.strip()
                if n and n in self.mod2mem:
                    self.bare[n] = self.mod2mem[n]
        for m in re.finditer(r"^use (?:crate|keel_cli)::([A-Za-z_][A-Za-z0-9_:]*?)(?:::(\{[^}]*\}))?(?: as ([a-z_]+))?;", main, re.M):
            path, names, alias = m.group(1), m.group(2), m.group(3)
            segs = path.split("::")
            if alias:
                self.alias[alias] = segs[0]
                continue
            if names:
                for n in names.strip("{}").split(","):
                    n = n.strip()
                    if n:
                        mem = self.path_member(segs[0], n if len(segs) == 1 else segs[1])
                        if mem:
                            self.bare[n] = mem
            else:
                mem = self.path_member(segs[0], segs[1] if len(segs) > 1 else None)
                if mem:
                    self.bare[segs[-1]] = mem

    def path_member(self, first: str, second: str | None) -> str | None:
        if first in self.wrapper:
            t = self.wrapper[first]
            return t.get(second or "", t.get("*"))
        if first in self.mod2mem:
            return self.mod2mem[first]
        self.unknown.add(first)
        return None


def items(main_text: str) -> tuple[dict[str, tuple[int, int, str]], list[str]]:
    """name -> (first line, line count, stripped body) for every top-level item, in order."""
    lines = main_text.split("\n")
    stripped = strip(main_text).split("\n")
    starts = [i for i, l in enumerate(lines) if ITEM_HEAD.match(l) or l.startswith("use ") or l.startswith("#[")]
    out: dict[str, tuple[int, int, str]] = {}
    order: list[str] = []
    for k, s in enumerate(starts):
        e = starts[k + 1] if k + 1 < len(starts) else len(lines)
        head = ITEM_HEAD.match(lines[s])
        if not head:
            continue
        s0 = s
        while s0 > 0 and lines[s0 - 1].startswith("#["):
            s0 -= 1
        name = head.group(2)
        out[name] = (s0 + 1, e - s0, "\n".join(stripped[s0:e]))
        order.append(name)
    return out, order


def analyse(root: str):
    ws = Workspace(root)
    rs = Resolver(ws)
    main_text = open(os.path.join(ws.dirs[ws.bin], "src", "main.rs"), encoding="utf-8").read()
    its, order = items(main_text)
    direct: dict[str, set[str]] = {}
    calls: dict[str, set[str]] = {}
    unresolved: dict[str, set[str]] = defaultdict(set)
    for name in order:
        body = its[name][2]
        mems: set[str] = set()
        for m in CLI_REF.finditer(body):
            mem = rs.path_member(m.group(1), m.group(2))
            if mem is None:
                unresolved[name].add(m.group(1))
            else:
                mems.add(mem)
        for m in MEMBER_REF.finditer(body):
            mem = ws.crate_of(m.group(1))
            if mem and mem != ws.bin:
                mems.add(mem)
        for alias, mod in rs.alias.items():
            for m in re.finditer(r"\b%s::([A-Za-z_][A-Za-z0-9_]*)" % re.escape(alias), body):
                mem = rs.path_member(mod, m.group(1))
                if mem:
                    mems.add(mem)
        for bare, mem in rs.bare.items():
            if re.search(r"\b%s\b" % re.escape(bare), body):
                mems.add(mem)
        direct[name] = mems
        calls[name] = {o for o in order if o != name and re.search(r"\b%s\b" % re.escape(o), body)}

    def trans(name: str, seen: set[str]) -> tuple[set[str], set[str]]:
        """(members reached, paths unresolved) through every main.rs item this one calls."""
        out, unres = set(direct[name]), set(unresolved.get(name, ()))
        for c in calls[name]:
            if c not in seen:
                seen.add(c)
                o, u = trans(c, seen)
                out |= o
                unres |= u
        return out, unres

    rows = []
    for name in order:
        line, n, _ = its[name]
        needs, unres = trans(name, {name})
        if unres:
            least = []
        else:
            least = ["any"] if not needs else ws.least(needs)
        adds = {m: ws.adds(m, needs) for m in least if m in ws.deps and ws.adds(m, needs)}
        rows.append({"name": name, "line": line, "lines": n, "direct": sorted(direct[name]), "via": sorted(needs - direct[name]),
                     "least": least, "adds": adds, "calls": sorted(calls[name]), "unresolved": sorted(unres)})
    return ws, rows


def least_label(r: dict) -> str:
    """`keel-write(+keel-args)`: the least members, each with the dependencies it would add."""
    return ",".join(m + (f"(+{'+'.join(r['adds'][m])})" if m in r["adds"] else "") for m in r["least"]) or "unreachable"


def failures(ws: Workspace, rows: list[dict]) -> list[str]:
    out = []
    for r in rows:
        if r["name"] in EXCLUDED:
            continue
        if not r["least"]:
            why = f"unresolved {','.join(r['unresolved'])}" if r["unresolved"] else f"no member reaches {{{','.join(sorted(set(r['direct']) | set(r['via'])))}}}"
            out.append(f"{r['name']} (main.rs:{r['line']}): unreachable - {why}")
        elif r["least"] == [ws.bin]:
            out.append(f"{r['name']} (main.rs:{r['line']}): least member is {ws.bin} - reaches {','.join(sorted(set(r['direct']) | set(r['via'])))}")
    return out


def check(root: str) -> int:
    ws, rows = analyse(root)
    bad = failures(ws, rows)
    for b in bad:
        print("FAIL " + b)
    print(f"verb_homes --check: {len(rows)} items, {len(bad)} that only {ws.bin} can hold")
    return 1 if bad else 0


def probe() -> int:
    fails = 0

    def check_(name: str, ok: bool, detail: str) -> None:
        nonlocal fails
        print(f"  {'pass' if ok else 'FAIL'}  {name} - {detail}")
        fails += 0 if ok else 1

    # Chosen before the real tree is read (D0388): members a <- b, d <- c, the binary depends on all four.
    # b and c each carry a dependency the other lacks, so nothing but the binary could hold a body naming
    # both; a is a leaf under nothing c reaches, so c could add it.
    with tempfile.TemporaryDirectory() as tmp:
        os.makedirs(os.path.join(tmp, ".git"))
        open(os.path.join(tmp, "Cargo.toml"), "w").write('[workspace]\nmembers = ["members/keel-a", "members/keel-b", "members/keel-c", "members/keel-d", "keel-cli"]\n')
        for crate, deps in (("keel-a", []), ("keel-b", ["keel-a"]), ("keel-c", ["keel-d"]), ("keel-d", [])):
            d = os.path.join(tmp, "members", crate, "src")
            os.makedirs(d)
            dep_lines = "".join(f'{x} = {{ path = "../{x}" }}\n' for x in deps)
            open(os.path.join(tmp, "members", crate, "Cargo.toml"), "w").write(f'[package]\nname = "{crate}"\n[dependencies]\n{dep_lines}[dev-dependencies]\nkeel-c = {{ path = "../keel-c" }}\n')
        cli = os.path.join(tmp, "keel-cli", "src")
        os.makedirs(cli)
        open(os.path.join(tmp, "keel-cli", "Cargo.toml"), "w").write('[package]\nname = "keel-cli"\n[dependencies]\nkeel-a = { path = "x" }\nkeel-b = { path = "x" }\nkeel-c = { path = "x" }\nkeel-d = { path = "x" }\n')
        open(os.path.join(cli, "lib.rs"), "w").write(
            "pub use keel_a::x;\npub use keel_b::y;\npub use keel_c as z;\npub mod local;\n"
            "pub mod write {\n    pub use keel_c::issue::{record_issue};\n    pub use keel_b::write::*;\n}\n"
            "pub use keel_a::v::{validate_root, check_files};\n")

        def main_rs(body: str) -> None:
            open(os.path.join(cli, "main.rs"), "w").write("use keel_cli::write as w;\nuse keel_cli::y::{flag};\nuse keel_cli::{validate_root, check_files};\n" + body + "\nfn main() { both(); }\n")

        main_rs("fn only_b() { keel_cli::y::f(); }\nfn root_name() { validate_root(); }\nfn wrapped() { w::record_issue(); }\n"
                "fn plain_w() { w::other(); }\nfn bare() { flag(); }\nfn nothing() { let s = \"keel_cli::z::x\"; /* keel_cli::z::x */ }\n"
                "// keel_cli::z::a in a comment\nfn commented() { let _ = 1; }\nfn sibling() { keel_cli::x::f(); keel_cli::z::g(); }\n")
        ws, rows = analyse(tmp)
        by = {r["name"]: r for r in rows}
        check_("known-positive: dev-dependencies are not reach", "keel-c" not in ws.reach["keel-a"], f"keel-a reaches {sorted(ws.reach['keel-a'])}")
        check_("known-positive: a body naming a member and a leaf it could add lands on the member, naming the addition",
               by["sibling"]["least"] == ["keel-c"] and by["sibling"]["adds"] == {"keel-c": ["keel-a"]}, least_label(by["sibling"]))
        check_("known-positive: a body naming one member lands on it", by["only_b"]["least"] == ["keel-b"], str(by["only_b"]["least"]))
        check_("known-positive: a crate-root name resolves through lib.rs", by["root_name"]["least"] == ["keel-a"], str(by["root_name"]["least"]))
        check_("known-positive: a wrapper-module name resolves to the member that owns it", by["wrapped"]["least"] == ["keel-c"], str(by["wrapped"]["least"]))
        check_("known-positive: a wrapper's glob covers the rest", by["plain_w"]["least"] == ["keel-b"], str(by["plain_w"]["least"]))
        check_("known-positive: a bare imported name is the member it came from", by["bare"]["least"] == ["keel-b"], str(by["bare"]["least"]))
        check_("known-negative: a string literal is not a reference", by["nothing"]["least"] == ["any"], str(by["nothing"]["least"]))
        check_("known-negative: a comment is not a reference", by["commented"]["least"] == ["any"], str(by["commented"]["least"]))
        check_("known-positive: a clean fixture passes --check", failures(ws, rows) == [], str(failures(ws, rows)))

        main_rs("fn both() { keel_cli::y::f(); keel_cli::z::g(); }\nfn calls_both() { both(); }\nfn local_user() { keel_cli::local::f(); }\n"
                "fn unknown_user() { keel_cli::nowhere::f(); }\nfn calls_unknown() { unknown_user(); }\nfn names_main_in_prose() { let _ = \"main\"; }\n")
        ws, rows = analyse(tmp)
        bad = failures(ws, rows)
        named = {b.split(" ")[0] for b in bad}
        check_("known-negative: a body naming two members that each carry a dependency the other lacks is the binary's", any(b.startswith("both ") and "least member is keel-cli" in b for b in bad), "; ".join(bad)[:120])
        check_("known-negative: the caller of such a body is named through the call", "calls_both" in named, str(sorted(named)))
        check_("known-negative: a body reaching a local `pub mod` is the binary's", "local_user" in named and any("least member is keel-cli" in b for b in bad), str(sorted(named)))
        check_("known-negative: a path lib.rs does not resolve is named, never guessed", "unknown_user" in named and any("unresolved nowhere" in b for b in bad), str(sorted(named)))
        check_("known-negative: the caller of an unresolvable body is unresolvable too", any(b.startswith("calls_unknown ") and "unresolved nowhere" in b for b in bad), str(sorted(named)))
        check_("known-negative: `main` in a string is not a call to main", "names_main_in_prose" not in named, str(sorted(named)))
        check_("known-positive: main is excluded", "main" not in named, str(sorted(named)))

    root = repo_root()
    ws, rows = analyse(root)
    check_("this tree: keel-cli is the one binary", ws.bin == "keel-cli", ws.bin)
    check_("this tree: every member manifest is read", len(ws.deps) >= 17, f"{len(ws.deps)} members")
    # After sprint 750 main.rs keeps main, cmd_show and its tests; the floor is that the parse is
    # live, not a count tied to the file's former size (that floor went red the day the verbs moved).
    check_("this tree: main.rs yields items", len(rows) > 0, f"{len(rows)} items")
    check_("this tree: no body is the binary's alone", failures(ws, rows) == [], str(failures(ws, rows)))
    print(f"verb_homes probe: {fails} failure(s)")
    return 1 if fails else 0


def main(argv: list[str]) -> int:
    root = repo_root()
    if "--root" in argv:
        root = os.path.abspath(argv[argv.index("--root") + 1])
    if "--probe" in argv:
        return probe()
    if "--check" in argv:
        return check(root)
    ws, rows = analyse(root)
    if "--by-member" in argv:
        by: dict[str, list[dict]] = defaultdict(list)
        for r in rows:
            by[least_label(r)].append(r)
        for k in sorted(by, key=lambda k: -sum(r["lines"] for r in by[k])):
            print(f"{k:22} {sum(r['lines'] for r in by[k]):5} lines  " + " ".join(f"{r['name']}({r['lines']})" for r in by[k]))
        return 0
    for r in rows:
        print(f"{r['line']:5} {r['lines']:4} {r['name']:38} least={least_label(r):26} direct={','.join(r['direct'])} via={','.join(r['via'])}"
              + (f" UNRESOLVED={','.join(r['unresolved'])}" if r["unresolved"] else ""))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
