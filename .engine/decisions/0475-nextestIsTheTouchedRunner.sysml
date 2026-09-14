// D0475 (status: the `status` field below is the truth) — nextestIsTheTouchedRunner: the touched set runs under cargo-nextest, binaries in parallel and one process per test, and the receipt keeps per-test timings
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0475Accept event on sign-off.
package Decision0475 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    // #ProspectiveChange (D0070) - this Decision governs a process/schema change; the co-committed
    // change rides the process-change guard's hard lock.
    #ProspectiveChange part d0475 : Decision {
        :>> id = "86c756b2-fb71-43c1-8905-65e0cb9703a2";
        :>> title = "nextestIsTheTouchedRunner: the touched set runs under cargo-nextest, binaries in parallel and one process per test, and the receipt keeps per-test timings";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::accepted;
        :>> context = "cargo test runs test binaries one after another; each binary's tests run in threads but the binaries do not overlap. Six touched logs (touched_breakdown.py, 2026-09-14) sum the per-binary run time at 673-840 s; the slowest binary, the lib unit tests, takes 136-193 s and the next four take 42, 42, 35 and 32 s. cargo --timings (cargo-timing-20260914T035135064Z) shows the 71 integration binaries compile and link at a 3.4 s median in parallel, so consolidation into one binary (matklad) would buy nothing here and would destroy the per-binary selection D0421 depends on. Nothing measures which unit tests cost the 136-193 s (issue536). cargo-nextest is not installed on this host (cargo nextest --version: no such command). The adversarial panel of 2026-09-13 ranked this second; the reducer's alternative - spawning the compiled test executables ourselves - was set aside because it yields no per-test timing.";
        :>> decision = "NOT A FORK: one runner. keel-cli/src/touched.rs runs the touched set through cargo nextest run --release with the same --test selection, so binaries execute in parallel and every test is its own process; the receipt parser reads nextest's per-test output and records per-test durations in the touched receipt; the host installs a pinned cargo-nextest and the CI workflow installs the same pin before its suite; device_bound_taps.rs, the one test file binding a fixed address (grep 127.0.0.1: over keel-cli/tests), is reviewed for cross-binary collision before the switch. A failing test still names its binary in the receipt's failing list and refuses the land exactly as today.";
        :>> rationale = "The parallel wall is bounded below by the slowest binary, 136-193 s, against a serial sum of 673-840 s; the realistic figure is higher because twenty concurrent test processes each spawn keel.exe and git under Defender, and the Decision's implementing sprint states the measured before and after rather than this estimate. The lib binary is the critical path of both compile (283 s) and run and is unmeasured inside; nextest's per-test timing is the instrument that turns issue536 from a suspicion into a list. Process-per-test also removes the process-global hazard of the one test file that sets environment variables. The cost is real and named: a toolchain dependency on every host and in CI, and a rewrite of the --no-fail-fast summary parser at touched.rs:135 and :427.";
        :>> consequences = "touched.rs invokes nextest and parses its output; the receipt schema gains per-test durations; .github/workflows installs the pinned nextest; docs for keel suite and keel land name the runner; a follow-up measurement records the wall for the same touched set before and after, and the implementing sprint's DoD result quotes both. If the measured gain is under a factor of two the Decision is superseded with the numbers, not kept for its idea.";
    }

    // acceptance event (D0121 review-queue sign-off; D0066/D0106 — human-judged, not fabricated)
    verification d0475Accept : Test { :>> id = "1487f413-e209-4af4-a314-8820e1dbc8fa"; :>> method = VerificationMethod::confirmation; :>> procedureText = "chat acceptance of the three panel proposals labelled A (D0474), B (D0475), C (D0476) in the panel report of 2026-09-13; their words verbatim: ABC, continue! - their words, verbatim: “ABC, continue!” WARN: read-back - the quoted words do not name d0475 (D0201 B); recorded as given (D0423)."; }
    part d0475AcceptR1 : TestResult { :>> id = "01e7dca1-b151-4ae4-8912-8620c218b7be"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "8b731e6"; :>> judgedAt = "2026-09-14"; :>> judgedBy = "wweatherholtz"; :>> createdBy = "claudeFable5"; }
}
