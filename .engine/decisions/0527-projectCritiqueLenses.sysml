// D0527 (PROPOSED — NOT YET ACCEPTED) — projectCritiqueLenses: critique-policy.toml names a project schema package that supplies lens variants and guard critique checks names against it - or the onboarding path only documents the rule - or nothing changes
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0527Accept event on sign-off.
package Decision0527 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    // RESEARCH: the frozen CritiqueLens enum (.engine/schema/core/element.sysml:32), critique-policy.toml:12's MUST-be-a-variant clause, guard critique's unknown-lens failure, the propriety package's own WHY comment (.engine/schema/propriety/propriety.sysml:4), and the adversarial-panel roster at .engine/processes/adversarial-panel-review.sysml:20
    part d0527 : Decision {
        :>> id = "0203bc7f-12ef-4e09-b45b-06b27c6b2bd1";
        :>> title = "projectCritiqueLenses: critique-policy.toml names a project schema package that supplies lens variants and guard critique checks names against it - or the onboarding path only documents the rule - or nothing changes";
        :>> createdAt = "2026-09-19";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::proposed;
        :>> context = "GH#78 (st131), filed from a murder-mystery project: the seven critique lenses are the requirement-quality canon and the policy file constrains a project's required set to that enum, so a plot beat can only be critiqued for testability. The engine already states the right rule inside the propriety package - a kind the vocabulary cannot express is a new non-core package, never a forced fit - and nothing on the onboarding path points at it, so a domain project must choose between abusing a lens and a name guard critique fails loud on.";
        :>> decision = "RESEARCH: read before asking - element.sysml:32, critique-policy.toml, propriety.sysml:4's WHY comment, guard critique, project-onboarding step 3. OPTION A (recommended): critique-policy.toml gains lensPackage naming a non-core schema package whose enum members are accepted lens names alongside CritiqueLens; guard critique and critique-coverage read both and still fail loud on a name neither declares; project-onboarding step 3 states the rule the propriety package discovered. COST: two readers change (guard critique, critique-coverage) and a policy key is added; a project lens is a project's judgment, so a lens named by no critique question is a project defect the engine cannot see. OPTION B: document the path only - onboarding step 3 and the policy header say a lens the canon cannot express is a new non-core package - and leave the readers as they are. COST: the rule is stated and still not usable, because guard critique reads one enum; the project writes the package and cannot gate on it. OPTION C: nothing changes. COST: the murder-mystery case restated - a domain project critiques a plot for testability or turns critique off.";
        :>> rationale = "The propriety package is the precedent the engine already made for itself: when the canon could not express five propriety questions it added a non-core package rather than forcing the fit, and wrote the reason down where no adopter reads it. A is the same move offered to projects, with the guard still deciding what is declared. B leaves a documented rule that the tool refuses to honour, which is the shape of defect an adopter trusts least. The frozen surface is untouched under every option; the reporter asked for exactly that.";
        :>> consequences = "If A: a schema-reader change in keel-guards, a policy key, an onboarding step sentence, and a test with a fake lens package passing and an undeclared lens failing. If B: doc-sync only. If C: the story is dispositioned with the human's words.";
    }
    #DerivedFrom dependency from d0527 to st131;
}
