// D0486 (PROPOSED — NOT YET ACCEPTED) — The build skill names the members/ layering, not each member crate
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0486Accept event on sign-off.
package Decision0486 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    // #ProspectiveChange (D0070) - this Decision governs a process/schema change; the co-committed
    // change rides the process-change guard's hard lock.
    #ProspectiveChange part d0486 : Decision {
        :>> id = "b54ec923-701d-4e01-80d5-0e1fc12b999d";
        :>> title = "The build skill names the members/ layering, not each member crate";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::proposed;
        :>> context = "D0484 put the six sprint-714 leaf members into the build skill's Workspace layout paragraph as an enumerated list, and its consequences clause said the list would have to change under a marked Decision at every later D0479 extraction, which argues for the paragraph naming the pattern once the list grows. Sprint 718 is that extraction: members/keel-fs (atomic file writes), members/keel-model (the read model: corpus, indexer, model, queries, orient and the rest of the D0479 read layer) and members/keel-write (the write API, scaffold, reverify, claim, intake) join the workspace, and an enumerated paragraph is now wrong. The skill file (.engine/skills/build/SKILL.md) is a locked process definition (D0070), so this doc-sync line needs a co-committed marked Decision (D0209 clause 2).";
        :>> decision = "The build skill's Workspace layout paragraph describes members/ by its D0479 layering - leaf crates, then the read model keel-model, then the write layer keel-write, each a lib that keel-cli re-exports under its old module path - and names the workspace members table in Cargo.toml as the one list of crates. It enumerates no member. Nothing else in the skill changes: no step, gate or output of the build process moves.";
        :>> rationale = "One home per fact (D0105): Cargo.toml already holds the member list and the build reads it, so a second list in prose can only drift, and D0484 recorded that it would. Naming the layering is what a builder needs from the skill - which layer a failing test belongs to and why a change in keel-cli does not rebuild keel-model - and that sentence stays true as members are added within the layers. It is marked process-change because the file is locked and the guard cannot tell a layout sentence from a step. MEASURED: no command path changes - the paragraph is prose; the build gates run once in sprint 718 and the word gate in this text names none of them.";
        :>> consequences = "Later D0479 extractions that add a member inside an existing layer touch neither this paragraph nor a Decision; only a new LAYER does. The skill loses the at-a-glance crate list, which lives one file away in Cargo.toml. D0484's enumerated paragraph is replaced whole, so this Decision supersedes D0484's decision clause on wording while keeping its rule that the skill describes the workspace as its members.";
    }
    #SupersedeClause dependency from d0486 to d0484;
    #DependsOn dependency from d0486 to d0479;
}
