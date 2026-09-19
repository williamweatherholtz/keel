---
name: test-verify
description: |
  The D0425 VERIFIER's procedure: run the deliverable's gate set against the tree
  as it is - keel verify . --probe-from PAIR_FILE (the D0476 ladder: validate, guard,
  clippy, the sprint's D0388 probe pair, then suite --touched, in that order, stopping at
  the first red; detached, receipt read only after exit), then RENDER the receipt with
  scripts/verify_receipt.py (D0533): every verdict line is a field read from the receipt
  files, check-engine and sync-claude --check run inside it, DISCREPANCIES is computed,
  and the verifier types nothing but its noted writes. Use when dispatched
  as a verifier subagent, or asked to "verify," "is it green," "run the touched set,"
  or before any deliverable DoD or gate TestResult is recorded. READS and RUNS only:
  writes nothing under .tracking or .engine, records no TestResult (that is the
  recorder's, from this skill's receipt), does NOT build from scratch (use build)
  and does NOT commit (use repo-push).
metadata:
  version: 0.4.0
  domain: [rust, cargo-test, clippy, keel-suite, touched-set, verification, D0425, SysMLv2]
  writePolicy: readOnly
  engine: keel-ai-toolkit
---

# test-verify — the verifier's procedure (D0425 / D0432 / D0438)

A session splits into a PRIMARY that does the substance and a VERIFIER that runs the checks in
its own context (D0425). This skill IS the verifier's procedure. Before it existed the procedure
was retyped into every dispatch prompt (issue469) and a verifier that had it only in prose wrote to
the tree and reported the state from before its own edit (issue471). The primary's dispatch names
this skill, the sprint's probe pair, and nothing else.

Since D0533 the receipt is RENDERED, not typed: `scripts/verify_receipt.py` reads the files the
ladder wrote and prints the receipt; you launch, wait, run the renderer, and add the one line that
is yours. Three verifiers (sprints 676, 745, 750; issue490) typed `DISCREPANCIES: NONE` over a
receipt whose own `LADDER` row said `outcome=fail failed=2`, and one (sprint 726; issue567) typed
`killed` over a stub whose writer was alive. Every one of those lines is now a field read.

## Mandate — read this before any command

| You DO | You do NOT |
|---|---|
| run the commands below, verbatim, against the tree as it is | edit, create or delete ANY file under `.tracking/`, `.engine/`, `keel-cli/`, `.claude/` or `CLAUDE.md` |
| read files, receipts and logs | run `keel record`, `append-result`, `append-gate-result`, `accept`, `new sprint`, `git add/commit/push` |
| report every discrepancy between what was claimed and what a command returned, NAMING the command | "fix" a red by triaging, patching or re-recording — a red is a line in your receipt, and the RECORDER or PRIMARY acts on it |
| let the renderer say `NONE` under discrepancies - it prints that only when its computed list is empty | type, edit or "correct" ANY line of the rendered receipt: the file is the script's stdout; a line that looks wrong is a script defect for the primary to track, never a line for you to fix (D0533) |

A write you believe is owed (an untriaged obligation, a missing `#Resolves` edge, a gate result)
is ONE LINE in your receipt under `VERIFIER-NOTED WRITES`, addressed to the recorder - the label names whose
noticing it is, because a recorder once read `OWED WRITES: NONE` as its own count and wrote nothing
(sprint 735, issue590, D0516). issue471 is what
happens otherwise: a verifier hand-wrote `#Resolves d0437 part obligation... : Issue {` — a
marker on a part, not an edge — the parser skipped both Issues, parser-coverage went red, and the
receipt cited a validate run from before the edit.

## Procedure

Run from the project root. `KEEL` below is the binary named in the dispatch (default
`./target/release/keel.exe`); never `target/release/keel.exe` when a build may run — a copy
(`keel-serve.exe`, `keel-land.exe`) survives a relink.

### 1. Launch the ladder DETACHED, record the launch time

`keel verify . --probe-from PAIR_FILE` is the pre-commit ladder (D0476): `gate validate`,
`gate guard` (from its receipt, D0371), `cargo clippy --release --all-targets -- -D warnings`
(then the same with `--target x86_64-unknown-linux-gnu` - the triple CI lints - on any host that is
not it; a host without that std is a RED rung naming `rustup target add`, D0495/issue572),
the D0388 probe pair read from `--probe-from`, then `suite --touched` - in that order, STOPPING at the
first red, so a lint is reported in under a minute instead of after the twenty-minute run
(sprint705). It writes `.keel/metrics/verify-receipt.toml` naming the rung it stopped at; a rung
after the red is `not-run`, a pair not named is `not-named`. Its last rung is the run the land
will honour:

`keel suite --touched .` is the set the land will run (D0421/D0432): the integration tests whose
text names a changed `keel-cli/src/<stem>.rs`, plus the lib's own unit tests whenever any stem
was contributed (a `keel-cli/src` path, or `init` from `.engine/`). With the lib it takes 7-12 minutes on this host (434 s, 623 s, 713 s,
727 s measured) and a harness foreground call is capped at 600 s — the cap killed one run
(issue469). A binary observed green at the current content by an earlier run is SKIPPED and
named in the receipt's `skipped` (D0474): a rerun over an unchanged tree executes nothing, and that
is a receipt too - `--no-receipt` runs every binary when the dispatch asks for it. So:

```
python -c "import time; print(int(time.time()))" > .keel/metrics/verify-launch.epoch
(nohup KEEL verify . --probe-from "<ABS PAIR FILE>" > .keel/metrics/verify-touched.out 2>&1 < /dev/null & disown)
```

or the harness's `run_in_background` on the same command. Do NOT wait on it in a foreground call
with a timeout; do NOT read the receipt yet.

**No `keel-cli/src` change means an empty touched set** - the run takes seconds, its receipt says
`stems = []`, and that IS the answer. When the dispatch also names **`keel suite .`** (the full
suite, D0356: receipt `.keel/metrics/suite-receipt.toml` with `fingerprint`, `head`, `at`,
`passed`, `failed`, `outcome`, `seconds`, `log`; ~11 minutes here), launch it the same way AFTER the
touched run exits - the two would contend for cargo's lock - and read it under the same rules in
step 5, minus the `stems`/`lib` rows. The two receipts use the same two words the same way
(issue472): `at` is when the file was WRITTEN - the end of a done run, the start of a running stub -
and `seconds` is the run's wall clock, so `at` > launch epoch proves the run finished after you
launched it and `at - seconds` is within two seconds of the launch epoch when it is this run's;
`seconds` belongs in the `SUITE RECEIPT:` row as `ran=<n>s`.
`keel verify` and `keel suite` REFUSE to run from `target/release/keel.exe` (it cannot relink its
own image); run them from the copy the dispatch names.

### 2. While it runs: nothing by hand

The two checks outside the ladder (`KEEL gate check-engine .`, `KEEL sync-claude --check .`), HEAD,
the dirty-path count and the changed paths' stems are all read by the renderer in step 4, so their
lines are copied, not transcribed. Do not run the ladder's own rungs (validate, guard, clippy, the
pair, touched) a second time beside it; the two would contend for cargo's lock. Do not read the
receipt yet.

A guard rung that stopped the ladder is a FAIL the renderer prints with the failing guard's line
from `verify-touched.out` — not a thing to explain away. The dispatch may name guards it EXPECTS
red (a proposed Decision's known red); that expectation is a `--noted` line beside the red, never a
change to the rendered block.

Note `keel gate guard` before staging reads NOTHING for the index-reading guards (`process-change` scans
`git diff --cached`; issue464): say in the receipt that the commit tier was not exercised.

### 3. The D0388 probe pair - a file the primary wrote, named on the ladder's command line

The dispatch names the sprint's PAIR FILE: two lines, the known-positive command then the
known-negative, chosen and written by the primary before the tree was read (D0388/D0500). Each
line is one shell-free command line run from the project root (a `cargo test --release <name>`
filter, a `python scripts/probes/<x>.py --probe`, or a `KEEL <lens>` over a fixture). Pass the
file's absolute path as `--probe-from "<ABS PAIR FILE>"` in step 1 and transcribe NOTHING - do not
read the file into the command line, do not retype its lines into `--probe POS,NEG`: three
dispatches in six sprints retyped the pair wrong (issue571) and each cost an eighty-second climb to
a red no check produced. The ladder refuses the file at parse, exit 2, before any rung runs, if it
is not exactly two non-empty lines; that refusal is a `first red line` for the receipt, not a
verdict on the tree. The rung is green only when both lines exit 0, and its `[[rung]]` row's
command names the file and both lines - copy them into the `PROBE PAIR:` line from the receipt,
never from the dispatch. A ladder that stopped below the probe rung still names the pair in that
row with `verdict = "not-run"`: the receipt line is then `PROBE PAIR: <the row's command> -> not
run (ladder stopped at <rung>)`. Only a dispatch that names no file launches the ladder without
`--probe-from`; the receipt's probe rung then reads `not-named` and the receipt line is
`PROBE PAIR: not named by the dispatch` — never invented, and never written over a named pair.

### 4. Wait, then render — the renderer decides whether the ladder has ended

```
KEEL verify --wait .
python scripts/verify_receipt.py --root . --keel KEEL --out "<ABS RECEIPT PATH>" [--noted "<one line>"]...
```

`KEEL verify --wait .` IS the wait (issue575/D0497): it blocks while the receipt says `running`
and the pid that wrote it is alive, printing one line per rung change, then the finished table.
Run it in a foreground call only when the ladder is already past clippy; otherwise
`run_in_background` it, or call it again - every call reads the receipt fresh.

Then run the renderer. It reads `.keel/metrics/verify-receipt.toml`, `touched-receipt.toml`,
`verify-launch.epoch` and `verify-touched.out`, runs `KEEL gate check-engine .` and
`KEEL sync-claude --check .` itself, reads HEAD, the dirty-path count and the changed paths' stems
by the rule `members/keel-suite/src/touched.rs` uses, and exits:

| Exit | It printed | You do |
|---|---|---|
| `0` | the receipt, green or red - a red is a rendered receipt too | step 6 |
| `2` | `WAIT: ladder in flight (<rung>, <seconds> s)` - the receipt says `running` and its writer (the `pid` the stub names, D0493; else any `keel`/`cargo` process) is alive | call again; a bounded loop is `timeout 540 sh -c 'until python scripts/verify_receipt.py ... ; do [ $? -eq 2 ] || break; sleep 60; done'`, and if it still says WAIT when the call returns, call once more |
| `3` | `KILLED at <rung>: ... verify-launch.epoch=<e>; receipt at=<a>` or `NO RECEIPT` - the stub says `running` and nothing is alive to finish it, or nothing was launched | that line IS the receipt: report it to the primary, do not relaunch, do not read either receipt for a verdict |

The sprint 726 verifier read the stub 17 s after launch and reported a green 718 s ladder as
killed; the renderer asks the process table, which a reader at second seventeen cannot.

### 5. What the renderer computes — read this to know what a line means, not to retype it

Every line below `VERIFIER RECEIPT` is a field, and `DISCREPANCIES` is the list of fields that
disagree with a green claim. `NONE` is printed only when that list is empty.

| Line | Read from | Discrepancy when |
|---|---|---|
| `LADDER: ... outcome= stopped_at= seconds= at= (ok\|STALE)` | `verify-receipt.toml` `outcome`, `stopped_at`, `seconds`, `at`; `verify-launch.epoch` | `outcome != pass`; `at` before the launch epoch (the PREVIOUS run's receipt: sprint 661 was recorded on one 46 minutes stale); `head` != `git rev-parse --short HEAD` |
| `validate= guard= clippy= probe= touched=` | the `[[rung]]` rows | any `fail` (with its `exit` and `command`); any `not-run` (never asked, never passed); a missing row |
| `first red line` | the first `FAIL`/`error`/`keel ...: fail` line of `verify-touched.out` | printed only when the ladder is not green |
| `KEEL gate check-engine . -> <line>; exit=` / `KEEL sync-claude --check . -> ...` | run by the renderer under `--keel` | exit != 0 |
| `PROBE PAIR: <row command> -> <verdict>` | the probe rung's `command` (the file and both lines, D0500) | `not-named` when the dispatch named a file; `not run (ladder stopped at <rung>)` is a statement, not a discrepancy |
| `TOUCHED RECEIPT: outcome= passed= failed= seconds= at= launch= (ok\|STALE)` | `touched-receipt.toml` | `outcome != pass` (with `passed`/`failed`); each `failing` name on its own line; `outcome = running` (the stub); `at` before the launch; `at` AFTER the ladder's `at` - written by a later run (`keel land` did this on 2026-09-19: 736 passed in the file over the ladder's 963), so it is not the ladder's evidence |
| `stems=[...] changed=[...] (MATCH\|MISMATCH) lib= head= log= runner=` | `stems`, `lib`, `head`, `log`, `runner`; `changed` = the module stem of every changed or untracked `keel-cli/src/<stem>.rs` / `members/<crate>/src/<stem>.rs` (`view/mod.rs` -> `view`; `main.rs`/`lib.rs` -> none) plus `init` for any path under `.engine/` (issue530), over `merge-base origin/<branch>` | MISMATCH (the receipt is over another change set); `lib=false` with non-empty stems (the lib run is where pass-alone/fail-together tests show, issue459); `head` != HEAD |
| `long pole (critical path)` | the first `[[timing]]` row | never - it names the slow test (issue536) |
| `test result line` | the ladder's `keel suite --touched: <verdict> - ...` line in `verify-touched.out`, its binary set compacted to a count | never - the verdict is in `TOUCHED RECEIPT` |
| `VERIFIER-NOTED WRITES` | your `--noted` lines, or `NONE` | never - it is your noticing, addressed to the recorder (D0516) |

An empty `stems` with no source change is a receipt too: the renderer prints it and that is the
answer. A `skipped` binary (D0474: observed green at this content by an earlier run) is in the
receipt's `skipped`, never counted as passed by this run.

### 6. The receipt is the file the renderer wrote

`--out "<ABS RECEIPT PATH>"` writes the render to the scratchpad path the dispatch names (never
under the project). Do not open it to edit. The shape it renders:

```
VERIFIER RECEIPT  <date>  head=<sha>  tree=<clean|N dirty paths>
LADDER: KEEL verify . --probe-from <ABS PAIR FILE> -> outcome=<pass|fail> stopped_at=<none|rung> seconds=<n> at=<epoch> (<at> >= <launch>: ok|STALE)
  validate=<verdict> guard=<verdict> clippy=<verdict> probe=<verdict> touched=<verdict>
  first red line: <verbatim from verify-touched.out> | none
KEEL gate check-engine . -> <line>; exit=<n>
KEEL sync-claude --check . -> <line>; exit=<n>
PROBE PAIR: --probe-from <file>: <pos> ; <neg> -> <verdict> | ... -> not run (ladder stopped at <rung>) | not named by the dispatch
TOUCHED RECEIPT: outcome=<..> passed=<n> failed=<n> seconds=<n> at=<epoch> launch=<epoch> (<at> > <launch>: ok|STALE)
  stems=<[...]> changed=<[...]> (MATCH|MISMATCH) lib=<bool> head=<sha> log=<path> runner=<runner>
  long pole (critical path): <binary> "<test>" <millis> millis (<verdict>)
  test result line: "<the ladder's touched verdict line, set compacted>"
DISCREPANCIES: NONE | <one computed line each>
VERIFIER-NOTED WRITES: NONE | <your --noted lines: a write you noticed is owed beyond the dispatch, for the recorder>
```

A line you believe is wrong - a discrepancy you can see the files contradict - is a defect in the
renderer: put it in a `--noted` line and re-render, and the primary tracks it as an Issue. It is
never a reason to type over the file: a typed correction is the class this step exists to end.

The recorder reads THIS file and nothing else; it never reads the primary's description.

## Anti-patterns — each one has happened

1. **Foreground touched run** (issue469): killed at the cap, eleven minutes lost. Detach.
   **Rungs run by hand in an order of your own** (sprint705, D0476): a lint found after the
   twenty-minute run cost a second one. The ladder is one command; it owns the order.
2. **Reading the receipt while `outcome = "running"`** or with `at` before the launch (issue468,
   sprint 661): the previous run's pass over a different change set.
3. **A filter the primary chose instead of the touched set** (issue459/D0432): six tests passed
   alone, the lib run failed one of them.
4. **Writing to the tree** (issue471): the mandate above. A verifier with a fix in mind writes it
   down for the recorder.
5. **Reporting pre-edit state** (issue471): every receipt line is from a command run in this
   dispatch, after the last write the tree saw.
6. **`validate` green read as SysML-conformant** (issue097): it is the engine's authority, not the
   kernel's.
7. **Reading a CI verdict from an exit code** (D0420/issue434): only `gh run list --json conclusion`.
8. **Typing the summary line** (issue490, sprints 676/745/750; issue567, sprint 726): `DISCREPANCIES:
   NONE` over a red `LADDER` row, `killed` over a live writer. The receipt is the renderer's output
   (D0533); the only line you write is `--noted`.

## Questions This Skill Answers

- "Verify this sprint" / "run the verifier" / "is the tree green?"
- "Run the touched set" / "what will the land run?" / "run the ladder" / "keel verify"
- "Is the receipt honest?" (outcome / at / stems / lib)
- "What evidence backs this gate?" — the receipt file, line by line
