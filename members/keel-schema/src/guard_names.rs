//! The enforced guard-name list - the control inventory as an authored fact (D0479, sprint 732).
//!
//! The list lived beside the guards in `keel-cli/src/guards.rs`; the view layer's proof census
//! (`control_proof`) read it from there, which was the one upward reference keeping view above
//! guards after sprint 717. A name here is a fact the binary states (`keel version`, `--help`,
//! `keel show controls`), so it descends to the facts member like the CLI surface did. Removing a
//! name silently disarms a guard, so this file is on the enforcement-surface lock
//! (`GUARD_SOURCE_FILES`) exactly as guards.rs is: the edit needs a co-committed marked Decision.
//! `guards.rs` re-exports it under its old path; `every_enforced_guard_dispatches` still holds each
//! name to a `run_one` arm.

/// The ENFORCED forward guards, in CLI/runner order.
///
/// `issues` joined the enforced set at IRL-d (D0077). HONEST-STATE doctrine (D0098): the enforced set
/// holds only INTEGRITY guards — the recorded model must not lie, be malformed, or be untraceable.
/// COMPLETENESS / self-assurance (`assured` composite readiness + `critique`-COVERAGE) was DEMOTED
/// from this set: it is computed as a NON-BLOCKING burndown (`keel gate guard assured` / `keel critique-
/// coverage` stay runnable, surfaced in orient), never a hard commit gate — incomplete implementation
/// flagged AS incomplete is honest state, not a failure. NOTE: critique INDEPENDENCE stays enforced
/// (critic-independence — honesty); only critique COVERAGE demoted. The requirement-rootedness hard
/// guard (D0098 honesty: a chartered capability with no driving Need) joins next (requirementRootednessGuard).
pub const GUARD_NAMES: [&str; 77] =
    ["evidence-cited", "gating-workflow-history", "process-applicability", "doc-guard-count", "actors", "acceptance-events", "sprint-coverage", "ceremony", "charter", "process-change", "issues", "viewpoint-renderer", "manifest-coverage", "critic-independence", "process-skill", "requirement-rootedness", "decision-rationale", "attestation-substance", "marker-vocabulary", "duplicate-identity", "decision-requirement-link", "verification-trace", "priority-inversion", "retro-backlog", "confirmation-authenticity", "engine-lint", "doc-sync", "hook-config-integrity", "activation-manifest", "sequence-multiplicity", "parser-coverage", "base-first-justification", "edge-endpoints", "ownership", "attestation-authority", "type-collision", "attribute-vocabulary", "resolver-kind", "stale-gate-prose", "impossible-evidence-date", "identity-present", "identity-well-formed", "tool-reference", "scaffold-placeholder", "claude-surface-drift", "decision-scaffolding", "release-recorded", "enrollment-binding", "control-event-coverage", "question-coverage", "claim-ancestry", "judgment-request-quality", "manifest-key-portability", "control-map-reconciled", "sprint-closure", "untrusted-routing", "control-defect-registry", "cli-surface-declared", "decision-amends-process", "unit-extras-present", "acceptance-binds-to-text", "stpa-currency", "untrusted-taint", "gate-environment-parity", "instruments-declared", "release-checksums-published", "wrapper-pin-checksummed", "plan-covers-step", "id-is-a-uuid", "step-check-resolves", "consent-scope", "working-tree-eol", "direction-cited", "cli-reference", "custom-harness-routed", "source-reference", "sitting-review-method"];
