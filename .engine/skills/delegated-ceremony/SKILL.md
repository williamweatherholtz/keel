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
| VERIFIER | `verifier` (haiku) | the tree; the commands' output (validate, check-engine, guard, sync-claude --check, suite --touched, clippy, the probe pair) | ONE receipt file in the scratchpad. Nothing under `.tracking/`, `.engine/`, `keel-cli/`, `.claude/`, `CLAUDE.md`; no `keel record`; no git |
| RECORDER | `recorder` (haiku) | the verifier's receipt file ONLY - never the primary's account of the work | `keel record ...` calls ONLY. No Write/Edit on any tree file (the agent has neither tool); no git |

The agent types are `.claude/agents/verifier.md` and `.claude/agents/recorder.md`. They carry the
role into the SubagentStop hook's payload (`agent_type`), which is how a recorder that leaves a red
tree becomes a `recorder:tree-red` refusal in the fire ledger (process step dcyRedTreeIsARefusal).

## Verifier brief — dispatch verbatim, filling the four slots

```
You are the D0425 VERIFIER. Follow the test-verify skill (.claude/skills/test-verify/SKILL.md)
verbatim - it is your whole procedure. Binary: <KEEL, a copy such as ./target/release/keel-serve.exe>.
Project root: <ABS ROOT>. D0388 probe pair for this sprint: <CHECK> - known-positive <CASE>,
known-negative <CASE> (run exactly this; if none is named say PROBE PAIR: not named by the dispatch).
Write the receipt to <ABS SCRATCH PATH>/verifier-receipt.txt in the skill's shape and return only
its path and its DISCREPANCIES line. You write nothing else anywhere.
```

The primary fills `<KEEL>`, `<ABS ROOT>`, the pair and the scratch path. It adds no description of
what it believes passed: a verifier that inherits the primary's belief is the testimony problem
(issue266) in another form.

## Recorder brief — dispatch verbatim, filling the slots

```
You are the D0425 RECORDER. Your ONLY input is the verifier's receipt at <ABS RECEIPT PATH>; read it
first and do not read or trust any other account of the work. Binary: <KEEL>. Project root: <ABS ROOT>.
Records owed: <for each: record gate-result --file F --gate G | record result --file F --task T>, all
with --sha <SHA> --judged-at <DATE> and --evidence quoting the receipt's own lines for that gate or
task. Rules: (1) every write is a keel record verb run from the project root; you have no Write or
Edit tool and you touch no file under .tracking, .engine, keel-cli or .claude by any other means;
(2) a receipt whose DISCREPANCIES line is not NONE records the reds it names - never a pass over
them; (3) a result already on the tree without --evidence is a line under DISCREPANCIES in your
report, never something you delete or re-record; (4) a marker, edge or field you believe is needed
and the write API does not offer is a line under REFUSED in your report - never a line you type
(record sprint --fill is where a record's edges come from; the primary triages an obligation).
Report shape, written to <ABS SCRATCH PATH>/recorder-report.txt:
  RECORDER REPORT  <date>  receipt=<path>  sprint=<file>
  WROTE: <the exact keel record command> -> <the verdict line it printed>      (one per write)
  REFUSED: <what and why> | (omit when none)
  DISCREPANCIES: NONE | <one line each, naming the receipt line or command>
  <the verbatim last line of `KEEL gate --fast .` run AFTER your last write - the report's last line>
Before returning, run: python <ABS ROOT>/.claude/skills/delegated-ceremony/references/check_report.py
<ABS SCRATCH PATH>/recorder-report.txt --root <ABS ROOT>. If it refuses, fix the report or the write
it names and run it again; return the report only when it passes, and return its text verbatim.
```

## What the check refuses (references/check_report.py)

1. an undeclared marker anywhere in the report (`#Addresses`; the marker-vocabulary guard's own
   vocabulary, read line-anchored from `metadata def X` under `.engine` and `.tracking`);
2. a typed edge line - any line whose first token is a `#Marker`, declared or not;
3. a last line that is not `keel gate --fast .`'s (`gate: fast gate clean ...` / `gate: FAST GATE
   FAILED ...`), or a FAILED last line beside `DISCREPANCIES: NONE`.

`python check_report.py --probe --root <ROOT>` runs the D0388 pair: `fixtures/positive-undeclared-marker.txt`
(sprint 647's third act, refused) and `fixtures/negative-sprint647-receipt-driven.txt` (the same ceremony
as it should have been reported; passes).

## What the primary does with the report

Reads it back from the tree, not from the report: `KEEL show verification . --pending`, the sprint
file, `KEEL gate guard --no-receipt .`. A `REFUSED:` line is the primary's write to make (a `--fill`
edge, a DoD sentence naming an obligation) - or its decision not to. A `recorder:tree-red` line in
`.keel/metrics/hooks.jsonl` means a recorder left the tree red: the census counts them
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
