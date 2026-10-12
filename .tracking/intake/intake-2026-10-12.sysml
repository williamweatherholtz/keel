// Intake for 2026-10-12 — their words, my translation, the triage verdict (D0166/D0216).
// Written by `keel record statement` / `keel record story` (D0236): a Statement's text is
// VERBATIM, so it is escaped for the literal and otherwise untouched.
package ProjectIntake20261012 {
    private import EngineElement::*;
    private import EngineIntake::*;
    private import EngineRelationships::*;

    part st199 : Statement {
        :>> id = "4adb151e-028b-4d02-9168-fc94650be4a2";
        :>> title = "GH#99 exec-summary contract and browser suite floor the page at two tabs and three choices; a one-tab, two-option or records-only page cannot pass";
        :>> createdAt = "2026-10-12"; :>> createdBy = "githubRecorder";
        :>> text = "## Summary\n\nThe exec-summary (decision page) contract and its browser suite assume at least two tabs on the page and at least three choices on the first decision. A page that honestly carries one pending decision, a decision with two real options, or a records-only page (nothing pending, only the sitting's records) cannot pass the engine's own gate.\n\n## The two floors\n\n1. `.engine/scripts/check_templates.py` (the structure contract) fails a page with one tab:\n\n   ```\n   tabs/panels mismatch: 1 tabs, 1 panels (need >=2, paired)\n   ```\n\n2. `.engine/tests/exec_summary/test_exec_summary.py:144` (`test_selections_persist_across_reload`) clicks `label.choice` `nth(2)` of the first decision, so a first decision with two options times out instead of passing.\n\nA records-only page fails the suite outright, since it has no decision for either test to find.\n\n## How it was hit\n\nOn 2026-09-30 a downstream project (Penumbra) wired the page builder to run the engine's browser suite after writing the page (its issue336). The next page held exactly one pending decision and was refused by the contract above. The page was brought within the floors by hand that day. Since then the builder carries a local workaround for a one-tab page (its D1147): it skips the two tab-switch tests and checks the contract with the tab count removed (`TWO_TAB_TESTS` and `contract_failures_one_tab` in `tools/build_decision_page.py`). Every one-tab sitting pays that workaround, and it only exists downstream.\n\nThe owner's direction when this surfaced (2026-09-30):\n\n> The number of tabs is irrelevant. Because the software has spokes, and spokes can make tabs (or no?), then the number of tabs is variable. I don't care how many table - I care about good UX\n\n## Ask\n\nThat the contract and the suite accept each of these as a valid page:\n\n- a one-tab page (one pending decision);\n- a decision with two options (the persistence test should pick an option that exists, for example `nth(1)` or the last option, rather than `nth(2)`);\n- a records-only page (no pending decision; the sitting's records as the appendix).\n\nThe pairing check itself (tabs paired with panels) is sound; only the `>=2` floor and the fixed `nth(2)` are the problem.\n\nFiled on the owner's behalf from a Claude Code session, with their word on the page.\n";
        :>> saidBy = "williamweatherholtz"; :>> saidAt = "2026-10-11"; :>> channel = StatementChannel::github;
        :>> sourceUrl = "https://github.com/williamweatherholtz/keel/issues/99";
        :>> sourceTrust = SourceTrust::untrusted;
    }
}
