// D0522 (status: the `status` field below is the truth) — A record migrate tolerated at the door is tolerated through the run: the post-apply gate discounts it and the rollback preserves it
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0522Accept event on sign-off.
package Decision0522 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    // #ProspectiveChange (D0070) - this Decision governs a process/schema change; the co-committed
    // change rides the process-change guard's hard lock.
    #ProspectiveChange part d0522 : Decision {
        :>> id = "01a09c6d-0649-42ff-b501-7ed86a22c970";
        :>> title = "A record migrate tolerated at the door is tolerated through the run: the post-apply gate discounts it and the rollback preserves it";
        :>> createdAt = "2026-09-18";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::accepted;
        :>> context = "GH#87 (st165; public repo, untrusted, D0264): on plotwright, `keel migrate` reverted with `ERROR obligation35c54864: untriaged - no #Resolves edge`, and after the revert the file `.tracking/obligations/red-yield-35c54864.sysml` no longer existed. Read against this tree before triage: check_preconditions (members/keel-process/src/migrate.rs:908) partitions the porcelain into tolerated and blocking by is_tolerable_obligation (:875) - a NEW `.sysml` under `.tracking/obligations/`, the one directory only the engine writes - and report_tolerated (:887) prints that the record is `additive, and separable from migration state`. The run then applies, and project_gate (:1372) runs `gate validate`, `gate guard all` and `gate check-engine` over the whole tree, so guard issues fails on the very record the door tolerated: an engine-authored red-yield obligation (D0176/K7) is untriaged by construction the moment it is written, and the staleness migrate would cure is what keeps writing them, so the window in which migrate can pass is the gap between two red turns. On failure restore (:981) runs `git checkout <sha> -- .engine .tracking .claude` and then `git clean -fdq -- .engine .tracking .claude` (:996-997); the tolerated record was never committed, so `clean` deletes it, guard issues reports PASS on the next attempt because the evidence is gone, and `keel show status` names an obligation id that resolves to nothing. The human's direction, verbatim in st167 and st168: migrate is blocked in many projects, migration should be smooth, make downstream projects whole.";
        :>> decision = "The tolerance check_preconditions grants is a property of the run, not of its first step: the post-apply gate does not count a violation whose subject is a part declared in a tolerated record, and the rollback's `git clean` excludes every tolerated path, so a record the run did not write is neither what fails it nor what it destroys. The gate stays red for every other violation; a tolerated record that is MODIFIED rather than new stays refused at the door as today.";
        :>> rationale = "The two halves of one command disagreed about one file, and the disagreement destroyed the engine's own audit trail of a bypassed control - the record K7 exists to make un-silent. The narrow shape is chosen over the broad one the report also offered. Judging only regressions against a pre-apply gate run would mask every pre-existing red, and a stale engine under a new binary is red for many reasons before the apply (the report lists four guards); masking them is the hollowing-out the precondition's own comment refuses. Discounting violations whose subject is a tolerated record reuses the door's exact predicate, so the run tolerates one thing under one definition. Preserving the paths in `clean` is the smallest change that makes the rollback's own comment true: `discarding anything the interrupted run wrote` - the run did not write this.";
        :>> consequences = "Backlog task dcToleratedRecordsSurviveTheRun delivers this sprint (745) and resolves issue613. migrate's report names the violations it discounted and the paths it preserved, so the carve-out is visible in the run's output as the door's already is. With D0520 landed the plotwright-shaped tree has no other red inside the run, so the migration that reverted three times lands. Stated residual: an untriaged obligation is still red under `keel gate guard` after the migration, as it should be - the record asks for triage, and migrate is not that act. GH#87 is answered on GitHub with the tracked id once this record's status is known. HELD proposed under D0337 for the human's word and applied under the marker in the same commit, as D0519 was; their word keeps or reverts it.";
    }
    #DerivedFrom dependency from d0522 to st167;
    #DerivedFrom dependency from d0522 to st168;

    // acceptance event (D0121 review-queue sign-off; D0066/D0106 — human-judged, not fabricated)
    verification d0522Accept : Test { :>> id = "77faf282-168d-47dd-8531-10e4dcb3068d"; :>> method = VerificationMethod::confirmation; :>> procedureText = "recorded from chat - their words, verbatim: “Accept all decisions as recommended. Proceed to release and fix downstream” WARN: read-back - the quoted words do not name d0522 (D0201 B); recorded as given (D0423)."; }
    part d0522AcceptR1 : TestResult { :>> id = "a7ddef38-8883-4968-832a-aaa8568d91b3"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "4d00c1bc"; :>> judgedAt = "2026-09-18"; :>> judgedBy = "wweatherholtz"; :>> createdBy = "claudeFable5"; }
}
