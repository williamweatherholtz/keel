// D0544 (PROPOSED — NOT YET ACCEPTED) — singleWriterPinLiftedForTheRunKeptForTheGate: the launcher's in-place, one-run-per-tree clause is superseded whole - each run gets its own clone, and the serial pin is re-adopted for the gate that judges the merge candidate
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0544Accept event on sign-off.
package Decision0544 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    part d0544 : Decision {
        :>> id = "01be6647-3b9e-41ba-9990-a270ad361be8";
        // RESEARCH: three rounds of a nine-persona adversarial panel over four revisions of the direction artifact, every finding verified against the repository at f365cb7a; the two landscape spikes D0540 (94 rows) and D0541 (122 user threads); docs/reviews/runner-direction-panel-2026-09-22.md holds the reports and the finding-to-resolution maps.
        :>> title = "singleWriter: the launcher's in-place, one-run-per-tree clause is superseded whole - each run gets its own clone, and the serial pin is re-adopted for the gate that judges the merge candidate";
        :>> createdAt = "2026-09-22";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::proposed;
        :>> context = "D0182 (accepted) says in its decision clause: 'Committed mode is in-place execution: launch REFUSES on a dirty tree and snapshots the tree fingerprint at spawn; launcher concurrency pins to 1 with single-writer-per-tree during a run' - the honest answer while a run edited the tree in place, and the reason consecutive launches serialised behind the human's review of the last. Revision 4 of the panelled direction (D0542) materialises each run into a full clone with no remote, harvests only declared outputs to a run ref, and gates the merge candidate of trunk and that ref in one keel-owned checkout at a stable path, reset per gate. The clone makes the in-place clause and the run pin unnecessary; the gate still needs a single writer, because two green runs from one base can merge red and no result would notice - the suspect walk follows criterion text, not the tree around a result.";
        :>> decision = "OPTION A (recommended): supersede D0182's clause whole under D0398. Runs are N concurrent clones, bounded by the declared maximum in activation.toml; the gate is serial in one keel-owned checkout, its subject the merge candidate, its landing a fast-forward of trunk, a moving trunk re-queuing the run - the single-writer half re-adopted for the gate. OPTION B: keep the clause; one run at a time in place, no clones, and the parallel narrow agents st182 asked for wait for a daemon.";
        :>> rationale = "The human asked in st182 for agents that 'run in parallel and have more definite inputs and outputs'; per-actor write files already keep concurrent writes from colliding, so the run half of the pin bought nothing once the run left the tree. The gate half is where two writers still collide, and D0182's stated consequence - serialisation - moves there honestly. B keeps an accepted clause intact at the cost of the one property the human named first.";
        :>> consequences = "Under A: a #SupersedeClause edge to D0182; the slice-one CR names the stable gate checkout, its shared target directory as a sequential cross-run channel (a harvested build script runs there under the run's boundary), and gate parallelism 1; the measured floor states the gate's cost. Under B: the launcher stays as built and slice one is one run at a time. COST under A: a stable keel-owned gate checkout, its shared target directory stated as a sequential cross-run channel, and the gate serial while runs are parallel. COST under B: one run at a time, no clones, and the parallelism st182 asked for waits for a daemon.";
    }
    #SupersedeClause dependency from d0544 to d0182;
}
