// D0481 (status: the `status` field below is the truth) — touchedSetDescendsTheWorkspace: the touched set is computed per workspace member and its dependents, and a self-reading test keys on the read set observed when it last ran, not on one hash of the whole tree
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0481Accept event on sign-off.
package Decision0481 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    part d0481 : Decision {
        :>> id = "e5a9bcfa-c50b-406b-9c55-769691d7af54";
        :>> title = "touchedSetDescendsTheWorkspace: the touched set is computed per workspace member and its dependents, and a self-reading test keys on the read set observed when it last ran, not on one hash of the whole tree";
        :>> createdAt = "2026-09-14";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::accepted;
        :>> context = "D0474 keyed the touched receipt on two root hashes: code (every source path) and tree (everything outside .keel plus HEAD) for the eighteen tests that read this repository. Any edit anywhere moves one of the two, so a passing binary is re-run for changes it cannot see. The human's words (st126): 'i like the 'test binaries' improvements', and earlier the descent they want: 'you go one level deeper, now maybe one branch of 7's hash is different. you dig down until you cover all changed hashes.'";
        :>> decision = "The touched set is computed over the workspace graph: the members whose files changed plus every member that depends on them, read from the Cargo manifests, and only their test binaries are candidates. Within the candidates, a binary whose tests read this repository keys on the set of paths it was observed to read the last time it ran - recorded by the test support layer that opens files for those tests - hashed together with its member's code; the one tree hash of D0474 is retired for those binaries. A binary with no recorded read set falls back to the tree hash, so the scheme is honest before it is complete. This reverses one clause of D0474 (the single tree key) and leaves its code key and receipt shape in force. NOT A FORK: the human chose the descent in st126. NOT A PROCESS CHANGE: the land still runs the touched set before the push (D0421); what changes is how the set is computed.";
        :>> rationale = "A hash tree is only a saving if it is descended: the root tells you something changed, the children tell you what. Cargo's graph is the first level for free once members exist; the observed read set is the second level for the tests whose inputs are data, not code. Recording what a test actually read is the one honest key: a declared read set drifts, an observed one is a receipt.";
        :>> consequences = "Sprint dcTouchedSetDescendsTheWorkspace, after the leaf members exist. The receipt gains a per-binary read-set field; a test reading a path outside its recorded set on the next run simply widens the set. D0474's test for the two root keys is narrowed to the fallback case.";
    }
    #SupersedeClause dependency from d0481 to d0474;
    #DerivedFrom dependency from d0481 to st126;

    // acceptance event (D0121 review-queue sign-off; D0066/D0106 — human-judged, not fabricated)
    verification d0481Accept : Test { :>> id = "3b5a2cfb-650a-466b-a15b-8dff24616489"; :>> method = VerificationMethod::confirmation; :>> procedureText = "AUTO-ACCEPTED under standing consent (D0207). Their standing words, verbatim: 'issues raised are automatically accepted. they can be customizedly changed post-fact.' Not individually reviewed; override by a superseding Decision (D0290) or the human's quoted word in chat (D0289)."; }
    part d0481AcceptR1 : TestResult { :>> id = "3f38a465-ab02-4eb8-bc74-4f3fed222b63"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "13eb23b"; :>> judgedAt = "2026-09-14"; :>> judgedBy = "wweatherholtz"; :>> createdBy = "claudeFable5"; }
}
