"""The delegated-ceremony skill's check of a VERIFIER's receipt against the ladder it describes (issue603,
dcVerifierReceiptIsCheckedAgainstTheLadder, D0538).

Sprint 743's verifier wrote its receipt from a `running` stub read before the ladder ended: `LADDER: ... ->
KILLED during touched; exit=2`, `touched=killed`, `DISCREPANCIES: ... pid 29824 dead` - while `tasklist`
showed pid 29824 alive, `keel verify --wait .` printed `in flight: touched, pid 29824 alive`, and the ladder
then ended `touched fail (101)` after 1102 s, 880 passed and 23 failed. The verifier's own background
`--wait` returned that 101 and its report restated it as confirming the kill. Sprint 726's verifier did the
same (issue575); the control added then was a sentence in the test-verify skill saying to wait. Since D0533
the receipt is RENDERED from the receipt files, so a verifier no longer types `killed` - but nothing read the
file the recorder is handed against the ladder receipt at the moment of the hand-over: a render taken from
an earlier run, from the exit-3 KILLED path, or edited after the fact, reached the recorder as the evidence.
This check is that read (D0047): given the receipt file and --root it runs `KEEL verify --wait ROOT`, which
returns at once when the ladder has ended and blocks while its writer's pid lives, then holds the receipt's
every verdict field to `.keel/metrics/verify-receipt.toml` and `.keel/metrics/touched-receipt.toml`.

Refusals - each names the receipt line and the file value it disagrees with:

  1. the ladder has no verdict - `--wait` did not print `has ended`: it said KILLED (exit 2) or found no
     receipt. Nothing on disk is a verdict for the receipt to agree with; the ladder is relaunched;
  2. the receipt says the ladder did not end while `--wait` says it did - a first line `KILLED at <rung>` /
     `WAIT:` / `NO RECEIPT` (the renderer's exit-3 and exit-2 texts written as the file), a LADDER line
     carrying `KILLED` or `outcome=running`, or a rung verdict `killed`: the receipt was written before the
     ladder ended (sprint 743's shape). The refusal names the rung and exit `--wait` printed;
  3. the receipt describes another run - its LADDER `at=` is not the `at` of the ladder that ended: an
     earlier run's render, or a later run's over a different tree;
  4. LADDER `outcome=` / `stopped_at=` in the receipt differ from `verify-receipt.toml`;
  5. a rung verdict (`validate= guard= clippy= probe= touched=`) differs from that rung's `[[rung]]` row,
     or names a rung the file has no row for;
  6. TOUCHED RECEIPT `outcome=` / `passed=` / `failed=` / `at=` differ from `touched-receipt.toml` when the
     ladder reached the touched rung (`stopped_at` is `none` or `touched`); a receipt saying `not reached`
     over a ladder that reached it, or quoting a touched run over a ladder that stopped before it;
  7. `verify-receipt.toml` moved between `--wait` and the read - the file's `at` is not the `at` `--wait`
     printed: another ladder launched in between; the check is run again once it ends.

    python check_receipt.py RECEIPT --root DIR --keel KEEL     # exit 0 = agrees, 1 = refused (each printed), 2 = usage
    python check_receipt.py --probe                            # the D0388 pairs from fixtures/ beside this file
    python check_receipt.py --probe FIXTURE                    # one PAIRS row; exit 0 = that side holds (positive
                                                               #   REFUSED naming its expectation, negative PASSES)

The pairs are stated before the tree is read, all judged over sprint 743's ended ladder (fixtures/sprint743-
verify-receipt.toml, fixtures/sprint743-touched-receipt.toml - reconstructed from the `--wait` table and the
sprint's recorded counts, since the live files were overwritten by the next run - and fixtures/sprint743-
wait.txt, the `--wait` output as printed, exit 101). fixtures/positive-sprint743-killed-while-ended.txt is the
receipt as returned and is REFUSED naming the ended ladder's `touched fail (101)`;
fixtures/negative-sprint743-rendered-fail.txt is its corrected form - the D0533 rendering of that ended ladder,
touched fail, 880 passed, 23 failed - and PASSES (a red receipt that agrees with the ladder is a receipt);
fixtures/positive-rendered-killed-while-ended.txt is the renderer's exit-3 text written as the file and is
REFUSED the same way; fixtures/positive-sprint743-other-run-quoted.txt is the rendered receipt of the run that
followed (outcome=pass at 1789728746, 904 passed) judged over the 1789725917 ladder and is REFUSED naming the
other run's `at`.
"""
import os
import re
import subprocess
import sys
import tomllib

RUNGS = ("validate", "guard", "clippy", "probe", "touched")
KV = re.compile(r"(\w+)=(\S+)")
ENDED = re.compile(r"has ended \(head (\w+), at (\d+)\)")
TABLE = re.compile(r"^\s+(validate|guard|clippy|probe|touched)\s+(pass|fail \(-?\d+\)|not-run|not-named)\s+(\d+) s\s*$")
SUMMARY = re.compile(r"^keel verify: (pass|fail) - (.*?); receipt ")
KILLED_WAIT = re.compile(r"KILLED during (\w+)")


# ── reading ───────────────────────────────────────────────────────────────────────────────────────


def read_toml(path):
    try:
        with open(path, "rb") as fh:
            return tomllib.load(fh)
    except (OSError, tomllib.TOMLDecodeError):
        return None


def parse_receipt(text):
    """The verifier receipt's verdict fields, from the rendered shape (D0533) or sprint 743's typed one."""
    lines = text.splitlines()
    first = next((l.strip() for l in lines if l.strip()), "")
    r = {"first": first, "header": first.startswith("VERIFIER RECEIPT"), "ladder": {}, "ladder_tail": "",
         "rungs": {}, "touched": None, "touched_line": None}
    ladder_line = next((l for l in lines if l.startswith("LADDER:")), None)
    if ladder_line is not None and "->" in ladder_line:
        tail = ladder_line.split("->", 1)[1]
        r["ladder_tail"] = tail.strip()
        r["ladder"] = dict(KV.findall(tail))
    rung_line = next((l for l in lines if l.strip().startswith("validate=")), None)
    if rung_line is not None:
        r["rungs"] = dict(KV.findall(rung_line))
    touched_line = next((l for l in lines if l.startswith("TOUCHED RECEIPT:")), None)
    if touched_line is not None:
        r["touched_line"] = touched_line.strip()
        r["touched"] = dict(KV.findall(touched_line.split(":", 1)[1]))
    return r


def parse_wait(text):
    """What `keel verify --wait` printed: the ended ladder's head/at, its rung table, its summary - or KILLED."""
    w = {"ended": None, "table": {}, "summary": None, "killed": None}
    for line in text.splitlines():
        m = ENDED.search(line)
        if m:
            w["ended"] = (m.group(1), int(m.group(2)))
        m = TABLE.match(line)
        if m:
            w["table"][m.group(1)] = m.group(2)
        m = SUMMARY.match(line.strip())
        if m:
            w["summary"] = line.strip()
        m = KILLED_WAIT.search(line)
        if m and "--wait" in line:
            w["killed"] = m.group(1)
    return w


# ── the pure check ────────────────────────────────────────────────────────────────────────────────


def refusals(receipt_text, wait_text, wait_exit, ladder, touched):
    """Every disagreement between the receipt and the ladder that ended, as one line each. `ladder` and
    `touched` are the parsed .toml files read AFTER --wait returned (None when absent)."""
    out = []
    rc = parse_receipt(receipt_text)
    w = parse_wait(wait_text)

    # 1. no verdict on disk
    if w["ended"] is None or ladder is None:
        said = w["summary"] or next((l.strip() for l in wait_text.splitlines() if l.strip()), "(nothing)")
        what = f"KILLED during {w['killed']}" if w["killed"] else ("no ladder receipt" if ladder is None else "no `has ended` line")
        out.append(f"the ladder has no verdict: keel verify --wait exit={wait_exit} printed \"{said}\" ({what}) - a receipt cannot describe a run that did not end; relaunch the ladder")
        return out

    head_w, at_w = w["ended"]
    at_f = int(ladder.get("at", 0) or 0)
    verdict_line = w["summary"] or f"exit={wait_exit}"
    touched_w = w["table"].get("touched", "?")
    stopped_f = str(ladder.get("stopped_at", "?"))
    outcome_f = str(ladder.get("outcome", "?"))

    # 7. the file moved between the wait and the read
    if at_f != at_w:
        out.append(f"verify-receipt.toml moved between --wait (at={at_w}) and the read (at={at_f}): another ladder was launched in between - run the check again once it ends")
        return out

    # 2. the receipt says the ladder did not end
    tail = rc["ladder_tail"]
    claims = []
    if not rc["header"]:
        claims.append(f"first line \"{rc['first'][:80]}\"")
    if "KILLED" in tail:
        claims.append(f"LADDER line \"-> {tail[:60]}\"")
    if rc["ladder"].get("outcome") == "running":
        claims.append("LADDER outcome=running")
    killed_rungs = [n for n, v in rc["rungs"].items() if v == "killed"]
    if killed_rungs:
        claims.append("rung verdict killed for " + ", ".join(killed_rungs))
    if claims:
        ended_as = f"{stopped_f} {touched_w}" if stopped_f != "none" else "every rung green"
        out.append(f"the receipt says the ladder did not end ({'; '.join(claims)}) while it ended at={at_w}: --wait printed \"{verdict_line}\" (stopped at {ended_as}), exit={wait_exit} - the receipt was written before the ladder ended")

    # 3. another run
    at_r = rc["ladder"].get("at")
    if at_r is not None and at_r.isdigit() and int(at_r) != at_f:
        out.append(f"the receipt describes the run at={at_r}; the ladder that ended wrote at={at_f} - a render of another run, not of this ladder")
    elif rc["header"] and at_r is None:
        out.append("the receipt's LADDER line carries no at= field - it cannot be matched to the ladder that ended")

    # 4. outcome / stopped_at
    for key, val in (("outcome", outcome_f), ("stopped_at", stopped_f)):
        got = rc["ladder"].get(key)
        if got is not None and got != val:
            out.append(f"LADDER {key}={got} in the receipt; verify-receipt.toml says {key}={val}")

    # 5. rung verdicts
    rows = {str(x.get("name")): str(x.get("verdict", "absent")) for x in ladder.get("rung", [])}
    for name in RUNGS:
        got = rc["rungs"].get(name)
        if got is None:
            if rc["header"]:
                out.append(f"rung {name}: the receipt names no verdict; verify-receipt.toml [[rung]] says {rows.get(name, 'absent')}")
            continue
        want = rows.get(name, "absent")
        if got != want:
            exit_ = next((x.get("exit") for x in ladder.get("rung", []) if str(x.get("name")) == name), None)
            ex = f" (exit {exit_})" if want == "fail" and exit_ is not None else ""
            out.append(f"rung {name}: receipt says {got}, verify-receipt.toml [[rung]] says {want}{ex}")

    # 6. the touched receipt
    reached = stopped_f in ("none", "touched")
    tl = rc["touched_line"] or ""
    if reached:
        if touched is None:
            if rc["touched"] and rc["touched"].get("outcome") not in (None, "absent"):
                out.append(f"TOUCHED RECEIPT \"{tl[:70]}\" in the receipt; touched-receipt.toml is absent")
        elif "not reached" in tl or rc["touched"] is None:
            out.append(f"the ladder reached the touched rung (stopped_at={stopped_f}) and the receipt says \"{tl[:70] or 'nothing'}\" - written before that rung ran")
        else:
            t_at_f = int(touched.get("at", 0) or 0)
            t_at_r = rc["touched"].get("at")
            if t_at_r is not None and t_at_r.isdigit() and int(t_at_r) != t_at_f:
                out.append(f"TOUCHED RECEIPT at={t_at_r} in the receipt; touched-receipt.toml is dated at={t_at_f} - rewritten by a later run or quoted from an earlier one")
            for key in ("outcome", "passed", "failed"):
                got = rc["touched"].get(key)
                want = str(touched.get(key, "absent"))
                if got is not None and got != want:
                    out.append(f"TOUCHED RECEIPT {key}={got} in the receipt; touched-receipt.toml says {key}={want}")
    elif rc["touched"] and rc["touched"].get("outcome") not in (None, "absent") and "not reached" not in tl:
        out.append(f"the ladder stopped at {stopped_f}, before the touched rung, and the receipt quotes a touched run \"{tl[:70]}\" - another run's")
    return out


# ── running against a tree ────────────────────────────────────────────────────────────────────────


def run_wait(keel, root):
    try:
        r = subprocess.run([keel, "verify", "--wait", root], capture_output=True, text=True, timeout=3600)
    except (OSError, subprocess.TimeoutExpired) as e:
        return f"keel verify --wait did not run: {e}", 2
    return r.stdout + r.stderr, r.returncode


def check_file(path, root, keel, metrics=None):
    with open(path, encoding="utf-8") as fh:
        text = fh.read()
    wait_text, wait_exit = run_wait(keel, root)
    metrics = metrics or os.path.join(root, ".keel", "metrics")
    ladder = read_toml(os.path.join(metrics, "verify-receipt.toml"))
    touched = read_toml(os.path.join(metrics, "touched-receipt.toml"))
    return refusals(text, wait_text, wait_exit, ladder, touched), wait_text, wait_exit


# (fixture, what the refusal must name; None = must pass)
PAIRS = [
    ("positive-sprint743-killed-while-ended.txt", "stopped at touched fail (101)"),
    ("negative-sprint743-rendered-fail.txt", None),
    ("positive-rendered-killed-while-ended.txt", "stopped at touched fail (101)"),
    ("positive-sprint743-other-run-quoted.txt", "the receipt describes the run at=1789728746; the ladder that ended wrote at=1789725917"),
]
WAIT_FIXTURE = ("sprint743-wait.txt", 101)
LADDER_FIXTURE = "sprint743-verify-receipt.toml"
TOUCHED_FIXTURE = "sprint743-touched-receipt.toml"


def probe(only=None):
    """Run the PAIRS rows - all, or the one named `only`. One row is one side of `keel verify --probe POS,NEG`
    (D0476): the positive side exits 0 when REFUSED naming its expectation, the negative side when it PASSES.
    A name no row carries is a usage error, never a pass."""
    here = os.path.dirname(os.path.abspath(__file__))
    fx = os.path.join(here, "fixtures")
    rows = PAIRS if only is None else [p for p in PAIRS if p[0] == only]
    if not rows:
        print(f"probe: no PAIRS row is named `{only}`; the rows are: {', '.join(p[0] for p in PAIRS)}")
        return 2
    with open(os.path.join(fx, WAIT_FIXTURE[0]), encoding="utf-8") as fh:
        wait_text = fh.read()
    ladder = read_toml(os.path.join(fx, LADDER_FIXTURE))
    touched = read_toml(os.path.join(fx, TOUCHED_FIXTURE))
    if ladder is None or touched is None:
        print(f"probe: the ladder fixtures {LADDER_FIXTURE} / {TOUCHED_FIXTURE} are absent or unreadable")
        return 2
    all_hold = True
    for name, expect in rows:
        with open(os.path.join(fx, name), encoding="utf-8") as fh:
            got = refusals(fh.read(), wait_text, WAIT_FIXTURE[1], ladder, touched)
        over = f" over the ladder at={ladder.get('at')} (--wait exit {WAIT_FIXTURE[1]})"
        if expect is None:
            ok = not got
            print(f"probe: known-negative {name}{over} -> {'PASS' if ok else 'REFUSED'}: {len(got)} refusal(s)")
        else:
            ok = any(expect in r for r in got)
            print(f"probe: known-positive {name}{over} -> {'REFUSED' if ok else 'NOT REFUSED'} naming `{expect}`: {len(got)} refusal(s)")
        for r in got:
            print(f"  {r}")
        all_hold = all_hold and ok
    if all_hold:
        print("probe: every pair holds." if only is None else f"probe: {only} holds.")
        return 0
    print("probe: a pair does NOT hold." if only is None else f"probe: {only} does NOT hold.")
    return 1


def main(argv):
    root, keel, metrics = ".", None, None
    for flag in ("--root", "--keel", "--metrics"):
        if flag in argv:
            i = argv.index(flag)
            if i + 1 >= len(argv):
                print(__doc__)
                return 2
            val = argv[i + 1]
            argv = argv[:i] + argv[i + 2:]
            if flag == "--root":
                root = val
            elif flag == "--keel":
                keel = val
            else:
                metrics = val
    if "--probe" in argv:
        i = argv.index("--probe")
        only = argv[i + 1] if i + 1 < len(argv) and not argv[i + 1].startswith("--") else None
        return probe(only)
    if len(argv) != 1 or keel is None:
        print(__doc__)
        return 2
    found, wait_text, wait_exit = check_file(argv[0], root, keel, metrics)
    if found:
        print(f"check_receipt: REFUSED ({len(found)}):")
        for r in found:
            print(f"  {r}")
        return 1
    w = parse_wait(wait_text)
    head, at = w["ended"]
    print(f"check_receipt: pass - the receipt agrees with the ladder that ended at={at} (head {head}; --wait exit {wait_exit}: \"{w['summary']}\"): outcome, stopped_at, every rung verdict and the touched counts read from the receipt files")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
