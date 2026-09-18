# CLAUDE.md — how to work here

**keel**: work-tracking engine. Truth = plain-text SysML v2 in git. State = COMPUTED, never stored.
Built with its own discipline. Two models, never conflated: **engine** (tracks the work) vs
**deliverable** (what the work produces). Deliverable vocabulary never enters the engine.

- `.engine/` — the engine: schema, workflows, processes, skills, rules, decisions. Committed.
- `.tracking/` — this project's instance data (needs, requirements, work, decisions, results). Committed.

Response contract lives in `.claude/output-styles/keel.md` (D0130), not here (D0105: one home per fact).
Every rule below names its Decision; the Decision holds the history. Don't restate history here.

---

## 1. Invariants

1. **Text is truth; anything derivable is a view.** Author only irreducible facts: atomic items, typed
   edges, test results, judgments. Never author a document, matrix, baseline, ICD, BOM, report, status
   doc, handoff note (D0018). Materialized views: `#View`, regenerable.
2. **Atomic items, typed edges.** `:>`, `satisfy`, `verify`, `allocate`, `dependency`, `supersede`. No blobs.
3. **Identity = immutable UUID `id`.** `title` may duplicate; `displayLabel` computed.
4. **"We won't do X" is a Decision.** `#Supersede` retires its target whole; `#SupersedeClause` reverses one
   clause and leaves the target in force. Never both on one target (D0398). No `superseded` status member.
5. **`schema/core` is frozen.** Schema/process changes = Change Request + human sign-off.
6. **Reference procedure, don't embed it.** `ready`/`blocked`/`done`/order are computed. Phase gate = its
   `verify`-linked Tests passing.
7. **Constraint / requirement / indicator (D0088).** Constraint = executable predicate (a guard).
   Requirement = constraint elevated to stakeholder contract (Need/SysReq + satisfy/verify). Indicator =
   monitored, no threshold (`keel show indicators`). No defensible boundary → stays indicator.
8. **CLI surface is an authored fact (D0271).** Every command/lens is a `CliCommand` in
   `.engine/cli/commands.sysml` (family, effect, stability). `--help` renders from it; guard
   `cli-surface-declared` holds facts = help = dispatch (invocation included). A fact another actor owns is
   retired by a `#Supersede` edge beside it in that file, never deleted and re-authored (D0108, issue547). Never author an ICD.
9. **CLI/JSON is authority; HTML is the human's lens (D0093).** HTML stores nothing.

---

## 2. Orient — never from prose

```
keel show orient .                 # in-progress + ready/blocked/suspect frontier + burndown
keel show whats-next .             # ready list; declaration order IS priority (D0052)
keel show priority .          # priority metric + inversions (D0311)
keel show status .                 # engine pin, drift, model, work, hooks, CI
keel advance <sprint|process> [--to <step>]   # process cursor; refused while an earlier bound step is red (D0435/D0436)
```

Auto-follow the ranked frontier (D0052). Don't ask which item. Pause only for a content gate
(frozen schema, direction Decision) or an empty frontier.

Lenses: `verification` (EXAMINED vs EXERCISED, `--pending`), `suspect`, `orphans`, `view <name>`, `audit`,
`coverage`, `tier-satisfaction`, `rootedness`, `dispositions`, `sitting-coverage`, `concern-coverage`,
`commit-delta [--range A..B]` (D0282), `governing-version`, `open-issues`, `indicators` (+`triggered`,
D0333), `intake`, `control-structure [--svg]` (D0284/D0285; analyse with skill `stpa-self`, D0313/D0428),
`attestation` (D0232/D0312: receipt vs testimony; `proposed` = AI-examined pass awaiting a human),
`controls` (D0195/D0298), `control-census` (D0426: every control by whose act it binds; `removalCandidates`
= friction/hypothetical; `ucas` D0428), `why <term>`, `knowledge question-coverage` (D0161), `hardening`
(D0169/D0434; `stepTrigger` = each adopted step hook / gate / memory, us102), `authority-queue` (what waits for the human). Reports: `keel render report
<assurance|traceability|quality-debt|flow|governance|friction> [--html] [--trend]`. Any view:
`keel render <view> --mode graph|table|review`; a review round-trips via `keel record review`.

---

## 3. Route every request

| Route | When | Do |
|---|---|---|
| `CHANGE` | workflow / gate / schema / rule / meaning of a view | CR: state change + rationale → human acceptance → apply → validate → `Decision` → commit `CR:` |
| `EXECUTE` | produces the active phase's artifact | orient → act in phase → record items + edges + judgment → gate passes |
| `RECORD` | one atomic fact (Decision / TestResult / Issue) | author + provenance. Never a blob |
| `VIEW` | computed answer | compute, present. Never store |
| `ORIENT` | where things stand | `keel show orient` |
| `TRIVIAL` | typo, one rename, one doc line | do it, label it |

Split multi-part requests. No process fits → DEFINE the process (that is the output).
Recurring with no skill → create the skill first (D0040). Every `Issue` gets a `#Resolves` edge at record
time; resolution is computed. Six workflows: Business → Architecture → Delivery → Deploy → Operate;
Change Request cross-cuts and is itself frozen.

---

## 4. Working rules

**Writes**
- Write API only; direct edits for what it doesn't cover. Writes are atomic + serialised on
  `.keel-write-lock`; a lock miss fails loudly (issue184/185).
- Prose goes through a FILE, never a double-quoted shell arg — the shell executes backticks into the record
  (D0224, five times). `record decision --from F`, `record issue --description-from F`, `record task --dod-from F`,
  `record sprint N slug --charter dNNNN --points P --fill F` (D0301: writes no result). Nine authoring verbs are `record` sub-verbs (D0451).
- Results: `record result` / `record gate-result --evidence-from F` (PREFER for anything quoting a command;
  `--evidence "<one line>"` for a bare receipt; D0224/issue543). AI `method=test` with no `// RAN:`
  receipt is refused at the write (D0232/D0424). `ci-run id=<id> workflow=<name>` is verified by CI (D0323).
  A demo receipt that IS a command under `[demo] replayable` (`.engine/contracts/reverify.toml`) stays a pass
  and `keel record reverify --demos` re-runs it (D0444). Other AI-examined passes land `proposed` (D0312 B);
  `keel judge-set` is the human's judgment.
- `record statement` / `record story`: human words VERBATIM, then the story with `#DerivedFrom` (D0236).
  Elicit pain, not features; never offer a menu (D0216). A Need with no `#DerivedFrom` says it is my judgment.
- GitHub intake (`github pull`/`github ingest`, skill `github-intake`; D0453 router): private repo → `trusted`, act;
  public → `untrusted`, plan only, human accepts first; undetermined fails closed (D0263/D0264/D0314).
  `currency` = the unattended pass (D0338).
- Provenance never defaulted: actor AND date (D0129/issue182). `keel actor set <id>` or `KEEL_ACTOR`.
  AI = `Actor { kind = ActorKind::ai }`, never a `Person`.
- Owner edits own items (`createdBy`); others ADD or SUPERSEDE (D0108). Editing a done task's DoD makes it
  SUSPECT (D0307). Skills: `distributed-collaboration`, `actor-enrollment`.

**Decisions**
- One clause per Decision; layers = several Decisions with `#DependsOn` (D0303).
- `record decision` auto-accepts a non-fork under standing consent (D0207/D0291) — EXCEPT a
  `process-change`/`safety-change` marker OR text naming those words (D0337/D0439): HELD proposed for the
  human. Say `NOT A PROCESS CHANGE: <why>` for a mention in passing. Prose weighing alternatives without the
  OPTION marker is held as a fork (D0322) — write it as a fork or say `NOT A FORK: <why>`.
- Forks go through skill `decision-surfacing` (D0269/D0359): one published brief, republished to one URL
  when the pending set CHANGES; silent otherwise. `hook stop` is SILENT when green.
- `--supersedes` / `--supersedes-clause` / `--derived-from` author the edge WITH the Decision (D0352).
- Every schema/process/enforcement change (guards, `.githooks/`, CI yml) needs a co-committed
  `#ProspectiveChange`/`#SafetyChange` Decision (D0209 cl.2). Commit prefix `CR:`. Doc-sync rides every
  change, same commit.

**Acceptance (human only)**
- `method=confirmation` = a human's word on THAT claim. Never inferred. Confirm only what tests can't (D0051).
- Chat acceptance: `keel accept <d> --words "<verbatim>" --by <person>` (D0192/D0289). Quote exactly. A
  read-back miss or <10 chars = WARN, not refusal (D0423). Gesture citations are written by the surface that
  saw the gesture, never typed (D0411). Acceptance binds to TEXT; later edit → `accept --rebind` in the same
  commit (D0308/D0329).
- Governance binds the AI, not the human: no schedules, debts or reviews owed by them (D0204). Sprint
  closeOut/retro are AI-recorded (D0049).

**Corrections**
- Recurrable defect → tracked `Issue` + automated control, never a reminder (D0047). Retro findings → tracked
  items or the retro says why not (D0131).
- A check is probed before its answer is stated: one known-positive, one known-negative, chosen before the
  real tree is read; a KEPT check carries them as `--probe` (D0388).
- Adoption is declared in `.engine/contracts/activation.toml` (D0138/D0164); `keel activate|deactivate`.
- Two migrations: `migration` (own data, expand/migrate/contract, D0067) vs `project-migration` (engine
  moved under a project, D0275/D0336; `keel migrate` reverts any red). The engine cannot migrate itself.
- Several projects per repo (D0234): hook at root, `keel gate --workspace`, `keel projects`.
- Authoring friction is the #1 risk (D0054).

**Subagents (D0425)**
- Primary does substance. Process `delegated-ceremony` (skill of the same name carries both briefs; dispatch by
  pointing at it, agent types `verifier`/`recorder` in `.claude/agents/`). VERIFIER (haiku, skill `test-verify`,
  D0438) runs `keel verify . --probe-from PAIR_FILE` DETACHED (the D0476 ladder: validate, guard, clippy, the D0388
  pair - two lines the PRIMARY wrote, positive then negative, the verifier names the path and retypes nothing (D0500) -
  `suite --touched`, stopping at the first red), plus `gate check-engine`, `sync-claude --check`; writes a receipt only. RECORDER (haiku, no Write/Edit tool) writes ceremony through
  `keel record` from that receipt ONLY; a marker/edge the API lacks is a `REFUSED:` report line, never typed; its
  report ends with the last line of `keel gate --fast .`. `references/check_report.py` refuses a report otherwise,
  one whose `WROTE:` names a non-`record` command or writes one gate/task twice (D0473), and one accounting for nothing
  or, under `--owed N` (the count the dispatch named), for fewer than N (D0492), and one whose `REFUSED:` cites the
  receipt's `VERIFIER-NOTED WRITES` line - the verifier's noticing, never the recorder's count (D0516): a red after a write is
  a `DISCREPANCIES:` line. A recorder that leaves the tree red is ledgered `recorder:tree-red`; a verifier that writes
  the tree is ledgered `verifier:tree-written`, never blocked (D0501/D0502: each subagent is measured from its own start). Neither reads the
  other's conclusion as fact.

**Git**
- `main` only, commit directly. Never rebase/squash/force-push (D0129): rewriting history orphans
  `judgedAgainst` evidence. Integrate by merge: `keel sync` / `keel land` (gates workspace-wide before push).
- CI verdict = `conclusion` field (`gh run list --json conclusion` or `keel show status`), never a wrapper's exit
  (D0420). CI runs `audit adherence`, `audit ci-runs`, `audit history`.
- `land` runs the touched test set before the first push (D0421) inside the post-commit hook from a copy
  `target/release/keel-land.exe` (D0422). A binary observed green at the current content is skipped (D0474: keyed
  on the code, plus the tree for a test that reads this repo; `--no-receipt` runs all). Member unit tests are rows
  `lib:<member>` over the Cargo graph (owner of a changed source + dependents), keyed on that scope (D0481); a test
  that reads the repo ONLY via `keel_fs::test_support::{repo_root, repo_path}` is keyed on the paths it recorded -
  never `CARGO_MANIFEST_DIR`/`..` in an integration test. The set runs under cargo-nextest
  (D0475; pin `https://get.nexte.st/0.9.144/<platform>`, the one CI installs), the three cucumber `harness = false` binaries
  under `cargo test`; no nextest = the whole set serial, and the receipt's `runner` says so. `[[timing]]` rows name the
  long pole (issue536). A full set takes 7–15 min: run `git commit` DETACHED, then read
  `git status -sb` and the CI conclusion (issue453). Receipts (`.keel/metrics/*-receipt.toml`) say `running`
  while cargo runs; read `at`/`stems`/`head` after exit (D0387). The stub names its writer's `pid`: a second
  `suite --touched` / `verify` launch is REFUSED (exit 2, nothing written) while that pid is alive; a dead
  writer's stub is replaced (D0493). A changed path under `.engine/` contributes the stem `init` (issue530).
- `keel suite` gates nothing; it writes a receipt (D0356).

---

## 5. Validate — every `.sysml` change

```
keel gate validate .            # .tracking authority (kernel-free)
keel gate check-engine .        # .engine instance gate
keel gate guard [--no-receipt]  # all forward guards; catalogue .engine/docs/guards.md; count from `keel version`
keel verify [ROOT] [--probe POS,NEG | --probe-from FILE] [--no-receipt]  # the pre-commit ladder in cost order, stops at the first red (D0476); FILE = two lines, positive then negative (D0500)
keel verify --wait [ROOT]       # block on a launched ladder's receipt while its pid lives; table + exit code, or KILLED during <rung> (exit 2)
keel gate --fast           # per-edit tier
keel gate --workspace      # commit tier, multi-project
keel show enforcement-report    # hook fires, blocks, latency distribution, refusals (D0389/D0424)
keel record reverify --all-drift  # re-run gate at HEAD, fresh results on green (D0101)
keel record reverify --demos      # re-run replayable demo receipts (D0444)
```

- Green guard answers from `.keel/metrics/guard-receipt.toml` when inputs are equal (D0371); red deletes it.
- Honest-state gates, not completeness gates (D0098). Never fake a pass; never block true state.
- `gate validate` is NOT SysML conformance (issue097). Kernel-check a new construct first:
  `conda run -n sysml --no-capture-output python .engine/tools/validate/conformance_lane.py --construct f.sysml > out.txt 2>&1`
  Same tool bare = conformance lane; tracked as `conformanceIndicator`, never gated (D0132).
- Schema/workflow changes: `.engine/tools/validate/validate_schema.py` / `validate_workflows.py` via conda.
- Hooks (D0128/D0130/D0296): `post-edit` fast tier + advisory; `stop` validate + guards, blocks while
  dishonest, silent when green (D0359); `pre-bash` advisory, TWO denies: heredoc with a backslash (D0309), `cat`/`tee` with nothing feeding it (D0491);
  `pre-write` protects fact surfaces; `config-change` refuses `disableAllHooks`. Every fire = one ledger line
  (`.keel/metrics/hooks.jsonl`). `.claude/` is generated: `keel sync-claude` (`--check` is the drift guard).
  Launch via `keel claude`. Hooks resolve the binary by one probe: `KEEL_BIN` → `.keel/bin/keel` → pin cache
  → PATH (D0230/D0316/D0343/D0391).
- Read syntax notes first: `.engine/docs/sysmlv2-syntax-notes.md`.

---

## 6. Host

- Windows + PowerShell + git-bash. Adapt every command (issue065). Absolute paths; shells share one cwd.
- The shell never writes code (D0309/D0491): a string replacement in an existing file is the **Edit tool**;
  a new file is the **Write tool**, run by path. Heredoc + backslash = DENIED; a `cat`/`tee` with nothing feeding
  it (no heredoc, no `<`, no operand) = DENIED — it hangs for the tool timeout. Anchored edits from a script:
  `python scripts/textpatch.py replace|insert-after|insert-before|append --old/--new` (D0386); `--probe` first.
- `conda` not on PATH: `C:\Users\WilliamWeatherholtz\miniforge3\Scripts\conda.exe run -n sysml --no-capture-output ...`
- A running `target/release/keel.exe` blocks its own rebuild (issue150): run commands from a COPY
  (`keel-serve.exe`). `keel suite` refuses from the build image.
- Never pipe a JVM's output; redirect to a file. Sweep: `python .engine/tools/kill_stale_kernels.py`.
- No `keel-cli/src` or `.engine` edits while cargo or `keel-land` runs (issue477).
- Never `cargo fmt`. Check LF with bytes (`python -c "...count(b'\r\n')"`), not `grep -c $'\r'`.
