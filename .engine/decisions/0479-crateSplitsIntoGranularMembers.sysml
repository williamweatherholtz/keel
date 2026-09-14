// D0479 (status: the `status` field below is the truth) — crateSplitsIntoGranularMembers: keel-cli decomposes into many workspace members along the computed dependency layers, leaves first, one extraction per sprint, cycles broken before the boundary they cross is drawn
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0479Accept event on sign-off.
package Decision0479 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    part d0479 : Decision {
        :>> id = "2e08f768-d76d-4bd8-b07d-2bec8457db61";
        :>> title = "crateSplitsIntoGranularMembers: keel-cli decomposes into many workspace members along the computed dependency layers, leaves first, one extraction per sprint, cycles broken before the boundary they cross is drawn";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::accepted;
        :>> context = "keel-cli is one crate of 66,548 lines across 68 modules plus a 6,049-line main.rs and a 9,415-line guards.rs. Cargo's incremental unit is the crate, so any edit recompiles all of it and every test binary that links it; the human sees 45-50 minute rebuild-and-retest cycles. Their words (st126): 'totally agree on splitting the crate into workspace members. give us lots of granularity - no need to couple code just to keep it in one place. making it broad and uncoupled makes it modular and easy to rebuild.' The intra-crate graph was computed on 2026-09-14 (crate::X references per module): gitx has fan-in 34 and fan-out 1, json 18, perf 15, actor 10; guards depends on 29 modules; guards, view and orient form a three-way cycle; write and scaffold carry back-edges into guards and view.";
        :>> decision = "keel-cli is decomposed into workspace members along the computed dependency layers, and a member is drawn only where the graph already allows it: leaves first (git, perf, json, actor, schema, cli facts, device, github), then the read model, the write API, the views, the guards, the build-and-test tooling, the engine processes, the item processes (D0480), serve, and finally a thin keel-cli that only dispatches. A cycle is broken in its own sprint before the boundary it crosses is drawn. Each extraction keeps the CLI surface identical - the moved module is re-exported under its old path so no caller changes in the same commit - and is one sprint with its own DoD. A member's Cargo dependency list is its declared read set: Cargo's fingerprint then recompiles a member only when it or a member it depends on changed, which is the descent the human described - the workspace hash moves, one member's hash moves, only that branch is rebuilt. NOT A FORK: the human chose granularity in st126; the order is computed from the graph, not weighed. NOT A PROCESS CHANGE: code structure only.";
        :>> rationale = "Splitting where a cycle exists produces a boundary Cargo refuses, so the cycles are the work and the leaves are free. Re-exporting under the old path makes each extraction a mechanical move whose only observable is compile time, which keeps every sprint independently landable and every regression bisectable to one move. Granularity is the point, not a cost: a member nobody edits is a member nobody rebuilds.";
        :>> consequences = "Sprints dcWorkspaceHoldsTheLeafMembers through dcKeelCliIsThinDispatch, in declaration order, each chartered by this Decision. The touched-set and guard receipts follow the members (D0481, D0482). A layering guard is proposed separately and held for the human (D0483). The module graph script that produced the order is a one-off; the enforced fact is the Cargo dependency list itself.";
    }
    #DerivedFrom dependency from d0479 to st126;

    // acceptance event (D0121 review-queue sign-off; D0066/D0106 — human-judged, not fabricated)
    verification d0479Accept : Test { :>> id = "b7b6d9fd-4f86-4440-9901-8abedf6114db"; :>> method = VerificationMethod::confirmation; :>> procedureText = "AUTO-ACCEPTED under standing consent (D0207). Their standing words, verbatim: 'issues raised are automatically accepted. they can be customizedly changed post-fact.' Not individually reviewed; override by a superseding Decision (D0290) or the human's quoted word in chat (D0289)."; }
    part d0479AcceptR1 : TestResult { :>> id = "99765dc1-54f0-4780-a04f-2a7bd8ad92f1"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "13eb23b"; :>> judgedAt = "2026-09-14"; :>> judgedBy = "wweatherholtz"; :>> createdBy = "claudeFable5"; }
}
