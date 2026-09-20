#!/usr/bin/env python3
# ci-probe: --probe
"""The intra-crate module graph of keel-cli (D0479, dcGuardsViewOrientCycleIsBroken), and the
workspace's build-script edges (issue585, dcBuildScriptHasOneHomeBelowItsUsers).

Every top-level module under keel-cli/src (a file `X.rs` or a directory `X/`) is a node; every
`crate::Y::`, `crate::Y;`, `crate::Y ` or `crate::{... Y ...}` token in module X's SOURCE - comments
stripped, so a doc line that names `guards::run_in_parallel` in prose is not an edge - is an edge
X -> Y. Re-exported members (`pub use keel_git::gitx` in lib.rs) are nodes too, referenced the same way.

A crate's `build =` is a dependency edge too: the crate is built from that file. Cargo does not say so
(`cargo tree` never lists a build script), so five members reached UP to keel-cli/build.rs for the
provenance stamp while the layering check read only [dependencies] and passed. `--check` reads every
workspace member's manifest and names a build script that sits in a crate the user does not depend on:
a script in the crate's own directory, in a crate below it, or in a directory no crate owns
(build/provenance.rs, the workspace's) is fine.

    python scripts/modgraph.py                      # every edge with its count, then the cycles
    python scripts/modgraph.py --edges guards view  # the reference lines behind one edge
    python scripts/modgraph.py --check              # the D0479 layering + the build-script edges: exit 1 naming each
    python scripts/modgraph.py --build-edges        # every crate's build script and who owns it
    python scripts/modgraph.py --root DIR ...       # a fixture src tree instead of keel-cli/src
    python scripts/modgraph.py --workspace DIR ...  # a fixture workspace (its Cargo.toml) instead of the repo's
    python scripts/modgraph.py --probe              # the D0388 pair over a temp workspace, then this repo's --check

`--check` holds the layering the crate split needs before the next boundary is drawn: nothing in
view, orient, write or scaffold references guards, and nothing in view references orient, and no
crate is built from a script a crate above it owns. It names both ends and every offending line, so a
fixture with one such reference is reported by file:line. Exit 0 when the layering holds, 1 otherwise.
"""
import os
import re
import sys
import tempfile
import tomllib
from collections import defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(HERE)
DEFAULT_SRC = os.path.join(REPO, "keel-cli", "src")

# (from, to) pairs the layering forbids. Lower layers never reference higher ones (D0479: read
# model, write API, views, then guards); orient is a view over the model that other views may not
# reach back into.
FORBIDDEN = [("view", "guards"), ("orient", "guards"), ("write", "guards"), ("scaffold", "guards"), ("view", "orient")]

REF = re.compile(r"\bcrate::(\{[^}]*\}|[A-Za-z_][A-Za-z0-9_]*)")
STRING = re.compile(r'"(?:\\.|[^"\\])*"')


def strip_comments(line):
    """Drop a `//` comment - after removing string literals so a URL inside quotes survives."""
    no_str = STRING.sub(lambda m: " " * len(m.group(0)), line)
    i = no_str.find("//")
    return line if i < 0 else line[:i]


def modules(src):
    out = {}
    for name in sorted(os.listdir(src)):
        p = os.path.join(src, name)
        if os.path.isdir(p):
            out[name] = [os.path.join(dp, f) for dp, _, fs in os.walk(p) for f in fs if f.endswith(".rs")]
        elif name.endswith(".rs") and name not in ("lib.rs", "main.rs"):
            out[name[:-3]] = [p]
    # re-exported members are modules of the crate for reference purposes
    lib = os.path.join(src, "lib.rs")
    if os.path.exists(lib):
        with open(lib, encoding="utf-8") as f:
            for m in re.finditer(r"^pub use keel_\w+::(\w+);", f.read(), re.M):
                out.setdefault(m.group(1), [])
    return out


def edges(src):
    mods = modules(src)
    names = set(mods)
    counts = defaultdict(int)
    lines = defaultdict(list)
    for frm, files in mods.items():
        for path in files:
            with open(path, encoding="utf-8", errors="replace") as f:
                for n, raw in enumerate(f, 1):
                    code = strip_comments(raw)
                    for m in REF.finditer(code):
                        tok = m.group(1)
                        targets = re.findall(r"[A-Za-z_][A-Za-z0-9_]*", tok) if tok.startswith("{") else [tok]
                        for t in targets:
                            if t in names and t != frm:
                                counts[(frm, t)] += 1
                                lines[(frm, t)].append("%s:%d: %s" % (os.path.relpath(path, src), n, raw.strip()[:140]))
    return mods, counts, lines


def sccs(nodes, adj):
    """Tarjan; returns the strongly connected components with more than one node."""
    index, low, on, st, out, ctr = {}, {}, set(), [], [], [0]

    def visit(v):
        index[v] = low[v] = ctr[0]
        ctr[0] += 1
        st.append(v)
        on.add(v)
        for w in adj.get(v, ()):
            if w not in index:
                visit(w)
                low[v] = min(low[v], low[w])
            elif w in on:
                low[v] = min(low[v], index[w])
        if low[v] == index[v]:
            comp = []
            while True:
                w = st.pop()
                on.discard(w)
                comp.append(w)
                if w == v:
                    break
            if len(comp) > 1:
                out.append(sorted(comp))

    sys.setrecursionlimit(10000)
    for v in sorted(nodes):
        if v not in index:
            visit(v)
    return out


def _rel(root, path):
    return os.path.relpath(path, root).replace(os.sep, "/")


def crates(root):
    """{name: crate dir} for every [workspace].members entry of ROOT/Cargo.toml, the name read from each manifest."""
    with open(os.path.join(root, "Cargo.toml"), "rb") as f:
        ws = tomllib.load(f)
    out = {}
    for member in ws.get("workspace", {}).get("members", []):
        d = os.path.normpath(os.path.join(root, member))
        with open(os.path.join(d, "Cargo.toml"), "rb") as f:
            out[tomllib.load(f)["package"]["name"]] = d
    return out


def path_deps(manifest):
    """The names of every `path =` dependency in [dependencies] and [build-dependencies] - what the crate is built on."""
    out = set()
    for table in ("dependencies", "build-dependencies"):
        for name, spec in manifest.get(table, {}).items():
            if isinstance(spec, dict) and "path" in spec:
                out.add(spec.get("package", name))
    return out


def build_edges(root):
    """One row per crate that has a build script: (crate, script rel path, owner crate or None, manifest:line, upward).

    `upward` is True when the owner is neither the crate itself nor a crate it depends on, transitively,
    through path dependencies - the crate is then built from a file a crate above it (or beside it) owns.
    """
    dirs = crates(root)
    manifests = {}
    for name, d in dirs.items():
        with open(os.path.join(d, "Cargo.toml"), "rb") as f:
            manifests[name] = tomllib.load(f)
    deps = {n: path_deps(m) for n, m in manifests.items()}

    def closure(name):
        seen, todo = set(), [name]
        while todo:
            for dep in deps.get(todo.pop(), ()):
                if dep not in seen:
                    seen.add(dep)
                    todo.append(dep)
        return seen

    rows = []
    for name, d in sorted(dirs.items()):
        pkg = manifests[name]["package"]
        build = pkg.get("build", os.path.isfile(os.path.join(d, "build.rs")))
        if build is False or build is None:
            continue
        script = os.path.normpath(os.path.join(d, "build.rs" if build is True else build))
        owner = None
        for other, od in dirs.items():
            if script == od or script.startswith(od + os.sep):
                if owner is None or len(od) > len(dirs[owner]):
                    owner = other
        where = _rel(root, os.path.join(d, "Cargo.toml"))
        with open(os.path.join(d, "Cargo.toml"), encoding="utf-8") as f:
            for n, line in enumerate(f, 1):
                if re.match(r"^\s*build\s*=", line):
                    where += ":%d: %s" % (n, line.strip())
                    break
        upward = owner is not None and owner != name and owner not in closure(name)
        rows.append((name, _rel(root, script), owner, where, upward))
    return rows


def check_build_edges(root):
    """Print each upward build script; return their count."""
    bad = [r for r in build_edges(root) if r[4]]
    for name, script, owner, where, _ in bad:
        print("FORBIDDEN build script: %s -> %s (%s) - %s is not a dependency of %s" % (name, owner, script, owner, name))
        print("    " + where)
    return len(bad)


def _fixture(tmp, low_build):
    """A two-crate workspace: `top` depends on `low`; `low` is built from LOW_BUILD (relative to members/low)."""
    files = {
        "Cargo.toml": '[workspace]\nmembers = ["top", "members/low"]\nresolver = "2"\n',
        "top/Cargo.toml": '[package]\nname = "top"\nversion = "0.0.0"\nedition = "2021"\n\n[dependencies]\nlow = { path = "../members/low" }\n',
        "top/build.rs": "fn main() {}\n",
        "top/src/lib.rs": "",
        "members/low/Cargo.toml": '[package]\nname = "low"\nversion = "0.0.0"\nedition = "2021"\nbuild = "%s"\n' % low_build,
        "members/low/src/lib.rs": "",
        "build/provenance.rs": "fn main() {}\n",
    }
    for rel, text in files.items():
        p = os.path.join(tmp, rel)
        os.makedirs(os.path.dirname(p), exist_ok=True)
        with open(p, "w", encoding="utf-8", newline="\n") as f:
            f.write(text)
    return tmp


def probe():
    """D0388: the pair over a temp workspace first, then this repo's --check as the landed negative."""
    fails = 0
    with tempfile.TemporaryDirectory() as tmp:
        pos = _fixture(os.path.join(tmp, "pos"), "../../top/build.rs")
        rows = [r for r in build_edges(pos) if r[4]]
        ok = len(rows) == 1 and rows[0][0] == "low" and rows[0][2] == "top" and "members/low/Cargo.toml:5" in rows[0][3]
        print("probe positive (a member built from the crate above it): %s - %s" % ("PASS" if ok else "FAIL", rows))
        fails += not ok
        neg = _fixture(os.path.join(tmp, "neg"), "../../build/provenance.rs")
        rows = build_edges(neg)
        ok = not any(r[4] for r in rows) and [r[2] for r in rows] == [None, "top"]
        print("probe negative (the workspace-level script, top's own build.rs): %s - %s" % ("PASS" if ok else "FAIL", rows))
        fails += not ok
    n = check_build_edges(REPO)
    print("probe landed workspace: %s - %d upward build script(s)" % ("PASS" if n == 0 else "FAIL", n))
    fails += n != 0
    mods, counts, _ = edges(DEFAULT_SRC)
    bad = [(f, t) for (f, t) in FORBIDDEN if counts.get((f, t))]
    print("probe landed module graph: %s - %d forbidden edge(s) over %d modules" % ("PASS" if not bad else "FAIL", len(bad), len(mods)))
    fails += bool(bad)
    print("modgraph probe: %d failure(s)" % fails)
    return 1 if fails else 0


def main():
    args = sys.argv[1:]
    if args == ["--probe"]:
        return probe()
    src = DEFAULT_SRC
    root = REPO
    if "--root" in args:
        src = os.path.abspath(args[args.index("--root") + 1])
    if "--workspace" in args:
        root = os.path.abspath(args[args.index("--workspace") + 1])
    if "--build-edges" in args:
        for name, script, owner, where, upward in build_edges(root):
            print("%-14s built from %-28s owner %-10s %s%s" % (name, script, owner or "(no crate)", where, "  UPWARD" if upward else ""))
        return 0
    mods, counts, lines = edges(src)
    if "--edges" in args:
        i = args.index("--edges")
        key = (args[i + 1], args[i + 2])
        for ln in lines.get(key, []):
            print(ln)
        print("%s -> %s: %d reference(s)" % (key[0], key[1], counts.get(key, 0)))
        return 0
    if "--check" in args:
        bad = [(f, t) for (f, t) in FORBIDDEN if counts.get((f, t))]
        for f, t in bad:
            print("FORBIDDEN %s -> %s (%d reference(s))" % (f, t, counts[(f, t)]))
            for ln in lines[(f, t)]:
                print("    " + ln)
        upward = check_build_edges(root)
        if bad or upward:
            print("modgraph: %d forbidden edge(s), %d upward build script(s) - the layering does not hold" % (len(bad), upward))
            return 1
        print("modgraph: layering holds - no reference from view, orient, write or scaffold into guards; none from view into orient; no crate built from a script a crate above it owns (%d modules, %d edges, %d build scripts)" % (len(mods), len(counts), len(build_edges(root))))
        return 0
    adj = defaultdict(set)
    for (f, t), c in sorted(counts.items(), key=lambda kv: (-kv[1], kv[0])):
        adj[f].add(t)
        print("%-20s -> %-20s %d" % (f, t, c))
    print("--- cycles (strongly connected components)")
    for comp in sccs(mods, adj):
        print("  " + " <-> ".join(comp))
    return 0


if __name__ == "__main__":
    sys.exit(main())
