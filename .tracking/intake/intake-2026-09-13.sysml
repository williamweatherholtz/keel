// Intake for 2026-09-13 — their words, my translation, the triage verdict (D0166/D0216).
// Written by `keel record statement` / `keel record story` (D0236): a Statement's text is
// VERBATIM, so it is escaped for the literal and otherwise untouched.
package ProjectIntake20260913 {
    private import EngineElement::*;
    private import EngineIntake::*;
    private import EngineRelationships::*;

    part st121 : Statement {
        :>> id = "62dc0dd6-adfb-4028-a06f-512e7b63284a";
        :>> title = "GH#71 keel accept appends a verification named <id>Accept, colliding with a hand-authored one of the same name";
        :>> createdAt = "2026-09-13"; :>> createdBy = "githubRecorder";
        :>> text = "**keel 0.3.1.** A project Decision file authored by hand in the shape the docs show (`#ProspectiveChange part d0054 : Decision {...}` plus a `verification d0054Accept : Test { method = confirmation; procedureText = ''...'' }` describing the planned human attestation) collides with `keel accept d0054`: the command appends its own\n\n```\nverification d0054Accept : Test { ... method = VerificationMethod::confirmation; procedureText = ''<the --note>''; }\npart d0054AcceptR1 : TestResult { ... }\n```\n\nso `keel guard duplicate-identity` then FAILs (`duplicate declared name d0054Accept in package ProjectDecision0054`) and the commit is blocked right after the human accepted. Seen 2026-09-12 on obsidian-selfsync for d0054/d0055/d0056 (the same will happen to any file that already carries a `<id>Accept` verification).\n\n**Ask:** either (a) `keel accept` reuses an existing `<id>Accept` verification (append only the TestResult), or (b) it names its record distinctly (`<id>AcceptEvent`), or (c) `keel guard` / `keel validate` warns BEFORE acceptance that a hand-authored `<id>Accept` will collide. (a) seems most natural — the hand-authored procedureText is exactly the attestation the result records.\n\n**Workaround used:** delete the hand-authored placeholder after acceptance and keep keel's record.";
        :>> saidBy = "williamweatherholtz"; :>> saidAt = "2026-09-12"; :>> channel = StatementChannel::github;
        :>> sourceUrl = "https://github.com/williamweatherholtz/sysmlv2-ai-toolkit/issues/71";
        :>> sourceTrust = SourceTrust::untrusted;
    }

    part st122 : Statement {
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
    #DerivedFrom dependency from us110 to st122;
}
