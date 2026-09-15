// OBLIGATION (auto-recorded, D0176/K7): a control was overridden or yielded - an act that happened, so discoveredInField is true (dcControlCensusByActorKind); a human review
// discharges it (triage with a #Resolves edge). One file per fact so recording never deadlocks
// on the file being repaired.
package Obligationda53d7d0 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineRelationships::*;

    part obligationda53d7d0 : Issue {
        :>> id = "da53d7d0-636c-4c06-854a-379835e35b6e";
        :>> title = "turn gate yielded while red - the tree needs a correction pass";
        :>> createdAt = "2026-09-15"; :>> createdBy = "claudeFable5";
        :>> description = "The Stop hook's second red pass yielded (loop avoidance). Session: b43aa4fc-84f7-4c3d-bc35-61eccafbbda8. First problem at yield: keel gate guard: [ceremony] sprint721_modularMembersArchitecture: Retro gate recorded without avoidable-issue scan evidence (issue011). Discharge: make the tree green and triage this obligation with a #Resolves edge.";
        :>> discoveredInField = true;
        :>> severity = Severity::Low;
    }
    // Triage 2026-09-15: the red was the retro Test authored without the issue011 scan wording, accepted
    // at the write and caught one commit later by a guard whose line named the result; the tree went
    // green by hand (the wording added to the Test, the duplicate result removed). Resolves with the
    // scan checked at the write itself (issue566).
    #Resolves dependency from dcRetroScanIsCheckedAtTheWrite to obligationda53d7d0;
}
