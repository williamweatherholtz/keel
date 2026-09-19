// D0499 (status: the `status` field below is the truth) — The guard source's own tests follow the scratch rule - the seven fixed-name scratch joins in guards.rs test code are named through keel_fs::scratch, and no guard predicate changes
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0499Accept event on sign-off.
package Decision0499 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    // #ProspectiveChange (D0070) - this Decision governs a process/schema change; the co-committed
    // change rides the process-change guard's hard lock.
    #ProspectiveChange part d0499 : Decision {
        :>> id = "7c63d245-e6e1-4270-8dde-29bbc343b621";
        :>> title = "The guard source's own tests follow the scratch rule - the seven fixed-name scratch joins in guards.rs test code are named through keel_fs::scratch, and no guard predicate changes";
        :>> createdAt = "2026-09-16";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::accepted;
        :>> context = "D0498 made every test's scratch directory carry the process id and put a census in the keel-cli lib that fails any test-code temp_dir().join whose argument two processes could share. Seven of the thirty-six sites the census names sit in the test module of keel-cli/src/guards.rs (lines 4591, 7596, 7629, 7685, 8296, 8353, 8393 before the rewrite) - the file that holds every forward guard. That file is under the D0209 clause 2 hard lock: the enforcement surface changes only with a co-committed process-change Decision, because a control that modifies itself silently is the issue236 class. The lock reads the path, not the region, so a change confined to the file's #[cfg(test)] module fires it exactly as a change to a predicate would. Without this record the sprint 729 tree is red at the guard rung and the census cannot land, because excluding guards.rs from the census would be the dishonest alternative.";
        :>> decision = "The seven test-code scratch sites in keel-cli/src/guards.rs are rewritten from std::env::temp_dir().join(<fixed name>) to keel_fs::scratch(<the same name>), carrying the process id as D0498 requires. Nothing above the file's first #[cfg(test)] line changes: no guard predicate, no message, no catalogue row, no dispatch. This Decision is the co-committed process-change record the lock asks for; it governs those seven lines and nothing else.";
        :>> rationale = "The lock is right to fire and wrong to be argued around: the honest way through a hard lock is the record it names, not a carve-out for test regions, which would be a change to the lock itself and need this same record. The change is mechanical and inspectable - git diff --stat keel-cli/src/guards.rs shows seven insertions and seven deletions, each replacing one call with its D0498 form - and the D0388 pair plus the real-tree census in touched.rs are the tests that hold it. Recording it as a Decision rather than a test-only exemption keeps one rule for the enforcement surface: it changes when a human has signed for the change, whatever the region.";
        :>> consequences = "guards.rs's test module names its scratch through keel_fs::scratch; the process-change guard passes on the sprint 729 tree with this Decision co-committed; the census no_test_names_a_scratch_directory_two_processes_could_share covers guards.rs like every other file. Whether the lock should read the test region differently from the predicate region is not decided here - a later Decision may narrow the lock, and until one does every edit to guards.rs test code is recorded this way. Delivered by sprint 729 under D0498. Discharges obligation637916a4, the turn-gate yield on sprint 729's mid-flight tree, whose first red was the lock this record answers.";
    }

    // D0303 layering: this record applies D0498's rule to the one file the lock covers.
    #DependsOn dependency from d0499 to d0498;

    // acceptance event (D0121 review-queue sign-off; D0066/D0106 — human-judged, not fabricated)
    verification d0499Accept : Test { :>> id = "ba8d6b6f-df76-4a6a-a5ec-78451095797c"; :>> method = VerificationMethod::confirmation; :>> procedureText = "recorded from chat - their words, verbatim: “Accept all decisions as recommended. Proceed to release and fix downstream” WARN: read-back - the quoted words do not name d0499 (D0201 B); recorded as given (D0423)."; }
    part d0499AcceptR1 : TestResult { :>> id = "c8115fdb-bbac-4d5b-a9b5-e360253cdff8"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "4d00c1bc"; :>> judgedAt = "2026-09-18"; :>> judgedBy = "wweatherholtz"; :>> createdBy = "claudeFable5"; }
}
