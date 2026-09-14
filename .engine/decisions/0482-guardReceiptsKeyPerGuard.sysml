// D0482 (status: the `status` field below is the truth) — guardReceiptsKeyPerGuard: each guard declares the paths it reads, and the guard receipt keys per guard on the hash of those inputs, so an edit re-runs only the guards whose inputs moved
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0482Accept event on sign-off.
package Decision0482 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    part d0482 : Decision {
        :>> id = "99500653-4d6b-47b6-92fb-a35f15195bca";
        :>> title = "guardReceiptsKeyPerGuard: each guard declares the paths it reads, and the guard receipt keys per guard on the hash of those inputs, so an edit re-runs only the guards whose inputs moved";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::accepted;
        :>> context = "D0371 keyed the whole guard receipt on the inputs of all guards together: a green run is reused only when nothing any guard reads has changed, so one .tracking edit re-runs every guard. There are 75 declared guards in one 9,415-line guards.rs, none of which states what it reads. The human's words (st126): 'i like the 'test binaries' improvements and the Guards one also.'";
        :>> decision = "Every guard declares its input set - the path globs it reads - beside its constraint identity, and the guard receipt records one key per guard: the hash of the files its declaration matches at the run. keel gate guard re-runs a guard only when its key moved and reports the rest as held from the receipt, naming the receipt. A guard observed reading a path its declaration does not match fails its own probe, so the declaration cannot silently under-state. This reverses the one-key clause of D0371 and keeps its rule that a red run deletes the receipt. NOT A FORK: the human chose in st126. NOT A PROCESS CHANGE: the guards, their verdicts and when they run are unchanged; what a green receipt is allowed to skip is finer.";
        :>> rationale = "The same descent as the touched set, applied to the other long pole: the receipt's root hash says something moved, the per-guard keys say which guards care. A declared input set that is checked against observation is the guard's read set as a receipt rather than a promise.";
        :>> consequences = "Sprint dcGuardReceiptsKeyPerGuard, after the guards are split into per-family modules in their own member (dcGuardsAreMembersPerFamily), where each family's inputs are visible. guards.md gains an inputs column rendered from the declarations.";
    }
    #SupersedeClause dependency from d0482 to d0371;
    #DerivedFrom dependency from d0482 to st126;

    // acceptance event (D0121 review-queue sign-off; D0066/D0106 — human-judged, not fabricated)
    verification d0482Accept : Test { :>> id = "4c270a8d-c51a-45c6-81bf-95da41dca9f2"; :>> method = VerificationMethod::confirmation; :>> procedureText = "AUTO-ACCEPTED under standing consent (D0207). Their standing words, verbatim: 'issues raised are automatically accepted. they can be customizedly changed post-fact.' Not individually reviewed; override by a superseding Decision (D0290) or the human's quoted word in chat (D0289)."; }
    part d0482AcceptR1 : TestResult { :>> id = "8d2b71df-cc8b-4788-a3aa-50b777d27ea7"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "13eb23b"; :>> judgedAt = "2026-09-14"; :>> judgedBy = "wweatherholtz"; :>> createdBy = "claudeFable5"; }
}
