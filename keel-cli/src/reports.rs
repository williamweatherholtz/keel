//! The scorecards (D0087) and the orient dashboard (D0093): computed aggregates over the views,
//! orient and readiness. Regenerate, never commit as truth.
//!
//! Above view, orient and guards (D0479, dcGuardsViewOrientCycleIsBroken): a report composes a frontier,
//! a suspect walk and a readiness verdict with the view's coverage rows; nothing below reads a report.

use std::collections::HashSet;
use std::path::Path;

use crate::json::Json;
#[allow(clippy::wildcard_imports)] // the report cards are written in the view's vocabulary
use crate::view::*;

/// Compute a report's `(title, cards)`; shared by the JSON emitter and the HTML scorecard.
fn report_cards(root: &Path, name: &str) -> Result<(String, Vec<Json>), ViewError> {
    let model = Model::build(root)?;
    let orient = crate::orient::compute(root);
    let done = crate::done::done_names(root);
    let task_suspect: HashSet<String> = orient.suspect.iter().cloned().collect();
    let stale = compute_stale_verifications(root, &model);
    let cov = compute_coverage(&model, &done, &task_suspect, &stale);
    match name {
        "assurance" => Ok(("Assurance Scorecard".to_string(), assurance_cards(root, &model, &cov, &stale, &done, &task_suspect)?)),
        "traceability" => Ok(("Traceability / V&V Coverage".to_string(), traceability_cards(&model, &cov))),
        "quality-debt" => Ok(("Quality & Debt".to_string(), quality_debt_cards(root, &model, &cov, &stale, &task_suspect))),
        "flow" => Ok(("Flow / Velocity".to_string(), flow_cards(root, &model, &orient))),
        "governance" => Ok(("Governance / Decisions".to_string(), governance_cards(&model))),
        "friction" => Ok(("Authoring Friction (vs spreadsheet)".to_string(), friction_cards())),
        other => Err(ViewError::UnknownReport(other.to_string())),
    }
}

/// Computed report as JSON (D0087); `trend` adds a git-derived time-series for the headline metric.
///
/// # Errors
/// Returns [`ViewError`] for an unknown report name or a parse failure.
pub fn report(root: &Path, name: &str, trend: bool) -> Result<String, ViewError> {
    let (title, cards) = report_cards(root, name)?;
    let mut obj = vec![
        ("report".to_string(), Json::s(name.to_string())),
        ("title".to_string(), Json::s(title)),
        ("note".to_string(), Json::s("computed aggregate (D0087) — regenerate, never commit as truth".to_string())),
        ("cards".to_string(), Json::Arr(cards)),
    ];
    if trend {
        obj.push(("trend".to_string(), trend_json(root, name)));
    }
    Ok(Json::Obj(obj).dump())
}

/// A report's headline metric git-derived series `{label, series:[{commit,value}]}` over recent
/// commits (the report's primary scalar, via [`metric_value`]).
fn trend_json(root: &Path, report: &str) -> Json {
    let series: Vec<Json> = trend_series(root, report_headline_key(report))
        .into_iter()
        .map(|(sha, v)| Json::Obj(vec![("commit".to_string(), Json::s(sha)), ("value".to_string(), Json::s(format!("{v:.2}")))]))
        .collect();
    Json::Obj(vec![
        ("label".to_string(), Json::s(headline_label(report).to_string())),
        ("series".to_string(), Json::Arr(series)),
    ])
}

/// The headline metric key a report's `--trend` tracks (the report's primary scalar).
fn report_headline_key(report: &str) -> &str {
    match report {
        "assurance" => "coverage_pct",
        "traceability" => "req_verified_pct",
        "quality-debt" => "volatility",
        "governance" => "accepted_decisions",
        "friction" => "friction_verbs",
        _ => "burnup", // flow
    }
}

/// Computed report rendered as a human-digestible HTML scorecard (D0087).
///
/// # Errors
/// Returns [`ViewError`] for an unknown report name or a parse failure.
pub fn report_html(root: &Path, name: &str, trend: bool) -> Result<String, ViewError> {
    let (title, cards) = report_cards(root, name)?;
    let trend_data = if trend { trend_json(root, name) } else { Json::Null };
    Ok(REPORT_TEMPLATE
        .replace("/*STYLE*/", TABLE_STYLE)
        .replace("/*TITLE*/", &json_esc(&title))
        .replace("/*TREND*/", &trend_data.dump())
        .replace("/*CARDS*/", &Json::Arr(cards).dump()))
}

/// The orient DASHBOARD as a self-contained HTML scorecard (D0093) — the human's recurring home.
///
/// Cards: where things stand + what's ready + open issues + suspect/stale + assurance readiness,
/// reusing the report card template. A computed #View (regenerate-don't-commit), drilling down to the
/// `keel show orient` JSON authority.
///
/// # Errors
/// Returns [`ViewError`] if a tracking/instance file fails to parse.
pub fn orient_html(root: &Path) -> Result<String, ViewError> {
    let o = crate::orient::compute(root);
    let preview = |items: &[String], n: usize| -> String {
        if items.is_empty() {
            return "\u{2014}".to_string();
        }
        let shown: Vec<&str> = items.iter().take(n).map(String::as_str).collect();
        let more = items.len().saturating_sub(n);
        if more > 0 { format!("{} \u{2026} +{more} more", shown.join(", ")) } else { shown.join(", ") }
    };
    let rb = crate::guards::compute_readiness(root)?;
    let wip: Vec<String> = o
        .in_progress_sprints
        .iter()
        .map(|s| format!("{} (pending {})", s.sprint, s.pending.clone().unwrap_or_else(|| "\u{2014}".to_string())))
        .collect();
    let suspect_total = o.suspect.len() + o.invalid_evidence.len();
    let cards = vec![
        card("Progress", format!("{} / {}", o.done, o.outstanding), "completed vs outstanding tasks".to_string(), "good"),
        card("Ready to start", o.ready.len().to_string(), format!("unblocked now: {}", preview(&o.ready, 6)), if o.ready.is_empty() { "warn" } else { "good" }),
        card("Sprints in progress", o.in_progress_sprints.len().to_string(), if wip.is_empty() { "none".to_string() } else { preview(&wip, 4) }, if o.in_progress_sprints.len() <= 2 { "good" } else { "warn" }),
        card("Open issues", o.open_issues.len().to_string(), format!("unresolved: {}", preview(&o.open_issues, 6)), if o.open_issues.is_empty() { "good" } else { "warn" }),
        // Acceptance is the ONE human gate in an otherwise autonomous loop (D0049), so a proposal
        // nobody has seen is the single thing on this dashboard the human alone can clear. It gets a
        // card of its own rather than a line inside the Decisions surface, because that surface is
        // the accepted-only scorecard — it rendered everything EXCEPT what needs action (issue096).
        // "bad" and not "warn" when non-empty: this blocks the loop, it does not merely age.
        card(
            "Awaiting your acceptance",
            o.pending_acceptances.len().to_string(),
            if o.pending_acceptances.is_empty() {
                "no decision is waiting on you".to_string()
            } else {
                format!("proposed: {}", preview(&o.pending_acceptances, 6))
            },
            if o.pending_acceptances.is_empty() { "good" } else { "bad" },
        ),
        card("Suspect / stale", suspect_total.to_string(), format!("{} drift/criterion + {} invalid-evidence \u{2014} re-verify", o.suspect.len(), o.invalid_evidence.len()), if suspect_total == 0 { "good" } else { "warn" }),
        card(
            "Assurance readiness",
            rb.verdict().to_string(),
            format!("{} governed; {} coverage + {} critique + {} \u{2265}Medium + {} Critical + {} invariant blocker(s)", rb.governed, rb.coverage_gaps.len(), rb.critique_gaps.len(), rb.undispositioned_findings.len(), rb.unfixed_critical.len(), rb.invariant_violations.len()),
            rb.tone(),
        ),
    ];
    Ok(REPORT_TEMPLATE
        .replace("/*STYLE*/", TABLE_STYLE)
        .replace("/*TITLE*/", &json_esc("Orient \u{00b7} where things stand"))
        .replace("/*TREND*/", "null")
        .replace("/*CARDS*/", &Json::Arr(cards).dump()))
}

fn assurance_cards(root: &Path, model: &Model, cov: &[Coverage], stale: &HashSet<String>, done: &HashSet<String>, task_suspect: &HashSet<String>) -> Result<Vec<Json>, ViewError> {
    let total = cov.len();
    let ct = |t: &str| cov.iter().filter(|c| c.tier == t).count();
    let (verified, attested) = (ct("verified"), ct("attested"));
    // Headline from the shared coverage-ratio formula (D0090) — computed in exactly one place
    // (`coverage_pct_of`); verified/attested/total below are the structural breakdown for the detail.
    let covered_pct = coverage_pct_of(cov, "");
    let crit = compute_critique_coverage(model, stale, &CritiquePolicy::load(root)?);
    let crit_cov = crit.iter().filter(|c| c.covered).count();
    let crit_pct = pct(crit_cov, crit.len());
    let (att_total, att_missing) = compute_attestation(model);
    let att_pct = pct(att_total - att_missing.len(), att_total);
    // Open finding Issues by severity.
    let open: HashSet<String> = compute_issue_resolution(model, done).into_iter().filter(|i| i.open).map(|i| i.issue).collect();
    let sev_count = |s: &str| open.iter().filter(|n| model.items.get(*n).and_then(|i| i.attrs.get("severity")).map(String::as_str) == Some(s)).count();
    let (crit_f, high_f, med_f, low_f) = (sev_count("Critical"), sev_count("High"), sev_count("Medium"), sev_count("Low"));
    let undisp = crit_f + high_f + med_f;
    let suspect_load = task_suspect.len() + critique_suspect_set(model).len();
    let rb = crate::guards::compute_readiness(root)?;
    Ok(vec![
        card("Verification coverage", format!("{covered_pct}%"), format!("{verified} verified + {attested} attested of {total} (gate-covered)"), if total == 0 { "empty" } else { cov_tone(covered_pct) }),
        card("Critique coverage", format!("{crit_pct}%"), format!("{crit_cov} of {} elements Core-3 critiqued", crit.len()), cov_tone_of(crit_cov, crit.len())),
        card("Acceptance integrity", format!("{att_pct}%"), format!("{} of {att_total} accepted decisions attested", att_total - att_missing.len()), cov_tone_of(att_total - att_missing.len(), att_total)),
        card("Open findings (\u{2265}Medium)", undisp.to_string(), format!("{crit_f} Critical / {high_f} High / {med_f} Medium / {low_f} Low open"), if crit_f > 0 { "bad" } else if undisp > 0 { "warn" } else { "good" }),
        card("Suspect load", suspect_load.to_string(), format!("{} drift/criterion + {} failing-critique; {} stale verifications", task_suspect.len(), critique_suspect_set(model).len(), stale.len()), if suspect_load == 0 { "good" } else { "warn" }),
        card("Assurance readiness", rb.verdict().to_string(), format!("{} governed; {} coverage + {} critique + {} \u{2265}Medium + {} Critical + {} invariant blocker(s)", rb.governed, rb.coverage_gaps.len(), rb.critique_gaps.len(), rb.undispositioned_findings.len(), rb.unfixed_critical.len(), rb.invariant_violations.len()), rb.tone()),
    ])
}

fn traceability_cards(model: &Model, cov: &[Coverage]) -> Vec<Json> {
    let by_type = |ty: &str| -> Vec<&Coverage> { cov.iter().filter(|c| c.type_name == ty).collect() };
    let needs = by_type("Need");
    let reqs = by_type("SystemRequirement");
    // Headlines from the shared verified-ratio formula (D0090; single-source with metric_value); the
    // per-tier breakdown below stays local for the detail text.
    let n_pct = verified_pct_of(cov, "Need");
    let r_pct = verified_pct_of(cov, "SystemRequirement");
    // Edge completeness. A Need is satisfied by an OUTGOING satisfy edge (need -> requirement); a
    // requirement is verified by an INCOMING verify edge (test/critique -> requirement, #Verify).
    let names_of = |ty: &str| -> Vec<&String> { model.items.iter().filter(|(_, i)| i.type_name == ty).map(|(n, _)| n).collect() };
    let needs_names = names_of("Need");
    let n_tot = needs_names.len();
    let n_sat = needs_names.iter().filter(|n| has_outgoing(&model.edges, n, "satisfy")).count();
    let req_names = names_of("SystemRequirement");
    let r_tot = req_names.len();
    let r_ver = req_names
        .iter()
        .filter(|n| model.edges.iter().any(|e| e.kind == "verify" && &e.to == **n))
        .count();
    let r_tier = |t: &str| reqs.iter().filter(|c| c.tier == t).count();
    vec![
        card("Needs verified", format!("{n_pct}%"), format!("{} of {} needs reach a verified requirement", needs.iter().filter(|c| c.tier == "verified").count(), needs.len()), cov_tone(n_pct)),
        card("Requirements verified", format!("{r_pct}%"), format!("{} verified / {} attested / {} addressed / {} uncovered of {}", r_tier("verified"), r_tier("attested"), r_tier("addressed"), r_tier("uncovered") + r_tier("suspect"), reqs.len()), cov_tone(r_pct)),
        card("Needs with satisfy edge", format!("{}%", pct(n_sat, n_tot)), format!("{n_sat} of {n_tot} needs carry a satisfy edge"), cov_tone_of(n_sat, n_tot)),
        card("Requirements with verify edge", format!("{}%", pct(r_ver, r_tot)), format!("{r_ver} of {r_tot} requirements carry a verify edge (DO-178C-style traceability)"), cov_tone(pct(r_ver, r_tot))),
    ]
}

fn quality_debt_cards(root: &Path, model: &Model, cov: &[Coverage], stale: &HashSet<String>, task_suspect: &HashSet<String>) -> Vec<Json> {
    // Charter debt: grandfathered elements (pre-rigor) that are still not gate-covered or not critiqued.
    let gf_cov = crate::govern::grandfathered_under(root, COVERAGE_DECISION);
    let gf_crit = crate::govern::grandfathered_under(root, CRITIQUE_DECISION);
    let cov_debt = cov.iter().filter(|c| !is_covered_tier(c.tier) && gf_cov.as_ref().is_some_and(|g| g.contains(&c.element))).count();
    let crit_debt = compute_critique_coverage(model, stale, &CritiquePolicy::load_or_core3(root)).into_iter().filter(|c| !c.covered && gf_crit.as_ref().is_some_and(|g| g.contains(&c.element))).count();
    // Requirements volatility: supersede edges (churn signal).
    let supersedes = model.edges.iter().filter(|e| e.kind == "supersede").count();
    let decisions = model.items.values().filter(|i| i.type_name == "Decision").count();
    let vol_pct = pct(supersedes, decisions);
    let suspect_total = task_suspect.len() + critique_suspect_set(model).len();
    vec![
        card("Charter debt (coverage)", cov_debt.to_string(), format!("{cov_debt} grandfathered elements still not gate-covered (pre-D0079 rigor backlog)"), if cov_debt == 0 { "good" } else { "warn" }),
        card("Charter debt (critique)", crit_debt.to_string(), format!("{crit_debt} grandfathered elements still missing Core-3 critique (pre-D0080)"), if crit_debt == 0 { "good" } else { "warn" }),
        card("Requirements volatility", format!("{vol_pct}%"), format!("{supersedes} supersede edges across {decisions} decisions (churn / early-warning signal)"), if vol_pct >= 30 { "warn" } else { "good" }),
        card("Suspect + stale", suspect_total.to_string(), format!("{suspect_total} elements suspect; {} stale verifications to re-run", stale.len()), if suspect_total == 0 { "good" } else { "warn" }),
    ]
}

fn flow_cards(root: &Path, model: &Model, orient: &crate::orient::Output) -> Vec<Json> {
    let _ = model;
    let ready = orient.ready.len();
    let wip = orient.in_progress_sprints.len();
    let open_issues = orient.open_issues.len();
    let flows = collect_flows(root);
    let sprints = flows.len();
    let total_points: i64 = flows.iter().map(|f| f.points).sum();
    // Canonical velocity from the shared formula (D0090) — same number the velocity Indicator shows.
    let velocity = velocity_of(&flows);
    // Cycle time, time per point, the inter-commit gap and the point calibration come from GIT in
    // minutes (dcCycleTimeReadsFromGit; issue483/issue485): judgedAt is a date, and a day is coarser
    // than the work. Lead time and aging WIP still read the dates - they are day-scale questions.
    let git_cards = crate::view::flow::facts(root).map_or_else(|e| crate::view::flow::unavailable_cards(&e), |f| crate::view::flow::cards(&f));
    let leads: Vec<i64> = flows.iter().filter_map(|f| Some(f.retro? - f.created?)).collect();
    let lead_mean = if leads.is_empty() { 0 } else { leads.iter().sum::<i64>() / i64::try_from(leads.len()).unwrap_or(1) };
    // Predictability: spread of per-sprint points.
    let pts: Vec<i64> = flows.iter().map(|f| f.points).filter(|p| *p > 0).collect();
    let (pmin, pmax) = (pts.iter().min().copied().unwrap_or(0), pts.iter().max().copied().unwrap_or(0));
    // Aging WIP: as-of (latest recorded date) minus the refine date of any started-but-unfinished sprint.
    let as_of = flows.iter().filter_map(|f| f.retro.or(f.refine)).max().unwrap_or(0);
    let aging = flows.iter().filter(|f| f.refine.is_some() && f.retro.is_none()).filter_map(|f| Some(as_of - f.refine?)).max().unwrap_or(0);
    let mut cards = vec![
        card("Ready frontier", ready.to_string(), format!("{ready} task(s) ready to start now"), if ready == 0 { "warn" } else { "good" }),
        card("Work in progress", wip.to_string(), format!("{wip} sprint(s) with ceremony in progress (low WIP is healthy)"), if wip <= 2 { "good" } else { "warn" }),
        card("Velocity", format!("{velocity:.2}"), format!("~{velocity:.1} points/sprint (mean across {sprints} sprints, {total_points} pts total)"), "good"),
        card("Lead time", format!("{lead_mean}d"), format!("mean created→retro across {} sprints (DORA-style lead time)", leads.len()), "good"),
        card("Predictability", format!("{pmin}–{pmax} pts"), format!("per-sprint point spread (velocity {})", if pmax - pmin <= 4 { "consistent" } else { "variable" }), if pmax - pmin <= 4 { "good" } else { "warn" }),
        card("Throughput", sprints.to_string(), format!("{sprints} delivery sprints recorded"), "good"),
        card("Aging WIP", format!("{aging}d"), format!("oldest unfinished sprint age (as-of latest recorded date); {wip} in progress"), if aging <= 7 { "good" } else { "warn" }),
        card("Open issues", open_issues.to_string(), format!("{open_issues} open issue(s) on the board"), if open_issues == 0 { "good" } else { "warn" }),
    ];
    // The git cards sit where the day-resolution cycle cards sat: after velocity, before lead time.
    cards.splice(3..3, git_cards);
    cards
}

/// Authoring-friction benchmark (D0054/issue029): record one canonical fact (a passing test result)
/// via the write API vs the hand-edit and spreadsheet baselines. Makes the D0054 first-class friction
/// requirement VERIFIABLE — "the write path beats a spreadsheet" becomes a checkable claim.
fn friction_cards() -> Vec<Json> {
    vec![
        card("Write API: record a fact", "1 command".to_string(), "record result / record gate-result / record task / record review (D0451) — one invocation, with auto UUID + who/when/commit provenance + append-only enforcement".to_string(), "good"),
        card("Hand-edit .sysml", "~6 steps".to_string(), "open file, locate the DoD, author the TestResult line, generate a UUID, find the insertion point, save — error-prone, no enforcement".to_string(), "warn"),
        card("Spreadsheet (baseline)", "1 row".to_string(), "fast to type, but NO provenance, NO validation, NO computed resolution/suspicion — the JPL friction trap (D0054)".to_string(), "warn"),
        card("Verdict vs spreadsheet", "beats it".to_string(), "the write path ties the spreadsheet on steps (1 command) and dominates on provenance + validation + computed state — satisfies the D0054 first-class friction requirement".to_string(), "good"),
    ]
}

fn governance_cards(model: &Model) -> Vec<Json> {
    let decisions: Vec<&ItemInfo> = model.items.values().filter(|i| i.type_name == "Decision").collect();
    let total = decisions.len();
    let accepted = model.standing("accepted").len();
    // Retired = the target of a #Supersede edge (D0398); the status field has no such member.
    let retired = model.retired();
    let superseded = model.items.iter().filter(|(n, i)| i.type_name == "Decision" && retired.contains(n.as_str())).count();
    let proc_change = decisions.iter().filter(|i| matches!(i.marker.as_deref(), Some("ProspectiveChange" | "SafetyChange"))).count();
    let (att_total, att_missing) = compute_attestation(model);
    let att_pct = pct(att_total - att_missing.len(), att_total);
    let supersede_edges = model.edges.iter().filter(|e| e.kind == "supersede").count();
    vec![
        card("Decisions", total.to_string(), format!("{accepted} accepted / {superseded} retired of {total} total"), "good"),
        card("Acceptance integrity", format!("{att_pct}%"), format!("{} of {att_total} accepted decisions carry an attestation event", att_total - att_missing.len()), cov_tone(att_pct)),
        card("Process-change decisions", proc_change.to_string(), format!("{proc_change} #ProspectiveChange/#SafetyChange (governed process edits, D0070)"), "good"),
        card("Supersession", supersede_edges.to_string(), format!("{supersede_edges} supersede edges (decision evolution / churn)"), if supersede_edges <= total / 3 { "good" } else { "warn" }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_produces_cards_and_rejects_unknown() {
        // D0087: each report yields a non-empty cards array; unknown report errors. (cwd = crate dir.)
        let root = std::path::Path::new("..");
        for name in ["assurance", "traceability", "quality-debt", "flow", "governance", "friction"] {
            let json = report(root, name, false).unwrap_or_else(|e| panic!("report {name}: {e}"));
            assert!(json.contains("\"cards\""), "{name} has cards");
            assert!(json.contains("\"tone\""), "{name} cards carry a tone");
        }
        assert!(report(root, "bogus", false).is_err(), "unknown report errors");
        let html = report_html(root, "assurance", false).expect("assurance html");
        assert!(html.contains("class=\"cards\"") && !html.contains("/*CARDS*/"), "scorecard cards injected");
    }
}
