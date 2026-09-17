//! The priority lens (D0311): every ready item's effective class, what drives it, and the inversions.
//!
//! Declaration order IS priority (D0052); this lens says where a resolver's severity or a retro's
//! recurrence disagrees with it. It reads the frontier from orient and the records from the view
//! model, so it sits above both (D0479, dcGuardsViewOrientCycleIsBroken) - the `priority-inversions`
//! guard reads it from there.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use keel_json::json::Json;
use crate::view::{inversion_pairs, Model, ViewError};

/// The deliberate-rank records (D0429): every `#PrioritizedBy` edge, as `item -> citation` when its
/// target is a Statement or Decision, and as a violation line otherwise - a citation to an Issue, a
/// Task or an unresolved name would let an item rank itself, so it records nothing.
fn priority_records(model: &Model) -> (HashMap<String, String>, Vec<String>) {
    let mut recorded = HashMap::new();
    let mut violations = Vec::new();
    for e in model.edges.iter().filter(|e| e.kind == "prioritizedby") {
        match model.items.get(&e.to).map(|i| i.type_name.as_str()) {
            Some("Statement" | "Decision") => {
                recorded.insert(e.from.clone(), e.to.clone());
            }
            Some(t) => violations.push(format!(
                "#PrioritizedBy dependency from {} to {}: the target is of type {t}, not a Statement or Decision - a deliberate rank is recorded against the human's words or a recorded judgment, never against another work item (D0429)",
                e.from, e.to
            )),
            None => violations.push(format!("#PrioritizedBy dependency from {} to {}: the target resolves to no item (D0429)", e.from, e.to)),
        }
    }
    (recorded, violations)
}

/// One ready item's computed priority signals (D0311): where it stands, what it resolves, how many
/// retros have named it as the already-tracked home of a finding, and the class those combine to.
pub struct PrioritySignal {
    pub task: String,
    pub position: usize,
    pub resolver_severity: Option<String>,
    pub recurrences: usize,
    /// The rank class the signals justify: `Critical`, `High`, `Medium`, `Low` or `none`.
    pub effective: String,
    /// Which signal set the class - `severity`, `recurrence`, or `none`.
    pub driven_by: &'static str,
    /// The Statement or Decision a `#PrioritizedBy` edge cites as ranking this item (D0429), if any.
    pub ranked_by: Option<String>,
}

/// What [`priority_inversions`] reports: the inversions still standing, the deliberate ones with their
/// citations, and the records whose target cannot rank anything.
pub struct PriorityInversions {
    /// `(outranking item, high item, its class)` for every inversion no record covers.
    pub pairs: Vec<(String, String, String)>,
    /// `(item, citation)` for every ready item whose rank a `#PrioritizedBy` edge records.
    pub recorded: Vec<(String, String)>,
    /// One line per `#PrioritizedBy` edge whose target is not a Statement or Decision.
    pub violations: Vec<String>,
}

fn class_rank(s: &str) -> u8 {
    match s {
        "Critical" => 4,
        "High" => 3,
        "Medium" => 2,
        "Low" => 1,
        _ => 0,
    }
}

/// How many sprint retros name each open backlog item as the tracked home of a finding they chose not
/// to open an item for. One citation is tracking; every further one is the finding coming back while
/// the item sat (D0311, the heredoc case: eight recurrences under "already tracked").
fn retro_citations(root: &Path) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for f in keel_model::corpus::collect_sysml(&root.join(".tracking").join("delivery")) {
        let Ok(t) = std::fs::read_to_string(&f) else { continue };
        for retro in keel_model::textscan::retro_texts(&t) {
            let lower = retro.to_lowercase();
            if !keel_model::textscan::RETRO_NO_ITEM_JUSTIFICATIONS.iter().any(|j| lower.contains(*j)) {
                continue;
            }
            let mut seen: HashSet<String> = HashSet::new();
            for n in keel_model::textscan::named_items(&retro) {
                if seen.insert(n.clone()) {
                    *counts.entry(n).or_insert(0) += 1;
                }
            }
        }
    }
    counts
}

/// The rank class recurrence alone justifies: twice puts a finding with the High resolvers, three or
/// more with the Critical ones - each citation past the first is one more time the defect came back.
const fn recurrence_class(citations: usize) -> Option<&'static str> {
    match citations {
        0 | 1 => None,
        2 => Some("High"),
        _ => Some("Critical"),
    }
}

/// Every ready item's priority signals, in declaration (= priority) order (D0311).
///
/// # Errors
/// Propagates model-build failures.
pub fn priority_signals(root: &Path) -> Result<Vec<PrioritySignal>, ViewError> {
    let model = Model::build(root)?;
    // The frontier alone (issue439): the guard built on this paid orient::compute whole - suspect
    // walk, drift, burndown - to read `ready`, and was the critical path of every slow hook fire.
    let (ready, _compute_failures, _outstanding) = keel_perf::perf::phase("priority:frontier", || keel_model::orient::ready(root));
    let citations = keel_perf::perf::phase("priority:retro-citations", || retro_citations(root));
    let (records, _) = priority_records(&model);
    let severity_of = |task: &str| -> Option<String> {
        model
            .edges
            .iter()
            .filter(|e| e.kind == "resolves" && e.from == task)
            .filter_map(|e| model.items.get(&e.to))
            .filter_map(|i| i.attrs.get("severity").cloned())
            .max_by_key(|s| class_rank(s))
    };
    Ok(ready
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let sev = severity_of(t);
            let rec = citations.get(t).copied().unwrap_or(0);
            let rec_class = recurrence_class(rec);
            let sev_rank = sev.as_deref().map_or(0, class_rank);
            let rec_rank = rec_class.map_or(0, class_rank);
            let (effective, driven_by) = if rec_rank > sev_rank {
                (rec_class.unwrap_or("none").to_string(), "recurrence")
            } else if sev_rank > 0 {
                (sev.clone().unwrap_or_default(), "severity")
            } else {
                ("none".to_string(), "none")
            };
            PrioritySignal { task: t.clone(), position: i + 1, resolver_severity: sev, recurrences: rec, effective, driven_by, ranked_by: records.get(t).cloned() }
        })
        .collect())
}

/// The inversions the signals justify, less the recorded ones, with the records and their defects.
fn inversions_of(signals: &[PrioritySignal], record_violations: Vec<String>) -> PriorityInversions {
    let ready: Vec<(String, Option<String>)> = signals.iter().map(|s| (s.task.clone(), (s.effective != "none").then(|| s.effective.clone()))).collect();
    let recorded: Vec<(String, String)> = signals.iter().filter_map(|s| s.ranked_by.clone().map(|by| (s.task.clone(), by))).collect();
    let exempt: HashSet<String> = recorded.iter().map(|(t, _)| t.clone()).collect();
    PriorityInversions { pairs: inversion_pairs(&ready, &exempt), recorded, violations: record_violations }
}

/// `keel show priority` (D0311): the priority metric, made visible - every ready item in its declared
/// order with the computed signals and the class they justify, and the inversions the guard reports.
///
/// # Errors
/// Propagates model-build failures.
pub fn priority(root: &Path) -> Result<String, ViewError> {
    let signals = priority_signals(root)?;
    let rows: Vec<Json> = signals
        .iter()
        .map(|s| {
            Json::Obj(vec![
                ("position".to_string(), Json::Int(i64::try_from(s.position).unwrap_or(i64::MAX))),
                ("task".to_string(), Json::s(s.task.clone())),
                ("effective".to_string(), Json::s(s.effective.clone())),
                ("drivenBy".to_string(), Json::s(s.driven_by)),
                ("resolverSeverity".to_string(), s.resolver_severity.clone().map_or(Json::Null, Json::s)),
                ("retroRecurrences".to_string(), Json::Int(i64::try_from(s.recurrences).unwrap_or(i64::MAX))),
                ("rankedBy".to_string(), s.ranked_by.clone().map_or(Json::Null, Json::s)),
            ])
        })
        .collect();
    let model = Model::build(root)?;
    let (_, record_violations) = priority_records(&model);
    let inv = inversions_of(&signals, record_violations);
    let inversions: Vec<Json> = inv
        .pairs
        .into_iter()
        .map(|(lower, higher, class)| Json::Obj(vec![("outranks".to_string(), Json::s(lower)), ("item".to_string(), Json::s(higher)), ("class".to_string(), Json::s(class))]))
        .collect();
    let recorded: Vec<Json> = inv.recorded.into_iter().map(|(item, by)| Json::Obj(vec![("item".to_string(), Json::s(item)), ("rankedBy".to_string(), Json::s(by))])).collect();
    Ok(Json::Obj(vec![
        ("priority".to_string(), Json::s("D0052: declaration order IS priority. D0258/D0311: rank is reassessed against COMPUTED signals - the resolver Issue's severity, and how many sprint retros named the item as the already-tracked home of a finding (twice = High, three or more = Critical). An inversion is a lower-class item declared ABOVE a higher one; the priority-inversion guard warns on each, and refineAssessPriority discharges it by reordering or by recording why - a #PrioritizedBy edge from the item to the Statement or Decision that ranks it (D0429), listed under recorded.")),
        ("ready".to_string(), Json::Arr(rows)),
        ("inversions".to_string(), Json::Arr(inversions)),
        ("recorded".to_string(), Json::Arr(recorded)),
        ("recordViolations".to_string(), Json::Arr(inv.violations.into_iter().map(Json::s).collect())),
    ])
    .dump())
}

/// Backlog priority inversions: a ready item ranked ABOVE work that resolves a >= High Issue.
///
/// Closes issue084 (D0130). D0052 makes backlog DECLARATION ORDER the priority and requires the AI to
/// auto-follow the ranked frontier — but nothing computed whether recorded ORDER agreed with recorded
/// SEVERITY, so a mis-ordered backlog was indistinguishable from a curated one. It was mis-ordered:
/// `keelArchViews` (issue069, Low) ranked FIRST purely because an earlier session appended it to the
/// end of a COMPLETED block, while `dcStaleKernelInstanceGate` (issue081, High — an enforced commit
/// gate being routinely bypassed) ranked 14th, and the AI then narrated priority in prose instead of
/// reordering the file. Both inputs are recorded facts, so the inversion is COMPUTABLE.
///
/// Reported, never enforced: priority is a human judgment and ordering may be deliberate (a High item
/// can be legitimately deferred behind an enabler). The value is that the trade-off becomes VISIBLE
/// instead of resting on whoever last appended to the file.
///
/// D0429: an inversion the backlog RECORDS - a `#PrioritizedBy` edge from the outranking item to the
/// Statement or Decision that ranks it - is deliberate by the record and is not in `pairs`; it is in
/// `recorded` with its citation. The guard's "if that is deliberate say so" now names where.
///
/// # Errors
/// Returns [`ViewError`] if a tracking/instance file fails to parse.
pub fn priority_inversions(root: &Path) -> Result<PriorityInversions, ViewError> {
    // D0311: the class compared is the EFFECTIVE one - resolver severity or retro recurrence, whichever
    // is higher - so a finding that keeps coming back climbs the same ladder a High Issue does, and the
    // same guard, the same warning and the same refinement step carry it.
    let signals = priority_signals(root)?;
    let model = Model::build(root)?;
    let (_, record_violations) = priority_records(&model);
    Ok(inversions_of(&signals, record_violations))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_records_read_the_edge_and_refuse_a_target_that_is_not_a_word_or_a_judgment() {
        // D0429: `#PrioritizedBy dependency from <item> to <stNNN|dNNNN>;` is the record; an edge to
        // any other type is a violation, never a silent exemption - an item citing an Issue or a Task
        // would be ranking itself.
        let dir = std::env::temp_dir().join(format!("keel_prank_{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        let tracking = dir.join(".tracking");
        std::fs::create_dir_all(&tracking).unwrap();
        std::fs::write(
            tracking.join("m.sysml"),
            concat!(
                "package P {\n",
                "    part st1 : Statement { :>> text = \"do this first\"; }\n",
                "    part d1 : Decision { :>> title = \"first\"; }\n",
                "    part issue1 : Issue { :>> severity = \"High\"; }\n",
                "    action def W { action byWord; action byJudgment; action byIssue; action byNothing; }\n",
                "    #PrioritizedBy dependency from byWord to st1;\n",
                "    #PrioritizedBy dependency from byJudgment to d1;\n",
                "    #PrioritizedBy dependency from byIssue to issue1;\n",
                "    #PrioritizedBy dependency from byNothing to nowhere;\n",
                "}\n"
            ),
        )
        .unwrap();
        let model = Model::build_with_workers(&dir, 1).unwrap();
        let (recorded, violations) = priority_records(&model);
        assert_eq!(recorded.get("byWord").map(String::as_str), Some("st1"));
        assert_eq!(recorded.get("byJudgment").map(String::as_str), Some("d1"));
        assert_eq!(recorded.len(), 2, "{recorded:?}");
        assert_eq!(violations.len(), 2, "{violations:?}");
        assert!(violations.iter().any(|v| v.contains("byIssue") && v.contains("of type Issue")), "{violations:?}");
        assert!(violations.iter().any(|v| v.contains("byNothing") && v.contains("resolves to no item")), "{violations:?}");
        // Through the signals: the record is the citation, and a recorded item is out of `pairs`.
        let sig = |task: &str, class: Option<&str>, by: Option<&str>| PrioritySignal {
            task: task.to_string(),
            position: 0,
            resolver_severity: None,
            recurrences: 0,
            effective: class.unwrap_or("none").to_string(),
            driven_by: "none",
            ranked_by: by.map(str::to_string),
        };
        let inv = inversions_of(&[sig("byWord", None, Some("st1")), sig("enabler", None, None), sig("urgent", Some("High"), None)], violations);
        assert_eq!(inv.pairs, vec![("enabler".to_string(), "urgent".to_string(), "High".to_string())]);
        assert_eq!(inv.recorded, vec![("byWord".to_string(), "st1".to_string())]);
        assert_eq!(inv.violations.len(), 2);
        std::fs::remove_dir_all(&dir).ok();
    }
}
