// D0547 (status: the `status` field below is the truth) — governanceOutputsTheApiCannotProduceAreRefusedOrBlobHarvested: a step whose declared governance output has no record verb - a Need, a requirement, an edge edit - is either not runnable until a verb exists, or blob-harvested under validate plus the evidence quarantine
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0547Accept event on sign-off.
package Decision0547 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    part d0547 : Decision {
        :>> id = "dff9ca0b-db27-4eb4-899e-cbe871de46d3";
        // RESEARCH: three rounds of a nine-persona adversarial panel over four revisions of the direction artifact, every finding verified against the repository at f365cb7a; the two landscape spikes D0540 (94 rows) and D0541 (122 user threads); docs/reviews/runner-direction-panel-2026-09-22.md holds the reports and the finding-to-resolution maps.
        :>> title = "governanceOutputs: a step whose declared governance output has no record verb - a Need, a requirement, an edge edit - is either not runnable until a verb exists, or blob-harvested under validate plus the evidence quarantine";
        :>> createdAt = "2026-09-22";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::accepted;
        :>> context = "Revision 4 of the panelled direction (D0542) replays every governance output of a run through the write API so the write-time refusals fire - the receipt-owed refusal, provenance never defaulted, the AI-actor acceptance refusal - and the replayable set is the record sub-verbs minus result, gate-result and reverify. The record verbs cover a Decision, an Issue, a Statement, a story, a task, a sprint scaffold, a review batch, a measurement, a snapshot and a mint; they do not cover a Need, a SystemRequirement, a Persona, a UseCase, an Architecture item, a Risk, or an edit to an existing item's text or edges - CLAUDE.md itself says 'direct edits for what it doesn't cover'. Under revision 4 such a step is not runnable. The weaker path is a blob harvest checked by validate and the evidence-surface quarantine; validate alone once accepted six hand-written results, four with non-hex ids (issue454), which is why replay was chosen. The steelman named this as a human-decidable fork; the value critic noted the paying classes - 103 of the 129 counted defects - are Delivery ceremony the verbs already cover.";
        :>> decision = "OPTION A (recommended): refuse - a step declaring a governance output no record verb produces is not runnable, the refusal is recorded, and the write API grows by CliCommand facts (D0271) one verb at a time as steps need them; the count of such refusals is a slice-one indicator. OPTION B: harvest those outputs as blobs under validate plus the quarantine of evidence surfaces, accepting the weaker check for item types the API does not yet write.";
        :>> rationale = "A keeps one write path and lets the authoring friction it creates show up as a measured count rather than a silent bypass - which is also the fact that distinguishes the two hypotheses for the empty quadrant the landscape found. B lets Business and Architecture steps run in slice one at the cost of admitting the exact check issue454 defeated. The paying defect classes need no verb the API lacks, so A costs slice one nothing it was built to deliver.";
        :>> consequences = "Under A: revision 4 stands as written; each new record verb is a CliCommand fact and a write-path refusal set; the refusal count is an indicator. Under B: the slice-one CR adds a blob-harvest path with the quarantine parser and states the residual check gap. COST under A: Business and Architecture steps wait until a record verb exists for their item kind, and each verb is a CliCommand fact plus a write-path refusal set. COST under B: a blob-harvest path with the quarantine parser and the residual check gap issue454 exposed.";
    }

    // acceptance event (D0121 review-queue sign-off; D0066/D0106 — human-judged, not fabricated)
    verification d0547Accept : Test { :>> id = "91ddb7b5-8980-444f-8735-464bf03fa517"; :>> method = VerificationMethod::confirmation; :>> procedureText = "Their words: 'OPTION A: a step declaring a governance output with no record verb is not runnable; the refusal is recorded and counted' (pasted copy-for-AI digest with this option ticked, chat, 2026-09-22) - their words, verbatim: “OPTION A: a step declaring a governance output with no record verb is not runnable; the refusal is recorded and counted”"; }
    part d0547AcceptR1 : TestResult { :>> id = "1b4c8eec-0be4-498a-acb9-53ccb066d394"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "b65a13e4"; :>> judgedAt = "2026-09-22"; :>> judgedBy = "wweatherholtz"; :>> createdBy = "claudeFable5"; }
}
