// ProjectDeliveryS764 — Sprint 764: keel migrate's removed-types scan reads a // line comment as prose, so the RAN evidence line the write API writes no longer blocks a downstream tree from taking the engine resync (issue675), and v0.5.3 ships it.. EXECUTE. 2 pts.
package ProjectDeliveryS764 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    #CharteredBy dependency from migrateCommentIsProseStory to d0341;
    #Delivers dependency from migrateCommentIsProseStory to dcMigrateScannerSkipsLineComments;

    part migrateCommentIsProseStory : Story {
        :>> id = "9f77e337-b475-4df6-862b-f9c5dcf29e0d";
        :>> title = "Sprint 764: keel migrate's removed-types scan reads a // line comment as prose, so the RAN evidence line the write API writes no longer blocks a downstream tree from taking the engine resync (issue675), and v0.5.3 ships it. (2 pts)";
        :>> createdAt = "2026-09-23"; :>> createdBy = "claudeFable5"; :>> kind = WorkKind::code; :>> priority = WorkPriority::p0; :>> owner = "claudeFable5"; :>> estimatedPoints = 2;
    }

    action def DeliveryRunS764 {
        action storyMigrateCommentIsProse;
        verification storyMigrateCommentIsProseDoD : Test { :>> id = "0f129667-7d98-422a-9445-0d7cd5bec4f8"; :>> method = VerificationMethod::test; :>> procedureText = "DELIVERED BACKLOG ITEM: dcMigrateScannerSkipsLineComments. DONE MEANS: its DoD holds at the landing commit - the unit test a_removed_type_named_inside_a_line_comment_is_prose_not_schema passes beside its string-literal sibling, and keel migrate . --dry-run over keel-next plans the resync with 0 blockers where the published 0.5.2 binary reported 4."; }
        // RAN: cargo test -p keel-process --lib migrate -> 15 passed, 0 failed, including a_removed_type_named_inside_a_line_comment_is_prose_not_schema beside a_removed_type_named_inside_a_string_literal_is_prose_not_schema; the new test failed before the fix at migrate.rs:1779 (the comment arm blocked). keel migrate . --dry-run over keel-next at 692c62c: the published 0.5.2 binary (build bb1882d) reported 4 blockers, .tracking/backlog.sysml:11, 15, 19, 23; the binary built at this tree reports totals 21 file(s), 21 edit(s), 0 blocker(s). cargo clippy --workspace --all-targets -D warnings clean. keel land at 36057f44: touched tests 767 passed.
        part storyMigrateCommentIsProseDoDR1 : TestResult { :>> id = "6b79753f-9db9-4ed4-b665-70c5f5556c78"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "36057f44"; :>> judgedAt = "2026-09-23"; :>> judgedBy = "claudeFable5"; }
    }

    verification migrateCommentIsProseRefineGate : Test { :>> id = "dfaad1c2-75b0-4339-a841-f7f69eb654ee"; :>> title = "Sprint 764 refine gate (DoR)"; :>> createdAt = "2026-09-23"; :>> createdBy = "claudeFable5"; :>> method = VerificationMethod::inspect; :>> procedureText = "DoR: issue675 is established against the scanner's source, members/keel-process/src/migrate.rs without_string_literals and types_as, and against the writer that emits the line, members/keel-write/src/write.rs:892. One backlog action with a test DoD; 2 points, code changes in one module plus its unit test; charter d0341, the prose-is-not-schema line issue377 opened."; }
    // RAN: keel land at 36057f44: touched tests 767 passed; cargo test -p keel-process --lib migrate 15 passed
    part migrateCommentIsProseRefineGateR1 : TestResult { :>> id = "ad73a995-a890-410c-b403-59989de68607"; :>> outcome = VerdictKind::proposed; :>> judgedAgainst = "36057f44"; :>> judgedAt = "2026-09-23"; :>> judgedBy = "claudeFable5"; }
    verification migrateCommentIsProseStandupGate : Test { :>> id = "33e9451d-0a25-49e5-a9e4-d5a626044437"; :>> title = "Sprint 764 standup gate"; :>> createdAt = "2026-09-23"; :>> createdBy = "claudeFable5"; :>> method = VerificationMethod::inspect; :>> procedureText = "One item, no dependency. The fix must not move the check_preconditions lines three locked process files cite (guard source-reference against the process-change lock), so the helper sits after that function."; }
    // RAN: keel land at 36057f44: touched tests 767 passed; cargo test -p keel-process --lib migrate 15 passed
    part migrateCommentIsProseStandupGateR1 : TestResult { :>> id = "ff3c9986-9852-43a2-97d7-d4947d0f7681"; :>> outcome = VerdictKind::proposed; :>> judgedAgainst = "36057f44"; :>> judgedAt = "2026-09-23"; :>> judgedBy = "claudeFable5"; }
    verification migrateCommentIsProseImplementGate : Test { :>> id = "1d1fb283-9078-41a7-baac-06d63f17866d"; :>> title = "Sprint 764 implement gate"; :>> createdAt = "2026-09-23"; :>> createdBy = "claudeFable5"; :>> method = VerificationMethod::test; :>> procedureText = "without_line_comment drops the line from its first // after without_string_literals has blanked the literals; step_removed_types scans that. The new unit test failed before the change and passes after it; clippy -D warnings clean across the workspace."; }
    // RAN: keel land at 36057f44: touched tests 767 passed; cargo test -p keel-process --lib migrate 15 passed
    part migrateCommentIsProseImplementGateR1 : TestResult { :>> id = "3e641fcf-a543-46cf-87bb-c2359bda2eb3"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "36057f44"; :>> judgedAt = "2026-09-23"; :>> judgedBy = "claudeFable5"; }
    verification migrateCommentIsProseReviewGate : Test { :>> id = "715549b4-9138-4264-b44a-0bf5b0746df3"; :>> title = "Sprint 764 review gate — three passes per D0172"; :>> createdAt = "2026-09-23"; :>> createdBy = "claudeFable5"; :>> method = VerificationMethod::inspect; :>> procedureText = "Three passes. Correctness: a // inside a literal is blanked first, so it is never read as a comment; a real annotation before a trailing comment still blocks. Scope: only the removed-types and removed-enum scan changed; no other migrate step reads the helper. Evidence: the rebuilt binary's dry run over keel-next reports 0 blockers."; }
    // RAN: keel land at 36057f44: touched tests 767 passed; cargo test -p keel-process --lib migrate 15 passed
    part migrateCommentIsProseReviewGateR1 : TestResult { :>> id = "27ddb7b5-ed72-498f-803b-afbbeab61493"; :>> outcome = VerdictKind::proposed; :>> judgedAgainst = "36057f44"; :>> judgedAt = "2026-09-23"; :>> judgedBy = "claudeFable5"; }
    verification migrateCommentIsProseCloseOutGate : Test { :>> id = "9ea17208-cf71-45af-b5a9-fbbf1766d818"; :>> title = "Sprint 764 closeOut gate (autonomous, D0049)"; :>> createdAt = "2026-09-23"; :>> createdBy = "claudeFable5"; :>> method = VerificationMethod::inspect; :>> procedureText = "dcMigrateScannerSkipsLineComments passes at 36057f44 through keel record result; issue675 resolves by that result; v0.5.3 is cut from the tree that carries it."; }
    // RAN: keel land at 36057f44: touched tests 767 passed; cargo test -p keel-process --lib migrate 15 passed
    part migrateCommentIsProseCloseOutGateR1 : TestResult { :>> id = "6ab7ff81-afa1-4da7-ab48-cd55cb1e0ecb"; :>> outcome = VerdictKind::proposed; :>> judgedAgainst = "36057f44"; :>> judgedAt = "2026-09-23"; :>> judgedBy = "claudeFable5"; }
    verification migrateCommentIsProseRetroGate : Test { :>> id = "ce12896a-2937-4e8f-be58-99d6208e8d92"; :>> title = "Sprint 764 retro gate (autonomous, D0049)"; :>> createdAt = "2026-09-23"; :>> createdBy = "claudeFable5"; :>> method = VerificationMethod::analyze; :>> procedureText = "Avoidable-issue scan: issue675 is the third adopter-only defect keel-next has found in keel in two days, after issue673 and issue674: each is green in keel because keel's own tree never exercises the shape. No further item: the class is already tracked by issue673 and issue674's resolvers, and this fix carries its own test."; }
    // RAN: keel land at 36057f44: touched tests 767 passed
    part migrateCommentIsProseRetroGateR1 : TestResult { :>> id = "9fb63875-b385-46c7-a7b1-d32bd7bab272"; :>> outcome = VerdictKind::proposed; :>> judgedAgainst = "36057f44"; :>> judgedAt = "2026-09-23"; :>> judgedBy = "claudeFable5"; }
}
