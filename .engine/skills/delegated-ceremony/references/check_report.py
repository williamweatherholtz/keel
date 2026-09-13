"""The delegated-ceremony skill's own check over a RECORDER's report (dcDelegatedCeremonyIsASkill).

A recorder returns its report only after this passes. Three refusals, each one of sprint 647's failures:

  1. an undeclared marker - any `#Word` token in the report that no `metadata def Word` under .engine or
     .tracking declares (sprint 647 typed `#Addresses`; the marker-vocabulary guard turned the Stop hook
     red twice for a thing the primary never wrote);
  2. a typed edge line - a line whose first token is a `#Marker`, declared or not: a recorder's edges come
     from `record sprint --fill`, never from a line it types;
  3. a summary that outran the gate - the last non-empty line is not the verbatim last line of
     `keel gate --fast .` (`gate: fast gate clean ...` or `gate: FAST GATE FAILED ...`), or that line says
     FAILED while the report says `DISCREPANCIES: NONE`.

    python check_report.py REPORT [--root DIR]     # exit 0 = pass, 1 = refused (each refusal printed), 2 = usage
    python check_report.py --probe [--root DIR]    # the D0388 pair from fixtures/ beside this file

The pair is stated before the tree is read: fixtures/positive-undeclared-marker.txt is a report that types
`#Addresses dependency from ...` and is REFUSED; fixtures/negative-sprint647-receipt-driven.txt is the
sprint 647 ceremony as its recorder should have reported it - write-API calls quoting the receipt, ending
with the gate line - and PASSES.
"""
import os
import re
import sys

MARKER_TOKEN = re.compile(r"(?<![\w`'\"])#([A-Z][A-Za-z0-9]*)\b")
# Line-anchored, as the marker-vocabulary guard reads it (guards.rs markers_declared): prose that quotes
# `metadata def X;` mid-line - the guard's own repair hint inside an obligation - declares nothing.
METADATA_DEF = re.compile(r"^[ \t]*metadata[ \t]+def[ \t]+([A-Za-z][A-Za-z0-9]*)", re.M)
GATE_LAST_LINE = re.compile(r"^gate: (fast gate clean\b|FAST GATE FAILED\b)")


def declared_markers(root):
    """Every `metadata def X` under .engine and .tracking - the marker-vocabulary guard's own source (D0136)."""
    out = set()
    for top in (".engine", ".tracking"):
        for dirpath, _, files in os.walk(os.path.join(root, top)):
            for name in files:
                if not name.endswith(".sysml"):
                    continue
                try:
                    with open(os.path.join(dirpath, name), encoding="utf-8", errors="replace") as fh:
                        text = "\n".join(l for l in fh.read().splitlines() if not l.strip().startswith("//"))
                        out.update(METADATA_DEF.findall(text))
                except OSError:
                    continue
    return out


def refusals(report_text, declared):
    """The refusals for one report text - pure, so the pair can pin it without a tree."""
    found = []
    lines = report_text.splitlines()
    for n, line in enumerate(lines, 1):
        stripped = line.strip()
        first = stripped.split(" ", 1)[0] if stripped else ""
        if first.startswith("#") and MARKER_TOKEN.match(first):
            found.append(f"line {n}: typed edge line `{stripped[:60]}` - a recorder's edges come from record sprint --fill, never a typed line")
        for m in MARKER_TOKEN.finditer(line):
            if m.group(1) not in declared:
                found.append(f"line {n}: undeclared marker #{m.group(1)} - no metadata def declares it; a marker the vocabulary lacks is a REFUSED line, not a line to type")
    nonempty = [l for l in lines if l.strip()]
    last = nonempty[-1].strip() if nonempty else ""
    if not GATE_LAST_LINE.match(last):
        found.append(f"last line is not the gate's: `{last[:80]}` - end with the verbatim last line of keel gate --fast . run after the last write")
    elif "FAST GATE FAILED" in last and any(l.strip() == "DISCREPANCIES: NONE" for l in lines):
        found.append("DISCREPANCIES: NONE over a FAST GATE FAILED last line - the tree is red and the report says it is not")
    return found


def check_file(path, root):
    with open(path, encoding="utf-8") as fh:
        text = fh.read()
    return refusals(text, declared_markers(root))


def probe(root):
    here = os.path.dirname(os.path.abspath(__file__))
    fx = os.path.join(here, "fixtures")
    pos = check_file(os.path.join(fx, "positive-undeclared-marker.txt"), root)
    neg = check_file(os.path.join(fx, "negative-sprint647-receipt-driven.txt"), root)
    pos_ok = any("undeclared marker #Addresses" in r for r in pos)
    neg_ok = not neg
    print(f"probe: known-positive (types #Addresses) -> {'REFUSED' if pos_ok else 'NOT REFUSED'}: {len(pos)} refusal(s)")
    for r in pos:
        print(f"  {r}")
    print(f"probe: known-negative (sprint 647 receipt-driven report) -> {'PASS' if neg_ok else 'REFUSED'}: {len(neg)} refusal(s)")
    for r in neg:
        print(f"  {r}")
    if pos_ok and neg_ok:
        print("probe: both hold.")
        return 0
    print("probe: the pair does NOT hold.")
    return 1


def main(argv):
    root = "."
    if "--root" in argv:
        i = argv.index("--root")
        if i + 1 >= len(argv):
            print(__doc__)
            return 2
        root = argv[i + 1]
        argv = argv[:i] + argv[i + 2:]
    if "--probe" in argv:
        return probe(root)
    if len(argv) != 1:
        print(__doc__)
        return 2
    found = check_file(argv[0], root)
    if found:
        print(f"check_report: REFUSED ({len(found)}):")
        for r in found:
            print(f"  {r}")
        return 1
    print("check_report: pass - no typed or undeclared marker, report ends with the gate's last line")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
