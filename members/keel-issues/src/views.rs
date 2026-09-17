//! The item views (D0480, sprint 737): open-issues (D0077), dispositions (D0092) and intake (D0166),
//! sliced whole out of keel-view's view/mod.rs and view/staleness.rs by `scripts/extract_issues.py`.
//! keel-cli's `view` wrapper module re-exports the three, so `keel show open-issues`, `dispositions` and
//! `intake` and the server's lens table resolve them at their old paths.

use std::collections::BTreeMap;
use std::path::Path;

use keel_json::json::Json;
use keel_model::model::{ItemInfo, Model, ViewError};
use keel_model::queries::{at_least_medium, compute_issue_resolution, issue_disposition};

/// Open-issues view (D0077) as JSON: every OPEN issue + its resolvers + completeness, with counts.
///
/// # Errors
/// Returns [`ViewError`] if a tracking/instance file fails to parse.
pub fn open_issues(root: &Path) -> Result<String, ViewError> {
    let model = Model::build(root)?;
    let done = keel_model::done::done_names(root);
    let all = compute_issue_resolution(&model, &done);
    let total = all.len();
    let open_count = all.iter().filter(|i| i.open).count();
    let open_list: Vec<Json> = all
        .iter()
        .filter(|i| i.open)
        .map(|i| {
            let resolvers: Vec<Json> = i
                .resolvers
                .iter()
                .map(|r| {
                    Json::Obj(vec![
                        ("name".to_string(), Json::s(r.name.clone())),
                        ("kind".to_string(), Json::s(r.kind)),
                        ("complete".to_string(), Json::Bool(r.complete)),
                    ])
                })
                .collect();
            Json::Obj(vec![
                ("issue".to_string(), Json::s(i.issue.clone())),
                ("untriaged".to_string(), Json::Bool(i.resolvers.is_empty())),
                ("resolvers".to_string(), Json::Arr(resolvers)),
            ])
        })
        .collect();
    let out = Json::Obj(vec![
        ("total_issues".to_string(), Json::Int(i64::try_from(total).unwrap_or(i64::MAX))),
        ("open".to_string(), Json::Int(i64::try_from(open_count).unwrap_or(i64::MAX))),
        ("resolved".to_string(), Json::Int(i64::try_from(total - open_count).unwrap_or(i64::MAX))),
        ("open_issues".to_string(), Json::Arr(open_list)),
    ]);
    Ok(out.dump())
}

/// Dispositions view (D0092): every >= Medium finding + its typed disposition verdict.
///
/// Each verdict is `act`/`acceptRisk`/`dismiss` or `undispositioned` — the computed read of the
/// human-judgment gate (reads the typed verdict, not prose/proxy). `undispositioned` is what `assured`
/// enforces.
///
/// # Errors
/// Returns [`ViewError`] if a tracking/instance file fails to parse.
pub fn dispositions(root: &Path) -> Result<String, ViewError> {
    let model = Model::build(root)?;
    let mut findings: Vec<(&String, &ItemInfo)> = model
        .items
        .iter()
        .filter(|(_, i)| i.type_name == "Issue" && i.attrs.get("severity").is_some_and(|s| at_least_medium(s)))
        .collect();
    findings.sort_by(|a, b| a.0.cmp(b.0));
    let mut undisp = 0usize;
    let rows: Vec<Json> = findings
        .iter()
        .map(|(name, info)| {
            let verdict = issue_disposition(&model, name);
            if verdict.is_none() {
                undisp += 1;
            }
            Json::Obj(vec![
                ("finding".to_string(), Json::s((*name).clone())),
                ("severity".to_string(), Json::s(info.attrs.get("severity").cloned().unwrap_or_default())),
                ("dispositioned".to_string(), Json::Bool(verdict.is_some())),
                ("disposition".to_string(), verdict.map_or_else(|| Json::s("undispositioned".to_string()), Json::s)),
            ])
        })
        .collect();
    let total = rows.len();
    let out = Json::Obj(vec![
        ("ge_medium_findings".to_string(), Json::Int(i64::try_from(total).unwrap_or(i64::MAX))),
        ("dispositioned".to_string(), Json::Int(i64::try_from(total - undisp).unwrap_or(i64::MAX))),
        ("undispositioned".to_string(), Json::Int(i64::try_from(undisp).unwrap_or(i64::MAX))),
        ("findings".to_string(), Json::Arr(rows)),
    ]);
    Ok(out.dump())
}

/// The INTAKE view (D0166): what was said, what it became, and what nobody acted on.
///
/// Three gaps, none of which was computable before:
///   UNPARSED  - a `Statement` no `UserStory` cites. Direction they gave that nothing translated.
///   UNROUTED  - a `UserStory` whose `implication` is not `none` and which reaches no downstream item.
///               Triaged and then dropped, which is worse than untriaged because it looks handled.
///   UNSOURCED - a `Need` / `SystemRequirement` / `Issue` / `Decision` that no `UserStory` implicates. Work with
///               no recorded human statement behind it. Expected to be large and expected to be
///               uncomfortable: it is the ratio of what was asked for to what I invented.
///
/// UNSOURCED IS A FLOOR, NOT A TOTAL, and the view says so: nothing can force a statement to be
/// recorded, so an item may be genuinely requested and simply have no `Statement` written down. The
/// number measures the RECORD's completeness, never the human's.
///
/// # Errors
/// Returns [`ViewError`] if the model cannot be built.
pub fn intake(root: &Path) -> Result<String, ViewError> {
    // The item types a UserStory can implicate. Declared at the top: an item after statements is
    // confusing because items exist from the start of the scope regardless.
    const DOWNSTREAM: [&str; 4] = ["Need", "SystemRequirement", "Issue", "Decision"];
    // Kinds that owe no downstream item: an acceptance produces nothing, its outcome IS the
    // acknowledgement; a question is answered, a priority applied by reordering, a convention adopted in
    // prose, a correction absorbed by an existing record. Declared with DOWNSTREAM because both are items
    // and an item after statements reads as a surprise.
    const SELF_TERMINATING: [&str; 6] =
        ["none", "attestation", "question", "priority", "convention", "correction"];
    let model = Model::build(root)?;
    let is = |n: &str, ty: &str| model.items.get(n).is_some_and(|i| i.type_name == ty);

    let statements: Vec<&String> =
        model.items.iter().filter(|(_, i)| i.type_name == "Statement").map(|(n, _)| n).collect();
    let stories: Vec<&String> =
        model.items.iter().filter(|(_, i)| i.type_name == "UserStory").map(|(n, _)| n).collect();

    // a story cites its statement with #DerivedFrom; it names its outcome with #Implicates
    let mut cited: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut routed: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut implicated: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for e in &model.edges {
        if e.kind == "derivedfrom" && is(&e.from, "UserStory") && is(&e.to, "Statement") {
            cited.insert(e.to.as_str());
        }
        if e.kind == "implicates" && is(&e.from, "UserStory") {
            routed.insert(e.from.as_str());
            implicated.insert(e.to.as_str());
        }
    }

    let mut unparsed: Vec<String> =
        statements.iter().filter(|s| !cited.contains(s.as_str())).map(|s| (*s).clone()).collect();
    // ONLY THE PRODUCTIVE KINDS OWE AN OUTCOME. The first version required a downstream item for every
    // implication except `none`, and using it immediately flagged a legitimate `attestation` as unrouted:
    // an acceptance PRODUCES nothing, its outcome IS the acknowledgement. Same for a question answered, a
    // priority applied by reordering, a convention adopted in prose, a correction that edits an existing
    // record. Reporting those as gaps would train the reader to ignore the number - the warning-fatigue
    // failure issue160 records one layer up. They are counted separately as self-terminating, so they stay
    // visible without being defects.
    let kind_of = |s: &str| -> String {
        model
            .items
            .get(s)
            .and_then(|i| i.attrs.get("implication"))
            .map(|k| k.rsplit("::").next().unwrap_or(k).to_string())
            .unwrap_or_default()
    };
    let mut unrouted: Vec<String> = stories
        .iter()
        .filter(|s| {
            !SELF_TERMINATING.contains(&kind_of(s).as_str()) && !routed.contains(s.as_str())
        })
        .map(|s| (*s).clone())
        .collect();
    let self_terminating = stories
        .iter()
        .filter(|s| SELF_TERMINATING.contains(&kind_of(s).as_str()))
        .count();
    // A story with NO #DerivedFrom is an invention wearing a story's clothes - reported separately from
    // unrouted, because the two need different fixes: one needs a source, the other needs an outcome.
    let mut unsourced_stories: Vec<String> = stories
        .iter()
        .filter(|s| !model.edges.iter().any(|e| e.kind == "derivedfrom" && e.from == ***s))
        .map(|s| (*s).clone())
        .collect();

    let mut unsourced: Vec<String> = model
        .items
        .iter()
        .filter(|(n, i)| DOWNSTREAM.contains(&i.type_name.as_str()) && !implicated.contains(n.as_str()))
        .map(|(n, _)| n.clone())
        .collect();
    let downstream_total = model
        .items
        .values()
        .filter(|i| DOWNSTREAM.contains(&i.type_name.as_str()))
        .count();

    for v in [&mut unparsed, &mut unrouted, &mut unsourced_stories, &mut unsourced] {
        v.sort();
    }
    let cap = |v: &[String], n: usize| -> Json {
        Json::Arr(v.iter().take(n).map(|s| Json::s(s.clone())).collect())
    };

    // per-implication tally, so the triage distribution is visible rather than inferred
    let mut by_kind: BTreeMap<String, i64> = BTreeMap::new();
    for s in &stories {
        let k = model.items.get(*s).and_then(|i| i.attrs.get("implication")).cloned().unwrap_or_default();
        *by_kind.entry(k.rsplit("::").next().unwrap_or("unrecorded").to_string()).or_insert(0) += 1;
    }

    Ok(Json::Obj(vec![
        ("statements".to_string(), Json::Int(i64::try_from(statements.len()).unwrap_or(0))),
        ("userStories".to_string(), Json::Int(i64::try_from(stories.len()).unwrap_or(0))),
        ("unparsed".to_string(), Json::Int(i64::try_from(unparsed.len()).unwrap_or(0))),
        ("unparsed_statements".to_string(), cap(&unparsed, 20)),
        ("unrouted".to_string(), Json::Int(i64::try_from(unrouted.len()).unwrap_or(0))),
        ("unrouted_stories".to_string(), cap(&unrouted, 20)),
        ("selfTerminating".to_string(), Json::Int(i64::try_from(self_terminating).unwrap_or(0))),
        ("storiesWithNoStatement".to_string(), Json::Int(i64::try_from(unsourced_stories.len()).unwrap_or(0))),
        ("storiesWithNoStatement_list".to_string(), cap(&unsourced_stories, 20)),
        ("downstreamItems".to_string(), Json::Int(i64::try_from(downstream_total).unwrap_or(0))),
        ("unsourced".to_string(), Json::Int(i64::try_from(unsourced.len()).unwrap_or(0))),
        ("unsourced_sample".to_string(), cap(&unsourced, 20)),
        ("byImplication".to_string(), Json::Obj(by_kind.into_iter().map(|(k, v)| (k, Json::Int(v))).collect())),
        (
            "unsourcedNote".to_string(),
            Json::s(
                "unsourced counts downstream items no UserStory implicates. It is a FLOOR on the                  record's completeness, never a claim about the human: nothing can force a statement to                  be written down, so an item may be genuinely requested and simply unrecorded."
                    .to_string(),
            ),
        ),
    ])
    .dump())
}
