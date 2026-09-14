"""Does the ready frontier honour a #DependsOn edge between two work items? (dcReadyHonoursItemDependencies, issue549/issue552, D0388)

Builds a throwaway tracking repo and asks the built binary for `show orient`. Per D0388 the two known cases were
chosen before the real tree was read, and both must hold before the real tree's frontier is believed:

  pos: dcSecond #DependsOn dcFirst, dcFirst undone; storyChartered #CharteredBy a PROPOSED Decision.
       The frontier must list dcSecond under blocked (waitsOn dcFirst, why "not done") and NOT under ready,
       and storyChartered must not be ready. A binary from before the fix lists all three ready - run against
       target/release/keel-land.exe at 130675d it did, which is how this case earned its name.
  neg: the same repo after a pass on dcFirst and the Decision accepted: blocked is empty, dcSecond and
       storyChartered are ready, answerStatus COMPUTED.

    python scripts/probes/ready_honours_item_dependencies.py pos [--bin PATH]
    python scripts/probes/ready_honours_item_dependencies.py neg [--bin PATH]

Exits 0 when the named case holds, 1 with the frontier it read otherwise. `--bin` defaults to
target/release/keel.exe under the repository this script lives in.
"""
import json
import os
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))

ACTORS = ('package Actors {\n    private import EngineElement::*;\n'
          '    part hum : Person { :>> id = "00000000-0000-4000-8000-000000000101"; :>> title = "hum"; }\n}\n')
BACKLOG = ('package Fx {\n    private import EngineElement::*;\n    private import EngineWork::*;\n'
           '    private import EngineVerification::*;\n    private import EngineRelationships::*;\n\n'
           '    action def Build {\n'
           '        action dcFirst;\n'
           '        verification dcFirstDoD : Test { :>> id = "00000000-0000-4000-8000-000000000011"; :>> method = VerificationMethod::test; :>> procedureText = "the predecessor"; }\n'
           '        action dcSecond;\n'
           '        verification dcSecondDoD : Test { :>> id = "00000000-0000-4000-8000-000000000012"; :>> method = VerificationMethod::test; :>> procedureText = "the dependant"; }\n'
           '        action storyChartered;\n'
           '        verification storyCharteredDoD : Test { :>> id = "00000000-0000-4000-8000-000000000013"; :>> method = VerificationMethod::test; :>> procedureText = "the chartered story"; }\n'
           '    }\n'
           '    #DependsOn dependency from dcSecond to dcFirst;\n'
           '    #CharteredBy dependency from storyChartered to d0001;\n}\n')


def decision(status):
    return ('package Decision0001 {\n    private import EngineElement::*;\n    part d0001 : Decision {\n'
            '        :>> id = "00000000-0000-4000-8000-000000000001";\n        :>> title = "probe";\n'
            '        :>> createdAt = "2026-09-01";\n        :>> createdBy = "hum";\n'
            '        :>> status = DecisionStatus::%s;\n        :>> context = "c";\n'
            '        :>> decision = "the charter under probe";\n        :>> rationale = "r";\n'
            '        :>> consequences = "q";\n    }\n}\n' % status)


def git(root, *args):
    subprocess.run(["git", "-C", root, "-c", "user.email=p@x", "-c", "user.name=p", *args], check=True,
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def write(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)


def orient(root, binary):
    env = dict(os.environ, KEEL_OFFLINE="1", KEEL_ACTOR="claudeOpus5")
    p = subprocess.run([binary, "show", "orient", "."], cwd=root, env=env, capture_output=True, text=True)
    try:
        return json.loads(p.stdout)
    except json.JSONDecodeError:
        sys.exit("orient did not answer JSON (exit %s):\n%s\n%s" % (p.returncode, p.stdout[:2000], p.stderr[:2000]))


def main():
    args = sys.argv[1:]
    if not args or args[0] not in ("pos", "neg"):
        sys.exit(__doc__)
    case = args[0]
    binary = os.path.join(REPO, "target", "release", "keel.exe")
    if "--bin" in args:
        binary = os.path.abspath(args[args.index("--bin") + 1])
    root = tempfile.mkdtemp(prefix="keel-itemdeps-probe-")
    try:
        write(os.path.join(root, ".keel", "actor"), "claudeOpus5\n")
        write(os.path.join(root, ".tracking", "actors.sysml"), ACTORS)
        write(os.path.join(root, ".tracking", "backlog.sysml"), BACKLOG)
        dec = os.path.join(root, ".engine", "decisions", "0001-probe.sysml")
        write(dec, decision("proposed"))
        git(root, "init", "-q", ".")
        git(root, "add", "-A")
        git(root, "commit", "-q", "-m", "seed")
        if case == "neg":
            sha = subprocess.run(["git", "-C", root, "rev-parse", "--short", "HEAD"], capture_output=True, text=True, check=True).stdout.strip()
            env = dict(os.environ, KEEL_OFFLINE="1", KEEL_ACTOR="claudeOpus5")
            p = subprocess.run([binary, "record", "result", "--file", ".tracking/backlog.sysml", "--task", "dcFirst", "--sha", sha,
                                "--verdict", "pass", "--judged-by", "claudeOpus5", "--judged-at", "2026-09-14", "--evidence", "probe"],
                               cwd=root, env=env, capture_output=True, text=True)
            if p.returncode != 0:
                sys.exit("record result refused:\n%s%s" % (p.stdout, p.stderr))
            git(root, "add", "-A")
            git(root, "commit", "-q", "-m", "the pass")
            write(dec, decision("accepted"))
        o = orient(root, binary)
        ready = o.get("ready", [])
        blocked = o.get("blocked")
        if case == "pos":
            ok = (blocked == [{"item": "dcSecond", "waitsOn": "dcFirst", "why": "not done"}]
                  and "dcFirst" in ready and "dcSecond" not in ready and "storyChartered" not in ready)
        else:
            ok = (blocked == [] and "dcSecond" in ready and "storyChartered" in ready and o.get("answerStatus") == "COMPUTED")
        if not ok:
            print("%s FAILED: ready=%s blocked=%s answerStatus=%s" % (case, ready, blocked, o.get("answerStatus")))
            sys.exit(1)
        print("%s holds: ready=%s blocked=%s" % (case, ready, blocked))
    finally:
        shutil.rmtree(root, ignore_errors=True)


if __name__ == "__main__":
    main()
