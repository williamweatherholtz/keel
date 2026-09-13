// The delegated-ceremony skill's own registry declaration, beside the skill it describes (D0222):
// the unit travels with its binding, so an adopting project's process-skill guard is green on arrival.
package SkillsRegistryDelegatedCeremony {
    private import EngineElement::*;
    private import EngineSkills::*;

    part delegatedCeremony : AISkill {
        :>> id = "ec8bded7-752d-4834-8114-0dceef76b08d";
        :>> title = "delegated-ceremony";
        :>> purpose = "Deploys .engine/processes/delegated-ceremony.sysml: the D0425 verifier and recorder roles as a process. Carries the two dispatch briefs verbatim - the verifier's points at the test-verify skill, a probe pair and a receipt path and writes nothing to the tree; the recorder's reads that receipt ONLY, writes through keel record verbs ONLY, refuses a marker or edge the vocabulary or the API lacks, and ends its report with the verbatim last line of keel gate --fast . run after its last write, checked by references/check_report.py before it is returned.";
        :>> location = ".engine/skills/delegated-ceremony/SKILL.md";
        :>> triggerCondition = "A sprint's gate results, DoD results or sprint record are owed and the session can spawn subagents; any dispatch of a D0425 verifier or recorder.";
        :>> writePolicy = WritePolicy::direct;
    }
}
