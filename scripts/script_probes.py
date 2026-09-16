#!/usr/bin/env python3
# ci-probe: --probe
# not-an-instrument: it is a gate - it runs the other scripts' own probes and relays their exit codes; the only
# number it prints is how many it ran. It measures nothing about the model.
"""Run every script probe in the tree - the ONE runner ci.yml and .githooks/pre-commit both call (issue574).

Discovery from the tree, not a list (D0361): every `scripts/**/*.py` whose first `# ci-probe: <flag>` line
names a flag is run with that flag and must exit 0. Until this file existed the discovery lived only in a
heredoc inside ci.yml's `script probes` step, so two commits (eb82b52's fix 6a493f4 among them) reached main
red on `scripts/module_home.py --probe` with every local gate green - the third member of the
local-gate-differs-from-CI class (issue453, issue572, issue574). The hook now runs this whenever a staged
path matches scripts/**/*.py; CI runs it on every push; the two cannot drift because there is one runner.

    python scripts/script_probes.py [ROOT]     # run the probes under ROOT/scripts (default: the cwd)
    python scripts/script_probes.py --probe    # the D0388 pair over two temp trees, before any real tree is read

Exit 0 = every probe passed (or none was found - said so, not hidden); 1 = `SCRIPT PROBES FAILED:` naming each
script and its flag; 2 = the runner itself could not run. `target/release` is prepended to PATH as ci.yml did,
so a probe that shells out to `keel` finds the built binary on both surfaces.
"""
import glob
import os
import subprocess
import sys
import tempfile

MARKER = "# ci-probe:"


def discover(root):
    """[(relative path, flag)] for every scripts/**/*.py under ROOT carrying a `# ci-probe:` line, sorted."""
    found = []
    for path in sorted(glob.glob(os.path.join(root, "scripts", "**", "*.py"), recursive=True)):
        flag = None
        with open(path, encoding="utf-8", errors="replace") as fh:
            for line in fh:
                s = line.strip()
                if s.startswith(MARKER):
                    flag = s.split(":", 1)[1].strip()
                    break
        if flag:
            found.append((os.path.relpath(path, root).replace(os.sep, "/"), flag))
    return found


def run_probes(root, out=print):
    """Run each discovered probe from ROOT; returns (ran, failed) where failed is [(path, flag, exit)]."""
    env = dict(os.environ)
    env["PATH"] = os.path.join(os.path.abspath(root), "target", "release") + os.pathsep + env.get("PATH", "")
    ran, failed = 0, []
    for rel, flag in discover(root):
        ran += 1
        out(f"== {rel} {flag}")
        r = subprocess.run([sys.executable, rel, flag], cwd=root, env=env)
        if r.returncode != 0:
            failed.append((rel, flag, r.returncode))
    return ran, failed


def report(ran, failed, out=print):
    """The verdict lines CI has printed since D0394; returns the exit code."""
    if ran == 0:
        out("no scripts/**/*.py carries a `# ci-probe:` line; nothing to run")
    if failed:
        out("SCRIPT PROBES FAILED:")
        for rel, flag, code in failed:
            out(f"  {rel} {flag} -> exit {code}")
        return 1
    out(f"{ran} script probe(s) passed")
    return 0


def _tree(scripts):
    """A temp tree with the given {relative path: source}; the D0388 pair reads these, never the real tree."""
    root = tempfile.mkdtemp(prefix="script-probes-")
    for rel, src in scripts.items():
        path = os.path.join(root, rel)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8", newline="\n") as fh:
            fh.write(src)
    return root


def probe():
    """Known-positive: a tree whose one probe exits 1 is SCRIPT PROBES FAILED naming it. Known-negative: a tree
    with one passing probe and one unmarked script is `1 script probe(s) passed`. Chosen before any real tree
    is read (D0388)."""
    lines = []
    pos = _tree({"scripts/failing.py": "# ci-probe: --probe\nimport sys\nsys.exit(1)\n"})
    ran, failed = run_probes(pos, out=lines.append)
    code = report(ran, failed, out=lines.append)
    text = "\n".join(lines)
    if not (code == 1 and ran == 1 and "SCRIPT PROBES FAILED:" in text and "scripts/failing.py --probe -> exit 1" in text):
        print("PROBE FAILED (known-positive): a failing probe was not reported by path and flag\n" + text)
        return 1
    lines.clear()
    neg = _tree({"scripts/passing.py": "# ci-probe: --probe\nimport sys\nsys.exit(0)\n",
                 "scripts/unmarked.py": "import sys\nsys.exit(1)  # never run: no marker\n"})
    ran, failed = run_probes(neg, out=lines.append)
    code = report(ran, failed, out=lines.append)
    text = "\n".join(lines)
    if not (code == 0 and ran == 1 and "1 script probe(s) passed" in text and "unmarked" not in text):
        print("PROBE FAILED (known-negative): one passing probe beside an unmarked script was not `1 script probe(s) passed`\n" + text)
        return 1
    print("PROBE PASS: failing probe -> SCRIPT PROBES FAILED naming scripts/failing.py --probe; passing + unmarked -> 1 script probe(s) passed")
    return 0


def main(argv):
    if argv[1:] == ["--probe"]:
        return probe()
    if len(argv) > 2 or (len(argv) == 2 and argv[1].startswith("-")):
        print(__doc__)
        return 2
    root = argv[1] if len(argv) == 2 else "."
    if not os.path.isdir(os.path.join(root, "scripts")):
        print(f"script_probes: no scripts/ directory under {os.path.abspath(root)} - the runner cannot run")
        return 2
    ran, failed = run_probes(root)
    return report(ran, failed)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
