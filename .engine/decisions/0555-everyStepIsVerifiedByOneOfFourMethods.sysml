// D0555 (status: the `status` field below is the truth) — everyStepIsVerifiedByOneOfFourMethods: every runnable step declares how it is verified, by analysis, inspection, demonstration or test, and a step with no declared method is not runnable; static analysis and secret scanning are the analysis and test instances a code step declares, not a hard-coded pair
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0555Accept event on sign-off.
package Decision0555 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    part d0555 : Decision {
        :>> id = "8bb44fc2-af8c-4010-9f25-ab5b6e7bd6cf";
        :>> title = "everyStepIsVerifiedByOneOfFourMethods: every runnable step declares how it is verified, by analysis, inspection, demonstration or test, and a step with no declared method is not runnable; static analysis and secret scanning are the analysis and test instances a code step declares, not a hard-coded pair";
        :>> createdAt = "2026-09-23";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::accepted;
        :>> context = "D0548 asked whether static analysis and secret scanning are required predicates on every runnable code step. The human answered on 2026-09-23 with OPTION A ticked and these words (st198 here): 'we need a much more abstract idea here than some hardcoded focus on code. at a high-level, everythign should be verified by one of the 4 verification lenses: {analysis, inspection, demonstration, test}'. Their brainstorm the same day in keel-next (st201 there): 'Can something be verified by test? Then no need for input - it's verified. Can a step be verified through demonstration? Analysis? Inspection?' keel's own schema already carries VerificationMethod with these members (D0006) and the requirement kinds carry a verificationMethod; the runner direction's I-11 makes checkedBy mandatory on a runnable step.";
        :>> decision = "Every runnable step declares its verification method, one of analysis, inspection, demonstration or test, and the check bound to that method; a step with no declared method is not runnable. The method decides who judges: test and demonstration by the tool, analysis by the AI through a rubric with independent review, inspection by the human. For a code step, static analysis is a declared analysis check and secret scanning a declared test; they are required where the step's process declares them, not by a rule naming code. This supersedes D0548 whole. NOT A FORK: the human stated the rule in their own words; this records it. NOT A PROCESS CHANGE: this repository's processes do not change; the rule binds the runner design and keel-next's meta-process, whose steps already carry checkedBy.";
        :>> rationale = "A rule naming two tools for code steps is one instance of a general rule and would be re-asked for every other step kind; the general rule is the one the human stated and the one the schema already has vocabulary for. It also answers when the human is asked: never for a step whose method the tool can run.";
        :>> consequences = "keel-next's ProcessStep carries verificationMethod beside checkedBy; its N-17 candidate, the verification method decides whether the human is asked, rests on this; D0548 is retired by the edge beside this record; keel-next HANDOFF question 2 is answered.";
    }
    #Supersede dependency from d0555 to d0548;
    #DerivedFrom dependency from d0555 to st198;

    // acceptance event (D0121 review-queue sign-off; D0066/D0106 — human-judged, not fabricated)
    verification d0555Accept : Test { :>> id = "23e8aadf-e01b-4b1c-8c20-f555979a1111"; :>> method = VerificationMethod::confirmation; :>> procedureText = "AUTO-ACCEPTED under standing consent (D0207). Their standing words, verbatim: 'issues raised are automatically accepted. they can be customizedly changed post-fact.' Not individually reviewed; override by a superseding Decision (D0290) or the human's quoted word in chat (D0289)."; }
    part d0555AcceptR1 : TestResult { :>> id = "a223b413-82fd-40f4-90ff-96118d6e30db"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "996db40d"; :>> judgedAt = "2026-09-23"; :>> judgedBy = "wweatherholtz"; :>> createdBy = "claudeFable5"; }
}
