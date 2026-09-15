// D0483 (status: the `status` field below is the truth) — workspaceLayeringIsGuarded: a guard reads the Cargo manifests and refuses a member dependency that points up the declared layer order
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0483Accept event on sign-off.
package Decision0483 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    // #ProspectiveChange (D0070) - this Decision governs a process/schema change; the co-committed
    // change rides the process-change guard's hard lock.
    #ProspectiveChange part d0483 : Decision {
        :>> id = "200aa2de-0134-4cfe-9d26-8d41545c4a0d";
        :>> title = "workspaceLayeringIsGuarded: a guard reads the Cargo manifests and refuses a member dependency that points up the declared layer order";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::accepted;
        :>> context = "D0479 orders the members in layers computed from the module graph. Cargo refuses a cycle but accepts any acyclic edge, so a later convenience dependency from a leaf member up to view or serve would re-couple what the pass decoupled and nothing would say so until rebuild times crept back.";
        :>> decision = "A guard, workspace-layering, reads every member's Cargo.toml and the layer order declared in .engine/contracts/workspace-layers.toml, and fails naming any dependency edge from a lower layer to a higher one. Known-positive: a fixture manifest with keel-git depending on keel-view fails naming both. Known-negative: the real workspace at the commit that adds the guard passes. This is an enforcement change and is held for the human's word (D0209).";
        :>> rationale = "The decoupling is only durable if it is a control, not a convention (D0047). Cargo enforces acyclicity; the layer order is the stronger fact and needs its own reader.";
        :>> consequences = "Sprint dcWorkspaceLayeringIsGuarded, after the leaf members exist. guards.md gains the row; the guard count in keel version moves by one.";
    }
    #DerivedFrom dependency from d0483 to st126;

    // acceptance event (D0121 review-queue sign-off; D0066/D0106 — human-judged, not fabricated)
    verification d0483Accept : Test { :>> id = "0fc45fe4-ad53-4a6d-afe1-ef1655c09f42"; :>> method = VerificationMethod::confirmation; :>> procedureText = "Said in chat 2026-09-14 over the twenty-eighth publish of the decision brief, which carried fifteen held asks; one instruction covers the fourteen non-forks, d0483 among them. The fork d0466 is not covered - it needs an option letter and stays held. - their words, verbatim: “Accept all proposals. Proceed to design with needs, etc.” WARN: read-back - the quoted words do not name d0483 (D0201 B); recorded as given (D0423)."; }
    part d0483AcceptR1 : TestResult { :>> id = "477ee50f-8ede-4d19-8193-5f2f30cccc18"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "d110d0a"; :>> judgedAt = "2026-09-14"; :>> judgedBy = "wweatherholtz"; :>> createdBy = "claudeFable5"; }
}
