// OBLIGATION (auto-recorded, D0176/K7): a control was overridden or yielded - an act that happened, so discoveredInField is true (dcControlCensusByActorKind); a human review
// discharges it (triage with a #Resolves edge). One file per fact so recording never deadlocks
// on the file being repaired.
package Obligation87f79374 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineRelationships::*;

    part obligation87f79374 : Issue {
        :>> id = "87f79374-ac59-40e0-968b-53b934bdc9cd";
        :>> title = "turn gate yielded while red - the tree needs a correction pass";
        :>> createdAt = "2026-09-15"; :>> createdBy = "claudeFable5";
        :>> description = "The Stop hook's second red pass yielded (loop avoidance). Session: b43aa4fc-84f7-4c3d-bc35-61eccafbbda8. First problem at yield: keel gate guard: [process-change] locked file(s) changed (.engine/skills/test-verify/SKILL.md) with NO co-committed process-change Decision (a #ProspectiveChange/#SafetyChange-marked .engine/decisions/*.sysml) and NO co-committed sprint record chartered by a marked Decision the human has ACCEPTED (D0465). HARD LOCK: process definitions (D0070) AND the enforcement surface — guard source, hook config, CI workflows (D0209 clause 2) — may change only with a human-signed Decision, because a silently self-modified control is the issue236 self-modification class. Record one with `keel record decision --process-change ...` (the flag emits the marker; issue213), or land the edit in a sprint whose #CharteredBy names the accepted Decision. [issues] obligation1ab52489: untriaged — no #Resolves edge (D0077; link a resolving action or Decision). Discharge: make the tree green and triage this obligation with a #Resolves edge.";
        :>> discoveredInField = true;
        :>> severity = Severity::Low;
    }
    // Triage: the same red as obligation1ab52489, seen by the verifier subagent's stop - the locked-file
    // edit (test-verify stems row, issue530) is authorised by D0494, co-committed and held for the human.
    #Resolves dependency from d0494 to obligation87f79374;
}
