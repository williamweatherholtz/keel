// Intake for 2026-09-20 — their words, my translation, the triage verdict (D0166/D0216).
// Written by `keel record statement` / `keel record story` (D0236): a Statement's text is
// VERBATIM, so it is escaped for the literal and otherwise untouched.
package ProjectIntake20260920 {
    private import EngineElement::*;
    private import EngineIntake::*;
    private import EngineRelationships::*;

    part st181 : Statement {
        :>> id = "d969341c-aa8c-4634-919c-2aad4f3e2bb2";
        :>> title = "A branch of keel where the tool owns AI tasks, deterministic control first, customizable processes with inputs, outputs, hooks";
        :>> createdAt = "2026-09-20"; :>> createdBy = "claudeFable5";
        :>> text = "Let's make a branch of keel where the tool owns AI tasks instead of the reverse. We mentioned this earlier, a-la Jira, Linear, etc. I want to use the great stuff we've made so far, just with absolute deterministic control as a starting point. And strong customizable processes with clear inputs, outputs, hooks, and definition\n\nLet's br";
        :>> saidBy = "wweatherholtz"; :>> saidAt = "2026-09-20"; :>> channel = StatementChannel::chat;
    }

    part st182 : Statement {
        :>> id = "55ea1a75-0127-47bc-bca0-104da1fcd911";
        :>> title = "Browser UI, ditch the CLI; client-server with narrow parallel agents; a process that defines processes - inputs, outputs, steps, keel/agent ports, nesting";
        :>> createdAt = "2026-09-20"; :>> createdBy = "claudeFable5";
        :>> text = "C, but I'm interested in going fully browser UI, I think and effectively ditching the cli. It's brought more issues than solutions (how many EOL bugs again?).\n\nAnyway, we've developed some truly excellent ideas, some of which I feel are best-in-class for my work. And I don't want to lose that, but I do want a full client-server architecture here that gives AI agents much more defined and narrow scope which can run in parallel and have more definite inputs and outputs. And keel can still detect discrepancies and lapsed guards, etc.\n\nI envision it working thusly - a process is defined which develop processes. That process is to define inputs, outputs, and steps. At the start/end of a step, what goes into or out of keel? What goes into or out of an agent? Then chain together steps ad nauseum until the process is defined. And a process can be made to use other processes nested-like. Is that defensible? Is it free from critique?";
        :>> saidBy = "wweatherholtz"; :>> saidAt = "2026-09-20"; :>> channel = StatementChannel::chat;
    }

    part st183 : Statement {
        :>> id = "e955f0a4-dd99-482d-94ed-7102f0945c02";
        :>> title = "Both step kinds: a constrained container option and an unconstrained agent option; agents can build and test toolchains; the bash issue seems insurmountable";
        :>> createdAt = "2026-09-20"; :>> createdBy = "claudeFable5";
        :>> text = "Let's just do both. A constrained container option and and an unconstrained agent option. Or is that not a good idea?\n\nIf not, I do think we do it. Agents can help make and test viable tool chains without too much difficulty, I expect. Though the bash issue still seems insurmountable";
        :>> saidBy = "wweatherholtz"; :>> saidAt = "2026-09-20"; :>> channel = StatementChannel::chat;
    }

    part st184 : Statement {
        :>> id = "a8dc5124-4799-4f6b-beb9-13c63332e07e";
        :>> title = "Research alternative harnesses: sandbox containers for AI agents, custom processes and verification workflows, 50 tools, what gained a following recently";
        :>> createdAt = "2026-09-20"; :>> createdBy = "claudeFable5";
        :>> text = "Research alternative harnesses like this. We're talking bazel and GitHub actions, but surely there are sandbox container setups for AI agents. Any strongly considering custom processes and verification workflows? Other strengths we should integrate? Investigate 50 alternative tools, noting what has gained some following recently";
        :>> saidBy = "wweatherholtz"; :>> saidAt = "2026-09-20"; :>> channel = StatementChannel::chat;
    }

    part st185 : Statement {
        :>> id = "0836a4b1-7366-499d-9c1d-d0f4f11a3df4";
        :>> title = "Direction: the tool-owns-the-task work is a keel subsystem/mode, possibly a totally separate project because it is architecturally distinct";
        :>> createdAt = "2026-09-20"; :>> createdBy = "claudeFable5";
        :>> text = "Yes I meant a keel subsystem/mode, though I might make it a totally separate project because I feel like it's quite distinct architecturally.";
        :>> saidBy = "wweatherholtz"; :>> saidAt = "2026-09-20"; :>> channel = StatementChannel::chat;
    }

    part st186 : Statement {
        :>> id = "012caff1-5cc2-414f-953f-d7b43a93c217";
        :>> title = "Direction: runner daemons claim leased steps (option ii) wherever feasible";
        :>> createdAt = "2026-09-20"; :>> createdBy = "claudeFable5";
        :>> text = "I think (ii) is right where feasible.";
        :>> saidBy = "wweatherholtz"; :>> saidAt = "2026-09-20"; :>> channel = StatementChannel::chat;
    }
}
