// D0530 (PROPOSED — NOT YET ACCEPTED) — analysisSuspicion: an analysis element goes suspect when an architecture element it analyses changed after its last reassessment, following marker edges the schema declares as suspicion-carrying - or a project computes it in its own view - or nothing changes
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0530Accept event on sign-off.
package Decision0530 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    // RESEARCH: the suspicion computation and its edge set (relationships.sysml:28-30: satisfy, verify, :>, allocate, dependsOn), EngineSafety's element types, obsidian-selfsync D0017 carried as st102, and the reporter's asked edge set (#Analyzes, #RealizedBy, #Senses direct; #Explains, #Mitigates, #Inverts, #Enforces transitive to fixpoint; hazards and losses never flagged by an architecture change)
    part d0530 : Decision {
        :>> id = "688fa0a6-67f6-4414-8de0-1eba5d0cb67e";
        :>> title = "analysisSuspicion: an analysis element goes suspect when an architecture element it analyses changed after its last reassessment, following marker edges the schema declares as suspicion-carrying - or a project computes it in its own view - or nothing changes";
        :>> createdAt = "2026-09-19";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::proposed;
        :>> context = "GH#66 (st102, re-ingested as st143): a downstream STPA whose control structure is a first-class model with 100+ marker edges into the architecture has no signal when a controller changes under an analysed UCA, so the analysis silently diverges from the code it abstracts. The ask names a reassessedAtCommit stamp on analysis elements, propagation to fixpoint along named markers, and surfacing in orient, the safety view and show suspect.";
        :>> decision = "RESEARCH: read before asking - relationships.sysml:28-30, the suspect lens, EngineSafety, st102's edge list and its hazard exclusion. OPTION A (recommended): a metadata def may declare carriesSuspicion (direct or transitive), EngineSafety's analysis elements gain reassessedAtCommit [0..1], and the suspect computation follows every declared suspicion-carrying edge from a changed element to the analysis elements downstream of it, stopping at any element whose stamp is later than the change; hazards and losses are reached by no such edge. COST: a schema addition under CR, a per-element stamp the analyst must write when they reassess (a new authoring obligation, D0054), and a suspect lens whose reasons now name a marker path. OPTION B: the project computes it in a TOML view over its own markers, with no schema change and no stamp - the view lists analysis elements whose architecture changed since the analysis element's own last commit. COST: the signal is per project, absent from orient and show suspect, and the element's last commit is a weaker proxy than a reassessment stamp. OPTION C: nothing changes. COST: the STPA diverges from the code with no signal, in the project that carries a 25-requirement safety analysis.";
        :>> rationale = "The engine's suspicion already propagates along five edge kinds because those are the kinds a change invalidates through; a project's declared analysis edges are the same relation in a domain the core did not name, and D0139(B) already lets the project declare them. A lets the declaration say what it means for suspicion, which is the one property the engine needs to compute the rest, and keeps the reassessment an explicit human act rather than a commit-date guess. B is available today and is the right interim for the reporter, which is why it is named rather than dismissed. The stamp is a real authoring cost and the COST says so.";
        :>> consequences = "If A: a CR with the schema addition and marker attribute, the propagation with a test where a controller edit suspects its UCAs and not its hazards, and the suspect lens naming the path. If B: a view template shipped with the stpa skill and no engine change. If C: the story is dispositioned with the human's words.";
    }
    #DerivedFrom dependency from d0530 to st102;
    #DerivedFrom dependency from d0530 to st143;
}
