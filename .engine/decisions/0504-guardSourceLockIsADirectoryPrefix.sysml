// D0504 (status: the `status` field below is the truth) — The guard-source lock names the member directory by prefix, and a new guard file is inside it by construction
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0504Accept event on sign-off.
package Decision0504 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    // #ProspectiveChange (D0070) - this Decision governs a process/schema change; the co-committed
    // change rides the process-change guard's hard lock.
    #ProspectiveChange part d0504 : Decision {
        :>> id = "5cf0178e-31f3-4694-9ce7-74157d43618e";
        :>> title = "The guard-source lock names the member directory by prefix, and a new guard file is inside it by construction";
        :>> createdAt = "2026-09-16";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::accepted;
        :>> context = "Sprint 733 is the fourth D0479 extraction: keel-cli/src/guards.rs (9,256 lines) becomes member keel-guards, one module per guard family, with receipt, contentkey, hardening and plan_cover riding in the same member. The enforcement-surface lock (D0209 clause 2, sprint 443) named its guard sources as a list of files, GUARD_SOURCE_FILES, with keel-cli/src/guards.rs first, and enforcement_surface_covers_every_guard_source scanned keel-cli/src for any file defining a guard outside that list. After the split the guard-defining files are nine family modules plus the crate root, and the scan of one crate's src no longer sees them: a scan aimed at keel-cli/src would pass with nothing to check, which is the class D0388 names (a control that goes silent when its subject moves). D0503 put GUARD_NAMES under the same lock when the list left guards.rs; this Decision is the third meeting of the lock that D0503's consequences said would come.";
        :>> decision = "GUARD_SOURCE_FILES loses keel-cli/src/guards.rs (the file no longer exists) and keeps keel-cli/src/adherence.rs and members/keel-schema/src/guard_names.rs. A second constant, GUARD_SOURCE_DIRS, names members/keel-guards/src/ and is_enforcement_surface holds a path locked when it is in the file list or starts with a locked directory. The coverage test reads the workspace manifest and scans every member's src, so a guard written into any member is found, and it asserts the scan reached the guards member's own files. The lock test names the new paths on both sides: lib.rs, identity.rs and receipt.rs under the member are locked; the old path, the member's Cargo.toml, keel-cli/src/main.rs and the docs are not. every_enforced_guard_dispatches reads the family tables rather than the text of a file that no longer exists; family_union_tests holds the tables equal to GUARD_NAMES in its order and holds the four runnable-only names apart. The receipt (D0371) and the content key it is keyed on are inside the locked directory, which is a widening: a green-guard skip is enforcement logic and an edit to it needed a marked Decision already under D0209's intent.";
        :>> rationale = "A file list has to be edited every time a guard file is added, and the coverage test exists because that edit is forgettable; a directory prefix makes the addition of a tenth family file locked by construction and leaves the coverage test to catch the one remaining hole, a guard defined outside the member. Scanning every member instead of one crate's src is the same reasoning D0503 used for the list: the check follows the subject. The dispatch test moved from text to tables because the tables are the dispatch now, and a test that scans source for a match arm would have been asserting the shape of code that no longer exists. Reading the manifest for the member list rather than hard-coding twelve paths means the next extraction (dcSuiteIsAMember) is covered without editing this test.";
        :>> consequences = "Every file under members/keel-guards/src/ is a locked surface: adding a family, editing the runner, the receipt or the content key needs a co-committed marked Decision, as guards.rs did. The Cargo.toml of the member is not locked, so a dependency change is an ordinary edit; the lock is on logic. The coverage test now depends on keel_model::corpus::workspace_members and on the manifest listing at least twelve members, so removing members below that count fails the test on purpose. guards.md still groups guards by tier while the code groups by family; the retro records the two groupings and whether the catalogue should follow the code. The pattern D0503 named is now recorded three times (D0486, D0503, this): an extraction sprint that moves a locked source carries a marked Decision naming where the lock now lands.";
    }
    #DependsOn dependency from d0504 to d0479;
    #DependsOn dependency from d0504 to d0503;

    // acceptance event (D0121 review-queue sign-off; D0066/D0106 — human-judged, not fabricated)
    verification d0504Accept : Test { :>> id = "79a35447-9bf4-40f4-ab70-f70fd93d8970"; :>> method = VerificationMethod::confirmation; :>> procedureText = "recorded from chat - their words, verbatim: “Accept all decisions as recommended. Proceed to release and fix downstream” WARN: read-back - the quoted words do not name d0504 (D0201 B); recorded as given (D0423)."; }
    part d0504AcceptR1 : TestResult { :>> id = "eede314f-8491-4a4b-ac42-487b1f403af7"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "4d00c1bc"; :>> judgedAt = "2026-09-18"; :>> judgedBy = "wweatherholtz"; :>> createdBy = "claudeFable5"; }
}
