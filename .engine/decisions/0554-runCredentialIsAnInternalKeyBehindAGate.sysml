// D0554 (status: the `status` field below is the truth) — runCredentialIsAnInternalKeyBehindAGate: the agent in a run holds a run-scoped internal key issued by a keel-owned gate, the gate holds the one organisation API key and meters per run, and the key goes invalid at reap
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0554Accept event on sign-off.
package Decision0554 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    part d0554 : Decision {
        :>> id = "a7c11203-5c5a-4b4f-8a5d-9f92efce40aa";
        :>> title = "runCredentialIsAnInternalKeyBehindAGate: the agent in a run holds a run-scoped internal key issued by a keel-owned gate, the gate holds the one organisation API key and meters per run, and the key goes invalid at reap";
        :>> createdAt = "2026-09-23";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::accepted;
        :>> context = "The human accepted D0546 as OPTION B on 2026-09-23 with these words (st198 here): 'add a custom gate interface that requires a separate key to access - we won't have multiple e.g. anthropic API keys to leverage. But a custom internal gate is proper, yes?' OPTION B as written named an organisation API key from a Console-minted pool, one per run, set inactive at reap. The human has one organisation key to leverage, not a pool. The panel's revision 4 section 3.3 already places an egress proxy holding the provider credential for constrained steps, checking the run credential's validity on every forwarded call, metering usage against a pricing table; the runner-daemon landscape found the proxy holding the one credential to be field consensus (E2B, Docker sandboxes, Cloudflare, Anthropic's git-credential proxy) and the run credential as the run id whose validity is computed from the lease to be the one invention. keel-next's component list names it compNxProxy.";
        :>> decision = "The credential an agent holds during a run is a run-scoped internal key issued by a gate keel owns; the gate holds the single organisation API key, forwards provider calls only for a valid run key, meters each run's usage, and refuses a key whose run has been reaped. Inactivation at run end is the internal key going invalid, computed from the run's lease record, so no provider key is rotated. Per-token billing for runs stands, as B said. The bridge auth clause of D0094 is superseded as B said, in the narrower form that the gate, never the agent, sets the provider key. NOT A FORK: the human chose B and stated the gate in their own words; this records the shape their words give it. NOT A PROCESS CHANGE: no workflow, gate predicate or schema of this repository changes; the gate is a component of the runner design.";
        :>> rationale = "A pool of provider keys needs an admin credential to mint and inactivate them and one key per run; a single key behind a gate needs one credential the gate guards and one internal key per run the gate can invalidate without any provider call. The human has one key. The gate's host joins the trust boundary; that cost is named and is the same host the runner's server already needs.";
        :>> consequences = "compNxProxy in keel-next carries this design; the agent's environment holds the internal key and the gate's address, never a provider key; a constrained step cannot run before the gate exists; the gate's host is one the human controls. keel-next HANDOFF question 1 is answered.";
    }
    #DerivedFrom dependency from d0554 to st198;

    // acceptance event (D0121 review-queue sign-off; D0066/D0106 — human-judged, not fabricated)
    verification d0554Accept : Test { :>> id = "fbef462b-68a2-4727-94b8-ba394d9ba888"; :>> method = VerificationMethod::confirmation; :>> procedureText = "AUTO-ACCEPTED under standing consent (D0207). Their standing words, verbatim: 'issues raised are automatically accepted. they can be customizedly changed post-fact.' Not individually reviewed; override by a superseding Decision (D0290) or the human's quoted word in chat (D0289)."; }
    part d0554AcceptR1 : TestResult { :>> id = "e6e9edb3-8903-48ef-aa46-21a446d9035c"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "996db40d"; :>> judgedAt = "2026-09-23"; :>> judgedBy = "wweatherholtz"; :>> createdBy = "claudeFable5"; }
}
