//! Guard family `critique` - split from `keel-cli/src/guards.rs` by `scripts/split_guards.py` (sprint 733).
//!
//! Each guard's dispatch arm, code and tests sit together; the shared scanners, the runner and the
//! lock predicates are the crate root's (`super`). Nothing here was retyped: the text is guards.rs's,
//! with `crate::` paths pointing at the members and private items opened to the crate.

use super::*;

/// The `critique` family: every guard it dispatches, in `GUARD_NAMES` order, with the tier note each
/// arm carried in `run_one` (sprint 733). The root's union test holds these tables equal to `GUARD_NAMES`.
pub(crate) const FAMILY: Family = Family {
    name: "critique",
    arms: &[
        ("critic-independence", critic_independence),
        ("critique", critique),
        ("assured", assured),
        ("critique-rigor", critique_rigor), // runnable-only (not in GUARD_NAMES)
    ],
};

/// Guard: every assurance element carries its required-lens critiques (D0080/D0079).
///
/// An element missing a required-lens critique (per the declared critique policy, D0097 — default
/// Core-3) is reported here. This is critique-COVERAGE — a COMPLETENESS measure, so under the honest-
/// state doctrine (D0098) it is NOT in the enforced `GUARD_NAMES`: it is a non-blocking burndown,
/// RUNNABLE via `keel gate guard critique` / `keel critique-coverage` and surfaced in orient, never a hard
/// commit gate (an un-critiqued element is honest incomplete state, not a lie). Critique INDEPENDENCE
/// (`critic-independence`) stays enforced — that is honesty, not completeness.
#[must_use]
pub fn critique(root: &Path) -> GuardReport {
    match keel_view::view::critique_gaps(root) {
        Ok(gaps) => {
            let violations = gaps
                .into_iter()
                .map(|e| format!("{e}: missing a required-lens critique (D0080/D0097 policy; run the element-critique skill)"))
                .collect();
            GuardReport { name: "critique", scanned: 0, warnings: Vec::new(), violations }
        }
        Err(e) => GuardReport { name: "critique", scanned: 0, warnings: Vec::new(), violations: vec![format!("error reading critique coverage: {e}")] },
    }
}

/// Guard: the composite assurance-readiness gate (D0079 c).
///
/// Reports the exact blockers when the deliverable is not assured (coverage/critique gaps, stale
/// verification, undispositioned >= Medium findings, open Critical, invariant violations). This is the
/// SELF-ASSURANCE composite (completeness/readiness), so under the honest-state doctrine (D0098) it is
/// NOT in the enforced `GUARD_NAMES`: a NON-BLOCKING burndown verdict, RUNNABLE via `keel gate guard
/// assured` / `keel gate assured` and surfaced in orient, never a hard commit gate. Incompleteness flagged
/// AS incomplete is honest state; suppressing it or blocking on it both destroy the honest picture.
#[must_use]
pub fn assured(root: &Path) -> GuardReport {
    match assured_blockers(root) {
        Ok(blockers) => GuardReport { name: "assured", scanned: 0, warnings: Vec::new(), violations: blockers },
        Err(e) => GuardReport { name: "assured", scanned: 0, warnings: Vec::new(), violations: vec![format!("error computing readiness: {e}")] },
    }
}

/// Guard (D0080/issue031): a Critical-severity finding's target must carry a non-aiModel critic.
///
/// ENFORCED (vacuous until a Critical finding exists). aiModel-vs-aiModel critique shares blind spots,
/// so the highest-stakes elements require cognition-distinct (human/tool) independence.
#[must_use]
pub fn critic_independence(root: &Path) -> GuardReport {
    match keel_view::view::critical_independence_gaps_scanned(root) {
        Ok((scanned, gaps)) => {
            let violations = gaps
                .into_iter()
                .map(|e| format!("{e}: target of a Critical-severity finding but has only aiModel critiques — requires a human/tool critic (D0080 independence, issue031)"))
                .collect();
            GuardReport { name: "critic-independence", scanned, warnings: Vec::new(), violations }
        }
        Err(e) => GuardReport { name: "critic-independence", scanned: 0, warnings: Vec::new(), violations: vec![format!("error reading critique independence: {e}")] },
    }
}

/// Diagnostic (D0080/issue030): low-rigor critiques + affirming-only critics, as WARNINGS.
///
/// RUNNABLE via `keel gate guard critique-rigor` but NOT in the enforced `GUARD_NAMES` — rigor is a
/// heuristic signal for human attention, not a hard gate (a shallow-but-honest critique is not a
/// commit-blocker). Surfaces critiques lacking adversarial structure / substance and never-find critics.
#[must_use]
pub fn critique_rigor(root: &Path) -> GuardReport {
    match keel_view::view::critique_rigor(root) {
        Ok(findings) => GuardReport { name: "critique-rigor", scanned: findings.len(), warnings: findings, violations: Vec::new() },
        Err(e) => GuardReport { name: "critique-rigor", scanned: 0, warnings: Vec::new(), violations: vec![format!("error reading critique rigor: {e}")] },
    }
}

/// Readiness blocker summaries (the `guard assured` violation set) — empty iff READY.
///
/// # Errors
/// Returns [`ViewError`] if a tracking/instance file fails to parse.
pub fn assured_blockers(root: &Path) -> Result<Vec<String>, ViewError> {
    let b = compute_readiness(root)?;
    let mut out = Vec::new();
    let note = |out: &mut Vec<String>, label: &str, v: &[String]| {
        if !v.is_empty() {
            out.push(format!("{label}: {} ({})", v.len(), v.iter().take(5).cloned().collect::<Vec<_>>().join(", ")));
        }
    };
    // BLOCKING categories only (stale_verifications is advisory — see ReadinessBlockers::ready).
    note(&mut out, "coverage gaps", &b.coverage_gaps);
    note(&mut out, "critique gaps", &b.critique_gaps);
    note(&mut out, "undispositioned >=Medium findings", &b.undispositioned_findings);
    note(&mut out, "unfixed Critical findings", &b.unfixed_critical);
    note(&mut out, "invariant violations", &b.invariant_violations);
    Ok(out)
}
