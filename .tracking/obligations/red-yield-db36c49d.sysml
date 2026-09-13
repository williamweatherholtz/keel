// OBLIGATION (auto-recorded, D0176/K7): a control was overridden or yielded - an act that happened, so discoveredInField is true (dcControlCensusByActorKind); a human review
// discharges it (triage with a #Resolves edge). One file per fact so recording never deadlocks
// on the file being repaired.
package Obligationdb36c49d {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineRelationships::*;

    part obligationdb36c49d : Issue {
        :>> id = "db36c49d-029c-41cb-93bb-9b7c9b8fe6c0";
        :>> title = "turn gate yielded while red - the tree needs a correction pass";
        :>> createdAt = "2026-09-13"; :>> createdBy = "claudeFable5";
        :>> description = "The Stop hook's second red pass yielded (loop avoidance). Session: b43aa4fc-84f7-4c3d-bc35-61eccafbbda8. First problem at yield: keel gate guard: [retro-backlog] .tracking/delivery/sprint698_decisionScaffoldingIndexesItsEdgesOnce.sysml: the retro records findings but this commit ADDS no tracked item the retro names (it names d0131, none of which this commit adds) — a finding must become a tracked, prioritized item in the SAME commit, or the retro must say why none is needed (D0131; issue335: naming an existing task is what every retro does, and is not tracking a finding). Discharge: make the tree green and triage this obligation with a #Resolves edge.";
        :>> discoveredInField = true;
        :>> severity = Severity::Low;
    }
    #Resolves dependency from dcDecisionScaffoldingIndexesItsEdgesOnce to obligationdb36c49d;
}
