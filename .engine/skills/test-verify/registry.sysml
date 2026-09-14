// The test-verify skill's own registry declaration, beside the skill it describes (D0222).
//
// MOVED verbatim out of the shared skills-registry by a committed migration transform (D0222).
// The shared registry
// could not travel with a unit, so adopting any unit left the receiving project's
// `process-skill` guard failing on arrival (issue252). Moved, never copied: the id is
// unchanged and `duplicate-identity` is what proves it.
package SkillsRegistryTestVerify {
    private import EngineElement::*;
    private import EngineSkills::*;

    part testVerify : AISkill {
        :>> id = "b4d9f201-6c0e-4f3b-9a7d-2e8c4f0b3d61";
        :>> title = "test-verify";
        :>> purpose = "The D0425 verifier's procedure (D0438): launch keel verify . --probe POSITIVE,NEGATIVE detached - the D0476 ladder: validate, guard, clippy, the dispatch's D0388 probe pair, then suite --touched, stopping at the first red - and read its receipt and the touched receipt only after exit (stopped_at, outcome, at, stems, lib); run check-engine and sync-claude --check beside it; write a receipt naming every command and every discrepancy. Reads and runs only - writes nothing to the tree, records nothing; the recorder writes from the receipt.";
        :>> location = ".engine/skills/test-verify/SKILL.md";
        :>> triggerCondition = "Dispatched as the D0425 verifier subagent; asked to verify or run the touched set; before any deliverable DoD or gate TestResult is recorded.";
        :>> writePolicy = WritePolicy::readOnly;
    }
}
