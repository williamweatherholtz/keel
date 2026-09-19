// Intake for 2026-09-19 — their words, my translation, the triage verdict (D0166/D0216).
// Written by `keel record statement` / `keel record story` (D0236): a Statement's text is
// VERBATIM, so it is escaped for the literal and otherwise untouched.
package ProjectIntake20260919 {
    private import EngineElement::*;
    private import EngineIntake::*;
    private import EngineRelationships::*;

    part st169 : Statement {
        :>> id = "e0f16b0d-a879-40ca-91dc-ae54f9e8a48d";
        :>> title = "Retro findings are empirical and become issues, not decisions";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> text = "I didn't care if the name is caps, and I usually want issues coming out of retros, not decisions. Usually retro findings are based on empirical evidence. If they're proposing a non occasion issue, it could be a decision, I suppose\n\nI wrote this text for the''retro'' item. Looks like it bless to all others?";
        :>> saidBy = "wweatherholtz"; :>> saidAt = "2026-09-19"; :>> channel = StatementChannel::chat;
    }

    part us123 : UserStory {
        :>> id = "c5b4ee50-a198-41ce-bc80-e2ee433dc304";
        :>> title = "A retro finding lands as an Issue; a Decision from a retro says what it proposes beyond the evidence";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the human who reads retros";
        :>> iWant = "a retro finding that reports something observed to land as an Issue with a resolver, and to see a Decision come out of a retro only when the finding proposes something the evidence did not occasion";
        :>> implication = ImplicationKind::convention;
        :>> triageNote = "Their words arrived as the notes field of the fifty-second brief, typed for the previous page's retro item (d0510, a sitting review finished by analysis like the retro) and attached to the d0523/d0524 answers because the page has one notes field. The preference is already the practice: sprint 746's retro produced issue625-628 with resolvers and no Decision (D0131 tracked items, D0047 Issue plus control). What admits the other outcome is guard retro-backlog, which accepts a Decision d0NNN as a finding's stated home (D0461, guards.md row retro-backlog). Candidate control, not authored here because the words end in I suppose: a finding whose home is a Decision states what it proposes beyond the observation, else the guard warns. The caps clause cannot be placed: no accepted decision on either page mentions casing.";
    }
    #DerivedFrom dependency from us123 to st169;

    part st170 : Statement {
        :>> id = "7cd58874-737c-4918-bbe9-fe43a4cc8b70";
        :>> title = "GH#94 decision-scaffolding still reports a superseded compound Decision, blocking the commit that supersedes it";
        :>> createdAt = "2026-09-19"; :>> createdBy = "githubRecorder";
        :>> text = "## What happened\n\nAfter a compound Decision (d0037) was superseded whole by three single-clause Decisions — `#Supersede dependency from d0038 to d0037;` recorded by `keel record decision --supersedes d0037`, and the CLI itself printed `d0037 is RETIRED whole` — the `decision-scaffolding` guard keeps reporting the retired Decision as a violation:\n\n```\nERROR d0037: a COMPOUND Decision - its text enumerates 3 clauses ...\n[guard:decision-scaffolding] FAIL — 195 scanned, 194 warning(s) + 1 counted-history line(s), 1 violation(s)\n```\n\nThe prescribed remedy in the guard's own message is exactly what was done (''record one Decision per clause, with #DependsOn edges between them''). Because the compound Decision's text is signed and cannot be edited, there is no action left that clears the violation, so the commit that performs the remedy is blocked by the guard that prescribed it.\n\n## Expected\n\nA superseded (retired) Decision should be exempt from `decision-scaffolding`, or counted as history the way pre-2026-09-05 compound Decisions are (''counted, not reported''). Partial delivery of a retired Decision is not a live risk: nothing can be chartered to it any more.\n\n## Repro\n\n1. Record a compound Decision (see #93 for how the API lets it through).\n2. Record a superseding Decision with `--supersedes` that carries one clause; add siblings with `#DependsOn`.\n3. `keel gate guard decision-scaffolding` — the retired Decision is still the violation.\n\n## Environment\n\nkeel 0.5.0 (4a0abc8), downstream project plotwright, 2026-09-19.\n";
        :>> saidBy = "williamweatherholtz"; :>> saidAt = "2026-09-19"; :>> channel = StatementChannel::github;
        :>> sourceUrl = "https://github.com/williamweatherholtz/keel/issues/94";
        :>> sourceTrust = SourceTrust::untrusted;
    }

    part st171 : Statement {
        :>> id = "e619e7ea-830f-488c-b2ef-41593de02c06";
        :>> title = "GH#93 record decision accepts a compound Decision that decision-scaffolding refuses at commit";
        :>> createdAt = "2026-09-19"; :>> createdBy = "githubRecorder";
        :>> text = "## What happened\n\n`keel record decision` accepted and (under standing consent d0207) auto-accepted a Decision whose text enumerates three numbered clauses. The `decision-scaffolding` guard then refused it at commit time:\n\n```\nERROR d0037: a COMPOUND Decision - its text enumerates 3 clauses - and decision-scaffolding cannot see which clause an edge charters, so partial delivery would be invisible (issue331). D0303 option C: record one Decision per clause, with #DependsOn edges between them, and let each be chartered on its own.\n```\n\nBy then the Decision was accepted, so its text was signed (D0308) and could not be reworded. The only way out was to record three new Decisions and supersede the compound one whole — three UUIDs, a `#Supersede` edge and a retired Decision, for a defect the write API could have refused in the same second it took to detect at commit.\n\n## Expected\n\nThe write API should run the same clause count that `decision-scaffolding` runs and refuse the record (or at least refuse the auto-acceptance) before the text is signed. The guard already has the detector; the record verb just does not call it.\n\n## Repro\n\n1. `keel record decision --from draft.md` where the `--- decision` section reads ''(1) ... (2) ... (3) ...'' with three clauses.\n2. Observe: recorded and accepted at record time under d0207.\n3. `keel gate guard decision-scaffolding` — violation on the new Decision.\n\n## Environment\n\nkeel 0.5.0 (4a0abc8), downstream project plotwright, 2026-09-19. Related: #92 (record review accepted what judge-set refused — same shape: a write verb without the check that a later gate applies).\n";
        :>> saidBy = "williamweatherholtz"; :>> saidAt = "2026-09-19"; :>> channel = StatementChannel::github;
        :>> sourceUrl = "https://github.com/williamweatherholtz/keel/issues/93";
        :>> sourceTrust = SourceTrust::untrusted;
    }

    part st172 : Statement {
        :>> id = "28ac3b2b-3bc2-40e7-b28d-2063a31909e5";
        :>> title = "GH#92 [control gap] record review writes human-judged confirmation dispositions from an agent environment; judge-set refuses the same act under D0178/K6";
        :>> createdAt = "2026-09-19"; :>> createdBy = "githubRecorder";
        :>> text = "`keel judge-set` refuses to record a human verdict from an agent environment; `keel record review --batch` records thirteen of them from the same environment without objection. Both write `method=confirmation` verifications judged by a `Person`. One of them is wrong, and it is the permissive one.\n\nBinary `keel 0.5.0`, build `4a0abc8`. Project: plotwright. Same shell, same session, same actor binding (`KEEL_ACTOR=claudeOpus5`), same minute.\n\n## What happened\n\nThe supervisor said, verbatim, ''Proceed through all items'' against a page listing thirteen finding dispositions and three proposed results. I put both to the engine.\n\n`judge-set` did the right thing:\n\n```\n$ keel judge-set .tracking/delivery/sprint4_castPortsAndBeatNodes.sysml --words ''Proceed through all items'' --by wweatherholtz --date 2026-09-19 --all\nkeel judge-set: this session carries agent-environment markers and no interactive terminal (D0178/K6),\n  and attestation-policy.toml declares no recording delegation for confirmationRecord.\n  The verdict is the human's own act: run `keel judge-set` from YOUR terminal, or give it from the\n  console approve queue / the deck.\n```\n\n`record review` did not:\n\n```\n$ keel record review --batch .keel/review-batch-dispositions.json --sha 45a4a98 --judged-by wweatherholtz --judged-at 2026-09-19\nissue001Disp1  (issue001 disposition:act)\n...\nissue013Disp1  (issue013 disposition:act)\napplied 13 disposition(s) to .tracking/critiques-wweatherholtz.sysml\n```\n\nEach of those thirteen is:\n\n```\nverification issue001Disp1 : Test { method = VerificationMethod::confirmation; disposition = DispositionKind::act; ... }\npart issue001Disp1R1 : TestResult { outcome = VerdictKind::pass; judgedBy = ''wweatherholtz''; ... }\n```\n\nA `method=confirmation` record, `judgedBy` a Person, written by an AI actor from an agent environment with no delegation declared and no quoted words — exactly the shape K6 exists to refuse, and exactly the shape D0106 calls fabricated when an AI supplies it. The batch file's rationales were drafted by me; nothing in the record says so.\n\nI deleted the file before committing. `keel show dispositions` went back to 13 undispositioned. Nothing landed.\n\n## Why it matters\n\n`confirmation-authenticity` would presumably have caught the missing delegation at the gate — I did not commit to find out. But a write path that produces a record the gate then refuses is the shape #86 had: the write API reporting success on something the tree cannot honestly hold. And a finding disposition is the single most consequential human act the engine records (D0092's own words: ''the most important recurring human act is JUDGING a finding''). It should have the same door as a gate verdict.\n\nThere is a second, smaller asymmetry visible in the same pair: `judge-set` demands the human's words and quotes them into the record; `record review` has no `--words` and stores only the batch's `rationale`, which the tool cannot distinguish from the human's own. So even under a declared delegation, a review batch carries less provenance than a judge-set.\n\n## Suggested fixes\n\n1. Apply the D0178/K6 check in `record review` whenever a batch entry would write a `method=confirmation` verification — the disposition verdicts `act` / `accept-risk` / `dismiss`. Element-review verdicts (`accept` / `finding`) are `method=critique` and are correctly the recording actor's own.\n2. Give `record review` the same delegation shape as `judge-set`: `--words` required under `confirmationRecord` delegation, quoted into each disposition's record, so the human's utterance and the drafter's rationale are two fields rather than one.\n3. Failing both, have the refusal message name what to do — the `judge-set` one does this well: ''run it from YOUR terminal, or give it from the console approve queue / the deck''.\n\nFor what it is worth: the `judge-set` refusal text is the clearest thing the engine has said to me all session. It told me exactly whose act this was and where to go.\n";
        :>> saidBy = "williamweatherholtz"; :>> saidAt = "2026-09-19"; :>> channel = StatementChannel::github;
        :>> sourceUrl = "https://github.com/williamweatherholtz/keel/issues/92";
        :>> sourceTrust = SourceTrust::untrusted;
    }

    part st173 : Statement {
        :>> id = "88e2a716-caea-4296-9c08-5efc598906af";
        :>> title = "GH#91 [bug] migrate refuses on a removed TYPE but not on a removed ENUM MEMBER - the resync leaves an unrepresentable value behind";
        :>> createdAt = "2026-09-19"; :>> createdBy = "githubRecorder";
        :>> text = "`keel migrate` refuses and enumerates affected items when the engine REMOVES a type, but says nothing when it removes an ENUM MEMBER. The resync applies, and the tree is left holding a value the new schema rejects — caught only by `check-engine` afterwards, or not at all if nobody runs it.\n\nBinary `keel 0.4.1`, build `0c131b7`. Observed resyncing plotwright from `b171cd7`.\n\n## What happened\n\nThe resync applied 589 files. Immediately after:\n\n```\n$ keel gate check-engine .\nERROR: .engine/decisions/0020-howTheRootingWorkIsSequenced.sysml:30 — unknown member `superseded` in enum `DecisionStatus`\n       hint: valid members: accepted, proposed, rejected\n1 .engine semantic diagnostic(s).\n```\n\n`DecisionStatus` lost its `superseded` member between the two vintages. `d0020` was carrying it. Migrate's own step list ran `removed-types` and reported `already current` — correctly, because no *type* was removed. There is no equivalent step for a removed enum *member*, so the instance was left behind silently.\n\nThe asymmetry is explicit in the documented behaviour: migrate ''never deletes a file it does not ship, and where the engine REMOVED a type it refuses and lists each affected item with its replacement rather than guessing.'' That guarantee is exactly right, and an enum member is the same class of change — a value the new schema can no longer represent — with none of the protection.\n\n## Why it is more than cosmetic\n\nOnly `check-engine` catches it. `keel gate validate` passed clean at 20 files throughout, because it validates `.tracking` and this instance lives in `.engine/decisions/`. A project that runs `validate` and `guard` after a resync — which is what the migrate output itself tells you to do, though it does name `check-engine` too — and does not happen to read the `check-engine` line would carry an unrepresentable enum value indefinitely.\n\n**And the obvious repair is a trap.** `superseded` reads as though it should become `accepted`. I tried that, and two guards refused it, correctly:\n\n```\nERROR d0020: accepted but no passing acceptance event (D0066)\nERROR d0020: accepted but its acceptance event is not human-judged — a sign-off must be a real\n      human attestation, never AI-fabricated (D0106/D0016)\n```\n\nd0020 is a fork that was held for the supervisor and never signed. The truthful value was `proposed`, and what the removed member had been encoding — that d0026 superseded it — is carried by the `#Supersede dependency from d0026 to d0020` edge, not by the status. That is the right design, and dropping the member is a good change. The problem is only that the migration did not carry the instances across it.\n\nWorth noting how easily this goes wrong unattended: an agent resolving this without reading d0020's history would pick `accepted`, because that is what a superseded-but-formerly-live decision looks like, and would then be one step from fabricating the acceptance event the guards demand.\n\n## Suggested fixes\n\n1. **Extend the `removed-types` refusal to removed enum members.** Same mechanism, same message: refuse, list each affected item and its declared replacement, do not guess. The engine already knows both enum definitions at migrate time.\n2. Where a mechanical mapping is unambiguous and lossless, offer it as a named migration step the way `process-as-action` and `release-as-occurrence` already are. `DecisionStatus::superseded` is *not* such a case — the correct target depends on whether the decision was ever accepted — which is an argument for refusing rather than transforming.\n3. Independently: consider having the post-apply gate run `check-engine`, so a semantic diagnostic in `.engine/` participates in verified-or-reverted rather than surviving the apply. Today the apply can leave `.engine` semantically invalid and still be considered green by the guard set.\n";
        :>> saidBy = "williamweatherholtz"; :>> saidAt = "2026-09-19"; :>> channel = StatementChannel::github;
        :>> sourceUrl = "https://github.com/williamweatherholtz/keel/issues/91";
        :>> sourceTrust = SourceTrust::untrusted;
    }

    part st174 : Statement {
        :>> id = "f0e1cd77-4c15-471b-95fd-7f9b6f53cb34";
        :>> title = "Decision capture needs a permanent solution";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> text = "GitHub ingest and backlog ingest. We need a permanent solution to decision capture";
        :>> saidBy = "wweatherholtz"; :>> saidAt = "2026-09-19"; :>> channel = StatementChannel::chat;
    }

    part st175 : Statement {
        :>> id = "9b9c7d04-df14-4849-9a01-279a930bce65";
        :>> title = "GH#96 No human-verdict channel is remote-capable, self-capturing and self-bootstrapping; a fresh project's sum is a terminal and a hand-edited policy file (issue632)";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> text = "Tracked as `issue632`, resolver `dcVerdictChannelIsOneSurface` (keel main 5d2e799).\n\n## What happened\n\nOn mm-site (a fresh keel project, engine 0.5.0), three Decisions sat `proposed` after a design panel. The only path the tooling offered the project's human was another terminal and `keel accept dNNNN --words ''...'' --by william --date ...` three times, retyping words he had already given in chat. He refused:\n\n> I'm not going to start another terminal to copy and paste words and manually edit things to accept a decision. That's slow and labor intensive. Getting my feedback, critique, and plainly illustrating the implications of the executive decisions I have to make is a crucial part of the experience here. I'm not to be gatekept - we need a perfectly reliable, easy way for me to drive development that works local or remote. So far, everything has been a compromise (keel serve is local, pages don't actually capture feedback so I have to manually copy text, etc.)\n\nAnd, on the framing:\n\n> ''Never route me to a terminal'' is an oversimplification. I'm the executive and the bottleneck, I need careful artifacts to make sure I'm getting all the right data to make the right decisions.\n\n## Why the existing fix did not reach him\n\nThe same complaint is on record as st074 behind D0289, which lets an agent session record an acceptance under a declared delegation when it quotes the human. Grants are not inherited (issue376): a scaffolded project's `attestation-policy.toml` ships with both `delegatedRecording` lines commented out, so the first thing a new project's human meets is a hand-edit of a TOML file to permit the channel that would spare them hand-edits (issue621 covers the other half).\n\n## Channel inventory, measured against what he asked for\n\n| channel | remote | captures the verdict | carries D0192/D0198 substance itself | shows implications + critique | bootstraps its own delegation |\n|---|---|---|---|---|---|\n| `keel serve` | no | yes | yes | partial | no |\n| GitHub gesture | yes | yes | yes | no | no — deprecated with its workflow (D0289) |\n| Smartsheet inbox | — | — | — | — | retired (D0206) |\n| D0173 mobile inbox transport | designed | no live carrier | | | |\n| deck | yes | read only | | | |\n| rendered HTML decision queue | yes | **no** — he reads it, then types elsewhere | no | partial | no |\n| quoted chat words | yes | yes, by hand | only where already granted | only if the agent builds the artifact | no |\n\nNone meets all five columns. None presents, beside the choice, the critique recorded against the Decision and the concrete effect of each verdict on the frontier — so even the surfaces that exist ask for a blessing rather than support a judgment (the `dsGround` failure).\n\n## Done means (the tracked DoD)\n\nOne surface, reachable from a phone or a browser away from the machine running keel, that lists every pending gesture with the substance the CLI computes — context, clauses reversed, critique on record, frontier effect of accept vs reject. A tap or a sentence there **is** the gesture: captured by the surface, written through the write API with the D0192/D0198 substance, never restated by the human in a terminal, chat or file. Works local and remote; verdicts land as receipts an agent session can act on. A delegation grant is itself a gesture on that surface. Each existing channel above is retired or named as the transport. Demo: fresh project, proposed Decision, phone, acceptance recorded with the human's words, frontier re-ranked — zero terminal, zero paste.\n\n## Also observed while filing\n\n- Two sessions minted `issue622` concurrently; `duplicate-identity` caught it at the later gate, not at `keel record issue` time. Minting over the tree, not over the file, would refuse the second mint.\n- A `keel record decision` probe with an unknown flag (`--bogus`) wrote the Decision instead of refusing; an unknown flag on a write verb should refuse.\n";
        :>> saidBy = "williamweatherholtz"; :>> saidAt = "2026-09-19"; :>> channel = StatementChannel::github;
        :>> sourceUrl = "https://github.com/williamweatherholtz/keel/issues/96";
        :>> sourceTrust = SourceTrust::untrusted;
    }

    part st176 : Statement {
        :>> id = "54383813-b605-4533-9c28-1fc917805569";
        :>> title = "GH#95 no lens lists a need-class UserStory from which no Need was derived";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> text = "## What happened\n\nA UserStory was recorded through `keel record story` with `implication = ImplicationKind::need` on 2026-09-18. No Need was ever derived from it. Nothing reported that. The supervisor asked for the same thing again a day later (''we chatted about this one, yeah?''), which is how it was noticed.\n\n`keel show whats-next` ranks backlog; `keel show tier-satisfaction` counts Needs decomposed into SystemRequirements and SystemRequirements verified by Tests; `keel show authority-queue` lists what awaits the human. None of them looks one tier up: a story the intake process itself classified as a Need, with no `#DerivedFrom` edge from any Need, is invisible to every lens.\n\n## Expected\n\nA lens (in `orient`, or its own `keel show`) listing every UserStory whose `implication` is `need` and from which no Need is derived, with the time waited - the same shape as `proposed_results`. The intake process asks the AI to classify each story's implication precisely so it can be read back; today the field is written and never read.\n\n## Repro\n\n1. `keel record story --from-statement stNNN --implication need ...`\n2. Record nothing else.\n3. `keel show orient .`, `keel show whats-next .`, `keel show tier-satisfaction .` - no line mentions the story.\n\n## Environment\n\nkeel 0.5.0 (4a0abc8), downstream project plotwright, 2026-09-19. The story in question waited 26 hours and was found by the supervisor, not the tool.\n";
        :>> saidBy = "williamweatherholtz"; :>> saidAt = "2026-09-19"; :>> channel = StatementChannel::github;
        :>> sourceUrl = "https://github.com/williamweatherholtz/keel/issues/95";
        :>> sourceTrust = SourceTrust::untrusted;
    }

    part st177 : Statement {
        :>> id = "7bb11423-b96a-4eaf-99b0-6d9b501ce328";
        :>> title = "GH#32 Protected-path enforcement is tool-bound, not path-bound: a shell redirect and rm write .engine/decisions/ ungated, and the override obligation never fires";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> text = "## Summary\n\nProtected-path enforcement is bound to the **tool** that writes, not to the **path** being written. A shell redirect and `rm` reach `.engine/decisions/` — a governance surface — completely ungated, and because the write never touches the gate, `keel override`'s obligation record never fires either.\n\n## Observed\n\nUnder the strict profile, an in-place `sed` over a fact surface IS blocked:\n\n```\n$ sed -i '/part you : Person/d' .tracking/actors.sysml\n[keel] in-place sed over a fact surface - use the keel write API (D0176; blocking under the strict profile)\n```\n\nGood. But in the same session, against a path that appears in the PreToolUse protected list (`.engine/decisions/`):\n\n```\n$ sed -e 's/package Decision0001 {/package Decision0230 {/' /\n      .engine/decisions/0001-landscapeProcessCharter.sysml /\n      > .engine/decisions/0230-landscapeProcessCharter.sysml      # succeeded, ungated\n\n$ rm .engine/decisions/0001-landscapeProcessCharter.sysml          # succeeded, ungated\n```\n\nBoth went through with no gate, no prompt, no record.\n\n## The obligation never fired\n\nBefore the write I armed the sanctioned unlock:\n\n```\n$ keel override .engine/decisions/0230-landscapeProcessCharter.sysml --reason ''...''\noverride armed for `.engine/decisions/0230-landscapeProcessCharter.sysml` - SINGLE USE,\nexpires in 900s; consumption records an obligation (D0176/K7).\n```\n\nAfter the write:\n\n```\n$ keel enforcement-report .\n  ''overrideLedgerEvents'': 0,\n  ''overrideObligationsUnsynced'': 0,\n  ''overrideObligationsTracked'': 0,\n```\n\nThe override was armed and never consumed, because the write bypassed the mechanism that would have consumed it. D0176's guarantee that *''consumption records an obligation''* holds only for writes that arrive through Write/Edit.\n\n## Why it matters\n\n`.engine/decisions/` holds accepted Decisions and their acceptance events. The pre-write hook lists it as protected precisely because it is a governance surface. A path that can be written by redirect is a path where a Decision's `status`, or an acceptance `TestResult`'s `judgedBy`, can be altered with no gate and no obligation trail.\n\nThe pre-bash hook already demonstrates the right instinct — it pattern-matches `sed -i` on fact surfaces. It just does not cover `>` redirect, `tee`, `cp`, or `rm` to the same paths.\n\n## Expected\n\nEither:\n\n1. **path-bound enforcement** — any write to a protected path is gated regardless of mechanism; or\n2. **widen the pre-bash matcher** to cover redirect / `cp` / `mv` / `tee` / `rm` targeting protected paths, the way it already covers `sed -i`.\n\n(1) is the durable fix; (2) closes the observed hole. Either way `keel override` should be the only way through, so the obligation ledger stays honest.\n\n## Related\n\nSibling finding in the same session: `guard:scaffold-placeholder` passes with a placeholder `Person` still registered in `.tracking/actors.sysml`. Filed separately. Together — an ungated write path to `.engine/decisions/` plus an unowned registered `Person` — they are the two halves of a fabricated-acceptance path. **I did not test that end to end** and am not claiming it; each is reported as the individual defect it is.\n\n## Environment\n\n```\nkeel 0.3.0\nbuild commit: c444ca5\napi contract: 2.0.0 (derived)\nguards: 51 (42 hard-blocking, 9 warning-only)\n```\nWindows 11, Git Bash. Project scaffolded 2026-08-29 via `keel init . --profile strict`.\n";
        :>> saidBy = "williamweatherholtz"; :>> saidAt = "2026-08-30"; :>> channel = StatementChannel::github;
        :>> sourceUrl = "https://github.com/williamweatherholtz/keel/issues/32";
        :>> sourceTrust = SourceTrust::untrusted;
    }

    part st178 : Statement {
        :>> id = "d3fd0749-135e-4d01-b3bb-1a851424b80c";
        :>> title = "GH#31 No item type for a design INPUT - a value the design consumes but does not decide";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> text = "## Summary\n\nThere is no item type for **a value the design consumes but does not decide** — a design input. Frost depth, static pressure, soil infiltration rate, available supply flow, mature plant size. Every existing type is something you *decide*, something you *require*, something you *watch without gating*, or something someone *uttered*.\n\nThis surfaced on a real project (residential landscape/irrigation) whose author said, verbatim: *''I'll be providing more facts to serve as basis for calculations.''* There is nowhere correct to put them.\n\n## Walking the type list\n\n| Type | Why it does not fit |\n|---|---|\n| `Decision` | implies someone *chose* it, with a rationale and a supersession path. `FROST = 30 in` is not a choice anyone made. |\n| `SystemRequirement` / `Requirement` | a *shall*, not an *is*. A design input constrains the design; it is not a contract the system must meet. |\n| `Indicator` + `Measurement` | closest — `Measurement` already carries `value` / `measuredAt` / `source`. But `Indicator` is invariant-7 **''MONITORED with no enforced threshold — never gated''**, and a design input *is* gated: zone sizing must not exceed available GPM. Using it would mis-signal the semantics. |\n| `Statement` | what someone *said*, not what is true. Correct for the utterance, wrong for the value. |\n| `Claim` | a work-claim (who is working an item). Unrelated despite the name. |\n| `Brief` / `Persona` / `Component` | not values. |\n\n## What I did instead, and why it only half-works\n\nFor an assumed supply floor I recorded a `Decision`:\n\n```\nd0231 — Design to a supply floor of 7 GPM at the point of connection, pending measurement\n```\n\nThat is *defensible here*, because choosing to design to a floor pending measurement genuinely **is** a decision, with a real rationale and a real supersession path (the bucket test).\n\nIt breaks on the very next fact. `FROST = 30 in` recorded as a `Decision` would fabricate an author, a rationale and a supersession path for a fact about the world. The workaround does not generalise.\n\n## What is NOT being asked for\n\nNot an `Assumption` type. issue105 is right that assumption-ness must be **computed** (unverified + depended-upon) rather than authored, and that should not change. This asks for the **carrier** the computation runs over — today there isn't one, so the computed lens has nothing to point at.\n\nConcretely, in this project `keel assumptions` returns `count: 0` while the design demonstrably rests on an unmeasured GPM figure. It returns 0 correctly, because nothing depends on `d0231` yet — but the reason there is no natural dependent is that there is no type for the values that would depend on it either.\n\n## Sketch of a fit (not prescriptive)\n\nA part def carrying `value` / `unit` / `source` / `asOf`, which:\n\n- participates in the `#DependsOn` propagation set, so dependents go suspect when the value moves;\n- is verifiable by a `Test`, so **measured vs assumed is computed** from whether a passing result exists, rather than declared in a field that can disagree with reality;\n- is *gateable*, unlike `Indicator`.\n\nThat last point is the whole difference from `Indicator`, and it is what makes this a separate type rather than a relaxation of that one.\n\n## Environment\n\n```\nkeel 0.3.0\nbuild commit: c444ca5\napi contract: 2.0.0 (derived)\nguards: 51 (42 hard-blocking, 9 warning-only)\n```\nWindows 11. Project scaffolded 2026-08-29 via `keel init . --profile strict`.\n";
        :>> saidBy = "williamweatherholtz"; :>> saidAt = "2026-08-30"; :>> channel = StatementChannel::github;
        :>> sourceUrl = "https://github.com/williamweatherholtz/keel/issues/31";
        :>> sourceTrust = SourceTrust::untrusted;
    }

    part st179 : Statement {
        :>> id = "990f5a29-7c12-464e-9f3d-a6d5e89f71db";
        :>> title = "GH#29 keel: decisions auto-accepted under standing consent";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> text = "<!-- keel-standing-thread -->\n\nEvery Decision auto-accepted under your standing consent (D0207) is posted here as a comment rather than as its own issue, so this thread is the override surface for all of them and **nothing here needs you**.\n\nTo reverse one, reply `reject dNNNN <why>` naming the decision.\n\nNobody is assigned, so you can mute this thread. A decision that genuinely needs you is a FORK: it opens its own issue and assigns you, which notifies through a mute.";
        :>> saidBy = "github-actions[bot]"; :>> saidAt = "2026-08-25"; :>> channel = StatementChannel::github;
        :>> sourceUrl = "https://github.com/williamweatherholtz/keel/issues/29";
        :>> sourceTrust = SourceTrust::untrusted;
    }

    part st180 : Statement {
        :>> id = "8c592dcf-efa0-4295-adf8-9f1505079139";
        :>> title = "GH#9 keel land can report 'landed' while just-authored facts never reach the pushed commit";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> text = "Found in Penumbra (recorded there as issue125), keel 0.2.3+.\n\n**What happened:** a session authored three Claim records via hand-edit of .tracking/claims/<actor>.sysml, ran `keel land`, and got 'keel land: landed.' Three days later, git ancestry showed those additions first reached a commit that day - the claims never existed in any pushed tree, so the exclusion they assert was phantom for three days. guard:claim-ancestry caught it at the eventual commit (good control), but only then.\n\n**Root:** on a shared checkout with branch churn, land can push while the working tree's just-authored facts are uncommitted - and still print 'landed'.\n\n**Suggested control:** land should either (a) commit-then-push what the write API just authored, or (b) refuse with 'authored but uncommitted: <files>' - never print 'landed' over a working-tree-only write. A cheap invariant: after land, `git status --porcelain .tracking/` must be empty or land names what it left behind.";
        :>> saidBy = "williamweatherholtz"; :>> saidAt = "2026-08-24"; :>> channel = StatementChannel::github;
        :>> sourceUrl = "https://github.com/williamweatherholtz/keel/issues/9";
        :>> sourceTrust = SourceTrust::untrusted;
    }

    part us124 : UserStory {
        :>> id = "facfd739-8265-4e11-a109-84192c1d921a";
        :>> title = "A verdict is given where the brief is read and lands in the tree without a terminal";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the human who decides";
        :>> iWant = "to give a verdict on a proposed Decision where I read it, from a phone or a browser away from the machine, and have it recorded without retyping the words anywhere";
        :>> implication = ImplicationKind::designChange;
        :>> triageNote = "Their words came as one line after a night in which three Decisions on a fresh project sat proposed because the only offered path was a terminal (st175). The pain is on record as issue632 and its resolver dcVerdictChannelIsOneSurface already names the five properties done means; what no item chose was the surface, so this story routes to the resolver and to the fork Decision that chooses it (D0264: an untrusted sibling story shares the same targets). Backlog ingest is the second half of the same sentence and is the triage of the eighty-seven unparsed Statements, recorded as their own stories.";
    }
    #DerivedFrom dependency from us124 to st174;

    part us125 : UserStory {
        :>> id = "f650d9cb-355b-4a88-a080-2a7023e51d19";
        :>> title = "Every pending gesture is answerable from one remote-capable surface that captures and bootstraps itself";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the human who decides, on a fresh project";
        :>> iWant = "one surface that lists every pending gesture with the substance the CLI computes, where a tap or a sentence IS the gesture, works local and remote, and where granting the recording delegation is itself a tap - so a fresh project never starts with a TOML hand-edit";
        :>> implication = ImplicationKind::need;
        :>> triageNote = "Filed by the project owner from a downstream session and already tracked before ingest: the issue names issue632 and dcVerdictChannelIsOneSurface in its first line, and the resolver DoD (backlog.sysml, dcVerdictChannelIsOneSurfaceDoD) is the Done-means section of the issue verbatim. Public repo, so untrusted (D0264): routed to the resolver and to the held fork Decision that chooses the surface, which the human accepts or not. The two also-observed items are triaged apart: the concurrent issue622 mint became issue624 and d0524 (class 5 of duplicate-identity, accepted); the unknown-flag write is checked against keel-cli/tests/unknown_flag_is_refused.rs before any new item.";
    }
    #DerivedFrom dependency from us125 to st175;
    // Routing (github-intake step 3; D0264 untrusted: a routed story names a Decision among its
    // targets). d0525 is the held fork that chooses the verdict surface; the resolver is issue632's.
    #Implicates dependency from us124 to dcVerdictChannelIsOneSurface;
    #Implicates dependency from us124 to d0525;
    #Implicates dependency from us125 to dcVerdictChannelIsOneSurface;
    #Implicates dependency from us125 to d0525;

    part us126 : UserStory {
        :>> id = "8ae0db72-1570-4f45-b10b-0995a1d2cb23";
        :>> title = "An adopter's .gitignore survives keel init";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "an author adopting keel on a project that already has content";
        :>> iWant = "keel init to leave my existing .gitignore lines in place and add only the keel entries it does not find";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "Confirmed at keel-cli/src/main.rs:3019: the file is written whole. Tracked as issue634, resolver dcInitAppendsToGitignore. Untrusted tier (D0264): the fix is work only once the batch routing Decision is accepted.";
    }
    #DerivedFrom dependency from us126 to st158;

    part us127 : UserStory {
        :>> id = "6fd9725e-1725-4955-90dc-2c2629e5dd35";
        :>> title = "Keel accept appends a second <id>Accept event to a Decision that already declares one, so a hand-authored Deci";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream keel project";
        :>> iWant = "the defect fixed: keel accept on a Decision whose file already declares its <id>Accept verification (a hand-authored Decision in the documented shape) records the human's result against that existing event instead of appending a second one, so the file validates after the accept";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "members/keel-write/src/write.rs:2005-2038 accept_decision_locked appends the <id>Accept verification unconditionally. A project Decision file authored by hand in the shape the docs show already carries the event; accepting it produces a duplicate declaration and a red validate. Reported downstream as GH#71 (st121, re-ingested as st138). Tracked as issue635, resolver dcAcceptFindsTheExistingEvent. Untrusted tier (D0264): work once d0526 is accepted.";
    }
    #DerivedFrom dependency from us127 to st121;

    part us128 : UserStory {
        :>> id = "204dac66-31a2-4619-b3e7-5b11fdd2bfc2";
        :>> title = "Keel validate refuses a SysML v2 file at its first named dependency because the parser requires `from` immedia";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream keel project";
        :>> iWant = "the defect fixed: keel validate accepts a SysML v2 dependency that carries a name before its from clause (`dependency name from A to B;`), as the kernel grammar does, and the parser test suite holds a named and an anonymous dependency side by side";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "members/keel-model/src/parser.rs:738-753 parse_bare_dependency demands `from` as the next token, so a kernel-valid `dependency name from A to B;` fails validate. Reported downstream as GH#72 (st125, re-ingested as st137). Tracked as issue636, resolver dcValidateReadsANamedDependency. Untrusted tier (D0264): work once d0526 is accepted.";
    }
    #DerivedFrom dependency from us128 to st125;

    part us129 : UserStory {
        :>> id = "045ad99d-5b93-416b-bd2b-173dd39126cd";
        :>> title = "Keel land --help runs the landing and pushes the current branch because land reads its first argument as the r";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream keel project";
        :>> iWant = "the defect fixed: keel land takes only the arguments its CliCommand fact declares; `--help` prints usage and exits 0 without landing, and any other unknown flag is refused with exit 2 before a single git command runs";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "keel-cli/src/main.rs:4322 hands land the bare repo_arg and members/keel-args/src/lib.rs:255-259 treats any first argument as the root, so `--help` is a root named `--help` that falls back to the cwd and the landing runs. A help request that pushes is the D0129 class (an irreversible act on a mis-parse). Reported downstream as GH#73 (st124, re-ingested as st136). Tracked as issue637, resolver dcLandRefusesAnUnknownArgument. Untrusted tier (D0264): work once d0526 is accepted.";
    }
    #DerivedFrom dependency from us129 to st124;

    part us130 : UserStory {
        :>> id = "2b142d49-8ecd-43a4-8b6f-53395497dae4";
        :>> title = "A claim commit can carry ride-along edits because keel claim does not commit and keel land does not check the";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream keel project";
        :>> iWant = "the defect fixed: keel claim commits the claim file alone in its own commit, and keel land refuses a commit whose diff touches a claim file together with any other path, naming both";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "members/keel-write/src/claim.rs:36 cmd writes the claim and commits nothing; members/keel-process/src/sync.rs:140 cmd_land has no check that a commit touching a claim file touches nothing else. The reporter's proposal (make the claim commit structurally unable to carry a ride-along) is the fix. GH#74 (st123, re-ingested as st135). Tracked as issue638, resolver dcAClaimCommitCarriesOnlyTheClaim. Untrusted tier (D0264): work once d0526 is accepted.";
    }
    #DerivedFrom dependency from us130 to st123;

    part us131 : UserStory {
        :>> id = "6cdf25e1-9c3a-485a-a42d-a20acad2b8d6";
        :>> title = "Keel record decision reports its required flags in two stages and answers --help with the first-stage refusal";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream keel project";
        :>> iWant = "the defect fixed: keel record decision with missing required inputs reports every missing flag in one message, and `keel record decision --help` prints the usage instead of the refusal";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "keel-cli/src/main.rs:1937-1952 checks one group of flags, refuses, and only on a second run checks the rest; `--help` reaches the same refusal (observed live in this session). GH#75 (st134). Tracked as issue639, resolver dcRecordDecisionNamesEveryMissingFlagAtOnce. Untrusted tier (D0264): work once d0526 is accepted.";
    }
    #DerivedFrom dependency from us131 to st134;

    part us132 : UserStory {
        :>> id = "cc5e85dd-0208-4575-bb70-2316324447d5";
        :>> title = "The activation manifest's charteredBy key has no writer, so keel onboard reports NOT CHARTERED until the autho";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream keel project";
        :>> iWant = "the defect fixed: keel activate and keel deactivate accept --charter dNNNN and write charteredBy under [processes] in .engine/contracts/activation.toml, and keel onboard step 5 names that flag instead of a hand edit";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "keel-cli/src/main.rs:3210,3337 (cmd_activation) writes [processes] active and never charteredBy; project-onboarding step 5 requires the key and the generated CLAUDE.md forbids editing .engine by hand. The only path is the one the discipline exists to avoid. GH#77 (st132). Tracked as issue640, resolver dcActivateWritesTheCharter. Untrusted tier (D0264): work once d0526 is accepted.";
    }
    #DerivedFrom dependency from us132 to st132;

    part us133 : UserStory {
        :>> id = "43914d8b-a546-496f-9028-7d6b5b849812";
        :>> title = "Pre-bash cannot see a backtick inside a keel record argument, the class keel's own docs say recurred five time";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream keel project";
        :>> iWant = "the defect fixed: The pre-bash hook's classifier recognises a keel record invocation whose double-quoted argument contains a backtick and denies it naming the argument, before the shell runs it";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "members/keel-hooks/src/lib.rs:685-748 bash_classify has no rule for a record verb whose double-quoted argument carries a backtick, so the one injection class the write path refuses after the fact is not refused before the shell substitutes it. GH#79 (st130). Tracked as issue641, resolver dcPreBashSeesABacktickInsideARecordArgument. Untrusted tier (D0264): work once d0526 is accepted.";
    }
    #DerivedFrom dependency from us133 to st130;

    part us134 : UserStory {
        :>> id = "03cd3a8e-a8f2-463c-a240-4d1b9512cef3";
        :>> title = "Guard hook-config-integrity warns on a correct hook command because it compares the literal $CLAUDE_PROJECT_DI";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream keel project";
        :>> iWant = "the defect fixed: guard hook-config-integrity expands $CLAUDE_PROJECT_DIR (and ${CLAUDE_PROJECT_DIR}) in a hook command before comparing it to the expected binary path, so a settings file written the way Claude Code documents passes";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "members/keel-guards/src/enforcement.rs:1113-1160 hook_config_integrity compares the command string at line 1148 without expanding the variable Claude Code substitutes at run time; the guard reports a wrong verdict on a right configuration. GH#80 (st129). Tracked as issue642, resolver dcHookIntegrityExpandsTheProjectDir. Untrusted tier (D0264): work once d0526 is accepted.";
    }
    #DerivedFrom dependency from us134 to st129;

    part us135 : UserStory {
        :>> id = "35874e6b-c0d5-4a49-8fe2-6931de3f56af";
        :>> title = "Two machines building the same commit stamp different build ids because rev-parse --short has no fixed width";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream keel project";
        :>> iWant = "the defect fixed: keel-cli/build.rs stamps the build commit with `git rev-parse --short=8`, so two machines building one commit report one stamp";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "keel-cli/build.rs:32 calls `git rev-parse --short` with no width, and git widens the abbreviation as the repository grows, so one commit produces 7 characters on one clone and 8 on another; the D0474 receipt keyed on the stamp then never matches across machines. GH#81 (st128). Tracked as issue643, resolver dcBuildStampsAFixedWidthCommit. Untrusted tier (D0264): work once d0526 is accepted.";
    }
    #DerivedFrom dependency from us135 to st128;

    part us136 : UserStory {
        :>> id = "4f57d214-22f2-4c02-8a16-5fa059336fef";
        :>> title = "Marker edges declared inside an action def body are silently dropped by the parser unless they are succession";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream keel project";
        :>> iWant = "the defect fixed: The parser records every `#Marker dependency from A to B;` declared inside an action def body, not only succession markers, so a #Covers edge authored beside its sitting review is indexed and traced";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "members/keel-model/src/parser.rs:366-374, the action-def-body Hash arm, handles succession markers and skips every other marker form, so 74 #Covers edges in a downstream delivery file validated and were never indexed. GH#60 (st108, re-ingested as st149). Tracked as issue644, resolver dcParserKeepsEveryMarkerInAnActionDef. Untrusted tier (D0264): work once d0526 is accepted.";
    }
    #DerivedFrom dependency from us136 to st108;

    part us137 : UserStory {
        :>> id = "855e0bbd-c2e1-4672-8ae5-66ed6d5e530b";
        :>> title = "Keel migrate stages the engine's copy over an engine file the project has extended, deleting the project's own";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream keel project";
        :>> iWant = "the defect fixed: keel migrate treats an engine-shipped file the project has extended (a schema file with project metadata defs, a process file with a project step) as a Blocker naming the file and the project's additions, instead of staging the engine's version over it";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "members/keel-process/src/migrate.rs ~590-640 resyncs engine files by staging the new version; PROJECT_OWNED (:112-126) names the files a project owns whole but nothing recognises a project's additions to an engine-owned file, so 25 project metadata defs were staged away. GH#61 (st107, re-ingested as st148). Tracked as issue645, resolver dcMigrateBlocksOnAProjectExtendedEngineFile. Untrusted tier (D0264): work once d0526 is accepted.";
    }
    #DerivedFrom dependency from us137 to st107;

    part us138 : UserStory {
        :>> id = "d0659778-6078-4c3e-a3dd-9568f0dfc954";
        :>> title = "An Issue reads as dispositioned by a confirmation Test that no human has judged";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream keel project";
        :>> iWant = "the defect fixed: issue_disposition reads a disposition only from a Test that has a TestResult judged by a human; a confirmation Test carrying acceptRisk with no result leaves the Issue open";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "members/keel-model/src/queries.rs:273-280 issue_disposition reads the disposition attribute and :306-310 closes the Issue without requiring a TestResult, so a Test declaring acceptRisk closes an Issue on the AI's declaration alone (the D0016 class). GH#62 (st106, re-ingested as st147). Tracked as issue646, resolver dcDispositionNeedsItsResult. Untrusted tier (D0264): work once d0526 is accepted.";
    }
    #DerivedFrom dependency from us138 to st106;

    part us139 : UserStory {
        :>> id = "51ece9d7-986c-4769-9db9-da74b3a0227b";
        :>> title = "Verified-or-reverted migration gates before a project's overlay can run, so an overlaid project can never migr";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream keel project";
        :>> iWant = "the defect fixed: keel migrate runs a project's declared post-migrate overlay (tools/after_migrate.py or the path named in .engine/contracts/migration.toml) between apply and gate, so the gate judges the repaired tree and a project pin survives a verified migration";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "D0336's apply-gate-revert leaves no step where a project's tools/after_migrate.py can restore the pins the apply reset; the first verified run rolled back 484 files on ~87 failures every one of which the overlay repaired. The first defect in the same report is resolved by D0519/issue609. GH#68 (st114, re-ingested as st141). Tracked as issue647, resolver dcMigrateTracksTheOverlayHook. Untrusted tier (D0264): work once d0526 is accepted.";
    }
    #DerivedFrom dependency from us139 to st114;

    part us140 : UserStory {
        :>> id = "4c4c1482-80b6-4046-824a-cb59dd7c091c";
        :>> title = "Keel record result cannot address a standalone verification because task_exists_in_pkg only looks inside actio";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream keel project";
        :>> iWant = "the defect fixed: keel record result addresses a `verification x : Test` declared at package level (not inside an action def) and appends the TestResult beside it";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "members/keel-write/src/write.rs:217-226 task_exists_in_pkg walks action defs only; append_result_locked :823-852 therefore answers unknown for a package-level `verification x : Test`. GH#69 (st113, re-ingested as st140). Tracked as issue648, resolver dcRecordResultFindsAStandaloneTest. Untrusted tier (D0264): work once d0526 is accepted.";
    }
    #DerivedFrom dependency from us140 to st113;

    part us141 : UserStory {
        :>> id = "6d485005-05a9-444b-834e-c6b5bc321209";
        :>> title = "Keel land prints landed while an authored file sits unstaged, so the session believes the work is in the tree";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream keel project";
        :>> iWant = "the defect fixed: keel land reads `git status --porcelain` after its commit and before printing landed, and refuses with the dirty paths named when anything is left unstaged or uncommitted";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "members/keel-process/src/sync.rs:203 prints the landed line after the push with no porcelain check; found in Penumbra as its issue125 where a session authored a file, landed, and the file was never committed. GH#9 (st059, re-ingested as st180). Tracked as issue649, resolver dcLandReadsPorcelainBeforeLanded. Untrusted tier (D0264): work once d0526 is accepted.";
    }
    #DerivedFrom dependency from us141 to st059;

    part us142 : UserStory {
        :>> id = "4c8fc4e5-ea2e-425c-9bb7-9fa1f1b6869d";
        :>> title = "Protected-path enforcement is bound to the tool that writes, so rm and a shell redirect bypass it and .engine/";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream keel project";
        :>> iWant = "the defect fixed: The pre-bash and pre-write hooks protect a fact surface by its path, whatever tool writes it: .engine/decisions is a tracking target, and an rm, mv or shell redirect aimed at a protected path is classified and denied like an Edit";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "members/keel-hooks/src/lib.rs:685-696 tracking_target excludes .engine/decisions and the classifier leaves rm unclassified, so the surfaces the hooks exist to protect are protected against one tool only. GH#32 (st054, re-ingested as st177). Tracked as issue650, resolver dcProtectedPathsBindToThePath. Untrusted tier (D0264): work once d0526 is accepted.";
    }
    #DerivedFrom dependency from us142 to st054;

    part us143 : UserStory {
        :>> id = "d764afb9-0449-4851-8aa5-65804da96bb1";
        :>> title = "Guard scaffold-placeholder passes while the shipped placeholder actors, Your Name among them, are still declar";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream keel project";
        :>> iWant = "the defect fixed: guard scaffold-placeholder fails on any actor keel init wrote as a placeholder (the STARTER_ACTORS names, `Your Name` included) that is still declared and still unrenamed when the project has its first non-scaffold item";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "keel-cli/src/main.rs:64 STARTER_ACTORS ships `Your Name`; members/keel-guards/src/sprints.rs:533 scaffold_placeholder and members/keel-guards/src/identity.rs:124 actors never treat those names as placeholders, so a project can record under a name nobody holds and the guard says nothing. GH#33 (st053, re-ingested as st157). Tracked as issue651, resolver dcScaffoldPlaceholderSeesTheStarterActors. Untrusted tier (D0264): work once d0526 is accepted.";
    }
    #DerivedFrom dependency from us143 to st053;

    part us144 : UserStory {
        :>> id = "c5251ebd-994f-4cec-92e1-aab9b55a6fee";
        :>> title = "Keel trace reports no edges for items that carry marker-form dependencies, while marker-census counts 111 of t";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream keel project";
        :>> iWant = "the defect fixed: keel trace reports every typed edge reaching an item in both directions, marker-form dependencies included, matching its own help text; the indexer records every `#Marker dependency` as an edge of that marker's kind";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "members/keel-model/src/queries.rs:85-100 traces only the succession dependencies members/keel-model/src/indexer.rs:255-262 records, so every marker edge is invisible to the view the help text promises walks them all. GH#34 (st046, re-ingested as st156). Tracked as issue652, resolver dcTraceWalksMarkerEdges. Untrusted tier (D0264): work once d0526 is accepted.";
    }
    #DerivedFrom dependency from us144 to st046;

    part us145 : UserStory {
        :>> id = "157a3f09-ec44-4d81-8346-524347b4d67e";
        :>> title = "Guard decision-scaffolding reports a compound Decision after it has been superseded whole by single-clause Dec";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream keel project";
        :>> iWant = "the defect fixed: guard decision-scaffolding does not report a compound Decision that a #Supersede edge has retired, whatever its date, and says how many it exempted for that reason";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "members/keel-guards/src/decisions.rs:355 compound_decisions has no exemption for a Decision a #Supersede edge has retired, so the D0303-prescribed fix (split it, supersede the compound) leaves the guard red. GH#94 (st170). Tracked as issue653, resolver dcCompoundGuardExemptsASupersededDecision. Untrusted tier (D0264): work once d0526 is accepted.";
    }
    #DerivedFrom dependency from us145 to st170;

    part us146 : UserStory {
        :>> id = "7d66dc39-fe35-4c7b-9623-30e3d098658d";
        :>> title = "Keel record decision writes and auto-accepts a compound Decision the guard will then report";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream keel project";
        :>> iWant = "the defect fixed: keel record decision runs clause_count over the decision text before writing and refuses a text enumerating two or more clauses, naming the count and D0303, so a compound Decision is never recorded and then auto-accepted";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "keel-cli/src/main.rs:1900-1980 never calls clause_count; the write lands, standing consent accepts it, and decision-scaffolding turns red on the next gate. A refusal at the write is the D0047 form of the guard. GH#93 (st171). Tracked as issue654, resolver dcRecordDecisionRefusesACompoundClause. Untrusted tier (D0264): work once d0526 is accepted.";
    }
    #DerivedFrom dependency from us146 to st171;

    part us147 : UserStory {
        :>> id = "83e513aa-ca06-4ad1-8ba3-c40bc90634e8";
        :>> title = "Keel record review records a human verdict from an agent environment that keel judge-set refuses";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream keel project";
        :>> iWant = "the defect fixed: keel record review applies the same verdict_channel_refusal as keel accept, keel reject and keel judge-set: a human verdict carried in a review file is refused from an agent environment unless a delegation grant names the recorder";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "keel-cli/src/main.rs:2448 cmd_apply_review has no verdict_channel_refusal; only accept (:3533), reject (:3811) and judge-set (:3935) call it, so the review round-trip is a fourth channel that launders the refused act. GH#92 (st172). Tracked as issue655, resolver dcReviewRoundTripRefusesAnAgentVerdict. Untrusted tier (D0264): work once d0526 is accepted.";
    }
    #DerivedFrom dependency from us147 to st172;

    part us148 : UserStory {
        :>> id = "b2cee4c7-2fc5-4015-9f52-3b389fead3fe";
        :>> title = "Keel migrate says nothing when the engine removes an enum member the project uses";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream keel project";
        :>> iWant = "the defect fixed: keel migrate enumerates the items affected when the engine removes an enum MEMBER (as it does for a removed type), naming each `Enum::member` use, and refuses until they are migrated";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "members/keel-process/src/migrate.rs:274-276 REMOVED_ENUMS lists whole enums only and the scan at :816-835 matches `{en}::`, the enum name, never a member, so a removed member reaches the project as a validate failure after apply. GH#91 (st173). Tracked as issue656, resolver dcMigrateSeesARemovedEnumMember. Untrusted tier (D0264): work once d0526 is accepted.";
    }
    #DerivedFrom dependency from us148 to st173;

    part us149 : UserStory {
        :>> id = "4e96c4ec-8cac-48a8-b5d0-d24e1bc8b53f";
        :>> title = "A UserStory recorded as a need with no Need derived from it is invisible to every lens";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream keel project";
        :>> iWant = "the defect fixed: keel show intake lists every UserStory with implication need that no Need cites through #DerivedFrom, under needStoriesWithoutNeed, so a need that was translated and never elevated is visible";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "keel show intake reports unparsed and unrouted only; a story whose implication is need and which no Need cites is neither, so the elevation D0166 prescribes can be skipped silently. GH#95 (st176). Tracked as issue657, resolver dcIntakeShowsANeedStoryWithoutANeed. Untrusted tier (D0264): work once d0526 is accepted.";
    }
    #DerivedFrom dependency from us149 to st176;

    part us150 : UserStory {
        :>> id = "73399687-0176-40b2-8edf-4f54390c04ad";
        :>> title = "Record issue refuses a resolver declared in a project action def outside backlog.sysml, and its message says '";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream keel project";
        :>> iWant = "the defect fixed: record issue --resolver accepts an action declared inside any project action def under .tracking (not only backlog.sysml and delivery/), and its refusal message names the file set searched in grammatical English";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "members/keel-model/src/resolvers.rs:9-33 declared_task_names scans .tracking/backlog.sysml and delivery/ only; the refusal wording is the reporter's quote. GH#41 (st050, re-ingested as st150). Tracked as issue658, resolver dcResolverLookupCoversEveryDef. Untrusted tier (D0264): work once d0526 is accepted.";
    }
    #DerivedFrom dependency from us150 to st050;

    part us151 : UserStory {
        :>> id = "5f8a97aa-adc5-4806-9b05-844f99711aaa";
        :>> title = "record task accepts the analyze method";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "a downstream author recording an STPA run";
        :>> iWant = "keel record task to accept method analyze as the schema declares it";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "Already fixed: issue451 is resolved by dcMethodVocabularyIsTheSchemas, which reads the method vocabulary from the schema. Verified on this tree. GH#70. Declined as new work; the reporter is answered on the issue.";
    }
    #DerivedFrom dependency from us151 to st139;

    part us152 : UserStory {
        :>> id = "ac636652-c997-428d-9ea5-98897c81cda4";
        :>> title = "The onboarding skill's Decision prose goes through a file";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "a downstream author following project-onboarding";
        :>> iWant = "the onboarding skill to pass Decision prose through --from FILE, not shell arguments";
        :>> implication = ImplicationKind::correction;
        :>> triageNote = "Premise no longer holds: keel record decision --from FILE exists since 016cadaa (D0224) and the onboarding skill names it; the remaining hazard class is tracked as dcNoShellInjectionIntoRecords. GH#76. Declined as new work.";
    }
    #DerivedFrom dependency from us152 to st133;

    part us153 : UserStory {
        :>> id = "6897e2c0-792d-485c-8faa-ce44ad3c54d9";
        :>> title = "add-task lands inside its def";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "a downstream author with a delivery def in a package";
        :>> iWant = "keel add-task to place the task inside the named def rather than after the package brace";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "Already fixed by dcAddTaskLandsInsideItsDef (passing). GH#63, ingested twice (st105, st146). Declined as new work.";
    }
    #DerivedFrom dependency from us153 to st105;

    part us154 : UserStory {
        :>> id = "e4bc5174-567e-4b60-90d0-fbbac125fe42";
        :>> title = "An open task is exempt from sprint-closure by a typed signal";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "a downstream author with an open backfill task";
        :>> iWant = "sprint-closure to exempt an open task by a typed fact, never by comment text";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "Resolved by D0260: guard 55 reads the highest-numbered sprint and the #Resolves edge, not a comment phrase. GH#64, ingested twice (st104, st145). Declined as new work.";
    }
    #DerivedFrom dependency from us154 to st104;

    part us155 : UserStory {
        :>> id = "e01e2a0f-6bb6-475b-8589-1a75c25673f5";
        :>> title = "usage.md stops naming the skills registry";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "a downstream author reading usage.md";
        :>> iWant = "usage.md to describe how a skill is registered today";
        :>> implication = ImplicationKind::convention;
        :>> triageNote = "Doc-sync: .engine/docs/usage.md:114 still names skills-registry; the line is corrected in this commit. GH#65, ingested twice (st103, st144).";
    }
    #DerivedFrom dependency from us155 to st103;

    part us156 : UserStory {
        :>> id = "46ed5b07-47c6-4fab-8a55-14834587e2da";
        :>> title = "The standing-consent thread is a bot's own text";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the project";
        :>> iWant = "nothing - this is the standing-thread boilerplate the retired GitHub channel posted";
        :>> implication = ImplicationKind::none;
        :>> triageNote = "github-actions[bot] text from the decision channel D0291 disconnected. Self-terminating; no story implication. GH#29, ingested twice (st057, st179).";
    }
    #DerivedFrom dependency from us156 to st057;

    part us157 : UserStory {
        :>> id = "87cfe47b-1291-46b9-844f-33b02556ef51";
        :>> title = "The status-report skill does not name a missing script";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "a downstream operator following the status-report skill";
        :>> iWant = "the deployed skill to run a command that exists in the project";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "Moot: report.py is embedded in the binary and the skill deployed today names keel render report; verified against the shipped skill text. GH#35, ingested twice (st045, st155). Declined.";
    }
    #DerivedFrom dependency from us157 to st045;

    part us158 : UserStory {
        :>> id = "adb7841f-d105-4ed6-babc-fe20ef6e89a6";
        :>> title = "A removed type is read from the schema, not prose";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "a downstream author pinned to 0.3.0 by a migrate refusal";
        :>> iWant = "keel migrate to detect a removed type from the schema diff rather than a prose list";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "Already fixed: issue377 resolved by dcRemovedTypeScannerReadsSchemaNotProse. GH#36, ingested twice (st044, st154). Declined as new work.";
    }
    #DerivedFrom dependency from us158 to st044;

    part us159 : UserStory {
        :>> id = "52088d37-6d57-4b05-8e22-b76e903d4add";
        :>> title = "A deactivated process's skill says so";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "a downstream author reading a deployed skill";
        :>> iWant = "sync-claude to deploy a deactivated process's skill with its state written first";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "Resolved by D0348 and dcDeactivatedSkillSaysSo. GH#37, ingested twice (st043, st153). Declined as new work.";
    }
    #DerivedFrom dependency from us159 to st043;

    part us160 : UserStory {
        :>> id = "49fcbe41-6fa6-48a7-8776-62110e99e2b9";
        :>> title = "The decision workflows are gone with their channel";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the project owner";
        :>> iWant = "the decision-issue and decision-record workflows removed with the channel they served";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "Resolved by D0291: the GitHub decision channel is disconnected; the workflows named in GH#38 were removed with it. Declined as new work.";
    }
    #DerivedFrom dependency from us160 to st152;

    part us161 : UserStory {
        :>> id = "ccefe2f2-225d-4f06-bbfd-7b385e845493";
        :>> title = "A strict project makes its initial commit";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "a downstream author on a fresh strict profile";
        :>> iWant = "the keystone lock to let a project's first commit through";
        :>> implication = ImplicationKind::bug;
        :>> triageNote = "Already fixed: issue272 (dcKeystoneLockIsAboutChangeNotArrival) and issue274 (dcObligationRecorderValidates). GH#40, ingested twice (st051, st151). Declined as new work.";
    }
    #DerivedFrom dependency from us161 to st051;

    part us162 : UserStory {
        :>> id = "b8dd39bb-7b39-4b82-ae15-685a7060b57e";
        :>> title = "A project critiques in its own vocabulary";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a non-software project";
        :>> iWant = "to gate critique on lenses my domain can answer without touching the frozen core";
        :>> implication = ImplicationKind::architecture;
        :>> triageNote = "The seven-lens canon is frozen (element.sysml:32) and critique-policy.toml constrains to it; the propriety package already states the rule for this case. Fork recorded for the human. GH#78.";
    }
    #DerivedFrom dependency from us162 to st131;

    part us163 : UserStory {
        :>> id = "6a57c33d-07b2-4bc4-8dbc-365a49f853fe";
        :>> title = "A project declares its own ceremony chain";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "a solo author working evening sittings";
        :>> iWant = "keel advance to run the gates my sprint declares, planning included, instead of six baked names";
        :>> implication = ImplicationKind::architecture;
        :>> triageNote = "advance matches six fixed names (cursor.rs, orient.rs:299-314); planning is a declared Process nothing can gate on. Fork recorded for the human. GH#82.";
    }
    #DerivedFrom dependency from us163 to st159;

    part us164 : UserStory {
        :>> id = "286aae27-202c-4aa4-8a84-074b3af146a8";
        :>> title = "A project's marker edges are traversable in views";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream STPA model";
        :>> iWant = "a TOML view to traverse the marker edges my project declares";
        :>> implication = ImplicationKind::architecture;
        :>> triageNote = "schema.rs:43,251-262 reads edge kinds from the embedded schema only; the vocabulary is derived elsewhere (D0150). Fork recorded for the human. GH#67, ingested twice (st101, st142).";
    }
    #DerivedFrom dependency from us164 to st101;

    part us165 : UserStory {
        :>> id = "1aaf2adf-399c-480a-85af-54c972abeff9";
        :>> title = "Analysis goes suspect when its architecture changes";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of a downstream STPA model";
        :>> iWant = "a UCA or safety requirement to go suspect when a controller it analyses changed after my last reassessment";
        :>> implication = ImplicationKind::need;
        :>> triageNote = "The suspicion computation follows five core edge kinds only (relationships.sysml:28-30). Fork recorded for the human: engine propagation with a stamp vs a project view. GH#66, ingested twice (st102, st143).";
    }
    #DerivedFrom dependency from us165 to st102;

    part us166 : UserStory {
        :>> id = "c08b14c7-f298-4814-97e1-9947938c3ab3";
        :>> title = "A design input has a home";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the author of an irrigation design";
        :>> iWant = "a place for a value my design consumes but does not decide, that a constraint can be gated against";
        :>> implication = ImplicationKind::need;
        :>> triageNote = "Every existing type mis-signals (the walk in st055 holds on each row); the non-core package is the precedent. Fork recorded for the human. GH#31, ingested twice (st055, st178).";
    }
    #DerivedFrom dependency from us166 to st055;

    part us167 : UserStory {
        :>> id = "8ef92d45-7765-4880-9d05-c5702ae74fdd";
        :>> title = "A proposal document is ingested as needs and use cases";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the project owner";
        :>> iWant = "a proposal written by another AI process ingested all the way down to needs and use cases";
        :>> implication = ImplicationKind::none;
        :>> triageNote = "Historical directive (2026-08-21) acted on at the time; docs/proposals was the intake path before D0166 made statements tracked items. No open implication.";
    }
    #DerivedFrom dependency from us167 to st017;

    part us168 : UserStory {
        :>> id = "4c164252-e468-4548-8ef1-ff3de923e396";
        :>> title = "Design proceeds in sprints with BDD rigor";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the project owner";
        :>> iWant = "the design carried out in sprints with full architecture and BDD/TDD verification";
        :>> implication = ImplicationKind::convention;
        :>> triageNote = "Historical directive; the Delivery workflow and cucumber suites are the standing form. No open implication.";
    }
    #DerivedFrom dependency from us168 to st027;

    part us169 : UserStory {
        :>> id = "7a52d1ff-cd7a-48ef-a2db-90092e59b64b";
        :>> title = "The report is an HTML template with a skill";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the project owner";
        :>> iWant = "an HTML template and a skill in .claude for the report";
        :>> implication = ImplicationKind::none;
        :>> triageNote = "Historical directive; keel render report --html and the deployed skills are its form. No open implication.";
    }
    #DerivedFrom dependency from us169 to st028;

    part us170 : UserStory {
        :>> id = "b604cca1-b087-4556-9c2a-a2560d9314e5";
        :>> title = "Real assurance, not stricter ceremony";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the project owner";
        :>> iWant = "process launching, verification and assurance managed strictly, with real assurance behind it";
        :>> implication = ImplicationKind::priority;
        :>> triageNote = "Historical direction that the hardening and attestation lenses answer. No open implication.";
    }
    #DerivedFrom dependency from us170 to st031;

    part us171 : UserStory {
        :>> id = "3e6e30dd-2f18-4f8c-9f78-a05aa21bb157";
        :>> title = "GitHub issues are ingested and fixed";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the project owner";
        :>> iWant = "the open GitHub issues ingested and fixed";
        :>> implication = ImplicationKind::process;
        :>> triageNote = "D0263 made GitHub intake a process; this batch is its run. Routed to D0263.";
    }
    #DerivedFrom dependency from us171 to st060;

    part us172 : UserStory {
        :>> id = "e8c513e3-754b-4b23-824d-9bc5f21559fd";
        :>> title = "Pending decisions come with a link to the brief";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the project owner";
        :>> iWant = "a link to the executive brief whenever there are decisions for me to make";
        :>> implication = ImplicationKind::process;
        :>> triageNote = "D0269 (decision surfacing) is that process. Routed.";
    }
    #DerivedFrom dependency from us172 to st061;

    part us173 : UserStory {
        :>> id = "e11c6a8a-7bdc-4226-a3ac-369f0a6028b3";
        :>> title = "The STPA diagram reads top-down with straight edges";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the project owner";
        :>> iWant = "the STPA diagram with authority descending, control actions down, feedback up, ortholinear edges and semicircles at crossings";
        :>> implication = ImplicationKind::designChange;
        :>> triageNote = "D0285 holds the five conventions by construction. Routed.";
    }
    #DerivedFrom dependency from us173 to st067;

    part us174 : UserStory {
        :>> id = "662fdf54-daa8-4f86-9c44-120804fcd0a4";
        :>> title = "GitHub issue notifications are disconnected";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the project owner";
        :>> iWant = "no GitHub issue notifications; the pipeline disconnected";
        :>> implication = ImplicationKind::process;
        :>> triageNote = "D0291 disconnected the channel. Routed.";
    }
    #DerivedFrom dependency from us174 to st088;

    part us175 : UserStory {
        :>> id = "deb9e7f2-8c37-415d-b826-c398fe1e1c23";
        :>> title = "The twelve downstream issues are all work";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the project owner";
        :>> iWant = "the twelve downstream issues routed as work because they are essentially mine";
        :>> implication = ImplicationKind::attestation;
        :>> triageNote = "The word that routed GH#42-53 under D0292; recorded there as the quote receipt. Routed.";
    }
    #DerivedFrom dependency from us175 to st089;

    part us176 : UserStory {
        :>> id = "5cde49ac-f97d-4dfc-8c26-937806b6f187";
        :>> title = "Go";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the project owner";
        :>> iWant = "the proposed work to proceed";
        :>> implication = ImplicationKind::none;
        :>> triageNote = "A go-ahead word in chat; no standalone implication.";
    }
    #DerivedFrom dependency from us176 to st091;

    part us177 : UserStory {
        :>> id = "4944e95d-9e7c-484b-a2a6-aa7fb84c1598";
        :>> title = "A brief grapples with implications, not administration";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the project owner";
        :>> iWant = "each decision brief to show the implications and real technical changes with logical clarity, fewer references, stronger diagrams";
        :>> implication = ImplicationKind::correction;
        :>> triageNote = "D0377 (the terse, capped brief) and the judgment-request-quality guard are the standing answer; the logic exhibits in the brief builder came from these words. Routed to D0377.";
    }
    #DerivedFrom dependency from us177 to st099;

    part us178 : UserStory {
        :>> id = "62d72fc1-046f-440d-b8a0-df448247eff1";
        :>> title = "A meta-analysis of the last week's pace";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the project owner";
        :>> iWant = "to know whether performance tanked in the last week and why, and CLAUDE.md tightened";
        :>> implication = ImplicationKind::question;
        :>> triageNote = "Answered at the time from the sprint record; CLAUDE.md was compressed. The measurable half is st119's.";
    }
    #DerivedFrom dependency from us178 to st118;

    part us179 : UserStory {
        :>> id = "526a6de9-14bb-4430-94d0-692d49eb9417";
        :>> title = "Velocity needs points on every item";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the project owner";
        :>> iWant = "points applied to items in sprint planning so velocity is measurable, and a highly parallel agentic flow tried";
        :>> implication = ImplicationKind::priority;
        :>> triageNote = "D0038 and D0044 hold the estimation model; D0425 holds the subagent split. Routed to D0038.";
    }
    #DerivedFrom dependency from us179 to st119;

    part us180 : UserStory {
        :>> id = "6e449c6d-59d6-4a40-bba4-4302938c6d15";
        :>> title = "Backslash edits go through one wrapper";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the project owner";
        :>> iWant = "the trailing-backslash problem solved once by a wrapper";
        :>> implication = ImplicationKind::correction;
        :>> triageNote = "D0386: scripts/textpatch.py is the wrapper and the pre-bash hook denies the heredoc form. Routed.";
    }
    #DerivedFrom dependency from us180 to st127;

    part us181 : UserStory {
        :>> id = "8cc61113-3552-4bb5-aefb-57aa59b73e01";
        :>> title = "Migration is smooth for downstream projects";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the project owner";
        :>> iWant = "keel migrate to stop refusing downstream projects; migration should be smooth";
        :>> implication = ImplicationKind::priority;
        :>> triageNote = "D0519 (the project-migration writes its own evidence) came from these words; the remaining migrate defects in this batch (GH#61, GH#68, GH#91) are tracked against it. Routed to D0519 and the batch Decision.";
    }
    #DerivedFrom dependency from us181 to st167;

    part us182 : UserStory {
        :>> id = "d168580e-2c0f-435d-a2d0-df10df991f77";
        :>> title = "Downstream projects are made whole";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the project owner";
        :>> iWant = "to continue and make downstream projects whole";
        :>> implication = ImplicationKind::priority;
        :>> triageNote = "The directive this batch executes; the batch routing Decision derives from it.";
    }
    #DerivedFrom dependency from us182 to st168;

    // Routing (github-intake step 3; D0264 untrusted: every routed story names a Decision among its
    // targets). d0526 routes the batch; the five forks are d0527, d0528, d0529, d0530, d0531.
    #DerivedFrom dependency from us127 to st138;
    #Implicates dependency from us127 to dcAcceptFindsTheExistingEvent;
    #Implicates dependency from us127 to d0526;
    #DerivedFrom dependency from us128 to st137;
    #Implicates dependency from us128 to dcValidateReadsANamedDependency;
    #Implicates dependency from us128 to d0526;
    #DerivedFrom dependency from us129 to st136;
    #Implicates dependency from us129 to dcLandRefusesAnUnknownArgument;
    #Implicates dependency from us129 to d0526;
    #DerivedFrom dependency from us130 to st135;
    #Implicates dependency from us130 to dcAClaimCommitCarriesOnlyTheClaim;
    #Implicates dependency from us130 to d0526;
    #Implicates dependency from us131 to dcRecordDecisionNamesEveryMissingFlagAtOnce;
    #Implicates dependency from us131 to d0526;
    #Implicates dependency from us132 to dcActivateWritesTheCharter;
    #Implicates dependency from us132 to d0526;
    #Implicates dependency from us133 to dcPreBashSeesABacktickInsideARecordArgument;
    #Implicates dependency from us133 to d0526;
    #Implicates dependency from us134 to dcHookIntegrityExpandsTheProjectDir;
    #Implicates dependency from us134 to d0526;
    #Implicates dependency from us135 to dcBuildStampsAFixedWidthCommit;
    #Implicates dependency from us135 to d0526;
    #DerivedFrom dependency from us136 to st149;
    #Implicates dependency from us136 to dcParserKeepsEveryMarkerInAnActionDef;
    #Implicates dependency from us136 to d0526;
    #DerivedFrom dependency from us137 to st148;
    #Implicates dependency from us137 to dcMigrateBlocksOnAProjectExtendedEngineFile;
    #Implicates dependency from us137 to d0526;
    #DerivedFrom dependency from us138 to st147;
    #Implicates dependency from us138 to dcDispositionNeedsItsResult;
    #Implicates dependency from us138 to d0526;
    #DerivedFrom dependency from us139 to st141;
    #Implicates dependency from us139 to dcMigrateTracksTheOverlayHook;
    #Implicates dependency from us139 to d0526;
    #DerivedFrom dependency from us140 to st140;
    #Implicates dependency from us140 to dcRecordResultFindsAStandaloneTest;
    #Implicates dependency from us140 to d0526;
    #DerivedFrom dependency from us141 to st180;
    #Implicates dependency from us141 to dcLandReadsPorcelainBeforeLanded;
    #Implicates dependency from us141 to d0526;
    #DerivedFrom dependency from us142 to st177;
    #Implicates dependency from us142 to dcProtectedPathsBindToThePath;
    #Implicates dependency from us142 to d0526;
    #DerivedFrom dependency from us143 to st157;
    #Implicates dependency from us143 to dcScaffoldPlaceholderSeesTheStarterActors;
    #Implicates dependency from us143 to d0526;
    #DerivedFrom dependency from us144 to st156;
    #Implicates dependency from us144 to dcTraceWalksMarkerEdges;
    #Implicates dependency from us144 to d0526;
    #Implicates dependency from us145 to dcCompoundGuardExemptsASupersededDecision;
    #Implicates dependency from us145 to d0526;
    #Implicates dependency from us146 to dcRecordDecisionRefusesACompoundClause;
    #Implicates dependency from us146 to d0526;
    #Implicates dependency from us147 to dcReviewRoundTripRefusesAnAgentVerdict;
    #Implicates dependency from us147 to d0526;
    #Implicates dependency from us148 to dcMigrateSeesARemovedEnumMember;
    #Implicates dependency from us148 to d0526;
    #Implicates dependency from us149 to dcIntakeShowsANeedStoryWithoutANeed;
    #Implicates dependency from us149 to d0526;
    #DerivedFrom dependency from us150 to st150;
    #Implicates dependency from us150 to dcResolverLookupCoversEveryDef;
    #Implicates dependency from us150 to d0526;
    #Implicates dependency from us151 to dcMethodVocabularyIsTheSchemas;
    #Implicates dependency from us151 to d0526;
    #Implicates dependency from us152 to dcNoShellInjectionIntoRecords;
    #Implicates dependency from us152 to d0526;
    #DerivedFrom dependency from us153 to st146;
    #Implicates dependency from us153 to dcAddTaskLandsInsideItsDef;
    #Implicates dependency from us153 to d0526;
    #DerivedFrom dependency from us154 to st145;
    #Implicates dependency from us154 to d0260;
    #DerivedFrom dependency from us155 to st144;
    #Implicates dependency from us155 to d0526;
    #DerivedFrom dependency from us156 to st179;
    #DerivedFrom dependency from us157 to st155;
    #Implicates dependency from us157 to d0526;
    #DerivedFrom dependency from us158 to st154;
    #Implicates dependency from us158 to dcRemovedTypeScannerReadsSchemaNotProse;
    #Implicates dependency from us158 to d0526;
    #DerivedFrom dependency from us159 to st153;
    #Implicates dependency from us159 to d0348;
    #Implicates dependency from us159 to dcDeactivatedSkillSaysSo;
    #Implicates dependency from us160 to d0291;
    #DerivedFrom dependency from us161 to st151;
    #Implicates dependency from us161 to dcKeystoneLockIsAboutChangeNotArrival;
    #Implicates dependency from us161 to d0526;
    #Implicates dependency from us162 to d0527;
    #Implicates dependency from us163 to d0528;
    #DerivedFrom dependency from us164 to st142;
    #Implicates dependency from us164 to d0529;
    #DerivedFrom dependency from us165 to st143;
    #Implicates dependency from us165 to d0530;
    #DerivedFrom dependency from us166 to st178;
    #Implicates dependency from us166 to d0531;
    #Implicates dependency from us171 to d0263;
    #Implicates dependency from us172 to d0269;
    #DerivedFrom dependency from us173 to st068;
    #DerivedFrom dependency from us173 to st069;
    #Implicates dependency from us173 to d0285;
    #Implicates dependency from us174 to d0291;
    #Implicates dependency from us175 to d0292;
    #DerivedFrom dependency from us177 to st100;
    #Implicates dependency from us177 to d0377;
    #Implicates dependency from us179 to d0038;
    #Implicates dependency from us180 to d0386;
    #Implicates dependency from us181 to d0519;
    #Implicates dependency from us181 to d0526;
    #Implicates dependency from us182 to d0526;
    #Implicates dependency from us126 to dcInitAppendsToGitignore;
    #Implicates dependency from us126 to d0526;
}
