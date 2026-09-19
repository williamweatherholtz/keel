// Intake for 2026-09-19 — their words, my translation, the triage verdict (D0166/D0216).
// Written by `keel record statement` / `keel record story` (D0236): a Statement's text is
// VERBATIM, so it is escaped for the literal and otherwise untouched.
package ProjectIntake20260919 {
    private import EngineElement::*;
    private import EngineIntake::*;
    private import EngineRelationships::*;

    part st169 : Statement {
        :>> id = "e0f16b0d-a879-40ca-91dc-ae54f9e8a48d";
        :>> title = "Retro findings are empirical and become issues, not decisions";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> text = "I didn't care if the name is caps, and I usually want issues coming out of retros, not decisions. Usually retro findings are based on empirical evidence. If they're proposing a non occasion issue, it could be a decision, I suppose\n\nI wrote this text for the''retro'' item. Looks like it bless to all others?";
        :>> saidBy = "wweatherholtz"; :>> saidAt = "2026-09-19"; :>> channel = StatementChannel::chat;
    }

    part us123 : UserStory {
        :>> id = "c5b4ee50-a198-41ce-bc80-e2ee433dc304";
        :>> title = "A retro finding lands as an Issue; a Decision from a retro says what it proposes beyond the evidence";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> asA = "the human who reads retros";
        :>> iWant = "a retro finding that reports something observed to land as an Issue with a resolver, and to see a Decision come out of a retro only when the finding proposes something the evidence did not occasion";
        :>> implication = ImplicationKind::convention;
        :>> triageNote = "Their words arrived as the notes field of the fifty-second brief, typed for the previous page's retro item (d0510, a sitting review finished by analysis like the retro) and attached to the d0523/d0524 answers because the page has one notes field. The preference is already the practice: sprint 746's retro produced issue625-628 with resolvers and no Decision (D0131 tracked items, D0047 Issue plus control). What admits the other outcome is guard retro-backlog, which accepts a Decision d0NNN as a finding's stated home (D0461, guards.md row retro-backlog). Candidate control, not authored here because the words end in I suppose: a finding whose home is a Decision states what it proposes beyond the observation, else the guard warns. The caps clause cannot be placed: no accepted decision on either page mentions casing.";
    }
    #DerivedFrom dependency from us123 to st169;
}
