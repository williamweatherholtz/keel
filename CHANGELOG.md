# Changelog

All notable releases of **keel** — the SysML-v2 work-tracking engine (text-is-truth; every
state computed, every fact authored via the write API, every change gated by honest-state guards).

## v0.1.0 — 2026-06-29

First public release. The engine is self-hosting: it has tracked its own construction through
168 sprints and 104 architecture decisions.

### Distribution
- Prebuilt `keel` binaries for Linux (x86_64), macOS (arm64), and Windows (x86_64) — no Rust
  toolchain required downstream.
- `keel init DIR` scaffolds a fresh project (binary-embedded `.engine/`); the `introduction`
  skill onboards a newcomer to first value.

### What `keel` does (no JVM, no kernel — pure Rust)
- **Orient / plan** — `keel orient`, `whats-next`, `suspect` compute state from authored facts + git
  (no status files; the model is the only tracker).
- **Write API** — `append-result`, `append-gate-result`, `add-task`, `apply-review` author facts with
  enforced UUIDs, provenance (who/when/commit), and append-only semantics.
- **Computed views** — `view`, `render`, `report`, `diagram`, `coverage`, `critique-coverage`,
  `tier-satisfaction`, `rootedness`, `boundary` / `boundary-sweep` (white/black-box subsystem critique),
  `sitting-coverage`, `dispositions`, `indicators` — all regenerable, never stored as truth.
- **Honest-state commit gate** — 13 hard-blocking forward guards + 1 warning guard
  (`keel guard`): truthful / well-formed / traceable, never "complete" (completeness is a
  non-blocking burndown surfaced in `orient`). Decisions must carry a substantive rationale;
  interconnects are typed edges, not prose.
- **Assurance** — antagonistic element critique (lens-tagged verifications), severity-carrying
  findings with typed human dispositions, git-temporal suspicion + `keel reverify` to refresh
  reproducible verifications at HEAD.
- **Interactive console** — `keel serve` (localhost): orient / decisions / sections / boundaries /
  findings / reports, with an optional `claude` agent bridge for directed, recorded critique.

### Releases
- A `v*` tag triggers `.github/workflows/release.yml`, which builds the three binaries and attaches
  them to a GitHub Release.

## v0.3.1 — 2026-08-29

The pin, the wrapper, and the library — the portability release (D0250/D0251/D0252).

### The engine pin is BINDING (D0251)
- `engine-version.toml` escalates from a parity warning to a binding pin: a mismatched binary
  REFUSES writes (at the write-lock choke point, all paths by construction) and gates (in the one
  gate body: `gate`, `sync`, `land`, pre-commit, plus `validate`/`guard` directly). Reads warn and
  proceed; `version`/`migrate` never refuse; an absent declaration keeps working.
- `keelw`: a committed POSIX-sh wrapper resolves the pin against `.keel/bin/<version>/`, downloads
  EXACTLY the pinned version on a miss, verifies the SHA-256 committed in `keel-wrapper.toml`
  (never trust-on-first-use), and never falls back to PATH. `keel init` ships both and seeds the
  cache with the running binary, so a fresh project works offline immediately.

### The library (D0250)
- `keel library init|sync|list`: a machine-local clone of your portable-content repository under
  `<home>/.keel/library`. Sync is fetch + fast-forward ONLY (diverged = a named defect; ahead =
  the sanctioned post-publish state); an unreachable remote is a STATED staleness over the
  last-good cache. A project's gate is byte-identical with the library present and absent —
  availability is never activation, held by test.
- `keel process import --from-library <name>` resolves the cache and delegates to the one import
  path; `keel process publish <name>` exports into the clone and commits (naming unit + version),
  never pushes, and refuses to commit an unchanged unit.

### Assurance
- Guards 53–54: `manifest-key-portability` (absolute AND traversal keys refuse; the unit manifest
  is repository-relative always) and `control-map-reconciled` (every control event names a declared
  control or states why it is instrumentation; dangling `provenBy` proof pointers refuse).
- The control-propriety panel (process + schema + skill + `keel view propriety`): five adversarial
  lenses with a named defeater per verdict. Its first round found and fixed two High defects in
  the session's own controls.
- Arming probes: the write lock, the launcher, the pre-write tiers, the fire ledger, the pre-push
  hook and reverify are now PROVEN to fire by test, not classified by reasoning. The pre-push probe
  found and fixed a field defect: the bootstrap push of a fresh project was refused (issue306).

### Breaking / behavioral
- A project whose `engine-version.toml` names another version now refuses writes and gates under
  this binary — run the pinned version, or `keel migrate` (which re-stamps and announces the
  escalation). Pre-D0190 trees with no declaration are unaffected.

## v0.5.1 — 2026-09-18

The downloaded 0.5.0, run over a tree `keel init` 0.4.1 had scaffolded, still rolled itself back - on
the adopter's own words. This release makes that run land (sprint 746; GH#86-90 answered).

### Downstream adoption (D0523; issue615-617, issue620)
- **A verb fold travels with `keel migrate`** (D0523). The new `verb-respell` step rewrites every
  retired `keel <verb>` reference in the living docs the resync does not write - the project's
  `CLAUDE.md` as the 0.4.1 `keel init` template wrote it, the comments of its project-owned contracts,
  a file it added under `.engine/` - to the spelling this binary dispatches (`keel orient` ->
  `keel show orient`, `keel add-task` -> `keel record task`, `keel report` -> `keel render report`),
  one reported edit per file, idempotent. A renamed verb's spelling is one shipped fact,
  `.engine/contracts/verb-renames.toml`, read from the binary by both the step and guard
  `cli-reference`'s `today it is` clause. `CLAUDE.md` joins the run's scope: an uncommitted one
  refuses the run; a rollback restores it with `.engine/`, `.tracking/` and `.claude/`. The
  process-change lock treats a locked contract whose text is exactly the respell of its HEAD text as
  the engine arriving (the D0441 shape one step on); one byte beyond the fold is back under the lock.
- The guard set now runs over an adopter-shaped tree in test, so a guard red on every adopter can no
  longer be green here by construction (issue615, GH#88).
- `WriteError::SchemaLacksMember` is a named refusal: it has a registry row, a census row and a
  ledger line (issue616).
- `keel onboard` no longer tells a first-time adopter to restore an `activation.toml` from a history
  that has none (issue617, GH#89).

## v0.5.0 — 2026-09-18

The release downstream adopters asked for: `keel migrate` lands in a project that is not this one.
256 commits since v0.4.1; 164 Decisions added, 252 Issues resolved (`keel show commit-delta --range v0.4.1..v0.5.0`).

### Downstream adoption (GH#68, GH#86-90; D0519-D0522)
- `keel migrate` no longer reverts itself in an adopting project: a resync is not a self-modification
  of the owner's items (D0441/D0519) and the run writes the evidence its own post-apply gate reads.
- Guard `source-reference`, outside the self-build, holds the adopter to its OWN living docs and sets
  the shipped engine docs aside, saying how many (D0520). A manifest's `charteredBy` resolves in
  `.engine/decisions/` and `.engine/reference/decisions/`, the directory the resync deploys to (D0520).
  The resync never seeds `parser-coverage-baseline.toml` into a project that has none (D0520).
- A write emits only what the tree's own schema declares: a `VerdictKind` member the adopter's
  `element.sysml` lacks is a refusal naming the member, the vintage and `keel migrate` — nothing is
  written (D0521).
- A record `migrate` tolerates at the door is tolerated through the run: the post-apply gate discounts
  it and the rollback's `git clean` preserves it (D0522).

### CLI surface — BREAKING (D0449-D0453, D0458)
- Five families folded under one verb each; the old top-level names are REMOVED, not aliased:
  `render <diagram|report|decision-card>`, `show <orient|whats-next|status|…>` (eleven lenses),
  `record <decision|issue|task|story|statement|result|gate-result|sprint|review>` (nine authoring verbs),
  `gate …` / `audit …` (twelve gating verbs), `github <pull|ingest|…>`. 34 top-level verbs remain.
  Every command is a `CliCommand` fact in `.engine/cli/commands.sysml`; `--help` renders from it.
- `keel accept <d> --words "<verbatim>"` records a human's chat acceptance as their own words in a
  declared pair; a read-back miss or short words WARN in the note rather than refuse (D0375/D0423).
  `keel reject` is the rejection's CLI path (D0393/D0470); `keel judge-set` records a human's verdict on
  a set of proposed results, one record per item (D0443).
- A Decision whose text names `process-change`/`safety-change` is held proposed under standing consent
  even without the marker line (D0439). `#Supersede` retires a whole target; `#SupersedeClause`
  reverses one clause (D0398). Every id written from today is an RFC 4122 v4 UUID (D0430).

### Landing and verification
- `keel land` runs the touched integration-test set before the first push (D0421) under cargo-nextest
  (D0475), computed per workspace member and its dependents (D0481), skipping binaries observed green
  at the current content (D0474); it names the CI conclusion of the base it pushes onto (D0420).
- `keel verify [--probe POS,NEG | --probe-from FILE] [--wait]`: the pre-commit ladder in cost order,
  stopping at the first red, one receipt (D0476/D0497/D0500). `keel suite` writes a receipt and gates
  nothing (D0356). One touched run per tree at a time (D0493).
- The enforced guards run across a thread pool in declared order (D0368); a green gate answers from a
  receipt keyed per guard on the inputs it reads (D0371/D0482). Every guard warning is one of two
  stated classes: actionable, or counted history (D0413).
- `keel advance <process> [--to <step>]` works for any adopted process and is refused while an earlier
  bound step is red (D0436); a `ProcessStep` declares `checkedBy` — a guard, a declared rule, or
  `gate:<phase>` (D0434/D0435).

### Release integrity
- Every release asset ships with its SHA-256 and one `SHA256SUMS` the build verified (D0385); a
  `Release` names its tag as a field and guard `release-recorded` binds on it (D0400). `keelw`
  verifies the committed entry in `keel-wrapper.toml`; the 0.5.0 entries are copied from this
  release's published `SHA256SUMS`.

### Internals
- `keel-cli` decomposed into workspace members along computed dependency layers (D0479-D0513);
  the layering is a guard (D0483). On a self-build tree the hooks run a stable copy at `.keel/bin`
  they refresh themselves (D0391); the post-commit land runs from its own image (D0422).
- Ceremony is delegated: a verifier subagent runs the gate set and writes a receipt; a recorder writes
  through `keel record` only and its report is refused otherwise (D0425/D0473/D0492/D0516).

### API contract
- `KEEL_API_VERSION` stays 2.0.0: no `/api/*` read shape changed in this range.
