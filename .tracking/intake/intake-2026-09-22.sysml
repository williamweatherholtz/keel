// Intake for 2026-09-22 — their words, my translation, the triage verdict (D0166/D0216).
// Written by `keel record statement` / `keel record story` (D0236): a Statement's text is
// VERBATIM, so it is escaped for the literal and otherwise untouched.
package ProjectIntake20260922 {
    private import EngineElement::*;
    private import EngineIntake::*;
    private import EngineRelationships::*;

    part st189 : Statement {
        :>> id = "2df0190e-7145-4b1b-acf7-a316506d70bf";
        :>> title = "GH#98 keel init scaffolds keel-gate.yml with the pre-D0452 verbs, so a fresh project's CI gate is red on its first push";
        :>> createdAt = "2026-09-22"; :>> createdBy = "githubRecorder";
        :>> text = "## What happened\n\n`keel init` scaffolds `.github/workflows/keel-gate.yml`. On keel 0.5.0 that workflow fails on its first push with exit code 2, because its steps call the verbs that D0452 folded away:\n\n```\nkeel validate .\nkeel guard .\nkeel rules . --enforce\nkeel audit-history --since origin/main\n```\n\nThe 0.5.0 binary answers each with the usage text of the `keel gate` router and exits 2. The pinned install step works (it reads `engine-version.toml` and downloads the release), so the failure lands on the first gating step, after a green install, and reads as ''the project's facts are bad'' rather than ''the template is stale''.\n\nObserved in williamweatherholtz/plotwright: run 35631021976 (commit 4b51bc1) and run 35540937910 (commit 37c9318), both `failure` at the `validate` step with the usage text in the log.\n\n## Why it matters\n\nD0452 says the commit that removes the names rewrites every CI workflow under `.github/workflows` and passes its own rewritten CI. The scaffold that `keel init` writes for a *new* project is a call site too, and it was not rewritten, so every project initialised on 0.5.0 inherits a red gate. That is the issue327 shape again (a scaffolded gate that reports the wrong thing), one release later.\n\n## Two smaller things in the same file\n\n- The download URL is `github.com/williamweatherholtz/sysmlv2-ai-toolkit/releases/...`. It works today only because GitHub 301-redirects the renamed repository to `williamweatherholtz/keel`. The template should name the current repository.\n- The `audit-history` step ends in `|| true`, so it can never fail the workflow. If it is informational, the template should say so; if it is a gate, the `|| true` hides it.\n\n## Suggested fix\n\nRewrite the template to the 0.5.0 verbs, or to the one tiered command the pre-commit hook already runs:\n\n```\nkeel gate --workspace .\nkeel audit history --since origin/main\n```\n\nand add the scaffolded workflow to the D0452 byte-equality set so the next fold catches it.\n\n## Workaround used downstream\n\nplotwright deleted the scaffolded workflow (its own header says to, once CI wires keel itself) and runs `keel gate --workspace .` from its own workflow, with the binary downloaded from the pinned release and checked against the published sha256.\n";
        :>> saidBy = "williamweatherholtz"; :>> saidAt = "2026-09-21"; :>> channel = StatementChannel::github;
        :>> sourceUrl = "https://github.com/williamweatherholtz/keel/issues/98";
        :>> sourceTrust = SourceTrust::untrusted;
    }
}
