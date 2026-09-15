"""The delegated-ceremony skill's own check over a RECORDER's report (dcDelegatedCeremonyIsASkill).

A recorder returns its report only after this passes. Seven refusals; the first three are sprint 647's
failures, the next two are sprint 703's (issue532, dcRecorderReportRefusesNonRecordWrites), the last two
are sprint 723's (issue568, dcRecorderReportAccountsForEveryOwedRecord):

  1. an undeclared marker - any `#Word` token in the report that no `metadata def Word` under .engine or
     .tracking declares (sprint 647 typed `#Addresses`; the marker-vocabulary guard turned the Stop hook
     red twice for a thing the primary never wrote);
  2. a typed edge line - a line whose first token is a `#Marker`, declared or not: a recorder's edges come
     from `record sprint --fill`, never from a line it types;
  3. a summary that outran the gate - the last non-empty line is not the verbatim last line of
     `keel gate --fast .` (`gate: fast gate clean ...` or `gate: FAST GATE FAILED ...`), or that line says
     FAILED while the report says `DISCREPANCIES: NONE`;
  4. a write outside the record API - a `WROTE:` line whose command is not `[keel] record <sub-verb>`
     (sprint 703's recorder ran scripts/textpatch.py against the sprint file and reported it as a write;
     the D0425 recorder's only write path is the record API, and a report that admits another is refused);
  5. one record written twice - two `WROTE:` lines naming the same `--gate` or `--task` (sprint 703's
     recorder found the ceremony guard red after its retro-gate write and recorded the gate three more
     times with reworded evidence; a red after a write is a DISCREPANCIES line, never a second attempt);
  6. a report that accounts for nothing - zero `WROTE:` lines and no `REFUSED:` line, with or without
     `--owed` (sprint 723's first recorder ran no record command, wrote a three-line report reading
     DISCREPANCIES: NONE, and the five refusals above passed it: each reads the WROTE lines that ARE there,
     none reads what is absent. A recorder dispatched with nothing to write does not exist);
  7. a shortfall against the dispatch - under `--owed N`, `WROTE:` lines plus `REFUSED:` lines number
     fewer than N. The brief's list of owed records was text the checker never saw - a reminder, which
     D0047 says is not a control; the count is what the dispatch states and the report must meet.

    python check_report.py REPORT [--root DIR] [--owed N]   # exit 0 = pass, 1 = refused (each refusal printed), 2 = usage
    python check_report.py --probe [--root DIR]             # the D0388 pairs from fixtures/ beside this file
    python check_report.py --probe FIXTURE [--root DIR]     # one PAIRS row; exit 0 = that side holds (positive REFUSED
                                                            #   naming its expectation, negative PASSES) - the shape
                                                            #   `keel verify --probe POS,NEG` needs, both sides exit 0

The pairs are stated before the tree is read. fixtures/positive-undeclared-marker.txt types
`#Addresses dependency from ...` and is REFUSED; fixtures/negative-sprint647-receipt-driven.txt is the
sprint 647 ceremony as its recorder should have reported it - write-API calls quoting the receipt, ending
with the gate line - and PASSES. fixtures/positive-sprint703-textpatch-write.txt is sprint 703's report as
returned (one WROTE: line names textpatch.py) and is REFUSED naming that line;
fixtures/positive-sprint703-retro-recorded-twice.txt writes the retro gate twice and is REFUSED naming both
lines; fixtures/negative-sprint703-record-only.txt is the same report with the offending line removed and PASSES.
fixtures/positive-sprint723-nothing-written.txt is sprint 723's first report as returned (three lines, no
WROTE, no REFUSED) and is REFUSED naming zero writes; fixtures/negative-sprint723-seven-owed.txt is the second
recorder's report (seven WROTE lines) and PASSES under --owed 7; fixtures/positive-sprint723-six-of-seven.txt is
that report with one gate line removed and is REFUSED under --owed 7 naming the shortfall (owed 7, accounted 6).
"""
import os
import re
import sys

MARKER_TOKEN = re.compile(r"(?<![\w`'\"])#([A-Z][A-Za-z0-9]*)\b")
# Line-anchored, as the marker-vocabulary guard reads it (guards.rs markers_declared): prose that quotes
# `metadata def X;` mid-line - the guard's own repair hint inside an obligation - declares nothing.
METADATA_DEF = re.compile(r"^[ \t]*metadata[ \t]+def[ \t]+([A-Za-z][A-Za-z0-9]*)", re.M)
GATE_LAST_LINE = re.compile(r"^gate: (fast gate clean\b|FAST GATE FAILED\b)")
WROTE_LINE = re.compile(r"^WROTE:\s*(.*)$")
REFUSED_LINE = re.compile(r"^REFUSED:\s*\S")
# The binary as a brief names it: `keel`, `KEEL`, `<KEEL>`, `$KEEL`, or a path ending in keel[-suffix][.exe].
KEEL_BINARY = re.compile(r"^(<KEEL>|\$KEEL|KEEL|(?:.*[\\/])?keel(?:-[\w-]+)?(?:\.exe)?)$", re.I)
RECORD_KEY = re.compile(r"--(gate|task)\s+(\S+)")


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


def wrote_command(line):
    """The command a WROTE: line reports, or None when the line is not one."""
    m = WROTE_LINE.match(line.strip())
    return m.group(1).strip() if m else None


def is_record_write(command):
    """`record <sub-verb> ...`, with or without the binary in front - the recorder's only write path."""
    tokens = command.split()
    if tokens and KEEL_BINARY.match(tokens[0]):
        tokens = tokens[1:]
    return len(tokens) >= 2 and tokens[0] == "record" and re.fullmatch(r"[a-z][a-z-]*", tokens[1]) is not None


def refusals(report_text, declared, owed=None):
    """The refusals for one report text - pure, so the pairs can pin it without a tree.

    `owed` is the count of records the dispatch named, or None when the caller did not say; the
    nothing-written refusal (6) needs no count, the shortfall refusal (7) needs one.
    """
    found = []
    lines = report_text.splitlines()
    seen = {}
    wrote = 0
    refused = 0
    for n, line in enumerate(lines, 1):
        stripped = line.strip()
        first = stripped.split(" ", 1)[0] if stripped else ""
        if first.startswith("#") and MARKER_TOKEN.match(first):
            found.append(f"line {n}: typed edge line `{stripped[:60]}` - a recorder's edges come from record sprint --fill, never a typed line")
        for m in MARKER_TOKEN.finditer(line):
            if m.group(1) not in declared:
                found.append(f"line {n}: undeclared marker #{m.group(1)} - no metadata def declares it; a marker the vocabulary lacks is a REFUSED line, not a line to type")
        if REFUSED_LINE.match(stripped):
            refused += 1
        command = wrote_command(line)
        if command is None:
            continue
        wrote += 1
        if not is_record_write(command):
            found.append(f"line {n}: write outside the record API `{command[:60]}` - the recorder's only write path is keel record <sub-verb>; a change the API cannot make is a REFUSED line for the primary")
            continue
        for kind, key in RECORD_KEY.findall(command):
            if (kind, key) in seen:
                found.append(f"line {n}: --{kind} {key} written again (first at line {seen[(kind, key)]}) - a red after a write is a DISCREPANCIES line, never a second record with reworded evidence")
            else:
                seen[(kind, key)] = n
    if wrote == 0 and refused == 0:
        found.append("zero WROTE: lines and no REFUSED: line - a recorder dispatched with nothing to write does not exist; this report is a recorder that did nothing and said NONE (sprint 723, issue568)")
    if owed is not None and wrote + refused < owed:
        found.append(f"owed {owed}, accounted {wrote + refused} ({wrote} WROTE, {refused} REFUSED) - every record the dispatch named is a WROTE line or a REFUSED line; a shortfall is work not done, not work not mentioned")
    nonempty = [l for l in lines if l.strip()]
    last = nonempty[-1].strip() if nonempty else ""
    if not GATE_LAST_LINE.match(last):
        found.append(f"last line is not the gate's: `{last[:80]}` - end with the verbatim last line of keel gate --fast . run after the last write")
    elif "FAST GATE FAILED" in last and any(l.strip() == "DISCREPANCIES: NONE" for l in lines):
        found.append("DISCREPANCIES: NONE over a FAST GATE FAILED last line - the tree is red and the report says it is not")
    return found


def check_file(path, root, owed=None):
    with open(path, encoding="utf-8") as fh:
        text = fh.read()
    return refusals(text, declared_markers(root), owed)


# (fixture, what the refusal must name; None = must pass, owed count or None)
PAIRS = [
    ("positive-undeclared-marker.txt", "undeclared marker #Addresses", None),
    ("negative-sprint647-receipt-driven.txt", None, None),
    ("positive-sprint703-textpatch-write.txt", "write outside the record API `python scripts/textpatch.py", None),
    ("positive-sprint703-retro-recorded-twice.txt", "--gate livingDocsNameOnlyDeclaredCliVerbsRetroGate written again", None),
    ("negative-sprint703-record-only.txt", None, None),
    ("positive-sprint723-nothing-written.txt", "zero WROTE: lines and no REFUSED: line", None),
    ("negative-sprint723-seven-owed.txt", None, 7),
    ("positive-sprint723-six-of-seven.txt", "owed 7, accounted 6", 7),
]


def probe(root, only=None):
    """Run the PAIRS rows - all of them, or the one named `only` (a fixture file name). One row is one side
    of `keel verify --probe POS,NEG` (D0476): that ladder needs each side to exit 0 when its case holds, so
    the positive side is `--probe positive-….txt` (exit 0 = REFUSED naming its expectation) and the negative
    side is `--probe negative-….txt` (exit 0 = PASS). A name no row carries is a usage error, never a pass."""
    here = os.path.dirname(os.path.abspath(__file__))
    fx = os.path.join(here, "fixtures")
    rows = PAIRS if only is None else [p for p in PAIRS if p[0] == only]
    if not rows:
        print(f"probe: no PAIRS row is named `{only}`; the rows are: {', '.join(p[0] for p in PAIRS)}")
        return 2
    declared = declared_markers(root)
    all_hold = True
    for name, expect, owed in rows:
        with open(os.path.join(fx, name), encoding="utf-8") as fh:
            got = refusals(fh.read(), declared, owed)
        under = f" under --owed {owed}" if owed is not None else ""
        if expect is None:
            ok = not got
            print(f"probe: known-negative {name}{under} -> {'PASS' if ok else 'REFUSED'}: {len(got)} refusal(s)")
        else:
            ok = any(expect in r for r in got)
            print(f"probe: known-positive {name}{under} -> {'REFUSED' if ok else 'NOT REFUSED'} naming `{expect}`: {len(got)} refusal(s)")
        for r in got:
            print(f"  {r}")
        all_hold = all_hold and ok
    if all_hold:
        print("probe: every pair holds." if only is None else f"probe: {only} holds.")
        return 0
    print("probe: a pair does NOT hold." if only is None else f"probe: {only} does NOT hold.")
    return 1


def main(argv):
    root = "."
    owed = None
    if "--root" in argv:
        i = argv.index("--root")
        if i + 1 >= len(argv):
            print(__doc__)
            return 2
        root = argv[i + 1]
        argv = argv[:i] + argv[i + 2:]
    if "--owed" in argv:
        i = argv.index("--owed")
        if i + 1 >= len(argv) or not argv[i + 1].isdigit():
            print(__doc__)
            return 2
        owed = int(argv[i + 1])
        argv = argv[:i] + argv[i + 2:]
    if "--probe" in argv:
        i = argv.index("--probe")
        only = argv[i + 1] if i + 1 < len(argv) and not argv[i + 1].startswith("--") else None
        return probe(root, only)
    if len(argv) != 1:
        print(__doc__)
        return 2
    found = check_file(argv[0], root, owed)
    if found:
        print(f"check_report: REFUSED ({len(found)}):")
        for r in found:
            print(f"  {r}")
        return 1
    accounted = f", every one of the {owed} owed records accounted for" if owed is not None else ""
    print(f"check_report: pass - no typed or undeclared marker, every write a keel record sub-verb written once{accounted}, report ends with the gate's last line")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
