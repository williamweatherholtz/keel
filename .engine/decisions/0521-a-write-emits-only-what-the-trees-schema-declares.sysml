// D0521 (PROPOSED — NOT YET ACCEPTED) — A write emits only what the tree's own schema declares; a verdict member the tree's VerdictKind lacks is a refusal naming the vintage and keel migrate
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0521Accept event on sign-off.
package Decision0521 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    // #ProspectiveChange (D0070) - this Decision governs a process/schema change; the co-committed
    // change rides the process-change guard's hard lock.
    #ProspectiveChange part d0521 : Decision {
        :>> id = "54d5add6-b4d6-408f-b03b-2097bfb93434";
        :>> title = "A write emits only what the tree's own schema declares; a verdict member the tree's VerdictKind lacks is a refusal naming the vintage and keel migrate";
        :>> createdAt = "2026-09-18";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::proposed;
        :>> context = "GH#86 (st166; public repo, untrusted, D0264): on plotwright, binary `keel 0.4.1` build 0c131b7 with the project's `.engine` at b171cd7, `keel record gate-result` wrote `outcome = VerdictKind::proposed` and the next `keel gate validate` rejected the tree, blocking every sprint ceremony downstream. Read against this tree before triage: members/keel-write/src/write.rs:625 `proposed_tier` returns the binary's constant PROPOSED (:657) for an AI-examined pass (D0312 B) and neither it nor its two callers (:814 DoD result, :1270 gate result) read the tree's schema; .engine/schema/core/element.sysml:27 gained the `proposed` member at 24dbcfb3 on 2026-09-10, and tag v0.4.1 is b171cd7f of 2026-09-06, so every tree `keel init` scaffolded from the release lacks the member while every binary built from main since emits it. Nothing in write.rs validates what it wrote: `keel gate validate` takes 82 seconds on this tree, so a post-write validate is not a control the write path can afford, and the write was reported as success. The human's direction, verbatim in st167 and st168: migrate is blocked in many projects, migration should be smooth, make downstream projects whole.";
        :>> decision = "Before a write lands an enum member the binary knows, it reads the member's enum in the tree's own schema (.engine/schema/core/*.sysml) and refuses when the tree does not declare it, writing nothing and naming the member, the enum, the tree's engine vintage and `keel migrate` as the act that cures it. Applied today to the one member a write can emit that a released schema lacks: `VerdictKind::proposed` from `record gate-result` and `record result` (the DoD path). A tree with no `.engine/schema/core/element.sysml` is the write API's own fixture and is not read.";
        :>> rationale = "A write that produces a file the tree's validator rejects has not recorded a fact; it has left the adopter a red tree to hand-edit, and `record` reported success while doing it. Refusing is the honest fail-closed: nothing is written, the tree stays valid, and the message names the one act that makes the write possible. The rejected alternative - writing `pass` when the tree lacks `proposed` - would grade an AI-examined claim as a standing pass on an older schema, which is the fabricated-tier D0312 B exists to prevent; writing `inconclusive` would record a verdict nobody reached. The read is of the enum's declaration line in the tree's schema file, the same text `validate` registers (keel_model::validate::validate_root registers every `.engine` package before it validates), so the write and the gate agree by construction rather than by a second vocabulary constant. Cost is one file read per proposal-tier write.";
        :>> consequences = "Backlog task dcAWriteRefusesTheMemberTheTreesSchemaLacks delivers this sprint (745) and resolves issue614; a new WriteError variant carries the refusal and the CLI prints it verbatim. On a plotwright-vintage tree the ceremony is refused with the instruction until `keel migrate` lands the schema; with D0520 and D0522 landed the migrate that was blocked succeeds, so the refusal is a door, not a wall. Stated residual: this reads one enum for one member; a later member added to any other enum the write emits needs the same read, and the test that holds the refusal is the place to add it. GH#86 is answered on GitHub with the tracked id once this record's status is known. HELD proposed under D0337 for the human's word and applied under the marker in the same commit, as D0519 was; their word keeps or reverts it.";
    }
    #DerivedFrom dependency from d0521 to st167;
    #DerivedFrom dependency from d0521 to st168;
}
