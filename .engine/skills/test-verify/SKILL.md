---
name: test-verify
description: |
  The D0425 VERIFIER's procedure: run the deliverable's gate set against the tree
  as it is - keel verify . --probe-from PAIR_FILE (the D0476 ladder: validate, guard,
  clippy, the sprint's D0388 probe pair, then suite --touched, in that order, stopping at
  the first red; detached, receipt read only after exit), plus check-engine and
  sync-claude --check outside it - and report every discrepancy naming the command. Use when dispatched
  as a verifier subagent, or asked to "verify," "is it green," "run the touched set,"
  or before any deliverable DoD or gate TestResult is recorded. READS and RUNS only:
  writes nothing under .tracking or .engine, records no TestResult (that is the
  recorder's, from this skill's receipt), does NOT build from scratch (use build)
  and does NOT commit (use repo-push).
metadata:
  version: 0.3.0
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

## Mandate — read this before any command

| You DO | You do NOT |
|---|---|
| run the commands below, verbatim, against the tree as it is | edit, create or delete ANY file under `.tracking/`, `.engine/`, `keel-cli/`, `.claude/` or `CLAUDE.md` |
| read files, receipts and logs | run `keel record`, `append-result`, `append-gate-result`, `accept`, `new sprint`, `git add/commit/push` |
| report every discrepancy between what was claimed and what a command returned, NAMING the command | "fix" a red by triaging, patching or re-recording — a red is a line in your receipt, and the RECORDER or PRIMARY acts on it |
| say `NONE` under discrepancies only when every check below ran to completion | summarise: the receipt carries the counts and the exact lines, not your reading of them |

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

### 2. Run the two checks outside the ladder while it runs

Each line of the receipt is `<command> -> <the verdict line the command printed>; exit=<code>`.
The ladder's own rungs (validate, guard, clippy, the pair, touched) are read from its receipt in
step 4 - do not run them a second time beside it; the two would contend for cargo's lock.

```
KEEL gate check-engine .              # .engine instance reference resolution
KEEL sync-claude --check .            # the claude-surface-drift check
git rev-parse --short HEAD
git status --short                    # count and list; the recorder needs to know the tree was dirty
```

A guard rung that stopped the ladder is a FAIL to report with the failing guard's line from
`verify-touched.out` — not a thing to explain away. The dispatch may name guards it EXPECTS red (a
proposed Decision's known red); report them as red and cite the dispatch's expectation beside each.

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

### 4. Read the ladder receipt — ONLY after the process exits

```
KEEL verify --wait .
```

That command IS the wait (issue575): it blocks while the receipt says `running` and the pid that
wrote it is alive, printing one line per rung change, then prints the finished table and exits as
the ladder did - 0 green, the red rung's code otherwise. `KILLED during <rung>` with exit 2 is a
ladder that died mid-rung: report that line, not a verdict on either side. Exit 2 with no receipt
means nothing was launched. The sprint 726 verifier read the stub 17 s after launch and reported a
green 718 s ladder as killed; a sentence saying "wait" was the control, and it was skipped. Run this
in a foreground call only when the ladder is already past clippy; otherwise `run_in_background` it
too, or call it again - every call reads the receipt fresh.

`.keel/metrics/verify-receipt.toml` carries `head`, `at`, `seconds`, `outcome`, `stopped_at`
(`none` on green, else the rung) and one `[[rung]]` row per rung (`name`, `verdict` = pass | fail |
not-run | not-named, `exit`, `seconds`, `command`). A `not-run` rung was never asked - report it as
not run, never as passed; a red rung's first error line is in `verify-touched.out`. Clippy is a
rung, so there is no build-lock wait and no `TIMEOUT` to report; a ladder killed before it wrote
has no receipt newer than the launch epoch, and that is the line.

### 5. Read the touched receipt — when the ladder reached it

```
KEEL verify --wait .
```

Step 4's command already returned, so this one returns at once with the same table; run it again
here so the touched receipt is never read before the ladder that writes it has ended (sprint 661,
sprint 726). When its table says `stopped_at` is `none` or `touched`, read
`.keel/metrics/touched-receipt.toml` and check, in this order, each as its own receipt line:

| Check | Honest when | Why (issue468 / D0387) |
|---|---|---|
| `outcome` | is not `"running"` | the stub written at launch says `running`; a killed run leaves it |
| `at` | > the epoch in `verify-launch.epoch` | otherwise this is the PREVIOUS run's receipt; sprint 661 was recorded on one 46 minutes stale |
| `stems` | == the sorted set of module stems for every changed `keel-cli/src/<stem>.rs` or `members/<crate>/src/<stem>.rs` (`git diff --name-only origin/main -- keel-cli/src members` plus untracked; `view/mod.rs` -> `view`, D0479), PLUS `init` when any path under `.engine/` changed or is untracked (`git status --short -- .engine`): that tree is compiled into the binary, so `members/keel-suite/src/touched.rs` `embedded_stem` attributes it to the tests that run `keel init` (issue530) | the receipt must be over THIS change set |
| `lib` | `true` whenever `stems` is non-empty - so also when the only stem is `init` from a changed `.engine/` path - or a changed source no member owns, or `keel-cli` itself is a touched member (`lib` at touched.rs:575; sprint 744's verifier read the old row, "any `keel-cli/src` path", as the whole rule and filed a discrepancy against an honest receipt) | the lib run is where pass-alone/fail-together tests show (issue459) |
| `passed` / `failed` / `failing` | copied verbatim | the recorder's `--evidence` quotes these |
| `ran` / `skipped` | together they are the set; `skipped` names only binaries with an `[[observed]]` row at `code_key` (and `tree_key` when in `self_reading`) | a skipped binary was observed green at this content by an earlier run (D0474) - report it as skipped, never as passed by this run |
| `head` | == `git rev-parse --short HEAD` | |
| `runner` | `cargo-nextest <version>` - or `cargo-test (nextest not installed)`, which is reported as such | D0475: the set runs under nextest, binaries in parallel; the fallback is serial and carries no timings |
| `[[timing]]` | one row per test nextest ran, slowest first (`binary`, `test`, `millis`, `verdict`) | the first row is the run's critical path (issue536); quote it in the receipt so a slow set names its long pole |

An empty `stems` with no `keel-cli/src` change is a receipt too: report it as such and run
nothing more.

### 6. Write the receipt to the path the dispatch names

Plain text, this shape, in the scratchpad path the dispatch gives (never under the project):

```
VERIFIER RECEIPT  <date>  head=<sha>  tree=<clean|N dirty paths>
LADDER: KEEL verify . --probe-from <ABS PAIR FILE> -> outcome=<pass|fail> stopped_at=<none|rung> seconds=<n> at=<epoch> (<at>launch: ok|STALE)
  validate=<verdict> guard=<verdict> clippy=<verdict> probe=<verdict> touched=<verdict>   (from its [[rung]] rows)
  first red line: "<verbatim from verify-touched.out>" | none
KEEL gate check-engine . -> <line>; exit=<n>
KEEL sync-claude --check . -> <line>; exit=<n>
PROBE PAIR: <check> -> positive <case>: <outcome>; negative <case>: <outcome> | <row command> -> not run (ladder stopped at <rung>) | not named by the dispatch
TOUCHED RECEIPT: outcome=<..> passed=<n> failed=<n> seconds=<n> at=<epoch> launch=<epoch> (<at>launch: ok|STALE)
  stems=<[...]> changed=<[...]> (MATCH|MISMATCH) lib=<bool> head=<sha> log=<path>
  test result line: "<verbatim from verify-touched.out>"
DISCREPANCIES: NONE | <one line each, naming the command>
VERIFIER-NOTED WRITES: NONE | <one line each: a write you noticed is owed beyond the dispatch, for the recorder>
```

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

## Questions This Skill Answers

- "Verify this sprint" / "run the verifier" / "is the tree green?"
- "Run the touched set" / "what will the land run?" / "run the ladder" / "keel verify"
- "Is the receipt honest?" (outcome / at / stems / lib)
- "What evidence backs this gate?" — the receipt file, line by line
