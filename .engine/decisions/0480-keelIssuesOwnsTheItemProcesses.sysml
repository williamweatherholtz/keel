// D0480 (status: the `status` field below is the truth) — keelIssuesOwnsTheItemProcesses: the item/issue subset of the processes - record issue, task, story and statement, intake, GitHub ingest, resolution and open-issues views - lives in member keel-issues, apart from the processes that govern keel itself
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0480Accept event on sign-off.
package Decision0480 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    part d0480 : Decision {
        :>> id = "dc01a05a-ad0e-478d-aaad-b7d6179b4b33";
        :>> title = "keelIssuesOwnsTheItemProcesses: the item/issue subset of the processes - record issue, task, story and statement, intake, GitHub ingest, resolution and open-issues views - lives in member keel-issues, apart from the processes that govern keel itself";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::accepted;
        :>> context = "The human, on being offered an external tracker for the ticket layer (st126): 'I say let's just make keel-issues or something, which can manage the subset of processes related to items/issues (and not e.g. decisions about keel itself).' Today that subset is spread across main.rs, write.rs, intake_write.rs, github_ingest.rs, pm.rs, plan_cover.rs and view/, in the same crate as decision recording, activation and the guards on the engine.";
        :>> decision = "A workspace member keel-issues owns the code that interprets the item and issue processes: recording an issue, task, story or statement, intake and GitHub ingest, the resolution computation and the open-issues, intake and dispositions views. The processes that govern keel itself - decisions, acceptance, activation, guards, change requests - stay out of it. The process text in .engine/processes/ does not move and does not change; what changes is which member's code carries each process, so a project that only tracks items can depend on keel-issues without the engine-governance stack. NOT A PROCESS CHANGE: every process keeps its text, steps and gates; this assigns code to members. NOT A FORK: the human named the member and the boundary in st126.";
        :>> rationale = "The boundary the human drew is the one a tracker would have drawn from outside, drawn inside instead: items and their state on one side, the engine's own governance on the other. Making it a Cargo member makes the boundary enforced rather than described, and gives the tracker-shaped subset a rebuild unit of its own.";
        :>> consequences = "Sprint dcKeelIssuesIsAMember, after the view member exists. The CLI facts for the record verbs it carries name the member in their synopsis when it lands. Nothing about the processes' adoption (activation.toml) changes.";
    }
    #DerivedFrom dependency from d0480 to st126;

    // acceptance event (D0121 review-queue sign-off; D0066/D0106 — human-judged, not fabricated)
    verification d0480Accept : Test { :>> id = "d5cd185d-0232-4594-8231-9e798655ded1"; :>> method = VerificationMethod::confirmation; :>> procedureText = "AUTO-ACCEPTED under standing consent (D0207). Their standing words, verbatim: 'issues raised are automatically accepted. they can be customizedly changed post-fact.' Not individually reviewed; override by a superseding Decision (D0290) or the human's quoted word in chat (D0289)."; }
    part d0480AcceptR1 : TestResult { :>> id = "f3673ce3-805d-4dfc-bdf3-743b627c2d4d"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "13eb23b"; :>> judgedAt = "2026-09-14"; :>> judgedBy = "wweatherholtz"; :>> createdBy = "claudeFable5"; }
}
