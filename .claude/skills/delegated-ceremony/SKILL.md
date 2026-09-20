---
name: delegated-ceremony
description: |
  The D0425 ceremony split as a deployed process: the PRIMARY does the substance, a
  VERIFIER subagent runs the checks and writes one receipt, a RECORDER subagent writes
  the ceremony through the keel write API from that receipt ONLY. Use when a sprint's
  gate results, DoD results or sprint record are owed and the session can spawn
  subagents; dispatch by pointing at this skill, never by retyping a brief. Carries
  the two dispatch briefs verbatim and the recorder's report check
  (references/check_report.py). Deploys .engine/processes/delegated-ceremony.sysml.
metadata:
  version: 0.1.0
  domain: [D0425, subagents, verifier, recorder, ceremony, write-api, keel, SysMLv2]
  writePolicy: direct
  engine: keel-ai-toolkit
---

# delegated-ceremony — the verifier and recorder briefs (D0425)

Deploys `.engine/processes/delegated-ceremony.sysml`. Before this skill existed the recorder ran on
a brief written for the occasion. Sprint 647's did three things this skill makes structurally hard:
it typed a `#Addresses` marker no schema declares (the Stop hook went red twice; obligationf1702eed),
it hand-deleted three TestResult parts it had recorded without `--evidence` and re-recorded them,
and it reported "All issues resolved ... ready for commit" over a tree whose guard was FAILED.

## The three roles, by what each may read and write

| Role | Agent type | Reads | Writes |
|---|---|---|---|
| PRIMARY | the session | everything | the substance: code, design, analysis; the dispatches below |
| VERIFIER | `verifier` (haiku) | the tree; the commands' output (`keel verify . --probe-from <PAIR FILE>` - the D0476 ladder: validate, guard, clippy, the probe pair, suite --touched, stopping at the first red - plus check-engine and sync-claude --check) | ONE receipt file in the scratchpad. Nothing under `.tracking/`, `.engine/`, `keel-cli/`, `.claude/`, `CLAUDE.md`; no `keel record`; no git |
| RECORDER | `recorder` (haiku) | the verifier's receipt file ONLY - never the primary's account of the work | `keel record ...` calls ONLY. No Write/Edit on any tree file (the agent has neither tool); no git |

The agent types are `.claude/agents/verifier.md` and `.claude/agents/recorder.md`. They carry the
role into the SubagentStop hook's payload (`agent_type`), which is how a recorder that leaves a red
tree becomes a `recorder:tree-red` refusal in the fire ledger, and a verifier that writes the tree a
`verifier:tree-written` one (process step dcyRedTreeIsARefusal). The hook measures each agent from
the tree at its OWN start (D0501), and a verifier is NEVER blocked (D0502): a red the primary left is
the primary's, read at the turn gate, and the verifier's only answer to it is the receipt.

## Verifier brief — dispatch verbatim, filling the four slots

```
You are the D0425 VERIFIER. Follow the test-verify skill (.claude/skills/test-verify/SKILL.md)
verbatim - it is your whole procedure. Binary: <KEEL, a copy such as ./target/release/keel-serve.exe>.
Project root: <ABS ROOT>. D0388 probe pair for this sprint: the pair file at <ABS PAIR FILE> - two
lines, the known-positive command then the known-negative, written by the primary; pass its path as
`--probe-from <ABS PAIR FILE>` and transcribe nothing (if no file is named, launch without it and say
PROBE PAIR: not named by the dispatch). The receipt is RENDERED (D0533): after `KEEL verify --wait .`
run `python scripts/verify_receipt.py --root . --keel <KEEL> --out <ABS SCRATCH PATH>/verifier-receipt.txt`
until it exits 0 or 3 (exit 2 = WAIT, call again), adding `--noted "<line>"` only for a write you noticed is
owed; never open or edit the file it wrote. Return only its path and its DISCREPANCIES line. You write
nothing else anywhere.
```

The primary fills `<KEEL>`, `<ABS ROOT>`, the pair file's path and the scratch path. The pair file
is the primary's (D0500, issue571): it chose the check and both cases before reading the tree
(D0388) and wrote them as the file's two lines; the verifier NAMES the path and retypes nothing,
because three dispatches in six sprints retyped the pair wrong into `--probe POS,NEG` and each
cost an eighty-second climb to a red no check produced. The primary adds no description of what it
believes passed: a verifier that inherits the primary's belief is the testimony problem (issue266)
in another form.

## Recorder brief — dispatch verbatim, filling the slots

```
You are the D0425 RECORDER. Your ONLY input is the verifier's receipt at <ABS RECEIPT PATH>; read it
first and do not read or trust any other account of the work. Binary: <KEEL>. Project root: <ABS ROOT>.
Records owed: <for each: record gate-result --file F --gate G | record result --file F --task T>, all
with --sha <SHA> --judged-at <DATE> and --evidence quoting the receipt's own lines for that gate or
task. Rules: (1) every write is a sub-verb of keel record, run from the project root; you have no Write or
Edit tool and you touch no file under .tracking, .engine, keel-cli or .claude by any other means;
(2) a receipt whose DISCREPANCIES line is not NONE records the reds it names - never a pass over
them; (3) a result already on the tree without --evidence is a line under DISCREPANCIES in your
report, never something you delete or re-record; (4) a marker, edge or field you believe is needed
and the write API does not offer is a line under REFUSED in your report - never a line you type
(record sprint --fill is where a record's edges come from; the primary triages an obligation);
(5) one write per owed record: a red `gate --fast` after your write is a line under DISCREPANCIES naming
the guard, never a second record of the same gate or task with reworded evidence, and never a tool
(textpatch, sed, python, an editor) run against the file - a WROTE line naming one is refused;
(6) every owed record is accounted for: <COUNT> records are owed, and your report carries one WROTE line
or one REFUSED line for each - a report that accounts for fewer is refused, and a report with no WROTE
line and no REFUSED line is refused whatever the count (sprint 723's first recorder wrote nothing and
reported NONE). The dispatch's <COUNT> is the ONLY owed count: the receipt's VERIFIER-NOTED WRITES line
is the verifier's noticing addressed to you, never your count - a REFUSED line that cites it as the
reason a record was not written is refused (sprint 735's second recorder read OWED WRITES: NONE as
"no writes owed" and wrote nothing, D0516); (7) a write that lands after a refused attempt is a WROTE
line naming the command that landed, never a REFUSED line for the attempt that did not - the check reads
your WROTE lines against the TestResult parts the tree gained since HEAD and refuses a report claiming
fewer (sprint 741's recorder landed a DoD result on a retry and filed the first attempt as REFUSED, D0537).
Report shape, written to <ABS SCRATCH PATH>/recorder-report.txt:
  RECORDER REPORT  <date>  receipt=<path>  sprint=<file>
  WROTE: <the exact keel record command> -> <the verdict line it printed>      (one per write)
  REFUSED: <what and why> | (omit when none)
  DISCREPANCIES: NONE | <one line each, naming the receipt line or command>
  <the verbatim last line of `KEEL gate --fast .` run AFTER your last write - the report's last line>
Before returning, run: python <ABS ROOT>/.claude/skills/delegated-ceremony/references/check_report.py
<ABS SCRATCH PATH>/recorder-report.txt --root <ABS ROOT> --owed <COUNT>. If it refuses, fix the report or
the write it names and run it again; return the report only when it passes, and return its text verbatim.
```

The primary fills `<COUNT>` with the number of records it listed under "Records owed" - the count is
the control (D0492); the list alone was the reminder sprint 723's first recorder ignored.

## What the check refuses (references/check_report.py)

1. an undeclared marker anywhere in the report (`#Addresses`; the marker-vocabulary guard's own
   vocabulary, read line-anchored from `metadata def X` under `.engine` and `.tracking`);
2. a typed edge line - any line whose first token is a `#Marker`, declared or not;
3. a last line that is not `keel gate --fast .`'s (`gate: fast gate clean ...` / `gate: FAST GATE
   FAILED ...`), or a FAILED last line beside `DISCREPANCIES: NONE`;
4. a `WROTE:` line whose command is not `[keel] record <sub-verb>` - sprint 703's recorder ran
   `scripts/textpatch.py` against the sprint file and reported it as a write (issue532, D0473);
5. two `WROTE:` lines naming one `--gate` or one `--task` - sprint 703's recorder recorded a red retro
   gate three more times with reworded evidence; a red after a write is a DISCREPANCIES line;
6. zero `WROTE:` lines and no `REFUSED:` line, with or without `--owed` - sprint 723's first recorder ran
   no record command and returned three lines reading DISCREPANCIES: NONE (issue568, D0492);
7. under `--owed N`, `WROTE:` plus `REFUSED:` lines numbering fewer than N - the refusal names the
   shortfall (`owed 7, accounted 6`);
8. a `REFUSED:` line that cites the receipt's `VERIFIER-NOTED WRITES` line (or its old label `OWED
   WRITES`) as its reason - sprint 735's second recorder, dispatched `--owed 1`, wrote nothing and
   reported `REFUSED: ... receipt line 19 states OWED WRITES: NONE; no writes owed per verifier`, and
   refusal 7 counted the REFUSED line (issue590, D0516). That receipt line lists writes the verifier
   noticed for the recorder; the dispatch's count is the only owed count;
9. under `--root`, `WROTE:` lines numbering fewer than the `part <name> : TestResult` lines the working
   tree gained since HEAD under `.tracking` (`git diff HEAD -U0 -- .tracking` plus untracked `.tracking`
   files) - the refusal names each gained result no `WROTE:` key claims (`--gate G` claims `G R<n>`,
   `--task T` claims `T R<n>` / `T DoDR<n>`). Sprint 741's recorder was refused by the write API on the
   Test's name, retried with the action's, landed `dcTouchedSetDescendsTheWorkspaceDoDR1`, and reported it
   as `REFUSED:`; seven WROTE plus one REFUSED met `--owed 8` while the tree held eight (issue602, D0537).
   The eight refusals above read the report alone; this one reads it against the tree it describes.

`python check_report.py --probe --root <ROOT>` runs the D0388 pairs: `fixtures/positive-undeclared-marker.txt`
(sprint 647's third act, refused) and `fixtures/negative-sprint647-receipt-driven.txt` (the same ceremony
as it should have been reported; passes); `fixtures/positive-sprint703-textpatch-write.txt` (sprint 703's
report as returned, refused naming the textpatch line), `fixtures/positive-sprint703-retro-recorded-twice.txt`
(the retro gate written twice, refused naming both lines) and `fixtures/negative-sprint703-record-only.txt`
(the same report with the textpatch line removed; passes); `fixtures/positive-sprint723-nothing-written.txt`
(sprint 723's first report as returned, refused naming zero writes), `fixtures/negative-sprint723-seven-owed.txt`
(the second recorder's seven writes under `--owed 7`; passes) and `fixtures/positive-sprint723-six-of-seven.txt`
(one gate line removed under `--owed 7`; refused naming the shortfall);
`fixtures/positive-sprint735-refused-citing-owed-writes.txt` (sprint 735's second recorder's first report
under `--owed 1`; refused naming the OWED WRITES citation) and `fixtures/negative-sprint735-one-owed.txt`
(its rewritten report, one WROTE line under `--owed 1`; passes);
`fixtures/positive-sprint741-landed-write-filed-refused.txt` (sprint 741's report as returned - seven WROTE,
one REFUSED - judged over the eight results that tree gained; refused under `--owed 8` naming
`dcTouchedSetDescendsTheWorkspaceDoDR1`) and `fixtures/negative-sprint741-eight-written.txt` (its corrected
form, eight WROTE over the same eight; passes). `--probe <FIXTURE>` runs one row and
exits 0 when that side holds - the form each line of a pair file takes, since the ladder needs both sides
to exit 0 (sprint 724's first dispatch named the bare checker runs, and the positive's exit 1 stopped the
ladder at the probe rung).

## What the primary does with the report

Reads it back from the tree, not from the report: `KEEL show verification . --pending`, the sprint
file, `KEEL gate guard --no-receipt .`. A `REFUSED:` line is the primary's write to make (a `--fill`
edge, a DoD sentence naming an obligation) - or its decision not to. A `recorder:tree-red` line in
`.keel/metrics/hooks.jsonl` means a recorder left the tree red; a `verifier:tree-written` line means
a verifier wrote under the project (issue578). The census counts both
(`KEEL show control-census .`), and a rising count is this process's own trigger to harden.

## Anti-patterns - each one has happened

1. Retyping the procedure into the dispatch (issue469) - point at the skill.
2. Telling the recorder what passed - it reads the receipt, or it reads nothing.
3. A recorder "fixing" a red by editing the tree (issue471, sprint 647) - a red is a report line.
4. Returning a summary whose last line is prose - the gate's line is the last line, always.

## Questions this skill answers

- "Record the ceremony" / "run the verifier and recorder" / "delegate the gate results"
- "What may the recorder write?" - keel record verbs, nothing else
- "Why did the Stop hook go red after a subagent?" - `recorder:tree-red` in the ledger, or not
- "Did the verifier touch the tree?" - a `verifier:tree-written` line in the ledger, or not (never a block)
