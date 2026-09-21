# Agent-harness landscape, 2026-09-20 — 94 rows, five lenses, and what the tool-owns-the-task design takes

**Process:** adversarial-panel-review (D0187), one round, five parallel read-only web-enabled lenses with one shared brief (the brief is reproduced in Appendix 0). **Trigger:** the human's words of 2026-09-20 (st184): *"Research alternative harnesses like this. We're talking bazel and GitHub actions, but surely there are sandbox container setups for AI agents. Any strongly considering custom processes and verification workflows? Other strengths we should integrate? Investigate 50 alternative tools, noting what has gained some following recently"*. **Design under test:** the in-progress tool-owns-the-task design from st181–st183 (typed steps with declared inputs/outputs; constrained vs unconstrained step classes; gates = tests bound to steps; server never executes; runner daemons claim leased steps; browser UI). The design is not approved; nothing here is a backlog item yet.

**Check date:** every adoption number, star count, funding round and release date in this document was read from the web on 2026-09-20 (lens C read a few on 2026-09-21 UTC and says so) and carries its URL in the lens report that found it. **Epistemic status:** the external facts are lens-reported and were not independently re-fetched by the synthesiser; where a lens marked something "not found" or "unverified" that marking is kept. The only claims about keel's OWN state that appear below were re-checked here before use, and each names its check.

**Coverage:** 94 table rows across the five lenses (A sandboxes 21; B task-owning harnesses 19; C workflow/durable-execution engines 23; D hermetic build and CI runners 16; E verification, spec and attestation 15). Two tools appear in two lenses (OpenAI Codex cloud, A/B; Argo Workflows, C/D) and several rows bundle a family (lens E's review-agent row names five products, its eval row four, its benchmark row four), so the distinct-tool count is above 100. The 2026-09-02 corral spike (`docs/reviews/agent-corral-landscape-2026-09-02.md`) already covered fifteen agent-discipline and authorization projects; those were excluded here except Beads, which lens B re-examined because Gas Town changed it.

---

## 1. The answer, in the order the human asked

### "surely there are sandbox container setups for AI agents"

Yes — twenty-one, and the shape has converged (lens A). A microVM or gVisor container per session (Firecracker underneath E2B, Fly Sprites, Northflank; gVisor under Modal and Codex cloud; libkrun under microsandbox), snapshot/restore for warm starts, and — the one mechanism every serious vendor now ships — an **egress proxy that holds the secret**: the sandbox sees a placeholder, the proxy swaps in the real credential on the way out (E2B `Secret.fill`, Docker Sandboxes `sbx-cs-<rand>` placeholder swap, Cloudflare outbound Worker, Runloop Agent Gateway, NVIDIA OpenShell "providers", Anthropic's git-credential proxy for Claude Code on the web). Isolation itself is a commodity: Daytona (71.7k stars) archived its repo on 2026-06-11 and went closed-source citing AI-assisted vulnerability hunting; Earthly's OSS ended 2025-04-16; Blaxel was acquired by Baseten 2026-09-10. Modal's sandboxes are more than a third of a $4.65B company's revenue; E2B reports over a billion sandbox launches.

What none of the twenty-one does: **materialise only declared inputs, or harvest only declared outputs.** The closest are Docker Sandboxes' workspace-dir-only mount, Codex cloud's read-only `.git`, microsandbox's explicit volume mounts and OpenShell's Landlock path allowlist — all of them restrict what the agent may *touch*, none of them defines what the run *produced*. That is the design's novel claim and, per lens A, its build risk: no SDK has `materialise → run → harvest`, so it is keel's own code.

### "Any strongly considering custom processes and verification workflows?"

Nobody does both, in 94 rows. The field splits cleanly:

| Camp | Who | What "done" is | What they lack |
|---|---|---|---|
| **Process definers** | Spec Kit (138.1k stars), OpenSpec (69.7k), BMAD (53.3k), Kiro, Symphony `WORKFLOW.md`, Camunda BPMN, Kestra YAML, Argo templates, Conductor JSON, Temporal/Restate/DBOS code | a checkbox the agent flips, a function that returned, a pod that exited 0 | a test bound to the step |
| **Verifiers** | Harbor / Terminal-Bench, SWE-bench, Runloop Scenario, Inspect (UK AISI), `claude plugin eval`, Dagster asset checks, in-toto / SLSA, Lean-kernel proof agents | a hidden test, a kernel, a scorer, a signed layout | a persistent work item; the process ends when the eval does |

Three tools put *something* automated between the agent's claim and the status flip, and each is a partial precedent for a gate: **Gas Town's Refinery** merge queue runs verification gates on the merged stack before `gt done` counts (lens B); **GitHub Copilot coding agent** structurally cannot approve its own PR and its CI will not run until a human clicks (B); **Dagger v0.21** auto-generates a `check` for every `generate` (D). **Dagster asset checks** are the nearest thing to a check declared as an object attached to the artifact and rendered red (C). None of the four owns the work item end to end.

The empty quadrant — a process definer whose steps are done when a bound test passes, on a persistent tracked item — is where the design sits. Lens E's phrasing: every tool that made verification deterministic gave up the persistent work item; every tool that kept the work item let the agent say when it was done.

### "Other strengths we should integrate?"

Ranked by how directly the mechanism lands on an open question in the design. Each names the tool that has it running and the lens paragraph with the URL.

| # | Mechanism | Running in | Lands on |
|---|---|---|---|
| 1 | **Run token is the only credential; a proxy maps it to scoped rights.** Sandbox sees a placeholder, proxy swaps the secret after the request leaves. | E2B, Docker Sandboxes, Cloudflare, Runloop, Anthropic proxy (A); GitHub JIT runner token + per-job `GITHUB_TOKEN`, GitLab job token (D) | the "no credentials in the sandbox" clause — write it as *no credentials, one run token, egress only through the proxy* |
| 2 | **Reserve → acquire-only-this-job token → heartbeat-or-lose-it.** | Buildkite job-acquisition-token (2026-08-31, D); Temporal task token + heartbeat, Conductor `responseTimeoutSeconds`, Camunda job activation (C); Symphony Unclaimed→Claimed→Running→Released FSM, Linear 10-s liveness, Paperclip budget-debiting checkout (B); K8s SIG agent-sandbox `SandboxClaim` (A) | the runner lease protocol, wholesale |
| 3 | **Declared outputs are the only thing that leaves the sandbox.** Move known outputs out, delete the sandbox. | Bazel, REAPI `Command.output_paths → ActionResult`, Pants `output_files`, Argo `inputs.artifacts`/`outputs.artifacts` (D, C) | the harvest step; Argo's artifact schema is the vocabulary to borrow |
| 4 | **Step N's products are step N+1's only admissible materials, by hash.** | in-toto layout `MATCH` rule (E) | the drift guard — a write with no run behind it is a MATCH failure; already specified and signable |
| 5 | **A task is a directory: instruction, hidden test, oracle solution, environment.** | Harbor `task.toml` / `instruction.md` / `tests/test.sh` / `solution/solve.sh` / `environment/Dockerfile` (E) | the authored shape of a constrained step; generalises the D0388 probe pair to every step (positive = oracle passes, negative = empty run fails) |
| 6 | **Online setup phase, then offline agent phase with secrets stripped.** | Codex cloud (A, B) | the constrained step's lifecycle, proven at 8M weekly users |
| 7 | **The sandbox image is a declared input, digested into the step.** | Nix fixed-output derivations; the 2026-07-30 Nix critique (D) | closes the hidden-input hole lens D found in the design |
| 8 | **In-flight runs pin to the definition version they started on; migration is an explicit boundary.** | Restate deployment pinning, Vercel WDK skew protection, DBOS version pinning, Camunda instance migration (C) | the frozen meta-process — a process edit must not silently re-type running steps |
| 9 | **The observer writes the receipt, never the runner.** | Tekton Chains snapshots the run; GitHub attestations from OIDC; SLSA VSA shape (D, E) | the gate result — the server records what it harvested and tested, the agent records nothing |
| 10 | **Approval policies over tool calls: approve / reject / escalate-to-human, by pattern.** | Inspect (UK AISI) `human_reviewer`; OpenAI Agents SDK `needs_approval(args)`; Pydantic AI validate-then-approve (E, C) | the unconstrained step's routing rule — validation runs *before* a human ever sees a proposal |
| 11 | **The tool set an agent may call *is* the process definition.** | Camunda ad-hoc sub-process: the LLM picks within a declared activity set (C) | the answer to "closed constrained step is too rigid" — a constrained step may declare a tool *set*, not a fixed sequence |
| 12 | **Handoff state ≠ Done.** A run may legitimately end at Human Review. | Symphony `active_states` (B) | the state machine; keel's `proposed` (D0312) is already this |
| 13 | **Rebuild the queue from workspace state on restart; no scheduler DB.** | Symphony (B) | "state is computed" applied to the runner queue |
| 14 | **Snapshot-then-branch:** golden per-process snapshot with dependencies, one branch per step, discard after harvest. | Fly Sprites, E2B, Modal checkpoint/restore (A) | the cost objection to per-step sandboxes |
| 15 | **A typed schema on the human's answer.** | Prefect `pause_flow_run(wait_for_input=Model)`, Windmill approval form fields, Airflow `assigned_users` + `awaiting_input` (C) | the human port of a step — keep the verbatim words (D0192), add the typed field beside them |
| 16 | **Refuse to run rather than run unconfined; mock `expect:` blocks abort at score 0.** | `claude plugin eval` (Claude Code v2.1.269, 2026-09-11) (E) | the runner's posture when the sandbox cannot be established |

Two things the lenses found that keel already has and nobody else does, which should therefore be *kept*, not traded: the human's words recorded verbatim as the acceptance (`keel accept --words`, D0192/D0289 — lenses B, C and E each looked and found no precedent), and done-as-a-passing-test on a persistent item (above).

### "noting what has gained some following recently"

| Tool | Signal (date read 2026-09-20; URL in lens) | Lens |
|---|---|---|
| Paperclip | 81.1k stars since launch 2026-03-04 — strongest growth in any lens | B |
| Multica | 50.9k stars from repo creation 2026-01-13 (5.5k Apr → 48.4k Sep) | B |
| OpenAI Symphony | 27.3k stars since 2026-04-27; Linear-as-dispatcher spec | B |
| Codex cloud | <1M → 8M weekly Codex users Feb–Jul 2026 | B |
| Temporal | $300M D @ $5B (2026-02-17) then $550M E @ $12.55B (2026-09-15); >$250M ARR | C |
| Devin / Cognition | $26B valuation 2026-05-27; ~$492M ARR May 2026 | B |
| Factory | $150M C (2026-04-16) → $5B by Sep 2026 | B |
| Cursor cloud agents | >35% of Cursor's own merged PRs agent-authored (Apr 2026); SpaceX $60B acquisition Jun 2026 | B |
| Modal | $355M C @ $4.65B; sandboxes >⅓ of revenue | A |
| microsandbox | 8.3k stars, fastest-growing OSS sandbox | A |
| NVIDIA OpenShell | 8.7k stars; YAML policy for fs/net/process/providers | A |
| K8s SIG agent-sandbox | 4.0k stars, v1beta1 `SandboxClaim`/`WarmPool` | A |
| Dagger container-use | 4,046 stars since 2025-05-23 | D |
| Blacksmith / Namespace / Depot | $45M B (2026-08-12, 6,000 orgs) / $23M (2026-03-23) / $10M (2026-03) — CI-runner vendors | D |
| Kestra | $25M A (Mar 2026); 1.0 and 2.0 within twelve months; 28.2k stars | C |
| Trigger.dev | $16M A (2025-12-17); 30k devs, 100Ms runs/month | C |
| Orkes Conductor | $60M (2026-04-23); 32.2k stars | C |
| Spec Kit / OpenSpec / BMAD | 138.1k / 69.7k / 53.3k stars — process-as-prose | E |
| CodeRabbit | $143M @ $1.5B (Aug 2026) — LLM review | E |
| Harmonic / Axiom | $295M / $200M @ $1.6B — Lean-kernel proof agents | E |
| Prefect ← Dagster | acquisition 2026-07-13 (consolidation) | C |
| OpenAI ← Ona (ex-Gitpod) | acquisition 2026-06-11 | B |

Declining or gone, and why it matters: **Vibe Kanban** (28.1k stars) — company shut 2026-04-10, "couldn't find a business model" for a free board with no gate and no server truth (B); **Daytona** archived 2026-06-11 (A); **Earthly** OSS ended 2025-04-16 (D). The lesson the lenses drew from all three: the sandbox and the board are commodities; the ownership-and-gate layer is where value can sit, and nobody has occupied it.

---

## 2. What converges across lenses

Findings that two or more lenses reached independently, without being pointed at each other.

1. **One credential per run, held by a proxy, never inside the sandbox** — A (six vendors), B (Anthropic, Cursor secret injection), D (GitHub JIT, GitLab, Buildkite JAT). The design's "no credentials" clause is the industry default; the design's *addition* is that the run token is also the write credential for the harvest.
2. **Declared-output harvesting exists only in build systems** — D has it in five; A, B, E have it in zero; C has it in one (Argo). Every agent-facing product mounts the whole repo and takes a diff or a PR.
3. **DONE is computed outside the agent only where the process ends with the eval** — E (Harbor, Inspect, plugin eval, SWE-bench), C (Dagster checks), B (Refinery, Copilot non-self-approval). Every process-defining tool flips a status on the agent's word; Anthropic's own Routines docs say a green run "does not mean the task succeeded" and ship anyway (B).
4. **Drift detection does not exist** — all five. The nearest cousins detect *code* drift against a journal (Temporal non-determinism error, Restate fail-loud mismatch, C) or specify it without an agent product using it (in-toto MATCH, E).
5. **The tracker is winning ownership of the task, and the lease protocol is mature elsewhere** — B (Linear as control plane for Codex, Devin, Cursor, Copilot, Factory, Symphony), C (Temporal/Conductor/Camunda/Hatchet worker-polls-with-timeout), D (Buildkite JAT), A (SandboxClaim). Nobody exposes lease + expiry + run token together; the pieces exist separately.
6. **Pin the run to the definition version** — C (four engines), D (sandbox image as input). A frozen meta-process without pinning is a known failure mode with a known fix.
7. **Nobody nests processes, and nobody records the human's words** — A, B, C, E. Nesting's nearest prior art is Argo `templateRef`/DAG and Temporal child workflows (C, D), both code-level.

---

## 3. The adversarial case against the design, and its disposition

The brief asked every lens to argue against the design. Consolidated, with what the synthesis does with each. "Accept" changes the design draft; "hold" is the human's call; "reject" names why.

| Critique | From | Disposition |
|---|---|---|
| "Network off" is a minority default and unimplementable for an LLM agent, which must reach a model endpoint. | A | **Accept.** Rewrite the clause as *egress only to declared providers, through the proxy, with the run token* (OpenShell "providers", srt proxy-only, Codex two-phase). |
| Agents need to grep the whole repo; materialising only declared inputs fights that. Codex solved exfiltration with network-off, cheaper and less brittle. | B, A | **Accept in part.** The repo at a declared ref *is* a declared input; the constraint the design cares about is on outputs and credentials, not on reading. Keep "declared inputs" for the extra materials (Argo artifacts) and stop implying the agent cannot see the tree. |
| The sandbox image is a hidden input. | D | **Accept.** Digest the sandbox definition into the step (mechanism 7). |
| The market pays for agent-drives-tool, whole-repo-in, PR-out, LLM reviewer, persistence; the one strict board with server truth (Vibe Kanban) died; the winners (Paperclip, Multica) are permissive and adapter-agnostic. | B, C, D, E | **Reject as a design argument, accept as a build constraint.** keel is not selling a board; its bet is computed truth (D0374 said the same of trackers). But the runner must be adapter-agnostic — Multica detects 26 agent CLIs, Paperclip has adapters for three — or nothing will run in it. |
| Every runner-owning system has a CLI besides its UI; "browser UI, no CLI" would make keel the only one an agent cannot script. | B | **Hold — this is the human's direction (st182).** The evidence says: the *human's* surface can be browser-only; the *runner's* surface is a protocol (Linear's typed activity stream, Symphony's polling contract), and a runner daemon is by construction a command-line program. The CLI the human wants gone is the human-facing one; the lens finding does not contradict that. |
| Typed declared-I/O steps have been available for years (Argo, Conductor, Kestra) and lost mindshare to "just write async code" (Temporal, Inngest, Vercel). Declaration is expensive to author; authoring friction is keel's #1 risk (D0054 — CLAUDE.md §4, checked). | C, D | **Accept as the central risk.** The meta-process must *generate* step declarations (Harbor's task directory as the template; agents author steps; Dagger's auto-`check` per `generate` as the pattern), or the design dies of the friction the human already named. |
| A closed constrained step is too rigid for agent work whose next step is unknown until the previous one returns. | C | **Accept via mechanism 11.** A constrained step declares a tool *set* (Camunda ad-hoc sub-process); the unconstrained class exists for the rest. |
| Determinism contracts are a footgun: Temporal needed patch markers, worker versioning and a lint pack. If gates replay an agent, expect the same cost. | C | **Reject — not applicable, and worth saying why.** keel's gate is a test over *harvested outputs*, not a replay of the agent. Nothing in the design re-executes an LLM call. |
| Required-reviewer gates record a click; verbatim-words acceptance has no precedent and may be friction nobody wants. | D | **Hold as stated in the corral spike.** It has no precedent (three lenses looked); it is also the one attestation shape that survives a status field being edited. Keep it; add the typed answer beside it (mechanism 15). |
| Nesting has no prior art. | A, C | **Accept as a risk.** Prototype nesting last; Argo `templateRef` is the nearest vocabulary. |
| Orchestrators are expensive: Gas Town's 141 orphaned processes in a week, "token burner". | B | **Accept as a control.** Lease expiry + budget-debiting checkout (Paperclip) + a runner that refuses to start when the sandbox cannot be established (plugin eval's posture) are the three mechanisms that bound it. |

---

## 4. The open brainstorm question, read against the evidence

The question left open before this spike: *where does the agent process run — (i) the server spawns agents, or (ii) runner daemons claim leased steps?*

Everything in this landscape that is not a vendor who owns the compute chose (ii): Buildkite's reserve/acquire ladder, Temporal and Conductor workers polling a typed queue with a timeout, Symphony's orchestrator-side claim over Linear, Multica's daemon-pulls-from-board with code staying on the user's machine, K8s `SandboxClaim`, Paperclip's heartbeat checkout. The (i)-shaped systems — Codex cloud, Jules, Cursor cloud agents, Devin, Claude Code on the web — are all hosted products whose vendor runs the VM. A self-hosted server that never executes (st182) is (ii) by definition; the evidence adds that the protocol is already designed (mechanism 2) and that a bundled local runner is what Multica and Symphony ship.

---

## 5. What this spike does not do

No tool is adopted, trialled or integrated. No backlog item is created — the design these strengths feed is not approved, and an item against an unapproved design would be state authored ahead of the decision. The sixteen mechanisms in §1 are inputs to the next design section, where each either enters the draft or is declined with a reason. The record of this spike is D0540 (charter), sprint 758, the knowledge record `.tracking/knowledge/agent-harness-landscape-2026-09-20.sysml`, and this document.

---

# Appendix 0 — the shared lens brief (verbatim)

```text
# Landscape research brief (shared by all lenses)

Today is 2026-09-20. Use WebSearch and WebFetch for EVERYTHING - your training data is stale for this field.
Every adoption number carries the date you read it and the URL. Do not guess a star count or a funding round;
if you cannot find it, write "not found". Do not write anything under the repository; write your result ONLY
to the scratchpad file named in your dispatch, and also return it as your final message.

## What we are designing (so you know what "relevant" means)

A work-tracking server ("keel") that OWNS AI tasks the way Linear/Jira own human tasks - the tool assigns
work to agents, not the reverse. Its truth is plain text in git; state is computed, never stored. We are
designing:

1. Processes defined as chains of typed STEPS with declared inputs and outputs; processes nest; a frozen
   meta-process defines processes. Per step: what goes in/out of the server, in/out of the agent, in/out of
   a human, in/out of the outside world.
2. Two step classes: CONSTRAINED - an agent runs in a materialised sandbox holding only the step's declared
   inputs (no credentials, no git remote, network off), tools = the step's declared outputs made callable,
   and the server HARVESTS only declared outputs after exit; UNCONSTRAINED - a full-tool agent in the real
   repo whose outputs are proposals that must pass a constrained/deterministic step before landing.
3. Gates = tests bound to steps; a step is done when its gate is green, never when the agent says so.
   Human sign-off quotes the human's words verbatim. Drift guards detect writes with no run behind them.
4. Server never executes anything itself; runner daemons claim ready steps (lease), run, push harvested
   outputs with the step's run token. Browser UI, no CLI.

## For EACH tool, answer these seven things (terse; "n/a" is allowed, "unknown" is honest)

1. What it is (one line) and who makes it.
2. Execution / isolation model: container, microVM, Wasm, worktree, none; are inputs materialised
   (only declared files present) or is it the whole repo; network and credential posture.
3. How steps/processes are defined: code, YAML, DSL, UI; typed inputs/outputs?; nesting/sub-workflows?;
   hooks (before/after step)?; versioning of definitions?
4. Verification / gating: what makes a step DONE - a test, an approval, the agent's say-so; human-in-the-loop
   primitive; attestations/receipts; can it detect drift (a change with no run behind it)?
5. Task ownership: does the TOOL assign/claim/queue the task for the agent, or does the agent drive the tool?
   Lease/claim protocol? Parallelism model (per task sandbox? shared repo?).
6. Adoption signal WITH NUMBERS AND DATES: GitHub stars and growth, funding, launch dates, named users,
   any 2025-2026 news. Flag "gained a following recently" explicitly when true, with the evidence.
7. One strength keel should integrate (concrete mechanism, not a slogan) and one weakness/critique.

## Output format

- A summary table: | # | Tool | Maker | Isolation | Step definition | Done = | Task owner | Adoption (date) | Following recently? |
- Then one paragraph per tool (3-6 sentences) covering items 1-7 with inline URLs.
- End with "## Patterns across this lens" - 3-6 bullets: what converges, what nobody does, what keel should
  copy, and what in this lens argues AGAINST our design (be adversarial; we want the critique).
- Then "## Sources" - every URL you relied on, one per line.
- Cover every tool named in your dispatch; ADD up to 4 more if you find ones that fit the lens better and say
  why you added them. Aim for 10-14 tools total. Target 1500-2500 words.
```


---

# Appendix A — lens A report, verbatim: sandboxes and isolation runtimes for agents

## Landscape lens A - Sandboxes / isolated execution for AI agents

Read date for every number: 2026-09-20 unless stated. GitHub counts are as rendered on the repo page that day.
Focus per dispatch: item 2 (isolation: materialised inputs vs whole repo, network, credentials) and item 5 (does the
provider own/queue tasks or is it compute only).

### Summary table

| # | Tool | Maker | Isolation | Step definition | Done = | Task owner | Adoption (date) | Following recently? |
|---|---|---|---|---|---|---|---|---|
| 1 | E2B | E2B Inc. | Firecracker microVM; whole FS you upload/clone; internet ON by default, allow/deny lists, egress-proxy secret fill | SDK code (templates) | agent/caller say-so | caller drives | 13.9k stars; $21M A (2025), $37M+ total (May 2026); 1B+ sandbox launches (Aug 2026) | yes - 1B launches, "94% of Fortune 100" claim |
| 2 | Daytona | Daytona Platforms | dedicated-kernel sandbox (Kata/Sysbox-class); whole FS; network per tier, `networkBlockAll`/allow lists | SDK code, snapshots | say-so | caller drives | 71.7k stars but repo ARCHIVED Jun 2026; $24M A (Feb 2026) | yes (stars/funding) but went closed-source |
| 3 | Modal Sandboxes | Modal Labs | gVisor on KVM; image + volumes; `block_network`, CIDR/domain allowlists; secrets as env | Python SDK | say-so | caller drives (Modal Functions queue, Sandboxes do not) | $355M C at $4.65B (May 2026); 1B+ sandboxes; >1/3 of revenue | yes |
| 4 | Fly.io Sprites (+Machines) | Fly.io | Firecracker microVM, persistent 100GB disk, ~300ms checkpoint; DNS allow/deny | REST/CLI/MCP | say-so | caller drives | launched 9 Jan 2026; no stars (proprietary) | modest - launch buzz, no numbers |
| 5 | Cloudflare Sandbox SDK / Containers | Cloudflare | container (Cloudflare Containers); whole FS; outbound-Worker egress proxy injects creds, agent never sees token | TypeScript SDK | say-so | caller drives (Durable Object-addressed) | 1.1k stars; GA 13 Apr 2026; Figma named | yes - GA + Agents Week |
| 6 | Vercel Sandbox | Vercel | Firecracker microVM; whole FS; egress ON by default, firewall policy, TLS-terminating proxy CA | JS/Python SDK, CLI | say-so | caller drives | GA 30 Jan 2026; proprietary | moderate |
| 7 | microsandbox | Super Rad Company | libkrun microVM, embeddable, branchable snapshots; allowlist network, volume mounts | YAML + SDK, MCP | say-so | caller drives | 8.3k stars (was ~5k mid-2026) | yes - fastest-growing OSS in lens |
| 8 | Docker Sandboxes (`docker sandbox run`) | Docker | own VMM microVM per agent session; ONLY workspace dir mounted; host proxy swaps credential placeholders; kits = YAML network/creds/files | YAML kits | say-so | none - wraps an interactive agent | GA per Docker blog Jan 30 2026 / arch post 16 Apr 2026; 6 agents supported | yes |
| 9 | Docker MCP Gateway | Docker | one container per MCP server, no direct internet, 1 CPU/2GB caps; secrets via Docker Desktop; interceptors | CLI flags / catalog | n/a (tool proxy) | none | 1.6k stars, MIT | moderate |
| 10 | Anthropic sandbox-runtime (srt) | Anthropic | OS-level: Seatbelt / bubblewrap / Windows user+WFP; live FS with deny-read/allow-write lists; proxy-only network with domain lists | JSON config | say-so | none (Claude Code drives) | 5.3k stars; research preview | yes - "How we contain Claude" (May 2026) |
| 11 | OpenAI Codex CLI sandbox + Codex cloud | OpenAI | CLI: Seatbelt / bwrap+seccomp (+Landlock) / Windows restricted token, workspace-write, `.git` read-only, network OFF; cloud: container per task, setup phase online, agent phase offline, secrets stripped before agent | TOML config; cloud env UI | approval policy + human PR review | CLI: none; CLOUD: user submits task, one container per task, output = diff/PR | part of openai/codex (stars not read) | yes |
| 12 | gVisor | Google | user-space application kernel (runsc) | n/a substrate | n/a | n/a | 19.4k stars | steady |
| 13 | Firecracker | AWS | KVM microVM VMM; ~125ms boot | n/a substrate | n/a | n/a | 36.8k stars; CVE-2026-5747 fixed in latest release | steady - now under E2B, Vercel, Fly, Sprites |
| 14 | Kubernetes SIG agent-sandbox | Kubernetes SIG Apps (Google-initiated) | Pod + RuntimeClass (gVisor/Kata); PVC workspace; CRDs Sandbox/Template/Claim/WarmPool | YAML CRDs | say-so | CLAIM protocol (SandboxClaim from WarmPool) but no task queue | 4.0k stars; K8s blog Mar 2026; Red Hat build Jul 2026; API v1beta1 | yes |
| 15 | Runloop | Runloop AI | VM devboxes; network policies; Agent Gateway / MCP Hub so creds never enter devbox | Blueprints + Scenarios (input, setup, invocation, scorers) | SCORER pass/fail (only tool in lens with a gate) | Runloop schedules benchmark runs; devbox itself caller-driven | $7M seed 30 Jul 2025; 20k concurrent devboxes | moderate |
| 16 | Blaxel | Blaxel (now Baseten) | microVM, <25ms resume, EROFS+tmpfs; per-sandbox egress proxy/firewall; env secrets | SDK/CLI | say-so | Blaxel also hosts agents + batch jobs (owns jobs, not tasks) | $7.3M seed; ACQUIRED by Baseten 10 Sep 2026 | yes (acquisition) |
| 17 | Northflank Sandboxes | Northflank | Kata (Cloud Hypervisor/Firecracker) or gVisor on K8s; BYOC; volumes | API/UI/templates | say-so | jobs + schedules, not agent tasks | "millions of microVMs monthly", 100k+ devs; 100k sandboxes in 24s (ComputeSDK Jul 2026) | moderate |
| 18 | Wassette | Microsoft | Wasm Component (Wasmtime), deny-by-default FS/net/env per tool | OCI components via MCP | n/a | none | 951 stars; "not production ready" | small |
| 19 | Extism | Dylibso | Wasm plugin runtime, host-controlled HTTP/paths | host code | n/a | none | 5.8k stars, BSD-3 | steady |
| 20 | NVIDIA OpenShell (ADDED) | NVIDIA | Landlock + seccomp + egress proxy; YAML policy in 4 domains (fs, net, process, providers); Docker/Podman/microVM/K8s drivers | YAML policy | say-so | none - runtime only | 8.7k stars; GTC/RSAC 2026, Apache-2.0 | yes |
| 21 | ComputeSDK (ADDED) | ComputeSDK | none itself - one API over 30+ providers; weekly public TTI benchmarks | TS code | n/a | none | 275 stars; benchmark leaderboard | small but referenced by vendors |

### Per-tool notes

**1. E2B.** Cloud sandboxes on Firecracker microVMs, Python/JS SDKs, self-hostable runtime (https://e2b.dev/, https://github.com/e2b-dev/E2B). Isolation: a whole filesystem from a template; nothing is materialised for you - you `Sandbox.create()` then upload or clone. Network is ON by default; `allowInternetAccess:false` kills it, and allow/deny lists (CIDR or domain via SNI/Host inspection) are editable on a running sandbox; per-host request transforms with `Secret.fill` inject headers at the egress proxy "ensuring sensitive data never passes through the sandbox", and `iam.tokens` inject workload-identity JWTs (https://docs.e2b.dev/sandbox/internet-access). No step/gate/claim model: the caller's code drives every command; done is whatever the caller decides. Adoption: 13.9k stars (read 2026-09-20), $21M Series A (2025), $37M+ total (May 2026), 1B+ sandbox launches and 7M monthly downloads (Aug 2026) per https://ai.engineer/orgs/e2b and https://bex.co/blog/2026/09/11/ai-sandbox-funding-modal-daytona-e2b. Copy: egress-proxy secret substitution (the sandbox holds a placeholder, the proxy holds the key). Critique: default-open network and no notion of harvesting outputs - everything the agent wrote is equally "the result".

**2. Daytona.** Sandbox infrastructure with sub-90ms starts, snapshots, and SDKs (https://github.com/daytonaio/daytona). Isolation: "dedicated kernel, filesystem, network stack" per sandbox; whole-FS model; network tiered - low tiers cannot loosen, high tiers get full internet by default with `networkBlockAll`, 10 CIDR or 100 domain allow-list entries (https://www.daytona.io/docs/en/network-limits/). Caller-driven; no gate primitive. Adoption: 71.7k stars (read 2026-09-20) and $24M Series A led by FirstMark (Feb 2026, https://www.prnewswire.com/news-releases/daytona-raises-24m-series-a-to-give-every-agent-a-computer-302680740.html) - but the repo was ARCHIVED 11 Jun 2026 and development moved private, citing AI-assisted vulnerability hunting against open isolation code (https://www.daytona.io/dotfiles/updates/daytona-is-going-closed-source); a community fork Nightona has 12 stars (https://github.com/nightona-co/nightona). Copy: the non-overridable tier floor (policy the tenant cannot loosen). Critique: the star count is a legacy of the earlier dev-environment product and the open runtime is dead; "self-hostable" now means the SDK, not the control plane (https://bex.co/blog/2026/09/09/daytona-closed-source-self-hostable-meaning).

**3. Modal Sandboxes.** gVisor-isolated containers on Modal's serverless cloud, GPU-capable (https://modal.com/docs/guide/sandboxes). Isolation: image + mounted `modal.Volume`s; `block_network=True`, `outbound_cidr_allowlist`, `outbound_domain_allowlist` (SNI, beta) compose additively; secrets arrive as env via `modal.Secret`, optional OIDC identity token; Sandboxes deliberately lack the workspace access Functions have (https://modal.com/docs/guide/sandbox-networking). Explicitly caller-driven: "you create a Sandbox, then call `sb.exec()`" - Modal Functions have queues, Sandboxes do not; lifetime max 24h with filesystem snapshots beyond. Adoption: $355M Series C at $4.65B (May 2026), >$300M ARR, 1B+ sandboxes launched, sandboxes >1/3 of revenue (https://siliconangle.com/2026/05/21/serverless-ai-infrastructure-startup-modal-labs-seals-355m-funding-round/, https://modal.com/blog/modal-series-c). Copy: additive network controls where the tightest layer wins and a per-sandbox OIDC identity (a run token the outside world can verify). Critique: gVisor is a software kernel boundary, weaker than a VM; secrets are plain env inside the sandbox.

**4. Fly.io Sprites (and Fly Machines).** Persistent Firecracker microVMs with 100GB sparse NVMe, ~300ms copy-on-write checkpoints and restore to any of the last 5, scale-to-zero billing, MCP endpoint at sprites.dev/mcp (https://simonwillison.net/2026/Jan/9/sprites-dev/, https://docs.sprites.dev/). Isolation: whole persistent disk; DNS-based allow/deny rules per Sprite via API; no documented credential proxy. Caller/agent-driven; nothing owns the task. Adoption: launched 9 Jan 2026; proprietary, no public numbers found (https://www.sdxcentral.com/news/flyio-debuts-sprites-persistent-vms-that-let-ai-agents-keep-their-state/). Copy: checkpoint-before-run / restore-after as the cheap way to make a CONSTRAINED step idempotent - run, harvest, roll back. Critique: persistence is the product, which is the opposite of a materialised-input sandbox; state accretes across steps unless you discipline it.

**5. Cloudflare Sandbox SDK / Containers.** TypeScript SDK giving a Durable-Object-addressed Linux container that sleeps and wakes by name (https://github.com/cloudflare/sandbox-sdk, https://blog.cloudflare.com/sandbox-ga/). Isolation: Cloudflare Containers (container, not VM); whole FS with `gitClone`/`writeFile`; the standout is the programmable egress proxy: an outbound Worker injects credentials per domain so "the agent never sees the token", and policy can tighten "as a task progresses" (https://www.infoq.com/news/2026/04/cloudflare-sandboxes-ga/). Caller-driven; snapshots auto on sleep. Adoption: GA 13 Apr 2026 during Agents Week, 1.1k stars (read 2026-09-20), Figma Make named as production user, active-CPU pricing. Copy: identity-aware egress that is a program, not a list - keel's run token could be the identity the proxy keys on. Critique: container-grade isolation; SDK still "APIs may change before v1.0".

**6. Vercel Sandbox.** Firecracker microVM primitive with root, persistent-by-default filesystem snapshots, JS/Python SDK and CLI (https://vercel.com/docs/sandbox/concepts). Isolation: dedicated kernel; source arrives by git clone, tarball, or snapshot; outbound HTTP ON by default, restricted by the sandbox firewall whose transformation rules TLS-terminate through a per-sandbox proxy CA (containers inside do not inherit it); each agent can get its own Linux user. Caller-driven, session timeout default 5 min. Adoption: GA 30 Jan 2026 (https://vercel.com/blog/vercel-sandbox-is-now-generally-available); proprietary, no star count. Copy: per-sandbox CA plus firewall transformation rules - a proxy that can rewrite requests is how declared outputs become "callable tools". Critique: "a sandbox without a network boundary is only half a sandbox" is Vercel's own blog title, yet egress is on by default.

**7. microsandbox.** Open-source, embeddable libkrun microVM runtime with branchable snapshots, OCI images, MCP server, SDKs in TS/Rust/Python/Go, no daemon (https://github.com/superradcompany/microsandbox). Isolation: hardware VM on macOS/Linux/Windows (WHP); YAML config with per-host/port network allowlists and explicit volume mounts - the closest OSS fit to "only declared inputs present". No task model. Adoption: 8.3k stars (read 2026-09-20) vs ~5k cited mid-2026 (https://rywalker.com/research/microsandbox) - gained a following recently. Copy: VMs spawned as child processes of the runner, so keel's runner daemon needs no cloud account. Critique: single small vendor; cloud in closed beta; Windows path is young.

**8. Docker Sandboxes (`docker sandbox run`).** Docker's purpose-built VMM runs each coding-agent session in a microVM using the host's native hypervisor (Hypervisor.framework / WHP / KVM) with a private Docker daemon (https://www.docker.com/blog/why-microvms-the-architecture-behind-docker-sandboxes/). Isolation: ONLY the project workspace is mounted (same absolute path), git config preserved; a host-side proxy replaces credential placeholders (`sbx-cs-<rand>`) with real keys for Anthropic, GitHub, OpenAI etc. so "the sandbox sees only a sentinel value"; SSH agent forwarded, keys stay on host; network allow/denylist, denylist mode recommended (https://docs.docker.com/ai/sandboxes/security/credentials/). Kits are declarative YAML (`spec.yaml`) bundling network rules, install commands, env, credential sources and files to inject (https://docs.docker.com/ai/sandboxes/customize/kits/). No task ownership - it wraps an interactive Claude Code/Codex/Gemini/Copilot/Kiro/Docker Agent session. Adoption: GA stated 30 Jan 2026 in Docker's blog, architecture post 16 Apr 2026, org-level policy management. Copy: the kit - one YAML naming exactly what a step may see, reach and inject; also placeholder credentials. Critique: still whole-workspace mount including `.git`, so the agent can rewrite history; "done" is when the human closes the session.

**9. Docker MCP Gateway.** CLI plugin that runs each MCP server in its own container with no direct internet, secrets from Docker Desktop, signature verification, and interceptors (`--block-secrets`, `--log-calls`, custom scripts) (https://www.docker.com/blog/docker-mcp-gateway-secure-infrastructure-for-agentic-ai/, https://github.com/docker/mcp-gateway). It isolates TOOLS, not the agent, and owns no tasks. Adoption: 1.6k stars, MIT (read 2026-09-20); 1 CPU / 2GB per-call caps noted at https://bex.co/blog/2026/07/08/docker-mcp-gateway-sandbox-model. Copy: interceptors as the place where a step's declared outputs are enforced at the tool boundary. Critique: a tool proxy adds a hop per call and can only see what flows through MCP.

**10. Anthropic sandbox-runtime (srt).** OS-level sandbox with no container: Seatbelt on macOS, bubblewrap (+seccomp) on Linux, a dedicated `srt-sandbox` user with WFP filters on Windows (alpha) (https://github.com/anthropic-experimental/sandbox-runtime). Isolation: the LIVE filesystem, not a materialised copy - reads are deny-then-allow, writes allow-only; network namespace removed and all traffic forced through HTTP/SOCKS5 proxies with domain allow/deny and resolved-IP checks against loopback/metadata ranges; Unix sockets blocked by default. Credentials are the weak spot: proxy tokens travel as env and, on Windows, as visible command-line args. Anthropic's May 2026 post describes the layered approach and that git credentials are injected by proxy so the agent never holds them (https://simonwillison.net/2026/May/30/how-we-contain-claude/). No task model. Adoption: 5.3k stars (read 2026-09-20). Copy: forcing every byte through a proxy by deleting the network namespace - cheap, container-free, and keel already runs Claude Code under it. Critique: whole-repo visibility by default; nothing harvests outputs.

**11. OpenAI Codex CLI sandbox + Codex cloud.** CLI: modes read-only / workspace-write (default) / danger-full-access; `.git`, `.agents`, `.codex` stay read-only in every writable mode; network OFF by default with an optional `network_proxy` domain allowlist blocking private ranges; Seatbelt / bwrap+seccomp (Landlock as supplementary) / Windows restricted tokens (https://learn.chatgpt.com/docs/agent-approvals-security). Approval policy (`on-request` default) is a human-in-the-loop primitive but binds to COMMANDS, not outcomes. Codex cloud is the one place in this lens where the TOOL owns the task: you submit a prompt, it creates a container per task, clones the repo at a SHA, runs the setup script online, then runs the agent phase with internet off by default and secrets removed before the agent starts, and returns a diff you turn into a PR (https://learn.chatgpt.com/docs/environments/cloud-environment). Adoption: part of openai/codex (star count not read); active 2026 hardening notes at https://codex.danielvaughan.com/2026/05/14/codex-cli-windows-sandbox-engineering-restricted-tokens-acls-elevated-architecture/. Copy: the two-phase lifecycle - secrets and network exist in setup, are gone before the agent runs; `.git` read-only. Critique: "done" is still the agent's diff plus a human's eyeball; there is no test gate.

**12-13. gVisor and Firecracker (substrates).** gVisor is Google's Go user-space kernel (19.4k stars, read 2026-09-20; https://github.com/google/gvisor) used by Modal, Claude.ai and agent-sandbox; Firecracker is AWS's KVM VMM (36.8k stars; latest release fixes CVE-2026-5747; https://github.com/firecracker-microvm/firecracker) under E2B, Vercel, Fly, Sprites. Neither defines steps, gates or ownership. For keel the only question is which the runner uses; both leave "what is in the filesystem" entirely to the caller.

**14. Kubernetes SIG agent-sandbox.** CRD controller for isolated, stateful singleton pods: `Sandbox`, `SandboxTemplate`, `SandboxClaim`, `SandboxWarmPool`; isolation delegated to `RuntimeClass` (gVisor ~150ms overhead or Kata 150-500ms boot); pause/resume/TTL with workspace on PVC (https://github.com/kubernetes-sigs/agent-sandbox, https://agent-sandbox.sigs.k8s.io/). Task ownership: it has a CLAIM protocol - a claim binds a caller to a pre-warmed sandbox "like PVC-to-PV" - but claims are for machines, not work items (https://bex.co/blog/2026/09/04/kubernetes-agent-sandbox-gvisor-kata). Adoption: 4.0k stars (read 2026-09-20), Kubernetes blog Mar 2026, Red Hat supported build Jul 2026, API `agents.x-k8s.io/v1beta1`; Google's Nov 2025 framing at https://opensource.googleblog.com/2025/11/unleashing-autonomous-ai-agents-why-kubernetes-needs-a-new-standard-for-agent-execution.html. Copy: SandboxClaim as the shape of keel's lease - claim from a warm pool, bound to one step, released on exit. Critique: Kubernetes-only, beta, and nothing about inputs or outputs.

**15. Runloop.** Devboxes (VMs), Blueprints (images), Snapshots, and - uniquely here - Scenarios and Benchmarks (https://docs.runloop.ai/, https://runloop.ai/benchmarks). Isolation: "isolated, ephemeral virtual machines", network policies for egress, and credentials never enter the devbox because LLM calls go through an Agent Gateway and tools through an MCP Hub (https://docs.runloop.ai/devboxes/overview). A Scenario "captures input context, environment setup, an agent invocation, and one or more Scenario Scorers that produce pass/fail signals" - the one tool in this lens where done is a scorer, not say-so; Runloop schedules benchmark runs across devboxes (https://github.com/api-evangelist/runloop-ai). Adoption: $7M seed 30 Jul 2025 (https://venturebeat.com/infrastructure/runloop-lands-7m-to-power-ai-coding-agents-with-cloud-based-devboxes), 20k+ concurrent devboxes, SWE-Bench Verified shipped. Copy: Scenario = (materialised input, setup, invocation, scorer) is keel's CONSTRAINED step almost verbatim. Critique: positioned for evals, not production work; the scorer is theirs, not git-tracked.

**16. Blaxel.** Managed agent infra: microVM sandboxes resuming in <25ms from standby, EROFS base + tmpfs overlay, per-sandbox egress proxy/firewall/egress IP, env secrets, REST filesystem API; also hosts agents, batch jobs and MCP servers (https://docs.blaxel.ai/Sandboxes/Overview). It owns JOBS (scheduled/triggered) but not tasks assigned to agents. Adoption: YC S25, $7.3M seed, acquired by Baseten 10 Sep 2026 (https://finance.yahoo.com/technology/ai/articles/baseten-acquires-blaxel-build-infrastructure-164200371.html). Copy: standby snapshot economics (memory+FS frozen, cents to hold). Critique: now an inference company's feature; roadmap risk.

**17. Northflank Sandboxes.** Kata (Cloud Hypervisor/Firecracker) or gVisor per workload on Kubernetes, BYOC into your VPC, volumes 4GB-64TB, indefinite persistence (https://northflank.com/product/sandboxes). Compute plus generic jobs/schedules; no agent-task ownership. Adoption: "millions of microVMs monthly", 100k+ developers; 100k concurrent sandboxes in 24s in ComputeSDK's 2026 invitational, 97ms median TTI (Jul 2026) per https://northflank.com/blog/top-ai-sandbox-platforms-for-code-execution. Copy: per-workload runtime choice (gVisor for cheap steps, Kata for untrusted ones). Critique: their content marketing is most of the "2026 sandbox" search results; discount the self-ranking.

**18-19. Wasm: Wassette and Extism.** Wassette runs Wasm Components as MCP tools on Wasmtime with deny-by-default filesystem, network and env per tool, fetched from OCI registries; 951 stars, MIT, "not production ready" (https://github.com/microsoft/wassette, https://opensource.microsoft.com/blog/2025/08/06/introducing-wassette-webassembly-based-tools-for-ai-agents/). Extism is a general Wasm plugin framework with host-controlled HTTP and paths; 5.8k stars, BSD-3 (https://github.com/extism/extism). Both sandbox TOOLS in microseconds, not agents or repos, so they answer "make declared outputs callable" not "isolate the step". Copy: capability-per-tool deny-by-default. Critique: no POSIX toolchain - no cargo, no git - so useless for a build/test step.

**20. NVIDIA OpenShell (added: the only policy-first agent runtime with real traction).** Rust, Apache-2.0 runtime announced GTC 2026: each agent or sub-agent gets a sandbox enforced by Landlock (fs) and seccomp (process) plus a policy egress proxy; policy is YAML in four domains - filesystem, network, process, providers (endpoint-bound credentials) - static domains lock at creation, network/providers hot-reload; drivers for Docker, Podman, microVM, Kubernetes; runs as a K3s-in-Docker control plane (https://github.com/NVIDIA/openshell, https://docs.nvidia.com/openshell/about/overview). No task queue. Adoption: 8.7k stars (read 2026-09-20), release 0.1.0. Copy: "providers" as a policy domain - credentials bound to an endpoint the agent can call but never read. Critique: heavy control plane; 0.1.0.

**21. ComputeSDK (added: the abstraction layer keel's runner would sit on).** MIT TypeScript SDK with one sandbox API over 30+ providers (E2B, Modal, Daytona, Vercel, Cloudflare, Blaxel, Northflank, Runloop...) and a weekly public time-to-interactive leaderboard (https://github.com/computesdk/computesdk, https://www.computesdk.com/benchmarks/). 275 stars (read 2026-09-20); small, but vendors cite its numbers. Copy: provider-neutral `create / exec / readFile / stop` is the minimal runner contract. Critique: lowest-common-denominator API drops each provider's credential-proxy and network features - exactly the parts keel cares about.

### Patterns across this lens

- **Converged: microVM per session + egress proxy that holds the secret.** Firecracker/libkrun/Kata everywhere except Modal, Cloudflare and gVisor-based paths. Five independent vendors (E2B `Secret.fill`, Docker placeholder swap, Cloudflare outbound Worker, Runloop Agent Gateway, OpenShell providers, Anthropic's git-credential proxy) landed on the same design: the sandbox holds a placeholder or nothing; the proxy at the boundary substitutes the real credential per destination. keel's "no credentials in a CONSTRAINED step" is mainstream; the run token as the ONLY credential, presented to a proxy that maps it to scoped rights, is the concrete mechanism to copy.
- **Nobody materialises inputs.** Every product mounts or clones the WHOLE workspace; the closest are Docker (workspace dir only), Codex (`.git` read-only), microsandbox (explicit volume mounts) and OpenShell (Landlock path allowlist). Not one harvests declared outputs after exit - the result is "whatever the filesystem now contains" or a diff. That is keel's genuinely novel claim in this lens, and also its risk: no vendor SDK has a `materialise(inputs) -> run -> harvest(outputs)` verb, so keel builds it.
- **Nobody gates on a test except Runloop.** "Done" is the caller's say-so or a human approval of commands (Codex) or a diff (Codex cloud). Runloop's Scenario Scorer is the only pass/fail primitive and it lives in an eval product. No tool records a receipt or detects a write with no run behind it. Copy Runloop's Scenario tuple; keep the scorer in git.
- **Task ownership: only Codex cloud owns work; agent-sandbox owns machines.** Every SDK is caller-driven compute. SandboxClaim/WarmPool is the right shape for keel's lease (claim a warm sandbox, bind one step, release), and Codex cloud's two-phase lifecycle (setup online with secrets, agent offline without) is the right shape for a CONSTRAINED step.
- **Against keel's design (1): network-off is a minority default.** E2B, Vercel, Daytona high tiers and Sprites default to internet ON, because package installs, LLM calls and MCP tools need egress; Codex cloud and Docker denylist mode are the exceptions. A CONSTRAINED step with network off cannot call the model - keel must either pre-bake dependencies into an image per step (Codex setup phase) or admit an allowlisted inference endpoint through the proxy, which is the OpenShell "providers" domain. "Network off" as stated is unimplementable for an LLM agent; write it as "egress only to declared providers via the proxy".
- **Against keel's design (2): the market is paying for PERSISTENCE, not ephemerality.** Sprites (100GB disk, checkpoints), Cloudflare (sleep/wake by name), Vercel (persistent by default), Blaxel (standby), agent-sandbox (PVC) all sell state that survives the run because agents that re-install toolchains per step are slow and expensive. Materialised-per-step sandboxes recreate that cost; the mitigation the lens offers is snapshot-then-branch (microsandbox, Sprites, Blaxel): a golden per-process snapshot with deps, branched per step, discarded after harvest. Also note Daytona's June 2026 argument that open isolation code is now an AI-searchable attack surface - keel's isolation should lean on a substrate (Firecracker/libkrun/srt), not roll its own.

### Sources

https://e2b.dev/
https://github.com/e2b-dev/E2B
https://docs.e2b.dev/sandbox/internet-access
https://ai.engineer/orgs/e2b
https://bex.co/blog/2026/09/11/ai-sandbox-funding-modal-daytona-e2b
https://github.com/daytonaio/daytona
https://www.daytona.io/docs/en/network-limits/
https://www.daytona.io/dotfiles/updates/daytona-is-going-closed-source
https://bex.co/blog/2026/09/09/daytona-closed-source-self-hostable-meaning
https://www.prnewswire.com/news-releases/daytona-raises-24m-series-a-to-give-every-agent-a-computer-302680740.html
https://github.com/nightona-co/nightona
https://modal.com/docs/guide/sandboxes
https://modal.com/docs/guide/sandbox-networking
https://modal.com/blog/modal-series-c
https://siliconangle.com/2026/05/21/serverless-ai-infrastructure-startup-modal-labs-seals-355m-funding-round/
https://simonwillison.net/2026/Jan/9/sprites-dev/
https://docs.sprites.dev/
https://www.sdxcentral.com/news/flyio-debuts-sprites-persistent-vms-that-let-ai-agents-keep-their-state/
https://github.com/cloudflare/sandbox-sdk
https://blog.cloudflare.com/sandbox-ga/
https://www.infoq.com/news/2026/04/cloudflare-sandboxes-ga/
https://vercel.com/docs/sandbox/concepts
https://vercel.com/blog/vercel-sandbox-is-now-generally-available
https://github.com/superradcompany/microsandbox
https://rywalker.com/research/microsandbox
https://www.docker.com/blog/docker-sandboxes-run-claude-code-and-other-coding-agents-unsupervised-but-safely/
https://www.docker.com/blog/why-microvms-the-architecture-behind-docker-sandboxes/
https://docs.docker.com/ai/sandboxes/security/credentials/
https://docs.docker.com/ai/sandboxes/customize/kits/
https://docs.docker.com/ai/sandboxes/network-policies/
https://www.docker.com/blog/docker-mcp-gateway-secure-infrastructure-for-agentic-ai/
https://github.com/docker/mcp-gateway
https://bex.co/blog/2026/07/08/docker-mcp-gateway-sandbox-model
https://github.com/anthropic-experimental/sandbox-runtime
https://simonwillison.net/2026/May/30/how-we-contain-claude/
https://learn.chatgpt.com/docs/agent-approvals-security
https://learn.chatgpt.com/docs/environments/cloud-environment
https://codex.danielvaughan.com/2026/05/14/codex-cli-windows-sandbox-engineering-restricted-tokens-acls-elevated-architecture/
https://github.com/google/gvisor
https://github.com/firecracker-microvm/firecracker
https://github.com/kubernetes-sigs/agent-sandbox
https://agent-sandbox.sigs.k8s.io/
https://bex.co/blog/2026/09/04/kubernetes-agent-sandbox-gvisor-kata
https://opensource.googleblog.com/2025/11/unleashing-autonomous-ai-agents-why-kubernetes-needs-a-new-standard-for-agent-execution.html
https://docs.runloop.ai/
https://docs.runloop.ai/devboxes/overview
https://runloop.ai/benchmarks
https://github.com/api-evangelist/runloop-ai
https://venturebeat.com/infrastructure/runloop-lands-7m-to-power-ai-coding-agents-with-cloud-based-devboxes
https://docs.blaxel.ai/Sandboxes/Overview
https://finance.yahoo.com/technology/ai/articles/baseten-acquires-blaxel-build-infrastructure-164200371.html
https://northflank.com/product/sandboxes
https://northflank.com/blog/top-ai-sandbox-platforms-for-code-execution
https://github.com/microsoft/wassette
https://opensource.microsoft.com/blog/2025/08/06/introducing-wassette-webassembly-based-tools-for-ai-agents/
https://github.com/extism/extism
https://github.com/NVIDIA/openshell
https://docs.nvidia.com/openshell/about/overview
https://github.com/computesdk/computesdk
https://www.computesdk.com/benchmarks/
https://gist.github.com/wincent/2752d8d97727577050c043e4ff9e386e


---

# Appendix B — lens B report, verbatim: harnesses where a tracker or orchestrator owns the agent's task

## Lens B - harnesses where a tracker/orchestrator owns the agent's task

Read date for every number below: 2026-09-20 unless stated. Star counts are what the GitHub page showed on that date. All 19 named tools are covered; none added (the dispatch already exceeds the 10-14 target, and every candidate I found - Paseo hub, aq.dev, Pane - is a thinner variant of Multica/Conductor).

### Summary table

| # | Tool | Maker | Isolation | Step definition | Done = | Task owner | Adoption (date) | Following recently? |
|---|---|---|---|---|---|---|---|---|
| 1 | Codex cloud tasks | OpenAI | container per task; setup phase online, agent phase offline | environment config + AGENTS.md; no typed steps | agent stops; human inspects diff, opens PR | tool queues (from Linear/GitHub/Slack); `--attempts 1-4` best-of-N | <1M -> 8M weekly Codex users Feb-Jul 2026 | YES |
| 2 | Symphony | OpenAI (Apache-2.0 spec + Elixir ref) | per-issue persistent workspace dir; agent runs on host | WORKFLOW.md = YAML + prompt template; 4 workspace hooks | issue leaves `active_states` (e.g. reaches Human Review) | ORCHESTRATOR claims (Unclaimed->Claimed->Running->Released), polls Linear | 27.3k stars since 2026-04-27 | YES |
| 3 | Claude Code on the web / Routines / Managed Agents | Anthropic | isolated VM per session; 4 network levels; git creds via proxy outside sandbox | prompt + cloud environment; Routines = saved prompt + triggers | session idles; human reviews diff / creates PR; Routine "green" = no infra error only | tool triggers (schedule/API/GitHub event); no claim protocol | Managed Agents beta 2026-04-08; $0.08/session-hour | YES |
| 4 | Devin | Cognition | fresh VM per session; network/commands deny-by-default with scoped allow | playbooks (`!plan`, `!implement`) | PR opened; human reviews | tracker assigns (assign Linear/Jira ticket to Devin); parallel per-session VMs | $26B valuation 2026-05-27; ~$492M ARR May 2026 | YES |
| 5 | Cursor cloud agents | Anysphere | VM per agent; subagents on separate VMs; secrets injected, outbound domain restriction | `.cursor/environment.json` / snapshot; Automations on events | "merge-ready PR" + video/screenshot artifacts | tool triggers (Slack/Linear/GitHub/webhooks/cron) | >35% of Cursor's own merged PRs by cloud agents (Apr 2026); $60B SpaceX deal Jun 2026 | YES |
| 6 | Copilot coding agent | GitHub | ephemeral GitHub Actions runner; agent firewall; 59-min hard cap | `copilot-setup-steps.yml`, hooks, MCP | draft PR + requests review; cannot approve own PR | tool assigns (issue assignee = Copilot) | Microsoft study: adopters +24% merged PRs (2026) | steady |
| 7 | Jules | Google | fresh cloud VM per task | prompt + setup script; plan shown for approval | branch/PR created; issue comment | tool queues (`jules` label, API, scheduled tasks) | GA 2026-05-19; free tier 15 tasks/day | YES |
| 8 | Factory Droids | Factory | remote workspaces / sandboxes; `droid exec -w` worktree locally | droid exec autonomy tiers; Missions | exit code 0 + JSON `is_error` | tracker delegates (Linear); no built-in queue in exec | $150M C (Apr 2026) -> $5B (Sep 2026) | YES |
| 9 | Ona | Ona (ex-Gitpod; OpenAI acq. announced 2026-06-11) | ephemeral env per agent, OS-level, in-VPC, kernel policy | devcontainer + automations | PR out | tool triggers (Linear/Slack/GitHub/API) | BNY, Vanta, Pearson named; "83% PRs co-authored" | YES (acquisition) |
| 10 | OpenHands (Cloud / Agent Canvas) | All Hands AI | Docker sandbox per conversation | automations (GitHub/Slack/webhook) | PR opened by resolver | tool triggers (issue label) | 88.7k stars; $18.8M A 2025-11-18 | YES |
| 11 | Linear agents (Agent Interaction SDK) | Linear | n/a (protocol only) | n/a | agent emits `response`; session state computed from last activity | TRACKER assigns via delegation; webhooks; 10-s liveness | Codex/Devin/Cursor/Copilot/Factory installed | YES (became the control plane) |
| 12 | Rovo Dev in Jira | Atlassian | dedicated cloud sandbox per session | plan proposed, human refines | human creates PR from sandbox | tracker (Jira work item) starts sessions | Team '26 May 2026; Rovo credits metering | moderate |
| 13 | Gas Town + Beads | Steve Yegge | git worktree per polecat; host machine, full creds | convoys of beads; hooks; Refinery merge queue with gates | `gt done` -> MR bead -> Refinery verifies then merges | Mayor slings beads; `bd update --claim` atomic | GT 18.1k, Beads 27.3k stars; v1.0 Apr 2026 | YES (and backlash) |
| 14 | Vibe Kanban | Bloop (sunset 2026-04-10) | worktree per task attempt | kanban statuses | PR merged / human moves card | human assigns card; agent via MCP can create tasks | 28.1k stars; company shut down | was; now declining |
| 15 | Conductor | Conductor (YC) | worktree per workspace, Mac local | conductor.json scripts | human reviews diff, PR | human creates workspace (Linear/GitHub issue import) | not found (stars n/a, closed source) | modest |
| 16 | Paperclip | paperclipai (MIT) | worktrees / operator branches via adapters | org chart, heartbeats, approval gates | review/approval handoff; budgets | atomic checkout on heartbeat | 81.1k stars since 2026-03-04 | YES (strongest in lens) |
| 17 | Backlog.md | MrLesk | none (markdown in repo) | task frontmatter: acceptance criteria + DoD checkboxes | status field flipped by agent/human | human assigns; agent self-serves via MCP | 6.8k stars | slow |
| 18 | Claude Code Agent Teams / claude-squad | Anthropic / smtg-ai | Teams: shared cwd; squad: worktree + tmux per instance | Teams: shared task list + `TaskCompleted` hook | Teams: teammate marks complete unless hook exit 2 | lead assigns or file-lock self-claim | squad 8.5k stars; Teams experimental | moderate |
| 19 | Multica | multica-ai (Apache-2.0+) | daemon spawns CLI in worktree on your box | issue statuses Todo->In Progress->In Review->Done | human moves In Review -> Done | tool assigns (agent = assignee) | 50.9k stars; repo created 2026-01-13, 5.5k Apr -> 48.4k Sep | YES |

### Per-tool notes

**1. Codex cloud tasks (OpenAI).** Hosted agent that runs each task in an isolated container: a setup phase with network to install dependencies, then an agent phase offline by default ([docs](https://learn.chatgpt.com/docs/cloud), [Docker analysis](https://codex.danielvaughan.com/2026/03/30/codex-cli-docker-containerised-environments/)). Tasks are dispatched from GitHub issues/PRs, GitLab, Linear issues, Slack threads or `codex cloud exec`, which takes `--attempts 1-4` for best-of-N runs of the same task ([agent37](https://www.agent37.com/blog/codex-cloud)). Done = the agent stops and returns a summary + diff; the human "requests a follow-up or opens a PR when ready" - there is no gate, and no receipt beyond the transcript. Adoption: under 1M weekly Codex users in Feb 2026 to 5M by early June and 8M by mid-July after GPT-5.6 ([New Stack](https://thenewstack.io/gpt-5-6-codex-user-surge/)). Strength to copy: the two-phase runtime (online setup, offline agent) is exactly keel's CONSTRAINED step, proven at scale. Weakness: the whole repo is mounted, not declared inputs, and nothing distinguishes "agent finished" from "task done".

**2. Symphony (OpenAI).** Apache-2.0 spec plus Elixir reference implementation released 2026-04-27 that turns Linear into a dispatcher ([OpenAI](https://openai.com/index/open-source-codex-orchestration-symphony/), [repo 27.3k stars](https://github.com/openai/symphony)). Ownership is orchestrator-side: it polls the tracker, filters by `active_states`/`required_labels`, and "serializes state mutations through one authority to avoid duplicate dispatch" with an internal Unclaimed->Claimed->Running->RetryQueued->Released lock independent of the assignee field ([SPEC.md](https://raw.githubusercontent.com/openai/symphony/main/SPEC.md)). Each issue gets a deterministic sanitized workspace directory that persists across runs; hooks `after_create`/`before_run`/`after_run`/`before_remove` run in it; `max_concurrent_agents` (default 10) with per-state caps. A run ends when the worker exits; the issue is finished only when it leaves an active state - "a successful run can end at a workflow-defined handoff state (for example Human Review), not necessarily Done" - and Symphony itself performs no tracker writes; the agent moves the ticket. Some OpenAI teams reported a 500% increase in landed PRs in three weeks ([Help Net Security](https://www.helpnetsecurity.com/2026/04/28/openai-symphony-codex-orchestration-linear/)). Strength: state-driven claim with rebuild-from-workspace on restart (no scheduler DB) - keel's "state is computed" applied to the queue. Weakness: the agent decides the handoff state itself, so "done" is the agent's say-so laundered through a status field; explicitly labelled an engineering preview OpenAI won't maintain as a product.

**3. Claude Code on the web / Routines / Managed Agents (Anthropic).** Cloud sessions run in an isolated Anthropic-managed VM per session with four network levels (None/Trusted/Full/Custom); git credentials and signing keys stay outside the sandbox and a proxy signs on the session's behalf ([docs](https://code.claude.com/docs/en/claude-code-on-the-web)). Routines make the tool the initiator: a saved prompt fires on a cron, an authenticated POST, or a GitHub event, each run being a fresh session that pushes to `claude/`-prefixed branches; the docs are candid that "a green status means the session exited without an infrastructure error. It does not mean the task succeeded" ([routines](https://code.claude.com/docs/en/routines)). Managed Agents (beta header `managed-agents-2026-04-01`, launched 2026-04-08, $0.08/active session-hour) is the API form: agent + environment + session + events, with self-hosted sandboxes ([overview](https://platform.claude.com/docs/en/managed-agents/overview), [pricing](https://www.opslyft.com/blog/anthropic-managed-agents)). Strength: the credential proxy (secrets never inside the VM, attached to matching requests after they leave) is the cleanest answer to "no credentials in the sandbox" in this lens. Weakness: no gate primitive at all - done is a human reading a diff; Auto-fix even replies on GitHub under the human's username.

**4. Devin (Cognition).** Autonomous engineer; assign a Linear/Jira ticket to the Devin service account or apply a playbook label (`!plan`, `!implement`, `!triage`, `!review`) and a session starts ([Linear docs](https://docs.devin.ai/integrations/linear), [Jira docs](https://docs.devin.ai/integrations/jira)). Each session is a fresh VM with deny-by-default network and command scopes and session-scoped credentials ([Fastio](https://fast.io/resources/devin-ai-review/)); Outposts run N sessions in parallel on customer infrastructure. Done = a PR URL attached to the session; the human reviews it, or sends a stop signal from Linear. Adoption: $26B valuation on a >$1B Series D 2026-05-27, ARR from $37M (May 2025) to ~$492M (May 2026) ([Idlen](https://www.idlen.io/news/cognition-devin-25-billion-valuation-windsurf-vibe-coding-april-2026/), [Sacra](https://sacra.com/c/cognition/)). Strength: playbook-as-label - the tracker label selects the process variant, so the tracker, not the prompt, chooses the step chain. Weakness: everything is a session transcript; no declared outputs, no verifiable receipt.

**5. Cursor cloud agents (Anysphere).** VM per agent with cloned repos, dependencies and injected team-scoped secrets; outbound domains restrictable; environments from `.cursor/environment.json`, a Dockerfile or a snapshot ([docs](https://cursor.com/docs/cloud-agent)). Triggered from Slack/Linear/GitHub `@cursor`, API, or Automations (schedule, PR, Sentry, PagerDuty); subagents run on separate VMs so parallel work doesn't collide ([explainx Aug 2026](https://www.explainx.ai/blog/cursor-event-driven-cloud-agents-isolated-vms-august-2026)). Done = a "merge-ready PR" with proof artifacts (video, screenshots, logs) and optional remote-desktop verification. Cursor says >35% of its own merged PRs were agent-authored by April 2026, up from 30% at the February launch ([buildfastwithai](https://www.buildfastwithai.com/blogs/cursor-cloud-agents-development-environments-2026)); SpaceX agreed to acquire Anysphere for $60B in June 2026 ([Wikipedia](https://en.wikipedia.org/wiki/Cursor_(code_editor))). Strength: artifacts attached to the run as evidence - a step's declared outputs can include a recording. Weakness: the artifact is the agent's demo of itself, not an independent test; nothing gates the merge.

**6. GitHub Copilot coding agent.** Assign the issue to `Copilot` (or use the Agents tab, `@copilot` on a PR, Slack/Teams); it runs in an ephemeral GitHub Actions environment behind an agent firewall, opens a draft PR immediately, pushes commits as it works, and requests review; 59-minute hard cap; it cannot approve its own PR and Actions on its PR require human approval to run ([docs](https://docs.github.com/copilot/concepts/agents/coding-agent/about-coding-agent), [changelog 2026-03-24](https://github.blog/changelog/2026-03-24-ask-copilot-to-make-changes-to-any-pull-request/)). Setup via `copilot-setup-steps.yml`, hooks, MCP. A Microsoft study of tens of thousands of engineers found adopters merged ~24% more PRs ([arXiv 2607.01418](https://arxiv.org/abs/2607.01418)). Strength: structural separation of roles - the agent's identity can't approve, and its CI won't run until a human clicks - the closest thing here to keel's "an agent's proposal must pass a step it doesn't control". Weakness: the draft PR is the only surface; the issue itself has no computed state beyond "linked PR".

**7. Google Jules.** Async agent, GA at I/O 2026-05-19 after Labs (Dec 2024) and public beta (Mar 2026) ([digitalapplied](https://www.digitalapplied.com/blog/google-jules-gemini-async-coding-agent-guide)). A task enters a pool via UI, the `jules` label on a GitHub issue, the REST API or Scheduled Tasks; the scheduler provisions a fresh VM, clones, and a Gemini planner writes a plan the human approves before code is written ([docs](https://jules.google/docs), [running tasks](https://jules.google/docs/running-tasks/)). Done = branch/PR created and a comment on the issue. Free tier 15 tasks/day; MCP support for Linear, Neon, Supabase added Feb 2026. Strength: plan approval as an explicit gate BEFORE execution, quoting the plan the human accepted. Weakness: approval is a click, not words; post-plan there is no gate until the PR.

**8. Factory Droids.** `droid exec` is a headless one-shot runner with autonomy tiers (read-only default, `--auto low|medium|high`, `--skip-permissions-unsafe`) that "fails fast on permission violations"; `-w` gives each job a git worktree so parallel jobs on the same repo don't collide; success = exit 0 plus JSON `is_error`; explicitly no built-in queue or claim, distribution left to CI matrices or xargs ([docs](https://docs.factory.ai/droid-exec/overview)). Linear delegation spins a Droid in an isolated cloud workspace ([Linear integration](https://linear.app/integrations/factory)). Funding: $150M Series C at $1.5B on 2026-04-16 (Khosla), C-2 at $4B in July, then $200M to $5B ([factory.ai](https://factory.ai/news/series-c), [TFN](https://techfundingnews.com/factory-jumps-to-5b-in-5-months-with-200m-for-its-ai-droids/)); named users Nvidia, Adobe, EY, Morgan Stanley. Strength: declared autonomy tier per invocation with hard refusal above it - a per-step capability ceiling keel can express as the step class. Weakness: success is the agent's exit code; the tier limits what it may touch, not what it must prove.

**9. Ona (formerly Gitpod).** "Task in, pull request out": each agent runs in its own ephemeral environment with OS-level isolation, inside the customer VPC, with kernel-level policy enforcement, scoped credentials and audit trails; triggers from Linear, Slack, GitHub PR automations, API/CLI and schedules ([ona.com](https://ona.com/)). Named customers BNY, GSR, Vanta, Pearson, EquipmentShare; claims "83% of PRs co-authored by Ona" internally. OpenAI announced its acquisition 2026-06-11 to give Codex persistent long-running environments ([digitalapplied](https://www.digitalapplied.com/blog/openai-acquires-ona-cloud-execution-long-running-agents-analysis)). Strength: Guardrails as a separate product layer - policies enforced by the environment, not the prompt. Weakness: done is still a PR; the acquisition means the standalone roadmap is uncertain.

**10. OpenHands Cloud / Agent Canvas (All Hands AI).** MIT; 88.7k stars, release 1.20.0 ([repo](https://github.com/OpenHands/OpenHands)); $18.8M Series A 2025-11-18 ([BusinessWire](https://www.businesswire.com/news/home/20251118768131/en/OpenHands-Raises-$18.8M-Series-A-to-Bring-Open-Source-Cloud-Coding-Agents-to-Enterprises)). The control plane spawns a per-conversation Docker agent-server container and receives event webhooks back; the GitHub/GitLab resolver fires when an issue gets a configurable label, edits, tests, and opens a PR; automations run on GitHub/Slack/webhook events ([resolver](https://nebuladeck.dev/blog/openhands-coding-agent-guide), [DockerSandboxService](https://newreleases.io/project/github/OpenHands/OpenHands/release/cloud-1.39.0)). Now positions as a control center for any ACP agent (Claude Code, Codex, Gemini). Strength: the event-webhook channel from sandbox to control plane is the harvest path keel wants, already open source. Weakness: done is PR-opened; the resolver has no gate between "tests ran" and "PR opened".

**11. Linear agents (Agent Interaction SDK).** Linear is the tracker that owns the task: delegating an issue (assignment) or @mentioning an agent creates an `AgentSession`; the agent receives webhooks and answers with typed activities (`thought`, `action`, `elicitation`, `response`, `error`); state (`pending`, `active`, `awaitingInput`, `complete`, `stale`, `error`) is "tracked automatically based on the last emitted activity", the webhook must ack in 5 s and post an activity within 10 s or be marked unresponsive; PRs are attached as `externalUrls` ([agent-interaction](https://linear.app/developers/agent-interaction), [design note](https://linear.app/now/our-approach-to-building-the-agent-interaction-sdk)). Codex, Devin, Cursor, Copilot, Factory all ship as Linear agents ([integrations](https://linear.app/integrations/agents)); Symphony (#2) uses it as its control plane. Strength: computed session state from a typed activity stream plus a liveness deadline - keel's lease heartbeat, specified. Weakness: `complete` is whatever the agent's last `response` says; Linear has no notion of a test.

**12. Atlassian Rovo Dev in Jira.** From a Jira work item you start one or several cloud sessions in a dedicated sandbox configured with secrets/env vars; it proposes a plan the human refines, executes, runs tests, and the human creates the PR from the sandbox; consumes Rovo credits ([support doc](https://support.atlassian.com/rovo/docs/work-with-rovo-dev-in-jira/), [Atlassian blog](https://www.atlassian.com/blog/company-news/rovo-dev-in-jira)). Team '26 (May 2026) added assignable Rovo agents and Jira-automation triggers for third-party coding agents ([buzzclan](https://buzzclan.com/atlassian/atlassian-rovo/)). Strength: the tracker also brokers third-party agents through its automation rules - one queue, many runners. Weakness: no public claim/lease semantics; done is a human clicking "create PR".

**13. Gas Town + Beads (Steve Yegge).** Beads (27.3k stars, MIT, now Dolt-backed) is a git-synced dependency graph issue tracker: `bd ready` lists unblocked work, `bd update --claim` atomically sets assignee + in_progress, `bd close` finishes; hash IDs avoid merge collisions ([repo](https://github.com/steveyegge/beads)). Gas Town (18.1k stars, open-sourced 2026-01-01, v1.0 April 2026) layers a Mayor that slings beads/convoys to polecats each in its own git worktree ("hooks"), a Witness/Deacon for stuck-worker recovery, and a Refinery merge queue that batches MRs and "runs verification gates on the merged stack" before merging ([repo](https://github.com/gastownhall/gastown), [Yegge](https://steve-yegge.medium.com/gas-town-from-clown-show-to-v1-0-c239d9a407ec)). Critiques: "ultimate token burner", 141 orphaned processes in a week, thin merge-queue observability ([Tenzin](https://tenzinwangdhen.com/posts/gastown-good-bad-ugly/)). Strength: the Refinery - done is a merge queue's gate on the stacked result, not the polecat's `gt done`. Weakness: full host credentials for every polecat; the chaos is the isolation story.

**14. Vibe Kanban (Bloop).** Kanban board that runs a worktree per task attempt for 10+ agent CLIs, with inline diff review, dev server, PR creation, and an MCP server so agents create tasks ([repo 28.1k stars](https://github.com/BloopAI/vibe-kanban)). Bloop shut down 2026-04-10 - "the vast majority are free users and we couldn't find a business model"; server features sunset 30 days later, local workspaces live on as community OSS ([shutdown post](https://www.vibekanban.com/blog/shutdown)). Done = human moves the card / merges. Strength: the attempt as first-class object (several attempts per task, each its own worktree). Weakness/lesson: a free board with no gate and no server truth had no moat.

**15. Conductor (conductor.build).** Free Mac app: every workspace is a git worktree with its own branch, terminal, diff and review path; `conductor.json` setup/run/archive scripts; imports Linear/GitHub issues; runs Claude Code, Codex, Cursor, OpenCode in parallel ([docs](https://www.conductor.build/docs), [YC](https://ycombinator.com/companies/conductor)). Done = human reviews diff and opens PR. Adoption numbers: not found. Strength: lifecycle scripts per workspace (setup/run/archive) as authored config. Weakness: human-driven; nothing owns or claims the task.

**16. Paperclip.** MIT "control plane, not execution plane" for agent companies: org chart, goals, issues with single assignee/blockers/parents, monthly per-agent budgets, board approval gates; agents wake on heartbeats and check out issues atomically so "no double-work and no runaway spend"; adapters for Claude Code, Codex, OpenClaw; execution isolation via worktrees/operator branches ([repo 81.1k stars](https://github.com/paperclipai/paperclip), [PRODUCT.md](https://github.com/paperclipai/paperclip/blob/master/doc/PRODUCT.md), [contabo](https://contabo.com/blog/what-is-paperclip-ai/)). Launched 2026-03-04, 30k stars in three weeks, ~70k by June. Strength: budget as a lease bound - the checkout debits a budget, so a runaway agent loses its claim. Weakness: PRODUCT.md deliberately leaves lifecycle, lease and verification unspecified; done is a review/approval handoff decided by another agent.

**17. Backlog.md (MrLesk).** MIT, 6.8k stars; tasks are markdown files with frontmatter, checkbox acceptance criteria and a Definition-of-Done list; `backlog mcp start` exposes workflow guidance to Claude Code/Codex/Gemini/Kiro ([repo](https://github.com/MrLesk/Backlog.md)). Done = status field flipped, by the agent or human; no isolation, no claim. Strength: DoD checkboxes in the task file - the nearest cousin to keel's text-is-truth. Weakness: the agent ticks its own boxes; nothing computes done.

**18. Claude Code Agent Teams / claude-squad.** Agent Teams (experimental): a lead populates a shared task list with dependencies; teammates self-claim "using file locking to prevent race conditions"; all share one working directory ("two teammates editing the same file leads to overwrites"); `TaskCompleted` hook exit 2 blocks completion, `TeammateIdle` exit 2 keeps the teammate working; limitations note "teammates sometimes fail to mark tasks as completed" ([docs](https://code.claude.com/docs/en/agent-teams)). claude-squad (8.5k stars, AGPL) instead gives each instance a tmux session and worktree, with review/checkout before merge ([repo](https://github.com/smtg-ai/claude-squad)). Strength: `TaskCompleted` as a veto hook is a gate at the completion write. Weakness: shared cwd, and the veto is a shell script the lead configures, not a bound test.

**19. Multica.** Apache-2.0-plus, 50.9k stars from repo creation 2026-01-13 (5.5k early April, 28.9k mid-May, 48.4k September) ([repo](https://github.com/multica-ai/multica), [star-history](https://www.star-history.com/multica-ai/multica/)). Linear-like board where the agent is the assignee; a local daemon detects 26 agent CLIs and spawns each in a git worktree; the agent comments progress and moves the issue to In Review; a human moves it to Done before merge; hosted or self-hosted (Go/Postgres). Strength: the daemon-pulls-from-board topology with code staying on your machines is keel's runner-daemon shape exactly. Weakness: no lease/heartbeat visible in docs, and Done is a human drag with no test behind it.

### Patterns across this lens

- **Converges: per-task isolation + PR as the only output.** Every hosted tool (1,3-10,12) gives one VM/container per task and ends in a pull request; every local orchestrator (13-19) gives one git worktree per task and ends in a diff. Nobody materialises only declared inputs - the whole repo is always mounted. "Done" is universally either the agent stopping or a human clicking; only Gas Town's Refinery and Agent Teams' `TaskCompleted` hook put anything automated between the agent's claim and the status flip.
- **Converges: the tracker is winning ownership.** Linear (11) is now the de-facto control plane - Codex, Devin, Cursor, Copilot, Factory and Symphony all take work from it, and Linear computes session state from the activity stream with a 10-second liveness deadline. Symphony and Paperclip add an orchestrator-side atomic claim; Beads adds `--claim`. Nobody exposes a lease with expiry + run token; Linear's `stale` is the closest.
- **Nobody does:** declared-output harvesting, drift detection (a write with no run), verbatim human words as the acceptance record, or step nesting. Cursor's video artifacts and Codex's best-of-N are the only "evidence" primitives, and both are agent-produced.
- **Copy:** Symphony's rebuild-the-queue-from-workspace-state on restart and its "handoff state, not Done" distinction; Anthropic's credential proxy (secrets attached after the request leaves the sandbox); Copilot's structural rule that the agent identity cannot approve and its CI needs a human click; Paperclip's budget-debiting checkout as a lease bound; Linear's typed activity stream as the runner->server protocol.
- **Against our design:** (a) The market voted for "whole repo in the sandbox, PR out" and the only tool that tried to be a strict board with truth on a server (Vibe Kanban) died for lack of a business model; the winners (Paperclip 81k, Multica 51k) grew by being permissive and adapter-agnostic, not by gating. (b) Anthropic's own docs concede a green run "does not mean the task succeeded" and ship anyway - buyers tolerate that. (c) Materialising only declared inputs fights every agent's need to grep the repo; Codex solved data exfiltration with network-off instead, cheaper and less brittle. (d) Yegge's numbers (141 orphaned processes, "token burner") say the cost of an orchestrator is the orchestrator, and keel's typed steps add authoring friction that CLAUDE.md already names as the #1 risk. (e) Every tool here has a CLI in addition to a UI; "browser UI, no CLI" would make keel the only runner-owning system an agent cannot script.

### Sources

https://openai.com/index/open-source-codex-orchestration-symphony/
https://github.com/openai/symphony
https://raw.githubusercontent.com/openai/symphony/main/SPEC.md
https://codex.danielvaughan.com/2026/04/28/openai-symphony-codex-orchestration-linear-autonomous-agent-workflows/
https://www.helpnetsecurity.com/2026/04/28/openai-symphony-codex-orchestration-linear/
https://learn.chatgpt.com/docs/cloud
https://codex.danielvaughan.com/2026/03/30/codex-cli-docker-containerised-environments/
https://www.agent37.com/blog/codex-cloud
https://thenewstack.io/gpt-5-6-codex-user-surge/
https://code.claude.com/docs/en/claude-code-on-the-web
https://code.claude.com/docs/en/routines
https://code.claude.com/docs/en/agent-teams
https://platform.claude.com/docs/en/managed-agents/overview
https://www.opslyft.com/blog/anthropic-managed-agents
https://docs.devin.ai/integrations/linear
https://docs.devin.ai/integrations/jira
https://fast.io/resources/devin-ai-review/
https://www.idlen.io/news/cognition-devin-25-billion-valuation-windsurf-vibe-coding-april-2026/
https://sacra.com/c/cognition/
https://cursor.com/docs/cloud-agent
https://www.explainx.ai/blog/cursor-event-driven-cloud-agents-isolated-vms-august-2026
https://www.buildfastwithai.com/blogs/cursor-cloud-agents-development-environments-2026
https://en.wikipedia.org/wiki/Cursor_(code_editor)
https://docs.github.com/copilot/concepts/agents/coding-agent/about-coding-agent
https://github.blog/changelog/2026-03-24-ask-copilot-to-make-changes-to-any-pull-request/
https://arxiv.org/abs/2607.01418
https://jules.google/docs
https://jules.google/docs/running-tasks/
https://www.digitalapplied.com/blog/google-jules-gemini-async-coding-agent-guide
https://docs.factory.ai/droid-exec/overview
https://linear.app/integrations/factory
https://factory.ai/news/series-c
https://techfundingnews.com/factory-jumps-to-5b-in-5-months-with-200m-for-its-ai-droids/
https://ona.com/
https://www.digitalapplied.com/blog/openai-acquires-ona-cloud-execution-long-running-agents-analysis
https://github.com/OpenHands/OpenHands
https://www.businesswire.com/news/home/20251118768131/en/OpenHands-Raises-$18.8M-Series-A-to-Bring-Open-Source-Cloud-Coding-Agents-to-Enterprises
https://newreleases.io/project/github/OpenHands/OpenHands/release/cloud-1.39.0
https://nebuladeck.dev/blog/openhands-coding-agent-guide
https://linear.app/developers/agent-interaction
https://linear.app/now/our-approach-to-building-the-agent-interaction-sdk
https://linear.app/integrations/agents
https://support.atlassian.com/rovo/docs/work-with-rovo-dev-in-jira/
https://www.atlassian.com/blog/company-news/rovo-dev-in-jira
https://buzzclan.com/atlassian/atlassian-rovo/
https://github.com/gastownhall/gastown
https://github.com/steveyegge/beads
https://steve-yegge.medium.com/gas-town-from-clown-show-to-v1-0-c239d9a407ec
https://tenzinwangdhen.com/posts/gastown-good-bad-ugly/
https://github.com/BloopAI/vibe-kanban
https://www.vibekanban.com/blog/shutdown
https://www.conductor.build/docs
https://ycombinator.com/companies/conductor
https://github.com/paperclipai/paperclip
https://github.com/paperclipai/paperclip/blob/master/doc/PRODUCT.md
https://contabo.com/blog/what-is-paperclip-ai/
https://github.com/MrLesk/Backlog.md
https://github.com/smtg-ai/claude-squad
https://github.com/multica-ai/multica
https://www.star-history.com/multica-ai/multica/


---

# Appendix C — lens C report, verbatim: durable-execution and workflow engines, agent orchestration frameworks

## Landscape lens C - durable-execution / workflow engines with typed steps, and agent orchestration frameworks

Read 2026-09-20/21. Star counts are from `api.github.com` on 2026-09-21 unless another date is given. Focus per dispatch: item 3 (step typing, nesting, hooks, definition versioning) and item 4 (HITL primitive, what "done" is, replay/determinism contract, how LLM steps are handled). Isolation and task-ownership answers are terse because in this lens almost nobody materialises inputs or sandboxes per step - that is the first finding.

### Summary table

| # | Tool | Maker | Isolation | Step definition | Done = | Task owner | Adoption (date) | Following recently? |
|---|---|---|---|---|---|---|---|---|
| 1 | Temporal | Temporal Technologies | none (worker process); sandbox only via activities calling Modal/E2B/Daytona/Docker | code (Go/Java/Py/TS/.NET); activities typed by language; child workflows; interceptors; patch()/Worker Versioning | activity returned / workflow returned; Signal+Update for approvals | tool queues, worker polls task queue (lease = task token, heartbeat) | 23.2k stars; $300M D @ $5B Feb-2026; $550M E @ $12.55B Sep-2026; $250M ARR; OpenAI/Netflix/Nvidia | YES - agentic-AI wave, 200%+ YoY |
| 2 | Restate | Restate GmbH (Berlin) | none (your service process) | code (TS/Java/Kotlin/Go/Py/Rust); typed handlers; virtual objects; service-to-service calls; deployment pinning | handler returns; awakeables/promises for HITL | tool invokes handlers over HTTP, journals; workers = your deployments | 4.4k stars; $7M seed Jun-2024; ~20 staff | modest; agent-SDK integrations (Pydantic AI, ADK) 2026 |
| 3 | Inngest (AgentKit) | Inngest Inc. | none (runs in your app/serverless) | code (TS/Py/Go); step.run/step.ai; step.invoke for nesting; middleware hooks; no formal versioning | step returns; step.waitForEvent for HITL | tool dispatches via event; function re-executes with memoised steps | 5.9k stars; $21M A (Altimeter) 2025 | moderate - AgentKit, Series A |
| 4 | Trigger.dev | Trigger.dev Ltd (London, YC) | container per run (MicroVM planned); whole deployed bundle | code (TS); Zod-typed payloads; triggerAndWait subtasks; onStart/onSuccess/onFailure lifecycle hooks; runs pinned to deployment version | task returns; waitpoint token completed | tool queues runs; workers pull; process checkpoint/restore | 16.4k stars; $16M A Dec-2025; 30k devs, 100Ms runs/mo (Jun-2026) | YES |
| 5 | DBOS | DBOS Inc. | none (library in your process) | code (Py/TS/Go/Java/Kotlin); decorators; child workflows w/ lineage; workflow versioning + patching (Mar-2026) | step checkpointed to Postgres; recv/set_event for HITL | Postgres queues; workers pull by version | 1.6k stars (py); $8.5M seed Mar-2024 | modest; agent SDK integrations |
| 6 | Hatchet | Hatchet (YC W24) | none (worker) | code (Py/TS/Go); Pydantic/Zod-typed task I/O; DAGs + durable tasks; child spawning; on-failure tasks | task returns; wait_for_events | Postgres-backed queue; worker leases | 8.0k stars; $500K (YC, Ritual) Apr-2024 | moderate |
| 7 | Windmill | Windmill Labs | nsjail + PID namespace per job; whole script | scripts (TS/Py/Go/Bash/SQL) with typed sigs -> JSON schema; flows (OpenFlow) in UI/YAML; branches/loops/subflows; error handlers | step returns; approval step (suspend + resume URL) | tool queues jobs; workers pull | 18.0k stars; $15M total, $5M A Oct-2024; $2.7M rev 2024 | steady |
| 8 | Kestra | Kestra Technologies (Paris) | task runners (process/Docker/K8s) | YAML flows; typed inputs/outputs; subflows; flow revisions; listeners/triggers | task succeeds; Pause task / HITL approval | server schedules, workers execute | 28.2k stars; $25M A Mar-2026 ($36M total); 2.0 released | YES - Series A, 1.0 + 2.0 in 12 months |
| 9 | Dagster | Dagster Labs -> Prefect (acq. Jul-2026) | none (code location process / k8s job) | Python asset graph; typed I/O managers; asset checks; Components YAML | materialisation event + asset checks pass | tool launches runs; agents (Dagster+ hybrid) pull | 16.2k stars; $48.8M total; acquired by Prefect 2026-07-13 | declining independence; MCP/`dg api` for agents |
| 10 | Prefect | Prefect Technologies | none (flow run process / k8s / ECS) | Python decorators; Pydantic-typed params; subflows; state-change hooks; deployments versioned | task/flow state Completed; pause_flow_run(wait_for_input=Model) | workers poll work pools | 23.9k stars; acquired Dagster Jul-2026 | YES (consolidator) |
| 11 | Apache Airflow 3 | ASF (Astronomer) | none (worker) or KubernetesPodOperator | Python DAGs; XCom untyped; TaskGroups/TriggerDagRun; callbacks; DAG versioning (3.x) | task success; HITL/ApprovalOperator `awaiting_input` (3.3) | scheduler queues, executor/worker runs | 46.9k stars; HITL added 3.1 (2025) | steady, huge base |
| 12 | Argo Workflows | Argo project (CNCF) | pod per step (container); artifacts materialised in/out | YAML CRDs; typed-by-name params/artifacts; templateRef, DAG/steps nesting; exit handlers/hooks; WorkflowTemplate versions via git | pod exit 0; `suspend` template + resume with supplied params | controller schedules pods | 17.0k stars | steady (K8s base) |
| 13 | Camunda 8 / BPMN | Camunda Services GmbH | none (job workers) | BPMN XML + DMN; typed-ish via variables/FEEL; call activities, ad-hoc subprocess; execution/task listeners; process versions + instance migration (8.9) | job completed; User Task (forms, assignment, listeners) | Zeebe queues jobs; workers activate with lease/timeout | 4.3k stars (monorepo); 8.9 Apr-2026 | YES in enterprise "agentic orchestration" |
| 14 | Conductor (Netflix/Orkes) | conductor-oss / Orkes | none (worker) | JSON workflow defs; task defs w/ input/output keys; SUB_WORKFLOW; versioned defs | task COMPLETED; Human task w/ user forms, SLAs, escalation | server queues, workers poll (lease = responseTimeout) | 32.2k stars; Orkes $60M Apr-2026 | YES |
| 15 | Dapr Workflows + Dapr Agents | Dapr (CNCF graduated; Diagrid, Nvidia) | none (sidecar + app) | code (Py/.NET/Java/Go); activities; child workflows; no formal versioning (name-suffix) | activity returns; wait_for_external_event | sidecar runtime schedules | 26.1k (dapr) / 748 (dapr-agents); Agents 1.0 GA Mar-2026 | moderate |
| 16 | LangGraph (+ Platform) | LangChain Inc. | none | Python/TS graph; TypedDict/Pydantic state; subgraphs; pre/post model hooks; no def versioning (assistants versioned in Platform) | node returns; `interrupt()` + `Command(resume=)` | caller drives; Platform queues runs | 42.0k stars; 1.0 Oct-2025 | YES (largest agent framework) |
| 17 | Mastra | Mastra (YC W25) | none | TS; Zod-typed createStep; nested workflows; .then/.branch/.parallel; no def versioning | step returns; suspend()/resume() snapshot | caller drives | 28.2k stars; $13M seed; 1.0 Jan-2026 | YES |
| 18 | CrewAI Flows | CrewAI Inc. | none | Python @start/@listen/@router; Pydantic state; crews nested in flows | method returns; @human_feedback, @persist | caller drives | 58.8k stars; $18M (Oct-2024) | YES |
| 19 | Microsoft Agent Framework | Microsoft | none (Foundry Hosted Agents for hosting) | Python/.NET executors + edges; typed messages; sub-workflows; checkpoint at superstep | executor emits; RequestInfoExecutor/RequestPort | caller drives; Foundry hosts | 13.6k stars; 1.0 GA Apr-2026 | YES (SK+AutoGen merged) |
| 20 | OpenAI Agents SDK | OpenAI | none | Python/TS; Pydantic/Zod tools; handoffs; hooks (RunHooks/AgentHooks) | run returns; needs_approval + RunState serialise/resume (v0.8 Feb-2026) | caller drives | 29.6k stars | YES |
| 21 | Google ADK | Google | none (Agent Engine hosting) | Python/Java/Go/Kotlin; Sequential/Parallel/Loop agents; callbacks before/after model & tool; no def versioning | agent event; LongRunningFunctionTool + ResumabilityConfig | caller drives; Runner | 21.6k stars; Kotlin 1.0 2026 | YES |
| 22 | Pydantic AI (graph/durable) | Pydantic Services | none | Python; typed deps/output; pydantic-graph nodes; durable via Temporal/DBOS/Prefect/Restate | run returns; CallDeferred / ApprovalRequired -> DeferredToolRequests | caller drives (or host engine) | 20.1k stars | YES |
| 23 | Vercel Workflow DevKit | Vercel | step = isolated function route; workflow = replayed orchestrator | TS `"use workflow"`/`"use step"`; TS types; hooks (defineHook); runs pinned to deployment | step returns; hook.resume(token) | Vercel queue invokes routes | 2.4k stars; launched Oct-2025 | YES (new, growing) |

### Per-tool notes

**1. Temporal.** Durable-execution platform (open-source server + Temporal Cloud) from Temporal Technologies. No isolation of its own: workflow and activity code run in your worker; the Apr-2026 "agentic sandboxes" post wraps Modal/E2B/Daytona/Docker sandbox clients so every sandbox op runs as an Activity ([temporal.io](https://temporal.io/blog/introducing-temporal-and-agentic-sandboxes-openai-agents-sdk)). Steps are typed by the host language; child workflows nest; interceptors are the before/after hook; definitions evolve via `patch()` markers or Worker Versioning (pinned build IDs) ([docs](https://docs.temporal.io/develop/python/workflows/versioning)). DETERMINISM CONTRACT: workflow code must produce the same command sequence on replay of Event History; every LLM call, tool call, clock or random goes in an Activity, whose result is recorded once - the OpenAI Agents SDK integration (GA 2026-03-23) does exactly this, `activity_as_tool` turning activities into agent tools ([temporal.io](https://temporal.io/blog/announcing-openai-agents-sdk-integration)). "Done" = the workflow function returned; HITL = Signals/Updates awaited inside the workflow; no notion of a step being verified by a test, and drift detection is n/a. Adoption is the strongest in the lens: 23,200 stars (2026-09-21), $300M Series D at $5B 2026-02-17 ([businesswire](https://www.businesswire.com/news/home/20260217453156/en/Temporal-Raises-$300M-Series-D-to-Make-Agentic-AI-Real-for-Companies)), $550M Series E at $12.55B on 2026-09-15 with >$250M ARR and customers OpenAI, Netflix, Nvidia, JPMorgan ([yahoo](https://finance.yahoo.com/technology/articles/temporal-reaches-12-5bn-valuation-092853078.html)). Integrate: the task-token lease (worker polls a task queue, heartbeats, the server re-queues on timeout) and the "non-deterministic work is an Activity with a recorded result" rule. Critique: the determinism contract is a footgun that spawned a whole tooling layer (patching, versioning, a lint rule set) and it has nothing to say about what a result *means*.

**2. Restate.** Durable-execution engine from Restate GmbH (Berlin; Flink founders). No isolation; your handlers are HTTP services the Restate server invokes. Steps are typed handler functions; `ctx.run("name", fn)` journals a non-deterministic side effect; virtual objects give keyed state; services call services. Versioning is the cleanest here: deployments are immutable, each version gets its own endpoint, in-flight invocations stay pinned, new ones route to latest ([restate.dev 2026-03-11](https://www.restate.dev/blog/dealing-with-versioning-in-long-running-agents)). DETERMINISM CONTRACT: journal replay; a mismatch between journal and code "fails loudly rather than silently continuing." LLM calls are `ctx.run` entries so the recorded response is reused on replay (Pydantic AI's `RestateAgent` does this) ([pydantic.dev](https://pydantic.dev/articles/restate-durable-execution-pydanticai)). HITL = awakeables/durable promises the outside world resolves by ID. 4,449 stars (2026-09-21); $7M seed Jun-2024 ([TechCrunch](https://techcrunch.com/2024/06/12/restate-raises-7m-for-its-lightweight-workflows-as-code-platform/)); no later round found. Integrate: fail-loud journal/code mismatch as a drift guard model, and deployment pinning by version. Critique: small team, small community, no isolation story.

**3. Inngest (AgentKit).** Event-driven durable functions (TS/Py/Go) from Inngest Inc.; AgentKit is its agent layer. No isolation. `step.run` (memoised side effect), `step.invoke` (nested function), `step.waitForEvent` (HITL), `step.ai.*` for model calls; middleware is the hook system. DETERMINISM CONTRACT: step memoisation - the function re-executes from the top on every wake, completed steps short-circuit with stored results, so "an LLM call that originally returned 'use the search tool' will return that same result on replay" ([inngest docs](https://www.inngest.com/docs/learn/durable-agents)). Code change between runs is not addressed in the docs; step IDs are the join key so renaming one re-runs it. HITL = `waitForEvent` with match expression and timeout ([agentkit](https://agentkit.inngest.com/advanced-patterns/human-in-the-loop)). 5,855 stars (2026-09-21); $21M Series A led by Altimeter, 2025 ([inngest.com](https://www.inngest.com/blog/announcing-inngest-series-a)). Integrate: "wait for an event matching an expression" as the shape of a gate that resumes a step. Critique: memoisation-by-step-ID is fragile under refactor and no versioning primitive exists.

**4. Trigger.dev.** Open-source TS background-task platform (YC; London) whose v4 (GA 2025-08-18) targets agents ([trigger.dev](https://trigger.dev/launchweek/2/trigger-v4-ga)). Each run is a container (MicroVMs planned) running your whole deployed bundle, credentials via env; not materialised inputs. Tasks are `task({id, run})` with Zod-typed payloads, `triggerAndWait`/`batchTriggerAndWait` for child tasks, `onStart/onSuccess/onFailure` lifecycle hooks, and runs pin to the deployment version they started on. No replay/determinism contract: durability is process checkpoint/restore plus waitpoints; a run may be resumed on a different machine. HITL = waitpoint tokens with callback URLs and timeouts, "do some work then allow a human to reject, suggest changes, or approve" ([v4 beta](https://trigger.dev/blog/v4-beta-launch)). "Done" = the task returned. 16,355 stars (2026-09-21); $16M Series A led by Standard Capital 2025-12-17; 30k developers, hundreds of millions of agent runs/month (Jun-2026) ([trigger.dev](https://trigger.dev/blog/series-a)). Gained a following recently - yes. Integrate: waitpoint tokens as first-class objects with their own URL and TTL. Critique: whole-bundle containers with live secrets are the opposite of a materialised sandbox.

**5. DBOS.** Postgres-backed durable-execution library (Py/TS/Go/Java/Kotlin) from DBOS Inc. (Stonebraker, Zaharia). No isolation - it embeds in your process. `@workflow`/`@step` decorators, child workflows with lineage tracking, queues, and since Mar-2026 "workflow patching in addition to versioning" plus a fork-workflow tool exposed over MCP ([dbos.dev](https://www.dbos.dev/blog/dbos-new-features-march-2026)). DETERMINISM CONTRACT: step outputs are checkpointed to Postgres; on recovery the workflow function re-runs and completed steps return their checkpoint; each workflow is pinned to the application version that started it, so old code must keep running to finish old runs. LLM calls are steps (OpenAI Agents SDK and Pydantic AI integrations). HITL = `DBOS.recv`/`set_event`. 1,579 stars on the Python repo (2026-09-21); $8.5M seed 2024-03-13 ([builtinboston](https://www.builtinboston.com/articles/dbos-launches-raises-8m-seed-20240313)); no later round found. Integrate: "fork a workflow from step N" as a replay-from-evidence primitive. Critique: durability keyed to a Postgres row in *your* database; nothing about who may run what.

**6. Hatchet.** Postgres-backed task orchestrator (YC W24). Workers run tasks; no isolation. Tasks declare Pydantic/Zod input/output types; two composition styles - declarative DAGs (parents' outputs flow to children) and "durable tasks" which "can only perform two operations - waiting (sleep or event) or spawning child tasks" ([hatchet docs](https://docs.hatchet.run/v1/durable-execution)). DETERMINISM CONTRACT: yes for durable tasks, enforced by that restriction - anything non-deterministic must be a child task. HITL = `wait_for_events` with or-groups. 7,976 stars (2026-09-21); $500K YC/Ritual Apr-2024 ([tracxn](https://tracxn.com/d/companies/hatchet/__thEC2vgVqEW9sgNbVKjHGV7HhkWyttOLklAx_0x4hVU/funding-and-investors)). Integrate: the "an orchestrator may only wait or spawn" restriction is a crisp way to make a process definition deterministic. Critique: small; the DAG/durable split forces a choice up front.

**7. Windmill.** Open-source scripts-to-workflows platform (Windmill Labs, AGPL). Jobs run under nsjail with PID-namespace isolation, but the script gets full network and workspace resources/secrets it asks for ([github](https://github.com/windmill-labs/windmill)). Scripts are typed functions (TS/Py/Go/Bash/SQL) whose signatures become JSON-schema forms; flows compose them (OpenFlow JSON/YAML, UI-editable) with branches, loops, subflows and error handlers. HITL = approval step: the flow suspends and exposes resume/cancel URLs, the approval form's fields land in `resume["field"]` for later steps ([docs](https://www.windmill.dev/docs/flows/flow_approval)). No replay contract - per-step results are stored; a failed step reruns. 17,991 stars (2026-09-21); $15M total, $5M Series A Oct-2024; $2.7M revenue 2024 ([tracxn](https://tracxn.com/d/companies/windmill/__iOVlgupAIVQ1iVW3XJ0XfP43tu4CkVf7d4HV4FQR3mM/funding-and-investors)). Integrate: typed signature -> generated form -> the human's answer becomes a typed step output. Critique: approval is a UX feature, not a gate; a known bug bypassed approval steps inside branches (issue #2639).

**8. Kestra.** Declarative YAML orchestration (Kestra Technologies, Paris); 1.0 LTS Sep-2025, 2.0 with a new engine 2026 ([kestra.io](https://kestra.io/1-0)). Tasks execute via task runners (process, Docker, Kubernetes) - a real per-task container option, but the whole task image, not declared inputs. Flows have typed `inputs`/`outputs`, `Subflow` tasks, listeners/triggers, and every save is a numbered revision. Agentic: AI Agent task with tools/MCP, "human-in-the-loop lets you review and approve AI-generated actions" (Pause task). No replay contract. 28,195 stars (2026-09-21); $25M Series A led by RTP Global Mar-2026, $36M total ([prnewswire](https://www.prnewswire.com/news-releases/kestra-raises-25-million-series-a-to-become-the-orchestration-standard-for-enterprises-302729018.html)). Gained a following recently - yes. Integrate: the flow YAML *is* the versioned artifact and UI edits round-trip to it - same "text is truth" stance as keel. Critique: YAML expressions are untyped strings at the joins.

**9. Dagster.** Asset-oriented Python orchestrator (Dagster Labs). No isolation beyond the code-location process / k8s job. Assets are typed Python functions with I/O managers; "asset checks" are tests bound to an asset and shown red/green - the closest thing in this lens to keel's gate-on-a-step. Components (YAML) and Compass (Slack AI analyst, 2025-10-23) plus an MCP server and `dg api` for agents ([dagster.io](https://dagster.io/blog/dagster-1-13-octopuss-garden)). No HITL primitive; no replay contract. 16,187 stars (2026-09-21); $48.8M total; acquired by Prefect announced 2026-07-13 ([dagster.io](https://dagster.io/blog/prefect-is-acquiring-dagster)). Integrate: asset checks - a check is a declared object attached to the artifact, evaluated after materialisation, and the UI treats a failed check as a failed asset. Critique: batch-data model; agents are consumers of Dagster via MCP, never things it assigns work to.

**10. Prefect.** Python orchestrator (Prefect Technologies). Flow runs execute in a worker-launched process/pod; no materialisation. `@flow`/`@task`, Pydantic-typed parameters, subflows, state-change hooks, deployments versioned. HITL = `pause_flow_run(wait_for_input=MyModel)` / `suspend_flow_run` which auto-generates a typed form ([docs](https://docs.prefect.io/v3/advanced/interactive)); `suspend` kills the process and reruns the flow using cached task results - so a weak replay contract exists via result persistence, not journaling. Pydantic AI runs on it as a durable backend ([pydantic.dev](https://pydantic.dev/docs/ai/capabilities/durable_execution/prefect/)). 23,885 stars (2026-09-21); acquired Dagster Jul-2026 ([prefect.io](https://www.prefect.io/prefect-acquires-dagster)). Integrate: a Pydantic model as the *type* of a human's input. Critique: pausing from inside a task is refused (issue #19435); HITL is flow-level only.

**11. Apache Airflow 3.** ASF scheduler. Workers or KubernetesPodOperator pods; no input materialisation. Python DAGs, XCom untyped, TaskGroups, DAG versioning shipped in 3.x. HITL arrived in 3.1: `HITLOperator`, `ApprovalOperator`, `HITLEntryOperator`, `HITLBranchOperator` with `assigned_users`, `response_timeout`, `defaults`, response via UI "Required Actions" or `PATCH .../hitlDetails`; 3.3 added the `awaiting_input` state that holds no pool slot ([airflow docs](https://airflow.apache.org/docs/apache-airflow/stable/tutorial/hitl.html)). No replay contract. 46,924 stars (2026-09-21). Integrate: `assigned_users` on the approval plus a first-class "awaiting input" state that costs nothing. Critique: the human's answer is an XCom value, not a quoted attestation; anyone in the list can answer.

**12. Argo Workflows.** Kubernetes-native workflow CRDs (CNCF). This is the one engine in the lens that materialises inputs: each step is a pod, `inputs.artifacts` are copied in and `outputs.artifacts` copied out, secrets only if mounted. YAML templates, `templateRef` to shared WorkflowTemplates, DAG and steps nesting, exit handlers and lifecycle hooks. HITL = `suspend` template; resume supplies output parameters (`supplied: {}`) and `--node-field-selector` targets one node ([argo docs](https://argo-workflows.readthedocs.io/en/latest/walk-through/suspending/)). Done = container exit 0. No replay contract (pods are re-run). 16,994 stars (2026-09-21). Integrate: declared artifacts in/out of a pod is exactly keel's CONSTRAINED step shape. Critique: YAML volume, and the resume API has had silent-no-op bugs (issue #12863).

**13. Camunda 8 / BPMN.** Enterprise process engine (Zeebe) from Camunda. Job workers run outside; no isolation. BPMN XML with call activities for nesting, execution/user-task listeners as hooks, and process versions with in-flight instance migration - extended in 8.9 (2026-04-14) to ad-hoc sub-processes ([camunda.com](https://camunda.com/blog/2026/04/camunda-8-9-fastest-path-to-agentic-orchestration/)). Agentic pattern: the AI Agent connector loops an LLM, and every tool it may call is a BPMN activity inside an ad-hoc sub-process - "the LLM selects which tools to use, but the orchestration layer controls whether and how each tool executes" ([docs](https://docs.camunda.io/docs/components/agentic-orchestration/ai-agents/)). HITL = User Task with forms, assignment, and global task listeners. No replay contract (engine state, not code replay). 4,284 stars on the monorepo (2026-09-21). Integrate: the tool set an agent may call is *the process definition*, not a runtime allow-list. Critique: BPMN tooling and licensing weight; 8.x SaaS-first.

**14. Netflix / Orkes Conductor.** JSON-defined workflow orchestrator (conductor-oss; Orkes commercialises). Workers poll for tasks of their type; no isolation. Task definitions carry input/output keys, `SUB_WORKFLOW` nests, definitions are versioned and instances pin to a version. Human task: a versioned JSON user form, assignment policies with SLA escalation chains, trigger policies on state change ([orkes docs](https://orkes.io/content/developer-guides/orchestrating-human-tasks)); native LLM tasks and MCP tools. Lease = `responseTimeoutSeconds` on the polled task. No replay contract. 32,212 stars (2026-09-21); Orkes raised $60M 2026-04-23 ([businesswire](https://www.businesswire.com/news/home/20260423550324/en/Orkes-Raises-$60M-as-Developers-Increasingly-Use-Its-Platform-to-Deploy-AI-Confidently-in-Production)). Integrate: worker-polls-by-task-type with a response timeout is a proven claim protocol. Critique: JSON definitions are stringly typed.

**15. Dapr Workflows + Dapr Agents.** CNCF-graduated runtime; Dapr Agents (Python, contributed by Nvidia) GA 1.0 2026-03-23 ([cncf.io](https://www.cncf.io/announcements/2026/03/23/general-availability-of-dapr-agents-delivers-production-reliability-for-enterprise-ai/)). Sidecar model; no isolation of steps. Workflows in code with activities and child workflows; versioning by renaming. DETERMINISM CONTRACT: workflow code deterministic, "no DateTime.Now, no random, no I/O - anything non-deterministic goes into an activity"; history replayed from the state store. HITL = `wait_for_external_event`. Dapr Agents wraps agent loops as such workflows; Diagrid's Feb-2026 essay is the sharpest critique of checkpoint-only frameworks ([diagrid.io](https://www.diagrid.io/blog/checkpoints-are-not-durable-execution-why-langgraph-crewai-google-adk-and-others-fall-short-for-production-agent-workflows)). 26,104 (dapr) / 748 (dapr-agents) stars (2026-09-21). Integrate: the distinction "checkpoint != durable execution: who detects failure and who resumes". Critique: sidecar operational weight; Agents is small.

**16. LangGraph (+ Platform).** Graph agent runtime from LangChain; 1.0 Oct-2025 ([changelog](https://changelog.langchain.com/announcements/langchain-1-0-now-generally-available)). No isolation. TypedDict/Pydantic state, subgraphs, pre/post model hooks; no definition versioning in OSS (Platform versions "assistants"). NO REPLAY CONTRACT: a checkpoint per superstep; on resume the interrupted *node re-executes from its start*, so side effects before `interrupt()` re-fire unless wrapped in `@task` ([docs](https://docs.langchain.com/oss/python/langgraph/interrupts)); durability modes `exit|async|sync` trade safety for speed ([reference](https://reference.langchain.com/python/langgraph/types/Durability)). HITL = `interrupt(payload)` -> `Command(resume=value)`, the canonical agent HITL API others copied. 42,042 stars (2026-09-21). Integrate: `interrupt` returning the human's value *as the node's expression value*. Critique: Diagrid's point stands - recovery, dedupe and "who calls resume" are the caller's problem.

**17. Mastra.** TypeScript agent framework (YC W25; Gatsby founders); 1.0 Jan-2026; $13M seed ([mastra.ai](https://mastra.ai/blog/seed-round)). No isolation. `createStep({inputSchema, outputSchema})` with Zod/Standard-Schema types, nested workflows as steps, `.then/.branch/.parallel/.foreach`. HITL = `suspend()` inside a step persists a snapshot (LibSQL default) and `resume()` re-enters that step ([docs](https://mastra.ai/docs/workflows/overview)); a nested-workflow resume bug restarted from the outermost step (issue #5650). No replay contract; no versioning. 28,219 stars (2026-09-21). Gained a following recently - yes. Integrate: schema on both sides of every step, validated at the join. Critique: snapshots, not journals; correctness of resume depends on the framework's own bookkeeping.

**18. CrewAI Flows.** Python multi-agent framework (CrewAI Inc.; $18M Oct-2024). No isolation. Flows are classes with `@start/@listen/@router`, Pydantic state, crews nested as steps, `@persist` for state and `@human_feedback` to pause for a human ([docs](https://docs.crewai.com/en/concepts/flows)). No replay contract; task replay keeps only the last execution. 58,837 stars (2026-09-21) - the largest count here. Integrate: nothing structural keel lacks. Critique: persistence saves state without any resume orchestration; concurrent recovery can duplicate work (Diagrid).

**19. Microsoft Agent Framework.** The AutoGen + Semantic Kernel successor; 1.0 GA 2026-04-03, both predecessors to maintenance ([visualstudiomagazine](https://visualstudiomagazine.com/articles/2026/04/06/microsoft-ships-production-ready-agent-framework-1-0-for-net-and-python.aspx)); Agent Harness and Foundry Hosted Agents GA Aug-2026 ([infoq](https://www.infoq.com/news/2026/08/agent-framework-harness-ga/)). No isolation in the library. Workflows are typed executors joined by edges; sub-workflows; checkpoints at superstep boundaries include pending requests, re-emitted on restore ([learn.microsoft.com](https://learn.microsoft.com/en-us/agent-framework/workflows/checkpoints)). HITL = `RequestInfoExecutor` / `RequestPort` ([learn.microsoft.com](https://learn.microsoft.com/en-us/agent-framework/workflows/human-in-the-loop)). No replay contract. 13,635 stars (2026-09-21). Integrate: pending human requests are *part of the checkpoint* so a restart re-asks. Critique: checkpoint-only; hosting is the Azure upsell.

**20. OpenAI Agents SDK.** Python/TS agent SDK. No isolation. Tools typed by Pydantic/Zod; handoffs nest agents; `RunHooks`/`AgentHooks` before/after. v0.8.0 (2026-02-05) added `needs_approval` on tools and `RunState` serialisation: the runner returns interruptions, you persist `state.to_string()`, decide, resume in another process ([docs](https://openai.github.io/openai-agents-js/guides/human-in-the-loop/)). No replay contract in the SDK; durability comes from hosting it on Temporal or DBOS. 29,589 stars (2026-09-21). Integrate: per-tool `needs_approval` predicate (can be a function of the arguments). Critique: the interruption is a value you must store; nothing detects a crash.

**21. Google ADK.** Agent Development Kit (Python/Java/Go/Kotlin). No isolation. Sequential/Parallel/Loop workflow agents compose; `before/after_model_callback`, `before/after_tool_callback` are the hooks; no definition versioning. HITL = `LongRunningFunctionTool` (returns an operation id, the run pauses, client later supplies the result) plus `ResumabilityConfig(is_resumable=True)` to persist the FunctionCall event and resume by invocation id ([adk docs](https://google.github.io/adk-docs/runtime/resume/)); Temporal and Restate plugins exist. Event-sourced sessions but no replay contract. 21,581 stars (2026-09-21). Integrate: a tool that legitimately returns "pending, here is an id" - the shape of a step that a runner daemon will finish later. Critique: resume is manual; tool failures can crash the run (Diagrid).

**22. Pydantic AI (graphs / durable).** Typed Python agent framework (Pydantic Services). No isolation. `pydantic-graph` nodes are dataclasses whose return type *is* the edge set; durable backends Temporal, DBOS, Prefect, Restate, Airflow, Lambda with vendor co-maintenance ([pydantic.dev](https://pydantic.dev/docs/ai/capabilities/durable_execution/overview/)). HITL = a tool raises `CallDeferred` or is marked `requires_approval`; the run *ends* with `DeferredToolRequests` and is resumed with `DeferredToolResults`; validation runs before approval "so rejected arguments never reach an approver". Determinism inherited from the host engine (model calls become activities/steps). 20,079 stars (2026-09-21). Integrate: validate-then-approve ordering, and "the run ends with a typed request" rather than blocking. Critique: durability is delegated entirely.

**23. Vercel Workflow DevKit.** TS durable execution (launched Oct-2025; `vercel/workflow`, 2,428 stars 2026-09-21). `"use workflow"` compiles to an orchestrator route, each `"use step"` to its own isolated function route - step code is genuinely separate from workflow code, though not input-materialised. Hooks: `defineHook<T>()`, `hook.create({token})`, `hook.resume(token, data)`; webhooks are sugar ([vercel docs](https://vercel.com/docs/workflows/concepts)). DETERMINISM CONTRACT: "all inputs and outputs are recorded in an event log ... the system replays execution deterministically"; LLM/AI SDK calls live in steps. Versioning: runs pin to the deployment they started on ("skew protection"), explicit upgrade boundaries otherwise. Gained a following recently - yes, mostly on Vercel. Integrate: pinning + typed hook token. Critique: Vercel-hosted first; young.

### Patterns across this lens

- **Two camps, and the line is exactly the determinism contract.** Journal/replay engines (Temporal, Restate, DBOS, Dapr Workflows, Hatchet durable tasks, Vercel WDK, Inngest by memoisation) require the orchestrator to be deterministic and push *every* LLM call into a recorded step; checkpoint engines (LangGraph, Mastra, CrewAI, MS Agent Framework, OpenAI SDK, ADK, Trigger.dev) snapshot state and re-run the interrupted unit. The replay camp is where the money went in 2026 (Temporal $850M in seven months; Kestra, Orkes, Trigger.dev, Inngest all raised) and the agent SDKs have all grown a "run me on Temporal/DBOS/Restate" adapter rather than building their own contract. Keel's "state is computed" stance belongs to the replay camp; the lesson is that non-deterministic work needs a *recorded result with an identity*, which is what keel's run token + harvested outputs already are.
- **Converged HITL shape: pause -> typed request -> external resume by token.** `interrupt/Command`, waitpoint token, awakeable, `waitForEvent`, `suspend` template, `RequestInfoExecutor`, `needs_approval`+`RunState`, `pause_flow_run(wait_for_input=Model)`, Airflow `awaiting_input`. Prefect, Windmill and Mastra type the human's answer with a schema; Airflow names who may answer. Nobody stores the human's words verbatim as an attestation - keel's verbatim quote is unique here and defensible.
- **Nobody makes "done" a test.** In all 23 tools a step is done when its function returns (or a pod exits 0). The only gate-like objects are Dagster asset checks (bound to the artifact, evaluated after, shown red) and Pydantic AI's validate-before-approve. Drift detection ("a write with no run behind it") does not exist anywhere; the nearest cousins are Restate's fail-loud journal mismatch and Temporal's non-determinism error, which detect *code* drift against history, not *artifact* drift.
- **Isolation is not this lens's job - except Argo.** Only Argo (pod per step, artifacts in/out declared) and Kestra task runners give a per-step container; Windmill gives nsjail; Trigger.dev containers run the whole bundle with live env; Temporal reaches sandboxes only through activity wrappers over Modal/E2B/Daytona. Keel's CONSTRAINED step (materialised declared inputs, outputs-as-tools, harvest after exit) is not something any engine here offers; the Argo `inputs.artifacts`/`outputs.artifacts` schema is the vocabulary to borrow.
- **Copy: the claim protocol and versioning-by-pin.** Temporal/Conductor/Hatchet/Camunda all use "worker polls a typed queue, gets a task with a token and a timeout, heartbeats or loses it" - a mature lease design keel's runner daemons should mirror. Restate/Vercel/DBOS/Camunda all pin an in-flight run to the definition version it started on and offer an explicit migration/patch boundary; keel's frozen meta-process needs the same pinning or a process edit will silently re-type running steps.
- **Adversarial: this lens argues that a server that never executes is fighting the tide.** Every engine that grew in 2025-26 is a runtime the agent code *calls into*, not a tracker that *assigns* to agents; the agent frameworks (CrewAI 58.8k, LangGraph 42k, OpenAI SDK 29.6k) own the loop and treat orchestration as a plug-in. Declared-I/O typed steps have been available for years (Argo, Conductor, Kestra) and lost developer mindshare to "just write async code" (Temporal, Inngest, Vercel: "the best workflow engine is a programming language"). Camunda's ad-hoc sub-process is the one design that lets the LLM choose *within* a declared tool set - keel's closed CONSTRAINED step may be too rigid for agent work whose next step is not known until the previous one returns. And nobody has managed to make a determinism contract ergonomic: Temporal needed patch markers, worker versioning and a lint pack; if keel's gates rely on replaying an agent, expect the same cost.

### Sources

https://temporal.io/blog/announcing-openai-agents-sdk-integration
https://temporal.io/blog/introducing-temporal-and-agentic-sandboxes-openai-agents-sdk
https://docs.temporal.io/develop/python/workflows/versioning
https://docs.temporal.io/develop/safe-deployments
https://www.businesswire.com/news/home/20260217453156/en/Temporal-Raises-$300M-Series-D-to-Make-Agentic-AI-Real-for-Companies
https://finance.yahoo.com/technology/articles/temporal-reaches-12-5bn-valuation-092853078.html
https://api.github.com/repos/temporalio/temporal
https://www.restate.dev/blog/dealing-with-versioning-in-long-running-agents
https://docs.restate.dev/ai/patterns/durable-agents
https://pydantic.dev/articles/restate-durable-execution-pydanticai
https://techcrunch.com/2024/06/12/restate-raises-7m-for-its-lightweight-workflows-as-code-platform/
https://api.github.com/repos/restatedev/restate
https://www.inngest.com/docs/learn/durable-agents
https://agentkit.inngest.com/advanced-patterns/human-in-the-loop
https://www.inngest.com/blog/announcing-inngest-series-a
https://api.github.com/repos/inngest/inngest
https://trigger.dev/launchweek/2/trigger-v4-ga
https://trigger.dev/blog/v4-beta-launch
https://trigger.dev/blog/series-a
https://api.github.com/repos/triggerdotdev/trigger.dev
https://www.dbos.dev/blog/dbos-new-features-march-2026
https://pydantic.dev/docs/ai/capabilities/durable_execution/dbos/
https://www.builtinboston.com/articles/dbos-launches-raises-8m-seed-20240313
https://api.github.com/repos/dbos-inc/dbos-transact-py
https://docs.hatchet.run/v1/durable-execution
https://docs.hatchet.run/v1/child-spawning
https://tracxn.com/d/companies/hatchet/__thEC2vgVqEW9sgNbVKjHGV7HhkWyttOLklAx_0x4hVU/funding-and-investors
https://api.github.com/repos/hatchet-dev/hatchet
https://github.com/windmill-labs/windmill
https://www.windmill.dev/docs/flows/flow_approval
https://github.com/windmill-labs/windmill/issues/2639
https://tracxn.com/d/companies/windmill/__iOVlgupAIVQ1iVW3XJ0XfP43tu4CkVf7d4HV4FQR3mM/funding-and-investors
https://kestra.io/1-0
https://www.prnewswire.com/news-releases/kestra-raises-25-million-series-a-to-become-the-orchestration-standard-for-enterprises-302729018.html
https://api.github.com/repos/kestra-io/kestra
https://dagster.io/blog/dagster-1-13-octopuss-garden
https://dagster.io/blog/prefect-is-acquiring-dagster
https://api.github.com/repos/dagster-io/dagster
https://docs.prefect.io/v3/advanced/interactive
https://github.com/PrefectHQ/prefect/issues/19435
https://www.prefect.io/prefect-acquires-dagster
https://www.businesswire.com/news/home/20260713065285/en/Prefect-Acquires-Dagster-Uniting-the-Two-Leading-Modern-Orchestrators
https://pydantic.dev/docs/ai/capabilities/durable_execution/prefect/
https://api.github.com/repos/PrefectHQ/prefect
https://airflow.apache.org/docs/apache-airflow/stable/tutorial/hitl.html
https://airflow.apache.org/blog/airflow-3.1.0/
https://api.github.com/repos/apache/airflow
https://argo-workflows.readthedocs.io/en/latest/walk-through/suspending/
https://github.com/argoproj/argo-workflows/blob/main/examples/suspend-template-outputs.yaml
https://github.com/argoproj/argo-workflows/issues/12863
https://api.github.com/repos/argoproj/argo-workflows
https://camunda.com/blog/2026/04/camunda-8-9-fastest-path-to-agentic-orchestration/
https://docs.camunda.io/docs/components/agentic-orchestration/ai-agents/
https://api.github.com/repos/camunda/camunda
https://orkes.io/content/developer-guides/orchestrating-human-tasks
https://orkes.io/content/developer-guides/ai-orchestration
https://www.businesswire.com/news/home/20260423550324/en/Orkes-Raises-$60M-as-Developers-Increasingly-Use-Its-Platform-to-Deploy-AI-Confidently-in-Production
https://api.github.com/repos/conductor-oss/conductor
https://www.cncf.io/announcements/2026/03/23/general-availability-of-dapr-agents-delivers-production-reliability-for-enterprise-ai/
https://docs.dapr.io/developing-applications/building-blocks/workflow/workflow-features-concepts/
https://www.diagrid.io/blog/checkpoints-are-not-durable-execution-why-langgraph-crewai-google-adk-and-others-fall-short-for-production-agent-workflows
https://api.github.com/repos/dapr/dapr
https://api.github.com/repos/dapr/dapr-agents
https://docs.langchain.com/oss/python/langgraph/interrupts
https://reference.langchain.com/python/langgraph/types/Durability
https://changelog.langchain.com/announcements/langchain-1-0-now-generally-available
https://api.github.com/repos/langchain-ai/langgraph
https://mastra.ai/docs/workflows/overview
https://github.com/mastra-ai/mastra/issues/5650
https://mastra.ai/blog/seed-round
https://api.github.com/repos/mastra-ai/mastra
https://docs.crewai.com/en/concepts/flows
https://api.github.com/repos/crewAIInc/crewAI
https://learn.microsoft.com/en-us/agent-framework/workflows/checkpoints
https://learn.microsoft.com/en-us/agent-framework/workflows/human-in-the-loop
https://visualstudiomagazine.com/articles/2026/04/06/microsoft-ships-production-ready-agent-framework-1-0-for-net-and-python.aspx
https://www.infoq.com/news/2026/08/agent-framework-harness-ga/
https://api.github.com/repos/microsoft/agent-framework
https://openai.github.io/openai-agents-js/guides/human-in-the-loop/
https://openai.github.io/openai-agents-python/ref/run_state/
https://api.github.com/repos/openai/openai-agents-python
https://google.github.io/adk-docs/runtime/resume/
https://developers.googleblog.com/build-long-running-ai-agents-that-pause-resume-and-never-lose-context-with-adk/
https://adk.dev/integrations/restate/
https://api.github.com/repos/google/adk-python
https://pydantic.dev/docs/ai/capabilities/durable_execution/overview/
https://pydantic.dev/docs/ai/capabilities/durable_execution/temporal/
https://api.github.com/repos/pydantic/pydantic-ai
https://vercel.com/docs/workflows/concepts
https://vercel.com/blog/the-best-workflow-engine-is-a-programming-language
https://api.github.com/repos/vercel/workflow


---

# Appendix D — lens D report, verbatim: hermetic build systems and CI runners

## Lens D - Hermetic build systems, CI runners and sandboxed pipeline engines

Read date for every number: 2026-09-20 (GitHub counts via api.github.com the same day). Focus: item 2 (how inputs are materialised / outputs harvested), item 5 (runner registration, claim/lease, token scoping), item 4 (attestations, human-approval environments).

### Summary table

| # | Tool | Maker | Isolation | Step definition | Done = | Task owner | Adoption (2026-09-20) | Following recently? |
|---|---|---|---|---|---|---|---|---|
| 1 | Bazel | Google / bazelbuild | linux-sandbox namespaces, symlink/hardlink forest of DECLARED inputs only, net off by default | Starlark rules, typed labels, declared outputs | action exits 0 AND declared outputs exist; cached by digest | tool schedules actions; remote workers pull via REAPI | 25,869 stars; Bazel 9 LTS 2026-01-20 | no (mature, steady) |
| 2 | REAPI (Remote Execution API) + NativeLink/Buildbarn | bazelbuild + TraceMachina et al. | worker-defined; Merkle input root from CAS, `output_paths` only returned | protobuf `Action`/`Command` | `ActionResult` (exit code + output digests) | client submits, scheduler leases workers | shared protocol of Bazel, Buck2, Pants, moon, Reclient | protocol quietly became the standard |
| 3 | Buck2 | Meta | none local; hermetic only via remote execution | Starlark, typed providers | action output artifacts produced | tool schedules | 4,432 stars; releases through 2026-08 | modest |
| 4 | Nix / Guix | NixOS Foundation / GNU | daemon build sandbox (namespaces, no network except fixed-output), store-path inputs only | `.drv` derivations (Nix lang / Scheme) | output hash in store; content-addressed optional | daemon builds; Hydra/remote builders | Nix 17,747 stars; Guix not on GitHub (not found) | steady; 2026-07 critique that sandbox is a hidden input |
| 5 | Pants | Pants Build community (ex-Toolchain) | per-process sandbox dir materialised from digest store; Sandboxer sidecar (2025-06) | Python rules API, typed `Process` inputs/outputs | process exit + captured `output_files`/`output_directories` | tool schedules | 3,829 stars | no |
| 6 | Dagger + container-use | Dagger Inc. (Hykes) | BuildKit containers, DAG of typed objects; `Env` with declared inputs/outputs for LLM | code (Go/Python/TS/...) typed functions | function returns; `check` per `generate` (v0.21, 2026-05) | agent drives Dagger; container-use gives one container+branch per agent | dagger 16,282 stars; container-use 4,046 stars (created 2025-05-23) | YES - agent pivot 2025-2026 |
| 7 | Earthly | Earthly Technologies | BuildKit containers, Earthfile targets | Earthfile DSL | target builds | tool | 12,048 stars; OSS maintenance ended 2025-04-16, cloud off 2025-07-16 | no - DEAD/frozen |
| 8 | GitHub Actions | GitHub/Microsoft | VM or self-hosted; whole checkout; per-job `GITHUB_TOKEN` | YAML workflows, reusable workflows, composite actions | job exit; environment required reviewers; artifact attestations (Sigstore) | service assigns job to runner (JIT single-use runner token) | actions/runner 6,275 stars; pricing revolt 2025-12 | dominant, not "new" |
| 9 | GitLab CI | GitLab Inc. | executor (shell/docker/k8s); whole checkout; job token | YAML, `include`, child pipelines | job exit; `when: manual`; deployment approvals (Premium) | runner long-polls `POST /jobs/request` with runner auth token | n/a (self-hosted product) | no |
| 10 | Buildkite | Buildkite Pty | self-hosted agents (any), hosted agents | YAML steps, dynamic pipelines via `pipeline upload` | step exit; `block` steps for humans | agent polls queue by tags; JAT one-job token 2026-08-31 | agent repo 1,076 stars; JAT GA 2026-08 | steady enterprise |
| 11 | Tekton (+Chains) | CD Foundation | pod per TaskRun, container per step; workspaces (PVC) | CRDs (Task/Pipeline YAML), typed params/results/artifacts | TaskRun succeeded; Chains signs SLSA provenance | controller schedules pods | pipeline 9,065 stars | no |
| 12 | Argo Workflows / Argo CD | Argo Project (CNCF) | pod per step; input/output artifacts via S3 | YAML DAG/steps, WorkflowTemplates; `suspend` | pod exit; `suspend` awaits `argo resume`; CD: PreSync hooks, sync windows | controller schedules | workflows 16,994; cd 24,206 stars; v4.1.4 2026-09-18 | steady |
| 13 | Woodpecker | community fork of Drone | docker/k8s/local backends, whole checkout | YAML | step exit | agents pull from gRPC queue | 7,898 stars; v3.18.1 2026-09-08 | steady niche |
| 14 | Depot / Namespace / Blacksmith | three startups | ephemeral VM per job (EC2 / microVM), GitHub JIT registration | GitHub Actions YAML (drop-in labels) | as GitHub | webhook `workflow_job` -> spawn -> register | Blacksmith $45M B 2026-08-12, 6,000 orgs; Namespace $23M A 2026-03-23; Depot $10M A 2026-03 | YES |
| 15 | Nx / Turborepo | Nrwl / Vercel | none (host shell), declared `inputs`/`outputs` for hashing only | JSON task graph | task exit; outputs cached by hash; Nx Self-Healing CI auto-applies fixes | tool orders tasks; Nx Agents distribute | nx 29,360; turborepo 31,120 stars; Nx 36M npm/month (+63% YoY) | Nx agent pivot 2026 |
| 16 | moon | moonrepo | none (host), CAS-backed outputs, REAPI-compatible remote cache | YAML tasks with inputs/outputs | task exit + outputs hashed | tool orders | 4,110 stars; v2.0 2026-02-18 | modest |

### Per-tool notes

**1. Bazel** (Google). The closest existing implementation of "declared inputs -> sandbox -> declared outputs". For each action Bazel builds an `execroot/` that "contains all input files to the action and serves as the container for any generated outputs"; `processwrapper-sandbox` "builds a sandbox directory consisting of symlinks that point to the original source files", runs the command there, then "moves the known output artifacts out of the sandbox into the execroot and deletes the sandbox"; `linux-sandbox` adds User/Mount/PID/Network/IPC namespaces, makes the rest of the filesystem read-only and can cut the network (https://bazel.build/docs/sandboxing). Undeclared inputs are the stated enemy: without sandboxing "Bazel doesn't know if a tool uses undeclared input files". Done = exit 0 and every declared output present; results are keyed by action digest and cached. Adoption: 25,869 stars (2026-09-20, https://api.github.com/repos/bazelbuild/bazel); Bazel 9 LTS shipped 2026-01-20 removing WORKSPACE entirely (https://blog.bazel.build/2026/01/20/bazel-9.html). Strength to copy: harvesting is "move only the declared output paths out, delete the sandbox" - nothing else survives. Weakness: the sandbox is filesystem-and-network only; it says nothing about who ran it or whether a human approved anything, and hermeticity still leaks through toolchains on the host.

**2. Remote Execution API (REAPI) + NativeLink** (bazelbuild/remote-apis; implementations by TraceMachina, Buildbarn, BuildBuddy, EngFlow). Added because it is the wire-level form of the shape keel is designing. Inputs are a Merkle tree: the `Action` carries an `input_root_digest` into CAS and "the files in the directory tree are available in the correct location on the build machine before the command is executed"; outputs are declared on `Command.output_paths` and "only the listed paths will be returned to the client as output"; the `ActionResult` holds exit code, output file/directory digests, stdout/stderr digests and `ExecutionMetadata` (worker, timings) (https://github.com/bazelbuild/remote-apis/blob/main/build/bazel/remote/execution/v2/remote_execution.proto). `Execute` returns a long-running Operation; `WaitExecution` re-attaches. Task ownership is the scheduler's: clients submit, workers are leased actions (Buildbarn/NativeLink schedulers). NativeLink advertises compatibility with Bazel, Buck2, Pants, Reclient, Soong, Siso (https://github.com/TraceMachina/nativelink). Strength: a content-addressed, protocol-level definition of "a run" (action digest -> result digest) that any worker can serve - keel's run receipt could literally be an `ActionResult`. Weakness: no notion of approval, actor identity, or provenance signing; it is trust-the-scheduler.

**3. Buck2** (Meta). Rust rewrite of Buck, Starlark rules, typed providers, open-sourced 2023. "Buck2 currently does not sandbox local-only build steps; in contrast, Buck2 using Remote Execution is always hermetic by design" (https://www.hermetiq.com/blog/bazel-vs-buck2-vs-pants). Done = declared output artifacts materialised; dependency graph (DICE) drives scheduling. 4,432 stars, pushed 2026-09-21, created 2022-01 (https://api.github.com/repos/facebook/buck2); external adoption exists but "smaller community, less documentation, and fewer production deployments outside Meta" (https://sourcegraph.com/blog/monorepo-build-tools). Strength: hermeticity is achieved by making the remote path the only trusted path - exactly keel's CONSTRAINED/UNCONSTRAINED split (local = proposal, remote = truth). Weakness: local runs are unsandboxed, so drift between local and remote is a live hazard.

**4. Nix and Guix** (NixOS Foundation; GNU). Derivations (`.drv`) name every input as a store path; the daemon builds in a sandbox with no network except for fixed-output derivations, which are allowed network because their output hash is declared up front (https://guix.gnu.org/en/blog/2024/fixed-output-derivation-sandbox-bypass-cve-2024-27297/). Content-addressed outputs are a Nix 2.x experimental path (https://releases.nixos.org/nix/nix-2.31.0/manual/store/derivation/outputs/content-address.html). Done = output path realised and hashed; no approval concept. Nix 17,747 stars (2026-09-20, https://api.github.com/repos/NixOS/nix); Guix hosts on Savannah - star count not found. Critique from 2026-07-30: the sandbox's own contents (`/bin/sh` via `sandbox-paths`, busybox vs nothing in Guix) are not in the `.drv`, so "Two people can evaluate a bit-identical `.drv`, run different actual build steps, and Nix create the same output hash" (https://fzakaria.com/2026/07/30/the-nix-sandbox-is-a-hidden-input). Strength to copy: fixed-output derivations - a step may touch the outside world ONLY if it pre-declares the hash of what it will bring back. Weakness: the sandbox definition itself must be a declared input or reproducibility is a fiction; keel's sandbox image must be part of the step's digest.

**5. Pants** (Pants Build community). Rules engine in Rust + Python; "Pants executes each subprocess in a hermetic subdirectory known as a sandbox... writing all the process's inputs into the sandbox directory, and then executing the process with the sandbox as its working directory"; inputs come from a content-addressable store and outputs are captured back as `output_files`/`output_directories` digests (https://www.pantsbuild.org/dev/docs/writing-plugins/the-rules-api/processes). The 2025-06-29 Sandboxer post shows how deep this goes: a separate Rust sidecar writes binaries so `pantsd` never holds a write descriptor on something it execs (ETXTBSY) - "the pantsd process itself doesn't write out binaries, but does execute them, and the Sandboxer process does write out binaries but doesn't execute them" (https://www.pantsbuild.org/blog/2025/06/29/introducing-the-sandboxer). 3,829 stars (2026-09-20). Strength: writer/executor separation as a security and correctness pattern - keel's server materialises, the runner executes, never the same process. Weakness: small community; Python-centric.

**6. Dagger + container-use** (Dagger Inc., Solomon Hykes; $20M Series A 2022-03, https://techcrunch.com/2022/03/30/docker-founder-launches-dagger-a-new-devops-platform/; no 2026 round found). Pipelines are typed functions over BuildKit containers. The LLM primitive (v0.18+) adds `Env`: "Environments configure any number of inputs and outputs for the LLM" - `withStringInput`, `withContainerInput`, `withContainerOutput` - and "an LLM can automatically discover and use any available Dagger Functions in the provided environment"; after the loop the caller reads `work.env().output("completed").asContainer()` (https://docs.dagger.io/features/llm/). v0.21.0 (2026-05-22) "automatically exposed a `check` for each `generate` function" (https://github.com/dagger/dagger/releases). container-use (2025-06-14) is an MCP server giving each agent its own container backed by a git branch; humans inspect with git and `cu merge` (https://dagger.io/blog/agent-container-use/). Stars: dagger 16,282; container-use 4,046 since 2025-05-23 (https://api.github.com/repos/dagger/container-use). Gained a following recently: yes - the whole agent line is 2025-2026. Strength: `Env` is the exact "declared inputs, declared outputs, tools = functions on the inputs" contract keel wants for CONSTRAINED steps, already typed and cached. Weakness: the agent drives Dagger (the LLM sits inside a function the developer calls); nothing owns or queues the task, and outputs are only "declared" - nothing verifies the agent's claim beyond a returned object.

**7. Earthly** (Earthly Technologies). Earthfile = Dockerfile x Makefile over BuildKit. On 2025-04-16 the company ended active OSS maintenance ("other than critical bug fixes"), shut Satellites/Cloud on 2025-07-16, pointed users to Dagger and pivoted to "Earthly Lunar" guardrails (https://earthly.dev/blog/shutting-down-earthfiles-cloud/). 12,048 stars, last push 2025-10-23 (https://api.github.com/repos/earthly/earthly). Status: frozen. Lesson: a hermetic-build DSL without an owner-of-work model had no moat once Docker/BuildKit were commodity - keel's value must be the ownership/gating layer, not the sandbox.

**8. GitHub Actions** (GitHub). Whole-repo checkout on a VM or self-hosted runner. Claim model: the service assigns a queued job to a runner; every job "executes on a runner registered with a single-use JIT token" and `GITHUB_TOKEN` is "created at the start of every workflow job, scoped to one repository and expiring when the job ends" (https://www.systemshardening.com/articles/cicd/github-actions-self-hosted-runner/, https://www.warpbuild.com/glossary/github-token). Human gate: environments with up to six required reviewers, self-review prevention, wait timers to 30 days, and custom (GitHub App) deployment protection rules, max 6 per environment (https://docs.github.com/en/actions/reference/workflows-and-actions/deployments-and-environments). Attestations: Sigstore-signed provenance recording "the repository, organization, environment, commit SHA, and triggering event" = SLSA Build L2; reusable workflows give L3 isolation; private repos use GitHub's Sigstore with no transparency log; `gh attestation verify` (https://docs.github.com/en/actions/concepts/security/artifact-attestations). News: a $0.002/min fee on self-hosted runners announced 2025-12-16 was postponed indefinitely within days after backlash (https://github.blog/changelog/2025-12-16-coming-soon-simpler-pricing-and-a-better-experience-for-github-actions/, https://winbuzzer.com/2025/12/18/github-postpones-self-hosted-action-runner-fees-following-community-revolt-xcxwbn/). actions/runner 6,275 stars. Strength: per-job, service-minted, auto-expiring token bound to one job - keel's "run token". Weakness: the sandbox holds the whole repo and whatever secrets the workflow names; drift (a push with no run) is invisible to it.

**9. GitLab CI** (GitLab). Runner is one Go binary with executors (shell, docker, kubernetes, docker-autoscaler); it loops `POST /api/v4/jobs/request` with its runner authentication token and receives a job payload carrying a short-lived job token, masked in logs and valid only while the job runs (https://docs.gitlab.com/runner/, https://docs.gitlab.com/ci/jobs/ci_job_token/). Registration tokens were removed in 18.0 in favour of create-in-UI + authentication token (https://docs.gitlab.com/security/tokens/). Human gate: `when: manual` and, on Premium/Ultimate, deployment approvals on protected environments with multiple approval rules (GA 15.0); notably "after a deployment job is approved, you must run the job manually" (https://docs.gitlab.com/ci/environments/deployment_approvals/). Strength: pull-based claim with a per-job token, plus approvers distinct from the job trigger. Weakness: whole checkout, no provenance signing built in, approval is a paid tier.

**10. Buildkite** (Buildkite). Pipelines are YAML steps (often generated at runtime with `pipeline upload`); agents are self-hosted (or hosted since 2024) and claim work by matching `queue=` and other tags (https://buildkite.com/docs/agent/queues). New 2026-08-31: job acquisition tokens - "the controller keeps its cluster agent token and issues a short-lived job acquisition token (JAT) after reserving work, and the workload uses that JAT to register an agent and acquire only the named job"; 15-minute default lifetime, up to one hour, passed as `BUILDKITE_AGENT_TOKEN` with `BUILDKITE_AGENT_ACQUIRE_JOB` (https://buildkite.com/resources/changelog/401-job-acquisition-tokens-for-ephemeral-agents/). `block` steps hold for a human. buildkite/agent 1,076 stars. Strength: the reserve -> one-job token -> acquire-only-that-job ladder is precisely keel's runner lease protocol, shipped. Weakness: Buildkite executes on whatever the agent's host has; hermeticity is entirely the customer's.

**11. Tekton + Chains** (CD Foundation). Kubernetes CRDs: a TaskRun is a pod, each step a container; typed `params`, `results`, `workspaces` (PVCs), and a newer structured artifacts API where "steps write a structured JSON file to $(step.artifacts.path)" that the controller lifts into `status.artifacts` (https://github.com/tektoncd/chains/pull/1841). Chains: "Once the run is observed as `completed`, Tekton Chains will take a snapshot of the completed TaskRun/PipelineRun", formats in-toto/SLSA v0.2 or v1.0 provenance from type-hinted `*IMAGE_URL`/`*IMAGE_DIGEST` or `*ARTIFACT_OUTPUTS` results, signs and uploads (https://tekton.dev/docs/chains/slsa-provenance/). pipeline 9,065 stars; latest v1.16.0 "Manx WALL-E" on the releases page as read 2026-09-20 (year rendered ambiguously in my fetch). Strength: provenance is produced by an observer that watches the run object, never by the step itself - keel's receipt should be written by the server from what it harvested, not by the agent. Weakness: heavyweight; every step is a pod and the "inputs" are whatever the workspace volume holds.

**12. Argo Workflows / Argo CD** (Argo Project, CNCF). Workflows: DAG/steps YAML, pod per step; "an input artifact is a file downloaded from storage (i.e. S3) and mounted as a volume within the container. An output artifact is a file created in the container that is uploaded to storage" and "the producer must declare an output, the caller must wire that output to an argument, and the consumer must declare a matching input" (https://argo-workflows.readthedocs.io/en/latest/walk-through/artifacts/). Human gate: `suspend: {}` holds the node "until a user or external process resumes it", via `argo resume` or UI, optionally with a timeout default (https://argo-workflows.readthedocs.io/en/latest/walk-through/suspending/). Argo CD gates a sync with PreSync hooks (any failure aborts the sync) and sync windows with `manualSync` (https://argo-cd.readthedocs.io/en/stable/user-guide/sync-waves/, https://argo-cd.readthedocs.io/en/stable/user-guide/sync_windows/). Stars: workflows 16,994, cd 24,206; v4.1.4 on 2026-09-18. Strength: explicit producer/wiring/consumer triple for artifacts - three declarations must agree or the workflow is invalid. Weakness: a resumed `suspend` records who clicked, not what they said; no verbatim-words primitive.

**13. Woodpecker** (community; Drone fork). Server + agents; "the agent polls the server's queue for new work, executes pipeline steps using the pipeline engine, and streams results back" over gRPC with docker, kubernetes or local backends (https://woodpecker-ci.org/docs/next/development/architecture). 7,898 stars; v3.18.1 on 2026-09-08 (https://github.com/woodpecker-ci/woodpecker/releases). Strength: the minimal viable server-owns-queue / agent-pulls-over-gRPC shape in a small Go codebase - a reference for keel's runner daemon. Weakness: whole-repo clone into a container, no attestations, no approval primitive beyond manual pipelines.

**14. Depot / Namespace / Blacksmith** (three runner vendors). All hook GitHub's `workflow_job` webhook, spawn a fresh single-tenant machine, register it as a runner, run one job, destroy it: Depot uses "ephemeral EC2 instances that are never reused" from a standby pool with a 1000 MiB/s cache (https://depot.dev/docs/github-actions/overview); Blacksmith runs microVMs on bare metal. Money: Blacksmith $45M Series B led by Peak XV at a $550M valuation on 2026-08-12, "over 6,000 organizations" (https://www.tamradar.com/funding-rounds/blacksmith-series-b-45m); Namespace $23M seed+A led by NEA, 2026-03-23 (https://namespace.so/blog/series-a); Depot $10M Series A 2026-03 (https://www.cbinsights.com/company/depot/financials). Gained a following recently: yes - driven by AI-generated code volume and GitHub's pricing revolt. Strength: one VM per job, born for the job, killed after - keel's per-step sandbox as a commodity. Weakness: they inherit GitHub's model wholesale (whole checkout, workflow-named secrets); speed, not containment, is the product.

**15. Nx / Turborepo** (Nrwl; Vercel). Task graphs with declared `inputs` and `outputs` used ONLY for hashing and cache restore - "the hash covers the resolved task definition, the package's source files (the inputs key), the lockfile, and the values of every environment variable declared in env"; on a hit "the saved outputs are restored" (https://turborepo.dev/docs/crafting-your-repository/caching). No sandbox: undeclared reads work fine and silently poison the cache. Turborepo remote cache supports HMAC-SHA256 artifact signatures via `TURBO_REMOTE_CACHE_SIGNATURE_KEY` (https://turborepo.dev/docs/core-concepts/remote-caching). Nx's 2026-02-04 roadmap: "Nx analyzes what went wrong, generates a fix, validates it, and applies it if confident", "over 50% of generated fixes are useful", 36M npm downloads/month (+63% YoY) (https://nx.dev/blog/nx-2026-roadmap); Polygraph (2026-06) orchestrates agents across repos (https://thenewstack.io/nx-polygraph-synthetic-monorepo-agents/). Stars: nx 29,360; turborepo 31,120. Strength: signed cache artifacts - harvested outputs carry a MAC so a runner cannot forge a "cached" result. Weakness/critique: declared-inputs-without-enforcement is the failure mode keel must avoid; and Nx Self-Healing lands fixes on confidence, not on a gate a human bound.

**16. moon** (moonrepo). Rust monorepo tool; tasks declare `inputs`/`outputs`; "an artifact is the outputs of a task, as well as the stdout and stderr of the task"; remote cache speaks the Bazel Remote Execution v2 API (bazel-remote, NativeLink), and v2.3 added a local CAS in the same content-addressed format (https://moonrepo.dev/docs/guides/remote-cache, https://moonrepo.dev/blog/moon-v2.3). v2.0 "Phobos" 2026-02-18 with WASM plugin toolchains (https://www.infoq.com/news/2026/05/moonrepo-2-release/). 4,110 stars. Strength: stdout/stderr are part of the artifact - the receipt travels with the outputs. Weakness: no sandbox; hashing only.

### Patterns across this lens

- **Converges: "declared outputs are the only thing that leaves."** Bazel moves known outputs out and deletes the sandbox; REAPI returns only `output_paths`; Argo uploads only declared output artifacts; Pants captures only `output_files`. Nobody harvests "whatever changed". Keel's harvest rule is the industry norm for build systems - and absent from every CI runner (GitHub, GitLab, Buildkite, Woodpecker), which return the whole workspace's side effects.
- **Converges: the one-job credential.** GitHub JIT runner token + per-job `GITHUB_TOKEN`, GitLab job token, Buildkite's 2026-08 reserve->JAT->acquire-only-this-job ladder. Copy Buildkite's three-step protocol verbatim for keel's runner lease: server reserves the step, mints a token bound to (step, runner, expiry), runner registers and may acquire only that step.
- **Nobody does: drift detection.** No tool in this lens can say "this artifact/commit has no run behind it". Provenance (Sigstore, Tekton Chains, SLSA) proves a run happened for an artifact that HAS an attestation; none flags the artifact that lacks one. Keel's drift guard is genuinely novel here; the closest analogue is a policy engine refusing unattested artifacts at admission.
- **Copy: the observer writes the receipt.** Tekton Chains snapshots the completed run object and signs from it; GitHub mints attestations from the OIDC token, not from the job's own claims. Keel's server, not the agent, should author the run receipt from what it harvested plus the lease it issued - and Tekton's "type-hinted results" show how a step names its outputs without holding the signing key.
- **Copy: fixed-output derivations.** Nix's one sanctioned hole in the sandbox is a step that pre-declares the hash of what it will fetch. That is the principled way to let a CONSTRAINED step touch the outside world at all.
- **Against our design (adversarial):** (a) Earthly died with 12k stars because a hermetic-build DSL is commodity - if keel's differentiator is the sandbox, it is undifferentiated; the ownership/gate layer must carry the value. (b) The Nix critique shows the sandbox image is itself an undeclared input; keel must digest the sandbox definition into the step or "constrained" is theatre. (c) Every tool with agents in production (Dagger Env, Nx Self-Healing, container-use) is AGENT-drives-tool, not tool-owns-agent; the market's revealed preference is that the human's IDE agent stays in charge and the pipeline is a callee. Keel inverts this and should expect friction proportional to the inversion. (d) Bazel-style strict declaration is famously expensive to author (rules, toolchains, undeclared-input hunts); "authoring friction is the #1 risk" is the lived experience of every Bazel migration. (e) GitHub's required-reviewer gate records a click, and the industry has been content with that; verbatim-words acceptance has no precedent here and will need its own UI affordance.

### Sources

https://bazel.build/docs/sandboxing
https://blog.bazel.build/2026/01/20/bazel-9.html
https://api.github.com/repos/bazelbuild/bazel
https://github.com/bazelbuild/remote-apis/blob/main/build/bazel/remote/execution/v2/remote_execution.proto
https://github.com/TraceMachina/nativelink
https://github.com/bazelbuild/bazel/issues/23620
https://www.hermetiq.com/blog/bazel-vs-buck2-vs-pants
https://sourcegraph.com/blog/monorepo-build-tools
https://api.github.com/repos/facebook/buck2
https://guix.gnu.org/en/blog/2024/fixed-output-derivation-sandbox-bypass-cve-2024-27297/
https://fzakaria.com/2026/07/30/the-nix-sandbox-is-a-hidden-input
https://releases.nixos.org/nix/nix-2.31.0/manual/store/derivation/outputs/content-address.html
https://api.github.com/repos/NixOS/nix
https://www.pantsbuild.org/blog/2025/06/29/introducing-the-sandboxer
https://www.pantsbuild.org/dev/docs/writing-plugins/the-rules-api/processes
https://api.github.com/repos/pantsbuild/pants
https://docs.dagger.io/features/llm/
https://dagger.io/blog/llm/
https://dagger.io/blog/agent-container-use/
https://github.com/dagger/dagger/releases
https://api.github.com/repos/dagger/dagger
https://api.github.com/repos/dagger/container-use
https://techcrunch.com/2022/03/30/docker-founder-launches-dagger-a-new-devops-platform/
https://earthly.dev/blog/shutting-down-earthfiles-cloud/
https://api.github.com/repos/earthly/earthly
https://docs.github.com/en/actions/concepts/security/artifact-attestations
https://docs.github.com/en/actions/reference/workflows-and-actions/deployments-and-environments
https://www.systemshardening.com/articles/cicd/github-actions-self-hosted-runner/
https://www.warpbuild.com/glossary/github-token
https://github.com/actions/runner/issues/4248
https://github.blog/changelog/2025-12-16-coming-soon-simpler-pricing-and-a-better-experience-for-github-actions/
https://winbuzzer.com/2025/12/18/github-postpones-self-hosted-action-runner-fees-following-community-revolt-xcxwbn/
https://api.github.com/repos/actions/runner
https://docs.gitlab.com/runner/
https://docs.gitlab.com/ci/jobs/ci_job_token/
https://docs.gitlab.com/security/tokens/
https://docs.gitlab.com/ci/environments/deployment_approvals/
https://buildkite.com/resources/changelog/401-job-acquisition-tokens-for-ephemeral-agents/
https://buildkite.com/docs/agent/self-hosted/job-acquisition-tokens
https://buildkite.com/docs/agent/queues
https://api.github.com/repos/buildkite/agent
https://tekton.dev/docs/chains/slsa-provenance/
https://github.com/tektoncd/chains/pull/1841
https://github.com/tektoncd/pipeline/releases
https://api.github.com/repos/tektoncd/pipeline
https://argo-workflows.readthedocs.io/en/latest/walk-through/artifacts/
https://argo-workflows.readthedocs.io/en/latest/walk-through/suspending/
https://argo-cd.readthedocs.io/en/stable/user-guide/sync-waves/
https://argo-cd.readthedocs.io/en/stable/user-guide/sync_windows/
https://github.com/argoproj/argo-workflows/releases
https://api.github.com/repos/argoproj/argo-workflows
https://api.github.com/repos/argoproj/argo-cd
https://woodpecker-ci.org/docs/next/development/architecture
https://github.com/woodpecker-ci/woodpecker/releases
https://api.github.com/repos/woodpecker-ci/woodpecker
https://depot.dev/docs/github-actions/overview
https://www.tamradar.com/funding-rounds/blacksmith-series-b-45m
https://www.blacksmith.sh/blog/actions-pricing
https://namespace.so/blog/series-a
https://www.cbinsights.com/company/depot/financials
https://turborepo.dev/docs/crafting-your-repository/caching
https://turborepo.dev/docs/core-concepts/remote-caching
https://nx.dev/blog/nx-2026-roadmap
https://thenewstack.io/nx-polygraph-synthetic-monorepo-agents/
https://api.github.com/repos/nrwl/nx
https://api.github.com/repos/vercel/turborepo
https://moonrepo.dev/docs/guides/remote-cache
https://moonrepo.dev/blog/moon-v2.3
https://www.infoq.com/news/2026/05/moonrepo-2-release/
https://api.github.com/repos/moonrepo/moon


---

# Appendix E — lens E report, verbatim: verification, spec-driven development and attestation

## Lens E - Verification, spec-driven process, attestation and process-definition tools

Read date for every number below: 2026-09-20 unless a different date is stated. Grouped rows share a paragraph where the tools answer the seven questions the same way. Added beyond the dispatch: Cursor Bugbot (folded into review agents, because it is the one reviewer that became a pre-push gate), Harbor (the harness behind Terminal-Bench 2.0, because it is the cleanest "hidden tests decide" implementation), Axiom Math / Math Inc Gauss (the two funded "agents write proofs" companies beside Harmonic).

### Summary table

| # | Tool | Maker | Isolation | Step definition | Done = | Task owner | Adoption (date) | Following recently? |
|---|---|---|---|---|---|---|---|---|
| 1 | in-toto + witness + Archivista | CNCF in-toto (orig. TestifySec) | none (wraps the command where it runs) | layout/policy: steps, functionaries, expected materials/products, Rego rules | signed link/attestation per step matches layout; inspections run at verify time | pipeline drives; tool records | in-toto 1.0k stars, witness 546 stars (2026-09-20); donated to in-toto 2023 | no (steady, niche) |
| 2 | SLSA + Sigstore (+ GitHub Artifact Attestations, npm provenance) | OpenSSF / Linux Foundation | none (attests the builder) | provenance predicate + VSA; no step DSL | verifier checks signed provenance against expected buildType/params; VSA records verified level or FAILED | n/a | Rekor 1M entries milestone; Rekor v2 GA Oct 2025; cosign ~6k stars (Jun 2026) | yes - adoption jumped after XZ / TanStack attacks |
| 3 | GitHub Spec Kit | GitHub | none (runs inside the host agent) | markdown templates + slash-command sequence: constitution, specify, plan, tasks, implement, converge | "Converged" report; bug flow needs verified/partial/failed verdict | agent drives | 138.1k stars, 12.4k forks (2026-09-20); 90k May 2026 -> 111k Jun 2026 | YES |
| 4 | AWS Kiro | Amazon | none (IDE/CLI in real repo) | specs: requirements (EARS) + design + tasks.md; hooks as JSON in .kiro/hooks, schema "v1" | task checkbox; PreToolUse / PreTaskExecution hooks can BLOCK | agent drives; hooks fire on events | GA 2026-05-07; announced 2025-07-14; credit pricing $20-$200 | yes (GA + Skills adopter) |
| 5 | OpenSpec | Fission-AI | none | openspec/specs + changes/<id>/{proposal,design,tasks.md,specs delta ADDED/MODIFIED/REMOVED}; propose-apply-archive | "all tasks complete" checkboxes; archive merges deltas | agent drives | 69.7k stars (2026-09-20) | YES |
| 6 | BMAD Method | bmad-code-org | none | agent personas + phased workflows (Clarify, Plan, Build and Verify, Learn); v6 module ecosystem | document handoff between agents; code-review rewrite v6.2.1 | agent drives | 53.3k stars, 6.0k forks (2026-09-20); 37k at v6.0.3 | YES |
| 7 | Tessl (Framework + Registry) | Tessl (Guy Podjarny) | none | .spec.md with @generate/@describe, capabilities with [@test] links; registry "tiles" | `tessl build` always runs the linked tests | agent drives | $125M raised; Registry >10k specs; Framework still closed beta mid-2026 | mixed - money yes, product not GA |
| 8 | AGENTS.md + Agent Skills (SKILL.md) | OpenAI (AAIF) / Anthropic (agentskills.io) | n/a | prose instructions; SKILL.md = YAML frontmatter + markdown | none - convention only | n/a | AGENTS.md >60k repos; Skills 32 tools Mar 2026, ~40 Jun 2026; skills.sh 89,753 skills | YES |
| 9 | `claude plugin eval` | Anthropic | fresh headless session per run; OS sandbox when Bash granted; no permission prompts | evals/<case>/{prompt.md,case.yaml,graders/*.md}; 6 grader types, no custom code | case score >= --threshold (default 1.0) else exit 1; 3 runs x 2 arms | tool drives the agent under test | shipped Claude Code v2.1.269, 2026-09-11 | too new |
| 10 | Review agents: CodeRabbit, Greptile, Graphite Diamond, Amp review, Cursor Bugbot | several | none (read PR diff/repo) | none; PR-triggered | advisory comments; Bugbot /review as pre-push gate | PR event drives | CodeRabbit $143M @ $1.5B (Aug 2026), 2M reviews/wk; Greptile $25M A (Sep 2025); Graphite -> Cursor (Dec 2025) -> SpaceX (Aug 2026); Bugbot 80% resolution (May 2026) | YES (category) |
| 11 | Eval platforms: Braintrust, Langfuse, promptfoo, DeepEval | several | none | datasets + scorers (code, LLM judge, human) in code/YAML | score threshold in CI gates merge | n/a | Braintrust $80M B @ $800M (2026-02-17); Langfuse -> ClickHouse (Jan 2026, 20k+ stars); promptfoo -> OpenAI (2026-03-09, 22.4k stars); DeepEval 15k stars (May 2026) | YES (consolidation) |
| 12 | Inspect (UK AISI) | UK AI Security Institute | Docker/K8s/Modal/Proxmox sandbox per sample | Python: Task = dataset + solver + scorer; approval policies | scorer output; human_approver approve/reject/terminate per tool call | tool drives the model | 2.8k stars, 744 forks (2026-09-20); >200 evals; used by Anthropic, DeepMind | modest, institutional |
| 13 | SWE-bench / SWE-bench Pro / Terminal-Bench 2.0 + Harbor | Princeton, Scale AI, Laude Institute | Docker per task; tests hidden until agent exits | task.toml + instruction.md + tests/test.sh + solution/ + environment/Dockerfile | FAIL_TO_PASS tests pass after agent exit; reward file | harness drives the agent | TB2: 89 tasks (36 dev / 53 held-out); SWE-bench Pro 276 private instances; OpenAI dropped SWE-bench Verified | yes (Pro/Harbor 2026) |
| 14 | Process Street, Tallyfy, Pipefy | several | none (SaaS) | UI templates: tasks, form fields, conditional logic, approval/stop tasks | approval or stop task holds the run until sign-off | tool assigns to humans/agents | Pipefy ~$150M raised; Process Street Cora agent; Beam agents submit into approvals | no |
| 15 | Formal-proof agents: Harmonic Aristotle, Axiom Math, Math Inc Gauss | Harmonic, Axiom, Math Inc | Lean 4 kernel checks the proof | English statement or Lean file; no process model | Lean type-checks the proof (deterministic) | user drives | Harmonic $295M total (Jan 2026 C); Axiom $200M @ $1.6B (Mar 2026); Gauss ~200k lines Lean (sphere packing) | YES |

### Per-tool paragraphs

**1. in-toto / witness / Archivista.** in-toto is the CNCF framework for supply-chain step attestation; witness and Archivista were built by TestifySec and donated to in-toto in 2023 (https://github.com/in-toto/community/issues/15). Isolation is none: `witness run` wraps whatever command the pipeline already runs and records materials (input file hashes), products (output hashes), command, environment and pluggable attestor output, then signs it (keyless via Sigstore or SPIFFE) and pushes to Archivista, a graph store for attestations (https://github.com/in-toto/witness). Process definition is the in-toto *layout*: named steps, the functionary keys allowed to perform each, `expected_materials`/`expected_products` rules (`CREATE`, `MODIFY`, `ALLOW`, `REQUIRE`, `MATCH` chaining one step's products to the next step's materials), plus *inspections* - commands the verifier runs at verification time (https://github.com/in-toto/in-toto); witness adds an embedded OPA Rego policy engine over attestation fields. DONE is a verifier's decision: the chain of signed link metadata matches the layout, and the attestation is a receipt of a run (hashes of real files) rather than testimony; drift is detectable exactly as a product hash with no step that produced it. It does not own tasks - the pipeline schedules, in-toto records. Adoption is steady and small: in-toto 1.0k stars, witness 546 stars (read 2026-09-20); layouts are widely acknowledged to have been "left behind since the advent of attestations", with ITE-10/11 policy work still prototype-grade (https://github.com/in-toto/attestation-verifier). Strength keel should copy: the MATCH rule - a step's declared outputs become the next step's *only* admissible inputs, verified by hash. Weakness: layout authoring is notoriously heavy and the policy language has been patched three times.

**2. SLSA + Sigstore (+ GitHub Artifact Attestations, npm provenance).** SLSA (OpenSSF) defines the *provenance* predicate a builder emits and a Verification Summary Attestation that records the verified level or `FAILED` (https://slsa.dev/spec/v1.1/verification_summary); Sigstore (cosign/Fulcio/Rekor, OpenSSF graduated) supplies keyless signing and a public transparency log. No isolation model of its own - it attests *who built what from what*. Verification is deterministic: check the signature, then that `buildType` and `externalParameters` equal the expected values (https://slsa.dev/spec/v1.0/verifying-artifacts). It has no step language and no notion of a human; the human's role is entirely upstream in the build definition. Adoption is the strongest in this lens: Rekor passed 1,000,000 entries (https://blog.sigstore.dev/celebrating-1-000-000-entries-in-rekor-1950b7c150df/), Rekor v2 went GA October 2025, cosign ~6k stars (June 2026, https://rywalker.com/research/sigstore), and it backs npm provenance, PyPI attestations, Homebrew and GitHub Artifact Attestations - adoption "accelerated sharply after the XZ Utils and TanStack attacks" though most npm packages still lack provenance. Strength: the VSA - a small signed *summary* of a verification so downstream consumers do not re-verify; keel's gate result is exactly a VSA. Weakness: provenance says the artifact came from a run, not that the run was *correct*; SLSA explicitly does not cover the semantic gate.

**3. GitHub Spec Kit.** GitHub's open-source toolkit for spec-driven development, run inside Copilot, Claude Code, Cursor, Gemini CLI and 30+ agents (https://github.com/github/spec-kit). No isolation; it is a set of markdown templates and slash commands executed by the host agent in the real repo. Process definition is a fixed command sequence - `constitution`, `specify`, `plan`, `tasks`, `implement`, `converge` - with two sibling flows (bug: assess -> fix -> test; idea: intake -> research -> define -> shape -> decide) and optional `clarify`/`checklist`/`analyze` "quality gates"; templates are project-local files with overrides, no formal versioning. DONE is the agent's own "Converged" report; the bug flow at least insists a fix carries a verdict of verified/partial/failed and states "missing verification is not a successful fix", but the verdict is still written by the agent. Adoption is the steepest curve in this lens: ~90k stars May 2026, 111k June 2026 (https://letsdatascience.com/news/github-open-sources-spec-kit-for-spec-driven-development-1d51a7f7), 138.1k stars / 12.4k forks read 2026-09-20 - gained a following recently. Strength: the *constitution* file - a small set of non-negotiable project rules that every downstream step is checked against; keel's frozen meta-process is the same object and should be renderable as one. Weakness: everything is the agent's testimony; nothing outside the agent decides that a task is done.

**4. AWS Kiro.** Amazon's spec-driven agentic IDE/CLI, announced 2025-07-14 and GA 2026-05-07 (https://www.bitdoze.com/kiro-ai-ide/), Claude on Bedrock, credit pricing from a free tier to $200/month. It runs in the real repo with the user's credentials; no sandbox. A spec is three files - `requirements.md` (EARS-style acceptance criteria), `design.md`, `tasks.md` - and agent hooks are JSON files under `.kiro/hooks` with a `version: "v1"` schema and triggers `PostFileSave/Create/Delete`, `PromptSubmit`, `AgentStop`, `PreToolUse`, `PostToolUse`, `PreTaskExecution`/`PostTaskExecution`; only `PromptSubmit`, `PreToolUse` and `PreTaskExecution` can block (https://kiro.dev/docs/hooks/). DONE is a task checkbox flipped by the agent, though a `PreTaskExecution` hook can refuse to start a task until a command passes, which is the nearest thing here to a bound gate. Kiro owns nothing - the developer drives, hooks react. Strength to copy: hook schema versioning with a migration tool (v0.x IDE and 2.x CLI formats migrate to v1), and the explicit split between hooks that may *block* and hooks that may only *observe*. Weakness: hooks fire on IDE events, not on process steps, so a spec's tasks and the hooks that guard them are not bound to each other.

**5. OpenSpec.** Fission-AI's brownfield-first SDD framework shipped as `@fission-ai/openspec` (https://github.com/Fission-AI/OpenSpec). No isolation. Its distinctive process object is the *change*: `openspec/changes/<id>/` holds `proposal.md`, `design.md`, `tasks.md` and *delta specs* organised under `ADDED / MODIFIED / REMOVED Requirements` with WHEN/THEN scenarios; `/opsx:archive` merges the deltas into `openspec/specs/` and moves the change to `changes/archive/<date>-<id>` - so the spec's history *is* the archive directory. DONE is "all tasks complete" (checkboxes) followed by archive; no test binding, no human primitive beyond the review the human chooses to do. 69.7k stars read 2026-09-20 - gained a following recently. Strength: delta specs - a change is authored as a typed diff against the current spec, and the merge is mechanical; keel's `#Supersede`/`#SupersedeClause` is this idea with edges, and OpenSpec shows the folder-per-change layout humans actually adopt. Weakness: completion is a checkbox, so drift (spec changed, code not, or the reverse) is invisible until someone re-reads.

**6. BMAD Method.** Open-source (MIT) multi-agent "Agile AI-Driven Development" framework, now v6.x, 53.3k stars / 6.0k forks (read 2026-09-20; 37k at v6.0.3, https://www.nitorinfotech.com/blog/what-is-the-bmad-method-a-complete-guide-to-ai-driven-software-development-in-2026/). No isolation - agent personas (analyst, PM, architect, dev, QA...) are invoked as skills in Claude Code/Codex/Cursor. Process is four phases whose artifacts feed the next phase (Clarify -> Plan -> Build and Verify -> Learn and Adjust), reorganised in v6 into a "module ecosystem" with a "BMad Builder" for custom agents/workflows (https://github.com/bmad-code-org/BMAD-METHOD/releases). DONE is a document handoff accepted by the next persona; v6.2.1 rewrote code review "from the ground up" but it remains an LLM persona reviewing an LLM's output. Strength: explicit *role separation with artifact contracts* - the dev agent may only consume a story the scrum-master agent emitted; keel's per-step declared inputs are the enforceable form of this. Weakness: heavy ceremony, and every "verification" is another model's opinion.

**7. Tessl.** Guy Podjarny's spec-driven company, $125M raised (https://www.calcalistech.com/ctechnews/article/i7ucn8teu). Two products: the Registry (GA, >10k library specs/skills as installable "tiles") and the Framework (closed beta for ~nine months as of mid-2026, https://codemyspec.com/blog/tessl-review). A `.spec.md` carries `@generate` (Tessl writes the code) or `@describe` (spec documents existing code), a capabilities list with `[@test]` links to test files, and the API; `tessl build` "will always run tests as part of the process" (https://docs.tessl.io/introduction-to-tessl/quick-start-guide-tessl-framework). So DONE is closer to keel's than any other SDD tool - the linked tests decide - but the test is a *reference* the spec points at, not a gate the system binds, and the workflow tile "pauses for a human review checkpoint" as a prompt instruction rather than a recorded sign-off. Strength: versioned, registry-hosted specs (`tessl-labs/spec-driven-development` at 1.0.5 then 2.0.1) - process definitions as published, versioned packages. Weakness: the Framework is not GA; the money is ahead of the product.

**8. AGENTS.md + Agent Skills.** Two conventions, not tools. AGENTS.md (OpenAI, Aug 2025, now under the Linux Foundation's Agentic AI Foundation alongside MCP and goose) is a prose file adopted by >60,000 repositories and read by Codex, Cursor, Devin, Copilot, Gemini CLI and others (https://www.linuxfoundation.org/press/linux-foundation-announces-the-formation-of-the-agentic-ai-foundation). Agent Skills (Anthropic, opened as a standard 2025-12-18 at agentskills.io) is a folder with `SKILL.md` = YAML frontmatter + markdown; 32 tools by March 2026, ~40 by June 2026, Vercel's skills.sh listing 89,753 skills (https://www.paperclipped.de/en/blog/agent-skills-open-standard-interoperability/, https://agentman.ai/blog/agent-skills-ecosystem-report-2026). Neither has isolation, typing, nesting, versioning or any notion of done - they are instructions the agent may or may not follow. Both gained a following recently. Strength: portability - a keel step's *agent-facing instruction* should be emitted as a SKILL.md so any harness can run it. Weakness: they are the purest form of "process as prose", which keel's invariant 1 rejects; a repo can carry a perfect AGENTS.md and nothing checks adherence.

**9. `claude plugin eval`.** Anthropic's plugin test runner, shipped in Claude Code v2.1.269 on 2026-09-11 (https://code.claude.com/docs/en/plugin-evals). Each run is a fresh, non-interactive session with only the plugin loaded; runs "never stop to ask for permission" - ungranted tools are removed, and if `Bash` is granted every command runs under the OS-level sandbox (writes confined to the run workspace, home/config unreadable, network limited to granted domains; native Windows has no backend so the run is *refused* rather than run unconfined). Suites live in `evals/<case>/{prompt.md, case.yaml, graders/*.md}`; six grader types - `regex`, `tool_used`, `tool_order`, `file_exists` (free, computed from transcript and files), `llm`, `baseline` (judge calls) - and explicitly "no custom-code graders". DONE: each case runs 3 times with the plugin and 3 without; a case passes when its with-arm score >= `--threshold` (default 1.0), any failure exits 1, `--max-cost-usd` exits 2 on partial. Fixtures via `context.scaffold_script` (runs outside the sandbox, only with `--scaffold`), MCP servers mocked with `expect:` blocks that abort the run at score 0 when violated. Strength keel should take whole: *refuse to run rather than run unconfined*, and the mock `expect:` block - a declared contract on what the agent may ask a tool, enforced by aborting. Weakness: graders can only see transcript, reply and files, and the doc itself warns small-judge `llm` graders are noisy.

**10. Code-review agents (CodeRabbit, Greptile, Graphite Diamond, Amp review, Cursor Bugbot).** All are PR-event-triggered reviewers reading the diff plus repository context; none isolates anything. CodeRabbit raised $143M at $1.5B (Series C, Aug 2026), runs >2M reviews/week, ~$50M ARR (https://www.businesswire.com/news/home/20260812311754/en/CodeRabbit-Raises-$143-Million-at-$1.5-Billion-Valuation-and-Introduces-Agentic-Change-Management, https://sacra.com/c/coderabbit/). Greptile builds a whole-repo call graph and its v4 agent (2026-03-05) does multi-hop investigation; $25M Series A from Benchmark Sept 2025, per-review pricing since May 2026 (https://www.agent-wars.com/news/2026-05-01-greptile-per-review-pricing). Graphite's Diamond (Mar 2025) went to Cursor (announced Dec 2025), and Cursor to SpaceX (Aug 2026) (https://cursor.com/blog/graphite). Amp added a review agent in its VS Code extension (https://tessl.io/blog/amp-adds-agentic-code-review-to-its-coding-agent-toolkit/). Cursor Bugbot is the one that became a *gate*: a `/review` command runs before push, 80% of flagged bugs resolved by merge, ~90-second reviews, usage billing ~$1-1.50/PR (https://cursor.com/docs/bugbot). DONE for all of them is advisory - a comment - unless the team wires a required check. Category gained a following recently. Strength: Bugbot's dedupe - an identical diff is never re-reviewed or re-charged, i.e. a receipt keyed on content (keel's D0474 already does this for tests). Weakness: an LLM reviewing an LLM is testimony about testimony; none produces a signed receipt of what was checked.

**11. Eval platforms (Braintrust, Langfuse, promptfoo, DeepEval).** Datasets + scorers (deterministic code, LLM judge, human review) run offline in CI and online in production; a score threshold gates the merge. Braintrust: $80M Series B led by ICONIQ, 2026-02-17, ~$800M valuation; "CI evaluation gates the merge" is its pitch (https://www.braintrust.dev/articles/how-to-eval). Langfuse: acquired by ClickHouse Jan 2026 alongside a $400M Series D, 20k+ stars, 26M+ SDK installs/month, 2,000+ paying customers (https://clickhouse.com/blog/clickhouse-acquires-langfuse-open-source-llm-observability). promptfoo: acquired by OpenAI 2026-03-09 (~$86M), 22.4k stars, MIT kept, red-team plugins folded into OpenAI Frontier (https://www.techcrunch.com/2026/03/09/openai-acquires-promptfoo-to-secure-its-ai-agents/). DeepEval: pytest-style LLM assertions, 15k stars May 2026 (https://deepeval.com/blog/deepeval-got-a-new-look). None owns tasks or isolates; scorers are code/YAML, unversioned beyond git. The category consolidated in 2026 (three exits in three months) - gained a following recently. Strength: Braintrust's *same scorer offline and online* - keel's gate test should be runnable both as a step gate and as a monitor over landed outputs. Weakness: score thresholds hide the individual failure; a 0.9 pass is not a receipt of anything specific.

**12. Inspect (UK AISI).** Open-source Python eval framework used for nearly all UK AISI automated evals and by Anthropic and DeepMind; 2.8k stars / 744 forks (read 2026-09-20), >200 packaged evals (https://github.com/UKGovernmentBEIS/inspect_ai). Each sample can run in a sandbox - Docker built in, Kubernetes/Modal/Proxmox/Vagrant via extension - so untrusted model code never touches the host. A `Task` = dataset + solver (the agent strategy) + scorer (exact match, model-graded, custom function); tasks are Python, composable, no nesting DSL. Its human primitive is the most precise in this lens: *approval policies* at eval or task level - all tool calls, selected tool calls, or custom approvers that approve / reject / escalate - with `human_approver()` presenting approve / reject / terminate, and a 2026 `human_reviewer()` that lets an operator review a tool *result* and stop the sample (https://inspect.aisi.org.uk/approval.html, https://github.com/UKGovernmentBEIS/inspect_ai/pull/5385). DONE is the scorer; every event is written to a structured eval log that the viewer replays. Strength: approval as a *policy over tool calls* rather than a checkbox on a task - keel's human-in-loop step should name which output types need a human, not just "review". Weakness: it evaluates models, so it has no notion of a persistent work item, lease or drift.

**13. SWE-bench / SWE-bench Pro / Terminal-Bench 2.0 + Harbor.** The harness style where hidden tests decide. SWE-bench resolves an instance when `FAIL_TO_PASS` tests that failed on the original code pass after the agent's patch; Scale's SWE-bench Pro (Sept 2025) adds 276 instances from 18 private codebases and flags overfitting when public and held-out scores diverge >10 points (https://scale.com/blog/swe-bench-pro); OpenAI publicly stopped reporting SWE-bench Verified (https://openai.com/index/why-we-no-longer-evaluate-swe-bench-verified/) and a "SWE-Bench Pro Verified" paper appeared Sept 2026 (https://arxiv.org/html/2609.08149). Terminal-Bench 2.0 (89 tasks, 36 dev / 53 held-out, Docker per task) ships with Harbor, whose task format is the cleanest step contract in this lens: a directory is a task iff it has `task.toml`; `instruction.md` is the agent-facing spec, `tests/test.sh` is the verifier the agent never sees, `solution/solve.sh` is the oracle that must pass the tests, `environment/Dockerfile` is the materialised world (https://www.tbench.ai/news/announcement-2-0, https://pypi.org/project/harbor/0.1.0/). The harness owns the task: it builds the container, installs the agent, runs it, then runs the tests and writes a reward file. Strength keel should copy literally: *the oracle solution* - every gate ships with a known-positive that must pass and the original state as known-negative (keel's D0388 probe pair, generalised to every step). Weakness: single-shot, no human, no persistence; the environment is authored by hand per task.

**14. Process Street, Tallyfy, Pipefy.** SaaS process-definition tools for human work: UI-authored templates of steps with form fields (typed inputs), assignees, deadlines, conditional logic, and approval/stop tasks that "hold a run until sign-off" (https://www.process.st/agentic-ai-workflow/, https://tallyfy.com/products/pro/tracking-and-tasks/tasks/what-types-of-tasks-can-i-create-with-tallyfy/). Tallyfy's five task types (standard, approve/reject, expiring, email-draft, email-auto-send) are the closest thing anywhere to a typed step catalogue; Pipefy's phases with card movement and "AI Agents 2.0" (five behaviours, up to three actions each - fill fields, create cards, move items) put agents *inside* a human-designed pipe (https://www.processexcellencenetwork.com/ai/news/pipefy-launches-ai-agents-20-intelligent-document-processing-idp); Process Street's Cora and Beam integration explicitly forbid the agent from approving on a person's behalf. No isolation, no drift detection, and template versioning is a saved-copy model. Pipefy has raised ~$150M (Founders Fund, SoftBank); Process Street and Tallyfy numbers not found. Not gaining a following in the AI sense. Strength: the tool *assigns* work - a run reaches a step and the step's assignee is notified; agents are just another assignee type. Weakness: "done" is a person clicking Complete; the form field is the only receipt.

**15. Formal-proof agents (Harmonic Aristotle, Axiom Math, Math Inc Gauss).** Agents that write Lean 4 proofs the Lean kernel checks, so DONE is deterministic and independent of the agent. Harmonic's Aristotle "proves software correct": give it English or let it edit inside a Lean project, output backed by a machine-checked proof; 96.8% on the Verifiable Code Generation Arena; $295M raised over three rounds including a Jan 2026 Series C with Nvidia's NVentures; $300k to the Lean FRO and a $1M grant program (https://aristotle.harmonic.fun/, https://siliconangle.com/2026/01/15/nvidias-nventures-backs-harmonic-ai-series-c-mathematical-superintelligence/). Axiom Math raised $200M at $1.6B in March 2026 and verified the "246 theorem" (https://spectrum.ieee.org/axiom-math-246-theorem-formalization); Math Inc's Gauss produced ~200,000 lines of Lean for the sphere-packing formalisation. Academic side: Vero benchmarks agents building verified repositories (Dafny/Verus/Coq, 43 instances, https://arxiv.org/abs/2608.13522), and "Intent Formalization" is named a grand challenge (https://arxiv.org/pdf/2603.17150). Gained a following recently. Strength: the verifier is a *kernel*, not a judge - the same posture keel takes with gates. Weakness: the spec the proof discharges is still authored by a person or an agent; a proof of the wrong theorem is a green gate on the wrong requirement.

### Patterns across this lens

- **Convergence: everyone has discovered "outside the agent decides", nobody has bound it to a process model.** Hidden-test harnesses (Harbor, SWE-bench), Lean kernels, `claude plugin eval` thresholds and Braintrust CI gates all make DONE a computed verdict; but the SDD tools that define *processes* (Spec Kit, Kiro, OpenSpec, BMAD) all end in an agent-flipped checkbox. Keel's "gate = tests bound to steps" sits in the empty quadrant, and Harbor's `task.toml + tests/ + solution/` is the format to steal for it.
- **Receipt vs testimony is already a live distinction in supply chain, absent in AI process tools.** in-toto links and SLSA provenance are hash-of-real-files receipts; every SDD tool's "verified" is prose. Sigstore's VSA is the shape of keel's gate result, and in-toto's MATCH rule (step N's products are step N+1's only admissible materials) is the drift guard keel describes, already specified and signed.
- **Human sign-off primitives exist in three grades.** Process Street/Tallyfy: a stop task a person clicks. Inspect: a *policy* naming which tool calls need approve/reject/escalate, with the human's choice logged. Nobody records the human's words verbatim; keel's `--words` is unique in this lens. Inspect's policy-over-actions should be copied so keel's human step declares *which output kinds* require a human rather than one blanket review.
- **Process versioning is filesystem-shaped everywhere.** OpenSpec's `changes/archive/<date>-<id>`, Kiro's `version: "v1"` hook schema with migrators, Tessl's registry tiles at 1.0.5 -> 2.0.1. Nobody nests processes formally; BMAD's phases and Spec Kit's three flows are flat sequences. Keel's nesting is unopposed but also unproven - there is no prior art to lean on.
- **Against our design (1): the market is paying for unconstrained agents plus a reviewer, not constrained sandboxes.** CodeRabbit at $1.5B, Bugbot as pre-push gate, Spec Kit at 138k stars - all operate in the real repo with credentials and network, and a second LLM as the gate. If review-as-gate keeps its 80% resolution rate, buyers may not feel the need for materialised inputs and harvested outputs; keel's constrained step must be cheaper to author than a Spec Kit constitution or it will lose to "run it anyway and let Bugbot look".
- **Against our design (2): every tool that made verification deterministic gave up the persistent work item.** Harbor, Inspect, `plugin eval` and Lean are single-shot; the moment a task persists across runs (SWE-bench Pro's public/held-out drift check, Braintrust's online scorers) the tool falls back to statistical thresholds. Keel's claim that a *tracked* item can stay deterministically gated over its life is the unvalidated part, and the eval market's consolidation into observability vendors (Langfuse -> ClickHouse, promptfoo -> OpenAI) suggests the field expects monitoring, not gates, to be what persists.

### Sources

https://github.com/in-toto/witness
https://github.com/in-toto/in-toto
https://github.com/in-toto/community/issues/15
https://github.com/in-toto/attestation-verifier
https://www.cncf.io/blog/2023/08/17/unleashing-in-toto-the-api-of-devsecops/
https://slsa.dev/spec/v1.1/verification_summary
https://slsa.dev/spec/v1.0/verifying-artifacts
https://slsa.dev/spec/v1.0/provenance
https://blog.sigstore.dev/celebrating-1-000-000-entries-in-rekor-1950b7c150df/
https://blog.sigstore.dev/rekor-v2-ga/
https://rywalker.com/research/sigstore
https://github.com/actions/attest-build-provenance
https://github.com/github/spec-kit
https://letsdatascience.com/news/github-open-sources-spec-kit-for-spec-driven-development-1d51a7f7
https://www.marktechpost.com/2026/05/08/meet-github-spec-kit-an-open-source-toolkit-for-spec-driven-development-with-ai-coding-agents/
https://kiro.dev/docs/hooks/
https://www.bitdoze.com/kiro-ai-ide/
https://aitoolpick.org/blog/kiro-pricing-2026/
https://github.com/Fission-AI/OpenSpec
https://codemyspec.com/blog/openspec-explained
https://github.com/bmad-code-org/BMAD-METHOD
https://github.com/bmad-code-org/BMAD-METHOD/releases
https://www.nitorinfotech.com/blog/what-is-the-bmad-method-a-complete-guide-to-ai-driven-software-development-in-2026/
https://tessl.io/blog/tessl-launches-spec-driven-framework-and-registry
https://docs.tessl.io/introduction-to-tessl/quick-start-guide-tessl-framework
https://tessl.io/registry/tessl-labs/spec-driven-development
https://codemyspec.com/blog/tessl-review
https://www.calcalistech.com/ctechnews/article/i7ucn8teu
https://martinfowler.com/articles/exploring-gen-ai/sdd-3-tools.html
https://www.linuxfoundation.org/press/linux-foundation-announces-the-formation-of-the-agentic-ai-foundation
https://openai.com/index/agentic-ai-foundation/
https://www.paperclipped.de/en/blog/agent-skills-open-standard-interoperability/
https://agentman.ai/blog/agent-skills-ecosystem-report-2026
https://github.com/anthropics/skills
https://code.claude.com/docs/en/plugin-evals
https://daily.dev/posts/claude-code-adds-plugin-eval-command-to-test-and-benchmark-plugins-hgkdory2v
https://www.businesswire.com/news/home/20260812311754/en/CodeRabbit-Raises-$143-Million-at-$1.5-Billion-Valuation-and-Introduces-Agentic-Change-Management
https://sacra.com/c/coderabbit/
https://techcrunch.com/2025/07/18/benchmark-in-talks-to-lead-series-a-for-greptile-valuing-ai-code-reviewer-at-180m-sources-say
https://www.agent-wars.com/news/2026-05-01-greptile-per-review-pricing
https://dev.to/heraldofsolace/the-best-ai-code-review-tools-of-2026-2mb3
https://cursor.com/blog/graphite
https://finance.yahoo.com/news/exclusive-cursor-acquires-code-review-153008616.html
https://tessl.io/blog/amp-adds-agentic-code-review-to-its-coding-agent-toolkit/
https://sourcegraph.com/amp
https://cursor.com/docs/bugbot
https://www.digitalapplied.com/blog/cursor-bugbot-90-second-reviews-june-2026-release
https://www.braintrust.dev/articles/how-to-eval
https://www.voiceflow.com/blog/what-is-braintrust
https://clickhouse.com/blog/clickhouse-acquires-langfuse-open-source-llm-observability
https://langfuse.com/blog/joining-clickhouse
https://www.techcrunch.com/2026/03/09/openai-acquires-promptfoo-to-secure-its-ai-agents/
https://www.paperclipped.de/en/blog/promptfoo-ai-agent-red-teaming/
https://deepeval.com/blog/deepeval-got-a-new-look
https://www.confident-ai.com/docs/llm-evaluation/unit-testing-cicd
https://github.com/UKGovernmentBEIS/inspect_ai
https://inspect.aisi.org.uk/approval.html
https://github.com/UKGovernmentBEIS/inspect_ai/pull/5385
https://benchmarkingagents.com/inspect-uk-aisi/
https://www.tbench.ai/news/announcement-2-0
https://pypi.org/project/harbor/0.1.0/
https://arxiv.org/pdf/2601.11868
https://scale.com/blog/swe-bench-pro
https://labs.scale.com/leaderboard/swe_bench_pro_private
https://openai.com/index/why-we-no-longer-evaluate-swe-bench-verified/
https://arxiv.org/html/2609.08149
https://www.process.st/agentic-ai-workflow/
https://beam.ai/integrations/process-street
https://hackceleration.com/labs/review/process-street
https://tallyfy.com/products/pro/tracking-and-tasks/tasks/what-types-of-tasks-can-i-create-with-tallyfy/
https://tallyfy.com/approval-process-workflow/
https://www.processexcellencenetwork.com/ai/news/pipefy-launches-ai-agents-20-intelligent-document-processing-idp
https://tallyfy.com/pipefy-review/
https://aristotle.harmonic.fun/
https://siliconangle.com/2026/01/15/nvidias-nventures-backs-harmonic-ai-series-c-mathematical-superintelligence/
https://sacra.com/c/harmonic/
https://spectrum.ieee.org/axiom-math-246-theorem-formalization
https://www.buildmvpfast.com/blog/axiom-formal-verification-ai-hallucination-math-proof-2026
https://menlovc.com/perspective/ai-will-write-all-the-code-mathematics-will-prove-it-works/
https://arxiv.org/abs/2608.13522
https://arxiv.org/pdf/2603.17150
