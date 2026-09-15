// D0489 (status: the `status` field below is the truth) — aStepBindsToAProjectTest: a ProcessStep's checkedBy gains the binding kind test:<name>, a Test in the project's own model whose latest result decides the step
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0489Accept event on sign-off.
package Decision0489 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    // RESEARCH: Read 2026-09-15. cursor.rs:228-256 evaluates a step's checkedBy in three ways and no other: gate:<phase> is NeedsRun (a per-sprint check), a name guards::run_one knows is a compiled guard evaluated now, and a name in view::check's rules is a declared rule; anything else is Verdict::Unresolved. A project cannot add a compiled guard (it cannot edit keel) and a declared rule under .engine/rules/ is an engine file, so the only check a project fully owns - a Test it declares and records results against - cannot bind a step today. The engine already treats a phase gate as its verify-linked Tests passing (CLAUDE.md invariant 6), and a TestResult's judgedAgainst is what claim-ancestry and audit history read to decide whether a pass is current.
    // #ProspectiveChange (D0070) - this Decision governs a process/schema change; the co-committed
    // change rides the process-change guard's hard lock.
    #ProspectiveChange part d0489 : Decision {
        :>> id = "00bbb381-3e82-424c-9dc2-509afecad548";
        :>> title = "aStepBindsToAProjectTest: a ProcessStep's checkedBy gains the binding kind test:<name>, a Test in the project's own model whose latest result decides the step";
        :>> createdAt = "2026-09-15";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::accepted;
        :>> context = "Given projectProcessesResolveFromTheProjectTree, a project's process is found; its steps still have nothing to bind to that the project controls. Compiled guards and declared rules are keel's; a project's own check is a Test with results. This is the second of three one-clause Decisions for the process hook (D0303). What it means for the reader: a project writes 'this step is checked by our smoke test' in its process file, records the test's result as it does any result, and keel advance refuses the step until that result is a pass at the current commit.";
        :>> decision = "A ProcessStep's checkedBy may read test:<name>, where <name> is a Test declared anywhere in the project's model. The step is GREEN when that Test's latest TestResult whose judgedAgainst is an ancestor of HEAD has outcome pass and no later result on it is red; RED with the result named otherwise; UNRESOLVED (a hard step-check-resolves failure naming the test) when no such Test exists. One resolver evaluates the binding for keel advance and for the process-gates rung.";
        :>> rationale = "A Test with a recorded result is the one check a project owns end to end, and it is already how keel decides its own phase gates; binding a step to it makes a project's discipline the same mechanism as keel's rather than a weaker imitation. Reading the latest ancestor-of-HEAD result rather than the newest result by date is what keeps a pass recorded on an abandoned commit from greening a step on this one (the D0387 rule for receipts, applied to steps). An unknown test name is hard, not a judgment step, because a step that silently degrades to unchecked is the defect the whole hook exists to remove.";
        :>> consequences = "Held proposed for the human (a process change: a new binding kind a step can carry). On acceptance: cursor.rs gains the test: arm using the results reader keel-model already has; step-check-resolves accepts test:<name> that resolves and refuses one that does not; the fixture of projectProcessesResolveFromTheProjectTree binds its first step to a Test and is the D0388 pair - refused while red, moved after a pass recorded against HEAD. Chartered work: ssrStepBindsToAProjectTest (modular-members-requirements.sysml) derives from this Decision. Depends on projectProcessesResolveFromTheProjectTree; processGatesIsOneRung depends on this.";
    }
    #DerivedFrom dependency from d0489 to st126;
    #DependsOn dependency from d0489 to d0488;

    // acceptance event (D0121 review-queue sign-off; D0066/D0106 — human-judged, not fabricated)
    verification d0489Accept : Test { :>> id = "88a2a8ec-5c9a-4dae-8fe5-93372cb8b0d0"; :>> method = VerificationMethod::confirmation; :>> procedureText = "recorded from chat - their words, verbatim: “Accept the three process-hook clauses in order - Read, Bind, Rung. [x] Accept: a ProcessStep's check binding may read test:<name>; the latest TestResult on an ancestor of HEAD decides; an unresolvable name is a hard step-check failure (recommended)”"; }
    part d0489AcceptR1 : TestResult { :>> id = "1a82995b-df0f-458f-8f53-ae096b9699cd"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "85613c8"; :>> judgedAt = "2026-09-15"; :>> judgedBy = "wweatherholtz"; :>> createdBy = "claudeFable5"; }
}
