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

    part st190 : Statement {
        :>> id = "410d847d-5e7d-428f-b913-da8fd128daec";
        :>> title = "Direction: critique the runner-protocol design direction with a panel, publish an artifact, and make the case that it is the right direction";
        :>> createdAt = "2026-09-22"; :>> createdBy = "claudeFable5";
        :>> text = "critique this direction with a panel, make an artifact, and convince me this is the right direction.";
        :>> saidBy = "wweatherholtz"; :>> saidAt = "2026-09-22"; :>> channel = StatementChannel::chat;
    }

    part st191 : Statement {
        :>> id = "73d5fec2-abc3-4034-8a36-95843daff308";
        :>> title = "Direction for slice one: a cleanly deterministic versus non-deterministic handoff - clear inputs to the agent, keel tests the output, failed output feeds back to the agent, minimal context for agents, as much munging as possible programmatic and out of AI context";
        :>> createdAt = "2026-09-22"; :>> createdBy = "claudeFable5";
        :>> text = "for slice one: critical we get this right.  the handoff and interface needs to be cleanly determinstic vs non-detemrinistic.  e.g. make it clear what the inputs to the agent are, then test its output.  if the output fails (as measured by keel), then the agent gets feedback and tweaks.  i want minimal overhead context for agents, so as much munging programatically out-of-ai-context";
        :>> saidBy = "wweatherholtz"; :>> saidAt = "2026-09-22"; :>> channel = StatementChannel::chat;
    }

    part st192 : Statement {
        :>> id = "e97712c2-01c4-4601-9551-51082f07fb71";
        :>> title = "Correction: the brief's comment field must be unique per tab so feedback attaches to one ask";
        :>> createdAt = "2026-09-22"; :>> createdBy = "claudeFable5";
        :>> text = "fix artifact permanently so the comment section is unique per tab.  today, it's shared and i can't uniquely give feedback properly";
        :>> saidBy = "wweatherholtz"; :>> saidAt = "2026-09-22"; :>> channel = StatementChannel::chat;
    }

    part st193 : Statement {
        :>> id = "bd094b16-8b87-4bbc-b070-9746d4a2fb21";
        :>> title = "Correction: no placeholders, parentheticals or inserted comments on human-facing surfaces; cleaner is better";
        :>> createdAt = "2026-09-22"; :>> createdBy = "claudeFable5";
        :>> text = "''optional, on this ask only''.  Remove the parentheticals.  Don't insert distracting comments; support smooth UX and use cases.  Stop trying to add verbose data everywhere; cleaner is better.";
        :>> saidBy = "wweatherholtz"; :>> saidAt = "2026-09-22"; :>> channel = StatementChannel::chat;
    }

    part st194 : Statement {
        :>> id = "a0acd8c5-0c65-441d-b1af-79c496b32bf4";
        :>> title = "Correction: the brief's slice-one 'What changes' line was unreadable jargon; say what the command does in plain words";
        :>> createdAt = "2026-09-22"; :>> createdBy = "claudeFable5";
        :>> text = "and i have no idewa what you're geting at ''One verb:... '' what.are.you.talking.about. lol";
        :>> saidBy = "wweatherholtz"; :>> saidAt = "2026-09-22"; :>> channel = StatementChannel::chat;
    }

    part st195 : Statement {
        :>> id = "01282730-d370-4d54-9df3-29eabe97410f";
        :>> title = "Correction: the brief gives process prose instead of the facts that delineate each decision; the job is plain relevant content, a few decision-driving metrics, the consequential facts per option, less recommendation";
        :>> createdAt = "2026-09-22"; :>> createdBy = "claudeFable5";
        :>> text = "The commentary in the artifact is currently inane.  You're giving me prose telling me that it's time to choose, instead of just giving me context so i can actually do so.  I'm trying to make decisions here, not follow a process.  Your job is to follow a process, though - the process of making relevant content plain, picking a few metrics that are relevant to drive decision making (not just numbers for numbers sake), and clearly illustrate the consequential facts that delineate a decision.  I don't need as much of a recommendation if you can identify the relevant information";
        :>> saidBy = "wweatherholtz"; :>> saidAt = "2026-09-22"; :>> channel = StatementChannel::chat;
    }
}
