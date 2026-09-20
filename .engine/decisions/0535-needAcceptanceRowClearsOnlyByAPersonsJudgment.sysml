// D0535 (status: the `status` field below is the truth) — A needAcceptance row is cleared only by a Person's judgment - by the row's own predicate and the write layer's D0178 refusal, not by the confirmation-authenticity guard, which reads Decision acceptance events only
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0535Accept event on sign-off.
package Decision0535 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    part d0535 : Decision {
        :>> id = "a7f04cba-7667-4a6e-86ad-22d475ab2b87";
        :>> title = "A needAcceptance row is cleared only by a Person's judgment - by the row's own predicate and the write layer's D0178 refusal, not by the confirmation-authenticity guard, which reads Decision acceptance events only";
        :>> createdAt = "2026-09-19";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::accepted;
        :>> context = "D0534's consequences say the confirmation-authenticity guard already refuses an AI-judged confirmation result, so an AI cannot clear a needAcceptance row by writing one. Read before the code was: the guard (members/keel-guards/src/acceptance.rs confirmation_authenticity) sources confirmationAuthenticityRule and rejectionAuthenticityRule, whose subjectType is Decision and whose predicate is resultJudgedByHuman(Accept) / resultJudgedByHuman(Reject) - it reads a Decision's dNNNNAccept and dNNNNReject events and nothing else. A confirmation TestResult on a Need's acceptance Test is outside its scan. What does refuse an AI-kind judge on a confirmation result is the write layer (members/keel-write/src/write.rs, K6/D0178: refuse_ai_judgment on a method=confirmation result), and what keeps the row standing regardless of who wrote what is the row's predicate itself, which counts only a passing result whose judgedBy names a Person. The conclusion of the clause holds; the mechanism it names does not, and a Decision that names the wrong control teaches its reader to trust a guard that is not looking. NOT A FORK: this corrects a factual clause; no alternative is weighed. NOT A PROCESS CHANGE: no guard, gate, workflow or schema member changes; the clause names which existing controls hold.";
        :>> decision = "The clause of D0534 reading The confirmation-authenticity guard already refuses an AI-judged confirmation result, so an AI cannot clear a row by writing one is reversed and replaced by: a needAcceptance row is cleared only by a passing confirmation TestResult whose judgedBy names a Person, because that is the row's predicate; an AI-kind actor's confirmation result is refused at the write layer (D0178) and, were it typed by hand, would not clear the row. The confirmation-authenticity guard is not among the controls that hold this; extending it to Need acceptance is a separate change, if ever wanted. Every other clause of D0534 stands.";
        :>> rationale = "D0151 binds what is said, not only what is recorded: a control named as standing must be the control that is looking. Naming the guard when the write layer and the predicate are the real controls would send the next reader to a guard whose scan cannot fire on this class, and a false sense of a control is worse than a known gap. A #SupersedeClause leaves D0534 in force with its acceptance intact (D0398) and corrects the one clause, which is what D0303's one-clause discipline exists for.";
        :>> consequences = "D0534 remains accepted and in force; its acceptance binds to its unchanged text (acceptance-binds-to-text is untouched). The needAcceptance implementation carries a test that an AI-judged passing confirmation result does not clear the row, which is the D0388 known-negative dcAuthorityQueueListsNeedsAwaitingAcceptance already names. No guard is added or widened by this Decision.";
    }
    #SupersedeClause dependency from d0535 to d0534;

    // acceptance event (D0121 review-queue sign-off; D0066/D0106 — human-judged, not fabricated)
    verification d0535Accept : Test { :>> id = "dbae20ea-8c5f-49fe-b363-e1947951cc24"; :>> method = VerificationMethod::confirmation; :>> procedureText = "AUTO-ACCEPTED under standing consent (D0207). Their standing words, verbatim: 'issues raised are automatically accepted. they can be customizedly changed post-fact.' Not individually reviewed; override by a superseding Decision (D0290) or the human's quoted word in chat (D0289)."; }
    part d0535AcceptR1 : TestResult { :>> id = "610b0c30-5465-4a7a-b373-55350972199e"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "c3e4635a"; :>> judgedAt = "2026-09-19"; :>> judgedBy = "wweatherholtz"; :>> createdBy = "claudeFable5"; }
}
