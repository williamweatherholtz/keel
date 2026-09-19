// D0505 (status: the `status` field below is the truth) — A locked guard source's test anchors repoint to the one repository-root walk, and the lock still fires for it
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0505Accept event on sign-off.
package Decision0505 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    // #ProspectiveChange (D0070) - this Decision governs a process/schema change; the co-committed
    // change rides the process-change guard's hard lock.
    #ProspectiveChange part d0505 : Decision {
        :>> id = "d2ecedd2-24de-4bea-82d8-6298f0219a83";
        :>> title = "A locked guard source's test anchors repoint to the one repository-root walk, and the lock still fires for it";
        :>> createdAt = "2026-09-17";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::accepted;
        :>> context = "Sprint 734 (dcOneRepoRootHelper, chartered by D0479, resolving issue557) puts the one repository-root walk for tests in keel_fs::test_support::repo_root and replaces every local fn repo_root copy and every cwd-relative test anchor in keel-cli/src and members/*/src with a call to it. One of the eight keel-cli/src anchors sits inside keel-cli/src/adherence.rs, a GUARD_SOURCE_FILES entry since D0209 clause 2: the test this_repo_yields_the_empty_prefix anchored its root at CARGO_MANIFEST_DIR joined with .. (the shape that breaks when a crate moves two levels down, sprints 714 and 718). The process-change guard reads any change to that file as a locked-surface change and refuses the commit without a co-committed marked Decision or a sprint chartered by an accepted marked Decision (D0465). D0479 charters the sprint but carries no marker, so the guard's second door is closed and this Decision is the first.";
        :>> decision = "The only edit inside keel-cli/src/adherence.rs is one line of test code: this_repo_yields_the_empty_prefix computes its root as keel_fs::test_support::repo_root() instead of PathBuf::from(env!('CARGO_MANIFEST_DIR')).join('..'). project_prefixes, the adherence audit, the enforcement-surface lock, every guard's dispatch, severity and gate set are unchanged. The same repoint lands in the seven other keel-cli/src anchors (attestation, main, process_cmd twice, reports, serve, workspace), none of them locked. The anchor scan no_member_test_anchors_on_a_cwd_relative_path in keel-cli/src/touched.rs widens its population from members/*/src to keel-cli/src as well and asserts touched.rs itself is in it, so the next D0479 extraction carries no anchor into its member.";
        :>> rationale = "The lock is on the file, not on the logic, by design: D0209 clause 2 chose a path predicate because a semantic one (only guard logic) cannot be checked by a guard. A test-only edit to a locked file therefore meets the same door as a logic edit, and the honest response is to record the marked Decision that says what changed rather than to route around the lock by leaving the one anchor in place. Leaving it would make adherence.rs the one keel-cli/src file the widened scan reports, and the scan would have to carve an exception for a locked file, which is the class of silent hole D0388 names. Recording the Decision costs one record; the exception would cost a permanent asymmetry in the control.";
        :>> consequences = "The lock has now fired for a test-only edit to a guard source, the first time since D0209 that a marked Decision governs no change to enforcement logic. This is the lock working as specified, not a defect: a reader of this Decision can hold the two-line diff of adherence.rs against the decision text in one glance. If the pattern recurs, the fork is whether the lock should exempt lines inside a cfg(test) module; that is a separate process-change Decision, not this one. The widened scan means any future cwd-relative anchor in keel-cli/src, locked file or not, is red at the ladder's probe rung before it reaches the lock. This Decision depends on D0479 (the extraction cluster it serves) and D0504 (the lock's current shape, which names adherence.rs).";
    }
    #DependsOn dependency from d0505 to d0479;
    #DependsOn dependency from d0505 to d0504;

    // acceptance event (D0121 review-queue sign-off; D0066/D0106 — human-judged, not fabricated)
    verification d0505Accept : Test { :>> id = "6f5d4640-6721-4030-a289-a7ad65e6bd76"; :>> method = VerificationMethod::confirmation; :>> procedureText = "recorded from chat - their words, verbatim: “Accept all decisions as recommended. Proceed to release and fix downstream” WARN: read-back - the quoted words do not name d0505 (D0201 B); recorded as given (D0423)."; }
    part d0505AcceptR1 : TestResult { :>> id = "7057b20c-c550-451d-a319-3bf900de7a5d"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "4d00c1bc"; :>> judgedAt = "2026-09-18"; :>> judgedBy = "wweatherholtz"; :>> createdBy = "claudeFable5"; }
}
