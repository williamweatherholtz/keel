#!/usr/bin/env python3
# ci-probe: --probe
"""Render the D0425 verifier's receipt from the receipt FILES - never from the agent's reading of them.

Resolves issue490 (a verifier typed `DISCREPANCIES: NONE` over a touched run whose own receipt said
`outcome = "fail"`; it recurred in sprints 676, 745 and 750) and issue567 (a verifier read a `running`
stub seventeen seconds after launch, typed `killed`, and stopped while five cargo processes were alive).
Both are the same defect: a verdict line TYPED from evidence the agent summarised. This script is the
control (D0047): every line of the receipt below `VERIFIER RECEIPT` down to `DISCREPANCIES:` is a
rendering of a field, and the only line the verifier supplies is `VERIFIER-NOTED WRITES` (`--noted`),
which is its own noticing and nobody's verdict (D0516).

    python scripts/verify_receipt.py --keel <KEEL> [--root R] [--noted LINE]... [--out FILE]
    python scripts/verify_receipt.py --probe            # the D0388 known cases first

Exit codes - the skill's step 5 loops on them:
    0  the RESULT block was rendered (green OR red: the block says which; a red is a rendered receipt too)
    2  WAIT: the ladder receipt says `running` and its writer is alive - one line, no block; call again
    3  KILLED: the receipt says `running` and no writer is alive - the block is the death, never a verdict

What is read, and from where (nothing is inferred):
    .keel/metrics/verify-receipt.toml   the ladder (D0476): head, at, seconds, outcome, stopped_at, running,
                                        pid, one [[rung]] per rung (name, verdict, exit, seconds, command)
    .keel/metrics/touched-receipt.toml  the touched run (D0421): outcome, passed, failed, failing, seconds, at,
                                        stems, unattributed, lib, head, log, runner, [[timing]]
    .keel/metrics/verify-launch.epoch   the launch time the verifier wrote before `keel verify`
    .keel/metrics/verify-touched.out    the ladder's stdout - the first red line and the test summary line
    git                                 HEAD, the dirty-path count, and the changed paths' stems, computed
                                        exactly as members/keel-suite/src/touched.rs computes them
    KEEL gate check-engine . / KEEL sync-claude --check .   run HERE, their last line and exit copied

Liveness (issue567): the observable that separates in-flight from killed is a process, not a field. The
receipt stub names its writer's `pid` (D0493); when it does, that pid is the check. When it does not, any
live keel-serve.exe / keel.exe / cargo.exe is - the DoD's original observable - and `--alive yes|no`
overrides both for the probe fixtures.

DISCREPANCIES is COMPUTED: one line per rung that is not `pass`, per failing test name, per stale `at`,
per stems mismatch, per head mismatch, per non-zero outside check. `NONE` is what is printed when that
list is empty - it is never typed.
"""
from __future__ import annotations

import argparse
import datetime as _dt
import os
import subprocess
import sys
import tempfile
import time
import tomllib
from pathlib import Path

LADDER = "verify-receipt.toml"
TOUCHED = "touched-receipt.toml"
EPOCH = "verify-launch.epoch"
OUT = "verify-touched.out"
RUNGS = ("validate", "guard", "clippy", "probe", "touched")
LADDER_PROCESSES = ("keel-serve.exe", "keel.exe", "cargo.exe", "keel-serve", "keel", "cargo")


# ── the stem rule, mirrored from touched.rs (module_stem / embedded_stem / compute) ──────────────────


def source_rel(p: str) -> str | None:
    if p.startswith("keel-cli/src/"):
        return p[len("keel-cli/src/"):]
    if not p.startswith("members/"):
        return None
    rest = p[len("members/"):]
    if "/src/" not in rest:
        return None
    crate_dir, rel = rest.split("/src/", 1)
    return rel if crate_dir and "/" not in crate_dir else None


def module_stem(path: str) -> str | None:
    p = path.replace("\\", "/")
    rel = source_rel(p)
    if rel is None or not rel.endswith(".rs"):
        return None
    parts = rel[: -len(".rs")].split("/")
    last = parts.pop()
    if last == "mod":
        if not parts:
            return None
        stem = parts[-1]
    else:
        stem = last
    if stem in ("main", "lib", ""):
        return None
    return stem


def embedded_stem(path: str) -> str | None:
    p = path.replace("\\", "/")
    if p.startswith(".engine/") and len(p) > len(".engine/"):
        return "init"
    return None


def git(root: Path, *args: str) -> str | None:
    try:
        r = subprocess.run(["git", *args], cwd=root, capture_output=True, text=True, timeout=60)
    except (OSError, subprocess.TimeoutExpired):
        return None
    return r.stdout.strip() if r.returncode == 0 else None


def changed_stems(root: Path) -> list[str] | None:
    """The stems the binary would key on for the tree as it is now; None when git is unavailable."""
    branch = git(root, "rev-parse", "--abbrev-ref", "HEAD")
    if branch is None:
        return None
    remote = f"origin/{branch}"
    base = remote if git(root, "rev-parse", "--verify", "--quiet", f"{remote}^{{commit}}") is not None else None
    if base is None and git(root, "rev-parse", "--verify", "--quiet", "HEAD~1^{commit}") is not None:
        base = "HEAD~1"
    if base is not None:
        frm = git(root, "merge-base", base, "HEAD") or base
        diff = git(root, "diff", "--name-only", frm) or ""
        untracked = git(root, "ls-files", "--others", "--exclude-standard", "--", "keel-cli", "members", ".engine") or ""
        paths = [l for l in diff.splitlines() + untracked.splitlines() if l]
    else:
        paths = (git(root, "ls-files", "keel-cli/src", "keel-cli/tests", "members") or "").splitlines()
    stems = {s for p in paths for s in (module_stem(p), embedded_stem(p)) if s}
    return sorted(stems)


# ── liveness ──────────────────────────────────────────────────────────────────────────────────────


def pid_alive(pid: int) -> bool:
    if pid <= 0:
        return False
    if os.name == "nt":
        try:
            r = subprocess.run(["tasklist", "/FI", f"PID eq {pid}", "/NH"], capture_output=True, text=True, timeout=30)
        except (OSError, subprocess.TimeoutExpired):
            return False
        return str(pid) in r.stdout
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    return True


def any_ladder_process_alive() -> bool:
    if os.name == "nt":
        try:
            r = subprocess.run(["tasklist", "/NH"], capture_output=True, text=True, timeout=30)
        except (OSError, subprocess.TimeoutExpired):
            return False
        names = {line.split()[0].lower() for line in r.stdout.splitlines() if line.strip()}
        return any(n.lower() in names for n in LADDER_PROCESSES)
    for n in ("keel-serve", "keel", "cargo"):
        try:
            if subprocess.run(["pgrep", "-x", n], capture_output=True, timeout=30).returncode == 0:
                return True
        except (OSError, subprocess.TimeoutExpired):
            continue
    return False


def writer_alive(ladder: dict, alive_flag: str) -> bool:
    if alive_flag == "yes":
        return True
    if alive_flag == "no":
        return False
    pid = int(ladder.get("pid", 0) or 0)
    return pid_alive(pid) if pid else any_ladder_process_alive()


# ── reading ───────────────────────────────────────────────────────────────────────────────────────


def read_toml(path: Path) -> dict | None:
    try:
        with open(path, "rb") as f:
            return tomllib.load(f)
    except (OSError, tomllib.TOMLDecodeError):
        return None


def read_epoch(path: Path) -> int | None:
    try:
        return int(path.read_text(encoding="utf-8").strip())
    except (OSError, ValueError):
        return None


def out_lines(path: Path) -> list[str]:
    try:
        return path.read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError:
        return []


def first_red_line(lines: list[str]) -> str:
    for l in lines:
        t = l.strip()
        low = t.lower()
        if (
            t.startswith(("FAIL", "error[", "error:"))
            or " FAIL " in f" {t} "
            or "FAILED" in t
            or low.startswith(("keel verify: fail", "keel suite --touched: fail", "keel gate: fail"))
        ):
            return t
    return "none"


def compact_set(line: str) -> str:
    """`over [a, b, c]` -> `over [3 binaries]`: the set is in the receipt's `ran`; the line is for a reader."""
    i = line.find(" over [")
    if i < 0:
        return line
    j = line.find("]", i)
    if j < 0:
        return line
    n = len([x for x in line[i + 7 : j].split(",") if x.strip()])
    return f"{line[:i]} over [{n} binaries]{line[j + 1:]}"


def test_summary_line(lines: list[str]) -> str:
    """The touched run's own verdict line as the ladder printed it (`keel suite --touched: <verdict> - ...`)."""
    for l in reversed(lines):
        t = l.strip()
        if t.startswith("keel suite --touched:") and " - " in t:
            return compact_set(t)
    for l in reversed(lines):
        t = l.strip()
        if "tests run:" in t or t.startswith("test result:"):
            return t
    return "none"


def run_outside(keel: str | None, root: Path, *args: str) -> tuple[str, int | None]:
    if not keel:
        return "not run (no --keel)", None
    try:
        r = subprocess.run([keel, *args], cwd=root, capture_output=True, text=True, timeout=600)
    except (OSError, subprocess.TimeoutExpired) as e:
        return f"did not run: {e}", None
    text = (r.stdout + r.stderr).strip().splitlines()
    last = next((l.strip() for l in reversed(text) if l.strip()), "(no output)")
    return last, r.returncode


# ── rendering ─────────────────────────────────────────────────────────────────────────────────────


def fmt_list(xs: list) -> str:
    return "[" + ", ".join(str(x) for x in xs) + "]"


def render(root: Path, metrics: Path, keel: str | None, noted: list[str], alive_flag: str, now: int | None = None) -> tuple[str, int]:
    now = int(time.time()) if now is None else now
    ladder = read_toml(metrics / LADDER)
    if ladder is None:
        return f"NO RECEIPT: {metrics / LADDER} is absent or unreadable - nothing was launched\n", 3
    epoch = read_epoch(metrics / EPOCH)
    at = int(ladder.get("at", 0) or 0)
    outcome = str(ladder.get("outcome", ""))
    rung_now = str(ladder.get("running", "none"))

    if outcome == "running":
        elapsed = now - (epoch if epoch is not None else at)
        if writer_alive(ladder, alive_flag):
            return f"WAIT: ladder in flight ({rung_now}, {elapsed} s)\n", 2
        pid = ladder.get("pid", "unnamed")
        return (
            f"KILLED at {rung_now}: the receipt says running and no writer is alive "
            f"(pid={pid}; verify-launch.epoch={epoch if epoch is not None else 'absent'}; receipt at={at}) - "
            f"no verdict on either side; relaunch\n"
        ), 3

    disc: list[str] = []
    head_git = git(root, "rev-parse", "--short", "HEAD") or "unknown"
    dirty = (git(root, "status", "--short") or "").splitlines()
    tree = "clean" if not dirty else f"{len(dirty)} dirty paths"
    date = _dt.datetime.now(_dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    rungs = {str(r.get("name")): r for r in ladder.get("rung", [])}
    verdicts = {n: str(rungs[n].get("verdict", "absent")) if n in rungs else "absent" for n in RUNGS}
    stopped = str(ladder.get("stopped_at", "?"))
    seconds = ladder.get("seconds", "?")
    head_l = str(ladder.get("head", "?"))
    launch_ok = epoch is not None and at >= epoch
    launch_note = f"({at} >= {epoch}: ok)" if launch_ok else (f"({at} < {epoch}: STALE)" if epoch is not None else "(no verify-launch.epoch)")
    probe_cmd = str(rungs.get("probe", {}).get("command", "")) if "probe" in rungs else ""
    ladder_cmd = f"{keel or 'KEEL'} verify ."
    if probe_cmd.startswith("--probe-from"):
        # `--probe-from <file>: <pos> ; <neg>` - the separator is colon-space; a Windows drive letter is not
        ladder_cmd += " " + probe_cmd.split(": ", 1)[0]
    lines = [
        f"VERIFIER RECEIPT  {date}  head={head_git}  tree={tree}",
        f"LADDER: {ladder_cmd} -> outcome={outcome} stopped_at={stopped} seconds={seconds} at={at} {launch_note}",
        "  " + " ".join(f"{n}={verdicts[n]}" for n in RUNGS),
    ]
    out = out_lines(metrics / OUT)
    lines.append(f"  first red line: {first_red_line(out) if outcome != 'pass' else 'none'}")

    if outcome != "pass":
        disc.append(f"LADDER: outcome={outcome} stopped_at={stopped}")
    for n in RUNGS:
        v = verdicts[n]
        if v == "fail":
            disc.append(f"rung {n}: fail (exit {rungs[n].get('exit', '?')}) - {rungs[n].get('command', '')}")
        elif v in ("absent",):
            disc.append(f"rung {n}: no [[rung]] row in {LADDER}")
    not_run = [n for n in RUNGS if verdicts[n] == "not-run"]
    if not_run:
        disc.append(f"rungs not run: {fmt_list(not_run)}")
    if not launch_ok:
        disc.append(f"LADDER at={at} is not after verify-launch.epoch={epoch}: a previous run's receipt" if epoch is not None else f"{EPOCH} absent: the launch time was not recorded")
    if head_l != head_git:
        disc.append(f"LADDER head={head_l} != git HEAD {head_git}")

    ce_line, ce_exit = run_outside(keel, root, "gate", "check-engine", ".")
    sc_line, sc_exit = run_outside(keel, root, "sync-claude", "--check", ".")
    lines.append(f"{keel or 'KEEL'} gate check-engine . -> {ce_line}; exit={ce_exit if ce_exit is not None else 'n/a'}")
    lines.append(f"{keel or 'KEEL'} sync-claude --check . -> {sc_line}; exit={sc_exit if sc_exit is not None else 'n/a'}")
    if ce_exit not in (None, 0):
        disc.append(f"gate check-engine exit={ce_exit}: {ce_line}")
    if sc_exit not in (None, 0):
        disc.append(f"sync-claude --check exit={sc_exit}: {sc_line}")

    pv = verdicts["probe"]
    if pv == "not-named":
        lines.append("PROBE PAIR: not named by the dispatch")
        disc.append("probe rung: not-named - the dispatch named no pair file")
    elif pv == "not-run":
        lines.append(f"PROBE PAIR: {probe_cmd} -> not run (ladder stopped at {stopped})")
    elif pv == "absent":
        lines.append("PROBE PAIR: no probe rung in the receipt")
    else:
        lines.append(f"PROBE PAIR: {probe_cmd} -> {pv}")

    if stopped in ("none", "touched"):
        t = read_toml(metrics / TOUCHED)
        if t is None:
            lines.append(f"TOUCHED RECEIPT: {metrics / TOUCHED} absent or unreadable")
            disc.append(f"touched receipt absent: {metrics / TOUCHED}")
        else:
            t_out = str(t.get("outcome", "?"))
            t_at = int(t.get("at", 0) or 0)
            t_ok = epoch is not None and t_at > epoch
            t_note = f"({t_at} > {epoch}: ok)" if t_ok else (f"({t_at} <= {epoch}: STALE)" if epoch is not None else "(no verify-launch.epoch)")
            failing = [str(x) for x in t.get("failing", [])]
            lines.append(
                f"TOUCHED RECEIPT: outcome={t_out} passed={t.get('passed', '?')} failed={t.get('failed', '?')} "
                f"seconds={t.get('seconds', '?')} at={t_at} launch={epoch if epoch is not None else 'absent'} {t_note}"
            )
            stems = [str(x) for x in t.get("stems", [])]
            changed = changed_stems(root)
            match = "MATCH" if changed is not None and sorted(stems) == changed else ("no git: not compared" if changed is None else "MISMATCH")
            lines.append(
                f"  stems={fmt_list(stems)} changed={fmt_list(changed) if changed is not None else '(no git)'} ({match}) "
                f"lib={str(t.get('lib', '?')).lower()} head={t.get('head', '?')} log={t.get('log', '?')} runner={t.get('runner', '?')}"
            )
            timing = t.get("timing", [])
            if timing:
                top = timing[0]
                lines.append(f"  long pole (critical path): {top.get('binary', '?')} \"{top.get('test', '?')}\" {top.get('millis', '?')} millis ({top.get('verdict', '?')})")
            lines.append(f"  test result line: \"{test_summary_line(out)}\"")
            if t_out != "pass":
                disc.append(f"TOUCHED RECEIPT: outcome={t_out} passed={t.get('passed', '?')} failed={t.get('failed', '?')}")
            for name in failing:
                disc.append(f"failing test: {name}")
            if t_out == "running":
                disc.append("touched receipt still says running - the stub, not a verdict")
            if not t_ok:
                disc.append(f"TOUCHED at={t_at} is not after verify-launch.epoch={epoch}: a previous run's receipt" if epoch is not None else f"{EPOCH} absent: touched receipt not dated against a launch")
            if t_at > at:
                # the ladder writes touched-receipt.toml and THEN its own receipt; a touched receipt dated after
                # the ladder was written by a later run (keel land, a second suite) and is not the ladder's evidence
                disc.append(f"TOUCHED at={t_at} is after LADDER at={at}: rewritten by a later run, not the ladder's touched rung (its line is in {OUT})")
            if match == "MISMATCH":
                disc.append(f"stems {fmt_list(stems)} != changed {fmt_list(changed or [])}: the receipt is over another change set")
            if stems and str(t.get("lib", "")).lower() != "true":
                disc.append("lib=false with non-empty stems: the lib run did not happen")
            if str(t.get("head", "")) != head_git:
                disc.append(f"TOUCHED head={t.get('head', '?')} != git HEAD {head_git}")
    else:
        lines.append(f"TOUCHED RECEIPT: not reached (ladder stopped at {stopped})")

    lines.append("DISCREPANCIES: NONE" if not disc else "DISCREPANCIES:")
    lines.extend(f"  {d}" for d in disc)
    noted = [n for n in noted if n.strip()]
    lines.append("VERIFIER-NOTED WRITES: NONE" if not noted else "VERIFIER-NOTED WRITES:")
    lines.extend(f"  {n}" for n in noted)
    return "\n".join(lines) + "\n", 0


# ── the D0388 known cases ─────────────────────────────────────────────────────────────────────────


def _ladder_toml(outcome: str, stopped: str, running: str, touched_verdict: str, at: int, pid: int = 0) -> str:
    rows = []
    for n in RUNGS:
        v = touched_verdict if n == "touched" else "pass"
        if outcome == "running" and n in ("probe", "touched"):
            v = "not-run"
        cmd = "--probe-from C:/pair.txt: python pos.py ; python neg.py" if n == "probe" else f"KEEL {n}"
        rows.append(f'[[rung]]\nname = "{n}"\nverdict = "{v}"\nexit = {0 if v == "pass" else 1}\nseconds = 1\ncommand = "{cmd}"\n')
    return (
        f'head = "abc12345"\nat = {at}\nseconds = 10\noutcome = "{outcome}"\nstopped_at = "{stopped}"\nrunning = "{running}"\npid = {pid}\n\n'
        + "\n".join(rows)
    )


def _touched_toml(outcome: str, passed: int, failed: int, failing: list[str], at: int) -> str:
    return (
        f'head = "abc12345"\nat = {at}\nbase = "origin/main"\nstems = []\nunattributed = []\ntests = []\nlib = false\n'
        f'members = []\noutcome = "{outcome}"\npassed = {passed}\nfailed = {failed}\nfailing = [{", ".join(chr(34) + f + chr(34) for f in failing)}]\n'
        f'ran = []\nskipped = []\nseconds = 9\nrunner = "cargo-nextest 0.9.144"\nlog = "./.keel/metrics/touched-1.log"\n\n'
        f'[[timing]]\nbinary = "b"\ntest = "slowest"\nmillis = 5\nverdict = "pass"\n'
    )


def probe() -> int:
    fails = 0

    def check_(name: str, ok: bool, detail: str) -> None:
        nonlocal fails
        print(f"  {'pass' if ok else 'FAIL'}  {name} - {detail}")
        fails += 0 if ok else 1

    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        subprocess.run(["git", "init", "-q"], cwd=root, check=False, capture_output=True)
        subprocess.run(["git", "-c", "user.email=p@x", "-c", "user.name=p", "commit", "-q", "--allow-empty", "-m", "seed"], cwd=root, check=False, capture_output=True)
        head = git(root, "rev-parse", "--short", "HEAD") or "abc12345"
        metrics = root / ".keel" / "metrics"
        metrics.mkdir(parents=True)
        epoch = 1_000_000
        (metrics / EPOCH).write_bytes(f"{epoch}\n".encode())
        (metrics / OUT).write_bytes(b"running\nFAIL [ 1.0s] b two::names\n     Summary [ 9.0s] 5 tests run: 3 passed, 2 failed\n")

        # known-positive: a red touched run renders FAIL and names both failing tests - the issue490 case
        (metrics / LADDER).write_bytes(_ladder_toml("fail", "touched", "none", "fail", epoch + 50).replace("abc12345", head).encode())
        (metrics / TOUCHED).write_bytes(_touched_toml("fail", 3, 2, ["a::one", "b::two"], epoch + 49).replace("abc12345", head).encode())
        text, code = render(root, metrics, None, [], "auto", now=epoch + 60)
        check_("known-positive: outcome=fail renders a red block naming both failing tests",
               code == 0 and "outcome=fail" in text and "failing test: a::one" in text and "failing test: b::two" in text and "DISCREPANCIES: NONE" not in text,
               f"exit={code}; " + " | ".join(l for l in text.splitlines() if l.startswith(("LADDER", "DISCREP", "  failing"))))

        # known-negative: a green run renders PASS with the count and DISCREPANCIES: NONE
        (metrics / LADDER).write_bytes(_ladder_toml("pass", "none", "none", "pass", epoch + 50).replace("abc12345", head).encode())
        (metrics / TOUCHED).write_bytes(_touched_toml("pass", 7, 0, [], epoch + 49).replace("abc12345", head).encode())
        (metrics / OUT).write_bytes(b"     Summary [ 9.0s] 7 tests run: 7 passed, 0 failed\n")
        text, code = render(root, metrics, None, ["a write the recorder should see"], "auto", now=epoch + 60)
        check_("known-negative: outcome=pass renders PASS with the count and DISCREPANCIES: NONE",
               code == 0 and "outcome=pass" in text and "passed=7 failed=0" in text and "DISCREPANCIES: NONE" in text and "VERIFIER-NOTED WRITES:\n  a write" in text
               # the drive-letter path survives the command split (the first cut split on ':' and printed `--probe-from C`)
               and "verify . --probe-from C:/pair.txt -> outcome=pass" in text,
               f"exit={code}; " + " | ".join(l for l in text.splitlines() if l.startswith(("TOUCHED", "DISCREP"))))

        # known-positive (issue567): running + a live writer is WAIT, exit 2, no block
        (metrics / LADDER).write_bytes(_ladder_toml("running", "none", "clippy", "not-run", epoch + 5, pid=os.getpid()).replace("abc12345", head).encode())
        text, code = render(root, metrics, None, [], "yes", now=epoch + 78)
        check_("known-positive: running + alive writer is one WAIT line, exit 2, no RESULT block",
               code == 2 and text.startswith("WAIT: ladder in flight (clippy, 78 s)") and "TOUCHED RECEIPT" not in text and "DISCREPANCIES" not in text,
               f"exit={code}; {text.strip()}")
        text, code = render(root, metrics, None, [], "auto", now=epoch + 78)
        check_("known-positive: --alive auto reads the receipt's pid (this process) as alive",
               code == 2 and text.startswith("WAIT:"), f"exit={code}; {text.strip()}")

        # known-negative (issue567): running + no writer is KILLED naming the epoch and the receipt's at
        text, code = render(root, metrics, None, [], "no", now=epoch + 78)
        check_("known-negative: running + no writer renders KILLED at <rung> naming verify-launch.epoch and at",
               code == 3 and text.startswith("KILLED at clippy") and f"verify-launch.epoch={epoch}" in text and f"receipt at={epoch + 5}" in text and "DISCREPANCIES" not in text,
               f"exit={code}; {text.strip()}")

        # a touched receipt written AFTER the ladder finished belongs to a later run (keel land overwrote it
        # on 2026-09-19: 736 passed in the file, 963 in the ladder's stdout) - a discrepancy, not the evidence
        (metrics / LADDER).write_bytes(_ladder_toml("pass", "none", "none", "pass", epoch + 50).replace("abc12345", head).encode())
        (metrics / TOUCHED).write_bytes(_touched_toml("pass", 736, 0, [], epoch + 900).replace("abc12345", head).encode())
        text, code = render(root, metrics, None, [], "auto", now=epoch + 1000)
        check_("known-positive: a touched receipt dated after the ladder is a rewritten-by-a-later-run discrepancy",
               code == 0 and "rewritten by a later run" in text and "DISCREPANCIES: NONE" not in text,
               " | ".join(l for l in text.splitlines() if "rewritten" in l))
        (metrics / TOUCHED).write_bytes(_touched_toml("pass", 7, 0, [], epoch + 49).replace("abc12345", head).encode())

        # a stale receipt (at before the launch) is a discrepancy, never a pass
        (metrics / LADDER).write_bytes(_ladder_toml("pass", "none", "none", "pass", epoch - 50).replace("abc12345", head).encode())
        text, code = render(root, metrics, None, [], "auto", now=epoch + 60)
        check_("known-positive: a receipt dated before the launch is STALE and a discrepancy",
               code == 0 and "STALE" in text and "previous run's receipt" in text, " | ".join(l for l in text.splitlines() if "STALE" in l or "previous" in l))

        # no receipt at all
        (metrics / LADDER).unlink()
        text, code = render(root, metrics, None, [], "auto", now=epoch + 60)
        check_("known-negative: no receipt is NO RECEIPT, exit 3", code == 3 and text.startswith("NO RECEIPT"), text.strip())

    # the stem rule mirrors touched.rs
    check_("stem: keel-cli/src/view/mod.rs -> view", module_stem("keel-cli/src/view/mod.rs") == "view", str(module_stem("keel-cli/src/view/mod.rs")))
    check_("stem: members/keel-git/src/gitx.rs -> gitx", module_stem("members/keel-git/src/gitx.rs") == "gitx", str(module_stem("members/keel-git/src/gitx.rs")))
    check_("stem: main.rs and lib.rs name no module", module_stem("keel-cli/src/main.rs") is None and module_stem("members/keel-fs/src/lib.rs") is None, "None, None")
    check_("stem: a path under .engine/ contributes init", embedded_stem(".engine/skills/x/SKILL.md") == "init" and embedded_stem(".engine/") is None, "init, None")
    print(f"verify_receipt probe: {fails} failure(s)")
    return 1 if fails else 0


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--probe", action="store_true", help="run the D0388 known cases and exit")
    ap.add_argument("--root", default=".", help="project root (default: cwd)")
    ap.add_argument("--metrics", default=None, help="receipt directory (default: ROOT/.keel/metrics)")
    ap.add_argument("--keel", default=None, help="the keel binary; runs gate check-engine and sync-claude --check")
    ap.add_argument("--noted", action="append", default=[], help="one VERIFIER-NOTED WRITES line (repeatable)")
    ap.add_argument("--alive", choices=("auto", "yes", "no"), default="auto", help="override the writer-liveness read (fixtures)")
    ap.add_argument("--out", default=None, help="also write the rendered receipt to this file")
    a = ap.parse_args(argv)
    if a.probe:
        return probe()
    root = Path(a.root).resolve()
    metrics = Path(a.metrics).resolve() if a.metrics else root / ".keel" / "metrics"
    text, code = render(root, metrics, a.keel, a.noted, a.alive)
    sys.stdout.write(text)
    if a.out and code in (0, 3):
        Path(a.out).write_bytes(text.encode("utf-8"))
    return code


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
