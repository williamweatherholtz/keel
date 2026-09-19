// D0528 (PROPOSED — NOT YET ACCEPTED) — projectCeremonyChain: keel advance reads the sprint's gate chain from the gates the sprint file declares, each named for a Process the workflow declares - or the six-gate chain stays baked into advance - or nothing changes
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0528Accept event on sign-off.
package Decision0528 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    // RESEARCH: keel advance's fixed six names (members/keel-process/src/cursor.rs, orient.rs:299-314), agile-workflow.sysml's planning process at line 90 beside refinement (:42) and standup (:120), the sprint-planning skill, D0049's autonomous-gate framing, and the reporter's rename experiment where Standup dropped out and PlanningGate was not picked up
    part d0528 : Decision {
        :>> id = "cf3b72d9-198f-4fb8-a97f-4b76fe3fe9e0";
        :>> title = "projectCeremonyChain: keel advance reads the sprint's gate chain from the gates the sprint file declares, each named for a Process the workflow declares - or the six-gate chain stays baked into advance - or nothing changes";
        :>> createdAt = "2026-09-19";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::proposed;
        :>> context = "GH#82 (st159): a project renamed its standup gate to a planning gate and keel advance silently dropped the step - omission is supported, substitution is not - even though agile-workflow.sysml already declares planning as a Process with a skill. A solo author working evening sittings earns nothing from standup and cannot put planning in its place.";
        :>> decision = "RESEARCH: read before asking - cursor.rs's name matching, orient.rs:299-314, agile-workflow.sysml:42,90,120, D0049, the reporter's two advance transcripts. OPTION A (recommended): advance derives a sprint's chain from the gates its sprint file declares, in declaration order, each gate matched to a Process the active workflow declares by the name it carries (RefineGate to refinement, PlanningGate to planning); a gate naming no declared Process is refused with the names available. COST: the chain is per sprint, so two sprints in one project may run different ceremonies and orient must show which; sprint-closure and the delegated-ceremony briefs that name Refine..Retro read the chain instead of assuming it. OPTION B: keep the six names baked and document that a project omits, never substitutes. COST: planning exists as a Process nothing can gate on, and every domain project inherits a software team's standup. OPTION C: nothing changes. COST: the reporter's rename keeps silently dropping a step, which is a gate that disappears without a refusal - the D0098 class.";
        :>> rationale = "The engine already treats a workflow as authored text and the gate as its verify-linked Tests (invariant 6); a fixed name list inside advance is the one place the ceremony is code rather than model, and the rename experiment proves it. A puts the chain back in the model where the process declarations already are, with a refusal for a gate that names nothing, so a typo cannot drop a step the way it does today. B is honest but leaves a declared Process unusable. The silent drop under C is the worse defect than either alternative because it removes a control without saying so.";
        :>> consequences = "If A: cursor.rs and orient.rs read the chain from the sprint file; a test with a PlanningGate substituted for the standup gate advances through Planning, and a gate naming no Process is refused; the sprint-planning skill states that planning may be a gate. If B: a doc line in the skill and process file. If C: the story is dispositioned with the human's words.";
    }
    #DerivedFrom dependency from d0528 to st159;
}
