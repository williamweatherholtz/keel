// D0524 (PROPOSED — NOT YET ACCEPTED) — An allocated name is one namespace across packages: guard duplicate-identity fails two declarations of the same issueNNN / stNNN / usNNN wherever they sit
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0524Accept event on sign-off.
package Decision0524 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    // #ProspectiveChange (D0070) - this Decision governs a process/schema change; the co-committed
    // change rides the process-change guard's hard lock.
    #ProspectiveChange part d0524 : Decision {
        :>> id = "8ff59b08-9e59-4286-b15d-4190aa283332";
        :>> title = "An allocated name is one namespace across packages: guard duplicate-identity fails two declarations of the same issueNNN / stNNN / usNNN wherever they sit";
        :>> createdAt = "2026-09-18";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::proposed;
        :>> context = "On 2026-09-18 two sessions on two machines each minted issue620 and issue621 from a tree at 4a0abc85 (claudeFable5 into issues-claudeFable5.sysml at 1809e70f, claudeOpus5 into issues-claudeOpus5.sysml at 1228a756). Git saw no conflict - different files - and the merge 5170199e passed validate and every enforced guard, duplicate-identity included (issue624). Read against the tree: `record issue` allocates the number over ALL of .tracking (members/keel-issues/src/issue_write.rs:69-73) and says the duplicate-identity guard backstops offline clones; but that guard's class 2 keys a declared name on its PACKAGE, the D0108 per-actor files are separate packages, and no class reads a bare name across them. Every lens does read the bare name: `keel show open-issues` at the merge showed one issue621 carrying both sessions' resolvers and no issue620, the open one computed resolved by the other's passing DoD. The frontier lost a stranger's open defect with nothing said.";
        :>> decision = "A name the write API mints by a whole-tree number scan - issueNNN (issue_write.rs), stNNN and usNNN (intake_write.rs) - is one namespace across the whole .tracking tree. Guard duplicate-identity gains class 5: such a name declared in two places, whatever package each sits in, is a violation naming both locations; the prefix-and-digits shape only, so a disposition `issue620Disp1` beside its Issue is not one. Class 2 (a declared name repeated within one package) stands as it is for every other name. The colliding pair in this tree is renumbered on the later-landing side (claudeFable5's, to issue622/issue623; D0108, the owner edits their own) with every reference in the owner's files following; the other side's records are not touched.";
        :>> rationale = "The guard is the D0047 control the allocator's comment already claimed, so the fix is to make the claim true rather than add a second mechanism: `keel land` gates the MERGED tree before the push (this collision would have been refused there, exactly where both sides were green alone), and CI runs the same guard. Making class 2 tree-wide for every name would be wrong: a sprint's `refineGate` or a per-actor file's `Disp1` legitimately repeats across packages, and edges resolve those in scope; only the allocated shapes are resolved by bare name by every lens. Renumbering rather than qualifying: the lenses, the DoD texts and the GitHub comments all speak the bare name, so two items sharing it are one item to every reader whatever the parser does. NOT A FORK: a central allocator (a remote counter) would trade an offline write path for a network dependency the write lock does not have today, and the merge-time gate catches the same case one step later with no new dependency.";
        :>> consequences = "Guard duplicate-identity scans five classes; its guards.md row and identity.rs doc comment say so. A tree carrying the collision at HEAD (5170199e) fails this guard and is the known-positive of the pair (`an_allocated_name_is_one_namespace_across_packages`); the renumbered tree passes. The allocator's comment is corrected to name class 5. dcAllocatedNameIsOneNamespaceAcrossPackages resolves issue624 in sprint746. The two claudeOpus5 records keep their numbers; anything downstream that cited claudeFable5's issue620/621 by number (the commit message 1809e70f, GH#86-90 comments not yet written) means issue622/623 from this commit on.";
    }
}
