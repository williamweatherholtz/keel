---
name: sprint-review
description: |
  The per-SITTING review (D0049, finished by analysis under D0510). After a sitting
  (one or more sprints), summarize the sitting's content + metrics, run the transcript
  scan that feeds the autonomous retro, present it, and record the presentation as an
  analysis the AI judges - listing what still asks for the human's word, item by item.
  Use when asked "sprint review," "review the sitting," "what's our velocity," or at
  the end of a sitting.
metadata:
  version: 0.3.0
  domain: [agile, sprint-review, sitting, analysis, metrics, process-improvement, SysMLv2]
  writePolicy: direct
  engine: keel-ai-toolkit
---

# sprint-review (per-sitting, finished by analysis)

The presentation point of the ceremony (D0049). Per-sprint closeOut + retro run
autonomously (sprint-closeout / sprint-retro skills); the human does NOT gate each
sprint. After a **sitting** (one continuous work session, ≥1 sprint), this review
presents the sitting's content and records that it was presented. The human's word is
asked for on the ITEMS that need it - a Decision, a proposed result, a finding - through
the verb each one has, never on the sitting as a whole (D0510: a confirmation over a sitting
is either a duplicate of those per-item records or a reading receipt, and a receipt is not
testimony, D0232).

Four outputs:
1. **Sitting summary** — the sprints completed this sitting + what shipped.
2. **Metrics snapshot** — velocity, efficiency, accuracy + trailing trend.
3. **Improvement queue** — transcript-scan findings (feeds the autonomous retro).
4. **Presentation record + coverage** — the review is a `Test` with `method = analysis`, judged by
   the AI actor, whose `#Covers` edges name the sprint `Story` items it presented (D0049/issue040)
   and whose `procedureText` lists the outstanding per-item asks BY VERB: `keel accept <d>` for a
   held Decision, `keel judge-set` for proposed results, a disposition for a finding. Coverage is
   then COMPUTED: `keel show sitting-coverage` reports which delivery sprints a review has
   presented vs not yet (a VIEW, not a gate). Record shape:
   `verification sittingRev<id> : Test { :>> method = VerificationMethod::analysis; :>> procedureText = "PRESENTED: ... ASKS: keel accept d0NNN; keel judge-set ...; ..." }`
   + a `TestResult` (judgedBy = the AI actor, `// RAN:` receipt naming the views read) +
   `#Covers dependency from sittingRev<id> to <sprintStory>;` for each presented sprint. Guard
   `sitting-review-method` refuses a `method = confirmation` sitting review recorded after D0510.

## Expert Vocabulary Payload

**Velocity:** sum of `estimatedPoints` delivered per sprint; trailing 3-sprint average
gives a planning baseline.

**Efficiency** (D0038): `estimatedPoints / actualHours` for a sprint. Unitless ratio —
higher = more points per hour. Track the trailing average to see if the team is
improving or degrading. A sudden drop signals unexpected complexity or rework.

**Accuracy:** How close was the Fibonacci estimate to what the sprint actually cost?
Compare `actualHours` to the guideline range for `estimatedPoints` (see sprint-planning
skill). If 1 pt took 6 h (guideline < 2 h), the estimate was off by 3×. Note the
direction (over/under) for calibration at retro.

**Transcript review:** structured scan of the session conversation (or git log + commit
messages) for: errors taken, bad directions, unnecessary rework, missing context,
confusion points, repeated questions, workflow violations, anti-patterns from any skill.
Each finding becomes a **process-improvement item** classified by remediation type.

## Phase 1 — Verify DoD

1. Confirm all DoD TestResults for this sprint are recorded with `outcome = pass`.
2. Confirm the story's DoDR1 is present in the backlog.
3. If any gate is missing or failed, **stop** — the sprint is not reviewable until DoD passes.

## Phase 2 — Record Metrics

1. **Prompt for `actualHours`** if not yet set on the sprint Story. Ask: *"How many
   wall-clock hours did this sprint take?"* Do not proceed to metrics until you have it.
2. **Record `actualHours`** on the sprint Story in the delivery file.
3. **Compute sprint metrics:**

   | Metric       | Formula                              | This sprint | Trailing 3 avg |
   |--------------|--------------------------------------|-------------|----------------|
   | Velocity     | estimatedPoints                      | _           | _              |
   | Efficiency   | estimatedPoints / actualHours        | _           | _              |
   | Accuracy     | actualHours vs guideline range       | within / over / under | trend |

4. **Build the history table** from all past sprints with `estimatedPoints` + `actualHours`
   set. Display as a running log.

## Phase 3 — Transcript Review (process-improvement scan)

Scan the sprint's session conversation and git log for the following signals:

| Signal type             | Detection cue                                                       |
|-------------------------|---------------------------------------------------------------------|
| Wrong route taken       | Corrected direction ("no, not that"), re-do after wrong approach    |
| Skill gap               | Repeated question, missing context, skill had no guidance for it    |
| Workflow violation      | Acted before classifying; recorded confirmation without sign-off    |
| Anti-pattern triggered  | Any item from a skill's anti-pattern watchlist was hit              |
| Unnecessary rework      | File edited > once for the same logical change; validator run twice |
| Missing guard           | A check that SHOULD have been automatic was done manually           |
| Schema / process gap    | Had to improvise because no rule existed                            |
| Documentation drift     | Code/process changed but a doc wasn't updated                       |

For each finding:

1. **Describe** the incident in one sentence.
2. **Classify** the remediation type:
   - `skill-update` — a skill's behavioral instructions need a new rule or anti-pattern
   - `claude-md-change` — CLAUDE.md §N needs an addition or clarification
   - `decision` — an architectural choice needs to be recorded (it's now implicit)
   - `backlog-item` — new tooling/automation needed
   - `retro-note` — observe and discuss; no immediate process change
3. **Propose the fix** — exact skill section, CLAUDE.md paragraph, or new backlog item.

Format as:

```yaml
improvement_items:
  - incident: "<one sentence>"
    type: skill-update | claude-md-change | decision | backlog-item | retro-note
    target: "<skill name / CLAUDE.md §N / decision title / backlog action name>"
    proposed_fix: "<what to add, change, or record>"
    priority: high | medium | low
```

4. **Route high-priority items to retro** for immediate action. Medium/low go into the
   backlog or are held for the next retro.

## Phase 4 — Record the presentation; list the asks by verb (D0510)

The review finishes as an ANALYSIS the AI judges - nothing about the sitting itself is put
to the human. Split what the sitting holds into two buckets and record both:

1. **Test-backed work [recap].** Every `method=test/inspect/analyze` item is
   self-evidencing — its automated run (cargo test, clippy, `keel gate validate`, `keel
   guard`) IS the evidence (D0051). Recap it under `PRESENTED:`; a human "yes" on a green
   test adds nothing and is not asked for.
2. **Judgment-only items [the asks, one verb each].** Read them from the computed views,
   never from memory: `keel show authority-queue` (held Decisions → `keel accept <d> --words`,
   or the published brief), `keel show attestation` (`proposed` results → `keel judge-set`),
   `keel show dispositions` (undispositioned findings → `keel record review`). List each
   under `ASKS:` with its verb. Every ask already has its own record that binds to that
   item's text; the review points at them and adds no record of its own over the sitting.

Record `verification sittingRev<id> : Test { method = analysis }` with the `#Covers` edges to
the sprints presented and a passing result judged by the AI actor (`// RAN:` naming the views
read). When `ASKS:` is empty, write `ASKS: none` - the review is still recorded, because
`sitting-coverage` counts presentations, and an unrecorded presentation is a sprint no review
presented. Never record a `method = confirmation` sitting review: guard `sitting-review-method`
refuses one added after D0510, and the human's acceptance of a held item is `keel accept`.

## Anti-Patterns

- **Skipping actualHours** — never record the review gate without actualHours on the Story.
- **No transcript review** — metrics alone miss process drift. The transcript scan is
  mandatory, not optional.
- **Improvement items as prose blobs** — each finding must be a typed, actionable item
  (improvement_items list above), not a paragraph. Blobs can't be tracked or resolved.
- **Accepting every improvement item** — not every finding warrants a process change.
  Apply judgment: if a finding is a one-off, log as retro-note; if it will recur, fix.

## Questions This Skill Answers

- "Sprint review"
- "What's our velocity?"
- "How efficient were we this sprint?"
- "How accurate were our estimates?"
- "What should we improve?"
- "Review the sprint transcript for issues"
- "Did we follow the process correctly?"
