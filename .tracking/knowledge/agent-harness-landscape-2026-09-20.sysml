// KNOWLEDGE RECORD (D0196) - the agent-harness landscape spike and what the tool-owns-the-task design takes from it (D0540).
package KnowledgeAgentHarnessLandscape20260920 {
    private import EngineElement::*;
    private import EngineVerification::*;

    verification kAgentHarnessLandscape : Test {
        :>> id = "3b7d1f52-8c4e-4a9b-9e21-6f0d2c8a7b45";
        :>> title = "Landscape: ninety-four rows across sandboxes, task-owning harnesses, workflow engines, hermetic build and CI, and verification - nobody both defines the process and computes done, only build systems harvest declared outputs, the egress proxy holding the run credential is the converged sandbox shape";
        :>> createdAt = "2026-09-20"; :>> createdBy = "claudeFable5";
        :>> method = VerificationMethod::analyze;
        :>> procedureText = "REPLICATION (D0196). PROCESS: adversarialPanelReview (D0187, def pin 64f2dea), on the sprint 517 landscape-spike shape. TRIGGER: the human, 2026-09-20 (st184): 'Research alternative harnesses like this. We're talking bazel and GitHub actions, but surely there are sandbox container setups for AI agents. Any strongly considering custom processes and verification workflows? Other strengths we should integrate? Investigate 50 alternative tools, noting what has gained some following recently' - 'this' being the tool-owns-the-task design under brainstorm since st181, unapproved. ROSTER: five read-only web-enabled lens agents dispatched in parallel with one shared brief (the review's Appendix 0, verbatim) and a tool list each: A sandboxes and isolation runtimes (21 rows), B harnesses where a tracker or orchestrator owns the agent's task (19), C durable-execution and workflow engines and agent orchestration frameworks (23), D hermetic build systems and CI runners (16), E verification, spec-driven development and attestation (15); one round, no rebuttal. Each lens answered seven questions per tool (what and who; isolation and materialisation; step definition, typing, nesting, hooks, versioning; what makes a step done, HITL, receipts, drift; task ownership and lease; adoption with numbers and dates; one strength and one critique) and closed with the patterns of its lens and its sources. The synthesiser did not re-fetch the lenses' external facts; every lens claim about keel's own state was re-checked against source before use and names the check beside it. FINDINGS: (1) the empty quadrant - every process-defining tool lets the agent or a returned function say when a step is done, and every tool that computes done outside the agent owns no persistent work item and ends when the eval ends; (2) nobody materialises a step's declared inputs into an agent sandbox or harvests only its declared outputs - that is the build-system norm (Bazel, REAPI, Pants, Argo artifacts) and absent from every agent harness and CI; (3) the converged sandbox shape is a microVM or container behind an egress proxy that holds the only credential, with 'network off' unimplementable for a coding agent; (4) task-owning harnesses converge on per-task isolation, a claim FSM or lease with heartbeat, and a pull request as the only output; (5) workflow engines split into a journal-replay camp and a checkpoint camp, pin in-flight runs to the definition version, and pause into a typed request resumed by token for human input; (6) in-toto's MATCH rule is the one drift guard in the field, and Copilot's inability to approve its own pull request the one non-self-approval; (7) nobody records the human's words as the acceptance. DECISION: D0540 - saved as a spike, no tool adopted, no backlog item against the unapproved design; sixteen mechanisms feed the design draft; three critiques accepted into it, two held for the human. KEY LEARNING: the design sits where no tool sits, and the pieces it lacked exist and converge; the cost is that nothing can be bought, only mechanisms copied. ARTIFACTS: docs/reviews/agent-harness-landscape-2026-09-20.md (synthesis, Appendix 0 brief, Appendices A-E verbatim with sources); sprint 758; dcAgentHarnessLandscapeIsRecorded.";
    }
    part kAgentHarnessLandscapeR1 : TestResult {
        :>> id = "9e4c6a13-2d7f-4b58-8c0e-1a5b3d7f9e62";
        :>> outcome = VerdictKind::pass;
        :>> judgedAgainst = "68fb803bc14646ab17f5cb8f0106f9fcbd04223f";
        :>> judgedAt = "2026-09-20";
        :>> judgedBy = "claudeFable5";
        :>> notes = "Five lenses in one round, 94 rows above the fifty asked, every adoption number dated and URL-cited or written as not found; two lens claims about keel's own state re-checked against source and kept.";
    }
    dependency from kAgentHarnessLandscape to adversarialPanelReview;
    dependency from kAgentHarnessLandscape to d0187;
    dependency from kAgentHarnessLandscape to d0196;
    dependency from kAgentHarnessLandscape to d0540;
    dependency from kAgentHarnessLandscape to st184;
    dependency from kAgentHarnessLandscape to agentHarnessLandscapeSpikeStory;
    dependency from kAgentHarnessLandscape to dcAgentHarnessLandscapeIsRecorded;
}
