// D0532 (PROPOSED — NOT YET ACCEPTED) — The gate, guard, audit, hardening and activation verbs are guard source: they land under members/keel-guards/src/ and ride the enforcement-surface lock
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0532Accept event on sign-off.
package Decision0532 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    // #ProspectiveChange (D0070) - this Decision governs a process/schema change; the co-committed
    // change rides the process-change guard's hard lock.
    #ProspectiveChange part d0532 : Decision {
        :>> id = "52fbdd5b-c499-46e1-818f-c678fa3b0c78";
        :>> title = "The gate, guard, audit, hardening and activation verbs are guard source: they land under members/keel-guards/src/ and ride the enforcement-surface lock";
        :>> createdAt = "2026-09-19";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::proposed;
        :>> context = "Sprint 750 is the last D0479 extraction: keel-cli/src/main.rs (4,500 lines, 129 top-level items) becomes argument dispatch, and every verb body moves to the least member whose reach it needs, placed by scripts/extract_verbs.py from the table scripts/verb_homes.py computes. Five of those bodies - cmd_guard, cmd_audit, cmd_hardening, cmd_assured and cmd_activation, with their argument classifiers - reach only keel-guards and below, so the transform writes them to members/keel-guards/src/guards_verbs.rs and declares the module in members/keel-guards/src/lib.rs. D0504 made that directory a locked prefix: every file under it is enforcement surface, and the process-change guard refuses the lib.rs edit with no co-committed marked Decision. D0486, D0503 and D0504 each recorded the same meeting of the lock for an earlier extraction; D0504's consequences named the pattern - an extraction sprint that moves a locked source carries a marked Decision naming where the lock now lands.";
        :>> decision = "The verbs that run the guards, the gate audit, the hardening lens, the assured lens and activation are guard source. They live in members/keel-guards/src/guards_verbs.rs, inside the D0504 prefix, and an edit to how a guard is selected, how its arguments are classified, or how its verdict becomes an exit code is a locked edit needing a marked Decision, exactly as an edit to the guard body is. The transform does not carve an exception for them: placement is by reach, and the lock follows the directory. The other thirteen member verb files (args, fs, git, hooks, issues, model, parser, process, schema, serve, suite, view, write) are outside the lock, as their members' src directories are today.";
        :>> rationale = "A verb that turns a guard's report into pass or fail is the last link in the enforcement chain: an argument classifier that routes a guard name to the wrong family, or an exit-code mapping that returns 0 on a violation, weakens every guard at once while touching no guard body. D0209 clause 2 locked the enforcement surface for that reason, and D0504 chose a directory prefix over a file list so that a new guard file is locked by construction. Placing the guard verbs in that directory is the same construction working as intended, not a widening: these functions were in keel-cli/src/main.rs, which the lock never covered, so the lock is meeting code it should have covered since sprint 443. Moving them elsewhere to dodge the lock would put the guard runner's dispatch in a member that cannot see the guards.";
        :>> consequences = "Editing members/keel-guards/src/guards_verbs.rs needs a co-committed marked Decision or a sprint chartered by an accepted one (D0465). members/keel-guards/src/lib.rs gains one pub mod line and no other change. The lock test in keel-guards names guards_verbs.rs among the locked paths when its next edit lands. The verb file carries a module-level allow of the three pub-visibility lints (must_use_candidate, missing_errors_doc, missing_panics_doc) that the move makes reachable, the same allow every generated verb file carries. keel --help is byte-identical across the move, cli-surface-declared holds, and no dispatch line in main.rs changes, so the D0271 surface is untouched.";
    }
    #DependsOn dependency from d0532 to d0479;
    #DependsOn dependency from d0532 to d0504;
}
