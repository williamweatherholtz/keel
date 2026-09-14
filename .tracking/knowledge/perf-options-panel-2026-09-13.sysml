// KNOWLEDGE RECORD (D0196) - the adversarial panel over the eight compile/rerun options from the 2026-09-13 research spike; what it reordered and what it produced.
package KnowledgePerfOptionsPanel20260913 {
    private import EngineElement::*;
    private import EngineVerification::*;

    verification kPerfOptionsPanel : Test {
        :>> id = "567bcc5a-eaad-4ca7-ab12-f280a5cfeca8";
        :>> title = "Panel over eight options for the 936 s touched run: the content-keyed per-binary receipt ranks first because the third run of one fix was build 1.08 s and tests 736 s; nextest second for its per-test instrument; the verify ladder third; one binary and sccache rejected with the link number";
        :>> createdAt = "2026-09-13"; :>> createdBy = "claudeFable5";
        :>> method = VerificationMethod::analyze;
        :>> procedureText = "REPLICATION (D0196). PROCESS: adversarialPanelReview (D0187). MODE: the human had put the session in single-agent mode, so one context wore the lenses in turn - PESSIMIST-1 premise, PESSIMIST-2 operations, OPTIMIST-2 value, PEDANT-1 facts, PEDANT-3 mechanisms, REDUCER, SE. This record claims convergence, never independence: same-model lenses in one context satisfy no critic-independence requirement (skill guardrail, 2026-08-14 precedent). ARTIFACT: the ranked table of eight options from the research spike (content-keyed receipt; cheap gates first; cargo-nextest; incremental verify profile; finer .engine attribution; per-binary result cache; one integration binary; sccache/lld). EVIDENCE READ BEFORE ANY CLAIM: touched_breakdown.py over six .keel/metrics/touched-*.log - summed per-binary run 673-840 s, slowest binary 136-193 s then 42/42/35/32 s, the third run of sprint705's fix built in 1.08 s and tested for 736 s; cargo --timings on a one-file touch (cargo-timing-20260914T035135064Z) - wall 353 s, lib release 209 s, lib cfg(test) 283 s, 71 integration binaries at 3.4 s median compile+link in parallel; receipt.rs:153-158 keys on (len, mtime) with a 2 s RACY window; grep CARGO_MANIFEST_DIR over keel-cli/tests - 18 files read the repository's own tree; grep 127.0.0.1: - one file (device_bound_taps.rs) binds a fixed address; cargo nextest --version - not installed; Cargo.toml - no profile section; claude_surface.rs:80 PROTECTED_PATHS - five entries, no receipt. SIXTEEN FINDINGS, the load-bearing ones: F1 (blocker) option 1 as framed reused a verifier receipt that single-agent mode no longer produces - reframed as one green observation per content, whoever produced it; F2 (blocker) a whole-tree key reruns everything after the ceremony writes to .tracking - resolved by a two-tier key, K_code over keel-cli/.engine/Cargo/.githooks and K_tree over the whole tree, with the 18 self-reading binaries keyed on both; F5 a receipt the AI can Write is an honour system - resolved by adding the receipt files to the pre-write protected surfaces, CI remaining the authority; F8 reducer (spawn the executables ourselves) against optimist (nextest) resolved for nextest on evidence: the lib binary is the critical path of compile and run and only per-test timing can say why; F11 option 6 was mis-ranked second-order - it is the mechanism that makes option 1 pay, merged; option 5 deferred as issue416-class risk; F12 the SE invariant: no commit is pushed unless every binary in its touched set was observed green against the content its outcome depends on; F13/F14 options 7 and 8 rejected - link is 3.4 s median in parallel, not the bottleneck; F15 option 2 is a sequence, not a mechanism, until it is one command. VERDICT: all seven lenses ACCEPT the revised table; no irreducible disagreement handed to the human. WHAT IT PRODUCED: D0474 (content-keyed per-binary receipt, supersedes one clause of D0421, depends on D0371), D0475 (nextest as the touched runner), D0476 (keel verify ladder) - all three accepted by the human's words ABC, continue! on 2026-09-14; issue536 (the lib unit-test binary is the critical path twice and unmeasured); issue537 (57 of 71 binaries rerun on every .engine edit with no memo); the incremental verify profile left unauthored until a one-file touch is measured under it.";
    }
    part kPerfOptionsPanelR1 : TestResult {
        :>> id = "a246ec71-f899-4a99-96a3-871857641bf6";
        :>> outcome = VerdictKind::pass;
        :>> judgedAgainst = "8b731e6705ca3e60dcef014a0da8e3d74f9ff491";
        :>> judgedAt = "2026-09-13";
        :>> judgedBy = "claudeFable5";
        :>> notes = "Every number in the procedure was read from a log, a timings page or a grep on this host before the finding that cites it was written; the 190 s parallel wall is stated as a lower bound and the realistic 250-300 s as an estimate, neither as a measurement.";
    }
    dependency from kPerfOptionsPanel to d0196;
    dependency from kPerfOptionsPanel to d0187;
    dependency from kPerfOptionsPanel to d0474;
    dependency from kPerfOptionsPanel to d0475;
    dependency from kPerfOptionsPanel to d0476;
    dependency from kPerfOptionsPanel to issue536;
    dependency from kPerfOptionsPanel to issue537;
    dependency from kPerfOptionsPanel to kPerfSpike;
}
