// OBLIGATION (auto-recorded, D0176/K7): a control was overridden or yielded - an act that happened, so discoveredInField is true (dcControlCensusByActorKind); a human review
// discharges it (triage with a #Resolves edge). One file per fact so recording never deadlocks
// on the file being repaired.
package Obligation1ab52489 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineRelationships::*;

    part obligation1ab52489 : Issue {
        :>> id = "1ab52489-6168-419f-bd4e-98914f46517a";
        :>> title = "turn gate yielded while red - the tree needs a correction pass";
        :>> createdAt = "2026-09-15"; :>> createdBy = "claudeFable5";
        :>> description = "The Stop hook's second red pass yielded (loop avoidance). Session: b43aa4fc-84f7-4c3d-bc35-61eccafbbda8. First problem at yield: keel gate guard: [process-change] locked file(s) changed (.engine/skills/test-verify/SKILL.md) with NO co-committed process-change Decision (a #ProspectiveChange/#SafetyChange-marked .engine/decisions/*.sysml) and NO co-committed sprint record chartered by a marked Decision the human has ACCEPTED (D0465). HARD LOCK: process definitions (D0070) AND the enforcement surface — guard source, hook config, CI workflows (D0209 clause 2) — may change only with a human-signed Decision, because a silently self-modified control is the issue236 self-modification class. Record one with `keel record decision --process-change ...` (the flag emits the marker; issue213), or land the edit in a sprint whose #CharteredBy names the accepted Decision.. Discharge: make the tree green and triage this obligation with a #Resolves edge.";
        :>> discoveredInField = true;
        :>> severity = Severity::Low;
    }
    // Triage: the locked-file edit (the test-verify skill's stems row, issue530) is authorised by D0494,
    // a process-change Decision recorded and co-committed with the edit, held for the human (D0337).
    #Resolves dependency from d0494 to obligation1ab52489;
}
