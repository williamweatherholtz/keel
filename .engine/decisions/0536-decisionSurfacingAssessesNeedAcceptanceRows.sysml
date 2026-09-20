// D0536 (PROPOSED — NOT YET ACCEPTED) — The decision-surfacing process assesses needAcceptance rows beside decisionAcceptance rows - a Need awaiting the human's word goes on the same page, quoted, with what accepting it charters
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0536Accept event on sign-off.
package Decision0536 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    // #ProspectiveChange (D0070) - this Decision governs a process/schema change; the co-committed
    // change rides the process-change guard's hard lock.
    #ProspectiveChange part d0536 : Decision {
        :>> id = "18744ba3-cd2a-4731-926a-680723865145";
        :>> title = "The decision-surfacing process assesses needAcceptance rows beside decisionAcceptance rows - a Need awaiting the human's word goes on the same page, quoted, with what accepting it charters";
        :>> createdAt = "2026-09-19";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::proposed;
        :>> context = "D0534 made a Need no human-judged confirmation Test reaches a computed authority-queue row, and its fourth clause said the decision-surfacing brief lists those rows. The process that builds the brief (.engine/processes/decision-surfacing.sysml, deployed by skill decision-surfacing) reads only decisionAcceptance rows in its first step, so the row would compute and the page would never carry it - the seventeen-day wait of the three self-analysis Needs (issue661) would move from invisible to visible-in-JSON and still never reach the person whose word is owed. The skill file is under the process-change lock (D0070, D0465): the wording of what the assessment reads is a process step, and D0534 was accepted under standing consent without the marker, so it cannot carry this edit. This Decision is the marked charter for the one-paragraph change. The human's words behind it are st065: a pass is never AI testimony alone - and a Need the AI derived and never asked about is the same class of unasked word at the top of the tree.";
        :>> decision = "Step 1 of the decision-surfacing process reads both row kinds of keel show authority-queue: decisionAcceptance and needAcceptance. A needAcceptance row is presented like a Decision's ask - the Need's own text quoted, what accepting it charters (the SystemRequirements and Decisions that may then derive from it), the note that names the confirmation Test whose word is owed or says no acceptance test - and rows are grouped by the set one Test accepts, since the human accepts sets. The state test of step 2 counts a needAcceptance row entering or leaving as a change. Nothing else in the process changes: the page is still the channel, silence is still computed, and an item still leaves the page only by their answer or my stated withdrawal.";
        :>> rationale = "The row exists so the wait is seen by the person who can end it, and the process that carries asks to that person is this one; a second channel for Needs would fork the queue the process exists to keep single. A Need's acceptance is the stakeholder contract the whole Business workflow orders itself behind, so it belongs on the page with at least the standing a Decision has. Grouping by set matches how the human has accepted Needs every time so far (engineNeedsAccept, modularMembersNeedsAccept, the five keel-viewer gates): one word for one set, never a row per Need.";
        :>> consequences = "The skill's step 1 and step 2 carry the wording; the process file's actionText gains the same clause when the skill is next regenerated from it. The first brief built after this lands carries the thirty-four Needs the queue lists today, grouped into their sets, with the twenty-two nobody has ever asked about named as such. Delivered by sprint 752 (dcAuthorityQueueListsNeedsAwaitingAcceptance), which is chartered by D0534; this Decision holds the process clause D0534 could not.";
    }
    #DerivedFrom dependency from d0536 to st065;
    // the process clause D0534 (standing consent, unmarked) could not carry (D0303 layering)
    #DependsOn dependency from d0536 to d0534;
}
