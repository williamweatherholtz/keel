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
}
