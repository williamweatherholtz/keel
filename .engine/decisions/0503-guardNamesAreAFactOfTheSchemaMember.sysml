// D0503 (status: the `status` field below is the truth) — The enforced guard-name list is a fact of the schema member, locked where it lands
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0503Accept event on sign-off.
package Decision0503 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    // #ProspectiveChange (D0070) - this Decision governs a process/schema change; the co-committed
    // change rides the process-change guard's hard lock.
    #ProspectiveChange part d0503 : Decision {
        :>> id = "e2a2391c-be86-46ef-b3bb-4662e5cf62d2";
        :>> title = "The enforced guard-name list is a fact of the schema member, locked where it lands";
        :>> createdAt = "2026-09-16";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::accepted;
        :>> context = "Sprint 732 is the third D0479 extraction: the view layer (view/ and its ten submodules, govern, pm, arch and the control-proof census) becomes member keel-view. One reference kept view above guards after sprint 717 broke the cycle: control_proof::census read GUARD_NAMES from keel-cli/src/guards.rs, and a member cannot read a crate above it (D0479: a member's Cargo dependency list is its declared read set). GUARD_NAMES is a fact the binary states - keel version's guards line, --help, keel show controls - not a guard, so it belongs with the CLI facts in keel-schema. guards.rs is an enforcement source under the keystone lock (GUARD_SOURCE_FILES, D0209 clause 2), so moving a declaration out of it is a locked edit whether or not a guard changes, and the lock cannot tell a declaration move from a disarm. The stpa-diagram process file and its skill name the renderer by file path (keel-cli/src/view/stpa_diagram.rs), and a process file is a locked definition (D0070), so that path's doc-sync rides the same Decision. D0479 governs the move but carries no marker; sprint 718 met the same lock and recorded D0486.";
        :>> decision = "GUARD_NAMES is declared in members/keel-schema/src/guard_names.rs and re-exported by keel-cli/src/guards.rs under its old path, so every caller and every test that holds a name to a dispatch arm reads the same list. guard_names.rs joins GUARD_SOURCE_FILES: removing a name there is the same locked edit as deleting its arm, because either silently disarms a guard. The stpa-diagram process and skill name the renderer's home as members/keel-view/src/view/stpa_diagram.rs. No guard's name, dispatch, severity or gate set changes; no step, gate or output of any process moves.";
        :>> rationale = "One home per fact (D0105): the list was one file with the arms only because both were in keel-cli; once the view layer needs the list and cannot see the arms, the list is the fact and the arms are its enforcement, and the fact goes to the facts member as the CLI surface did (D0271). The lock follows the list rather than staying with the arms because every_enforced_guard_dispatches holds each name to a run_one arm - a name deleted from the list disarms without touching guards.rs (issue236's class), which is exactly what the keystone lock exists to make a signed edit. It is marked process-change because the file is locked and the guard cannot tell a layout move from a disarm. MEASURED: keel version reports guards: 75 before and after the move; keel --help is byte-identical to the render at 397e49c; enforcement_surface_covers_every_guard_source holds with the third path.";
        :>> consequences = "Adding a guard edits two locked files instead of one: its name in guard_names.rs and its arm in guards.rs, both under the same lock. dcGuardsAreMembersPerFamily meets the lock a third time when the arms move to keel-guards; the list stays in keel-schema and the families assemble against it. The pattern is now recorded twice: an extraction sprint that touches a locked source carries its own marked Decision even though D0479 governs the move, and the next extraction copies it rather than rediscovering it. This Decision depends on D0479 and reverses nothing.";
    }
    #DependsOn dependency from d0503 to d0479;

    // acceptance event (D0121 review-queue sign-off; D0066/D0106 — human-judged, not fabricated)
    verification d0503Accept : Test { :>> id = "08ba402c-44c1-4035-9a2d-88bffb68237c"; :>> method = VerificationMethod::confirmation; :>> procedureText = "recorded from chat - their words, verbatim: “Accept all decisions as recommended. Proceed to release and fix downstream” WARN: read-back - the quoted words do not name d0503 (D0201 B); recorded as given (D0423)."; }
    part d0503AcceptR1 : TestResult { :>> id = "e9138c4e-4ac5-41c5-b2fb-834059465c98"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "4d00c1bc"; :>> judgedAt = "2026-09-18"; :>> judgedBy = "wweatherholtz"; :>> createdBy = "claudeFable5"; }
}
