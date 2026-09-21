// Intake for 2026-09-21 — their words, my translation, the triage verdict (D0166/D0216).
// Written by `keel record statement` / `keel record story` (D0236): a Statement's text is
// VERBATIM, so it is escaped for the literal and otherwise untouched.
package ProjectIntake20260921 {
    private import EngineElement::*;
    private import EngineIntake::*;
    private import EngineRelationships::*;

    part st187 : Statement {
        :>> id = "7ef42f81-6411-48f4-b2be-fdd2f139c8b5";
        :>> title = "GH#97 authority-queue's contentionAdjudication row points at 'keel contentions', which is not a command";
        :>> createdAt = "2026-09-21"; :>> createdBy = "githubRecorder";
        :>> text = "`keel show authority-queue .` emits a row of kind contentionAdjudication whose shortName reads ''see `keel contentions` | conflicting conclusions, unadjudicated''. Running `keel contentions .` prints the top-level usage; the lens that exists is `keel show contentions .`. A reader following the row's own instruction gets the usage screen.\n\nSeen on keel 0.5.0 in the plotwright project on 2026-09-20, after `keel judge-set --all` passed two gates an agent had earlier failed (accurateChecksCloseOutGate, taskEngineResyncDoD), which is the contention the row reports.\n\nSuggested fix: the row names `keel show contentions`. Separately, `judge-set --all` might warn before it passes a result whose latest verdict is fail, since that is exactly what creates a contradictoryJudgment.\n";
        :>> saidBy = "williamweatherholtz"; :>> saidAt = "2026-09-20"; :>> channel = StatementChannel::github;
        :>> sourceUrl = "https://github.com/williamweatherholtz/keel/issues/97";
        :>> sourceTrust = SourceTrust::untrusted;
    }
}
