// D0550 (status: the `status` field below is the truth) — briefNoteIsPerAsk: the decision brief carries one note field inside each ask's tab, the digest emits each note beneath its own answer, and the brief checker refuses a page whose note count differs from its tab count
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0550Accept event on sign-off.
package Decision0550 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    part d0550 : Decision {
        :>> id = "c969b266-d015-494c-bf1d-1a83d70640fa";
        :>> title = "briefNoteIsPerAsk: the decision brief carries one note field inside each ask's tab, the digest emits each note beneath its own answer, and the brief checker refuses a page whose note count differs from its tab count";
        :>> createdAt = "2026-09-22";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::accepted;
        :>> context = "The human, 2026-09-22 (st192): 'fix artifact permanently so the comment section is unique per tab. today, it's shared and i can't uniquely give feedback properly'. The published runner-direction brief (D0542) and every standing decision page since D0404 carry one 'Anything to add' textarea below the tabs; the copy-for-AI digest emits it once, at the end, as 'Recorded with the answer'. With seven asks on one page a note written for one ask reads as a note on all of them - the human's first pasted digest carried a note about slice one under the whole page, and the AI had to infer which ask it belonged to. The source template templates/exec-summary/exec-summary.html already places a note inside each tab panel; the builders dropped it when the tabs came back (D0404). Their word 'permanently' asks for a control, not a fix on one page: the brief checker (scripts/check_templates.py --brief) is the gate every published brief passes, and a clause there binds every future page.";
        :>> decision = "Every executive brief carries exactly one note field per ask, inside that ask's tab panel, keyed to the ask; the copy-for-AI digest emits each note directly beneath its own ask's options, never once for the page; the reader's draft store keeps notes per ask. The brief checker gains the clause: the count of note fields equals the count of tab panels, and every note field sits inside a tab panel - a page with a shared note or a note outside a panel is refused, naming the counts. The runner-direction brief is rebuilt to this shape and republished at its URL. NOT A FORK: the human said what shape they want; this is its one instance and the control that keeps it. NOT A PROCESS CHANGE: no workflow, gate, schema or rule of the engine moves; the brief checker is the asi-templates surface's own contract check and gains one clause the way D0404, D0405 and D0406 added theirs.";
        :>> rationale = "A note the reader cannot attach to an ask is a note the AI attaches by guessing, and a guessed attachment on a page whose whole purpose is to let one answer be recorded against one record is the failure D0404 named when it put each ask on its own tab. The template already knew this; the builders lost it. Putting the clause in the checker rather than in one builder is D0047: a correction that could recur becomes a control, and the standing page's next republish fails the clause until it complies, which is the point.";
        :>> consequences = "scripts/check_templates.py --brief refuses a page whose note count differs from its panel count or whose note is outside a panel, with a probe pair; the asi-templates skill's section table gains the row; the runner-direction brief is rebuilt with seven per-ask notes and republished; the standing decision page's next republish must carry per-ask notes or the checker refuses it.";
    }

    // acceptance event (D0121 review-queue sign-off; D0066/D0106 — human-judged, not fabricated)
    verification d0550Accept : Test { :>> id = "3cebf380-96e7-4976-9342-69952765eab0"; :>> method = VerificationMethod::confirmation; :>> procedureText = "AUTO-ACCEPTED under standing consent (D0207). Their standing words, verbatim: 'issues raised are automatically accepted. they can be customizedly changed post-fact.' Not individually reviewed; override by a superseding Decision (D0290) or the human's quoted word in chat (D0289)."; }
    part d0550AcceptR1 : TestResult { :>> id = "557b514a-4d8d-4c93-8f8a-1e43829d30ec"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "b65a13e4"; :>> judgedAt = "2026-09-22"; :>> judgedBy = "wweatherholtz"; :>> createdBy = "claudeFable5"; }
}
