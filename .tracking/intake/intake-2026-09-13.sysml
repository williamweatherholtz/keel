// Intake for 2026-09-13 — their words, my translation, the triage verdict (D0166/D0216).
// Written by `keel record statement` / `keel record story` (D0236): a Statement's text is
// VERBATIM, so it is escaped for the literal and otherwise untouched.
package ProjectIntake20260913 {
    private import EngineElement::*;
    private import EngineIntake::*;
    private import EngineRelationships::*;

    part st121 : Statement {
        :>> id = "dbcd0d6e-9398-4367-a89e-bdf4dee10b68";
        :>> title = "Retros should yield Issues, not Decisions; the capital-letter form was never the question";
        :>> createdAt = "2026-09-13"; :>> createdBy = "claudeFable5";
        :>> text = "I didn't care if the name is caps, and I usually want issues coming out of retros, not decisions. Usually retro findings are based on empirical evidence. If they're proposing a non occasion issue, it could be a decision, I suppose\n\nI wrote this text for the''retro'' item. Looks like it bless to all others?";
        :>> saidBy = "wweatherholtz"; :>> saidAt = "2026-09-13"; :>> channel = StatementChannel::chat;
    }

    part us110 : UserStory {
        :>> id = "dcb3edd0-33ad-422c-8bf7-3d16978b4041";
        :>> title = "A retro finding becomes an Issue; a Decision only when it is not a one-off";
        :>> createdAt = "2026-09-13"; :>> createdBy = "claudeFable5";
        :>> asA = "the human who reads what retros produce";
        :>> iWant = "retro findings recorded as Issues by default, because they rest on empirical evidence; a retro discharged by a Decision only where the finding proposes a standing rule rather than a one-off defect";
        :>> soThat = "what comes out of a retro is a tracked, closable defect, and the retro-backlog guard's Decision needle is the exception it reads, not the norm";
        :>> implication = ImplicationKind::process;
        :>> triageNote = "Given as the note on the nineteenth-publish copy-out, written for the retro ask (d0461, held). Bears on the retro-backlog guard: which discharge forms it should prefer, not how it spells them.";
    }
    #DerivedFrom dependency from us110 to st121;
}
