// =====================================================================
// ModularMembersComponents - appArch output of the st126 modularity program (sprint 721, D0479):
// the three target structures the eight requirements in
// .tracking/requirements/modular-members-requirements.sysml land on. A Component here is the SHAPE
// the Delivery sprints build toward, not an allocation target: D0194 retired Component allocation,
// so an SR is allocated to a registered CodeElement once one carries it (allocations.sysml) and the
// charter chain (SR -> #CharteredBy work -> delivered code) is the other half. The keel-issues
// Component names what D0480 put inside the member and what it left out; the Decision on where a
// project's OWN Decision lives is held for the human (recorded beside sprint 721), so the member's
// closure is stated here without that one item either way.
// =====================================================================
package ModularMembersComponents {
    private import EngineElement::*;
    private import EngineArchitecture::*;
    private import EngineRelationships::*;

    part compKeelIssuesMember : Component {
        :>> id = "bc98d4c7-7f61-4e5c-8749-cf613263300f";
        :>> title = "keel-issues - the item/issue member";
        :>> createdAt = "2026-09-15"; :>> createdBy = "claudeFable5";
        :>> description = "Workspace member members/keel-issues (library plus a thin binary) holding the item and issue processes D0480 names: record issue / task / story / statement, intake write, GitHub ingest, plan cover, the resolution computation, and the lenses open-issues, intake, dispositions, whats-next, ready, blocked, done. Depends on keel-view and below (keel-model, keel-write, keel-parser, keel-json, keel-fs, keel-actor, keel-git), never on the process member or the guards member. Its layer sits above keel-view and below keel-cli in the manifest layer table (ssrLayerTableLivesInTheWorkspaceManifest).";
    }

    part compProjectProcessBinding : Component {
        :>> id = "046a1a86-d145-478a-8ec0-9015e8ec844a";
        :>> title = "Project process binding - resolver, test:<name> step check, process-gates rung";
        :>> createdAt = "2026-09-15"; :>> createdBy = "claudeFable5";
        :>> description = "Three pieces, one member (the process member D0479 schedules for cursor.rs and process_cmd.rs): (1) the process resolver reads .engine/processes/ and the project's .tracking/processes/ through one function; (2) the step-check resolver gains the binding kind test:<name>, a Test in the project's model whose latest ancestor-of-HEAD result decides the step; (3) the process-gates rung, one function in the guards member, lists every adopted process, computes each run's position with resolver (2), and emits one fixed-format verdict line per step taken past a red earlier step, which the fast tier, the commit tier and the CI gate job print unchanged. The subject of the three held process-change Decisions recorded with sprint 721.";
    }

    part compWorkspaceLayering : Component {
        :>> id = "08867efe-1fc3-4785-b6c3-0ecc357d5ad4";
        :>> title = "Workspace layering - the manifest layer table and the member-layering guard";
        :>> createdAt = "2026-09-15"; :>> createdBy = "claudeFable5";
        :>> description = "The one layer table under [workspace.metadata.keel.layers] in the workspace Cargo.toml (one key per [workspace] member), the stays-in-binary list with a reason per keel-cli module that is not scheduled out, and the member-layering guard (D0483) that reads both: a dependency pointing to the same or a higher layer is a red guard naming the edge; a member missing from the table is red naming the member; a keel-cli module with neither a scheduling backlog item nor a stays-in-binary reason is red naming the module. scripts/module_home.py and the build skill read the same table.";
    }
}
