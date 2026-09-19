// The sprint-review skill's own registry declaration, beside the skill it describes (D0222).
//
// MOVED verbatim out of the shared skills-registry by a committed migration transform (D0222).
// The shared registry
// could not travel with a unit, so adopting any unit left the receiving project's
// `process-skill` guard failing on arrival (issue252). Moved, never copied: the id is
// unchanged and `duplicate-identity` is what proves it.
package SkillsRegistrySprintReview {
    private import EngineElement::*;
    private import EngineSkills::*;

    part sprintReview : AISkill {
        :>> id = "f597ch3c-0e45-4b7f-ab80-0e37e4h3g000";
        :>> title = "sprint-review";
        :>> purpose = "Deploys .engine/processes/review-critique.sysml (D0172). Per-sitting review (D0049, finished by analysis under D0510): summarize the sitting's sprints + metrics (velocity/efficiency/accuracy), run the transcript scan that feeds the autonomous retro, and record the presentation as a method = analysis Test the AI judges, with #Covers edges to the sprints presented and the outstanding per-item asks listed by verb (keel accept, keel judge-set, a disposition) — never a confirmation of the sitting.";
        :>> location = ".engine/skills/sprint-review/SKILL.md";
        :>> triggerCondition = "Sprint review gate; asked for velocity, efficiency, estimate accuracy, or process issues.";
        :>> writePolicy = WritePolicy::direct;
    }
}
