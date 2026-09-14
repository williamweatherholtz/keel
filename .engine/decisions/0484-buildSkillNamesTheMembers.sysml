// D0484 (PROPOSED — NOT YET ACCEPTED) — The build skill describes the workspace as its members, not as two crates
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0484Accept event on sign-off.
package Decision0484 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    // #ProspectiveChange (D0070) - this Decision governs a process/schema change; the co-committed
    // change rides the process-change guard's hard lock.
    #ProspectiveChange part d0484 : Decision {
        :>> id = "31208838-fe7b-4b84-9d83-5159fe37105c";
        :>> title = "The build skill describes the workspace as its members, not as two crates";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::proposed;
        :>> context = "The build skill (.engine/skills/build/SKILL.md) told the reader the Cargo workspace is two crates, keel-parser and keel-cli, compiled under deny(warnings). Sprint 714, the first D0479 extraction, added six leaf members under members/ (keel-perf, keel-git, keel-json, keel-actor, keel-schema, keel-github), each a library keel-cli re-exports under its old module path. A skill that names a layout the tree no longer has misleads the next builder about where a module lives and which crate a failing test belongs to. The skill file is a locked process definition (D0070), so even this doc-sync line needs a co-committed marked Decision (D0209 clause 2).";
        :>> decision = "The build skill's 'Workspace layout' paragraph names the members/ crates as a layer of the workspace - each a lib re-exported by keel-cli - alongside keel-parser and keel-cli, and says all of them compile under the same deny set. Nothing else in the skill changes: no step, gate, or output of the build process moves.";
        :>> rationale = "Doc-sync rides every change in the same commit (CLAUDE.md section 4); leaving the layout paragraph at two crates would fork the truth between the manifest and the skill. The change is descriptive: it follows the tree D0479 chartered rather than choosing a new process. It is marked process-change because the file is locked and the guard cannot tell a layout sentence from a step - the marker is what lets the guard stay text-level and strict.";
        :>> consequences = "Later D0479 extractions that add members must touch the same paragraph, each under its own marked Decision or under a sprint chartered by an accepted marked Decision; that friction is the price of a locked skill surface and argues for the paragraph naming the pattern (members/ holds the D0479 members) rather than an enumerated list once the list grows.";
    }
    #DependsOn dependency from d0484 to d0479;
}
