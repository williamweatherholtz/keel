// D0488 (status: the `status` field below is the truth) — projectProcessesResolveFromTheProjectTree: keel advance and the step-check resolver read a process declared under a project's .tracking/processes/ exactly as one under .engine/processes/
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0488Accept event on sign-off.
package Decision0488 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    // RESEARCH: Read 2026-09-15. keel-cli/src/cursor.rs:172-173 - resolve_process walks root/.engine/processes only; the refusal at cursor.rs:202 says a name is 'neither a delivery file under .tracking/delivery/ nor a process under .engine/processes/'. The step-check-resolves guard (D0434) is scoped to ProcessStep parts under .engine/processes/ (guards.md row step-check-resolves). .engine/contracts/activation.toml adopts processes by name under [processes] with no path. No project on this tree declares a process under .tracking/; the Need nProcessIsReadNotHardcoded (accepted 2026-09-14) requires a fixture project with one the engine has never seen to run with zero engine edits, which the walk at cursor.rs:173 makes impossible today.
    // #ProspectiveChange (D0070) - this Decision governs a process/schema change; the co-committed
    // change rides the process-change guard's hard lock.
    #ProspectiveChange part d0488 : Decision {
        :>> id = "7d3e1fba-712f-43e3-8052-88555360fd72";
        :>> title = "projectProcessesResolveFromTheProjectTree: keel advance and the step-check resolver read a process declared under a project's .tracking/processes/ exactly as one under .engine/processes/";
        :>> createdAt = "2026-09-15";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::accepted;
        :>> context = "st126: 'how to hook into custom processes that have absolute discipline?' and 'we can read plain sysml processes, interpret them, add guards'. The engine reads its OWN processes from .engine/processes/ and enforces them through keel advance (D0435/D0436) and the step-check guard (D0434); a process a project authors in its own tree is not found by the resolver at all (cursor.rs:172-173), so today 'a project authors a process in plain SysML and the engine enforces it' is true only for keel. This is the first of three one-clause Decisions (D0303) for the process hook; the step binding and the gate rung depend on it. What it means for the reader: after this, a project writes a process file in its own .tracking/processes/ and keel advance treats it like keel's own - refuses a red step, moves past a green one - with no change to keel.";
        :>> decision = "The process resolver used by keel advance, by the step-check guard and by the process-gates rung reads process definitions (action <name> : Process) from BOTH .engine/processes/ and the project's .tracking/processes/, through one function; a name declared in both is refused with both paths named, never shadowed. The step-check-resolves guard extends its scope to the project directory with the same rule. Adoption stays where it is: a project process is enforced once .engine/contracts/activation.toml names it (D0138/D0164), so reading a process is not the same as adopting it.";
        :>> rationale = "The Need's measurement is a fixture process the engine has never seen running with zero edits to keel; a resolver that only walks .engine/processes/ cannot meet it, and a copy of the walk for the project directory would be a second resolver to drift (D0105). Shadowing is refused rather than resolved by precedence because a project that redefines refine or standup would silently change what keel's own gates mean. The adoption boundary is kept so a process file dropped into a tree does not enforce anything until the project says so - the same rule keel applies to itself.";
        :>> consequences = "Held proposed for the human (a process change: what keel advance enforces widens to a second directory). On acceptance: cursor.rs resolve_process gains the second root and the collision refusal; step-check-resolves scans both directories; a fixture project under keel-cli/tests with one two-step process is the D0388 pair (refused at the red step, moved after a pass); the process-applicability and activation-manifest guards accept a project process name. Chartered work: srProjectProcessIsReadFromItsTree and ssrStepBindsToAProjectTest (modular-members-requirements.sysml) derive from this Decision. The step binding kind and the gate rung are the two Decisions that depend on this one.";
    }
    #DerivedFrom dependency from d0488 to st126;

    // acceptance event (D0121 review-queue sign-off; D0066/D0106 — human-judged, not fabricated)
    verification d0488Accept : Test { :>> id = "2e20e849-0aa7-47d1-8ddb-71ea37032ddc"; :>> method = VerificationMethod::confirmation; :>> procedureText = "recorded from chat - their words, verbatim: “Accept the three process-hook clauses in order - Read, Bind, Rung. [x] Accept: the process resolver reads both .engine/processes and the project's .tracking/processes through one function; a name in both is refused; adoption stays with the activation contract (recommended)”"; }
    part d0488AcceptR1 : TestResult { :>> id = "55b1ab51-c844-4c4f-803b-a45789e1195b"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "85613c8"; :>> judgedAt = "2026-09-15"; :>> judgedBy = "wweatherholtz"; :>> createdBy = "claudeFable5"; }
}
