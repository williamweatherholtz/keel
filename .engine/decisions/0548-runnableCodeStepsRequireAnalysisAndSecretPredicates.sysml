// D0548 (PROPOSED — NOT YET ACCEPTED) — runnableCodeStepsRequireAnalysisAndSecretPredicates: static analysis and secret scanning are either required checkedBy predicates on every runnable code step in the frozen meta-process, or left as project habits
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0548Accept event on sign-off.
package Decision0548 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    part d0548 : Decision {
        :>> id = "505fb714-f4e9-4b21-884b-7402affb0345";
        // RESEARCH: three rounds of a nine-persona adversarial panel over four revisions of the direction artifact, every finding verified against the repository at f365cb7a; the two landscape spikes D0540 (94 rows) and D0541 (122 user threads); docs/reviews/runner-direction-panel-2026-09-22.md holds the reports and the finding-to-resolution maps.
        :>> title = "gatePredicates: static analysis and secret scanning are either required checkedBy predicates on every runnable code step in the frozen meta-process, or left as project habits";
        :>> createdAt = "2026-09-22";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::proposed;
        :>> context = "A gate is only the predicate someone bound. The runner landscape's thread D5 records a change that passed a green review and a green pipeline and handed an attacker a ticketing system, because nothing had been bound for it; keel's own schema allows a step with no check, and 13 of 191 process steps bind one today. Revision 4 of the panelled direction (D0542) makes runnable computed from inputs, outputs and checkedBy, so a required predicate is one more thing a step must declare before keel run will run it.";
        :>> decision = "OPTION A (recommended): the meta-process that defines processes requires two named predicates - a static-analysis check and a secret-scanning check - on every runnable step whose declared outputs include code; a step without them is not runnable, and the requirement lives in the frozen layer, not the project's habit. OPTION B: each project binds what it likes; the two checks are habits, and a green step may ship the D5 class.";
        :>> rationale = "The predicate belongs in the frozen layer because a habit is exactly what the green-but-breached case had; the cost is one more line per code step, which is authoring friction D0054 counts as the first risk. A is recommended because the direction's whole premise is that done means a check keel ran, and a class of green-but-breached the human can name is the case against leaving it to habit.";
        :>> consequences = "Under A: the slice-one schema CR names the two predicates and validate reports a runnable code step that lacks them; the first code step defined under keel run carries them. Under B: nothing is added and the gate set is whatever a process author binds. COST under A: one more predicate line per runnable code step and two named checks in the frozen layer. COST under B: a green step may ship the green-but-breached class the landscape recorded.";
    }
}
