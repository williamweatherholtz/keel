// OBLIGATION (auto-recorded, D0176/K7): a control was overridden or yielded - an act that happened, so discoveredInField is true (dcControlCensusByActorKind); a human review
// discharges it (triage with a #Resolves edge). One file per fact so recording never deadlocks
// on the file being repaired.
package Obligation664abc60 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineRelationships::*;

    part obligation664abc60 : Issue {
        :>> id = "664abc60-eb38-4a9e-af26-6b59e3f94ef9";
        :>> title = "turn gate yielded while red - the tree needs a correction pass";
        :>> createdAt = "2026-09-16"; :>> createdBy = "claudeFable5";
        :>> description = "The Stop hook's second red pass yielded (loop avoidance). Session: b43aa4fc-84f7-4c3d-bc35-61eccafbbda8. First problem at yield: keel gate guard: [retro-backlog] .tracking/delivery/sprint727_scriptProbesRunBeforeCommit.sysml: the retro records findings but this commit ADDS no tracked item the retro names (it names issue011, issue574, issue453, issue572, d0054, none of which this commit adds) — a finding must become a tracked, prioritized item in the SAME commit, or the retro must say why none is needed (D0131; issue335: naming an existing task is what every retro does, and is not tracking a finding) [instruments-declared] scripts/script_probes.py: produces a number, verdict or figure but no `Sensor` item declares it - an undeclared measure is outside the computed control structure, so no analysis reaches it and no verification case can verify it (D0361/D0363). Author a Sensor, or argue the exclusion with a `not-an-instrument:` line in the file. Discharge: make the tree green and triage this obligation with a #Resolves edge.";
        :>> discoveredInField = true;
        :>> severity = Severity::Low;
    }
    // Triage: the red was sprint 727's retro saying `not tracked` - a phrase the retro-backlog guard does not
    // read as a justification (textscan.rs RETRO_NO_ITEM_JUSTIFICATIONS) - beside the runner's missing
    // instrument line; both corrected in the commit that carries D0496, the process-change Decision this
    // sprint delivers, held for the human (D0337).
    #Resolves dependency from d0496 to obligation664abc60;
}
