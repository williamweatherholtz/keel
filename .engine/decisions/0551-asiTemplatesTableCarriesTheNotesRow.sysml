// D0551 (status: the `status` field below is the truth) — asiTemplatesTableCarriesTheNotesRow: the asi-templates skill's section table gains row 9, Notes - one note field per ask inside its tab, held by the brief checker - the process-change record for the skill edit D0550 requires
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0551Accept event on sign-off.
package Decision0551 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    // #ProspectiveChange (D0070) - this Decision governs a process/schema change; the co-committed
    // change rides the process-change guard's hard lock.
    #ProspectiveChange part d0551 : Decision {
        :>> id = "28b302ab-620b-4991-8a20-a30442eb3b02";
        :>> title = "asiTemplatesTableCarriesTheNotesRow: the asi-templates skill's section table gains row 9, Notes - one note field per ask inside its tab, held by the brief checker - the process-change record for the skill edit D0550 requires";
        :>> createdAt = "2026-09-22";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::accepted;
        :>> context = "D0550 (accepted 2026-09-22 under standing consent, on the human's words in st192: 'fix artifact permanently so the comment section is unique per tab. today, it's shared and i can't uniquely give feedback properly') makes one note field per ask the brief contract and adds the clause to scripts/check_templates.py --brief. The asi-templates skill's section-2.1 table is where every brief budget names its holder (issue413), and the skill file under .engine/skills is a process definition under the D0070 hard lock extended by D0209 clause 2, so the row cannot land without a marked Decision co-committed. This is that record; the substance is D0550's.";
        :>> decision = "Row 9 is added to the asi-templates skill's section table in .engine/skills/asi-templates/SKILL.md and its .claude mirror: Notes - one note field per ask, inside its tab, keyed to the ask; the digest emits each beneath its own answer, never once for the page (D0550) - budget one per tab, held by check_templates.py --brief. No other line of the skill or of any process moves. This is a process change in the D0070 sense only because the skill file is a locked path; the human directed the shape in st192.";
        :>> rationale = "The table is the one home for what the brief must carry and who holds it; a clause in the checker with no row in the table is a control the skill does not name, which is the issue413 shape the census exists to catch. Recording the row's Decision separately from D0550 keeps one clause per Decision (D0306) and lets the lock see a marked record beside the locked file.";
        :>> consequences = "The process-change guard passes on the commit carrying the skill edit; the budgets census reads the new row and finds its holder; nothing else changes.";
    }

    // acceptance event (D0121 review-queue sign-off; D0066/D0106 — human-judged, not fabricated)
    verification d0551Accept : Test { :>> id = "277360fe-a749-4d3c-b9fd-0bfd6d9eacb1"; :>> method = VerificationMethod::confirmation; :>> procedureText = "Their words, st192 (chat, 2026-09-22): 'fix artifact permanently so the comment section is unique per tab. today, it's shared and i can't uniquely give feedback properly' - the direction this skill row and D0550 carry out - their words, verbatim: “fix artifact permanently so the comment section is unique per tab. today, it's shared and i can't uniquely give feedback properly” WARN: read-back - the quoted words do not name d0551 (D0201 B); recorded as given (D0423)."; }
    part d0551AcceptR1 : TestResult { :>> id = "a8c01799-32b8-49b1-81a1-02124e308dc5"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "b65a13e4"; :>> judgedAt = "2026-09-22"; :>> judgedBy = "wweatherholtz"; :>> createdBy = "claudeFable5"; }
}
