// Delegated Ceremony - the D0425 verifier and recorder roles as a PROCESS, not a brief typed for the
// occasion. Sprint 647's recorder ran on a prompt written that day and (1) typed a #Addresses marker no
// schema declares, turning the Stop hook red twice and yielding obligationf1702eed, (2) hand-deleted three
// TestResult parts it had recorded without --evidence and re-recorded them, and (3) reported "ready for
// commit" over a tree whose guard was FAILED. Each of those is a role boundary this process names; the
// delegated-ceremony skill carries the two briefs the primary dispatches by pointing at.
// Deployed by .engine/skills/delegated-ceremony/SKILL.md; the verifier's own procedure is the test-verify
// skill (D0438) and is not restated here (D0105: one home per fact).
package ProcessDelegatedCeremony {
    private import EngineElement::*;
    private import EngineProcess::*;

    action delegatedCeremony : Process {
        :>> id = "fe85ee01-2f8b-46dc-b869-f6e4511e1186";
        :>> title = "Delegated Ceremony";
        :>> purpose = "Keep the primary agent on the substance (D0425) by handing each unit of management work to a subagent whose role fixes what it may read and what it may write. The VERIFIER reads the tree and runs the commands - keel gate validate, check-engine, guard --no-receipt, sync-claude --check, keel suite --touched detached, cargo clippy, the sprint's D0388 probe pair - and writes NOTHING to the tree: its whole output is one receipt file. The RECORDER reads the verifier's receipt file ONLY - never the primary's description of the work - and writes ONLY through the keel write API: record sprint --fill for a record and its edges, record gate-result / record result with --evidence quoting the receipt; never a file edit. A marker or edge the recorder needs that the vocabulary or the API lacks is a refusal it reports, not a line it types. Its report ends with the verbatim last line of keel gate --fast . run after its last write, so its summary cannot outrun the gate.";
        :>> cadence = Cadence::perSprint;
        // APPLIES-WHEN: a sprint's ceremony (gate results, DoD results, the sprint record) is to be written and a session can spawn subagents
    }

    action dcyDispatchVerifier : ProcessStep {
        :>> id = "b6b1b157-2ebe-4d55-8661-6b8776511048";
        :>> title = "Dispatch the verifier by pointing at its skill, the probe pair file and a receipt path";
        :>> triggerCondition = "The primary's substantive change is on the tree and a DoD or gate result is owed.";
        :>> actionText = "Write the sprint's D0388 pair as a file of two lines - the known-positive command then the known-negative, chosen before the tree is read - in the scratchpad. Dispatch a subagent of agent type verifier (haiku) with the verifier brief from the delegated-ceremony skill: it names the test-verify skill as the procedure, the binary copy to run, the pair file's absolute path (the verifier passes it as --probe-from and transcribes nothing, D0500) and the scratchpad path the receipt is written to. Nothing else - no restatement of the procedure (issue469) and no description of what the primary believes passed.";
        :>> producedArtifact = "A pair file of two lines, and a dispatch that names a skill, that file and a receipt path.";
        :>> owner = Owner::ai;
    }

    action dcyVerifierReceipt : ProcessStep {
        :>> id = "35eee02d-7bde-46eb-b4cc-4be840a2b810";
        :>> title = "The verifier runs the commands and writes one receipt; it writes nothing else";
        :>> triggerCondition = "The verifier is dispatched.";
        :>> actionText = "Follow the test-verify skill verbatim: every receipt line is a command run in this dispatch after the last write the tree saw, with the verdict line the command printed and its exit code. The set includes cargo clippy --release --all-targets -- -D warnings, because sprint 647's commit hook failed on a nursery lint the verifier's brief never ran. A write the verifier believes is owed is one line under VERIFIER-NOTED WRITES addressed to the recorder (issue471; the label names whose noticing it is, D0516). The verifier edits, creates or deletes no file under .tracking, .engine, keel-cli, .claude or CLAUDE.md and records no result.";
        :>> producedArtifact = "The VERIFIER RECEIPT file at the dispatched path.";
        :>> owner = Owner::ai;
    }

    action dcyDispatchRecorder : ProcessStep {
        :>> id = "eed3b964-3529-4944-a27a-e7ed02c95c04";
        :>> title = "Dispatch the recorder by pointing at its skill and the receipt path";
        :>> triggerCondition = "The verifier's receipt exists and its DISCREPANCIES line has been read by the primary.";
        :>> actionText = "Dispatch a subagent of agent type recorder (haiku) with the recorder brief from the delegated-ceremony skill, the receipt path, the sprint file, the gate or task names to record and their COUNT, which the recorder passes to the report check as --owed (D0492): the list is what the recorder reads, the count is what the check holds it to. The recorder reads the receipt file ONLY: the primary's account of the work is not an input, so a pass the primary believes in but the receipt does not show is never recorded. A receipt whose DISCREPANCIES line is not NONE is dispatched as what it is - the recorder records the reds the receipt names, never a pass over them.";
        :>> producedArtifact = "A dispatch that names a skill, a receipt and the records owed.";
        :>> owner = Owner::ai;
    }

    action dcyRecorderWrites : ProcessStep {
        :>> id = "f646316d-94ec-40b6-ac52-0d275fd50245";
        :>> title = "The recorder writes through the write API only, and refuses what the vocabulary lacks";
        :>> triggerCondition = "The recorder is dispatched.";
        :>> actionText = "Every write is a keel record verb: record sprint N slug --charter dNNNN --points P --fill F for a new record and its edges; record gate-result --evidence and record result --evidence with the receipt's own lines quoted. No Write or Edit tool touches a .sysml file - the recorder agent has neither. A result recorded without --evidence is not deleted and re-recorded by hand (sprint 647): it is a discrepancy the recorder reports. A marker, edge or field the recorder believes is needed and the write API does not offer is a line under REFUSED in its report - never a line it types into the tree, because the marker-vocabulary guard would turn the Stop hook red on the primary's next turn for a thing the primary never wrote. One write per owed record: a red gate --fast after a write is a line under DISCREPANCIES naming the guard, never a second record of the same gate or task with reworded evidence and never a tool run against the file - sprint 703's recorder did both (issue532, D0473).";
        :>> producedArtifact = "Records and edges on the tree, each carrying provenance and a receipt-quoting evidence string.";
        :>> owner = Owner::ai;
    }

    action dcyRecorderReport : ProcessStep {
        :>> id = "f7521055-75bc-462c-80d5-a6a0db905c23";
        :>> title = "The recorder's report ends with the gate's own last line, checked before it is returned";
        :>> triggerCondition = "The recorder's last write has landed.";
        :>> actionText = "Run keel gate --fast . after the last write and make its last line the verbatim last line of the report. Write the report to the dispatched path and run the skill's check (references/check_report.py) over it: the check refuses a report that types an undeclared marker, that does not end with a gate: line, that says DISCREPANCIES: NONE over a FAST GATE FAILED line, whose WROTE: line names a command that is not keel record with a sub-verb, or that writes one --gate or --task twice (D0473), that carries no WROTE: and no REFUSED: line, or that under --owed N accounts for fewer than N records as WROTE: or REFUSED: lines (D0492; sprint 723's first recorder wrote nothing and reported NONE), and a REFUSED line that cites the receipt's VERIFIER-NOTED WRITES line (or its old label OWED WRITES) as its reason - that line is the verifier's noticing for the recorder, never the recorder's owed count; the dispatch's count is the only owed count (D0516; sprint 735's second recorder read OWED WRITES: NONE as its own count). A refused report is corrected or its refusal is returned - the report is never returned around the check. A summary that outran the gate is exactly sprint 647's third failure.";
        :>> producedArtifact = "A report whose last line is the gate's verdict over the tree the recorder left.";
        :>> owner = Owner::ai;
    }

    action dcyRedTreeIsARefusal : ProcessStep {
        :>> id = "e8a993d3-dd75-49d4-b0bf-91f44888d9c7";
        :>> title = "A recorder that leaves a red tree is a counted refusal, not a red the primary inherits silently";
        :>> triggerCondition = "The subagent-stop hook fires for an agent of type recorder or verifier and the tree changed during its lifetime - measured from the fingerprint at that agent's own start (D0501), never from the session's.";
        :>> actionText = "For a recorder the subagent-stop hook runs the turn gate over the changed tree (D0174); when it blocks, the fire's ledger line is refused-class with control recorder:tree-red (keel-cli/src/main.rs subagent_block_control), so keel show control-census and enforcement-report count how often a recorder leaves a red tree. For a VERIFIER the hook never blocks (D0502): its receipt is its report and the tree's red is the primary's to fix at the turn gate; a verifier whose own fingerprint moved is one refused-class ledger line under verifier:tree-written (main.rs subagent_stop_route) - the one act the role forbids, counted beside recorder:tree-red. Either frequency decides whether this process's boundaries hold or the role needs a harder control. Any other agent type keeps the control the gate named.";
        :>> producedArtifact = "One ledger line per recorder red, countable by control name.";
        :>> owner = Owner::ai;
    }
}
