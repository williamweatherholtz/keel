"""The intra-crate module graph of keel-cli (D0479, dcGuardsViewOrientCycleIsBroken).

Every top-level module under keel-cli/src (a file `X.rs` or a directory `X/`) is a node; every
`crate::Y::`, `crate::Y;`, `crate::Y ` or `crate::{... Y ...}` token in module X's SOURCE - comments
stripped, so a doc line that names `guards::run_in_parallel` in prose is not an edge - is an edge
X -> Y. Re-exported members (`pub use keel_git::gitx` in lib.rs) are nodes too, referenced the same way.

    python scripts/modgraph.py                      # every edge with its count, then the cycles
    python scripts/modgraph.py --edges guards view  # the reference lines behind one edge
    python scripts/modgraph.py --check              # the D0479 layering: exit 1 naming each forbidden edge
    python scripts/modgraph.py --root DIR ...       # a fixture src tree instead of keel-cli/src

`--check` holds the layering the crate split needs before the next boundary is drawn: nothing in
view, orient, write or scaffold references guards, and nothing in view references orient. It names
both ends and every offending line, so a fixture with one such reference is reported by file:line.
Exit 0 when the layering holds, 1 otherwise.
"""
import os
import re
import sys
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


def main():
    args = sys.argv[1:]
    src = DEFAULT_SRC
    if "--root" in args:
        src = os.path.abspath(args[args.index("--root") + 1])
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
        if bad:
            print("modgraph: %d forbidden edge(s) - the layering does not hold" % len(bad))
            return 1
        print("modgraph: layering holds - no reference from view, orient, write or scaffold into guards; none from view into orient (%d modules, %d edges)" % (len(mods), len(counts)))
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
