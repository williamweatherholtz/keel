// OBLIGATION (auto-recorded, D0176/K7): a control was overridden or yielded - an act that happened, so discoveredInField is true (dcControlCensusByActorKind); a human review
// discharges it (triage with a #Resolves edge). One file per fact so recording never deadlocks
// on the file being repaired.
package Obligation637916a4 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineRelationships::*;

    part obligation637916a4 : Issue {
        :>> id = "637916a4-4406-447d-b095-8ae81655e1b0";
        :>> title = "turn gate yielded while red - the tree needs a correction pass";
        :>> createdAt = "2026-09-16"; :>> createdBy = "claudeFable5";
        :>> description = "The Stop hook's second red pass yielded (loop avoidance). Session: b43aa4fc-84f7-4c3d-bc35-61eccafbbda8. First problem at yield: keel gate guard: [process-change] locked file(s) changed (keel-cli/src/guards.rs) with NO co-committed process-change Decision (a #ProspectiveChange/#SafetyChange-marked .engine/decisions/*.sysml) and NO co-committed sprint record chartered by a marked Decision the human has ACCEPTED (D0465). HARD LOCK: process definitions (D0070) AND the enforcement surface — guard source, hook config, CI workflows (D0209 clause 2) — may change only with a human-signed Decision, because a silently self-modified control is the issue236 self-modification class. Record one with `keel record decision --process-change ...` (the flag emits the marker; issue213), or land the edit in a sprint whose #CharteredBy names the accepted Decision. [issues] issue577: its resolver `dcWorktreeScratchIsPerProcess` never names it (title, DoD text or decision) - an edge that exists is not a triage; a resolver that will close the issue says so (issue333/D0304). Name the issue in the resolver's DoD, or re-triage to the item that resolves it.. Discharge: make the tree green and triage this obligation with a #Resolves edge.";
        :>> discoveredInField = true;
        :>> severity = Severity::Low;
    }

    // Both reds were sprint 729's own tree mid-flight: the seven guards.rs test-site rewrites needed
    // their co-committed process-change record, and issue577's resolver DoD now names it. D0499 is
    // that record, and it names this obligation; the human's acceptance of D0499 is the review.
    #Resolves dependency from d0499 to obligation637916a4;
}
