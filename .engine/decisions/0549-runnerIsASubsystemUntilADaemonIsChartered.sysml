// D0549 (PROPOSED — NOT YET ACCEPTED) — runnerIsASubsystemUntilADaemonIsChartered: slice one lands inside keel as a schema change and one verb, and separation into its own project is decided only if the daemon slice is ever chartered
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0549Accept event on sign-off.
package Decision0549 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    part d0549 : Decision {
        :>> id = "ab567e4c-374b-4048-9556-ef1a6343cb97";
        // RESEARCH: three rounds of a nine-persona adversarial panel over four revisions of the direction artifact, every finding verified against the repository at f365cb7a; the two landscape spikes D0540 (94 rows) and D0541 (122 user threads); docs/reviews/runner-direction-panel-2026-09-22.md holds the reports and the finding-to-resolution maps.
        :>> title = "runnerHome: slice one lands inside keel as a schema change and one verb, and separation into its own project is decided only if the daemon slice is ever chartered";
        :>> createdAt = "2026-09-22";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::proposed;
        :>> context = "The human said in st185: 'Yes I meant a keel subsystem/mode, though I might make it a totally separate project because I feel like it's quite distinct architecturally.' Slice one of the panelled direction (D0542) is a schema CR on ProcessStep plus one CliCommand - a subsystem by construction. About a third of the issue ledger - 223 of 667 by the value critic's classification, the 80 unclassed an estimate - is keel the product: parser, adoption and view surfaces untouched by any runner protocol. Separating later is the project-migration process (D0275); separating now leaves that third behind with a smaller team. The question becomes live only if a daemon (slice two) is chartered on its recorded trigger.";
        :>> decision = "OPTION A (recommended): slice one lands in this repository; the subsystem-or-project question is asked again at slice two's trigger, and separation, if chosen then, is a governed project migration. OPTION B: a separate project now, adopting the engine as a downstream, with two backlogs from the first day.";
        :>> rationale = "Architectural distinctness is real - the human named it - but slice one has no daemon, lease server or proxy, and the two things it changes are in this repository's frozen core and CLI surface. Splitting a backlog before the first run exists spends the human's attention on a boundary nothing yet crosses. B is honest about the distinctness; A defers the cost to the moment it is paid.";
        :>> consequences = "Under A: no new repository; the slice-one CR is chartered here; the question is re-asked in the daemon's charter. Under B: a new project is introduced under the introduction process, the schema change lands upstream first, and the ledger's product third stays here. COST under A: nothing now; the question is re-asked in the daemon's charter. COST under B: a new repository introduced under the introduction process, two backlogs from the first day, and the schema change landing upstream first.";
    }
}
