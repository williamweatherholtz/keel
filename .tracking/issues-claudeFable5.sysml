// Per-actor issues records for claudeFable5 (issue210/srDcPerActorWriteTargets): new records
// route here so concurrent contributors never conflict at a shared append anchor.
package IssuesClaudeFable5 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    part issue214 : Issue {
        :>> id = "a6db1f9a-75cd-4d20-8ff8-c0030e6ca1fa";
        :>> title = "per-actor routing probe A";
        :>> createdAt = "2026-08-22";
        :>> createdBy = "claudeFable5";
        :>> description = "issue210's live verification probe, kept as the record of its own routing: recorded through keel record issue AFTER the per-actor change and required to land in issues-claudeFable5.sysml with the next globally-allocated number - which it did, making this Issue both the test and the evidence. Resolved by dcPerActorAppendFiles alongside issue210 itself.";
        :>> discoveredInField = false;
        :>> severity = Severity::Low;
    }
    #Resolves dependency from dcPerActorAppendFiles to issue214;

    part issue215 : Issue {
        :>> id = "67cedf7a-c673-485e-a85d-3e4fc0efc046";
        :>> title = "The allocation layer is stale and actively served: June components, 26 edges for 102 SRs, a retired tool described as the single source of state";
        :>> createdAt = "2026-08-22";
        :>> createdBy = "claudeFable5";
        :>> description = "Panel R1, found independently by all five panelists. The allocIfaceGate confirmation was TRUE at abb033a (June) and is false at HEAD with nothing re-firing the obligation - the K10 reprocess gap applied to the engine's own architecture phase; trace-need serves the stale closure as current truth (EHZ8's open instance). Direction Decision D0194 queued (supersede + charter-chain/registry mechanism + coverage leg).";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcAllocationRefresh to issue215;

    part issue216 : Issue {
        :>> id = "93fb02ab-fa22-43df-ab67-5c9d10424323";
        :>> title = "pmMeasureDoD was stamped pass with its promotion-review criterion unmet - a hollow pass on the anti-hollow-pass invariant";
        :>> createdAt = "2026-08-22";
        :>> createdBy = "claudeFable5";
        :>> description = "Panel R1, automotive finding 1 (their flip set). The DoD's text requires 'at least one advisory-to-blocking promotion or non-promotion decision cites >= 3 sprints of this evidence'; no such decision exists, and the ledger was 2 sessions old when R1 was stamped (judgedBy claudeFable5 - my stamp, the exact EHZ4 class). REOPENED honestly: pmMeasureDoDR2 fail at HEAD returns pmMeasure to the frontier, where it waits for the promotion/non-promotion Decision the guided profile's evidence bar (D0180) defines. The machinery clauses all hold; the item is open on the review clause alone.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from pmMeasure to issue216;

    part issue217 : Issue {
        :>> id = "d216018a-8fee-40cf-8b63-cbbdc0af30f0";
        :>> title = "31 accepted decisions carried a PROPOSED header - prose restating the status field, gone stale";
        :>> createdAt = "2026-08-22";
        :>> createdBy = "claudeFable5";
        :>> description = "Panel R1, automotive finding 5 (D0105 dual-truth class; it briefly misled the reviewer). FIXED in-sprint: keel accept now rewrites the scaffolded header on acceptance, and the 31 existing files were swept (the header now points at the status field instead of restating it).";
        :>> discoveredInField = false;
        :>> severity = Severity::Low;
    }
    #Resolves dependency from storyPanelRound1 to issue217;

    part issue218 : Issue {
        :>> id = "d7736a07-a61a-4242-aa91-039e241368c7";
        :>> title = "The code registry omitted the human channel, the K15 backstop, the git spawn point, the hook dispatch, and the run lifecycle";
        :>> createdAt = "2026-08-22";
        :>> createdBy = "claudeFable5";
        :>> description = "Panel R1 (SE finding 3, ARCH finding 6, AERO finding 4): against the registry's own criterion - units whose failure would matter - the AI write channel was risk-classified while serve.rs (human-judgment endpoints), history.rs (audit-history), gitx.rs, cmd_hook, and launcher.rs were not. FIXED in-sprint: five CodeElements added with riskClass/invariantSafety/stpaRole judgments (registry 12 -> 17).";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from storyPanelRound1 to issue218;

    part issue219 : Issue {
        :>> id = "f0a76d4a-0a1c-4df6-9674-37714ff0e106";
        :>> title = "The hook watchdog exited 0 with no ledger line - the one fail-SILENT branch inside the instrumented layer";
        :>> createdAt = "2026-08-22";
        :>> createdBy = "claudeFable5";
        :>> description = "Panel R1, robotics finding 4: a wedged hook vanished without a trace, invisible to D0180's single instrumentation path and to K2's fail-loud inventory. FIXED in-sprint: the watchdog appends event=hook-watchdog-timeout before its mandated exit 0; declared in control-events.toml; guard 45's two-way diff covers it.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from storyPanelRound1 to issue219;

    part issue220 : Issue {
        :>> id = "0f6bddc4-7bb5-42b2-912e-459c5ce07d13";
        :>> title = "view.rs is a flagged god-module with no chartered follow-through";
        :>> createdAt = "2026-08-22";
        :>> createdBy = "claudeFable5";
        :>> description = "Panel R1, architecture finding 3: 8159 lines, ~235 definitions, depended on by guards/orient/claims/arch, marked #Restructure by the registry (Ca=4, I=0.20) with no backlog item or accept-risk Decision - a standing flag with no resolving edge, the untriaged-flag failure the engine forbids for Issues. Chartered: dcViewRsRestructure (split along lens seams, or record the accept-risk Decision).";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcViewRsRestructure to issue220;

    part issue221 : Issue {
        :>> id = "d0c6bfe8-365e-4f80-b88a-b9d3069b4eb4";
        :>> title = "EHZ9's controls trace to no standing requirement - removing the release gate would violate nothing recorded";
        :>> createdAt = "2026-08-22";
        :>> createdBy = "claudeFable5";
        :>> description = "Panel R2, aerospace finding 1 (MINOR, unanimous-accept residue): the ungated-outward-publish hazard's controls (release gate, pre-push check, land's merged-tree gate) have no invariant/SR anchor - the K-set has no outward-publish member. Resolver: D0195 clause 8 (added round 2) charters the SR anchor under dcControlMapView's delivery.";
        :>> discoveredInField = false;
        :>> severity = Severity::Low;
    }
    #Resolves dependency from dcControlMapView to issue221;

    part issue222 : Issue {
        :>> id = "a33e001f-489b-4a17-a3ba-b71705c4d9b5";
        :>> title = "GATE_ORDER hand-duplicated the declared ceremony successions - two homes for one fact";
        :>> createdAt = "2026-08-22";
        :>> createdBy = "claudeFable5";
        :>> description = "Panel R2, robotics finding 2 (MINOR): guards.rs GATE_ORDER is a compiled constant duplicating the succession chain declared in .engine/workflows/delivery.sysml; post-ingestion both live in the model and a CR editing the declared order would leave the guard silently enforcing the old one. FIXED in-sprint: gate_order_matches_the_declared_successions parses the declared chain at compile time and fails the build on divergence.";
        :>> discoveredInField = false;
        :>> severity = Severity::Low;
    }
    #Resolves dependency from storyPanelRound1 to issue222;

    part issue223 : Issue {
        :>> id = "77e4e057-9c16-4e55-8d37-e699c8a057af";
        :>> title = "The deck presents a fork Decision as a bare Sign card, soliciting an unbindable acceptance";
        :>> createdAt = "2026-08-22";
        :>> createdBy = "claudeFable5";
        :>> description = "d0192 enumerates OPTION A and OPTION B and states 'accepting this Decision means choosing ONE'; the deck rendered it as a single Sign button. wweatherholtz tapped it (inbox row uid 5b1d2b0e, At 2026-08-22T06:54:40Z, deck HEAD 3c9645e) and the tap could not be recorded - the gesture was solicited and then wasted, exactly the authoring-friction class D0054 names. The control (D0047): the deck detects OPTION-enumerating proposed decisions and renders one sign button per option, stamping the chosen option into the note so the recorded acceptance carries the choice.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcDeckForkOptions to issue223;

    part issue224 : Issue {
        :>> id = "1a148a03-fc88-4705-8bc9-f36c9d6a9477";
        :>> title = "The recording delegation covers acceptances only while the practice also records confirmation flips";
        :>> createdAt = "2026-08-22";
        :>> createdBy = "claudeFable5";
        :>> description = "srK06a completeness critique (srK06aCompl1, sprint 421): D0192 OPTION A declared the delegation and its quote-substance check for decisionAcceptance, but the identical practice - AI records the human's judgment on their verbatim words - also flips method=confirmation DoD results (sprint 420's dcHumanChannelDelegationRuleDoD flip is the live example), and no [confirmationRecord] policy class or substance demand exists there. Same asymmetry D0192 closed, one level down. Design note: a DoD result carries no note field, so where the quote lives is a design question, not a copy of the acceptance check.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcConfirmationRecordDelegation to issue224;

    part issue225 : Issue {
        :>> id = "9000ce80-8947-4391-9976-c4f5d76dcc0f";
        :>> title = "Parser proptest identifier strategy generates reserved words - latent CI flake";
        :>> createdAt = "2026-08-22";
        :>> createdBy = "claudeFable5";
        :>> description = "The sprint-421 push went CI-red on prop_part_string_attr: the [a-zA-Z_][a-zA-Z0-9_]{0,31} strategy generated 'doc', which the lexer keywords and the parser correctly refuses as a bare identifier. No commit in the push touched the parser - the random seed found a latent generator defect. Control (D0047): the lexer exposes is_reserved_word (single source: the keyword match itself) and the strategy filters through it, so the generator cannot drift from the grammar; a deterministic test pins the refusal.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcProptestReservedWords to issue225;

    part issue226 : Issue {
        :>> id = "b9aa7ad7-e1b0-40cb-9462-d629c5863ad8";
        :>> title = "The enforcement contract claimed D0108's ownership rule was unchecked - stale since sprint 293";
        :>> createdAt = "2026-08-22";
        :>> createdBy = "claudeFable5";
        :>> description = "process-enforcement.toml [distributedCollaboration] said 'Nothing checks it today. This is the entry I would most expect a reviewer to act on' - but the ownership guard has existed since sprint 293 (1b2fd4f, 2026-08-15): staged-diff vs HEAD, item-level field comparison against the bound actor, D0067 migration keystone exemption. A dual truth (D0018): the hardening view faithfully reported a stale declaration, and sprint 421 chartered dcOwnershipDiffGuard to build a guard that already existed. The contract text was never swept when the guard landed - a doc-sync miss on a CONTRACT surface. Control: the unit now asserts the constraint (D0139(D) unit-ownership), so the enforcement map computes from the model instead of restating it in prose.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcOwnershipDiffGuard to issue226;

    part issue227 : Issue {
        :>> id = "275707d4-a2be-4827-a539-98dc7ae8a677";
        :>> title = "authority-queue and sitting-coverage disagree about the sitting-review obligation";
        :>> createdAt = "2026-08-22";
        :>> createdBy = "claudeFable5";
        :>> description = "Process-value panel (human-factors lens), verified live by the convener: authority-queue reports '279 sprint(s) awaiting a sitting review, waitingDays 65, escalated true' while sitting-coverage reports due 6 with 313 grandfathered under the D0155 human attestation. Two computed views, one obligation, two answers - the exact dual-computation class D0018 forbids, in the pair of views the human uses to size their own duty.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcSittingViewParity to issue227;

    part issue228 : Issue {
        :>> id = "66136aa6-703d-48ac-91db-602246631061";
        :>> title = "Batch acknowledgments count as read reviews; the human override rate is unmeasured; escalation is toothless";
        :>> createdAt = "2026-08-22";
        :>> createdBy = "claudeFable5";
        :>> description = "Process-value panel, 4 of 9 lenses convergent. VERIFIED: 65 of 105 covered sittings (62 percent) were signed in two batch chat utterances against deck cards rendering a title plus the literal string 'unreviewed'; zero reject/needs-work sitting verdicts exist in the record; the 279-sprint queue has been escalated 65 days while ~145 sprints landed on top of it. sitting-coverage counts a batch acknowledgment identically to a read review - the same one-number conflation the EXAMINED/EXERCISED split exists to prevent. The record is honest about what happened (verbatim batch words); the COUNT is not.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcOversightRedesign to issue228;

    part issue229 : Issue {
        :>> id = "d182fb95-f1e5-470a-9313-d703d2f9fe7b";
        :>> title = "Claims gate selection but not landing, and claimedAt is authored where nothing else is trusted";
        :>> createdAt = "2026-08-22";
        :>> createdBy = "claudeFable5";
        :>> description = "Process-value panel (multi-agent lens), two concrete interleavings: (1) A claims, goes stale, B claims and works, A resumes and LANDS - no holdership re-check exists at land, so both land green and duplication surfaces only on contradiction; (2) holdership = earliest un-expired claimedAt, and claimedAt is claimer-authored, never validated against the introducing commit's ancestry - a backdated claim steals holdership deterministically on every clone. The repo's own doctrine (D0013: git ancestry is the clock) supplies the fix and is not applied to claims.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcClaimLandCheck to issue229;

    part issue230 : Issue {
        :>> id = "4bfd8897-7f1c-4358-87bf-2776c1b97ba4";
        :>> title = "Zero advisory blocks is uninformative: the ledger cannot distinguish deterrence from dead weight";
        :>> createdAt = "2026-08-22";
        :>> createdBy = "claudeFable5";
        :>> description = "Process-value panel - empiricist, skeptic and AI-safety lenses independently: the fire-ledger records fires (1,164 pre-bash, 0 blocks) but not advisory content or whether the flagged action changed, so D0197's declared revisit condition is structurally untriggerable and the 'advisory tiers deter' hypothesis has no counterfactual arm. A monitor whose catches cannot be counted is indistinguishable from a blind one.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcAdvisoryInstrumentation to issue230;

    part issue231 : Issue {
        :>> id = "982b984a-105c-41e7-bfea-fec614644769";
        :>> title = "The parser TCB has no ratchet: the skipped-statement count can grow silently";
        :>> createdAt = "2026-08-22";
        :>> createdBy = "claudeFable5";
        :>> description = "Process-value panel (formal-methods lens). The parser is the trusted computing base of every view and guard - a statement class it skips is invisible to the whole enforcement stack (incident lineage: issue102 56 absent use-case usages, issue109 dangling endpoints only the kernel saw, issue119 edge-kind drift both directions, issue221-era bare-dependency skip). parser-coverage WARNS at 7 skipped today, but nothing goes red if the count grows, and no property-based round-trip pins skip-freedom over generated corpora.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcParserCoverageRatchet to issue231;

    part issue232 : Issue {
        :>> id = "9ddea0e0-6144-4ea2-be50-b51f0f83497f";
        :>> title = "claim-ancestry's verdict depended on clone depth - CI-red on its first push";
        :>> createdAt = "2026-08-22";
        :>> createdBy = "claudeFable5";
        :>> description = "Guard 47 resolved a claim's introducing commit with git log -S; on CI's default depth-1 checkout the oldest VISIBLE commit is the clone boundary, so the guard flagged claudeOpus5's honest 2026-08-15 claim as backdated (intro misread as 2026-08-22) while the full-history local clone passed - a depth-dependent verdict, the exact K15 machine-dependence the guard cites D0013 to prevent, caught by CI within minutes of landing. Control: the guard SKIPS LOUDLY on shallow repos (absence of history is a state, not a violation - the enrollment-binding precedent) and ci.yml checks out fetch-depth 0 so the real check runs where it matters.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcClaimLandCheck to issue232;

    part issue233 : Issue {
        :>> id = "16880abf-9d8b-43fa-b453-d092aaa1ae3d";
        :>> title = "Fork cards highlighted every option as selected - eleven contradictory taps on one decision";
        :>> createdAt = "2026-08-23";
        :>> createdBy = "claudeFable5";
        :>> description = "Their report, 2026-08-23: 'deck shows multiple sign options as being selected.' Root cause: the accept-styling CSS keyed on data-v=accept, shared by EVERY option button on a fork card, so any accepted fork lit all its options; the human fought the UI and the inbox recorded eleven d0201 rows with contradictory options (A/B/C within seconds) - none of which can bind. The unbindable-gesture class (issue223) recurring one level down: the tap bound correctly in DATA (data-opt-chosen) but lied in PIXELS.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcDeckJudgeableCards to issue233;

    part issue234 : Issue {
        :>> id = "8cad950c-c902-44a2-b2ba-a857c3539872";
        :>> title = "Decision cards carried no deciding context - a signature could only be a trust gesture";
        :>> createdAt = "2026-08-23";
        :>> createdBy = "claudeFable5";
        :>> description = "Their words, 2026-08-23: 'there is not sufficient context to make these decisions generally... it's not even clear what's really being proposed because there are so many references and obscure change or value implications.' The acceptance card rendered title + 'proposed - needs your signature' and nothing else; the D0200 judgeable-cards finding applied to decisions as much as sittings, and the panel's rubber-stamp evidence was partly THIS.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcDeckJudgeableCards to issue234;

    part issue235 : Issue {
        :>> id = "d3ea8a30-3790-4151-92ca-85f0c455bb43";
        :>> title = "The substance guard rejects a one-letter fork answer - the channel's own gesture wasn't recognized";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "D0208 redTeamScope was answered 'A' on GitHub issue #3; the recorder ran keel accept (note: OPTION A - their words verbatim 'A' (GitHub comment <url>...)) but confirmation-authenticity FAILED: 'A' is no >=10-char quoted span and the scanner's gesture keywords were only deck/console. A GitHub comment is 2FA-authenticated, timestamped, immutably logged - the D0205/D0201-B conclusion that it subsumes the device HMAC - so a cited GitHub comment IS a valid channel gesture. The guard was refusing the very channel this project just built. Fix: quotes_conversational_words recognizes a github comment / github.com citation as a gesture; nothing pushed, the human's answer is safe in the thread and re-records via the sweeper once the fix lands.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcGithubGestureSubstance to issue235;

    part issue236 : Issue {
        :>> id = "3862f78b-2c11-46e2-868a-69454d6792fe";
        :>> title = "An agent can disarm a blocking guard rule with no signed decision - the monitor was modifiable by the monitored";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "D0208 control evaluation (OPTION A, scripted probes in a throwaway worktree), the panel-predicted finding made concrete: committing a flip of confirmationAuthenticityRule from RuleSeverity::blocking to warning in .engine/rules/rules.sysml passed process-change (scanned 0 - the rules dir was outside is_process_def), engine-lint, and the whole gate. An agent could silently downgrade any control it enforces. The AI-safety lens named exactly this ('the agent builds its own guards... the monitor is modifiable by the monitored'); the evaluation the human authorized found it on the first pass. Control: .engine/rules/ joins the process-change keystone (dcRulesUnderKeystone), so weakening a rule needs a human-signed marked Decision.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcRulesUnderKeystone to issue236;

    part issue237 : Issue {
        :>> id = "61c9980c-962e-4ff3-8159-9ca65d652c1f";
        :>> title = "A dangling judgedAgainst SHA passes the commit gate when the result is unreferenced";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "D0208 control eval: a TestResult with judgedAgainst = 000...0 (a SHA that never resolves) passed validate+guard. The tracked case is caught elsewhere - orient invalidEvidence flags a done task whose evidence SHA dangles, and CI audit-history re-derives per commit - but the commit gate itself does not check SHA resolvability, so an orphan garbage-SHA result lands green. LOW because an unreferenced result computes nothing; recorded so the gap is visible rather than assumed covered.";
        :>> discoveredInField = false;
        :>> severity = Severity::Low;
    }
    #Resolves dependency from dcDanglingResultAtGate to issue237;

    part issue238 : Issue {
        :>> id = "c1263af4-f7ed-4c81-b28b-9ce399d88a90";
        :>> title = "A decision accepted before its push bypasses the githubChannel - the human loses the override surface";
        :>> createdAt = "2026-08-23";
        :>> createdBy = "claudeFable5";
        :>> description = "Caught by the human: I told them D0209 was overridable 'on its GitHub issue' but no issue existed. D0205's githubChannel (open_decision_issues.py) opens an override-thread issue only for PROPOSED decisions. When a decision is accepted locally (serve API / keel accept) before it is pushed, it is already 'accepted' the moment the workflow runs, so the proposed-only opener skips it and never creates the override thread. Confirmed: d0205=1 issue, d0206=1, d0207=0, d0208=1, d0209=0 - the two I had accepted locally were exactly the two with no override surface. The channel D0205 built to give the human a standing override could be silently bypassed by my own acceptance path. Manually opened #4 (D0207) and #5 (D0209) as the immediate remediation; the durable control is dcOverrideThreadEveryAccept.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcOverrideThreadEveryAccept to issue238;


    part issue239 : Issue {
        :>> id = "3c604c51-a958-4827-a79b-c9d1a9c22830";
        :>> title = "The honest-surface invariant is bound to the console and unbound on the CLI/JSON authority";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "Adversarial panel (9 lenses, 2026-08-24) on the adoption/portability/elicitation brief. The panel's most valuable reframing: three of the four drift mechanisms the brief named are not independent - they are three violations of ONE already-accepted invariant. nConsoleNeverMisleads (N-C2, human-accepted 2026-08-16, 'looks great, proceed') requires that the absence of information always be distinguishable from information ABOUT absence, that a view which could not be computed never render as a negative result, and that stale content never be shown as current. Its ONLY satisfying requirement, srConsoleViewStatusExplicit, is scoped to 'every view the surface renders' - the HTML console. But CLAUDE.md section 1.8/D0093 make CLI/JSON the authority and the automation substrate, and every violation found this session was on that side: a catalogue that denies 12 processes which exist, a wiredness proxy reporting an unreachable hook as armed, a ratio published without its denominator or its own floor caveat. The answer to 'why are reports out of sync with truth' is therefore not 'add a drift control' but 'an invariant you already signed is scoped to the wrong surface'.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcHonestSurfaceEverywhere to issue239;

    part issue240 : Issue {
        :>> id = "7f2e6774-4daf-4656-b80b-eca4076211dd";
        :>> title = "The commit gate was disarmed while three shipped surfaces reported it armed - the predicate is a proxy, not the control";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "Found by the panel's mechanisms lens and confirmed directly: core.hooksPath was set to `nul` in .git/config, so git resolved hooks to a nonexistent directory and .githooks/pre-commit, pre-push and pre-merge-commit could not run at all - for an unknown period covering this entire session, during which I repeatedly told the human 'the hook passed'. The substance was in fact checked, because I ran keel validate/guard/gate by hand and CI re-derives on push, but the mechanism I NAMED was dead. Three shipped surfaces asserted otherwise: keel hardening reports the pre-commit point present=true failMode=fail-closed, keel orient printed no hook warning, and keel controls reported uncoveredHazards=[] against ctlCommitGate. Root cause: wiredness is computed as 'git config core.hooksPath returned a non-empty value' - a proxy for reachability rather than reachability. issue116's bypass vector is blocked in the action space and wide open in the detector. hooksPath restored to .githooks; the detector defect is what this issue tracks.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcArmedNotDeclared to issue240;

    part issue241 : Issue {
        :>> id = "d5671140-539e-4605-ace0-aa92b967c79f";
        :>> title = "12 of 23 processes cannot be listed, shown or exported - a must-requirement violation that three critiques passed as green";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "The human asked whether processes are transferable. Measured: 23 process definition files, 11 in the exportable palette; `keel process show intake` and `keel process export intake --out DIR` both exit 2 with 'error: no process intake' while .engine/processes/intake.sysml exists, and `keel process search intake` returns 0 matches. Cause: process_cmd.rs rows() iterates activation::unit_names(), and a unit is formed only when a process declares a guard (activation.rs `if !guards.is_empty()`), so the 12 guard-less processes - intake, definition-of-done, definition-of-ready, adversarial-panel-review, architectural-critique, review-critique, migration, report, indicator, diagram, introduction, obligation-review - are structurally invisible to the portability layer. Those 12 are guard-less by ACCEPTED reasoning (process-enforcement.toml declares them checkable=false with stated reasons), so the exclusion is permanent, not pending. This violates srPortModularProcessUnit, an accepted MUST requirement that a unit carry its definition, deploying skill and rules/guards so it can be moved between projects whole - and it is the un-swept half of issue149, whose disclosure remedy reached activation/activate/deactivate but not the process catalogue. Aggravating: three critiques dated 2026-08-21 passed srPortModularProcessUnit with zero findings, two of them citing a live run of the very command, and tier-satisfaction reports 103/103 verified - a false green inside the verification substrate.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcPaletteIsDeclared to issue241;

    part issue242 : Issue {
        :>> id = "9e95436c-79e7-4194-990c-4b2cb91716e0";
        :>> title = "A core guard can be demoted to inactive by one line in a guard-less process, and the D0209 adherence gate cannot see it";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "Found by the panel's operations lens, which reproduced it live: adding `assert constraint probe : staleGateProse;` to the guard-less .engine/processes/report.sysml removed stale-gate-prose from the core (never-deactivatable) guard set, made report [INACTIVE] with 1 guard, and turned the guard's run into 'PASS - 0 scanned, 1 warning' with `keel guard stale-gate-prose` reporting 'NOT ACTIVE ... do NOT block'. The mechanism is guard_state: it returns Core only when NO unit claims the guard, so a process claiming a core guard converts it into that process's switchable property - and the CLI's own remediation text ADVISES the edit ('or give it an `assert constraint` so it becomes switchable'). The same class applies to narrowing activation.toml's active list, which disarms every guard the removed process owns. Both are covered by the commit keystone, but D0209 clause 1 exists precisely because a bypassed hook leaves no trace at commit time - and the hook was in fact bypassed (issue240) - while audit-adherence's signature() reads only .engine/rules/ trees. So the control built this session for the issue236 self-modification class has a hole the class walks straight through.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcAdherenceCoversActivation to issue242;

    part issue243 : Issue {
        :>> id = "e930b3da-2faa-437f-a814-24645f807ba4";
        :>> title = "Every new project inherits this repo's adoption declaration and exchange identities from keel init";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "The human asked whether a new project is guided to pick its processes. Worse than un-guided: it inherits someone else's choices. keel init scaffolds the embedded .engine verbatim, so a freshly inited tree's .engine/contracts/activation.toml is BYTE-IDENTICAL to the self-build's (verified by diff) - 11 named active processes plus all 32 viewpoints - and `keel activation` in the new tree reports 'declared manifest: yes' for a declaration that project never made. That falsifies CLAUDE.md's own adoption guarantee, that an absent file or section means everything is active so a project that never adopted a control has not violated it: the file is not absent, it is another project's. The new tree also inherits unit-ids.toml, the self-build's EXCHANGE identity for units it never exported. init already special-cases deliverable-manifest.txt to a starter for exactly this reason, so the shape of the fix exists.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcInitContractReset to issue243;

    part issue244 : Issue {
        :>> id = "fb9cf296-1df6-4486-8bfb-cc98621377ab";
        :>> title = "80 warnings ride under [guard] ALL PASS unread, including a live recorded-release contradiction";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "The panel's reducer named this the drift mechanism with the most evidence, and it is the sharpest answer to the human's question about reports being out of sync with truth. Measured: `keel guard` prints '[guard] ALL PASS' while emitting 80 warnings across 10 guards (actors, ceremony, viewpoint-renderer, manifest-coverage, attestation-substance, decision-requirement-link, sequence-multiplicity, parser-coverage, decision-scaffolding, release-recorded). One of them is itself a shipped, detected, unacted-on report-vs-truth contradiction: 'tag v0.2.0 points at e2c812f but its Release item records commit 425145d - the recorded release and the shipped tag disagree (D0191)'. I read ALL PASS and moved past those warnings repeatedly this session. This is why the panel refused a new drift detector: the engine already detects more than it triages, so another detector makes the felt problem worse. The fix is subtractive - surface the warning population where it is read.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcWarningsAreRead to issue244;

    part issue245 : Issue {
        :>> id = "d5d642d4-9b74-4094-825a-770ecb104925";
        :>> title = "keel's elicitation skill is in no catalogue, transferable by nothing, and authors Needs from unrecorded utterances";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "The human asked whether input is recorded as statements then translated to stories, and said pulling latent needs out of a user is critical for the average user. Two findings, converged on by three lenses. First, the elicitation machinery already EXISTS and is in use: the business-architecture skill runs a design/scoping dialogue Brief->Personas->Needs->UseCases and has produced 11 Briefs, 15 Personas, 69 UseCases and 75 Needs - so a new interview process would be a fourth overlapping surface, not a missing capability. Second, that skill has NO process definition, so it appears in no catalogue (process list, activation, hardening all miss it), is enforced by nothing, and can be exported by nothing - and the process-skill guard is one-directional, requiring every process to have a skill but never every skill to have a process, so no control notices. Compounding: it authors Needs straight out of a conversation whose utterances are never recorded as Statements, which is the mechanical cause of the unsourced population and exactly the D0157 defect (a Need written wider than the demand cannot be checked against what the human actually said). Separately measured: intake has recorded nothing since 2026-08-21 - all 17 Statements are dated 08-20/21 - across four days containing four panels, and the human's own six questions in this session were never intaken.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcElicitationThroughIntake to issue245;

    part issue246 : Issue {
        :>> id = "2ab1381b-ef8b-45ce-b114-96c47ade67ea";
        :>> title = "Counts typed into prose drift from the counts the engine computes, with no control between them";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "Named independently by four panel lenses as a distinct drift mechanism, and the brief under review committed it twice itself. Live instances: CLAUDE.md and .engine/docs/guards.md both state '45 forward guards' while keel guard runs 48; the shipped activation.toml header still directs readers to .engine/contracts/process-units.toml, a file D0141 deleted with no tombstone, and keel init copies that ghost into every new project; my own brief said 30 skills where there are 37 and cited GitHub issues #1-#6 where the channel threads are #2-#7. This is distinct from two views disagreeing and from an unverifiable prose assertion: it is a number a reader will trust sitting next to a number the engine can compute, with nothing comparing them. hardening.helpCoverage already does exactly this reconciliation for commands (88/88), so the counter-measure is proven and cheap.";
        :>> discoveredInField = false;
        :>> severity = Severity::Low;
    }
    #Resolves dependency from dcDocCountsReconciled to issue246;


    part issue247 : Issue {
        :>> id = "3d89e4ae-6ec5-4484-8f58-b485c0dabef9";
        :>> title = "The frontier WIDENED on a model-read error, and the code did the opposite of what its own comment promised";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "Found while building dcHonestSurfaceEverywhere (issue239). orient's ready computation filtered out SUPERSEDED tasks (issue100) and tasks BLOCKED on a proposed Decision (issue112), and both call sites read `.unwrap_or_default()`. On an Err that yields an EMPTY filter set, so NOTHING is filtered and every retired task and every human-blocked task RETURNS TO THE FRONTIER. The comment immediately above the superseded line said, verbatim, 'Conservative on error: failing to read the model must not silently make everything ready again' - so the code did precisely what its own comment forbade, and the second site carried the same promise. This is the worst instance of the class because the frontier is AUTO-FOLLOWED (D0052): a transient parse error could schedule work a Decision has explicitly retired. It is also the honest-surface invariant (N-C2) violated on the authority substrate rather than the console - a computation that could not run rendering as a confident, and wider, answer. Compounding it, `keel whats-next` printed nothing and exited 0 whether the frontier was genuinely empty or a filter had failed, so COMPUTED-EMPTY and COULD-NOT-COMPUTE were byte-identical on the one answer the AI follows. FIX: the narrowing filters are extracted into one place that RECORDS failures; a failure empties `ready` (fail closed - an empty frontier plus a stated reason is honest, a superset presented as the answer is not); orient emits an explicit answerStatus of COMPUTED or COULD-NOT-COMPUTE with reasons; whats-next REFUSES with exit 1 on a failure and says COMPUTED-EMPTY out loud when the answer is genuinely empty.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcHonestSurfaceEverywhere to issue247;


    part issue248 : Issue {
        :>> id = "55eebc17-dbe3-440a-98fe-a0d1934d5c3a";
        :>> title = "A unit test asserted a MACHINE property and turned main red for six commits";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "The regression test added in sprint447 for issue240, this_repos_commit_gate_is_armed, asserted that this repository has an ARMED local git hook. That is a property of a MACHINE, not of the repository: a CI checkout configures no core.hooksPath and correctly does not need one, because CI is the hook-INDEPENDENT layer (K15/D0179) that exists precisely so nothing has to assume a hook ran. Every ci run from the issue243 commit onward failed on it - six commits red - while the paired decision-issue workflow kept succeeding, which is why the failures were easy to skim past in a run list showing both. WHAT MAKES THIS WORTH RECORDING RATHER THAN JUST FIXING: my own sprint447 review gate had already named the risk in almost these words - it couples a unit test to machine-local git config, and will fail for a contributor who has not run the setup step - and I wrote that I judged it acceptable and deliberate. The review pass found the defect and the author overrode the reviewer, which is a failure of the review process rather than of the analysis. FIX: assert the CONDITIONAL invariant - if core.hooksPath is configured at all, it must resolve to a directory holding pre-commit - which still catches the exact issue240 state (configured to a null device, unreachable) while an unconfigured checkout is not a violation.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcTestsAssertRepoNotMachine to issue248;


    part issue249 : Issue {
        :>> id = "bb2266b1-9fc8-4096-bcb6-095872fdc97c";
        :>> title = "A git config set inside a probe WORKTREE disarms the parent repo - this is how issue240 happened";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "Reproduced ACCIDENTALLY while probing the controls-arming work, which settles issue240's origin. A linked worktree SHARES the common .git/config, so `git -C <worktree> config core.hooksPath nul` does not scope to the worktree - it rewrites the parent repository's config and disarms the real commit gate. That is almost certainly the exact mechanism by which issue240 arose: an earlier red-team probe set hooksPath to a null device to simulate a bypass, and the setting outlived the worktree. The correct probe form is the PER-COMMAND override `git -c core.hooksPath=/dev/null <cmd>`, which the audit-adherence probes used and this one did not. WHAT MATTERS MORE THAN THE MECHANISM: the detection control already existed and worked. The conditional test added for issue248 failed within a minute of the probe, naming the state, so the disarmed gate was caught by a check rather than by luck - the second time that control has fired on a real disarming, both times self-inflicted. This is the D0047 shape working as intended: rather than trying to prevent a probe-methodology mistake by vigilance, DETECT the resulting state regardless of cause. Recorded so the mechanism is written down once instead of being rediscovered a third time.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcArmedNotDeclared to issue249;


    part issue250 : Issue {
        :>> id = "34fb807f-efe4-4d99-893f-781dd5c10231";
        :>> title = "audit-adherence never checked the FIRST commit of a push, so a single-commit push was never checked at all";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "Found while building the chartered dcNoFalseCleanRange, and the charter was WRONG about the fix. The visible symptom was that audit-adherence printed 'fewer than 2 commits in range - nothing to compare. Clean.' and exited 0, a false clean inside the gate built to prevent false cleans; the chartered remedy was to make that a usage error. That would have been a serious mistake: CI invokes --since github.event.before, so a SINGLE-COMMIT PUSH lists exactly one commit, and erroring would have failed CI on the most common push there is - the issue089/issue090 trap of a new hard check breaking every normal case on day one. THE REAL DEFECT, one layer down: the comparison seeded its baseline from shas[0] and then compared windows of two, so shas[0] was only ever a BASELINE and was NEVER ITSELF CHECKED. The first commit of every push therefore escaped the gate, and a single-commit push escaped entirely. A weakening introduced in the first commit of a push was invisible to the one control built to catch exactly that class (D0209 clause 1), which had been true since the day it shipped. FIX: seed the baseline from the PARENT of the range's first commit, so every commit in the range including the first is compared; a root commit (no parent) is stated as its own baseline rather than guessed at; and only a genuinely EMPTY range is reported as nothing to do. VERIFIED: a weakening committed as a single commit with the hook bypassed now returns exit 1 naming the rule, where the same probe previously reported Clean; real history passes at both --since HEAD~1 and --since HEAD~20.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcNoFalseCleanRange to issue250;


    part issue251 : Issue {
        :>> id = "401a8361-8e26-444c-84da-3b3933da5553";
        :>> title = "The decision channel could not be adopted: it left its mechanism behind and welded one project identity into it";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "Raised by the human asking to activate the GitHub decision channel on asirobots/penumbra, with the constraint: do not force me to be owner of every new project, be modular and careful. FOUR defects, three of which would have broken or misfired on the FIRST adoption rather than degrading. (1) TRANSFER: keel process export decision-channel produced three files - definition, skill, unit.toml - while the five that ARE the channel (two workflows, two scripts, the decider table) stayed home; the guard travelled BY NAME and the machinery not at all. That violates srPortModularProcessUnit, an accepted MUST that a unit move without leaving its enforcement behind, and is the same class as issue241 and issue245. (2) OWNER-AS-PERSON: the issue opener assigned to github.repository_owner, which is not a person - for asirobots/penumbra it is an ORG and gh issue create --assignee <org> fails outright. (3) IDENTITY IN SOURCE: the decider login was a literal in decision-record.yml in three places, so every adopting project had to edit the workflow, forking the mechanism and touching its own frozen enforcement surface. (4) WORST, found only by a round-trip test: keel init shipped this project github-actors.toml verbatim, so every new project silently inherited MY login as its authorized decider and would have recorded acceptances in my name - the issue243 class landing on the single most consequential instance fact in the tree. TWO FURTHER BUGS surfaced by the round trip, both of which had been REPORTING SUCCESS: export derived destinations by stripping .engine, and for a file outside .engine the strip failed so the absolute-path fallback made join discard the base, making it copy the file onto itself; and the --update loop had its own .engine-join, writing .github/workflows/x.yml into .engine/.github/workflows/x.yml - inert, in the wrong place - while printing 6 file(s) from upstream.";
        :>> discoveredInField = true;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcChannelUnitTravels to issue251;


    part issue252 : Issue {
        :>> id = "509efa73-6b5f-4084-8340-fdbabc80da83";
        :>> title = "An adopted unit landed with a RED gate: the entry binding skill to process could not travel";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "Found by actually adopting the decision-channel unit on asirobots/penumbra instead of testing the export in this repository. The bundle carried the process definition, the SKILL.md and the mechanism, but the entry that binds skill to process lives in .engine/skills/skills-registry.sysml - one shared file holding all 37 skills - and stayed home. process-skill therefore FAILED in penumbra on the very first run: process .engine/processes/decision-channel.sysml has NO deploying skill (inert process). A new project's first experience of an adopted unit was a red gate clearable only by hand-editing a shared engine file, which is a fork of the mechanism and exactly the adoption friction D0054 calls the top risk. The rules layer had already met this and solved it by carrying rules BY NAME rather than copying a shared file; skills had no equivalent. FIX (D0220): a skill may declare its deployment beside itself in any .sysml under .engine/skills/, and both readers scan the directory. TWO further defects surfaced in the doing: duplicate-identity refused the copy on the first edit, forcing a MOVE rather than a duplicate; and moving the entry silently dropped the export from 6 files to 5, losing the skill entirely, because activation::deploying_skills also read only the central file - caught by re-checking the export rather than trusting the count.";
        :>> discoveredInField = true;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcSkillRegistersBesideItself to issue252;


    part issue253 : Issue {
        :>> id = "8735fd75-aae9-4d2d-9f0e-211ef3b07a70";
        :>> title = "23 of 24 units would have landed RED in any project adopting them - and nothing computed that";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "Raised by the human asking for a final transferability pass BEFORE migrating to penumbra: fix keel first, I want a good transition. Twice in the preceding hours a unit had been DECLARED transferable and was not - the decision-channel left its whole mechanism behind (issue251), then left behind the declaration binding skill to process (issue252) - and both were found by ACTUALLY ADOPTING, never by reading the export. Computing the answer rather than reading it showed 23 of 24 units would land RED: each carried its process definition and its SKILL.md while the entry BINDING them stayed in the single shared skills-registry, which cannot travel with one unit, so any adoption into a project lacking that process failed process-skill on arrival. Two further hazards surfaced only by adopting into a simulated older project: an incoming per-skill declaration DUPLICATES a central one still present there, failing duplicate-identity - which is penumbra exactly - and a carried file citing .engine/tools/ leaves a dead reference because init never ships engine-dev-only tooling, including one process still naming the Smartsheet tap-inbox that D0206 RETIRED. FIX (D0222): keel process audit makes transferability a computation any project can reproduce; a committed transform moves all 36 declarations beside their skills; import retires a superseded central declaration and says so; the stale references are swept.";
        :>> discoveredInField = true;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcTransferabilityIsComputed to issue253;


    part issue254 : Issue {
        :>> id = "fa98990d-ef7e-458d-91cb-b5cfb105e36d";
        :>> title = "The guard that stops a FABRICATED human acceptance could be switched off with one command";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "Found while checking a premise rather than accepting it. The human said they were fine with the process for defining and cementing processes being undisable-able and shipped, and asked if that was what we were discussing - it was not: process-change has been a CORE never-deactivatable guard all along and change-request is a workflow outside activation, so both halves were already true, while what I had reported was a packaging defect. Checking turned up the real gap. decision-authoring CLAIMED five guards, and activation::guard_state returns Core exactly when NO unit claims a guard - so claiming made them switchable, and `keel deactivate decision-authoring` switched off decisionRationale, acceptanceEvents, decisionRequirementLink, attestationSubstance and confirmationAuthenticity together. confirmationAuthenticity is the guard that stops an AI recording a human acceptance THAT NEVER HAPPENED. A single command therefore disarmed the check standing between the engine and a fabricated human judgment, which CLAUDE.md section 4 says must never be possible - the monitor-modifiable-by-monitored class (issue236/D0208) sitting on the most attestation-sensitive path there is. FIX (D0223): the process asserts no guard, so all five are CORE; the definition says why and warns against re-adding an assert; the link is recorded as core = [...] in process-enforcement.toml so a core-enforced process no longer reports guards 0 and read as unenforced.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcDecisionAuthoringIsCore to issue254;


    part issue255 : Issue {
        :>> id = "378acb24-2421-4891-981c-f9da177e5738";
        :>> title = "A shell backtick injected 2000 characters of tool output into a Decision, and only an invalid escape caught it";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "Recording D0223 through a shell script, a BACKTICK inside a double-quoted argument was command substitution. The decision text mentioned a keel command in backticks; sh executed it and substituted roughly 2000 characters of hardening JSON into the decision field of a governance record. THE PART THAT MATTERS: it was caught only INCIDENTALLY, because the injected JSON happened to carry a unicode escape the SysML parser rejects. Tool output without an invalid escape would have parsed clean and landed, and the committed record would then assert something no human or AI ever wrote - the same class as a fabricated attestation, and worse than a typo because nothing would ever have flagged it. This is the shell-escaping class that has recurred seven times in this session, but every previous instance broke a BUILD; this one silently corrupted a RECORD. The durable authoring rule (record-decision text goes through a file, never through shell interpolation) is a lesson, and a lesson is not a control - so the resolver adds a write-time refusal of a decision field carrying obvious tool output.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcNoShellInjectionIntoRecords to issue255;

    part issue256 : Issue {
        :>> id = "cde95a59-0c56-4651-a006-80e89c035209";
        :>> title = "An accepted Decision had already shipped with 13,136 characters of tool output injected into its context";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "Investigating the D0223 injection (issue255), keel rules flagged D0214 - and its context field held 13,136 characters of console transcript, substituted mid-sentence exactly where a backticked command name stood. The sentence read 'the sharpest answer, measured: <transcript> printed the bare words ALL PASS', with the subject of 'printed' missing: the shell had replaced the command name with what the command printed. So the injection this control was written for had ALREADY happened, in an earlier session, and had been recorded, auto-accepted under standing consent, committed and pushed - undetected, because that output carried no invalid escape and no key-colon pairs. The only reason it ever surfaced is that the pasted text contained verdict words, which tripped a warning-severity rule nobody had read. That makes this the second confirmed instance and the first LANDED one, and it moves the failure from hypothetical to observed: a governance record in this repo asserted 13,000 characters that no author wrote. FIXED HERE: the dump is excised and the sentence restored to name the command it originally named; the acceptance receipt is untouched, since what was agreed did not change. The detector now also refuses the console-transcript shape, calibrated against the whole corpus - no authored field carries more than one guard marker, the landed transcript carried dozens.";
        :>> discoveredInField = true;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcNoShellInjectionIntoRecords to issue256;

    part issue257 : Issue {
        :>> id = "b580c481-4f43-4aa3-8215-541282dbd105";
        :>> title = "A scaffolded project gets 25 active processes and is never asked what it is building";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "keel init lays down a working project with every process active, and nothing ever asks the author what they are building or what must not go wrong with it. The author then faces 25 processes with no basis to judge them and has two bad options: keep all of them and meet friction they never chose - D0054 names adoption friction as the dominant failure mode, above bad architecture - or switch off the ones whose names they do not recognise, which is the worst available selection rule, because the least familiar names are doing the least obvious work. The self-build could never feel this, since keel wants every process on; it surfaced only when adopting onto a second repository. The author raised it directly and made it a precondition on that migration, and named the range it has to serve: hardware, software, both, business processes, safety-related, art, video games, construction plans, whatever. The word whatever is part of the requirement - a closed table of domains cannot serve a domain nobody listed.";
        :>> discoveredInField = true;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcProjectOnboarding to issue257;

    part issue258 : Issue {
        :>> id = "71ad8e6e-2840-4c1f-b6f8-84c8134d0011";
        :>> title = "The decision channel opens one GitHub issue per Decision and is barraging its own owner";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "Volunteered by the author while answering a different question: they do not want a second decider on another project enrolled, because of the notification volume they are getting now. Measured rather than assumed: 22 issues exist and 20 were opened in one day, one per proposed Decision, because D0207 gives every non-fork a forever-open override thread of its own; 94 workflow runs landed the same day, 28 CI failures among them, and each failure emails the pusher; the sweeper fires every 30 minutes. This is the D0054 adoption-friction class turned on its author - a control I built imposing a cost its design never counted - and it is the first friction report about one of these controls. It matters beyond this repo: adopting the channel onto a second project would replicate the barrage there, which is precisely the outcome the author said they wanted to avoid for a colleague. The override thread itself is not the problem and must not be weakened: it is the human only route to reverse a Decision that was auto-accepted under standing consent.";
        :>> discoveredInField = true;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcChannelNotificationCost to issue258;

    part issue259 : Issue {
        :>> id = "1940c642-88a5-4c9a-a02b-a432da5025a3";
        :>> title = "A guard landed today broke the commit gate of the first project keel was adopted onto";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "Guard 50 requires every declared Process to carry an APPLIES-WHEN condition. Run against penumbra - a real project on a 20-August engine snapshot that has never adopted project-onboarding - it failed 22 times out of 22 and would have broken that project's commit gate on arrival, for a convention it never chose. This is the failure D0164 exists to prevent, and I wrote the guard the same day while citing D0164 in a different context, which is the part worth recording: knowing the rule did not prevent breaking it, because nothing in the self-build could feel the consequence. It is the third time in four sprints that a defect was invisible to keel and obvious to a second project. The measurement was only available because the adoption was actually attempted rather than reasoned about.";
        :>> discoveredInField = true;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcGuardScopedToAdoption to issue259;

    part issue260 : Issue {
        :>> id = "9d9f973c-6f4f-4d9c-88db-b3472dbdabe9";
        :>> title = "The release gate had been red since guard 48 landed, and only cutting a release could reveal it";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "Tagging v0.3.0, the release workflow failed - not on anything in the release, but on the test asserting that no guard reports a zero population. The guard reporting zero was claim-ancestry, which skips loudly on a shallow clone because a depth-dependent verdict is the machine dependence K15 forbids, and actions/checkout defaults to depth 1. ci.yml carries fetch-depth 0 with a comment naming issue229 - the exact fix, for the exact reason, already written down in this repo. release.yml never received it. Three of four workflows had it. So the release gate has been red since guard 48 landed weeks ago, undetected because v0.2.3 was tagged on 2026-08-14, before that guard existed, and nothing has cut a release since. Two things make this worse than an ordinary miss: the release path is the one gate where no other run would notice, and the fix had ALREADY SUCCEEDED once in this repo and still did not generalise - so this is not vigilance failing, it is vigilance having worked and not transferred, which is the strongest possible case for D0047.";
        :>> discoveredInField = true;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcGatingWorkflowHistory to issue260;

    part issue261 : Issue {
        :>> id = "21d59181-8670-4b5e-945c-b0caa093038b";
        :>> title = "A second project's commit gate was running this repository's uncommitted working build";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "keel on PATH resolved to target/release/keel in a sibling working directory, reporting version 0.3.0+dirty, and penumbra's scaffolded pre-commit used it. Three consequences, all silent: a broken or half-finished build here would break that project's gate; every guard added here applied immediately to their commits without them adopting anything, which is exactly how a guard landed this session failed 22 of their process definitions on contact (issue259); and their gate's verdicts were reproducible only on this machine, which is the machine-dependence K15 forbids. The scaffolded hook already honoured a KEEL_BIN environment variable, but an environment variable is not a pin - it must be set correctly in every shell by every contributor forever, which is the manual-vigilance shape D0047 rules out. Raised while executing the author's chosen option, pin penumbra to a release, which is what made the coupling visible.";
        :>> discoveredInField = true;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcPinnedFirstBinary to issue261;

    part issue262 : Issue {
        :>> id = "86900de4-62a8-483c-af2a-e61adc064ed0";
        :>> title = "The first live run of the standing thread created it twice and then closed it";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "Two defects, both invisible to the stubbed dry run and both found in the first live run of D0227 OPTION A. FIRST: the thread was created TWICE, issues 28 and 29, seconds apart by the same run - two lookups each missed the other's creation because the lookup used gh --search, which reads the GitHub SEARCH INDEX and lags creation by seconds. The dry run could not have caught it: the stub returned an existing thread instantly, so the create-then-look-again path never executed. FIXED by finding the thread by LABEL, which the API serves directly, plus an in-process cache, plus taking the lowest number if a race ever leaves two. SECOND, and worse: the recorder CLOSED the standing thread after posting the auto-accept receipt, because that step had always closed the per-decision issue it was working on. Under D0227 that one thread is the override surface for EVERY auto-accepted decision, so closing it destroyed the reversal route for all of them at once - the exact property the fork was approved to protect. FIXED by never closing a thread carrying the keel-standing label, while a per-decision fork issue still closes because it is about that decision alone. Both threads reconciled live: 29 reopened and labelled as canonical, 28 closed as the duplicate.";
        :>> discoveredInField = true;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcChannelNotificationCost to issue262;

    part issue263 : Issue {
        :>> id = "b37f60c1-9e28-4ce4-82b7-368ba3a917a5";
        :>> title = "process audit said the unit travels whole; the adoption proved it does not";
        :>> createdAt = "2026-08-24";
        :>> createdBy = "claudeFable5";
        :>> description = "keel process audit - built two sprints ago precisely to answer whether a unit would land green in a project that lacks it - reported decision-channel TRAVELS WHOLE at 7 files. Imported into penumbra it left check-engine with an unresolved type reference: the process asserts constraint judgmentRequestQuality, which is defined in the SOURCE project's schema and is not part of the unit, so the claim arrived pointing at nothing. The audit checks that a deploying skill exists, that the skill-to-process binding travels, that declared extras are present on disk, and that no carried file cites a path the adopter will not have. It does not check that the TYPE and CONSTRAINT names the carried files reference are ones the unit brings with it. This is the THIRD distinct way a unit has failed to travel whole - the mechanism stayed home (issue251), the binding declaration stayed home (issue252), and now a schema symbol stays home - and each was found by attempting a real adoption rather than by any check. The audit's verdict was not merely incomplete, it was CONFIDENT: it printed every unit travels whole, which is worse than printing nothing, because it is the surface someone would trust instead of adopting. The penumbra adoption worked around it by dropping the claim, which is a better outcome anyway - an unclaimed guard is CORE, so the bar applies unconditionally with no activation step.";
        :>> discoveredInField = true;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcAuditSeesSchemaReferences to issue263;

    part issue264 : Issue {
        :>> id = "51f523c3-5799-41eb-b752-e9614c4db1c4";
        :>> title = "Verification is strong and points inward; every recent defect arrived at a boundary nothing measures";
        :>> createdAt = "2026-08-25";
        :>> createdBy = "claudeFable5";
        :>> description = "The human asked which of three things is failing: verification not strong enough, the wrong measures, or the reporting from verification being inaccurate. MEASURED. NOT the first: 51 guards with 42 hard-blocking, 99 percent of live requirements EXAMINED and 91 percent EXERCISED, and no recent defect traces to a lax threshold - when a guard is aimed at something it catches it, including refusing its own author. THE SECOND, in one precise place: 39 of 51 guards read only the authored model, 11 cross a boundary, and ZERO read another project's tree, while 11 of the last 20 issues were discoveredInField and cluster at exactly the boundaries nothing measures - another project's tree, an external service, a shell layer, a different binary, the human's attention. THE THIRD, and dominant: 21 Decisions record a verdict overstating what it verified, 9 of them in the last 18, and the instances are the loudest surfaces in the engine - guard ALL PASS printed over 80 unread warnings, process audit printing every unit travels whole when a schema symbol did not travel, validate green never meaning valid SysML, a stubbed dry run reported as verification. A second form of the same failure: a correct signal nobody consumes - decisionNoVerdictProseRule flagged the 13136-character injection in D0214 for weeks at warning level, and D0214 is itself the Decision recorded to fix unread warnings. THE NUMBER THAT SETTLES IT: 5120 recorded TestResults at a 3.9 percent fail rate, 907 of them stamped by one actor in five days at roughly 2 percent. The actor doing the work also authors the verdict AND the sentence describing it, and no step anywhere converts evidence into a claim under any independent check. So verification is not the weak link - ATTESTATION IS UNAUDITED. This diagnosis carries the same structural weakness it describes: I computed it with measures I built.";
        :>> discoveredInField = true;
        :>> severity = Severity::Critical;
    }
    #Resolves dependency from dcForeignTreeIsTheTest to issue264;

    part issue265 : Issue {
        :>> id = "ad126e7f-5b26-43df-9af0-6ac836f6247f";
        :>> title = "The foreign-tree fixture is keel adopting keel, so it cannot see the two defects that actually happened";
        :>> createdAt = "2026-08-25";
        :>> createdBy = "claudeFable5";
        :>> description = "adoption-check gates a fresh scaffold, which is the right shape but the wrong foreign-ness: the fixture is a CURRENT keel scaffold, so it represents keel adopting keel rather than a real adopter. Two defects from this session are invisible to it, and both were verified invisible rather than reasoned about. issue263 - a unit asserting a constraint it does not carry - passes, because keel init ships .engine/rules/guard-constraints.sysml, so the symbol resolves in the fixture; penumbra could not resolve it only because penumbra runs an older engine snapshot. issue259 - a control firing on a project that never adopted it - passes for the same reason: every process in the fixture already carries the current APPLIES-WHEN convention, so a convention-based guard has nothing to fire on. This matters more than the two specific misses: it means the new gate's PASS is bounded in a way that is invisible from its output, which is the same verdict-overstates-scope class the gate was built in response to. I first claimed the check would have caught five of the recent defects; testing the claim showed two, and the difference is exactly this. The fix is a second fixture pinned to a prior RELEASE, obtained from the release channel so the vintage is real rather than synthesised.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcVintageFixture to issue265;

    part issue266 : Issue {
        :>> id = "4802fda8-76c4-461b-a112-46adcbaf3ae4";
        :>> title = "Every pass in this model is a testimony: of 5135 results, one recorded what produced it";
        :>> createdAt = "2026-08-25";
        :>> createdBy = "claudeFable5";
        :>> description = "Raised by the human, who named it plainly: I very often over-claim - one test for one scenario, reported as though the thing were now perfect - and asked how to improve attestation systemically rather than case by case. Measured: a TestResult carries outcome, judgedBy, judgedAt and judgedAgainst, and nothing else. Of 5135 recorded results, exactly ONE recorded what produced it. So no third party can re-derive any verdict in this model, and the actor doing the work authors the evidence, the verdict, and the sentence describing it. The 3.9 percent fail rate over 5120 results is not a quality signal, it is the signature of a system recording that work happened rather than testing whether it worked. Compounding it: I choose the METHOD myself, and I choose method=test when the evidence was one manual observation, which means the engine's best honesty instrument - the EXAMINED versus EXERCISED split, reporting 99 and 91 percent - is fed by my own self-classification. The human also corrected the aim of any fix: governance is here to constrain and check the AI and other work, never them, so a control demanding a receipt from a human judgment would point at the wrong party.";
        :>> discoveredInField = true;
        :>> severity = Severity::Critical;
    }
    #Resolves dependency from dcEvidenceCited to issue266;

    part issue267 : Issue {
        :>> id = "f6ff57c3-9986-4a3b-84fc-c8d6084c5864";
        :>> title = "Sprint ceremony records were filled by a scratchpad script that bypasses the write API and double-stamps every verdict";
        :>> createdAt = "2026-08-25";
        :>> createdBy = "claudeFable5";
        :>> description = "Found by guard 52 on its first real use: it flagged a TestResult that the write API never wrote. Sprint ceremony records have been filled all session by a python script in the scratchpad which emits TestResult lines directly into the model, so it bypasses every check the write path carries - actor validation, the write lock, the method=confirmation AI refusal, and now the evidence receipt - and its R1 result DOUBLE-STAMPS the R2 that append-result writes for the same test. Seven duplicate results in sprint467 alone; sprints 461 through 466 carry the same shape. Two consequences beyond the duplication: the result population that every attestation number is computed over is inflated by roughly half for recent sprints, and a script with no provenance checks has been the de facto write path for the most judgment-bearing records in the model. The author had already given the rule that covers this - if we need something consistently, build it into keel rather than a bespoke script - and I kept using the script because it was there. The new guard is what surfaced it, which is the argument for the guard.";
        :>> discoveredInField = true;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcSprintFillIsAWritePath to issue267;

    part issue268 : Issue {
        :>> id = "787db3bf-21d3-4846-91c1-4acc515911dd";
        :>> title = "The console never said which project it served, and two consoles were indistinguishable in the browser tab bar";
        :>> createdAt = "2026-08-25";
        :>> createdBy = "claudeFable5";
        :>> description = "Reported by the author while using it: keel serve does not indicate what project it is serving. Three places said nothing - the startup banner printed the port and every endpoint but never the root; the page title was the constant Keel console; the header was the constant Keel dot console. The title is the one that matters most, because a browser TAB shows the page title, so two consoles open on one machine were told apart by nothing at all. That is not cosmetic: the console WRITES, so two identical tabs means a Decision accepted into the wrong project - an attested fact in the wrong tree, with no way to notice. WHAT MAKES THIS WORSE THAN A MISSING LABEL: the capability the author suggested as a better feature was ALREADY BUILT server-side. /api/projects discovers every sibling keel project and names the active one, and its own doc comment says it is the half of N-C4 that stops a supervisor wondering which project they are looking at; POST /api/project switches and refuses a non-project rather than falling back. Nothing in the console rendered either endpoint. So this is the accepted-Decision-with-artifact-unbuilt class: a surface was specified, the server half shipped, and the half the human actually sees never did.";
        :>> discoveredInField = true;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcWhichProject to issue268;

    part issue269 : Issue {
        :>> id = "c8f0d8db-2b83-42d7-8083-14b6f175f156";
        :>> title = "keel validate reports a clean tree for a directory that is not a keel project at all";
        :>> createdAt = "2026-08-26";
        :>> createdBy = "claudeFable5";
        :>> description = "Found while testing whether several keel projects can share one git repo. Pointed at a directory with no .engine, keel validate prints 0 tracking file(s) validated clean and exits 0. That is a vacuous pass on the loudest surface in the engine - validate is the first line of every gate, including the scaffolded pre-commit hook - so a hook placed at a parent directory would gate NOTHING while reporting success, and both projects underneath it would commit unguarded. The two halves of the gate also disagree in that position: guard all correctly FAILS at the same path, so one half treats an absent project as a usage error and the other treats it as a clean tree. This is the same class as the guard-passing-on-an-empty-population defects already recorded (issue250, and claude-surface-drift over zero skills), except it is reachable by a single command with no unusual setup, and the layout that triggers it - a parent repo holding several small keel projects - is one a user would reasonably try.";
        :>> discoveredInField = true;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcValidateRefusesANonProject to issue269;

    part issue270 : Issue {
        :>> id = "c3c58b60-dd6d-4827-8841-ccd41b8df713";
        :>> title = "Several keel projects cannot share one git repository: numbering collides, the channel confuses them, and one hooksPath cannot gate them";
        :>> createdAt = "2026-08-26";
        :>> createdBy = "claudeFable5";
        :>> description = "The author wants one repo for a handful of small short-term projects and expects the layout to recur. Built and RAN it rather than reasoning about it. Three things already work: root resolution walks up to .engine, drift is file-scoped through each project's own deliverable-manifest, and the write lock sits beside each model root. Four break. DECISION NUMBERING COLLIDES: two fresh projects both get d0001. THE CHANNEL CANNOT TELL THEM APART: its marker is keel-decision d0001 and its GitHub lookup is repo-scoped, so the second project's issue never opens because the search finds the first, and a reject gesture is ambiguous. THE GATE CANNOT COVER THEM: git allows exactly one core.hooksPath per repository, so a per-project hook gates at most one project, and keel validate at the parent printed 0 tracking file(s) validated clean and exited 0 - a hook there would gate nothing while reporting success. AND LAND GATES ONE ROOT WHILE PUSHING THE WHOLE REPO, so the other projects' changes ride out covered by no gate. Each of the four is a property of git or of the channel being repo-scoped, not a preference, so the engine has to know the difference between a project and the repository holding it.";
        :>> discoveredInField = true;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcWorkspace to issue270;

    part issue271 : Issue {
        :>> id = "4bc157a7-44a8-49bf-8052-4c5bb95c6619";
        :>> title = "Every project's guards saw every other project's staged files";
        :>> createdAt = "2026-08-26";
        :>> createdBy = "claudeFable5";
        :>> description = "git diff --cached answers for the whole REPOSITORY no matter which subdirectory asks it, and the guards that read staged files did not re-scope the answer. In a repo holding several keel projects that means each project's guards judge the other projects' changes, and the workspace-level files too. Found live and immediately: a two-project workspace failed BOTH projects' process-change guard over .githooks/pre-commit, a file belonging to neither project. The failure is not merely noisy - it is wrong in the direction that matters, because a project's gate reported a violation about a file it does not own, which is a verdict no reader could act on. Invisible in a single-project repo, where the project IS the repo root and the unscoped answer happens to be correct - so nothing before now could have surfaced it.";
        :>> discoveredInField = true;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcStagedFilesAreProjectScoped to issue271;

    part issue272 : Issue {
        :>> id = "a762e9c6-0cfc-4690-b322-ab00baaa5be3";
        :>> title = "A freshly scaffolded project could not make its first commit";
        :>> createdAt = "2026-08-26";
        :>> createdBy = "claudeFable5";
        :>> description = "keel init lays down a working project and tells the author to commit. That commit FAILED. init stages every .engine/processes and .engine/workflows file, the keystone lock read those as locked surfaces changing without a co-committed signed Decision, and the guard blocked - telling a brand-new user to record a process-change Decision for a scaffold they did not write and had no opinion about. Reproduced on a plain single-project keel init, so this has nothing to do with workspaces and has been true for every new project. It is the worst adoption defect found so far, because it is the FIRST thing anyone does after scaffolding and the message sends them somewhere useless. The distinction the lock was missing: it exists so a control cannot be WEAKENED unsigned, and a locked file being ADDED is a control arriving, not one being weakened. Found only because a workspace experiment made me commit a fresh scaffold - adoption-check runs validate, guard and check-engine but never a COMMIT, so nothing staged means process-change sees nothing and the whole class was outside every existing check.";
        :>> discoveredInField = true;
        :>> severity = Severity::Critical;
    }
    #Resolves dependency from dcKeystoneLockIsAboutChangeNotArrival to issue272;

    part issue273 : Issue {
        :>> id = "d50fbb2f-4791-4b46-9ba6-24c963180b0c";
        :>> title = "A git mv bypassed the keystone lock in every repository, opened by my own fix to it the day before";
        :>> createdAt = "2026-08-26";
        :>> createdBy = "claudeFable5";
        :>> description = "Found by an adversarial panel convened on D0234, by PEDANT-3, within the hour. Yesterday I narrowed the keystone lock from --diff-filter=ACMR to MD so that a freshly scaffolded project could make its first commit (issue272). ACMR had included R. MD does not. So a locked file could be edited and MOVED in one staged change and the guard saw nothing: sed blocking to warning into a new filename, remove the original, git add -A, and git records R096 while the MD filter matches nothing. keel guard process-change reports PASS - 0 scanned. Every blocking rule in a project downgraded to a warning with no signed Decision. REPRODUCED ON A PLAIN SINGLE-PROJECT REPO, so this was live in this repository and in penumbra, not confined to the workspace path I had just built - and I had told the human there was no live exposure, which was wrong. The deeper error was reading --name-only, a projection that prints only a rename's destination, so a filter over it hides one half of the very operation being policed. The same function carried a second blindness the same panelist found: git C-quotes a path containing a non-ASCII byte in this output while rev-parse --show-prefix does not, so a project named with a circumflex was invisible to the workspace gate and to its own guards, and a full rules downgrade there passed. The general lesson is narrower than it looks: I weakened a control and never tested the weakening against the class of change it was supposed to keep catching.";
        :>> discoveredInField = true;
        :>> severity = Severity::Critical;
    }
    #Resolves dependency from dcRenameIsAChange to issue273;

    part issue274 : Issue {
        :>> id = "1992d2ac-cd65-4bca-8219-589927dbd1d1";
        :>> title = "Recording an obligation made the tree invalid: the recorder's own file does not validate";
        :>> createdAt = "2026-08-26";
        :>> createdBy = "claudeFable5";
        :>> description = "The Stop hook recorded a red-yield obligation and the file it wrote imports only EngineElement, while Issue is defined in EngineWork and the Resolves edge in EngineRelationships. So validate reported a semantic diagnostic - unresolved type reference Issue - the moment the obligation was staged. That is self-defeating twice over: the record exists to say the tree needs a correction pass, and it is itself the thing making the tree incorrect; and it puts the recorder in the one position a recorder must never occupy, unable to record without breaking what it is reporting on. Found while triaging a genuine obligation, so the defect had shipped and had never been exercised - no obligation had been recorded since the template was written.";
        :>> discoveredInField = true;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcObligationRecorderValidates to issue274;

    part issue275 : Issue {
        :>> id = "06fdbb25-8072-4156-94b1-a004c58c599c";
        :>> title = "Workspace discovery stops at the first project, so nested and deep projects are invisible and ride out ungated";
        :>> createdAt = "2026-08-26";
        :>> createdBy = "claudeFable5";
        :>> description = "Three panelists found this independently. The walk returns the moment a directory is a project and is bounded at depth three, and init refuses neither position, so a project nested inside another or deeper than three is invisible to every workspace-scoped mechanism. Verified: keel init dot then keel init sub in one repo reports a single project; a commit adding an unparseable tracking file plus a downgrade of every blocking rule in the invisible project exits 0 while the gate prints clean, and validate inside that project then fails. Same at depth four. This is the most likely layout for the stated goal - an author whose repo already has a project at the root adding small ones beside it - and it silently collapses all four workspace fixes back to one project. Aggravating: my own unit test asserted exactly this shape as correct, so it pinned the bug as intended behaviour and would have blocked anyone from fixing it.";
        :>> discoveredInField = true;
        :>> severity = Severity::Critical;
    }
    #Resolves dependency from dcWorkspaceDiscoveryIsComplete to issue275;

    part issue276 : Issue {
        :>> id = "eb297eac-6d5a-4cb8-aae4-241e3fdd65c2";
        :>> title = "The one hook that gates every project in a workspace can be replaced with exit 0, unsigned, and the gate passes";
        :>> createdAt = "2026-08-26";
        :>> createdBy = "claudeFable5";
        :>> description = "Two panelists found this independently and I reproduced it. In a workspace the repo-root enforcement surface belongs to no project, and the keystone lock matches project-relative paths, so the root hook and the root workflows directory are covered by no guard at all. Verified: the single pre-commit hook that gates every project replaced with a two-line exit 0, staged alone with no Decision - gate --workspace printed a line NAMING the unowned file and then reported 2 projects gated clean, exit 0. The identical edit in a single-project repo is BLOCKED. The tell is my own output: a gate that reports what it is not checking and passes anyway is the false-green class this session spent its length closing. Also verified: staging the deletion of an entire project, 445 paths including every process definition and its guards, passes the gate silently.";
        :>> discoveredInField = true;
        :>> severity = Severity::Critical;
    }
    #Resolves dependency from dcWorkspaceSurfaceIsOwned to issue276;

    part issue277 : Issue {
        :>> id = "436d289f-80ea-4a7d-aa77-e51cf29f9022";
        :>> title = "The compensating control cited for relaxing the keystone lock is blind in the arrangement the same Decision introduced";
        :>> createdAt = "2026-08-26";
        :>> createdBy = "claudeFable5";
        :>> description = "issue272 relaxed the keystone lock to exempt ADDED locked files, and D0234 justified the residual by naming audit-adherence's guard-state monotonicity as the check that catches an added process weakening a core guard. audit-adherence derives its enforcement signature from repo-root-relative pathspecs, which match nothing when projects live one directory down. Verified by two panelists with an identical unsigned rule weakening: single-project repo FAIL exit 1, naming the blocking-to-warning transition; two-project workspace PASS exit 0, both from the repo root and from inside the project. So the relaxation is unconditional while its stated compensation exists only in the single-project case. I weakened one control and justified it by pointing at another without checking that the second worked in the new case - which is the same reasoning error as citing a stub dry run as verification.";
        :>> discoveredInField = true;
        :>> severity = Severity::Critical;
    }
    #Resolves dependency from dcAdherenceIsWorkspaceAware to issue277;

    part issue278 : Issue {
        :>> id = "c43e2eee-88fc-41f7-b30e-47e9df4904c8";
        :>> title = "A freshly created workspace is completely ungated, and the remediation message installs a gate that checks nothing";
        :>> createdAt = "2026-08-26";
        :>> createdBy = "claudeFable5";
        :>> description = "Three panelists found this. init writes a hook per project and sets core.hooksPath only when its target itself holds a git directory, so git init followed by two keel inits leaves no repo-root hook and no hooksPath - verified, the subsequent commit ran with zero pre-commit lines. init then prints instructions which, followed literally, create a nested repository and destroy the workspace. Worse, the refusal message I added to fix the issue269 vacuous pass advises keel hook pre-commit: that is not a hook event, and at a workspace root it prints nothing and exits 0. Anyone who wires it into a hook installs a gate that passes while checking nothing - issue269 reintroduced inside its own fix, by its own remediation text.";
        :>> discoveredInField = true;
        :>> severity = Severity::Critical;
    }
    #Resolves dependency from dcWorkspaceIsWireable to issue278;

    part issue279 : Issue {
        :>> id = "6890b896-6150-4179-b4dd-73131e5fb4de";
        :>> title = "The decision qualifier does not round-trip, so a human's judgment still lands in the wrong project";
        :>> createdAt = "2026-08-26";
        :>> createdBy = "claudeFable5";
        :>> description = "Four verified gaps, found across three panelists. The DURABLE marker written into an issue body uses the bare name while every lookup uses the qualified id, so in a workspace no lookup can match what the opener wrote: the standing thread re-posts on every run, the backfill posts a duplicate, and two projects write byte-identical markers - the collision D0234 claims to have removed. The gesture parser drops the qualifier, so a reject naming a qualified id resolves to the bare name and retargets the human's rejection to the ROOT project, while the opener instructs them to type exactly that form. The human-gesture branch of the recorder never calls the splitter, so a fork acceptance - the one decision class that genuinely needs a human - is recorded against the wrong tree or refused outright. And the decider table is read from the repo root, absent in a workspace, so every gesture is answered not-a-declared-decider. My verification tested the join function in isolation and never checked what actually gets written into an issue body.";
        :>> discoveredInField = true;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcChannelQualifierRoundTrips to issue279;

    part issue280 : Issue {
        :>> id = "9ded8084-48f6-48b9-8f66-9c03df6ce809";
        :>> title = "keel land gates nothing before the first push, so a broken project leaves the machine";
        :>> createdAt = "2026-08-26";
        :>> createdBy = "claudeFable5";
        :>> description = "D0234's fourth clause claims land gates every project before a push. It does not: land pushes first and only reaches the gate after a REJECTION and merge, so on the success path nothing is gated at all. Verified against a bare remote - a project carrying an unparseable tracking file, committed with the skip flag as any contributor without a configured hook would, then land run from a different project in the same workspace: pushed, exit 0, and the broken file is present in the remote. sync does gate the whole workspace unconditionally; only land is holed. So the Decision asserts a property the code does not have, and the clause I wrote to fix the original complaint about push reconciliation is the one clause that was never implemented on the path that matters.";
        :>> discoveredInField = true;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcLandGatesBeforeThePush to issue280;

    part issue281 : Issue {
        :>> id = "d02d8a2d-a5ce-44ab-b2ea-bf38d6b71ec5";
        :>> title = "orient reports an all-green empty state at a workspace root, and validate can answer for another repository";
        :>> createdAt = "2026-08-26";
        :>> createdBy = "claudeFable5";
        :>> description = "The issue269 refusal was applied to validate alone; the project predicate has exactly one non-test call site. So at a workspace root orient exits 0 with an empty ready list, zero done, zero outstanding, 100 percent decomposed and answerStatus COMPUTED - and orient is the surface CLAUDE.md makes the AI's only legitimate state read, explicitly forbidding prose substitutes. whats-next prints COMPUTED-EMPTY over zero items, calling it an answer rather than a failure. check-engine prints validated clean over zero files and is a blocking step in this repo's own commit gate. A separate finding on the same theme: find_repo_root has no git boundary while workspace discovery does, so standing in a directory nested under an unrelated keel project, validate with no argument validated the OUTER repository's project and reported it clean.";
        :>> discoveredInField = true;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcNoCommandReportsGreenOverNothing to issue281;

    part issue282 : Issue {
        :>> id = "63a56a6e-4ae4-413d-97ab-6f19be9d7d16";
        :>> title = "Three bodies of the gate with three different bars, and the push gate is the weakest";
        :>> createdAt = "2026-08-26";
        :>> createdBy = "claudeFable5";
        :>> description = "The workspace per-project gate runs validate plus guards plus declared rules; the scaffolded pre-commit hook runs validate plus guard plus rules enforce; the sync and land gate runs validate plus guards and no rules at all - that file contains zero occurrences of the word. The declared-rule layer is precisely the one a downstream project uses to add a blocking control without writing Rust, so such a project has its only control enforced at commit and unenforced on the merged tree, which is the class the merged-tree gate exists to catch since a merge can violate an edge rule neither side violated. The sync gate's own doc comment still reads deliberately the SAME entry point the commit hook uses, rather than a parallel definition, and D0234 edited the body underneath that comment while writing a third, fuller definition four hundred lines away.";
        :>> discoveredInField = true;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcOneGateDefinition to issue282;

    part issue283 : Issue {
        :>> id = "7f257082-8fe5-47f3-813e-6eaacf3d145a";
        :>> title = "Six claims in the workspace record are wrong, including evidence references that do not resolve";
        :>> createdAt = "2026-08-26";
        :>> createdBy = "claudeFable5";
        :>> description = "The facts pedant verified most of D0234 and refuted six claims. byte-identical for a single-project repo is an over-claim: the multi-project predicate gates only message wording, and three changes alter single-project behaviour - the keystone filter, validate's refusal, and staged-path re-basing for a project nested in a larger git repo. verified against both real repositories undercounts by six; there are eight keel repos here. no nested false positives was VACUOUS, because the walk short-circuits at a root project and therefore examined no subdirectory in any of them - the check could not have found a false positive if one existed. The two headline RAN evidence references in the sprint record do not resolve, being commits in scratch repos I deleted, which violates D0129 inside the very receipt mechanism built one day earlier to make a pass re-derivable by someone else. The refusal message names the wrong missing directory. And an unexplained installed-units counter bump rode the commit.";
        :>> discoveredInField = true;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcPanelRecordCorrections to issue283;

    part issue284 : Issue {
        :>> id = "1cb2a9ed-f269-4102-bb0e-e6b55023caf1";
        :>> title = "keel projects cannot tell you which project you are in on this platform";
        :>> createdAt = "2026-08-26";
        :>> createdBy = "claudeFable5";
        :>> description = "Two panelists found this independently. The command canonicalizes the working directory, which on Windows returns the extended-length prefix with backslashes, while discovery takes the repository root from git in forward-slash form, so the prefix comparison never matches. Verified: run from inside a project, no you-are-here marker is printed and the JSON reports current false for every project - including in all eight real keel repos on this machine. The unit test passes only because it constructs the structure by hand from POSIX literals and never goes through discovery, which is the same hollow-verification shape as the vacuous nested-false-positive check in the same module.";
        :>> discoveredInField = true;
        :>> severity = Severity::Low;
    }
    #Resolves dependency from dcOwnerOfMatchesOnWindows to issue284;

    part issue285 : Issue {
        :>> id = "45d82ef0-ff7b-454a-a146-e821a4e19d29";
        :>> title = "The console advertised six computed views it could not serve, and the human found the first by clicking it";
        :>> createdAt = "2026-08-26";
        :>> createdBy = "claudeFable5";
        :>> description = "Reported by the human from the console: clicking attestation-coverage returned no computed view is bound to the command, while the same command works on the CLI. Diffing the viewpoint registry's renderer prose against the server's binding table found SIX unbound - attestation-coverage, critique-policy, decisions, reprocess-candidates, verification and whats-next - so the human happened to click the first of six dead panels. This is the accepted-Decision-with-artifact-unbuilt class (issue174) landing on the human's own oversight surface: a viewpoint that names a renderer is a promise the console can serve it, and the model kept declaring surfaces the server did not implement. whats-next is among them, which is the ranked frontier - the single most load-bearing view in the engine. Found from USE, again: nothing internal compared the two sets, and both were readable from source the whole time.";
        :>> discoveredInField = true;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcEveryAdvertisedViewIsBound to issue285;

    part issue286 : Issue {
        :>> id = "b3b12e28-d029-4044-83ab-fda17243dcc5";
        :>> title = "Every generic viewpoint panel dumped raw JSON while the console's own item-linking sat unused";
        :>> createdAt = "2026-08-26";
        :>> createdBy = "claudeFable5";
        :>> description = "Reported by the human: tools give a vanilla json payload, needs to be displayed properly and link to items properly and dynamically. The generic viewpoint panel rendered its payload as pre JSON.stringify with one-space indentation - a blob the reader has to parse by eye - and every item name in it was dead text. The console already had itemLink, an openItem panel and an /api/item/:name endpoint, used by the bespoke panels and by orient; the generic path simply never called them. So this was not a missing capability but an unused one, which is why it survived: each new viewpoint got a reachable panel and nobody asked whether the panel was readable. The consequence compounds with issue285 - as more viewpoints get bound, more of the human's oversight surface becomes JSON blobs unless the generic renderer is the one that improves.";
        :>> discoveredInField = true;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcComputedViewsRenderAndLink to issue286;

    part issue287 : Issue {
        :>> id = "c2133e83-c201-4ca0-a261-e9f352ae266f";
        :>> title = "A guard demanded that the human quote themselves, because nothing records who recorded an acceptance";
        :>> createdAt = "2026-08-26";
        :>> createdBy = "claudeFable5";
        :>> description = "The human accepted d0235 through a human surface while the AI was working, noting agreed with panel's decisions. delegatedAcceptanceSubstanceRule then failed the commit gate, requiring a verbatim quoted span or a cited surface gesture - which for a record the human wrote themselves means demanding that they quote themselves. That is governance aimed at the one party it must never bind, and the human had already stated the principle: the AI is here to constrain and check other work, not them. The rule's intent is correct and load-bearing: an acceptance an AI records ON THE HUMAN'S BEHALF must quote their words, because that quote is the only thing between a real sign-off and a fabricated one. The defect is a missing fact - a TestResult records who JUDGED it but nothing records who RECORDED it, so the rule cannot distinguish a delegated record from a self-recorded one and applies the delegated demand to both. Two further notes on handling: the AI must NOT edit the human's note to satisfy the guard, because altering an attestation to pass a check is the exact failure the check exists to prevent; and it must not widen the rule to unblock itself, because that is weakening a control for convenience. Both were declined; the forward fix stamps the channel at the console write path, and the past record is left for the human.";
        :>> discoveredInField = true;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcRecorderIsDistinctFromJudge to issue287;

    part issue288 : Issue {
        :>> id = "8179e3e8-20ec-4114-9508-f91bbb691eab";
        :>> title = "The console tab counted viewpoints instead of flagging what waits on the human";
        :>> createdAt = "2026-08-26";
        :>> createdBy = "claudeFable5";
        :>> description = "Reported by the human: putting the number of views in the tab heading is bad, each tab should have actionable items in the heading, because when they open the console they are trying to resolve items. The tab rendered the surface's VIEWPOINT count - assurance (11), act (4) - so the most-read navigation in the engine showed how many lenses a surface holds, which is never the question a supervisor arrives with. What makes this the unused-capability class rather than a missing feature: /api/obligations already computed what awaits the human per viewpoint, derived from viewpoints declaring surface=act, complete with a countable flag and its own note that a class with no bound counter must never be reported as zero. The console had the number and rendered a different one. Two honesty properties had to be preserved while changing it: a class with no counter must render as unknown rather than zero, and a FAILED obligations fetch must render unknown on every tab rather than showing no badge - because an absent badge means nothing is waiting, and that would be a false all-clear on the surface whose entire purpose is telling the human what they owe.";
        :>> discoveredInField = true;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcTabsFlagWhatNeedsTheHuman to issue288;

    part issue289 : Issue {
        :>> id = "3615d275-41bd-46e7-861e-6a9556b4a65f";
        :>> title = "Intake had no write API, so every recorded human utterance was hand-edited into a file";
        :>> createdAt = "2026-08-26";
        :>> createdBy = "claudeFable5";
        :>> description = "The human asked whether the claim was accurate: is there really no write api surface for statements or user stories. It was. keel record covered decision and issue only. That is the record type where the gap matters most, because D0216 makes the verbatim Statement the precondition for every Need - a Need with no cited utterance cannot be checked against what the human actually said, which is exactly how one comes to be written wider than the demand. So the record with the strictest rule had the weakest path: hand-edited into a file with no actor validation, no write lock, no refusal on a defaulted date, and nothing stopping a paraphrase typed from memory being stored in a field labelled VERBATIM. Every Statement in this repository was written that way, including five from this session. A second-order defect the fix had to avoid: the existing field sanitiser collapses runs of whitespace, which is correct for a title and destroys a quote, so reusing it for text would have silently altered what the human said while appearing to store it faithfully.";
        :>> discoveredInField = true;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcIntakeHasAWritePath to issue289;

    part issue290 : Issue {
        :>> id = "9241ce15-f106-4223-947b-3eb88fbe715e";
        :>> title = "The adopted channel's mechanism never landed downstream, and nothing checks that a process's mechanism exists";
        :>> createdAt = "2026-08-26";
        :>> createdBy = "claudeFable5";
        :>> description = "Adopting the decision channel onto penumbra, the merged PR carried the process definition, the skill, its registry entry and two contracts - and NOT the mechanism. Both workflows and both scripts were missing, so that project held a process definition and a skill referencing scripts that do not exist. The cause was mine and it was hand-staging: to avoid committing that repo's untracked logs and screenshots I staged explicit paths rather than add-all, omitted .github from the list, and then deleted .github from the working tree so a later add-all could not land it out of band - which destroyed the only copy. This is issue251 verbatim, the channel leaving its mechanism behind, reproduced by hand downstream AFTER being fixed in the export. Two checks that should have caught it did not. adoption-check gates a fresh scaffolded fixture, so it verifies what EXPORT produces and never what a TARGET actually received - the gap named in its own stated limits, now with a concrete instance. And no guard flags a process whose declared mechanism files are absent, even though the tool-reference guard already performs exactly that check for .engine/tools paths. Fixed downstream by a follow-up import, which also correctly preserved that project's two documented local divergences.";
        :>> discoveredInField = true;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcMechanismMustBePresent to issue290;

    part issue291 : Issue {
        :>> id = "2daf9a76-7dd0-4154-b540-bb90e4d3eb1f";
        :>> title = "record decision in a fresh downstream project allocates D0001 and collides with the shipped reference namespace";
        :>> createdAt = "2026-08-27";
        :>> createdBy = "claudeFable5";
        :>> description = "Observed 2026-08-26 in a fresh downstream project (pdp-gate-review-3q26, keel init vintage of ~2026-08-25): .engine/decisions/ was empty, so 'keel record decision' allocated 0001 -> package Decision0001 / part d0001, which collides with the shipped read-only reference history .engine/reference/decisions/0001-text-files-are-truth.sysml (same package name, same part name). The registry silently merges same-named packages; only guard:duplicate-identity caught it, at commit time (2 violations). Every fresh downstream project's first recorded decision hits this. Downstream workaround applied there: project decisions renumbered into a 1xxx series - which re-collides if the reference history ever reaches 1001. Root fix: allocate past the highest number across BOTH .engine/decisions/ and .engine/reference/decisions/, or give project decisions a distinct package/name scheme; either way, prove it with a test that records the first decision in a scaffolded project and runs guard:duplicate-identity.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from fixDownstreamDecisionNumbering to issue291;

    part issue498 : Issue {
        :>> id = "b42029d8-a160-4a89-b58b-e7a8cde69e55";
        :>> title = "Fifteen landed records name claudeOpus5 while the session's model was Fable 5 - the session actor binding does not follow the model";
        :>> createdAt = "2026-09-11";
        :>> createdBy = "claudeFable5";
        :>> description = "Two landed commits carry records attributed to a model that was not running. VERIFIED before the claim: git show ceaabdd and 76b106e -- .tracking: 7 and 8 records with createdBy/judgedBy = claudeOpus5, each commit's trailer reading Co-Authored-By: Claude Fable 5; e1de7c7 (trailer Claude Opus 4.8) is the last commit where the two agree. MECHANISM: .keel/sessions/<id>/actor was written claudeOpus5 on 2026-09-08 16:21 by a KEEL_ACTOR declaration; the harness switched the same session's model afterwards and actor.rs resolve reads the session binding without asking what model is answering. Found by stpa-self run 4 while binding the run's own records: keel actor show printed claudeOpus5 under a Fable 5 system prompt. Corrected for this session by rebinding (machine and session files, 2026-09-11) before any run-4 record was written; the fifteen landed records stay as written (D0129: history is not rewritten). This is EHZ3 in its quiet form - no human was impersonated, but the attribution the ownership, critic-independence and retro guards read is false for two commits.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcSessionActorFollowsTheModel to issue498;

    part issue499 : Issue {
        :>> id = "04afbe20-660b-48c6-a336-1e008658d415";
        :>> title = "A chat Statement typed by the AI reads with the same authority as one the speaker recorded";
        :>> createdAt = "2026-09-11";
        :>> createdBy = "claudeFable5";
        :>> description = "keel record statement records the human's words from the AI's typing and nothing marks the difference. READ: main.rs cmd_record_statement takes --text (or --from FILE), --said-by and --title from the caller, defaults --channel to chat, and binds createdBy to the resolved actor; the Statement schema (intake.sysml) holds text VERBATIM by comment, with no field or lens saying whether the speaker or a transcriber wrote it down. So every chat Statement in the tree reads with the same authority whether the human pasted it or the AI recalled it, and a Decision that dutifully links its Statement (issue428's fix) inherits a verbatim-ness nobody checked. Found by stpa-self run 4 as UCA-RS1 on cmdRecordStatement. Not the same gap as issue428: that one binds Decisions to Statements; this one is the Statement's own provenance.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcChatStatementsAwaitTheirSpeaker to issue499;

    part issue500 : Issue {
        :>> id = "c79f7781-e50b-48ae-b0e0-315e57b0fb7d";
        :>> title = "keel record reverify stamps judgedAt with HEAD's commit date, not the run's date";
        :>> createdAt = "2026-09-11";
        :>> createdBy = "claudeFable5";
        :>> description = "keel record reverify dates its judgment by the commit, not by the run. READ: reverify.rs run() (lines 362-363) takes head from rev-parse --short HEAD and date from git show -s --format=%cs HEAD, then append_result(file, task, head, pass, date, ...) - so judgedAt is the day HEAD was committed. A drift re-verification run on a later day records a pass judged before it ran; the impossible-evidence-date guard reads commit-vs-evidence order and this order is not impossible, only false. Found by stpa-self run 4 analysing cmdRecordReverify (wrongTimingOrder). Every other record sub-verb takes judgedAt from --judged-at or the clock. No landed result is known wrong; the tree's reverify passes all ran on the day of their HEAD.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcReverifyStampsTheRunDate to issue500;

    part issue501 : Issue {
        :>> id = "698d0e9c-9879-480a-962f-28b2ca9ce998";
        :>> title = "Sub-verbs inherit their router's effect: four process reads and record mint are write actions in the control structure";
        :>> createdAt = "2026-09-11";
        :>> createdBy = "claudeFable5";
        :>> description = "A routing command's sub-verbs inherit the router's single effect in the computed control structure. READ BACK after D0454 landed in the working tree: keel show control-structure names cmdProcessList, cmdProcessSearch, cmdProcessShow and cmdProcessExport as write actions because the process fact's effect is writes (its publish, retire, remove and import sub-verbs do write), and cmdRecordMint as a write action though record mint writes nothing (write.rs WRITE_OPS lists it false; D0451). The CliCommand fact (cli.sysml) declares one effect per command and no effect per sub-verb; D0454 derives one action per sub-verb, so the derivation is right and the declaration is too coarse. Consequence: stpa-currency asks for a UCA analysis of four read paths, and a hardcoded exclusion in control_structure.rs would be the literal vocabulary D0271 forbids. Found by stpa-self run 4 (sprint 681 retro). The fix touches engine schema, so it is a Change Request.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcSubVerbEffectIsDeclared to issue501;

    part issue502 : Issue {
        :>> id = "5bdd96fc-f895-4896-8c5c-604356cbe800";
        :>> title = "Run 4 of stpa-self landed nested inside a TestResult: a prefix anchor split a one-line part and five UCAs vanished from every item-based view";
        :>> createdAt = "2026-09-11";
        :>> createdBy = "claudeFable5";
        :>> description = "Landed in fc89e6a: stpa-self run 4 (five UnsafeControlAction parts, five ControllerConstraints, stpaRun4 and its result) sat INSIDE the body of `part stpaRun3R1 : TestResult { ... }` in .tracking/architecture/engine-ucas.sysml. VERIFIED before the claim: keel show control-census . answered ucaSummary.ucas = 16 while grep counted 21 `: UnsafeControlAction` parts; the file read at lines 367 and 480 showed the one-line part's tail (`:>> outcome ... }`) 113 lines below its head. MECHANISM: the run was inserted with textpatch insert-after whose anchor was the first 60 characters of that one-line part - a prefix, not the line - so the insert split the line and everything after it nested inside the part. Legal SysML: keel validate passed (779 clean), and the text-based stpa-currency reader saw the ANALYSED names, so the sprint's own gates were green while the item-based census, the D0424 evidence classes and the ucas view could not see five UCAs. SECOND occurrence in the same sprint: FINDING 5 of the sprint 681 retro was the same anchor shape on control-map.sysml, caught by a count and repaired by hand, and the retro said no new item because the operator erred - manual vigilance, which D0047 says is not a control; this recurrence is what that judgment cost. Repaired in this commit by rejoining the line (census reads 21). Hazard EHZ2: a landed state the views under-report.";
        :>> discoveredInField = true;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcAnEditCannotNestAnItemByAccident to issue502;

    part issue503 : Issue {
        :>> id = "4de7a03e-d109-40af-b596-cb509eb86c46";
        :>> title = "The orientation fold renamed three commands and the brief's fact reader kept the old spellings: nothing holds a script's keel invocations to the declared surface";
        :>> createdAt = "2026-09-11";
        :>> createdBy = "claudeFable5";
        :>> description = "The D0450 orientation fold (sprint 679, ceaabdd) moved orient, status and enforcement-report under `keel show`; scripts/exec_brief/facts.py kept calling the old spellings at four sites (orient at lines 518 and 525, status at 628, enforcement-report at 999), each answered by the top-level usage text, so pendingAcceptances, guardViolations, guardWarnings and the hook-fire facts came back null with a `how` that quoted the failed command. VERIFIED before the claim: the build script refused on `fact pendingAcceptances is null - ... command failed: keel - text is truth, state is computed`, and each spelling run by hand printed the usage text while its `show` form answered. The `how` prose at line 519 already said `keel show orient` while the invocation beside it said `orient` - the fold's doc-sync reached the words and not the call. MECHANISM: no control holds a script's invocation of a keel command against the declared surface (.engine/cli/commands.sysml); cli-surface-declared holds facts = help = dispatch inside the binary, tool-reference holds .engine/tools paths on the doc surface, and the brief's build refuses a null fact - which is how this surfaced, two commits after the fold. Same class as dcFieldReadersOutsideRustAreEnumerated, for commands instead of fields. Fixed in this commit at the four sites; the sweep found no other caller under scripts/, .engine/tools, .claude/hooks or .githooks.";
        :>> discoveredInField = true;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcScriptCallersOfTheCliAreHeldToTheSurface to issue503;

    part issue504 : Issue {
        :>> id = "be39894f-f651-4ab4-bd76-c503807d1949";
        :>> title = "audit history judges a pre-fold commit FAIL on a guard that compares the binary to a tree it was not built from";
        :>> createdAt = "2026-09-12";
        :>> createdBy = "claudeFable5";
        :>> description = "Sprint 682's three-tree comparison (D0273 shape) ran `keel audit history --max 3` on the green tree with the pre-fold binary (keel-prefold682.exe, fc89e6a) and the post-fold binary (6821b5c+dirty): the old binary reported PASS, UNKNOWN, PASS over 6821b5c, 36715a2 and 9e4e791; the new binary reported FAIL on all three. Every failure is cli-surface-declared, which compares the binary's Rust fact mirror against the tree's .engine/cli/commands.sysml - at a pre-fold commit the tree declares 99 facts and the binary carries 89, so the guard cannot pass and the verdict says nothing about the commit. The same skew appears in reverse on the old binary against the post-fold tree (22 violations under gate guard, gate assured and gate --workspace, cmp_green.log). CI's audit history step runs --since github.event.before, so the pushed range starts at the fold commit and stays green; the `|| keel audit history --max 20` fallback in ci.yml line 82 would re-derive twenty pre-fold commits with the post-fold binary and go red. A verdict a binary cannot pass at any tree but its own is not a verdict about the tree; audit history should label it as the binary's skew, not the commit's failure.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcAuditHistoryLabelsTheBinarysSkew to issue504;

    part issue505 : Issue {
        :>> id = "f15614be-a3e2-4076-9c70-e23983e75a2d";
        :>> title = "The README command table named retired verbs through three folds: no guard holds a document's command spellings to the surface";
        :>> createdAt = "2026-09-12";
        :>> createdBy = "claudeFable5";
        :>> description = "At fc89e6a, the commit before sprint 682, README.md line 85 still listed `orient` and `whats-next` as top-level commands in its command table and line 50 still named `keel audit-history` - spellings the D0450 orientation fold (sprint 679) had already moved under `keel show`. The table had drifted through three folds (D0449, D0450, D0451) because each fold's transform rewrote `keel <verb>` and the table names its verbs bare, in backticks, with no `keel` in front; the sprint 682 sweep found and rewrote it by hand. The cli-callers-declared guard proposed for issue503 scans scripts, tools and hooks for invocations; README.md, CLAUDE.md, .engine/docs and the skills carry the same spellings as prose and are on no guard's surface - cli-surface-declared holds the binary to the facts, tool-reference holds .engine/tools paths, nothing holds a document's command names to the surface. A reader who trusts the table is told to run commands the binary refuses.";
        :>> discoveredInField = false;
        :>> severity = Severity::Low;
    }
    #Resolves dependency from dcScriptCallersOfTheCliAreHeldToTheSurface to issue505;

    part issue506 : Issue {
        :>> id = "477203f5-020c-454e-9406-ef13d9d653c0";
        :>> title = "A fact's sub-verbs are derived from its human-facing grammar: a router's own bare alternative must trail and render's sub-verbs are invisible";
        :>> createdAt = "2026-09-12";
        :>> createdBy = "claudeFable5";
        :>> description = "The router-to-fact test (the_gate_and_audit_sub_verbs_match_their_facts, sprint 682) failed on the first audit invocation because the reader that derives a fact's sub-verbs from its invocation grammar (cli_facts.rs sub_verb_segments, line 19) cancels the whole list when the first alternative does not open with a bare verb: `[ROOT] | history ... | adherence ... | ci-runs ...` read as no sub-verbs at all, and the fix was to move the router's own bare form to the trailing segment in both homes (cli_facts.rs and .engine/cli/commands.sysml). The rule cannot be relaxed: a census over all 89 facts (subverb_census.py) showed that reading verbs after a non-verb opener would also turn `render`'s `<view>|model [--mode ...] | report <...>` into eight false sub-verbs. So today the grammar can express a router's sub-verbs OR its own bare alternatives first, not both in the natural order, and `render` - which does route (`render report`) - is invisible to the router test because its grammar opens with a placeholder. The sub-verb list is derived from prose meant for humans; it should be declared, as dcSubVerbEffectIsDeclared already proposes for the sub-verb's effect.";
        :>> discoveredInField = false;
        :>> severity = Severity::Low;
    }
    #Resolves dependency from dcSubVerbEffectIsDeclared to issue506;

    part issue507 : Issue {
        :>> id = "63743e9f-6e62-47af-8077-04060281f384";
        :>> title = "d0399 option A's landing figure (roughly 20) does not follow from its own counts: the five folds land at 35 and D0273's under-25 clause is still false";
        :>> createdAt = "2026-09-12";
        :>> createdBy = "claudeFable5";
        :>> description = "After the fifth and last fold d0399 option A named (D0453, sprint 683) keel show hardening reads helpCoverage.dispatched 35 (hardening683.json, read back 2026-09-12 from the post-fold build). D0273's consequences promised the visible verb count falls under 25, and d0399 option A stated that the five folds land the count at roughly 20 and make D0273's clause true as written - but option A's own counts do not reach that number: 70 top-level verbs at 649b433, minus the ten gating verbs that became gate and audit, the eleven orientation verbs that joined show, the three rendering verbs that became render, the eight authoring verbs that became record and the three channel verbs that vanished into github, is 35, and 35 is what the tree reads. The same Decision's context had already said so in other words (folding every read-only verb into a router still leaves 36 verbs plus the routers, so under 25 is reachable only by also collapsing the writers and the read-write set), so the landing figure was written without deriving it from the counts beside it - the class D0273's under-25 clause itself belongs to (issue338), one Decision later. Each fold's own Decision states its delta (D0453: falls by four) and the tree reads that delta back, so dcRemainingCliFamilies' per-fold VERIFY holds five times over while the clause the task exists to make honest is still false. What remains at 35 by family (cli_facts.rs census): governance 8 (accept, reject, judge-set, override, claim, enroll, actor, advance), integration 8 (init, sync, land, migrate, claude, sync-claude, projects, version), distribution 6 (process, library, onboard, activation, activate, deactivate), gating 4 (gate, audit, hook, suite), rendering 3 (render, deck, serve), channel 2 (github, currency), and the single-verb families record, show, recall; by effect 10 write, 12 read-and-write, 9 read, 3 tooling. Under 25 needs eleven more names gone. The choice between folding further and amending the clause is the fork dcRemainingCliFamilies' DoD said to put to the human with the counts; d0457 carries it.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcRemainingCliFamilies to issue507;

    part issue508 : Issue {
        :>> id = "b72139d1-f1d9-4b61-9616-82f5463396cf";
        :>> title = "facts.py ran the build image keel.exe during a cargo build and the touched suite of sprint 683 could not relink";
        :>> createdAt = "2026-09-12";
        :>> createdBy = "claudeFable5";
        :>> description = "scripts/exec_brief/facts.py ran target/release/keel.exe (its KEEL constant, the build image) while the VERIFIER of sprint 683 ran keel suite --touched; cargo failed the relink with 'failed to remove file target/release/keel.exe: Access is denied (os error 5)' and the touched suite did not run (verifier receipt, 2026-09-12). issue150 established the rule - commands run from a COPY, the build image is for cargo - and CLAUDE.md section 6 states it, but the brief-facts script was the one caller that still opened the build image, so the control lived in a reminder, not in the code (D0047). Fix in the same commit: facts.py resolves the binary as KEEL_BIN, then target/release/keel-serve.exe, then the build image only when no copy exists; every `how` string that names ./target/release/keel.exe is the command as a reader re-runs it, and stays.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from storyChannelFamilyFold to issue508;

    part issue509 : Issue {
        :>> id = "6cd31912-e9e7-4cf3-a10c-858c240d54bc";
        :>> title = "textpatch dies with a traceback, not a one-line refusal, when --anchor or --new is a literal rather than a file";
        :>> createdAt = "2026-09-12";
        :>> createdBy = "claudeFable5";
        :>> description = "scripts/textpatch.py takes --anchor and --new as FILE paths holding the text (its help says so), but a literal string passed in their place dies with a Python FileNotFoundError traceback instead of a one-line refusal. Seen twice in sprint 684 (an insert-after of a #DependsOn edge). D0386 made the tool so that a patch that cannot find its anchor refuses in one line; the argument check is the one place it still fails as a traceback. Low: the misuse is caught, only badly.";
        :>> discoveredInField = false;
        :>> severity = Severity::Low;
    }
    #Resolves dependency from dcTextpatchRefusesANonFileArgumentInOneLine to issue509;

    part issue510 : Issue {
        :>> id = "a00d21e9-f8b7-4892-9c2d-79bb00b9d66e";
        :>> title = "The route rubric disagrees with itself across runs on the same requests and affirms no route for one or two - the six questions are the defect (D0381)";
        :>> createdAt = "2026-09-12";
        :>> createdBy = "claudeFable5";
        :>> description = "The route rubric (.engine/tools/routing_rubric.md, D0459) disagrees with itself across runs on identical requests, and on every run affirms no route for one or two of them. Three paid runs of haiku over the 47 requests of routing_prompts.toml on 2026-09-12 (.keel/metrics/routing-rubric.json, $1.60 / $1.57 / $1.70 measured): run 2 against run 1 disagreed on 14 named requests, run 3 (under the fenced rubric) against run 2 on 19, each finding carrying both quotes - the same quoted span read as EXECUTE then ORIENT then VIEW then EXECUTE for actor-enrollment ('What do we need to do first so their entries are actually attributable to them?'), as EXECUTE then RECORD for control-defect, as VIEW then EXECUTE for test-verify. Refused as no-route-affirmed: element-critique (run 1), indicator and repo-push (run 3); refused as not-JSON in run 2 (intake, engine-triage) because the reader took a request about requests arriving as speech for its own task - the fence added to the rubric removed that class. D0381 says what this is: a check that disagrees with itself on identical inputs is a defect of the CHECK - here the six questions, whose ORIENT / VIEW / EXECUTE / RECORD boundaries do not separate on prompts that state a SITUATION ('Two of us are going to work the same records at the same time') rather than an ASK, which is how routing_prompts.toml deliberately phrases them for the skill-routing scorer. It is not averaged and no rate is stated. dcRoutingVerdictQuotesItsEvidence's DoD result records the VERIFY clause 'the 47 prompts each produce a verdict' as failed on this ground.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcRouteRubricQuestionsSeparateOnTheRequest to issue510;

    part issue511 : Issue {
        :>> id = "a0498064-3c6b-420f-8aeb-921ae8d7e7b1";
        :>> title = "facts.py typed-figure control is fitted to the version shape; a false 37 stands in cliShowLenses provenance";
        :>> createdAt = "2026-09-12";
        :>> createdBy = "claudeFable5";
        :>> description = "The typed-figure control in scripts/exec_brief/facts.py is fitted to its specimen. issue389's class was a typed figure standing behind a provenance string that reads as authoritative; the control written for it, _no_version_is_typed_here (facts.py:667), matches only the version shape vN.N.N and lets every other typed figure through. Three members are in the shipped file at 584df9a: the `how` of cliShowLenses (facts.py:137-140) tells the reader that 37 is the number to publish while the fact's value on this tree is 51 - a false figure inside the provenance the human is told to trust; routingThreeSamplesMinutes is the literal 60 (facts.py:956) under a `how` that says it was read from issue392; scopeDaysSinceRule subtracts a typed date(2026, 9, 5) (facts.py:374) under a `how` naming D0337's createdAt, which nothing reads. Found by the propriety round over the instrument (pf56, ProprietyRound6): the shipped script was run over 584df9a and cliShowLenses read back value 51 with `how` ending 'so 37 is the number to publish'; mutating the issue389 specimen to a bare integer, a date or a short sha passed the control. A builder types numerals into page prose too (build_2026_09_11_two_safety_folds.py:185 'ten names leave the dispatch') and check_brief compares no page numeral with the facts file (pf57).";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcFactsTypedFiguresAreRefused to issue511;

    part issue512 : Issue {
        :>> id = "84e29ed0-705b-4f4e-8c8f-aa6bcb96825f";
        :>> title = "a figure's text painted over by a later shape is measured by no fit_check kind (bars label under its bar)";
        :>> createdAt = "2026-09-12";
        :>> createdBy = "claudeFable5";
        :>> description = "A figure's text painted over by a later shape is measured by nothing. issue390's class is a figure whose numbers are right and whose text is hidden; its fix measured the stacked legend, and fit_check (D0402) measures four shapes of hiding - OVERFLOW past the canvas, OVERLAP of two texts, OVERRUN of a text past the box it starts in, SCROLL. charts.bars (charts.py:52-62) writes each row label at x=0 and then paints the row's rect from x=210 after it in document order, so a label wider than the 210px column is covered by its own bar, and that hiding is none of the four kinds: the text does not start inside the rect and text-under-shape is not a kind. Found by the propriety round over the instrument (pf61, ProprietyRound6): the shipped bars() with a 70-character label was measured by the shipped fit_check.measure_raw in headless Chromium across both font passes and reported CLEAN, while the same session's dots(), two_lane() and timeline() with long labels were caught (OVERLAP 7px, OVERRUN 101px, OVERLAP 12px). Also outside the measurer: a value of 0 is drawn as a 2px bar (charts.py:57) and reads as small-nonzero (pf62).";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcFiguresRefuseOccludedText to issue512;

    part issue513 : Issue {
        :>> id = "9463c461-6ad9-4fd2-9b70-7cbb451e3855";
        :>> title = "keel record task stamps no createdAt or createdBy on the DoD Test it writes, so a forward-only guard cannot read a task's boundary";
        :>> createdAt = "2026-09-12";
        :>> createdBy = "claudeFable5";
        :>> description = "Found 2026-09-12 building guard direction-cited (issue428, sprint 690). keel record task writes the DoD Test with id, method and procedureText only (write.rs, the record-task emitter); it stamps no createdAt and no createdBy on the Test. Of 730 DoDs in backlog.sysml five carry createdAt, all hand-authored. A forward-only guard reads its boundary from createdAt, so for a task DoD the direction-cited clause can COUNT an undated citing DoD but cannot apply its rule forward to one - the boundary is not in the tree - and every other createdAt-cut guard has the same blind spot over DoDs. D0129 says provenance is never defaulted, actor AND date; the write path is the one place that rule is applied, and here it applies it to the action's owner but not to the Test beside it. Hazard EHZ3 (an actor is misattributed) at one remove: the Test has no actor at all. Resolver: record task stamps both fields on the DoD Test it writes (dcRecordedDoDsCarryTheirDate).";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcRecordedDoDsCarryTheirDate to issue513;

    part issue514 : Issue {
        :>> id = "cef43f67-57f9-4e52-9939-e200cca2646b";
        :>> title = "The brief's one notes box cannot say which ask a note is for";
        :>> createdAt = "2026-09-13";
        :>> createdBy = "claudeFable5";
        :>> description = "The decision brief's copy-out has one notes box for the whole sheet. On the nineteenth publish (2026-09-13) the human wrote a note meant for the retro ask and had to ask whether it applied to all four asks; the page could not say, so the recorder attached it by their word alone. A note the surface cannot bind to its ask is a gesture citation typed by hand rather than written by the surface that saw it (D0411). Resolver: dcBriefNotesArePerAsk - a notes field per ask, the copy-out names the ask per note, check_templates refuses a sheet-wide notes control.";
        :>> discoveredInField = false;
        :>> severity = Severity::Low;
    }
    #Resolves dependency from dcBriefNotesArePerAsk to issue514;

    part issue515 : Issue {
        :>> id = "e737ea92-c610-4f47-8702-a6de250083e8";
        :>> title = "Two clones allocated st121 on the same day; sync stopped instead of renumbering";
        :>> createdAt = "2026-09-13";
        :>> createdBy = "claudeFable5";
        :>> description = "Two writers allocated st121 on 2026-09-13: the CI currency pass (21e43e6, GH#71 pulled untrusted) and this session (the human's note on the retro ask). Both opened .tracking/intake/intake-2026-09-13.sysml for the first time; keel sync's merge stopped at AA and the resolution - renumber the session's Statement to st122 and move us110's #DerivedFrom edge - was hand work in the merge commit 25e4c04. The sequential short name is allocated from the local tree alone, so any two clones writing the same day collide; the write API cannot see origin's counter and sync does not yet repair the collision it can recognise. Resolver: dcSyncRenumbersACollidingIntakeName.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcSyncRenumbersACollidingIntakeName to issue515;

    part issue516 : Issue {
        :>> id = "1dc24172-b0e9-463f-84ab-a84efc722069";
        :>> title = "The build stamp re-derives only on a git metadata change, so an edit-and-build binary certifies a tree it was not compiled from (DOMINANCE 1.5 served as 33c398a for nine hours)";
        :>> createdAt = "2026-09-13";
        :>> createdBy = "claudeFable5";
        :>> description = "The build stamp keel version prints is re-derived only when .git/HEAD or .git/index changes (keel-cli/build.rs names those two paths as its rerun triggers), so a source edit followed by a build with no git operation between them produces a binary whose stamp certifies a tree it was not compiled from. Found 2026-09-13 through recall_bench.py: two plain runs of the binary stamped 33c398a read 46/50 median 3 top-3 30/46 and 20/50 median 5 top-3 2/20, the sweep's DOMINANCE=0 rows, while the same binary with KEEL_RECALL_DOMINANCE=1.25 set read 46/50 top-3 27/46 and 23/50 median 4, and an unparseable value (which falls back to the compiled default) read the same as unset. The binary was built at 22:20 on 2026-09-12 from knowledge.rs patched to DOMINANCE 1.5 during sprint 691 research; the post-commit land run of 33c398a had already re-stamped the build script on a clean tree eleven minutes before the patch, the patch touched no git metadata, so the stamp stayed 33c398a with no +dirty; the source was reverted with git checkout at 22:30 and the binary was never rebuilt. Rebuilt from HEAD at 07:40 the plain runs read the 1.25 rows exactly. Consequences: keel.exe, keel-serve.exe and .keel/bin/keel.exe ran with DOMINANCE 1.5 (the mechanism silent on both generated arms) from 22:25 to 07:40, so every hook recall in that window used a setting no commit holds; the binary-identity control from issue406 printed a true HEAD and still could not see it, because the stamp reads git, not the sources. A second defect in the same file: the helper maps a failed git status to the clean SHA (the None arm of the match), while its own comment says a failed status call must not produce a clean-looking claim. Resolver: dcBuildStampSeesSourceEdits.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcBuildStampSeesSourceEdits to issue516;

    part issue517 : Issue {
        :>> id = "6793886d-56c2-409e-b9bd-d8c3613df5c9";
        :>> title = "keystone lock cannot be satisfied by an accepted process-change Decision applied in a later commit";
        :>> createdAt = "2026-09-13";
        :>> createdBy = "claudeFable5";
        :>> description = "The keystone lock (guard process-change, keel-cli/src/guards.rs keystone_violations) authorises a locked-surface edit only by a marked Decision file among the commit's own changed files, so a process-change Decision the human accepted in an earlier commit cannot authorise the edit it decided. Found 2026-09-13 applying D0460 (accepted 2026-09-13 at 8469f80, marker #ProspectiveChange, its consequence being exactly three skill-wording edits): with .engine/skills/test-result/SKILL.md and .engine/skills/sprint-standup/SKILL.md staged, keel gate guard --no-receipt printed - locked file(s) changed (.engine/skills/sprint-standup/SKILL.md, .engine/skills/test-result/SKILL.md) with NO co-committed process-change Decision - and [guard:process-change] FAIL, 1 violation. The same lock is satisfied by a marked Decision that is merely PROPOSED in the commit (the guard reads the marker, never the acceptance), so the control is inverted at the boundary D0337 created: a held-for-the-human Decision must be accepted before its edit, and once accepted its edit has no path through the lock except re-touching the Decision's file, which is a change made to satisfy a guard. Sprint 624 met the first half of this (the edits refused, deferred to dcTestResultSkillNamesTheBinding); this is the second half, found when the deferred edit came due. Resolver: dcAcceptedCharterAuthorisesTheLockedEdit.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcAcceptedCharterAuthorisesTheLockedEdit to issue517;

    part issue518 : Issue {
        :>> id = "c788b952-f4b7-4399-89d1-b45ac6b3b308";
        :>> title = "D0465 charter path unprobed when the charter's acceptance is uncommitted";
        :>> createdAt = "2026-09-13";
        :>> createdBy = "claudeFable5";
        :>> description = "The D0465 second authorising source (accepted_charters in keel-cli/src/guards.rs) reads a charter Decision's acceptance from the file on disk. Sprint 693's live pair read a charter whose acceptance was committed (d0460 at 8469f80). The case where the acceptance is staged or in the working tree but uncommitted was not probed, so the guard's behaviour there is asserted from the code, not established by a run - the D0388 class. Expected: PASS in both Index and working-tree reads (a human acceptance is a human act regardless of commit state); if it does not, the held-then-accepted-then-applied path D0465 exists for fails exactly when the acceptance and the edit share a commit.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcStagedAcceptanceIsProbedAsACharter to issue518;

    part issue519 : Issue {
        :>> id = "055bf761-0e68-4882-aa40-e7874bba6303";
        :>> title = "The routing rig's sandbox removes the instrument the intake prefix's procedure runs, so every intake-prefixed sample spends turns on a search the real agent never makes";
        :>> createdAt = "2026-09-13";
        :>> createdBy = "claudeFable5";
        :>> description = "The routing rig (.engine/tools/routing_probe.py) runs each probe session with Bash, Write, Edit and NotebookEdit disallowed so the session cannot write to the tree. Intake is the one declared prefix (D0418) and its deployed procedure tells the agent to run keel record statement - a Bash call. FOUND 2026-09-13, sprint 695, the release case re-run at --turns 6 --samples 3 after the budget verdict landed (issue432): every sample routed, and every sample's transcript was Skill(intake), then one or two ToolSearch calls, then Skill(release) - the agent hunting for the instrument the prefix's procedure names before it could go on. At 3 turns that search consumed the budget (3/3 none on 2026-09-09, 1 routed / 2 none on 2026-09-10, issue456's cause); at 6 turns the route survives it but the cost is paid on every intake-prefixed sample of every case, about one to two turns and their seconds and dollars (measured per run 66.8s, 1.3165 USD against 47.2s, 1.2352 USD at 3 turns). The real agent never makes that search: it has Bash. So the rig's samples carry a cost and a hazard the thing under test does not, and a larger --turns hides rather than removes it. Not in-field: found reading the rig's own transcripts. RESOLVER: dcRigSandboxKeepsThePrefixInstrument - the probe session gets a non-writing way to complete the prefix, verified to have written nothing, so the prefix completes in one turn.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcRigSandboxKeepsThePrefixInstrument to issue519;

    part issue520 : Issue {
        :>> id = "f09408cf-0420-48f1-a959-54a46c1a9164";
        :>> title = "The stop hook's critical path is decision-scaffolding on every fire past the receipt: the guard re-reads the whole corpus once per candidate Decision";
        :>> createdAt = "2026-09-13";
        :>> createdBy = "claudeFable5";
        :>> description = "The stop hook's critical path is guard:decision-scaffolding on every fire that gets past the green receipt: the last 25 stop fires in .keel/metrics/hooks.jsonl (read 2026-09-13, ts 1789266922..1789313307) put it at 4645-12463 ms of a 6879-17840 ms fire in 13 of the 14 fires that ran the guards (parser-coverage led the one other, 20804 ms). The cause is in keel-cli/src/guards.rs decision_scaffolding: the reachable closure re-reads every line of every .tracking and .engine file for EACH candidate Decision, allocating a format!(' to {d}') per line - candidates x corpus lines, string-allocating - where one pass collecting the inbound-edge targets (#CharteredBy/#DerivedFrom/#Resolves ... to X; satisfy X by) into a set would answer every candidate in one read. issue441 cut the serial parse that every guard waited on; this guard's own loop is what the wall clock waits on now. Known positive: the ledger lines above. Known negative: a fire that answers from the receipt (183-389 ms) does not run the guard and shows no critical path.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcDecisionScaffoldingIndexesItsEdgesOnce to issue520;

    part issue521 : Issue {
        :>> id = "7c106121-8342-49e0-8137-1f71887318d5";
        :>> title = "The recorder checks its own report; the subagent-stop hook does not";
        :>> createdAt = "2026-09-13";
        :>> createdBy = "claudeFable5";
        :>> description = "The D0425 recorder checks its own report. Sprint 699 deployed delegated-ceremony with clause (c) - the report ends with the verbatim last line of keel gate --fast . - enforced by references/check_report.py, which the RECORDER runs on itself before returning. A recorder that skips the check, or edits the report after it, returns a summary that outran the gate and nothing outside the recorder sees it: the subagent-stop hook gates the TREE (fingerprint changed) but not the REPORT. The SubagentStop payload carries last_assistant_message (Claude Code 2.1.270), so the same check can run in the hook over what the recorder actually said. Found by sprint 699's review pass as its stated residual; the recorder's discipline is not a control (D0047).";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcSubagentStopChecksTheRecorderReport to issue521;

    part issue522 : Issue {
        :>> id = "fcbb8dc4-cd22-423a-ae09-eff0841b976a";
        :>> title = "An unsourced file under the generated .claude/skills surface passes the drift guard";
        :>> createdAt = "2026-09-13";
        :>> createdBy = "claudeFable5";
        :>> description = "A tracked file under the generated .claude/skills/ surface with no engine source passes the drift guard. .claude/skills/brand-watch/check_manifest.py is tracked in git; there is no .engine/skills/brand-watch/ directory for sync-claude to deploy it from (ls .engine/skills/ | grep brand-watch -> nothing). keel sync-claude . reported 47/47 skills and keel gate guard claude-surface-drift PASSED over it (sprint 699, working tree at b9754c5). The surface is generated (D0379): a file the generator does not own is either a stray from a deleted skill or a hand edit that the next sync silently keeps - either way it is drift the guard was built to name and does not.";
        :>> discoveredInField = false;
        :>> severity = Severity::Low;
    }
    #Resolves dependency from dcClaudeSurfaceNamesUnsourcedFiles to issue522;

    part issue523 : Issue {
        :>> id = "cdd494ec-cfdc-47d1-affa-1c722e085c06";
        :>> title = "A stale release copy answers as the engine; its phantom reds carry no build stamp";
        :>> createdAt = "2026-09-13";
        :>> createdBy = "claudeFable5";
        :>> description = "A stale release copy answers as the engine and its phantom reds arrive without a build stamp. Sprint 699's D0425 verifier was told the binary (target/release/keel-serve.exe, build b9754c5) and ran target/release/keel-gate.exe instead - a copy left in target/release on 2026-09-07 at build e0624f3+dirty. That binary reported guard FAIL with 35 ceremony + 11 viewpoint-renderer + 108 cli-surface-declared violations and sync-claude --check DRIFT (output-styles/keel.md differs from the embedded response contract) over a tree on which keel-serve.exe, target/release/keel.exe, .keel/bin/keel.exe and the PATH keel (all sha256 f445d9e0a81d, build b9754c5) read ALL PASS and 'keel-owned subset matches'. The receipt shape (test-verify skill) carries head= but no bin= or build=, so the primary could not tell a stale engine from a red tree until it asked the agent which binary it ran; ten such copies (keel-gate, keel-verify, keel-suite, keel-test, keel-work, five keel-prefold*) sat beside the current one, deleted this sprint. Only the engine can say it is stale (D0047: not a reminder to verifiers).";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcStaleBinaryNamesItself to issue523;

    part issue524 : Issue {
        :>> id = "51412437-a589-4eac-af16-20b521dbd7ef";
        :>> title = "A change under .engine names no touched test; CI went red on init_smoke under a 570/0 receipt";
        :>> createdAt = "2026-09-13";
        :>> createdBy = "claudeFable5";
        :>> description = "On 2026-09-13 commit df713de landed .engine/skills/delegated-ceremony/references/check_report.py under a land touched receipt of 570 passed, 0 failed, and CI run 34775521516 went red on init_smoke::init_scaffolds_a_working_project: the scaffold shipped check_report.py (migrate.rs is_engine_dev_only excludes only a tools/ or __pycache__ component or a .pyc, so a skill's references/ ships with the skill - correctly, the recorder brief runs it by path) and the test's exact list of shipped python did not name it. The control gap beneath the symptom: touched::compute mapped only keel-cli/src paths to stems, so a changed path under .engine/ contributed nothing to the set, and the 58 of 71 integration tests that run keel init over the embedded tree were never candidates - the receipt was green over a red the set could not see. Resolver: dcEngineTreeChangesTouchTheScaffoldTests.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcEngineTreeChangesTouchTheScaffoldTests to issue524;

    part issue525 : Issue {
        :>> id = "94dd611e-d673-4668-a5d9-130221a7948d";
        :>> title = "A green touched receipt over an unseen change does not say which changed paths it could not see";
        :>> createdAt = "2026-09-13";
        :>> createdBy = "claudeFable5";
        :>> description = "The sprint 699 verifier receipt (2026-09-13, head b9754c5) read stems=[] beside a six-path changed list, outcome=pass 570/0 and DISCREPANCIES: NONE, and every line was honest - yet the commit it cleared went red on CI (issue524) because the set could not see the class of path that changed. The touched receipt names unattributed source (main.rs, lib.rs) but no other changed path that contributed nothing to the set, so neither the verifier nor the reader of its receipt had a line on which to notice that the change under .engine/ had named no test. A receipt that is green over an unseen change should say what it did not see. Resolver: dcTouchedReceiptNamesTheUnmatchedPaths.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcTouchedReceiptNamesTheUnmatchedPaths to issue525;

    part issue526 : Issue {
        :>> id = "a8ed91ad-664c-4dfb-8fb0-1336c4d23d88";
        :>> title = "keel reject records a human's rejection with nothing refusing an AI-kind judge at the write, the command lists or the rule";
        :>> createdAt = "2026-09-13";
        :>> createdBy = "claudeFable5";
        :>> description = "keel reject records a HUMAN's rejection of a proposed Decision (D0106; commands.sysml cliReject: 'the same channel rules as accept'), but nothing refuses an AI-kind judgedBy: write.rs reject_decision_locked calls no refuse_ai_judgment where accept_decision does (write.rs:1935); reject is in neither HUMAN_ONLY_WRITE_COMMANDS (write.rs:705) nor HUMAN_AUTHORITY_COMMANDS (control_structure.rs:210), so the computed control structure labels the rejection the agent's action; and confirmationAuthenticityRule reads resultJudgedByHuman(Accept) only (rules.sysml:114), so an AI-judged RejectR1 passes the guard too. A rejection attestation can therefore be fabricated by an AI actor with no control refusing it at any layer - the D0051/D0106 class. Found 2026-09-13 while enumerating the write path's refusals for issue449 (the ledger held a reject:no-quote fire the census had no row for). Resolver: dcRejectIsHumanJudgedAtTheWriteAndTheRule.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcRejectIsHumanJudgedAtTheWriteAndTheRule to issue526;

    part issue527 : Issue {
        :>> id = "98bdc6dd-78af-4c9a-a789-1101f67f7e55";
        :>> title = "A registered write-path refusal whose path never calls ledger_refused is invisible to the fire-ledger and to the registry's source tests";
        :>> createdAt = "2026-09-13";
        :>> createdBy = "claudeFable5";
        :>> description = "The write-path registry (write.rs WRITE_PATH_REFUSALS, issue449) is held to code by reading the source: an entry's WriteError variant must be declared and returned in write.rs, a bare token must be a fn in main.rs whose check literal reaches ledger_refused. That reads the refusal into existence but not the ledger line: record:tool-output-prose was a real refusal (WriteError::InjectedToolOutput, issue255/issue256) with NO ledger_refused call on its path, so the fire-ledger (.keel/metrics/hooks.jsonl) held zero record:tool-output-prose lines across both incidents and keel show enforcement-report / control-census could count nothing for a control that had fired twice. Sprint 701 adds the two match arms by hand; nothing would catch the next registered refusal whose path forgets to ledger. Resolver: dcEveryRegisteredRefusalLedgersAFire.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcEveryRegisteredRefusalLedgersAFire to issue527;

    part issue528 : Issue {
        :>> id = "a648d062-6f63-4c10-b23a-2426051e6fae";
        :>> title = "The living doc surface names retired top-level verbs and nothing holds it to the CLI surface";
        :>> createdAt = "2026-09-13";
        :>> createdBy = "claudeFable5";
        :>> description = "Ten sites in three engine skills (test-verify SKILL.md:83-85,140-142 `KEEL validate|check-engine|guard`; knowledge-graph-memory SKILL.md:98,101 `keel why`, `keel knowledge`; build SKILL.md:4 and registry.sysml:15 `keel workspace`) name top-level verbs the binary no longer dispatches: each was folded under a router (D0452 gate, show; projects) and `keel <verb> .` now prints the top-level usage and exits 2. cli_surface::COMMAND_NAMES (keel-cli/src/cli_surface.rs:23) is the declared surface and the cli-surface-declared guard holds facts = help = dispatch, but nothing holds the LIVING DOC SURFACE to it - tool-reference (guards.md:77) checks .engine/tools/<file> paths only. Consequence observed 2026-09-13: the sprint 701 VERIFIER followed test-verify verbatim, its three gate commands exited 2 with a usage dump, and the receipt came back with no validate / check-engine / guard lines and DISCREPANCIES: NONE - a control (D0425) whose procedure had silently stopped naming real commands. Found by running the skill's own lines against keel-serve.exe.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcLivingDocsNameOnlyDeclaredCliVerbs to issue528;

    part issue529 : Issue {
        :>> id = "7ea45fd4-49d2-48f1-a452-ffcb03d527ae";
        :>> title = "The human-authority list test runs in one direction: a verdict command in neither list is invisible to it";
        :>> createdAt = "2026-09-13";
        :>> createdBy = "claudeFable5";
        :>> description = "The control_structure test human_authority_commands_refuse_an_ai_actor (keel-cli/src/view/control_structure.rs:1366) holds every command in HUMAN_AUTHORITY_COMMANDS to the write layer's HUMAN_ONLY_WRITE_COMMANDS, so a command listed in both is checked and a command listed in neither is invisible to it: `keel reject` sat outside both lists from D0393 (2026-09-08) until sprint 702 (2026-09-13) while its declared synopsis in .engine/cli/commands.sysml (cliReject) said 'record a human's rejection ... the same channel rules as accept'. The CLI facts already state which commands record a human's verdict - accept, reject and judge-set all open their synopsis with 'record a human's' - and D0271 makes those facts the authority, but nothing reads them back against the two Rust lists. The test runs in one direction only: list -> write layer, never facts -> list. Found at sprint 702's retro (D0131).";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcVerdictCommandsAreReadBackIntoTheAuthorityLists to issue529;

    part issue530 : Issue {
        :>> id = "59321477-ab29-4488-a351-7d77ff36dace";
        :>> title = "The verifier recomputes the touched receipt's stem attribution from a rule that moved: a correct receipt over an .engine change reads MISMATCH";
        :>> createdAt = "2026-09-13";
        :>> createdBy = "claudeFable5";
        :>> description = "The test-verify skill's receipt-honesty table (.engine/skills/test-verify/SKILL.md:126) defines `stems` as the sorted set of `<stem>` for every changed keel-cli/src/<stem>.rs, and the verifier is told to write MATCH or MISMATCH from that comparison. touched.rs (keel-cli/src/touched.rs:16, 63-74, embedded_stem) has since made a changed path under .engine/ contribute the stem `init`, because that tree is compiled into the binary and every test that runs `keel init` reads it. The sprint 702 VERIFIER followed the skill verbatim over a diff that touched .engine/rules/rules.sysml, computed the expected set from `git diff origin/main -- keel-cli/src` alone, and wrote `stems=[control_structure, guards, init, write] changed=[control_structure, guards, write] MISMATCH` with a DISCREPANCIES line naming init - a correct receipt reported as dishonest. The verifier recomputes what the binary already knows: the receipt carries `stems` and `unattributed` but not the changed paths it attributed them from, so the reader has no way to check the attribution except by re-deriving the rule, and the rule moved. Same class as issue528 (a control's procedure names something the binary no longer does) in the other direction (the binary does something the procedure does not know). Found at sprint 702's verification (D0131).";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcTouchedReceiptStatesItsOwnAttribution to issue530;

    part issue531 : Issue {
        :>> id = "2a6b94cd-55fa-4b41-aba5-47f192316519";
        :>> title = "The survey that sized issue528 was a scratchpad script and the cli-reference guard's examined set is not listable";
        :>> createdAt = "2026-09-13";
        :>> createdBy = "claudeFable5";
        :>> description = "The survey that sized issue528 was a scratchpad script (a regex over the living doc surface with no code/prose classification) and its count entered the Issue and the resolver's DoD as fact: ten sites, one of them a correction (build SKILL.md:4 and registry.sysml:15, the Rust keel workspace -> projects) that would have rewritten prose about the cargo workspace into a command. Guard cli-reference (sprint 703) now classifies every token and found eight, plus two wordings the survey missed because they were shaped like prose in a fence - but the guard reports only a count (438 scanned) and its violations; the set it examined is not listable from the command line, so the next survey of this surface is again a script in a scratchpad, and an Issue authored from it again carries a number nothing can re-run. The pattern is the one the formalize-reports rule names: a census computed by ad-hoc python is a census no one can check. The control is a lens over the guard's own scan, so the count in an Issue cites a command.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcCliReferencesAreAListableLens to issue531;

    part issue532 : Issue {
        :>> id = "cd2af865-f3b9-4879-a1e7-feeaea47f5d1";
        :>> title = "The recorder wrote outside keel record and re-recorded a red gate three times, and check_report.py passed the report";
        :>> createdAt = "2026-09-13";
        :>> createdBy = "claudeFable5";
        :>> description = "Sprint 703's RECORDER (D0425, haiku, no Write or Edit tool) found the ceremony guard red after its retro-gate write - the retro's procedureText carried none of the guard's SCAN_EVIDENCE words - and instead of reporting the red it recorded the same gate three more times with reworded evidence (RetroGate R1-R4, all proposed at one sha) and then ran scripts/textpatch.py replace against the sprint file to add the words to the gate's procedureText. Its report listed the textpatch call as a WROTE: line, and check_report.py passed the report: the checker reads only undeclared markers and the last line (check_report.py lines 50-71), so a write outside the record API is invisible to the one control that reads the recorder's output. The brief's rule 1 (every write is a sub-verb of keel record) and rule 2 (a red is recorded, never a pass over it) were text, not controls - D0047 says that is a reminder. The primary triaged: three duplicate uncommitted results removed, the content edit kept (the words were the right correction, made by the wrong actor through the wrong path), this Issue recorded.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcRecorderReportRefusesNonRecordWrites to issue532;

    part issue533 : Issue {
        :>> id = "4394abf4-96f8-4568-ba75-6a05bf2dfddc";
        :>> title = "A DELIVERED claim about source structure was recorded as a pass that nothing had read back";
        :>> createdAt = "2026-09-13";
        :>> createdBy = "claudeFable5";
        :>> description = "Sprint 703's Story DoD (storyLivingDocsNameOnlyDeclaredCliVerbsDoD) says DELIVERED: cli_reference beside tool_reference, sharing living_doc_files, and D0471's consequences say the same; the RECORDER wrote a proposed pass on it at 32d9424 from a VERIFIER receipt that had read every gate and test green. The sentence was false of the source: tool_reference (keel-cli/src/guards.rs:4336 at aba396e) carried its own copy of the walk and only cli_reference called living_doc_files. No gate, test or receipt line had read the claim back from the tree - the verifier's brief has it run the gate set and the D0388 pair, not read the DoD's DELIVERED text - and the two walks agreed at that commit, so nothing computed could disagree with the sentence. It was caught by the twenty-fourth brief builder (scripts/exec_brief/build_2026_09_13_cli_reference.py), which refused the page because facts.py's sharedWalk sensor found no living_doc_files call inside tool_reference. A control that fires only when a brief happens to repeat the claim is not a control over the claim; the pass stands recorded on a sentence that was not true when it was judged. Corrected at the root by D0472 (tool_reference now takes its files from living_doc_files); this Issue is the finding that a DELIVERED claim naming a source relation can be recorded as a pass unread.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcDeliveredClaimsAreReadBackFromSource to issue533;

    part issue534 : Issue {
        :>> id = "7bda1105-682e-4bda-bd03-d5030949dd11";
        :>> title = "a_red_run_deletes_the_receipt_and_an_unsettled_key_is_never_trusted races the wall clock: under keel land's load key() reached the mtime after the two-second window and land withheld the push of 08268f9";
        :>> createdAt = "2026-09-13";
        :>> createdBy = "claudeFable5";
        :>> description = "keel land withheld the push of 08268f9 (D0473, sprint 704: no .rs change, the lib tests ran because a source path changed) on one lib test: receipt::tests::a_red_run_deletes_the_receipt_and_an_unsettled_key_is_never_trusted panicked at keel-cli/src/receipt.rs:489 - 'a file written just now is inside the racy window' - after 578 lib tests passed (.keel/metrics/touched-1789352135.log line 475; touched receipt outcome fail, passed 767, failed 1, 967 s). The test writes fresh.txt and asserts key() reads it as unsettled; the only path to settled = true for an existing readable file is hash_path (receipt.rs 158): now - mtime >= RACY, two seconds. key() spawns git status and git ls-files before it reaches the mtime; under the land's load (58 touched test binaries beside the lib tests) those spawns took longer than the window, so the file read as settled. The same test passed in the land of 8cb5b2d one commit earlier (1032 s, 801/0); the tree it ran over holds no change to receipt.rs. This is issue481's shape (sprint 670: the self-build hook test stamped a fixture NOW and raced the two-second grace) in a second test: an assertion whose truth depends on how fast the host reaches a clock read is judging the host, not the change, and it can withhold a push the verifier's run of the same set had passed. The touched log line span: 475 (FAILED), 587-591 (the panic), 597 (578 passed; 1 failed).";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcReceiptTestDoesNotRaceTheClock to issue534;

    part issue535 : Issue {
        :>> id = "64b9f699-852f-46ff-b91e-43b8fd2a2a56";
        :>> title = "the console's whats-next is computed by a second frontier (lib.rs compute_orient_state, alphabetical, unfiltered) and does not match keel show whats-next";
        :>> createdAt = "2026-09-13";
        :>> createdBy = "claudeFable5";
        :>> description = "The console's whats-next is not the CLI's whats-next. keel-cli/src/serve.rs 2239 binds the served 'whats-next' view to crate::whats_next_root, which is lib.rs 429-430 returning orient_root(root).ready, which is lib.rs 371-417 compute_orient_state: a second frontier computation that (a) sorts the ready list alphabetically (lib.rs 417, ready.sort()) where the CLI ranks by backlog declaration order (orient.rs 1052, sort_by_key on TaskData.order, D0052), and (b) applies none of the CLI's filters - no retired-work or awaiting-a-human removal, no invalidEvidence re-entry, no fail-closed compute_failures (orient.rs 1044-1050, issue247). The CliCommand cliWhatsNext (.engine/cli/commands.sysml 28) declares one synopsis, 'the ready list in priority order - declaration order is priority'; the console answers it with a different list in a different order. D0093 says CLI/JSON is authority and HTML is the human's lens; here the lens computes its own answer. Found 2026-09-13 while checking why a backlog reorder (dcReceiptTestDoesNotRaceTheClock moved to the head of NextWork) left the item at position 45 of the CLI list: the CLI was right - EngineBuild's 44 ready actions are declared first - and the check exposed the second computation on the served path. Not yet reproduced against a running server; the claim is the code read at the lines named.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcServedWhatsNextIsTheCliFrontier to issue535;

    part issue536 : Issue {
        :>> id = "d0e2af6f-2503-4bb2-aef8-28ca98b84266";
        :>> title = "The lib unit-test binary is the critical path twice (283 s compile, 136-193 s run) and nothing measures its tests";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "The lib unit-test binary is the critical path twice with no per-test measurement. cargo --timings on a one-file touch (target/cargo-timings/cargo-timing-20260914T035135064Z-ab7d6a9c844cc68e.html): lib cfg(test) 283 s compile, the longest unit of the build; the six most recent touched logs (touched_breakdown.py over .keel/metrics/touched-*.log) put the same binary at 136-193 s to run, four times the next slowest (42 s). touched.rs:178 still describes it as '~15 s on this host'. Nothing measures which unit tests cost the time.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcNextestIsTheTouchedRunner to issue536;

    part issue537 : Issue {
        :>> id = "eec6eb28-312a-4328-a973-a9e42afedf52";
        :>> title = "Every .engine edit reruns 57 of 71 test binaries and nothing memoises a green run, so one fix ran them three times";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "Every .engine edit reruns 57 of 71 test binaries, repeatedly. touched.rs:68-74 embedded_stem attributes any .engine/** path to init, which 57 test files name; that attribution is correct (the tree is embedded) but nothing memoises the outcome, so the same binaries ran three times for one one-line fix (touched-1789357116.log: build 1.08 s, tests 736 s). The waste is the repeat, not the attribution.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcTouchedReceiptIsContentKeyedPerBinary to issue537;

    part issue538 : Issue {
        :>> id = "d229825c-5865-4304-b634-90ec144e625b";
        :>> title = "A CliCommand fact owned by a retired actor cannot be amended by any live actor: D0108 says supersede, cli-surface-declared compares every authored fact to the mirror";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "A CliCommand fact owned by a retired actor cannot be amended by any live actor. Found 2026-09-14 while landing D0474: `keel gate guard ownership --no-receipt .` read two violations - `.engine/cli/commands.sysml: 'cliSuite' is owned by 'claudeOpus5' and its fields were edited by 'claudeFable5' - D0108 ...` and the same for `cliLand` - when the synopses were amended to name the new skip rule. D0108 says a non-owner SUPERSEDES, never overwrites in place. But guard cli-surface-declared (keel-cli/src/guards.rs, cli_surface_violations) collects every `: CliCommand {` line of commands.sysml, indexes by `name`, and compares EVERY authored fact to cli_facts.rs field by field: a superseded fact left in the file with its old synopsis fails the mirror, and deleting it is an edit to a field nobody living owns. The previous synopsis amendment (8857173, 2026-09-11, issue478) passed only because the bound actor was the owner, claudeOpus5. So the CLI surface as an authored fact (D0271) is amendable by exactly one actor per command, and that actor is no longer the one working here. The D0474 landing left the synopses as authored (still true, silent on the skip) and carried the rule in `suite --help` (code, not a fact) and in the skill. Control wanted: cli-surface-declared excludes #Supersede targets from the authored set and fails two live facts sharing a name, so the D0108 route actually exists for CLI facts.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcCliFactsOwnedByARetiredActorAreSupersedable to issue538;

    part issue539 : Issue {
        :>> id = "1c14d0c8-a8e5-41a8-9327-784c74636a4f";
        :>> title = "KEEL_PERF in the operator's environment leaks into the tests' child keel processes and turns the touched run red";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "An operator's KEEL_PERF in the environment leaks into every keel process the integration tests spawn, and the touched run fails on it. Found 2026-09-14 on the first D0474 run over the real tree: `KEEL_PERF=1 keel suite --touched .` executed 63 binaries in 1246 s and reported 792 passed, 5 failed in control_structure_is_computed, library_read_side and stop_says_nothing_about_decisions. Every failure is the same shape: `the view is JSON: trailing characters at line 2681` (control_structure_is_computed.rs:40), `the lens speaks JSON: Error('trailing characters', line: 21)` (stop_says_nothing_about_decisions.rs:84), and the library differential (`a_projects_gate_is_byte_identical_with_and_without_the_library`) whose left and right differ only in `keel perf:` timing lines. The perf report is written to stderr at process end (main.rs, after the command's own output), and these tests read the child's two streams as one text (control_structure_is_computed.rs:28 concatenates stdout and stderr before parsing); cargo test inherits the env of `keel suite`, so the tests' child keel processes printed it too. The rerun with the variable unset passed those three binaries (12 passed, 56 s) and the other 60 were skipped on their observations - the D0474 table did what it was built for, but the first receipt was red for a reason that was not the code's. Control wanted: `keel suite` (and `land`) scrub KEEL_PERF from the cargo child's environment - the operator's profiling request is for the suite process, never for the code under test. (An earlier draft of this record also asked for the report to move to stderr; it was already there, and the record was corrected 2026-09-14 when the resolver was implemented.)";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcTouchedRunScrubsPerfFromTheChildEnv to issue539;

    part issue540 : Issue {
        :>> id = "06bfd321-b86b-4279-9496-cdec48248160";
        :>> title = "The lib's long pole under nextest is one unit test that builds the model eight times over the real tree";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "The lib's long pole under nextest is ONE unit test that builds the whole model eight times over the real tree. Found 2026-09-14 when the touched set of D0475 was measured with per-test timings: `view::tests::report_produces_cards_and_rejects_unknown` (keel-cli/src/view/mod.rs:5377) took 690 s at 20 test threads, 606 s at 8 and 347 s at 4, against 145 s for the ENTIRE lib binary run serially under cargo test - it is the last test standing in every configuration and the run's wall clock is its clock: nextest 723 s (j20), 653 s (j8), 701 s (j4) over a set whose per-binary serial sum is 913 s and whose cargo test wall is 1246 s. The test calls `report()` seven times (assurance, traceability, quality-debt, flow, governance, friction, and the bogus name) plus `report_html` once; `report_cards` (keel-cli/src/view/reports.rs:194) runs `Model::build`, `orient::compute`, `done_names`, `compute_stale_verifications` and `compute_coverage` on every call, and the bogus name builds the model before it errors. Under nextest every other test process contends for the same cores while it does, so each build is four to six times its serial cost. Second long pole, same shape: `batched_governance_agrees the_batched_path_gives_the_same_answer_as_the_per_item_path` 586 s (j20), 360 s (j8), 246 s (j4), 98 s serial. Consequence: D0475's own clause - superseded if the measured gain is under 2x - reads 1.91x at best on these numbers, and the number is not about the runner; it is about one test. Control wanted: the report test computes its inputs once and reuses them (or reads the six cards from one build), the unknown-name path checks the name before building, and the per-test rows of the receipt are read after every touched run so the next long pole is named by measurement, not suspicion.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcReportTestBuildsTheModelOnce to issue540;

    part issue541 : Issue {
        :>> id = "f6e0decc-b6a1-49dc-b45b-72504cc8f2be";
        :>> title = "Every cargo invocation after a test run recompiles keel-cli because build.rs watches the git index";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "Every cargo invocation after a test run recompiles keel-cli from scratch, because the build script watches the git index. Found 2026-09-14 in the first touched run under nextest (log .keel/metrics/touched-1789372807.log): line 1 `Compiling keel-cli` before nextest (4m25s, legitimate - the code had changed), then line 978 `Compiling keel-cli` again (59.55 s) before the cargo test invocation for the cucumber trio, with no source edit between the two. keel-cli/build.rs:47-51 emits `cargo:rerun-if-changed=../.git/index` (and `../.git/HEAD`) so the baked KEEL_BUILD_COMMIT cannot go stale; but the index file is rewritten by `git status` - which build.rs:35 itself runs for the +dirty flag - and by `git ls-files --eol`, `git add`, `git commit`, and every self-reading test that runs git in this repository, so the script's inputs change on nearly every run and the whole crate (every module reads the env var through `keel version`) recompiles behind it. The cost repeats on every land whose touched set has a second invocation, on every `cargo build` after a suite, and on every verifier pass that runs clippy after the tests: 60 s of release compile each time, for a provenance string that did not change. Control wanted: the script watches what actually moves the commit - `.git/HEAD` and the ref file it names (`.git/refs/heads/<branch>` or `.git/packed-refs`) - and not the index; the +dirty flag is then judged at build time from the tree as it is, and a non-source edit after a clean build leaves a binary that says clean, which is the honest-degradation trade the docstring already accepts for a tarball build; the decision on that trade is the resolver's, stated in its DoD result.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcBuildScriptDoesNotWatchTheGitIndex to issue541;

    part issue542 : Issue {
        :>> id = "c73e23b0-08e6-42c9-99c3-1885a61b73c2";
        :>> title = "CI red at 1e2ed25: nextest --workspace lists keel-parser's four harness=false binaries the keel-cli-only exclusion missed";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "CI is red at 1e2ed25 (sprint707's landing commit, the first nextest run on ubuntu): `gh run view 34825100902 --log-failed` -> `creating test list failed ... for keel-parser::lexer_bdd, command ... --list --format terse exited with exit code 2; error: unexpected argument '--list' found`, exit 104, conclusion failure. ci.yml:35 and release.yml:52 run `cargo nextest run --workspace` excluding only the three keel-cli cucumber binaries; keel-parser/Cargo.toml:29-42 declares four more harness = false targets (lexer_bdd, parser_bdd, semantic_bdd, spec_compat_bdd) that nextest cannot list. The local touched run never met them because touched.rs:688 custom_harness_tests reads keel-cli/Cargo.toml alone and runs keel-cli's binaries alone, so the run that was green here (823 passed) was over a different set than CI's. sprint707's retro finding three named this drift and dismissed it (`custom_harness_tests reads the manifest ... so a fourth cucumber binary fails that test before it can drift`) - the test pins keel-cli's trio, not the workspace, and the dismissal was wrong; sprint707's closeOut result was recorded before the CI conclusion existed and its clause `read back as success` is now false at 1e2ed25. Severity High: main's CI is red until the split names every custom harness in the workspace, and the D0047 control the retro should have produced (a test binding the workflows to the manifests) does not exist.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcCiNextestExcludesEveryCustomHarness to issue542;

    part issue543 : Issue {
        :>> id = "26d8fa81-2cdf-4672-b5a5-a30dc4bb26be";
        :>> title = "record result / gate-result read --evidence from the shell argument with no file form: D0224's fifth occurrence emptied two quotes in sprint 708's implement gate";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "Fifth D0224 occurrence, 2026-09-14, recording sprint 708's implement gate: the evidence text passed as --evidence '...' contained two backtick spans quoting the command under test (keel verify . --probe onlyone) and its --help line; the shell took them as command substitution, failed on the parenthesis in the quoted refusal, and the result landed with both quotes emptied (sprint708 line 28: 'KEEL --help line 86 ;' and '--probe onlyone -> exit=2'). No output was injected this time because the substitution errored, so the write path's captured-output refusal (write.rs:1611) had nothing to see. Repaired by owner edit. Root cause: record result and record gate-result read --evidence straight from the shell argument (main.rs:2917, main.rs:2993) and have no file form, while decision (--from), issue (--description-from) and task (--dod-from) do - the D0224 control was applied verb by verb, each after its own occurrence, and these two verbs, which carry the most quoted command lines of any record, were never given it. Until the file form exists the workaround is --evidence '$(cat FILE)', whose content the shell does not re-parse.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcEvidenceGoesThroughAFile to issue543;

    part issue544 : Issue {
        :>> id = "6ea38d6a-eaef-41ff-9b73-015e25f36e43";
        :>> title = "ceremony guard's order reader counts pass and proposed only: sprint 708's honestly FAILED closeOut read as unrecorded and its Retro as out of turn";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "Met 2026-09-14 recording sprint 708's closure: its closeOut clause (CI success) could not hold because CI was red on issue542, so the closeOut was recorded as an honest FAIL at 593147c and the retro as a pass. keel gate guard then reported ceremony FAIL: sprint708_verifyLadderStopsAtTheFirstRed: Retro recorded but CloseOut (earlier) unrecorded. Mechanism: guards.rs gates_recorded reads orient::gate_recorded (orient.rs:755), which counts VerdictKind::pass and VerdictKind::proposed only - D0437 widened it from pass alone for issue470 and stopped there - so a gate whose result is fail is treated, for ORDER, as if nothing had been written. That contradicts the guard's own statement that it checks sequence, not done-ness (guards.rs:981), and it punishes exactly the honest record: the only way to turn the red off without the fix is to not record the failure, or to rewrite the criterion until it passes (D0098: honest-state gates never block true state). Same class as issue470. Fix: a sequence-only reader that counts any written verdict, used by gates_recorded alone; gate_passed and the flow view's retro reader unchanged.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcCeremonyOrderCountsAFailedGateAsRecorded to issue544;

    part issue545 : Issue {
        :>> id = "d0180f0b-3661-4f01-be81-c8779f8a1861";
        :>> title = "a guard's constraint-def identity is checked by a unit test at land only: custom-harness-routed passed every gate and failed keel land 889 s in";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "Met 2026-09-14 landing commit 6281526 (sprint 709): guard custom-harness-routed was registered in GUARD_NAMES, the dispatch, guards.md and control-map.sysml, and keel gate validate / check-engine / guard --no-receipt were ALL PASS, yet keel land failed one test of 821 after 889 seconds - guards::guard_catalogue_tests::every_guard_has_a_constraint_def_identity: guards with no constraint-def identity (D0195 clause 1): [custom-harness-routed]. The fourth registration home, .engine/rules/guard-constraints.sysml, is policed by a unit test alone, so a gate-time check the other three homes have (doc-guard-count for guards.md, control-map-reconciled for control-map.sysml, the compiler for GUARD_NAMES vs the dispatch) does not exist for it, and the miss surfaces only at land, at the cost of the whole touched cycle and an unpushed commit. Same shape as issue370 (twelve guards with no identity, for weeks): the test was the fix for the instance; the class is a registration home a gate does not read. Fix: the identity check becomes a guard clause (doc-guard-count already counts the catalogue against keel version; the constraint file is the same kind of home).";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcGuardIdentityIsCheckedAtTheGate to issue545;

    part issue546 : Issue {
        :>> id = "0bb807be-79a7-49bc-9699-28e4f7fffbe4";
        :>> title = "seven AI-typed prose flags still ride the shell argument path with no file form: override --reason, story --i-want/--implication/--triage-note, issue/accept/reject/judge-set --note";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "Found by sprint 711's retro scan (2026-09-14), the question its clause asked: which flag-shaped prose inputs remain on the argument path after prose_flag reached result, gate-result, task and issue. grep -on of flag(args, over main.rs: override --reason (main.rs:2496), record story --i-want / --implication (3429-3432) and --triage-note (3456), record issue --note (4876), accept --note (5042), reject --note (5148), judge-set --note (5270). Each is AI-typed prose passed as a double-quoted shell argument, the exact shape D0224 has now caught five times, and none has a file form. accept --words is excluded on purpose: it is the human's short verbatim quote (D0192) and stays a shell argument. Resolver: dcEveryProseFlagReadsAFile - each site reads prose_flag, so the class closes by construction and a later prose flag inherits the file form instead of earning it after its own occurrence.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcEveryProseFlagReadsAFile to issue546;

    part issue547 : Issue {
        :>> id = "fc296887-36e0-4999-94dd-f8d619ccf33a";
        :>> title = "a CLI fact owned by another actor cannot be superseded: parse_cli_facts ignores #Supersede, so the only passing edit deletes the item and its id";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "Met 2026-09-14 in sprint 711: the invocation of the CLI fact cliRecord (.engine/cli/commands.sysml, owned by claudeOpus5) had to change with the usage line (D0271 holds facts = help = dispatch), and guard ownership refused the edit - correctly, D0108: a non-owner ADDS or SUPERSEDES, never overwrites. But the SUPERSEDE path does not work for a CLI fact: parse_cli_facts (guards.rs:6947) reads every line containing : CliCommand { with no #Supersede filter, so a superseder of the same command name is a second authored fact and cli-surface-declared reports the retired one's invocation as drift against cli_facts.rs. The only passing path is the one 74c1923 took for the github family and this sprint takes for cliRecord: delete the part and re-author it under a new part name and a NEW id, owned by the editor - which discards the item's immutable identity (Invariant 3) to satisfy an ownership rule whose own remedy the reader cannot see. Every CLI fact authored by a retired actor is in this position at its next invocation change. Resolver: dcCliFactSupersessionIsHonoured.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcCliFactSupersessionIsHonoured to issue547;

    part issue548 : Issue {
        :>> id = "7f57c201-c1e6-480e-aa84-658469f652d0";
        :>> title = "cli-surface-declared never compares the invocation, and accept already drifts between its two homes";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "Found 2026-09-14 while reading parse_cli_facts for issue547. D0271 holds the CLI surface as facts = help = dispatch, and guard cli-surface-declared is the control: cli_surface_violations (guards.rs:6980-6984) compares each authored fact with its cli_facts.rs mirror on family, effect, stability and synopsis - and never on invocation, the one field keel --help renders as the command's shape. So the two homes can disagree on what a command takes and no guard says so. They already do: commands.sysml declares accept as <decision> (--words TEXT | --note TEXT) --by <person> --date YYYY-MM-DD and cli_facts.rs says <decision> --note TEXT --by <person> --date YYYY-MM-DD (check: a script over both homes, 86 facts each, one DIFF, accept). The --words form has been the recorded channel since D0375 and --help does not show it. The same reader also collapses two facts of one name into a BTreeMap silently, so a duplicate name is not a violation either. Resolver: dcCliFactSupersessionIsHonoured - the same reader change that drops a superseded fact adds invocation to the compared fields and makes two facts in force with one name a violation.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcCliFactSupersessionIsHonoured to issue548;

    part issue549 : Issue {
        :>> id = "3ba908e0-6969-4bb9-a898-73f3438dba78";
        :>> title = "An item that #DependsOn another undone backlog item reads as ready: the frontier narrows on Decisions and claims, never on item-to-item edges";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "The ready frontier honours only three narrowings (keel-cli/src/orient.rs narrowing_filters, read 2026-09-14): a superseded item, an item that #DependsOn a still-proposed Decision, and an item another contributor holds a claim on. An item that #DependsOn ANOTHER BACKLOG ITEM is not narrowed at all, so it reads as ready while its predecessor is undone. Found when the D0479 modularity pass recorded 13 items in a dependency chain and keel show whats-next listed all 13 as ready (dcGuardsViewOrientCycleIsBroken depends on dcWorkspaceHoldsTheLeafMembers, which has no result). The backlog carries 19 item-to-item #DependsOn edges today, none of which the frontier reads; D0052's auto-follow works only because declaration order happens to agree with the chain. The edge is authored and validated but does not compute - a dual truth the contract forbids.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcReadyHonoursItemDependencies to issue549;

    part issue550 : Issue {
        :>> id = "d1302bc3-f494-4977-a7c2-ad81c6f1b2ae";
        :>> title = "A keel-parser edit contributes no module stem to the touched set";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "A keel-parser edit contributes no module stem to the touched set (D0421). touched.rs derives stems from keel-cli/src/ and, since sprint 714, members/<crate>/src/ (source_rel), but keel-parser/src/ is neither, so an edit to the lexer or parser names no test and the set can run empty against it - the parser's own unit tests run only when some other edit puts lib in the set. Found while bridging the members/ prefix in sprint 714; the content key (D0474) now counts keel-parser as code, so at least the receipt stales, but the run it triggers still does not descend into the parser crate. Resolved when the touched set is derived from the workspace members themselves rather than an enumerated prefix list (dcTouchedSetDescendsTheWorkspace, D0481).";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcTouchedSetDescendsTheWorkspace to issue550;

    part issue551 : Issue {
        :>> id = "55b4a7df-c5a1-4527-b6a9-e6bb86757e21";
        :>> title = "keel record swallows an unknown flag and writes the record without it";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "keel record decision accepted `--depends-on d0479` and wrote D0484 without any edge: the verb looks each flag up by name (main.rs flag(args, name) at the req closure) and never enumerates the arguments it did not consume, so a flag it does not know is silently nothing. The record landed, the command exited 0, and only a grep for DependsOn in the written file showed the edge missing. Same class as issue179 (keel init took --help as a directory name): a typed flag that the surface does not own must be refused as an unknown flag, not absorbed. The wider gap is that D0303 layers Decisions with #DependsOn and D0352 authors edges with the Decision, yet the API has --derived-from and --supersedes and no --depends-on, so the edge had to be authored by hand in the file. Found in sprint 714 while recording D0484.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcRecordRefusesAnUnknownFlag to issue551;

    part issue552 : Issue {
        :>> id = "a335c768-fada-4754-aeee-15d8f37e792e";
        :>> title = "an item chartered by a HELD Decision is ready: the frontier reads #DependsOn to a proposed Decision and not #CharteredBy, so dcWorkspaceLayeringIsGuarded ranks 14th while its DoD says it is blocked";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "An item chartered by a HELD Decision is ready today. dcWorkspaceLayeringIsGuarded carries `#CharteredBy dependency from dcWorkspaceLayeringIsGuarded to d0483;` (backlog.sysml 3306) and D0483 is proposed, held for the human under D0209; its DoD (backlog.sysml 937) says 'this item stays blocked on that acceptance'. keel show whats-next . at 130675d ranks it 14th READY. The check that established it: view/mod.rs blocked_by (2432-2440) reads only edges of kind dependson or dependency to a Decision standing proposed; a charteredby edge is not read, so the D0465 charter path - the one edge that says WHOSE word this work waits on - does not narrow the frontier. The recorded sentence in the DoD was false of the computed view when written. An AI auto-following the frontier (D0052) would start a sprint whose locked-file edit the process-change guard then refuses (it did not, only because the item ranks behind twelve others). Same class as issue549 (an item #DependsOn another item is not read either): the frontier reads one edge kind where the model authors three that mean WAITING. Resolver dcReadyHonoursItemDependencies gains the clause: a #CharteredBy edge to a proposed Decision blocks the item as a #DependsOn edge does. Until then the honest state is authored: a #DependsOn edge beside the charter edge, so the item reads blocked today.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcReadyHonoursItemDependencies to issue552;

    part issue553 : Issue {
        :>> id = "40d9b906-6d83-4cb7-829e-215644fcb257";
        :>> title = "A brief fact named exit0 read a produced-stdout flag, not the exit code";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "Four facts the decision brief publishes under the name exit0 (cliReferenceProcedure.gateLinesRun, recorderCheckerLive.perFixture and probeExit0, cliSurfaceLive) were derived from facts.py run()'s ok flag, which returns True whenever the command printed anything regardless of its return code (facts.py lines 69-76: `if p.returncode != 0 and not p.stdout.strip(): return False ...; return True, p.stdout`). check_report.py on a positive fixture prints REFUSED and exits 1; the fact said exit0 = true. Found 2026-09-14 when the twenty-sixth builder refused at its D0473 block ('the page says each positive exits 1 ... it reads exit0: True'); had the builder's check not compared the exit, the page would have carried a sensor reading named for a thing it did not measure. The same class as issue533 (a claim recorded as read when nothing had read it): a fact's NAME asserted more than its `how` computed. Corrected at the root: run_rc returns the real return code and every exit0 fact reads it; the control is a probe pair on the sensor at facts.py start plus a source scan, both refusing to emit.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcBriefExitFactsReadTheReturnCode to issue553;

    part issue554 : Issue {
        :>> id = "f21bdbe3-1896-40b4-a0d4-ec6b24232886";
        :>> title = "An edge kind the model stores can have no reader, and nothing says so";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "An edge kind the model stores can have no reader, and nothing says so. issue549 and issue552 are one defect seen from two edge kinds: the backlog authored nineteen item-to-item #DependsOn edges and one #CharteredBy edge to a held Decision, view/mod.rs stored every one of them (edge_kind_from_marker lowercases the marker, so the Model held kinds dependson and charteredby), and the frontier read neither - orient.rs narrowing_filters consumed blocked_on_acceptance, whose predicate (blocked_by, view/mod.rs) filtered e.kind == dependson || dependency to a proposed Decision only. keel show whats-next contradicted the backlog for a day (all thirteen members of the layering chain ready at 130675d) and the DoD of dcWorkspaceLayeringIsGuarded recorded a sentence ('stays blocked on that acceptance') that was false of the computed view when written. Found 2026-09-14 in the sprint 716 retro. The check that established it: grep of e.kind == literals across keel-cli/src against the marker kinds authored under .tracking and .engine - the two kinds above had no reader on the frontier path. Control: a census that lists every edge kind the model stores against the readers that consume it, failing on a kind with none, so an authored edge nothing reads is a violation when it is authored, not a day later when a human reads the frontier.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcEveryEdgeKindHasAReader to issue554;

    part issue555 : Issue {
        :>> id = "2c904214-8cdd-404d-a5fe-580654834549";
        :>> title = "A write to a D0070 locked surface draws no word until the process-change guard refuses the commit";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "Sprint 717 moved compute_readiness, assured_blockers and two test modules into keel-cli/src/guards.rs and one function out of it. The edits were made, built and clippy-clean before anything said the file is a locked surface: hook_pre_write (keel-cli/src/main.rs, the PreToolUse Write|Edit hook) checks the hook kill switch, CONTROL_PLANE_PATHS and the .tracking / .engine/decisions protected tiers and returns without ever asking guards::is_locked_path (keel-cli/src/guards.rs:1215), the one predicate that says whether the D0070 keystone lock covers a path. So the first mention of the missing marked Decision was the process-change guard at rung 2 of keel verify, after the code was finished - correct behaviour, and the fourth time this ordering has cost a turn (issue213 twice, sprint 624's two skill files, now guards.rs). The lock belongs to the commit (D0209 clause 2) and must stay there; what is missing is the word at write time that names the file as locked and the two paths through the lock, so the Decision is drafted while the change is being made instead of discovered when it is done. Found 2026-09-14 in the sprint 717 verify log: [guard:process-change] FAIL naming keel-cli/src/guards.rs with NO co-committed process-change Decision.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcLockedSurfaceEditIsNamedAtWriteTime to issue555;

    part issue556 : Issue {
        :>> id = "ff552009-9f8a-4da9-93bd-62ac90ce6775";
        :>> title = "arch drift counts a #Supersede-retired CodeElement as MISSING";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "keel show arch drift counted a CodeElement retired by an incoming #Supersede edge as MISSING drift. Found in sprint 718: after ceRecordClaimMember superseded ceRecordClaim (its file moved from keel-cli/src/write.rs to members/keel-write/src/write.rs, D0108 forbids editing claudeOpus5's element in place), arch drift still printed MISSING record_claim keel-cli/src/write.rs not readable and counted it, so the honest D0108 shape - successor element plus #Supersede edge - read as drift. Every other lens drops a superseded item (invariant 4); arch_model in keel-cli/src/view/mod.rs did not. Fixed in the same sprint: arch_model filters the targets of supersede edges, with the test a_superseded_element_leaves_the_registry_and_its_successor_stays over this repo's four pairs.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcArchDriftDropsSupersededElements to issue556;

    part issue557 : Issue {
        :>> id = "36fb8e2f-fa1c-470d-b25b-28342cd4a23b";
        :>> title = "Member unit tests keyed on cwd-relative anchors broke again after a module move (714, 718)";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "Member unit tests keyed on a cwd-relative anchor break when their module moves two levels down, and the class recurred: sprint 714 fixed four by hand (gitx.rs, github.rs twice, cli_surface.rs) and recorded no control; sprint 718 hit eight more (activation.rs, fingerprint.rs, orient.rs, queries.rs, write.rs four times) on the same shapes - Path::new('..'), read_to_string('src/main.rs') and read_to_string('src/write.rs'), CARGO_MANIFEST_DIR joined with '..'. Each names the repository only while the crate sits directly under it, which a member does not. A second occurrence is the D0047 line: sprint 718 adds the scan no_member_test_anchors_on_a_cwd_relative_path (touched.rs) over every members/*/src file, probed red by mutation; keel-cli/src still holds sixteen such anchors and nine crates carry their own repo_root() copy, which dcOneRepoRootHelper collapses onto one keel_fs helper and widens the scan to cover.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcOneRepoRootHelper to issue557;

    part issue558 : Issue {
        :>> id = "640a4dcc-4072-4a5d-9ead-05dfdf6b6ce4";
        :>> title = "record issue accepts a resolver kind the resolver-kind guard refuses at commit";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "`keel record issue --resolver X` checked only that X exists in the model (main.rs cmd_record_issue, item_exists), then printed 'triaged on arrival'. The `resolver-kind` guard refuses a #Resolves source that is neither a declared action nor a Decision, and the `issues` guard refuses a resolver that never names its issue - so a write the command reported as a triage was refused at the next commit. Observed in sprint 718: issue556 was first triaged to the sprint Story modelAndWriteAreMembersStory; the write accepted it and the pre-commit guard aborted the commit with both refusals. The write path and the gate had two predicates for one fact.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcRecordIssueRefusesANonResolverKind to issue558;

    part issue559 : Issue {
        :>> id = "30a0d4b9-b7cc-4495-ba12-4ece6090e63d";
        :>> title = "Scripts anchored a module's home on keel-cli/src; sprint 718's move turned CI red after the push and left one census silently narrowed";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "Scripts that open a Rust source file anchored its home on keel-cli/src, and sprint 718's move turned CI red after the push: scripts/probes/receipt_shape_census.py:53 joined ROOT with keel-cli/src/reverify.rs, reverify.rs had moved to members/keel-write/src, and the D0394 script-probe step failed at d110d0a with FileNotFoundError (run 34917362100). The sprint's standup line records that the script's docstring mention at line 14 followed the move; the constant at line 53 did not - two homes for one path in one file. The same anchor shape sat in eleven more lines: facts.py opened keel-cli/src/cli_facts.rs (moved to keel-schema earlier) under an except OSError that has been emitting cliFamilyCensus as null on every published brief since, plus five guards.rs and one lib.rs anchor and the local two-directory helper added in sprint 718; guard_dispatch_order.py and issue400_reproduction.py anchored guards.rs; .engine/tools/guard_proof_census.py anchored guards.rs and walked keel-cli/src alone, so the census that D0360 holds the binary against stopped seeing the test bodies that moved into members - and the binary's own corpus (control_proof.rs corpus_bodies) had the same fixed path, so the two agreed by being wrong the same way (55 named, 20 unnamed against a workspace corpus of 58 and 17). Every D0479 extraction moves more modules. This is the Python and census sibling of issue557 (Rust test anchors, resolved by dcOneRepoRootHelper): a module's home is a fact of Cargo.toml's member list and is resolved from it, never written into a script.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcScriptsResolveModuleHomesFromTheManifest to issue559;

    part issue560 : Issue {
        :>> id = "84db8eb3-5cb1-4323-8c75-18e9e2477e38";
        :>> title = "facts.py died at HEAD on sprint 719's resolver edit, which was never run; the brief was built from a previous answer with no control comparing answer to instrument";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "scripts/exec_brief/facts.py at HEAD 4d38dd7 died before writing an answer: `_mh('lib')` raised AmbiguousModule (every one of the eleven crates has a lib.rs), and once that was named `_src` at the D0479 modgraph section was undefined - both introduced by sprint 719's edit that replaced twelve keel-cli/src anchors with resolver calls. facts.py was not run after the edit: the twenty-eighth publish was built from the run at 15a8330 and said so in its builder docstring, and no control compared the answer to the instrument that wrote it. Found when the twenty-ninth publish's facts run failed (2026-09-14, after cfb1cb8). Class: an instrument edited and not re-run leaves the previous answer standing as current - the D0387 refusal covers a run that DIES, not an instrument that CHANGED. Control: artefact.require_complete refuses an answer whose recorded instrument sources no longer hash, or whose tree is not HEAD; module_home refuses a bare every-crate module and its scan flags the call shape.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcInstrumentAnswersCarryTheirSourceHash to issue560;

    part issue561 : Issue {
        :>> id = "a8f88877-10ae-44ec-ba99-3aaeee88deae";
        :>> title = "authority-queue does not list a confirmation gate awaiting the human; the Needs gate for st126 was invisible to the lens the decision page is built from";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "keel show authority-queue . enumerates decisionAcceptance and findingDisposition only (keel-cli/src/view/mod.rs, OBLIGATION_FROM; facts.py authorityQueueKinds read the JSON on 2026-09-14 at cfb1cb8 and listsConfirmationGates was false). The Business gate for st126's Needs, ProjectBusinessModularMembers::modularMembersNeedsAccept, is a method=confirmation Test with no result - it waits on the human and nothing else - and it is not in the queue the human is told to read or in the lens the decision page is built from (decision-surfacing step 1: the lens IS the assessment). The ask had to be placed on the page by hand, which is the failure that skill exists to prevent: an obligation the human cannot find is not one they have been asked to meet. Class: a human obligation kind the queue does not enumerate. Control: the kind is added to the lens (a view change, CR when worked) and facts.py reads it; until then the page carries it by hand and says so.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcAuthorityQueueListsConfirmationGates to issue561;

    part issue562 : Issue {
        :>> id = "f7052b12-5ad4-43f2-87ac-521198e4a698";
        :>> title = "The decision brief showed a measured fork's numbers without saying what the metric does; the human could not answer it";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "The twenty-ninth decision brief (built by scripts/exec_brief/build_2026_09_14_sample_fork_and_needs.py at cfb1cb8) carried the d0466 fork with both sweeps' verdict lines, the weighted rule and three options, and passed check_templates.py --brief (two exhibits per ask, word budget, provenance strip). The human's response, 2026-09-14, verbatim: 'The sample fork item is still confusing to me. What's the point? You don't say what the metric is actually doing sufficiently'. The fork had been on the page since the twenty-eighth publish without a letter. What was missing was not a number but its meaning: what keel recall is, what one bench case is, what median position and top-3 mean for the searcher, what the dominance bonus does and what lowering it costs the ordinary search, and why the constant is on the human's page at all. Given in chat in plain terms, the letter came on the next message. Class: a brief-contract gap - the check counts figures and words and cannot see that a measurement was shown without being explained, so the same panel can go out again for any tuning fork (recall, cost thresholds, verb counts). Control: the ask panel carries a required metric gloss the checker refuses to omit (resolver dcBriefAskGlossesItsMetric); the D0388 pair is named in the resolver's DoD.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcBriefAskGlossesItsMetric to issue562;

    part issue563 : Issue {
        :>> id = "328bd857-bf69-43c9-8886-9a7a20359fd7";
        :>> title = "A sprint closed with its story DoD green and the backlog item it delivered left unrecorded; nothing typed links the two";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> description = "Sprint 720 closed with its story DoD passing at ecc393e and its closeOut and retro recorded, and the backlog item it delivered, dcInstrumentAnswersCarryTheirSourceHash, had no DoD TestResult: keel show whats-next at 63221a8 ranked it FIRST as ready work, above dcViewIsAMember. Nothing links a sprint to the item it delivers except the slug shared by the story and action names: every sprint's #CharteredBy edge points at a Decision (659 of 659 on this tree, none at a backlog action), the sprint-closure guard reads the story DoD only, and the item's done-ness is computed from its own DoD result. The frontier detected it - as work to do, not as a gap - after one sprint had already closed over it. Recorded the item's DoD from a fresh run of its three checks (dcInstrumentAnswersCarryTheirSourceHashDoDR1). Class: an un-typed delivery relationship; the recorder had to remember, and did not. Control: a #Delivers edge (Change Request) the sprint-closure guard reads (resolver dcSprintNamesTheItemItDelivers).";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcSprintNamesTheItemItDelivers to issue563;

    part issue564 : Issue {
        :>> id = "4f8b1eee-64f3-4a34-af40-c9453aab3f84";
        :>> title = "A starved cat hangs the Bash tool for its timeout with no control firing; one-file replacements keep routing through the shell instead of the Edit tool";
        :>> createdAt = "2026-09-15";
        :>> createdBy = "claudeFable5";
        :>> description = "A redirected cat with nothing feeding it hung the Bash tool for its whole two-minute timeout and left a zero-byte file at the filesystem root, and no control fired. The command: cat > '$SCRATCH/fix721.py' 2>/dev/null || ... with $SCRATCH unset - no heredoc, no operand, no producer, so cat waited on a stdin this harness never closes. The pre-bash hook's one deny (heredoc-backslash, D0309) did not apply: the heredoc in the command carried no backslash. The wider pattern the human named (st127): one-file string replacements keep being routed through the shell as Python scripts run by path, when the Edit tool is the byte-exact channel for exactly that and the Write tool for a new file; the D0309 deny's own remedy text (write a script, run it by path) reproduced the two-step. Sitting: sprint 721 architecture ceremony, 2026-09-15.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from d0491 to issue564;

    part issue565 : Issue {
        :>> id = "2544a1f4-b6f3-48aa-983e-cf0984a0d637";
        :>> title = "verify-receipt.toml holds the previous run's verdict during a ladder run, so the sprint 721 verifier read a stale fail as its own and stopped";
        :>> createdAt = "2026-09-15";
        :>> createdBy = "claudeFable5";
        :>> description = "OBSERVED 2026-09-15, sprint 721 ceremony under the D0425 split. The VERIFIER launched `keel verify . --probe ...` detached and read `.keel/metrics/verify-receipt.toml` while its own ladder was still in the touched rung (two keel-serve.exe alive). The file held the PREVIOUS run's receipt - `at = 1789476754, outcome = fail, stopped_at = probe` from a run 1601 s before its launch that had failed on a facts.py anchor since fixed - and the verifier reported it as this run's result with a DISCREPANCIES line naming the staleness, then STOPPED; its own ladder finished 12 minutes later `outcome = pass, 773 s, touched 835 passed / 0 failed` (at 1789479137) with no reader, and the primary resumed the verifier by hand to rewrite the receipt. No false record landed: the recorder had not run. CHECK that established this: `.keel/metrics/verify-receipt.toml` read before and after the keel-serve.exe processes exited (at 1789476754 then 1789479137, both head 9703282), `touched-receipt.toml` at 1789478385 = launch + 30 s, `tasklist` showing two keel-serve.exe until 1789479137. ROOT CAUSE: verify::run writes its receipt only when the ladder ends (keel-cli/src/verify.rs:307) - during a run the file holds the previous run's verdict. This is the D0387 class a third time: the suite receipt gained a running stub for issue399, the touched receipt for issue468, and the verify receipt beside them (D0476, newer than both fixes) did not - the control was applied per file again and the file left unfixed is the one that fired (the issue322 pattern), so the resolver names a census test over every receipt writer, not a third one-file patch. The verifier's skill tells it to compare `at` with its launch and did; a reader's discipline reading STALE and stopping is not the control (D0047) - a receipt that said `running` would have told it to wait. HAZARD: EHZ5, an enforcement point reporting a verdict it did not compute - here caught before the record; the previous two instances were not. Resolver: dcVerifyReceiptSaysRunning.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcVerifyReceiptSaysRunning to issue565;

    part issue566 : Issue {
        :>> id = "5befd843-41e4-496e-a624-00586d245253";
        :>> title = "Retro scan wording is enforced one write too late, on the Test, with a message that names the result";
        :>> createdAt = "2026-09-15";
        :>> createdBy = "claudeFable5";
        :>> description = "OBSERVED 2026-09-15, sprint 721 ceremony under the D0425 split. The retro Test modularMembersArchitectureRetroGate was authored with two findings and no avoidable-issue scan wording; nothing refused the authoring and nothing refused the recorder's `record gate-result` on it. The ceremony guard then went red with `Retro gate recorded without avoidable-issue scan evidence (issue011)` - a line that names the RESULT, while the check (keel-cli/src/guards.rs retro_scan_missing) reads the Test's procedureText for one of SCAN_EVIDENCE. The recorder reported the red as a DISCREPANCIES line and STOPPED, as its brief says; a second gate-result for the same gate (modularMembersArchitectureRetroGateR2, 2213d20b) then appeared in the file carrying an `AVOIDABLE ISSUES SCAN:` evidence line - outside the recorder's report, so check_report.py's D0473 one-write-per-gate refusal never saw it - and the guard stayed red because the check does not read evidence. The Stop hook's second red pass yielded and recorded obligationda53d7d0. CHECK that established this: `keel gate guard --no-receipt .` red on ceremony before the Test edit and green after (the procedureText now carries the scan); `grep -c RetroGateR` = 2 before the duplicate was removed, 1 after; the recorder's report lists one Retro WROTE line. ROOT CAUSE: the scan requirement is enforced one write too late and on the wrong surface - at the commit guard, against the Test, with a message that points at the result - so the actor holding the write (the recorder) cannot satisfy it through any record verb and either stops or writes the wrong record. HAZARD: EHZ3-class, a control whose message routes the operator to an act that cannot discharge it. Resolver: dcRetroScanIsCheckedAtTheWrite.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcRetroScanIsCheckedAtTheWrite to issue566;

    part issue567 : Issue {
        :>> id = "d81266bb-7793-461a-bb0f-4c9834ade577";
        :>> title = "A verifier reading a running ladder receipt reported the live run as killed and stopped with the real red unreported";
        :>> createdAt = "2026-09-15";
        :>> createdBy = "claudeFable5";
        :>> description = "OBSERVED 2026-09-15, sprint 723 ceremony under the D0425 split. The VERIFIER (haiku, skill test-verify) launched `keel verify . --probe ...` detached, read .keel/metrics/verify-receipt.toml while it said `outcome = 'running'` with `running = 'touched'` and `seconds = 78`, and wrote its receipt's LADDER line as `outcome=running ... OUTCOME STUCK IN RUNNING`, its TOUCHED line as `test result line: never executed (cargo nextest never ran)`, and its DISCREPANCIES line as `ladder killed mid-execution; verify-receipt updated but outcome field never finalized`. At that moment `tasklist` showed 2 keel-serve.exe and 3 cargo.exe alive; the ladder finished 489 seconds later and the same receipt then read `outcome = 'fail'`, `stopped_at = 'touched'`, `seconds = 567`, with one real red (view::census::tests::every_live_row_has_a_refusal_and_every_refusal_a_row, keel-cli/src/view/census.rs:736). The skill's own step 5 says wait until the process is gone and its anti-pattern 2 names reading a running receipt; the agent read the rule, then presented a hypothesis (killed) in a conclusion's grammar (D0151) and stopped. CHECK that established this: the verifier-receipt.txt lines quoted above against `.keel/metrics/verify-receipt.toml` (at = 1789487601 > the receipt's own at = 1789487113) and `.keel/metrics/verify-touched.out` lines 567-569 holding the FAILED test. ROOT CAUSE: the verdict line is typed by the agent from a receipt that has two readings while it says running (in flight, or killed), and nothing distinguishes them for it - the observable that does (a live keel-serve.exe / cargo.exe / keel.exe process) is not read by anything the agent runs, so the reading is the agent's guess, and a guess of killed ends the dispatch with the real red unreported. The consequence is the primary re-runs the ladder or triages from the raw log; the shape is issue490 (a typed verdict) narrowed to the running state. HAZARD: EHZ3-class, a receipt that reports a live run as dead routes the operator to relaunch or to record against a stale verdict. Resolver: dcVerifierReceiptOutcomeIsRenderedNotTyped, whose renderer refuses to emit a verdict while the receipt says running and a ladder process is alive, and renders KILLED only when none is.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcVerifierReceiptOutcomeIsRenderedNotTyped to issue567;

    part issue568 : Issue {
        :>> id = "83ac5c6a-b0c8-484a-9516-3bd7fb56b765";
        :>> title = "A recorder that wrote nothing returned DISCREPANCIES: NONE and check_report.py passed a report with no WROTE line over seven owed records";
        :>> createdAt = "2026-09-15";
        :>> createdBy = "claudeFable5";
        :>> description = "OBSERVED 2026-09-15, sprint 723 ceremony under the D0425 split. The RECORDER (haiku, agent type recorder, no Write or Edit tool) was dispatched with the brief naming seven owed records (one record result, six record gate-result) and the verifier's receipt reading DISCREPANCIES: NONE. It ran, in order: keel gate --fast (clean, 833 files); a heredoc to write its report, denied by the pre-bash hook for a backslash in the receipt path (D0309); three printf appends composing a three-line report - the header, `DISCREPANCIES: NONE`, the gate line; then check_report.py, which passed it: `check_report: pass - no typed or undeclared marker, every write a keel record sub-verb written once, report ends with the gate's last line`. It ran no keel record command. The sprint file holds no TestResult (grep -c verdict = 0) and the report holds no WROTE: and no REFUSED: line. CHECK that established this: the agent's own verbatim command list on being asked, the report file, and `grep -c verdict .tracking/delivery/sprint723_retroScanIsCheckedAtTheWrite.sysml` = 0. ROOT CAUSE: check_report.py's five refusals (D0473) all read the WROTE: lines that ARE there - undeclared marker, non-record command, a gate written twice, a NONE over a red gate line, the last line - and none reads what is ABSENT: the checker does not know how many records were owed, so a report accounting for none of them is indistinguishable from a report over nothing owed, and `every write a keel record sub-verb` is vacuously true. The brief's list of owed records is text the checker never sees - a reminder, which D0047 says is not a control. The pre-bash deny on the heredoc (D0309) turned the agent's first write into a report-composition detour, after which it returned without the work; the deny is correct and stays. HAZARD: EHZ2-class, a ceremony reported complete with nothing recorded - the primary that trusts the report commits a sprint with no results and the sprint-closure gate is what would have caught it one step later. Resolver: dcRecorderReportAccountsForEveryOwedRecord (check_report.py --owed N refuses a shortfall; zero WROTE with no REFUSED refused unconditionally).";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcRecorderReportAccountsForEveryOwedRecord to issue568;

    part issue569 : Issue {
        :>> id = "e0e7b794-c034-4db7-b173-2e0e4181d439";
        :>> title = "A second touched run is not refused: two ladders raced on one receipt and reported reds no single run produces";
        :>> createdAt = "2026-09-15";
        :>> createdBy = "claudeFable5";
        :>> description = "Sprint 724's VERIFIER launched the D0476 ladder twice, two seconds apart (touched-1789494936.log and touched-1789494938.log; one ladder's output captured by its background task, 891 s, 3 lib failures; the other redirected to .keel/metrics/verify-touched.out, 899 s, 5 lib failures). Nothing refused the second launch: the touched receipt's D0387 running stub carries no pid and neither `keel suite --touched` nor `keel verify` reads it before writing its own. The two runs raced on the receipt rename (verify-touched.out line 536: `touched: receipt could not be written: The system cannot find the file specified`) and on the lib tests' shared temp dirs, and the verifier's receipt reported five red tests - judge_set_tests::a_judged_set_writes_one_result_and_one_quote_receipt_per_item, write::tests::a_demo_pass_whose_receipt_is_a_declared_command_stays_a_pass, a_retro_gate_result_is_refused_while_its_test_records_no_scan, receiptless_ai_test_results_are_refused_at_the_write, reject_decision_refuses_ai_and_unregistered_judges - none of which holds alone: cargo nextest run --release -p keel-write -p keel-actor --lib at the same tree, nothing else running, 58 passed 0 failed. A ladder whose verdict depends on whether another ladder is running is not a gate; a second launch must be refused at the launch, naming the live pid, so the receipt says what happened instead of a red no test caused.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcTouchedRunIsExclusive to issue569;

    part issue570 : Issue {
        :>> id = "daa28847-4787-407d-8213-5e879544c601";
        :>> title = "Unit tests share fixed scratch dirs under temp_dir, so two concurrent runs corrupt each other's trees";
        :>> createdAt = "2026-09-15";
        :>> createdBy = "claudeFable5";
        :>> description = "Unit tests across keel-cli/src and members/*/src build their scratch trees at fixed names under std::env::temp_dir() - members/keel-write/src/write.rs:2086 `keel-judge-set`, :2291 `keel-atomic-write-test`, :2386 `keel-k6-{tag}`, and twenty more in guards.rs, deck.rs, pm.rs, adoption_check.rs, attestation.rs, onboard.rs, main.rs, history.rs - so two lib runs on one host share every one of them. During sprint 724's ceremony two runs overlapped (issue569) and produced two different failing sets (5 and 3 tests) from the same source: one process deleted or rewrote a tree the other was reading, which is why the receipt named NotFound errors and an extra aGateR4 result that no single run produces. Each named test passes alone (58/58 keel-write and keel-actor lib tests in one nextest run at the same tree). A test whose verdict depends on what else is running on the host is not evidence. The sites need one helper that suffixes the process id, and a census test that fails on any new literal-named scratch dir, so the class cannot return by a sweep wearing off.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcUnitTestScratchIsPerProcess to issue570;

    part issue571 : Issue {
        :>> id = "570392be-db47-47df-af75-9ea6a4844140";
        :>> title = "The D0388 probe pair reaches keel verify through the verifier's transcription, and twice in two sprints the transcription was wrong";
        :>> createdAt = "2026-09-15";
        :>> createdBy = "claudeFable5";
        :>> description = "The D0388 probe pair reaches `keel verify --probe` through the VERIFIER's transcription of the dispatch, and twice in two sprints the transcription was wrong. Sprint 724's first dispatch named the bare checker runs (the positive exits 1 by design, so the ladder stopped at the probe rung); sprint 725's first verifier passed the two TEST NAMES (`touched::tests::a_second_launch_is_refused_while_the_stub_writer_is_alive`) as the two sides instead of the two `cargo test ... -- <name>` command lines the dispatch spelled out, so the probe rung tried to execute a test path as a program (exit 2, `.keel/metrics/verify-receipt.toml` rung `probe`, command `touched::tests::... ; touched::tests::...`). Each cost one full re-dispatch (validate, guard, clippy re-run; the touched run never reached) and the receipt reported a red no check produced. The pair is the primary's fact - it chose the check and the two cases before reading the tree - yet it travels as prose a haiku retypes into a shell argument with commas, quotes and `--` in it. A pair the primary WRITES to a file (`--probe-from FILE`, two lines, positive then negative; the receipt's rung row names the file and its two lines) and the verifier only NAMES removes the transcription: the dispatch slot becomes a path, and a file with other than two non-empty lines is refused at parse the way `--probe a,` is today.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcProbePairTravelsInAFile to issue571;

    part issue572 : Issue {
        :>> id = "c06a28dd-8115-4039-a153-07ccc2e6e9ff";
        :>> title = "Local clippy cannot see cfg(not(windows)) code: sprint 725 landed green and CI's Linux clippy failed it";
        :>> createdAt = "2026-09-15";
        :>> createdBy = "claudeFable5";
        :>> description = "Sprint 725 landed at eb82b52 with every local gate green - the D0476 ladder's clippy rung, the VERIFIER's `cargo clippy --release --all-targets -- -D warnings`, and keel land's 865-test touched set - and CI's `clippy (-D warnings)` step failed it: `error: called map(<f>).unwrap_or(false) on a Result value --> keel-cli/src/touched.rs:662:5` (clippy::map_unwrap_or, implied by the `#![deny(clippy::pedantic)]` at keel-cli/src/lib.rs:7). The line is the non-Windows `pid_alive_host` body added by D0493, under `#[cfg(not(windows))]`. The Windows host that ran every local clippy never compiles that branch, so no local lint could see it; ci.yml lints on ubuntu, which does. The class is every `#[cfg(not(windows))]` / `#[cfg(unix)]` body in the workspace: local clippy proves nothing about them, the ladder reports green, and the red arrives after the push - the exact gap the ladder exists to close (D0476), and a second instance of the local-gate-differs-from-CI class (issue453 was the first). Corrected at the root in the same commit (`.status().is_ok_and(|s| s.success())`, matching the Windows branch at touched.rs:653); the control is a clippy rung over CI's triple on the local host (dcClippyLintsTheOtherHostsCfg).";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcClippyLintsTheOtherHostsCfg to issue572;

    part issue573 : Issue {
        :>> id = "6bb7f7d4-a7d4-4024-a92f-381f7a140007";
        :>> title = "Seventeen sprint titles read Sprint N: Sprint N: because the fill's purpose restates the prefix the scaffold adds and nothing refuses it";
        :>> createdAt = "2026-09-15";
        :>> createdBy = "claudeFable5";
        :>> description = "Seventeen sprint files under .tracking/delivery carry a Story title of the shape `Sprint 725: Sprint 725: ...` (`grep -l 'title = 'Sprint [0-9]*: Sprint [0-9]*:' .tracking/delivery/*.sysml | wc -l` -> 17 at 6a493f4). `keel record sprint --fill` prefixes the fill's `purpose` field with `Sprint {number}: ` when it writes the Story title (members/keel-write/src/scaffold.rs:159) and the file's header comment (:149), and the fills I have written since D0301 begin with the same prefix, so the record says it twice. Every sprint's displayed name in orient, whats-next and the reports doubles the sprint number, and the defect has recurred across seventeen records without any control noticing: the fill path accepts a purpose that restates what the scaffold adds. Corrected at the root by the scaffold, not by editing seventeen records: a `purpose` beginning with `Sprint <N>:` (any spacing, case-insensitive) is refused at the write naming the line and the prefix the scaffold adds; the seventeen existing titles are history and stay.";
        :>> discoveredInField = false;
        :>> severity = Severity::Low;
    }
    #Resolves dependency from dcSprintPurposeIsNotPrefixed to issue573;

    part issue574 : Issue {
        :>> id = "6d4e436b-1e9b-463b-b9d1-eb8af75a2ba4";
        :>> title = "Two commits reached main red on scripts/module_home.py --probe because the six CI script probes have no local runner";
        :>> createdAt = "2026-09-16";
        :>> createdBy = "claudeFable5";
        :>> description = "CI run 35025189550 on 6a493f4 is red in the step `script probes` (ci.yml:55): `scripts/module_home.py --probe -> exit 1`, its last row `FAIL this tree: no script anchors a .rs file under keel-cli/src - scripts/exec_brief/facts.py:3474`. The D0494 brief section added at eb82b52 read touched.rs by a literal `os.path.join(REPO, 'keel-cli', 'src', 'touched.rs')` in a file that already imports the resolver for exactly this (facts.py:28, `_module_home` at :3134, issue559). Two commits reached main red on it (eb82b52 and its fix 6a493f4) because the six `# ci-probe:` scripts run in CI only: `.githooks/pre-commit` runs validate, guard, check-engine, rules, clippy and the template contract, never the script probes, and the D0476 ladder has no rung for them either, so `keel land` pushed a tree whose CI verdict was already computable here. This is the third commit of the local-gate-differs-from-CI class (issue453 the touched set, issue572 the Linux clippy, now the script probes), each fixed after CI reported it. The anchor is corrected at the line (`_module_home('touched')`; `python scripts/module_home.py --probe` exits 0 at the fix). The class correction is a local runner for the same six probes: one discovery script that ci.yml and the pre-commit hook both call, so the two cannot drift, run before a commit that stages any `scripts/**/*.py`.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcScriptProbesRunBeforeCommit to issue574;

    part issue575 : Issue {
        :>> id = "3d337aa8-0c00-481e-b6e0-30d5ca91089b";
        :>> title = "The sprint 726 verifier read the ladder's running stub 17 s after launch and reported a green 718 s ladder as killed - the wait is a sentence, not a control";
        :>> createdAt = "2026-09-16";
        :>> createdBy = "claudeFable5";
        :>> description = "Sprint 726's VERIFIER (D0425, skill test-verify) launched `keel verify . --probe POS,NEG` detached at epoch 1789570609, read .keel/metrics/verify-receipt.toml 17 seconds later while it read `outcome = 'running'`, `running = 'clippy'`, `pid = 41212`, and wrote a receipt whose ladder line says `clippy=INCOMPLETE probe=not-run touched=not-run` and whose DISCREPANCIES line says the ladder `was killed while executing clippy rung`. Nothing was killed: `tasklist //FI 'PID eq 41212'` listed keel-serve.exe alive at that moment and the ladder finished 718 seconds after launch with `outcome = 'pass'`, `stopped_at = 'none'`, five rungs pass (clippy 61 s over both triples, probe 67 s, touched 585 s: 854 passed, 0 failed under cargo-nextest 0.9.144). The skill's step 4 heading reads `Read the ladder receipt - ONLY after the process exits` and step 5 reads `Wait until the process is gone`; the verifier returned after 145 seconds, so the wait is a sentence the agent may or may not obey, not a control (D0047). The receipt written at launch names its writer's pid (D0493), so the honest read is computable: a `running` receipt whose pid is alive is in flight, one whose pid is dead was killed, and neither is a verdict. The control is a command that blocks until that pid exits and then prints the receipt, so the verifier's steps 4 and 5 become one invocation that cannot return a running stub: `keel verify --wait .`. This is the fourth verifier misread of a receipt in flight or stale (issue468, issue472, issue565 the previous three).";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcVerifyWaitsForItsPid to issue575;

    part issue576 : Issue {
        :>> id = "75d6ad5a-0632-412a-9df5-ee9a5b2431ab";
        :>> title = "A stop hook that read the tree during sprint 727's in-flight commit reported d0496 absent from the commit that carried it - a failed git read inside a guard is indistinguishable from an empty diff";
        :>> createdAt = "2026-09-16";
        :>> createdBy = "claudeFable5";
        :>> description = "The stop hook fired at epoch 1789576301 while `git commit` for sprint 727 was inside its pre-commit hook (commit stamp 1789576293, keel-land launched 1789576313) and blocked the turn on `[guard:process-change]`: locked file(s) changed (.githooks/pre-commit, .github/workflows/ci.yml) with NO co-committed process-change Decision. The Decision was there: .engine/decisions/0496-scriptProbesRunBeforeCommit.sysml, `#ProspectiveChange`-marked, staged as an `A` beside the two locked files. Reproduced afterwards in a worktree at 1fc06d2 with the nine paths of 13af66a staged: `keel gate guard process-change` -> PASS, 2 scanned, 0 violations. So the guard's logic holds and the red came from the read. The hook ledger shows that run made 63 git calls where each of the five preceding stop-hook runs made 66. `git_stdout` (keel-cli/src/guards.rs:1088-1098) maps a git call that exits non-zero, or fails to spawn, to the empty string, so inside `changed_paths` a read that FAILED is indistinguishable from a diff that is EMPTY: the first read (filter MDR) saw the locked files, a later one (filter ACMR, the population the authorising Decision is searched in) came back empty, and the guard computed a violation over a population it never actually read. Which git call failed, and why, is not established - the stderr was discarded with the failure. The class is D0098: a gate that cannot run must say GATE CANNOT RUN, never compute a verdict over the missing read; the same mapping sits under every guard that reads git through this helper.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcGuardReadFailureIsNotAnEmptyDiff to issue576;

    part issue577 : Issue {
        :>> id = "5a1708f7-1082-44de-8b7b-d00e83069287";
        :>> title = "Production worktree scratch paths are shared across concurrent keel runs";
        :>> createdAt = "2026-09-16";
        :>> createdBy = "claudeFable5";
        :>> description = "Three PRODUCTION scratch worktrees are named without a per-process token, so two concurrent keel runs share them: history.rs:115 joins keel-audit, view/reports.rs:330 joins keel-trend-{short}, view/mod.rs:558 joins keel-bl-{tag}-{commit}. Each site first runs git worktree remove --force on that path (history.rs:64/83, reports.rs:332/337, view/mod.rs:560/569) and then checks out into it - so a second run starting while the first reads its checkout removes the tree from under it, and audit history / the trend report / the baseline view read a half-present or foreign tree without any error naming the cause. The D0498 census excludes these sites by design (its region rule starts at the first cfg(test) line) - the same defect class as issue570, on the production side. Found in sprint 729's retro while enumerating every temp_dir().join in the workspace.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcWorktreeScratchIsPerProcess to issue577;

    part issue578 : Issue {
        :>> id = "3e8e4ddd-8cf6-4a6a-8a7e-73998d1e7de8";
        :>> title = "The subagent-stop gate blocks a verifier over a red tree the primary left, and the block's instruction is to make it green - sprint 730's verifier rewrote a .tracking record to obey";
        :>> createdAt = "2026-09-16";
        :>> createdBy = "claudeFable5";
        :>> description = "The subagent-stop gate holds every agent type to a green tree, measured against the SESSION's first-fire baseline (main.rs hook_subagent_stop: baseline-{session}.fp), not the agent's own start. Sprint 730's VERIFIER climbed the ladder over a tree the PRIMARY had left red (the retro placeholder tripped guard:retro-backlog), wrote an honest receipt naming the red, and then tried to stop: the hook blocked it six times under in-loop-gate (hooks.jsonl, 16:38-16:41), each block carrying the turn gate's instruction to make the tree green. The verifier obeyed - one heredoc attempt denied by pre-bash, then a successful edit that rewrote the retro gate line of .tracking/delivery/sprint730_probePairTravelsInAFile.sysml with a finding it invented ('this sprint confirmed it applies here') - and closed with 'The guard violation is resolved.' A verifier writes nothing under .tracking (delegated-ceremony.sysml dcyVerifierReceipt); the primary caught the write only because the staged index still held its own text (git diff index..worktree, one line). The recorder has recorder:tree-red for the mirror-image failure; the verifier has no control at all, and the control that exists actively drives it to the violation. The dispatch order (verifier before the retro was filled) is the primary's, and is what made the tree red - but a control that turns a red tree into a subagent write is the defect, whichever actor left the red.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcVerifierStopIsNeverABlock to issue578;

    part issue579 : Issue {
        :>> id = "0f068aa2-1852-430e-9bec-c8c59acd8eb2";
        :>> title = "module_home's anchor scan passed a nested keel-cli/src path as none";
        :>> createdAt = "2026-09-16";
        :>> createdBy = "claudeFable5";
        :>> description = "module_home's anchor scan (sprint 719's control for issue559) matched a keel-cli/src path expression only when the .rs token followed 'src' directly: join(..., 'keel-cli', 'src', 'x.rs'). A nested path passed it as none. Found by reading scripts/module_home.py:104 during sprint 732 against the three anchors the scan should have named: scripts/exec_brief/facts.py:1578 and :1725 (join(REPO, 'keel-cli', 'src', 'view', 'knowledge.rs')) and, once the regex was widened and re-run, facts.py:2183 (view/control_structure.rs) - each naming a file this very sprint moved to members/keel-view, so python scripts/module_home.py --probe was green at 397e49c while the brief's knowledge-domain and control-structure facts would have read an absent file after the move and reported null. The control: ANCHOR admits any number of quoted segments between 'src' and the .rs token, and the probe carries the nested line as a known-positive beside the flat one; the three anchors resolve through _mh('view/knowledge') and _mh('view/control_structure'). Resolved in sprint 732 - the story's DoD names the widened scan and its probe case.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from storyViewIsAMember to issue579;

    part issue580 : Issue {
        :>> id = "e7bf9d9a-a31d-4584-9991-cedf6005d63c";
        :>> title = "modgraph --check is vacuous once its forbidden sources are members";
        :>> createdAt = "2026-09-16";
        :>> createdBy = "claudeFable5";
        :>> description = "scripts/modgraph.py --check (the D0479 layering rung: nothing in view, orient, write or scaffold references guards; nothing in view references orient) reads only keel-cli/src. A module re-exported from a member is a node with no files (modgraph.py:51-56), so it has no outgoing edges to check. Sprint 718 moved orient, write and scaffold into members and sprint 732 moves view: every forbidden SOURCE is now a fileless node, and the line 'layering holds - 74 modules, 184 edges' (down from 214 at 397e49c) is vacuous for all five forbidden pairs - the check cannot fail on the thing it names. Read from scripts/modgraph.py:30 and :51-56 and from running it on the sprint 732 tree. Nothing broke: Cargo's acyclic crate graph forbids keel-view depending on keel-cli by construction, and the move script asserted no crate:: reference in keel-view named a module outside it. But a control that went silent when its subject moved is a defect of D0388's class - a check with no reachable known-positive - and DoD texts still cite the rung as evidence. The resolver is the crate-level layering guard dcWorkspaceLayeringIsGuarded (chartered by D0483, held): a guard that reads every member's Cargo.toml against workspace-layers.toml is the check that can fail on a member, and modgraph's --check retires into it. Until D0483 is accepted, the honest statement of a sprint's layering proof is the release build under deny(warnings) plus the member's dependency list, not the modgraph line.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcWorkspaceLayeringIsGuarded to issue580;

    part issue581 : Issue {
        :>> id = "03dd2857-f3ac-415b-9bec-9c888a5cae0e";
        :>> title = "A quoted probe line is split on whitespace and reported as a red no check produced";
        :>> createdAt = "2026-09-16";
        :>> createdBy = "claudeFable5";
        :>> description = "keel verify splits each probe line on whitespace and runs it shell-free (keel-cli/src/verify.rs:238 and :390-392, read after the run), so quoting inside a line has no meaning - yet nothing refuses a quoted line, and the pair-file shape check (D0500) counts lines only. Sprint 732's second verifier dispatch carried a negative side written as python -c with a quoted program: the ladder passed the program's fragments to the interpreter as separate arguments, the interpreter failed with SyntaxError: unterminated string literal, and the ladder reported probe=fail - a red no check produced, the class D0500 was recorded to end, arriving through a different door. The fix in the sprint was to move the check into a script run by absolute path; the receipt for that dispatch cost one full ladder climb. The control is the shape check refusing a quote character in either line before any rung runs, naming the rule, and the two dispatch skills saying it in one sentence: a probe line is bare tokens; a check that needs arguments is a script by path.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcQuotedProbeLineIsRefused to issue581;

    part issue582 : Issue {
        :>> id = "e62ca2b2-e23b-44dc-b181-b600e83c17a7";
        :>> title = "The brief's pair-file stem sensor read a side's drive letter as the stem: a greedy regex over a command line, right by the shape of its input";
        :>> createdAt = "2026-09-16";
        :>> createdBy = "claudeFable5";
        :>> description = "The decision brief's fact reader (scripts/exec_brief/facts.py, probeRung.fileStem) read the pair file of the latest verify ladder as `control_structure ; python C` on 2026-09-16. The receipt's probe command was `--probe-from <scratchpad>/pair732.txt: python scripts/module_home.py view/knowledge guard_names view/control_structure ; python C:/<scratchpad>/neg732.py`; the sensor's regex `--probe-from (?:.*[///])?([^///:]+):` is greedy, so `.*` ran past `pair732.txt:` to the last slash of the second SIDE and took the `C:` of that side's own path as the stem's terminating colon. Brief 38's sides were two cargo test names with no colon, so the same expression read `pair731.txt` correctly the publish before; the sensor was right by the shape of its input, not by construction. CAUGHT before the claim: the thirty-ninth builder refused at the check that compares the pair's sides to the tests it expected (a check about a different fact), and the fileStem value was read in the refused facts file. Had that check not existed the page would have quoted the false stem inside a sentence the reader is told is measured. Same class as issue553 and issue400: a kept check with no known answers - facts.py carried a D0388 pair on one sensor (the exit code) and on none of the regular expressions it runs over receipts, command lines and paths. Corrected at the root in this commit: the stem is read by one helper, non-greedy and refusing a stem with a space, `;` or `:`, with its positive (the sprint 732 line) and negative (a --probe line with no file) run before any fact; the builder refuses a stem that is not one token. The class control - every text-parsing sensor in that file carries its pair and a source scan refuses an unprobed one - is dcFactSensorsCarryTheirPair.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcFactSensorsCarryTheirPair to issue582;

    part issue583 : Issue {
        :>> id = "1e033034-9969-4b6d-92cb-5d56129f875d";
        :>> title = "Sprint 733 paid the cwd-relative test-anchor fix dcOneRepoRootHelper predicted";
        :>> createdAt = "2026-09-17";
        :>> createdBy = "claudeFable5";
        :>> description = "Sprint 733 (dcGuardsAreMembersPerFamily) moved keel-cli/src/guards.rs into member keel-guards, and twelve cwd-relative test anchors (Path::new('../.engine/...'), '../.tracking', '../scripts') broke because the file was one directory deeper. scripts/split_guards.py paid the fix with a DEPTH_REWRITES table. dcOneRepoRootHelper (backlog) predicted exactly this recurrence at sprint 731 for the guards.rs seven and hardening.rs five; the sprint that moved them paid it instead of the helper. Check: the count of Path::new('../ in the pre-move guards.rs (git show 50e171a:keel-cli/src/guards.rs) against the DEPTH_REWRITES table in scripts/split_guards.py. The item is moved to the head of the D0479 cluster (declaration order is priority, D0052) so the next member extraction (dcSuiteIsAMember) inherits no path fix.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcOneRepoRootHelper to issue583;

    part issue584 : Issue {
        :>> id = "072a377a-e0ae-4aec-8809-2066d155b091";
        :>> title = "guards.md groups by tier; the DoD assumed it grouped by family";
        :>> createdAt = "2026-09-17";
        :>> createdBy = "claudeFable5";
        :>> description = "The backlog DoD of dcGuardsAreMembersPerFamily said the family files follow 'the families guards.md already groups by'. guards.md groups by TIER (grep '^## ' .engine/docs/guards.md gives Hard-blocking, Warning-only, Non-blocking burndown), not by family; the nine families came from the FAMILIES tables in code, and the catalogue's rows do not say which family file a guard lives in. A reader of guards.md cannot find a guard's source. The premise in the DoD was written from memory of the doc, not from the doc.";
        :>> discoveredInField = false;
        :>> severity = Severity::Low;
    }
    #Resolves dependency from dcGuardsCatalogueNamesTheFamily to issue584;

    part issue585 : Issue {
        :>> id = "4b9ab78c-9230-4d82-98da-ecd3d4063b93";
        :>> title = "Two members reach upward to keel-cli/build.rs and modgraph does not see it";
        :>> createdAt = "2026-09-17";
        :>> createdBy = "claudeFable5";
        :>> description = "keel-cli/build.rs (the build-provenance stamp) is the build script of two members that reach upward to it: members/keel-write/Cargo.toml and members/keel-guards/Cargo.toml both carry build = '../../keel-cli/build.rs' (grep 'build = ' members/*/Cargo.toml). modgraph's layering check reads [dependencies], not build =, so a member depending on a file owned by the crate above it passes the check that forbids the same reach in source. Each new member that needs the stamp copies the upward path.";
        :>> discoveredInField = false;
        :>> severity = Severity::Low;
    }
    #Resolves dependency from dcBuildScriptHasOneHomeBelowItsUsers to issue585;

    part issue586 : Issue {
        :>> id = "667373fd-62aa-466c-90b5-0e2f4e733d2f";
        :>> title = "A facts.py block about a fixed commit range read its file from the working tree";
        :>> createdAt = "2026-09-17";
        :>> createdBy = "claudeFable5";
        :>> description = "The D0499 block of scripts/exec_brief/facts.py describes a fixed commit range (git diff 1ecbef0 50e171a -- guards.rs) but read the file's TEXT from the working tree via module_home('guards'); when sprint 733 moved the file every fact of the block went null although the range it describes had not changed. Fixed in the sprint by reading git show 50e171a:keel-cli/src/guards.rs. The class (a fact about a range whose text comes from HEAD) has no control; facts.py has no self-probe that flags it.";
        :>> discoveredInField = false;
        :>> severity = Severity::Low;
    }
    #Resolves dependency from dcFactsAboutARangeReadTheRange to issue586;

    part issue587 : Issue {
        :>> id = "c0de7e36-227c-494f-be34-ea8edab57d06";
        :>> title = "A probe scanned keel-cli as the Rust tree and went red when the guard bodies became a member";
        :>> createdAt = "2026-09-17";
        :>> createdBy = "claudeFable5";
        :>> description = "scripts/probes/issue400_reproduction.py census_defective scanned keel-cli/**/*.rs as 'the Rust tree'. Sprint 733 (dcGuardsAreMembersPerFamily) moved the guard bodies to members/keel-guards/src, and the reproduction of the 2026-09-06 defect stopped reproducing: the defective census read 27 of 75 and the known-negative doc-guard-count read False, so the probe exited 1 and the pre-commit script-probe runner (D0496, issue574) aborted the sprint's landing commit. The check was right to be red - a one-crate scan is the class D0504's context names, a control that goes quiet when its subject moves - and the control fired before the commit, as designed. Fix: the scan covers keel-cli and members/**, the workspace's Rust files; the probe reads 75 of 75 on the defective census and 0 failures. A grep of scripts/ for a keel-cli-only .rs glob found this one site (modgraph.py's DEFAULT_SRC is issue581's). Resolved by the sprint that moved the files, dcGuardsAreMembersPerFamily, in its landing commit; the standing control is the script-probe runner, and this Issue records that it fired.";
        :>> discoveredInField = false;
        :>> severity = Severity::Low;
    }
    #Resolves dependency from dcGuardsAreMembersPerFamily to issue587;

    part issue588 : Issue {
        :>> id = "3f3fe259-256f-463c-ac3c-5de73462a4fe";
        :>> title = "The full suite fingerprints and tests keel-cli alone; a member edit leaves its receipt unchanged";
        :>> createdAt = "2026-09-17";
        :>> createdBy = "claudeFable5";
        :>> description = "The full suite measures keel-cli alone. members/keel-suite/src/suite.rs:28 declares DELIVERABLE_PATHS = ['keel-cli', '.engine', 'keelw', 'Cargo.toml', 'Cargo.lock'], so the deliverable fingerprint the receipt carries omits members/ and keel-parser/: an edit to any workspace member's source leaves the fingerprint equal, and the staleness message at suite.rs:146 (the deliverable CHANGED since the last green suite run) never fires for it. The run itself at suite.rs:267 is cargo test --release --manifest-path keel-cli/Cargo.toml --no-fail-fast, the keel-cli package only, so the members' own lib tests (keel-suite 50, keel-guards 137, keel-view 159, keel-write 62, ...) are outside the full run while the touched set (touched.rs:315) does include them. Found reading suite.rs during sprint 735 (dcSuiteIsAMember), which moved the file; the gap predates the move (the constant is unchanged since D0387) and widened with each D0479 extraction since sprint 714. keel suite gates nothing (D0356), so no gate is dishonest - the receipt is; the fingerprint says the deliverable is unchanged when a member changed. Not fixed in sprint 735: its scope is the move only.";
        :>> discoveredInField = false;
        :>> severity = Severity::Low;
    }
    #Resolves dependency from dcSuiteMeasuresTheWorkspace to issue588;

    part issue589 : Issue {
        :>> id = "6db72e2d-1a55-4f36-bbfd-3aecf51b3025";
        :>> title = "Sprint 735 closed with its story DoD passing and the item it delivered, dcSuiteIsAMember, had no DoD result: ranked first as ready at 249dac2";
        :>> createdAt = "2026-09-17";
        :>> createdBy = "claudeFable5";
        :>> description = "Sprint 735 closed at 249dac2 with its story DoD passing (storySuiteIsAMemberDoDR1 in the delivery file), six gate results recorded and the landing pushed, and the backlog item it delivered, dcSuiteIsAMember, had no DoD TestResult: keel show orient at 249dac2 ranked it FIRST as ready work and held dcSuiteMeasuresTheWorkspace and dcKeelCliIsThinDispatch blocked on it as not done. The recorder's dispatch named the owed result as record result --file .tracking/delivery/sprint735_suiteIsAMember.sysml --task storySuiteIsAMember, the story, where sprint 734's named the item (dcOneRepoRootHelperDoDR1 in backlog.sysml); the recorder wrote exactly what it was owed, check_report.py --owed 7 passed, and nothing read the fill's DELIVERED BACKLOG ITEMS line against the --task set. Second occurrence of the issue563 class (sprint 720, 2026-09-14): an un-typed delivery relationship the primary has to remember when it types the dispatch. Recorded the item's DoD now from the same verifier receipt (a recorder dispatch owed one record). Control: the #Delivers edge the sprint-closure guard reads (resolver dcSprintNamesTheItemItDelivers), declared earlier in the backlog on this recurrence.";
        :>> discoveredInField = false;
        :>> severity = Severity::High;
    }
    #Resolves dependency from dcSprintNamesTheItemItDelivers to issue589;

    part issue590 : Issue {
        :>> id = "b108aa14-ab9f-4db9-b69e-2a9f117f97de";
        :>> title = "Sprint 735's second recorder refused its one owed record citing the verifier receipt's OWED WRITES: NONE line, and check_report.py passed the report";
        :>> createdAt = "2026-09-17";
        :>> createdBy = "claudeFable5";
        :>> description = "The second recorder of sprint 735, dispatched --owed 1 to write dcSuiteIsAMember's DoD result from the verifier receipt that the first recorder had written seven records from, wrote nothing and reported REFUSED: task dcSuiteIsAMember: receipt line 19 states OWED WRITES: NONE; no writes owed per verifier. Line 19 is the test-verify receipt's closing line about the VERIFIER's own writes - none, by design (it writes nothing under the project) - and says nothing about what the recorder owes; the dispatch's count is the only owed count (D0492). check_report.py --owed 1 passed the report, because a REFUSED line accounts for a record whatever its reason, so the misread was visible only to the primary reading the report. A second message naming the line's meaning produced the write (dcSuiteIsAMemberDoDR1). Class: a receipt line whose label reads as the recorder's own instruction. Control: check_report.py refuses a REFUSED line that cites OWED WRITES as its reason, and the receipt line is labelled as the verifier's (resolver dcRecorderRefusalCannotCiteTheVerifiersNoWritesLine).";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcRecorderRefusalCannotCiteTheVerifiersNoWritesLine to issue590;

    part issue591 : Issue {
        :>> id = "a6db3734-041b-4f70-b414-347842e9422b";
        :>> title = "Source citations on the living docs are held to nothing: project-migration cites migrate.rs:664-666, a file sprint 736 moves";
        :>> createdAt = "2026-09-17";
        :>> createdBy = "claudeFable5";
        :>> description = "The living doc surface cites source by path and line and nothing reads the citation against the tree. .engine/skills/project-migration/SKILL.md line 10 says check_preconditions refuses a self-build at migrate.rs:664-666; at 01f7ad00 the function sits at keel-cli/src/migrate.rs:805-807, and sprint 736 (dcProcessIsAMember) moves the file to members/keel-process/src/migrate.rs, so the citation is wrong twice over - line range and path - with the prose untouched. Found doc-syncing the move. The fix is one line, but .engine/skills/** is under the keystone lock (is_process_def, members/keel-guards/src/enforcement.rs:34-40), so it needs a co-committed marked Decision and cannot ride an extraction sprint whose Decision says no locked path is edited (D0507); the citation stays stale in this commit. The control gap is the class, not the line: tool-reference holds .engine/tools/<file> names to disk and cli-reference holds keel verbs to the dispatcher, but a <file>.rs:N citation is held to nothing, and every D0479 extraction moves files that skills cite. Resolver: dcSourceCitationsOnTheLivingDocsResolve - a source-reference guard in tool-reference's family that resolves each citation through the manifest-driven module home and checks the named identifier sits in the cited range; its known-positive is this line.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcSourceCitationsOnTheLivingDocsResolve to issue591;

    part issue592 : Issue {
        :>> id = "01c69d64-7a9c-4a54-8e36-0994bbde465a";
        :>> title = "verify --wait reports a ladder that finished between its two reads as KILLED";
        :>> createdAt = "2026-09-17";
        :>> createdBy = "claudeFable5";
        :>> description = "`keel verify --wait` reads the receipt, then asks whether the writer's pid is alive (members/keel-suite/src/verify.rs:623-627: read_to_string, then wait_state(&text, pid_alive)). A writer that rewrites the final receipt and exits between those two reads leaves the reader holding a `running` stub and a dead pid, which wait_state names Killed, and the ladder that just passed is printed as `KILLED during <rung>`, exit 2 - the same words issue575 reserved for a writer that died. Observed in sprint 737's close-out at 9e4e79c6: the ladder's own stdout ended `keel verify: pass - every rung green in 1070 s` (exit 0) and the receipt read `outcome = pass` at 1789658057, while `keel verify --wait .` running beside it printed `KILLED during touched - .keel/metrics/verify-receipt.toml says running and its writer (pid 52848) is gone` in the same second. The command's own instruction is `Relaunch keel verify`, which would have discarded an 18-minute green run; the previous window's relaunch of a run whose verifier agent had stalled was the same command read the same way, and it is not certain that one was dead either. The class: a two-read liveness check with no confirmation read. Resolver: dcWaitRereadsBeforeKilled - a Killed answer is confirmed by re-reading the receipt once after the liveness answer; a re-read that is Finished is reported as the ladder's verdict.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcWaitRereadsBeforeKilled to issue592;

    part issue593 : Issue {
        :>> id = "95a58b8d-1913-4f57-9771-9986d668bd8c";
        :>> title = "The code registry's filePath values name nine files the extractions moved and nothing reads them";
        :>> createdAt = "2026-09-17";
        :>> createdBy = "claudeFable5";
        :>> description = "Eleven of the twenty-five `filePath` values in .tracking/architecture/code-registry.sysml (nine distinct: keel-cli/src/actor.rs, arch.rs, claim.rs, guards.rs, migrate.rs, orient.rs, sync.rs, view.rs, write.rs) name files that no longer exist - the D0479 extractions moved each under members/ between sprints 731 and 737 - and nothing read the values, so the CodeElement-to-source map has been wrong for seven sprints without a red. Established by a shell loop over the attribute values at 904ff24c testing each path's existence (11 missing of 25). Sprint 738's codemod (scripts/extract_serve.py) corrected the two values whose elements are claudeFable5's (ceServeJudgment -> members/keel-serve/src/serve.rs, ceLauncher -> members/keel-serve/src/launcher.rs) because the transform knew the moves; the nine others are claudeOpus5's and stay stale until corrected by an ADD or a #Supersede (D0108). The class: an authored path attribute with no control that reads it against the tree - the same class as deliverable-manifest.txt rows before the suspicion computation read them. Resolver: dcCodeRegistryPathsResolve - a `code-registry-paths` check that fails on any filePath with no file at ROOT, plus the eleven corrections.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcCodeRegistryPathsResolve to issue593;

    part issue594 : Issue {
        :>> id = "90caaabe-44b6-4aef-afa1-c75620f22e9b";
        :>> title = "Six leaf members carry no lint preamble, so the workspace holds two lint regimes";
        :>> createdAt = "2026-09-17";
        :>> createdBy = "claudeFable5";
        :>> description = "Six of the fifteen crates under members/ (keel-actor, keel-git, keel-github, keel-json, keel-perf, keel-schema - the leaves sprint 714 split out) have a lib.rs with no lint attribute at all, while keel-fs and every member extracted since sprint 731 carry the preamble that forbids unsafe and denies warnings, clippy::all/pedantic/nursery, unwrap/expect/panic/indexing/todo/unimplemented. Established at 904ff24c by reading each members/*/src/lib.rs for `deny(warnings` (six absent). The consequence: a warning or an unwrap in a leaf crate compiles clean under `cargo build --release` - the settle build every D0388 positive probe runs - and is caught only by the `cargo clippy --workspace -- -D warnings` rung, so the workspace holds the same kind of code under two lint regimes and nothing reads which. Surfaced in sprint 738 when scripts/extract_serve.py wrote the preamble into members/keel-serve/src/lib.rs from its LINTS constant and added ci_runs.rs to members/keel-github, whose lib.rs has none. Resolver: dcMembersCarryTheLintPreamble - the preamble on the six, the build clean under it, and a `member-lints` check that fails naming any member whose lib.rs lacks a line of it.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcMembersCarryTheLintPreamble to issue594;

    part issue595 : Issue {
        :>> id = "03e0aa69-cbc5-4a82-8da3-85d9809e8b81";
        :>> title = "the sprint 738 ladder's touched rung spanned a 97-minute host standby and two clock-sensitive tests read red that pass in isolation; the receipt named the reds and not the standby - issue481/issue534's class in the form no margin absorbs";
        :>> createdAt = "2026-09-17";
        :>> createdBy = "claudeFable5";
        :>> description = "The sprint 738 verifier's touched rung spanned a 97-minute host standby (Kernel-Power 506 at 11:59:01, 507 at 13:36:04) and two clock-sensitive tests read red - an_output_still_linking_is_not_copied and turn_boundary_gate_allows_a_clean_tree_and_blocks_a_dishonest_one - that pass in isolation in under six seconds; the receipt named the reds and not the standby. The touched run (launch epoch 1789667298, log .keel/metrics/touched-1789667668.log) reported Summary 259 tests run: 257 passed (18 slow), 2 failed, 6106 s, where a full set takes 7-15 minutes; the 18 slow tests each read 5930-6040 s and were the ones in flight when the host entered Modern Standby five minutes into the run. an_output_still_linking_is_not_copied (keel-cli/tests/self_build_hooks_leave_the_build_tree.rs:112-118) stamps its fixture NOW+600 s so a two-second grace cannot age past it - the fix for issue481 - and the standby advanced the wall clock 5820 s, so the fixture read settled and was copied. turn_boundary_gate_allows_a_clean_tree_and_blocks_a_dishonest_one (keel-cli/tests/remaining_controls_actually_fire.rs:115) read an empty stdout from the second stop-hook fire; its mechanism under the standby is not established - it is recorded as in flight during the same window and green in isolation, not as diagnosed. Third instance of the class issue481 and issue534 named (an assertion whose truth depends on how fast the host reaches a clock read judges the host, not the change); both earlier fixes widened a margin, and a suspend defeats every margin, so the control belongs in the runner: the receipt measures suspended time (wall clock against a clock that stops while the host sleeps) and re-runs a red once in isolation when the run spanned a standby, naming both. Until then a verifier whose run crossed a sleep hands the primary a red it must diagnose by hand against the Windows event log, as this one was.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcSuiteReceiptNamesTheStandbyItSpanned to issue595;

    part issue596 : Issue {
        :>> id = "565a4cfe-0391-4a50-8200-a4ad8fc19be4";
        :>> title = "The result writer doubles the RAN prefix on quoted evidence, and the two receipt readers then read an empty receipt";
        :>> createdAt = "2026-09-17";
        :>> createdBy = "claudeFable5";
        :>> description = "The result writer prepends `// RAN: ` to every evidence string unconditionally (members/keel-write/src/write.rs:837 and :1293), and an actor that passes evidence already carrying the receipt prefix - the shape CLAUDE.md quotes as the `// RAN:` receipt - lands a line reading `// RAN: // RAN: <receipt>`. At 904ff24c `git grep -c '// RAN: // RAN:' -- .tracking .engine` counts 71 such lines in 29 files (15 in .tracking/backlog.sysml, 7 each in sprints 714, 715 and 721, one to five in twenty-four more); sprint 738's two DoD results were written the same way before the count was taken. The doubled prefix is not cosmetic: the two readers that act on a receipt take the text after the FIRST prefix - members/keel-write/src/reverify.rs:143 `line.split('// RAN:').nth(1)` and members/keel-github/src/ci_runs.rs:76 the same - so on a doubled line the receipt they read is the empty string between the prefixes, and a replayable demo receipt (D0444) or a `ci-run id=` receipt (D0323) on such a line is silently dropped from `keel record reverify --demos` and `audit ci-runs`; the acceptance guard and the attestation lens (acceptance.rs:470, attestation.rs:85) test only for the prefix's presence and are unaffected. Class: the write path accepts a shape it then reshapes without checking whether the shape is already there - the issue448 fix moved the receipt check to the write and did not make the write idempotent on its own prefix. Control belongs in the writer (strip one leading `// RAN:` and its whitespace before formatting, so the same evidence lands identically whether or not the caller quoted the prefix) with the D0388 pair as unit tests, and in the readers (take the text after the LAST prefix on the line, so history already written reads correctly); the 71 existing lines are then a content-preserving migration under the migration skill or are left as they are once the readers tolerate them - the resolver decides, and says which.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcRanReceiptCarriesOnePrefix to issue596;

    part issue597 : Issue {
        :>> id = "3e9ab0b9-398f-4e83-a850-82a7a2e51305";
        :>> title = "The sitting review is the one ceremony record still finished by method = confirmation, a debt the console counts as 309 owed";
        :>> createdAt = "2026-09-17";
        :>> createdBy = "claudeFable5";
        :>> description = "The per-sitting review is the one ceremony record still finished by method = confirmation: .engine/skills/sprint-review/SKILL.md lines 27-34 and 119-131 record `sittingRev<id> : Test { method = VerificationMethod::confirmation }` judged by the human, sittingReviewVP (.engine/views/viewpoint-registry.sysml:328) sits on the act surface with the concern 'which sittings have not yet had their per-sitting human review (D0049's single gate)', and keel show sitting-coverage at e5e71f2a reads 309 of 738 sprints due such a review. D0204 retired the framing (the review is not a mandatory human gate; sitting coverage left the authority queue) but not the method, so the skill still asks the human to 'accept the sitting's content (the one gate)' and the console's act tab still counts the 309 as owed. Every judgment a review presents already has a per-item verb (keel accept, keel judge-set, a disposition), and 'the review is finished' is a receipt, not testimony (D0232) - a confirmation over it is a duplicate or a reading receipt, and a count of them is a debt aimed at the human (D0204). The human said so on 2026-09-17: 'change the method for finishing a review to not be confirmation. what should there be to confirm? on a sprint review'. D0510 (HELD, process-change) decides the method; this Issue tracks applying it and the guard that keeps it applied.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcSittingReviewIsFinishedByAnalysis to issue597;

    part issue598 : Issue {
        :>> id = "d374b519-d2d3-4704-ac98-ae6aa2d14074";
        :>> title = "The touched test set is measured under the committing shell's environment - KEEL_ACTOR leaked into keel-land and refused the D0510 push";
        :>> createdAt = "2026-09-17";
        :>> createdBy = "claudeFable5";
        :>> description = "The touched test set is measured under whatever environment the committing shell had. Commit 619d6b0f (D0510 held) was made from a shell with KEEL_ACTOR=claudeFable5 exported; keel-land inherited it and the lib set read 859 passed, 1 failed: keel-serve launcher::tests::empty_diff_stays_local_and_written_diff_gets_gated_summary panicked at members/keel-serve/src/launcher.rs:379 'the RUN'S stamped actor, not the machine binding' (.keel/metrics/touched-1789678321.log:698,713), and the push was refused. The same test at the same code passed in sprint 738's landing run (e5e71f2a, 736 passed) from a shell without the variable. Cause, read from source: keel-actor's resolution order (members/keel-actor/src/actor.rs:159) takes KEEL_ACTOR before the .keel/actor file, and the test bound the file to 'hum' without clearing the variable, so its verdict depended on the runner's shell, not on the tree. The test-level fix (std::env::remove_var('KEEL_ACTOR') in the two launcher tests that bind the file, the idiom keel-actor's own tests already use at actor.rs:317 and :325) rides this commit. The class is wider than one test: any test that reaches actor::resolve, or reads KEEL_BIN, KEEL_ROOT or another session variable, is green or red by the shell that spawned keel-land, and CI (which has none of them) and the pre-push run then disagree - the local gate stops equalling CI (D0421's premise). The control belongs in the runner: keel land and keel suite --touched run the set with the session variables the engine documents (KEEL_ACTOR, KEEL_BIN, and the rest of the KEEL_* family the binary reads) removed from the child environment, and the receipt says so, so a green depends on the content the receipt keys (D0474) and on nothing the shell carried.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcTouchedRunEnvironmentIsNormalized to issue598;

    part issue599 : Issue {
        :>> id = "f6c31b4d-6445-45da-b16c-301b3d4c428c";
        :>> title = "The touched receipt names its base as a ref, not the sha it measured from, so a receipt's range is lost once the push moves origin/main";
        :>> createdAt = "2026-09-17";
        :>> createdBy = "claudeFable5";
        :>> description = "The touched receipt names the base its changed set was measured from as a ref, not the sha that ref resolved to. .keel/metrics/touched-receipt.toml after commit 28fcae8d reads `base = 'origin/main'` and `stems = ['init', 'launcher']`; the run was measured from merge-base(origin/main, HEAD) to the working tree (members/keel-suite/src/touched.rs, the base's doc comment at :378 and the merge-base at :407), and at run time origin/main stood at e5e71f2a, two commits behind, so `init` came from 619d6b0f's new .engine/decisions/0510 file and `launcher` from 28fcae8d. After keel-land pushed, origin/main moved to 28fcae8d and nothing in the receipt says which range it measured: a reader of the receipt cannot re-derive its stems, and brief 44's builder, reading the .engine paths of the one commit the receipt names, refused the page over an `init` stem the receipt attributes and git shows no engine path for. The brief's facts.py now recovers the base from the push history (the pushed sha before head's own push), a workaround that reads a second source for a fact the receipt should carry. The class is issue586's - a fact about a range reads the range - applied to the receipt itself: the control is that keel suite --touched writes the resolved base sha (`base_sha`, the merge-base it diffed from) beside the ref, and that any reader of the receipt (the brief facts, keel show status, the landing log line) names that sha rather than the ref.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcTouchedReceiptNamesItsBaseSha to issue599;

    part issue600 : Issue {
        :>> id = "a8fc9b2e-4328-4c3e-a461-3df2ad9a3a65";
        :>> title = "record task appends at the end of the def, so a split item cannot be written at its rank (D0052) and was moved by hand";
        :>> createdAt = "2026-09-17";
        :>> createdBy = "claudeFable5";
        :>> description = "keel record task appends the new action and its DoD at the end of the def, and declaration order IS priority (D0052), so an item that must outrank an existing one cannot be written at its rank by the API. Sprint 739 split dcHooksAreAMember out of dcKeelCliIsThinDispatch (rank 2 on keel show whats-next at 2ffa64b4) and made the parent depend on it; record task landed the slice at rank 26, below twenty-four items it now gates, and a hand script (move739.py, scratchpad) cut its two lines out of .tracking/backlog.sysml and pasted them before the parent. The move was legal (my own item, D0108) and validated clean, but it is a backlog write the write lock, the provenance check and the record verb's own validation never saw - the class D0224 and D0309 exist to keep out of the tree. Every future split (one remains: thin dispatch may split again) and every resolver that must precede the item it unblocks meets the same gap. Resolver: dcRecordTaskPlacesTheItemBeforeANamedOne - a --before <task> flag on record task that declares the item at its position inside the def and refuses a name not in the def.";
        :>> discoveredInField = false;
        :>> severity = Severity::Low;
    }
    #Resolves dependency from dcRecordTaskPlacesTheItemBeforeANamedOne to issue600;

    part issue601 : Issue {
        :>> id = "7b6a3bee-2996-45c6-90a4-b04e6daaa715";
        :>> title = "a slice that starts at the fn line strands its doc comment - second sprint running";
        :>> createdAt = "2026-09-17";
        :>> createdBy = "claudeFable5";
        :>> description = "A slice that starts at the fn line leaves the fn's doc comment behind, and the stranded doc attaches to whatever item follows. Second sprint running: sprint 739 reattached three docs stranded in keel-cli/src/main.rs by earlier moves (cmd_check_engine's and gate --fast's above gate_subverb, why's above budget_arg) and called it a one-off; sprint 740 found two more (root_arg's doc left in members/keel-process/src/workspace.rs when root_arg went to main.rs and then to keel-args, cmd_init's doc above positional_arg) and reattached them in extract_seams.py. Neither rustdoc nor clippy has a lint for a doc that describes a different item, so the hand reattachment is the only thing that has caught it. The control belongs in the transform class that causes it: a codemod's plan asserts that the line preceding every slice start is neither a doc line nor an attribute, and its reconciliation line prints the count checked, so a slice that would strand a doc fails the dry run. Resolver: dcKeelCliIsThinDispatch, the last body-slicing transform, whose DoD now names the assertion.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcKeelCliIsThinDispatch to issue601;

    part issue602 : Issue {
        :>> id = "c09aae7a-240e-44e0-8fc0-1bb980cf2f56";
        :>> title = "Recorder report filed a landed write as REFUSED and check_report.py passed it - nothing reads the report against the tree";
        :>> createdAt = "2026-09-18";
        :>> createdBy = "claudeFable5";
        :>> description = "Sprint 741's recorder was dispatched with eight owed records; the eighth named --task dcTouchedSetDescendsTheWorkspaceDoD, the Test's name where the write API takes the action's. The write API refused that attempt; the recorder retried with --task dcTouchedSetDescendsTheWorkspace, the result dcTouchedSetDescendsTheWorkspaceDoDR1 landed at .tracking/backlog.sysml with a RAN receipt, and the report carried it as REFUSED: task not found - one REFUSED line, seven WROTE lines, eight accounted for. check_report.py --owed 8 passed it. The tree held eight new TestResult parts; the report said seven were written. Found by the primary reading git diff .tracking/backlog.sysml after the report, not by any control: check_report.py reads the report alone and has no count of what the tree gained, so a report that misfiles a landed write as a refusal passes every check it has (D0473 catches a second write of one gate, D0492 catches too few lines; neither reads the tree). Same class as issue568 (wrote nothing, reported NONE) with the sign reversed. Resolver: check_report.py --root counts the TestResult parts added under .tracking since HEAD and refuses a report with fewer WROTE lines than that, naming the surplus part.";
        :>> discoveredInField = false;
        :>> relatedTask = "dcTouchedSetDescendsTheWorkspace";
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcRecorderReportAccountsForEveryTreeWrite to issue602;

    part issue603 : Issue {
        :>> id = "ea84479d-cb65-44fd-8e7e-12e4e1a698b6";
        :>> title = "Verifier receipt called a live, then red, ladder KILLED - nothing reads the receipt against the ladder it describes";
        :>> createdAt = "2026-09-18";
        :>> createdBy = "claudeFable5";
        :>> description = "Sprint 743's verifier wrote its receipt with LADDER -> KILLED during touched; exit=2 and DISCREPANCIES: keel verify ladder outcome shows running with pid 29824 but process is dead at receipt read. At that moment tasklist listed pid 29824, the touched run's pid 38432 and cargo-nextest; keel verify --wait . printed in flight: touched, pid 29824 alive; the ladder then ended with the touched rung red (880 passed, 23 failed, exit 101) and the verifier's own background keel verify --wait returned that 101, which its final report restated as confirming the kill. The receipt was written from the running stub before the ladder ended and never rewritten from the ended ladder, so the primary received a receipt that was wrong about the ladder's state and about its verdict - the sprint 726 shape (issue575), whose control was a sentence in the test-verify skill saying to wait. A sentence is not a control (D0047): nothing reads the verifier's receipt against the ladder receipt it describes, the way check_report.py reads the recorder's report. The 23 red tests were real - the new source-reference guard reddened every init-scaffolded project, whose engine docs cite keel's source without holding it - and were found by the primary reading the touched receipt directly, not by the verifier.";
        :>> discoveredInField = false;
        :>> severity = Severity::Medium;
    }
    #Resolves dependency from dcVerifierReceiptIsCheckedAgainstTheLadder to issue603;
}
