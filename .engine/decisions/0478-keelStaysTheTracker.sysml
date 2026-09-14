// D0478 (status: the `status` field below is the truth) — keelStaysTheTracker: no external issue tracker is adopted; plain-SysML processes, interpreted and guarded by keel, remain how process is enforced and state exposed
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0478Accept event on sign-off.
package Decision0478 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    part d0478 : Decision {
        :>> id = "8ed19336-9802-4f6f-90a7-f1fe58eae17e";
        :>> title = "keelStaysTheTracker: no external issue tracker is adopted; plain-SysML processes, interpreted and guarded by keel, remain how process is enforced and state exposed";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::accepted;
        :>> context = "After twenty engine-only sprints I asked whether the project was reinventing Jira/Linear and drew a fork between keeping everything in keel and handing the ticket layer to a tracker (2026-09-14). D0374 had saved the tracker comparison as a spike with nothing adopted. The human's answer is st126: 'use an issue tracker to enforce process and track/expose state. I think that's fine, but I do think Keel does it better in many ways - we can read plain sysml processes, interpret them, add guards (really jsut CI/CD), etc. This has become a more and more obvious need as this project goes forward - how to hook into custom processes that have absolute discipline?'";
        :>> decision = "No external issue tracker is adopted, trialled or integrated. The way process is enforced and state exposed stays keel's: processes are plain SysML text, interpreted by the binary, with guards as the CI/CD that holds them; state is computed from the tree, never stored. The fork I drew is answered from the human's side: the question was never keel versus a tracker but how granular keel's own parts are (D0479, D0480). NOT A FORK: the human chose, in st126, and this records the choice. NOT A PROCESS CHANGE: no workflow, gate, guard or schema changes; this fixes the direction the modularity pass serves.";
        :>> rationale = "The one capability a tracker would add - a hosted surface for humans - was already judged a later Decision by D0374, and the capability keel has that no tracker offers is the one the human names: custom processes with absolute discipline, readable as text and enforced by guards. Rebuild cost and coupling, the pain that prompted the question, are properties of one 66,548-line crate, not of the approach, and are answered by decomposition.";
        :>> consequences = "D0374 stands unchanged. The modularity pass (D0479) is chartered by this direction; the item/issue process subset gets its own member (D0480) instead of an external home. Any later wish for a hosted human surface is a new Decision carrying its own trial, as D0374 already says.";
    }
    #DerivedFrom dependency from d0478 to st126;

    // acceptance event (D0121 review-queue sign-off; D0066/D0106 — human-judged, not fabricated)
    verification d0478Accept : Test { :>> id = "a67fe4d7-6de3-4173-9c03-e5f28cc8a860"; :>> method = VerificationMethod::confirmation; :>> procedureText = "AUTO-ACCEPTED under standing consent (D0207). Their standing words, verbatim: 'issues raised are automatically accepted. they can be customizedly changed post-fact.' Not individually reviewed; override by a superseding Decision (D0290) or the human's quoted word in chat (D0289)."; }
    part d0478AcceptR1 : TestResult { :>> id = "1239c497-85d0-49a7-8157-f81f5b3ceb06"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "13eb23b"; :>> judgedAt = "2026-09-14"; :>> judgedBy = "wweatherholtz"; :>> createdBy = "claudeFable5"; }
}
