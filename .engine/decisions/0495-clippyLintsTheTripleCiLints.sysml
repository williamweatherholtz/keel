// D0495 (status: the `status` field below is the truth) — The pre-commit clippy lints the triple CI lints - a host that is not x86_64-unknown-linux-gnu runs clippy a second time for that target, and a host without its std is red, not skipped
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0495Accept event on sign-off.
package Decision0495 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    // #ProspectiveChange (D0070) - this Decision governs a process/schema change; the co-committed
    // change rides the process-change guard's hard lock.
    #ProspectiveChange part d0495 : Decision {
        :>> id = "0b83b2db-3e12-4707-a322-e8b3755f155d";
        :>> title = "The pre-commit clippy lints the triple CI lints - a host that is not x86_64-unknown-linux-gnu runs clippy a second time for that target, and a host without its std is red, not skipped";
        :>> createdAt = "2026-09-15";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::accepted;
        :>> context = "Sprint 725 landed at eb82b52 with every local gate green - the D0476 ladder's clippy rung, the VERIFIER's clippy, the pre-commit hook's `cargo clippy --workspace --all-targets -- -D warnings`, and keel land's 865-test touched set - and CI's `clippy (-D warnings)` step failed the commit on keel-cli/src/touched.rs:662 (`clippy::map_unwrap_or`, pedantic). The line is the non-Windows `pid_alive_host` body D0493 added under `#[cfg(not(windows))]`. The Windows host that ran every local clippy never compiles that branch; ci.yml lints on ubuntu-latest, whose triple is x86_64-unknown-linux-gnu, and does. The probe pair run before this Decision (issue572): the old line under `cargo clippy --release --all-targets --target x86_64-unknown-linux-gnu --manifest-path keel-cli/Cargo.toml -- -D warnings` exits 101 naming the line; the same command without `--target` exits 0; the corrected line exits 0 under both. The Linux-target check needs only the target's std (`rustup target add x86_64-unknown-linux-gnu`), no linker, and took 1 m 41 s cold on this host. issue453 was the first instance of the local-gate-differs-from-CI class; this is the second.";
        :>> decision = "Every clippy the engine runs before a push lints the triple CI lints. The D0476 ladder's clippy rung and the pre-commit hook's clippy step each run `cargo clippy ... -- -D warnings` once for the host and, when the host triple is not x86_64-unknown-linux-gnu, a second time with `--target x86_64-unknown-linux-gnu`; the ladder's receipt names both command lines in the clippy rung row. A host whose toolchain lacks that target's std is a RED rung or a refused commit naming `rustup target add x86_64-unknown-linux-gnu` as the remedy - never a skipped check (D0098: a check that cannot run must never pass silently).";
        :>> rationale = "A pre-commit lint that cannot see the code CI compiles proves nothing about that code, and the ladder's whole claim (D0476) is that a red is reported before the push, not after it. `#[cfg]` bodies for the other host are exactly the code a single-host developer never compiles and never lints; linting them costs a second cargo check (about two minutes cold, seconds incremental) and needs no cross-linker because clippy stops before linking. The alternative of adding the other-host lint only to CI is what exists today and is what failed. This is a process-change: the pre-commit hook is an enforcement surface under D0209 clause 2, so the change is co-committed with this Decision and held for the human under D0337.";
        :>> consequences = "keel-cli/src/verify.rs `Rung::Clippy` runs both commands and its rung row names both; `.githooks/pre-commit` runs both; a missing target std is red with the rustup remedy on the terminal. `.engine/skills/test-verify/SKILL.md` names the second command beside the first (`.claude/` copy via sync-claude, same commit). Delivered by dcClippyLintsTheOtherHostsCfg, which resolves issue572. Rust CI is unchanged; a host that IS x86_64-unknown-linux-gnu runs clippy once, as before.";
    }

    // acceptance event (D0121 review-queue sign-off; D0066/D0106 — human-judged, not fabricated)
    verification d0495Accept : Test { :>> id = "a7674545-f514-4e55-82cf-0a22400a64b0"; :>> method = VerificationMethod::confirmation; :>> procedureText = "recorded from chat - their words, verbatim: “Accept all decisions as recommended. Proceed to release and fix downstream” WARN: read-back - the quoted words do not name d0495 (D0201 B); recorded as given (D0423)."; }
    part d0495AcceptR1 : TestResult { :>> id = "4888ad1c-e383-4568-ad94-a1270f68261f"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "4d00c1bc"; :>> judgedAt = "2026-09-18"; :>> judgedBy = "wweatherholtz"; :>> createdBy = "claudeFable5"; }
}
