// D0476 (status: the `status` field below is the truth) — verifyLadderStopsAtTheFirstRed: keel verify runs the pre-commit checks in cost order and stops at the first red, writing one receipt that names the rung
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0476Accept event on sign-off.
package Decision0476 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    // #ProspectiveChange (D0070) - this Decision governs a process/schema change; the co-committed
    // change rides the process-change guard's hard lock.
    #ProspectiveChange part d0476 : Decision {
        :>> id = "ffab42e9-da53-42e1-9421-78d1b75d663f";
        :>> title = "verifyLadderStopsAtTheFirstRed: keel verify runs the pre-commit checks in cost order and stops at the first red, writing one receipt that names the rung";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::accepted;
        :>> context = "The verification set before a commit is validate (0.46 s), guard (25.5 s, or its receipt), clippy -D warnings (48 s), the D0388 probe pair and the touched run (936 s). In sprint705 a clippy lint (duration_suboptimal_units at receipt.rs:492) was discovered after a 20-minute run because the set ran as separate commands in no enforced order; the fix cost a second 20-minute run. With the D0425 verifier suspended by the human (single-agent mode, 2026-09-13) the order is the AI's discipline alone, and manual vigilance is not a control (D0047). The adversarial panel of 2026-09-13 ranked this third and reframed it: a sequence is not a mechanism until it is one command.";
        :>> decision = "NOT A FORK: one command, one order, one receipt. keel verify runs gate validate, gate guard (answering from its receipt per D0371), cargo clippy --release -D warnings, the D0388 probe pair named on the command line (--probe positive,negative), then suite --touched keyed per D0474 - in that order, stopping at the first red. It writes one receipt naming the rung it stopped at and, on green, the touched receipt that keel land will honour. It is declared as a CliCommand in .engine/cli/commands.sysml (D0271) so --help, dispatch and the facts agree.";
        :>> rationale = "Cost order is the whole point: a lint reported in under a minute saves the twenty-minute run that would have followed it, and the ladder makes the order a property of the tool rather than of the operator. One receipt gives the recorder - now the primary itself - a single artifact to quote, which is what D0232 asks a test-method result to cite. The command adds no new check; it sequences the five that exist, so its risk is in ordering and in refusing to continue past a red, both testable with the probe pair.";
        :>> consequences = "keel-cli/src gains verify.rs orchestrating the existing gate, guard, clippy, probe and touched entry points; commands.sysml gains the cliVerify fact; usage docs name the ladder; the delegated-ceremony and test-verify skills point at keel verify instead of listing the rungs. Residual: the ladder runs on the working tree, so a commit that stages less than the tree verifies more than it ships - the same residual D0421 carries for land, resolved there by land's own run.";
    }

    // acceptance event (D0121 review-queue sign-off; D0066/D0106 — human-judged, not fabricated)
    verification d0476Accept : Test { :>> id = "827d937e-42e1-4336-938d-a372f56f24c2"; :>> method = VerificationMethod::confirmation; :>> procedureText = "chat acceptance of the three panel proposals labelled A (D0474), B (D0475), C (D0476) in the panel report of 2026-09-13; their words verbatim: ABC, continue! - their words, verbatim: “ABC, continue!” WARN: read-back - the quoted words do not name d0476 (D0201 B); recorded as given (D0423)."; }
    part d0476AcceptR1 : TestResult { :>> id = "324e26e6-52a6-4854-a5bb-fffa115caa7e"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "8b731e6"; :>> judgedAt = "2026-09-14"; :>> judgedBy = "wweatherholtz"; :>> createdBy = "claudeFable5"; }

    // grounded (D0098 rootedness): this Decision serves the Need that a pass is never AI testimony alone -
    // st065, the human's words behind the D0425 ceremony line (verifier, recorder, ladder, rendered receipt)
    #DerivedFrom dependency from d0476 to nVerificationByAuthorityAndMethod;
}
