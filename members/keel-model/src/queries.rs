//! Secondary read-only query views re-added by readdDroppedViews (resolves issue025).
//!
//! `outstanding`, `item`, `trace` (up + downstream), `trace-need`, `workflows` — query.py
//! subcommands dropped at M4, re-implemented over the Rust authority: the indexer's action DAG,
//! the parser's satisfy/allocate edges, and the workflow action defs. The `viewpoints` listing
//! is a TOML view (`keel show view viewpoints`), not here.
//!
//! Also the predicates the frontier narrows on (sprint 718, D0479): issue resolution, the retired
//! and blocked sets, the proposed Decisions, the claim rows and the repo's date. Each is COMPUTED
//! from the model every call; nothing here is stored.

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

use keel_parser::ast::Item;
use keel_parser::{parse, tokenize};

use keel_json::json::Json;
use crate::model::{Model, ViewError};

fn idx(root: &Path) -> crate::indexer::ExtractedIndex {
    crate::indexer::extract(&root.join(".tracking"))
}

/// `keel outstanding` — backlog tasks that are not done.
#[must_use]
pub fn outstanding(root: &Path) -> String {
    let tasks = idx(root).tasks;
    let done = crate::done::done_names(root);
    let mut out: Vec<String> = tasks.keys().filter(|t| !done.contains(t.as_str())).cloned().collect();
    out.sort();
    Json::Obj(vec![("outstanding".to_string(), Json::Arr(out.into_iter().map(Json::s).collect()))]).dump()
}

/// `keel show item <name>` — one task's detail (done, deps, `DoD` text, results).
#[must_use]
pub fn item(root: &Path, name: &str) -> String {
    let index = idx(root);
    let done = crate::done::done_names(root);
    let Some(t) = index.tasks.get(name) else {
        return Json::Obj(vec![("error".to_string(), Json::s(format!("no task '{name}'")))]).dump();
    };
    let results: Vec<Json> = t
        .results
        .iter()
        .map(|r| {
            Json::Obj(vec![
                ("n".to_string(), Json::Int(i64::from(r.n))),
                ("outcome".to_string(), Json::s(r.outcome.clone())),
                ("judgedAgainst".to_string(), Json::s(r.judged_against.clone())),
            ])
        })
        .collect();
    Json::Obj(vec![
        ("name".to_string(), Json::s(name)),
        ("done".to_string(), Json::Bool(done.contains(name))),
        ("deps".to_string(), Json::Arr(t.deps.iter().map(|d| Json::s(d.clone())).collect())),
        ("dod".to_string(), Json::s(t.dod_text.clone().unwrap_or_default())),
        ("results".to_string(), Json::Arr(results)),
    ])
    .dump()
}

fn reach(start: &str, adj: &HashMap<String, Vec<String>>) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut q: VecDeque<String> = VecDeque::new();
    if let Some(ns) = adj.get(start) {
        q.extend(ns.iter().cloned());
    }
    while let Some(n) = q.pop_front() {
        if seen.insert(n.clone()) {
            if let Some(ns) = adj.get(&n) {
                q.extend(ns.iter().cloned());
            }
        }
    }
    let mut out: Vec<String> = seen.into_iter().collect();
    out.sort();
    out
}

/// `keel trace <name>` — transitive upstream (deps) + downstream (dependents) over the
/// succession DAG. Covers the dropped `trace`, `upstream`, and `downstream`.
#[must_use]
pub fn trace(root: &Path, name: &str) -> String {
    let tasks = idx(root).tasks;
    let upstream_adj: HashMap<String, Vec<String>> = tasks.iter().map(|(k, v)| (k.clone(), v.deps.clone())).collect();
    let mut downstream_adj: HashMap<String, Vec<String>> = HashMap::new();
    for (k, v) in &tasks {
        for d in &v.deps {
            downstream_adj.entry(d.clone()).or_default().push(k.clone());
        }
    }
    Json::Obj(vec![
        ("name".to_string(), Json::s(name)),
        ("upstream".to_string(), Json::Arr(reach(name, &upstream_adj).into_iter().map(Json::s).collect())),
        ("downstream".to_string(), Json::Arr(reach(name, &downstream_adj).into_iter().map(Json::s).collect())),
    ])
    .dump()
}

fn parse_dir(dir: &Path) -> Vec<keel_parser::ast::Package> {
    crate::corpus::collect_sysml(dir)
        .iter()
        .filter_map(|p| {
            let src = std::fs::read_to_string(p).ok()?;
            let name = p.display().to_string();
            let tokens = tokenize(&src, &name).ok()?;
            parse(tokens, &name).ok()
        })
        .collect()
}

/// `keel trace-need <name>` — forward closure over satisfy/allocate edges from a Need
/// (need -satisfy-> requirement -allocate-> component).
///
/// D0194: the June Component layer this closure walks is SUPERSEDED - the output says so, because
/// serving it unlabeled as current truth was the panel's EHZ8 finding. Live allocation is the
/// charter chain + the SR->CodeElement edges (see `keel show arch` and allocations.sysml).
#[must_use]
pub fn trace_need(root: &Path, name: &str) -> String {
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for pkg in parse_dir(&root.join(".tracking")) {
        for it in &pkg.items {
            match it {
                Item::Satisfy(e) => adj.entry(e.need.clone()).or_default().push(e.by.clone()),
                Item::Allocate(e) => adj.entry(e.sr.clone()).or_default().push(e.to.clone()),
                _ => {}
            }
        }
    }
    Json::Obj(vec![
        ("need".to_string(), Json::s(name)),
        ("componentLayer".to_string(), Json::s("HISTORICAL - the June Component set this closure reaches is superseded by D0194; live allocation is the charter chain + SR->CodeElement edges (keel show arch, allocations.sysml)")),
        ("trace".to_string(), Json::Arr(reach(name, &adj).into_iter().map(Json::s).collect())),
    ])
    .dump()
}

/// `keel workflows` — each workflow action def's phases as Kahn topological waves over its
/// succession edges.
#[must_use]
pub fn workflows(root: &Path) -> String {
    let mut wfs: Vec<Json> = Vec::new();
    for pkg in parse_dir(&root.join(".engine").join("workflows")) {
        for it in &pkg.items {
            let Item::ActionDef(def) = it else { continue };
            let nodes: Vec<String> = def.actions.iter().map(|a| a.name.clone()).collect();
            if nodes.is_empty() {
                continue;
            }
            let edges: Vec<(String, String)> = def.successions.iter().map(|s| (s.first.clone(), s.then.clone())).collect();
            wfs.push(workflow_json(&pkg.name, &def.name, &nodes, &edges));
        }
    }
    Json::Obj(vec![("workflows".to_string(), Json::Arr(wfs))]).dump()
}

fn workflow_json(package: &str, name: &str, nodes: &[String], edges: &[(String, String)]) -> Json {
    let mut indeg: HashMap<&str, usize> = nodes.iter().map(|n| (n.as_str(), 0)).collect();
    let mut succ: HashMap<&str, Vec<&str>> = HashMap::new();
    for (a, b) in edges {
        succ.entry(a.as_str()).or_default().push(b.as_str());
        *indeg.entry(b.as_str()).or_insert(0) += 1;
    }
    let mut waves: Vec<Vec<String>> = Vec::new();
    let mut placed = 0usize;
    let mut frontier: Vec<&str> = indeg.iter().filter(|(_, &d)| d == 0).map(|(n, _)| *n).collect();
    frontier.sort_unstable();
    while !frontier.is_empty() {
        waves.push(frontier.iter().map(|s| (*s).to_string()).collect());
        placed += frontier.len();
        let mut next: Vec<&str> = Vec::new();
        for n in &frontier {
            for m in succ.get(n).into_iter().flatten() {
                let e = indeg.entry(m).or_insert(0);
                *e = e.saturating_sub(1);
                if *e == 0 {
                    next.push(m);
                }
            }
        }
        next.sort_unstable();
        next.dedup();
        frontier = next;
    }
    let mut obj = vec![
        ("workflow".to_string(), Json::s(name)),
        ("package".to_string(), Json::s(package)),
        ("phaseCount".to_string(), Json::Int(i64::try_from(nodes.len()).unwrap_or(i64::MAX))),
    ];
    if placed < nodes.len() {
        obj.push(("error".to_string(), Json::s("dependency cycle (not all phases placed)")));
    }
    obj.push(("waves".to_string(), Json::Arr(waves.into_iter().map(|w| Json::Arr(w.into_iter().map(Json::s).collect())).collect())));
    Json::Obj(obj)
}

/// Is `name` DECLARED anywhere in the model? A cheap text scan, no model build (issue177).
///
/// WHY THIS EXISTS. Every name-taking read command used to answer for a name that does not exist:
/// `keel trace .` returned `{upstream: [], downstream: []}` with EXIT 0, and `keel show governing-version .`
/// reported a process and a process definition for it. D0093 makes the CLI the automation substrate, so
/// a consumer reads an empty relation set as "this item has no relations" rather than "there is no such
/// item" - and a typo in a script becomes a silent wrong answer, the reassuring kind.
///
/// A TEXT SCAN rather than `Model::build`, because this runs BEFORE the command decides to do real work
/// and must not double its cost. It matches a declaration keyword followed by the name and a `:`, which
/// is how every item in this model is introduced.
#[must_use]
pub fn is_declared(root: &Path, name: &str) -> bool {
    const KEYWORDS: [&str; 8] =
        ["part ", "verification ", "action ", "requirement ", "use case ", "item ", "attribute ", "enum "];
    for base in [".tracking", ".engine", ".knowledge"] {
        for f in crate::corpus::collect_sysml(&root.join(base)) {
            let Ok(text) = std::fs::read_to_string(&f) else { continue };
            for raw in text.lines() {
                let line = raw.trim_start();
                if line.starts_with("//") {
                    continue;
                }
                // A marker may precede the keyword: `#Capability part foo : Bar`.
                let after_marker = line.strip_prefix('#').map_or(line, |r| {
                    r.split_once(char::is_whitespace).map_or("", |(_, rest)| rest.trim_start())
                });
                for kw in KEYWORDS {
                    if let Some(rest) = after_marker.strip_prefix(kw) {
                        // A declaration may be bare (`action foo;` in a delivery run) or typed
                        // (`part foo : Story {`). Stopping only at `:` kept the `;` and made every
                        // bare action look undeclared - which broke `keel show item <a backlog action>`.
                        let decl =
                            rest.split([':', ';', '{']).next().unwrap_or("").trim();
                        if decl == name {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

// ── the predicates the frontier narrows on ───────────────────────────────────

// An Issue is RESOLVED (computed, never stored) iff a #Resolves resolver is COMPLETE — an action
// in `done` OR a Decision with status=accepted; else OPEN. An issue with no #Resolves edge is OPEN
// AND untriaged. `done` is supplied by orient (the single done-set authority).

pub struct ResolverStatus {
    pub name: String,
    pub kind: &'static str, // "action" | "decision"
    pub complete: bool,
}

pub struct IssueStatus {
    pub issue: String,
    pub resolvers: Vec<ResolverStatus>,
    pub open: bool,
}

/// The latest recorded disposition verdict on a finding Issue (D0092): the `disposition` attr of a
/// `#Dispositions`-linked confirmation Test (`act` | `acceptRisk` | `dismiss`), or `None` if
/// undispositioned. Reads the TYPED verdict — not a prose/proxy inference.
#[must_use]
pub fn issue_disposition(model: &Model, issue: &str) -> Option<String> {
    model
        .edges
        .iter()
        .filter(|e| e.kind == "dispositions" && e.to == issue)
        .filter_map(|e| model.items.get(&e.from).and_then(|t| t.attrs.get("disposition")).cloned())
        .next_back()
}

pub fn compute_issue_resolution<S: std::hash::BuildHasher>(model: &Model, done: &HashSet<String, S>) -> Vec<IssueStatus> {
    let mut issues: Vec<&String> = model.items.iter().filter(|(_, i)| i.type_name == "Issue").map(|(n, _)| n).collect();
    issues.sort();
    // A resolving Decision completes the Issue only while it is accepted AND in force (D0398).
    let accepted_decisions = model.standing("accepted");
    issues
        .into_iter()
        .map(|iss| {
            let mut resolvers: Vec<ResolverStatus> = model
                .edges
                .iter()
                .filter(|e| e.kind == "resolves" && &e.to == iss)
                .map(|e| {
                    let is_decision = model.items.get(&e.from).is_some_and(|i| i.type_name == "Decision");
                    let complete = if is_decision {
                        accepted_decisions.contains(&e.from)
                    } else {
                        done.contains(e.from.as_str())
                    };
                    ResolverStatus { name: e.from.clone(), kind: if is_decision { "decision" } else { "action" }, complete }
                })
                .collect();
            // D0092: an ACCEPT-RISK or DISMISS disposition CLOSES the issue on its own (the verdict IS
            // the resolution); ACT does not — it still needs its #Resolves resolver done.
            if let Some(v) = issue_disposition(model, iss) {
                if v == "acceptRisk" || v == "dismiss" {
                    resolvers.push(ResolverStatus { name: format!("disposition:{v}"), kind: "disposition", complete: true });
                }
            }
            resolvers.sort_by(|a, b| a.name.cmp(&b.name));
            let open = !resolvers.iter().any(|r| r.complete);
            IssueStatus { issue: iss.clone(), resolvers, open }
        })
        .collect()
}

/// Every Issue this project HOLDS, open or resolved (D0278: the control-defect registry needs to tell
/// "absent — tracked upstream" from "present and resolved — a stale entry").
///
/// # Errors
/// Returns [`ViewError`] if a tracking/instance file fails to parse.
pub fn all_issue_names<S: std::hash::BuildHasher>(root: &Path, done: &HashSet<String, S>) -> Result<Vec<String>, ViewError> {
    let model = Model::build(root)?;
    Ok(compute_issue_resolution(&model, done).into_iter().map(|i| i.issue).collect())
}

/// Names of OPEN issues (no complete `#Resolves` resolver), sorted. Used by orient to surface
/// `open_issues`. `done` is orient's done-set.
///
/// # Errors
/// Returns [`ViewError`] if a tracking/instance file fails to parse.
pub fn open_issue_names<S: std::hash::BuildHasher>(root: &Path, done: &HashSet<String, S>) -> Result<Vec<String>, ViewError> {
    let model = Model::build(root)?;
    Ok(compute_issue_resolution(&model, done).into_iter().filter(|i| i.open).map(|i| i.issue).collect())
}

/// Names that are the TARGET of a `#Supersede` edge — i.e. deliberately retired (§1.4).
///
/// Exists because `ready` ignored supersede entirely (issue100): a task recorded as superseded stayed
/// on the ranked frontier forever, so the authored fact said "retired" while the computed view said
/// "do this next". Since the AI auto-follows the frontier (D0052), that is not a cosmetic
/// disagreement — it actively schedules work a Decision has forbidden. Found when D0140 superseded two
/// migration items that would have silently deleted 493 edges, and they stayed ready.
///
/// # Errors
/// Returns [`ViewError`] if a tracking/instance file fails to parse.
pub fn superseded_names(root: &Path) -> Result<HashSet<String>, ViewError> {
    let model = Model::build(root)?;
    Ok(model.edges.iter().filter(|e| e.kind == "supersede").map(|e| e.to.clone()).collect())
}

/// Decisions with `status = proposed` — the ones WAITING on the single human gate (issue096).
///
/// Acceptance is the one human gate in an otherwise autonomous loop, and the console's Decisions
/// surface was wired to the `keel decisions` SCORECARD, which filters to accepted and therefore
/// rendered everything except what needs action. It failed silently too: nothing anywhere stated
/// that decisions were waiting, so an unattended proposal is indistinguishable from none.
///
/// Sourced from the model rather than from the scorecard on purpose. The scorecard's accepted-only
/// scope is CORRECT for what it does — scoring citations and critique coverage is meaningful only
/// for a committed decision — so this reads the same authored facts by a different question instead
/// of widening a view that is right as it stands.
///
/// # Errors
/// Returns [`ViewError`] if a tracking/instance file fails to parse.
pub fn pending_acceptances(root: &Path) -> Result<Vec<String>, ViewError> {
    Ok(proposed_decisions(&*Model::build(root)?))
}

/// Is `name` a declared item in the model?
///
/// Exists so a write path can REFUSE before authoring rather than leave the `edge-endpoints` guard
/// to catch it afterwards: `keel record issue` must produce a triaged Issue whose `#Resolves` edge
/// actually lands somewhere, and an edge to a name declared nowhere is worse than a missing edge
/// because every consumer treats it as present (issue109).
///
/// # Errors
/// Returns [`ViewError`] if a tracking/instance file fails to parse.
pub fn item_exists(root: &Path, name: &str) -> Result<bool, ViewError> {
    Ok(Model::build(root)?.items.contains_key(name))
}

/// Pure core of [`pending_acceptances`], for self-test.
///
/// Matches on the suffix because the authored value is the enum path `DecisionStatus::proposed`,
/// and matching the bare word would also catch a status like `counterproposed` if one were ever
/// added — while matching the full path would silently stop working if the enum were renamed.
#[must_use]
pub fn proposed_decisions(model: &Model) -> Vec<String> {
    let mut pending: Vec<String> = model.standing("proposed").into_iter().cloned().collect();
    pending.sort();
    pending
}

/// Task names blocked on a human acceptance: they `#DependsOn` a Decision that is still `proposed`
/// (issue112).
///
/// The frontier is AUTO-FOLLOWED (D0052), so an item it ranks is an item the next contributor will
/// start. `dcWorkClaim` needs a `Claim` type that only a human can sign into frozen core, and it
/// nonetheless ranked FIRST — so a successor would rediscover the wall this sprint just hit, and the
/// computed view would have told them the work was ready when it provably was not.
///
/// This is the same defect as issue100 (a superseded task staying ready) with a different cause, and
/// it needs its own predicate: superseded means RETIRED, blocked-on-acceptance means WAITING, and
/// conflating them would either hide work that resumes the moment a human answers, or retire it.
///
/// Exact, not heuristic: a `#DependsOn` edge to a Decision whose `status` is `proposed`. An accepted
/// or rejected Decision unblocks the item with no further edit, because nothing here is stored.
///
/// A `#CharteredBy` edge to a proposed Decision blocks the same way (issue552): the charter IS the
/// authority the item acts under, and an item chartered by a Decision the human has not accepted was
/// ranked first while the acceptance it needed sat on the queue.
///
/// # Errors
/// Returns [`ViewError`] if a tracking/instance file fails to parse.
pub fn blocked_on_acceptance(root: &Path) -> Result<HashSet<String>, ViewError> {
    Ok(blocked_by(&*Model::build(root)?))
}

/// Pure core of [`blocked_on_acceptance`], for self-test.
#[must_use]
pub fn blocked_by(model: &Model) -> HashSet<String> {
    let pending: HashSet<&String> = model.standing("proposed");
    model
        .edges
        .iter()
        .filter(|e| e.kind == "dependson" || e.kind == "dependency" || e.kind == "charteredby")
        .filter(|e| pending.contains(&e.to))
        .map(|e| e.from.clone())
        .collect()
}

/// Work items waiting on ANOTHER work item that is not done (issue549).
///
/// Each row is `(item, waitsOn, why)`. `done` is the frontier's done-set and `is_item` says whether a
/// name is a tracked work item at all - both are the caller's (orient's) authority, so this reads only
/// the edges.
///
/// Until this existed the ready set honoured `first A then B` successions and Decision edges, and
/// nineteen item-to-item `#DependsOn` edges declared in the backlog were read by nothing: every member
/// of the layering chain ranked ready at once, with its predecessor still undone. An edge to a
/// superseded item is stated as such rather than treated as satisfied - retired work never completes,
/// so a dependant of it is blocked until a Decision re-points or retires it too.
///
/// # Errors
/// Returns [`ViewError`] if a tracking/instance file fails to parse.
pub fn blocked_on_items<S: std::hash::BuildHasher>(
    root: &Path,
    done: &HashSet<String, S>,
    is_item: &dyn Fn(&str) -> bool,
) -> Result<Vec<(String, String, String)>, ViewError> {
    Ok(blocked_by_items(&*Model::build(root)?, done, is_item))
}

/// Pure core of [`blocked_on_items`], for self-test.
pub fn blocked_by_items<S: std::hash::BuildHasher>(
    model: &Model,
    done: &HashSet<String, S>,
    is_item: &dyn Fn(&str) -> bool,
) -> Vec<(String, String, String)> {
    let retired = model.retired();
    let mut out: Vec<(String, String, String)> = model
        .edges
        .iter()
        .filter(|e| e.kind == "dependson" && is_item(&e.from) && is_item(&e.to))
        .filter_map(|e| {
            if retired.contains(&e.to) {
                Some((e.from.clone(), e.to.clone(), "superseded".to_string()))
            } else if done.contains(&e.to) {
                None
            } else {
                Some((e.from.clone(), e.to.clone(), "not done".to_string()))
            }
        })
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Every `#Resolves` edge as `(resolver, issue, resolver_type)`.
///
/// `resolver_type` is the declared item type, or `""` when the resolver is not a typed item — which is
/// the normal case, since most resolvers are `action` names.
///
/// Feeds `guard resolver-kind`. Separate from [`untriaged_issues`] on purpose: that answers whether an
/// edge EXISTS, and this answers whether the thing on the other end could resolve anything.
///
/// # Errors
/// Returns [`ViewError`] if a tracking/instance file fails to parse.
pub fn resolves_edges(root: &Path) -> Result<Vec<(String, String, String)>, ViewError> {
    let model = Model::build(root)?;
    let mut out: Vec<(String, String, String)> = model
        .edges
        .iter()
        .filter(|e| e.kind == "resolves")
        .map(|e| (e.from.clone(), e.to.clone(), model.items.get(&e.from).map(|i| i.type_name.clone()).unwrap_or_default()))
        .collect();
    out.sort();
    Ok(out)
}

/// `(total_issues, untriaged)` — issues with NO `#Resolves` edge at all (D0077). Pure structure
/// (no done-set needed); the `issues` guard fails on a non-empty untriaged list.
///
/// # Errors
/// Returns [`ViewError`] if a tracking/instance file fails to parse.
pub fn untriaged_issues(root: &Path) -> Result<(usize, Vec<String>), ViewError> {
    let model = Model::build(root)?;
    let issues: Vec<&String> = model.items.iter().filter(|(_, i)| i.type_name == "Issue").map(|(n, _)| n).collect();
    let mut untriaged: Vec<String> = issues
        .iter()
        .filter(|n| !model.edges.iter().any(|e| e.kind == "resolves" && &e.to == **n))
        .map(|n| (*n).clone())
        .collect();
    untriaged.sort();
    Ok((issues.len(), untriaged))
}

/// Whole days between two ISO dates, or 0 if either is unparseable.
///
/// Dates only: the model records dates, and reporting a finer resolution than the data carries would
/// be false precision. Uses the standard civil-date algorithm rather than a dependency.
#[must_use]
pub fn days_between(from: &str, to: &str) -> i64 {
    let parse = |s: &str| -> Option<(i64, i64, i64)> {
        let mut it = s.split('-');
        Some((it.next()?.parse().ok()?, it.next()?.parse().ok()?, it.next()?.parse().ok()?))
    };
    let to_days = |(y, m, d): (i64, i64, i64)| -> i64 {
        let y = if m <= 2 { y - 1 } else { y };
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = y - era * 400;
        let mp = (m + 9) % 12;
        let doy = (153 * mp + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    };
    match (parse(from), parse(to)) {
        (Some(a), Some(b)) => to_days(b) - to_days(a),
        _ => 0,
    }
}

/// "Now", taken from git rather than the wall clock (D0013): the HEAD commit date.
///
/// Deterministic — two contributors computing this queue against the same commit get the same ages,
/// which a clock would not give them.
#[must_use]
pub fn repo_today(root: &Path) -> String {
    keel_git::gitx::git()
        .arg("-C")
        .arg(root)
        .args(["log", "-1", "--format=%cs"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_owned())
        .filter(|s| s.len() == 10)
        .unwrap_or_default()
}

/// One claim as authored: `(name, item, by, at, against)`. Liveness is NOT here — it is computed.
pub type ClaimRow = (String, String, String, String, String);

/// Raw claim tuples from the model, for `claim::claims`.
///
/// # Errors
/// Returns [`ViewError`] if a tracking file fails to parse.
pub fn claim_rows(root: &Path) -> Result<Vec<ClaimRow>, ViewError> {
    let model = Model::build(root)?;
    let mut out: Vec<ClaimRow> = model
        .items
        .iter()
        .filter(|(_, i)| i.type_name == "Claim")
        .map(|(n, i)| {
            let g = |k: &str| i.attrs.get(k).cloned().unwrap_or_default();
            (n.clone(), g("claimedItem"), g("claimedBy"), g("claimedAt"), g("claimedAgainst"))
        })
        .collect();
    out.sort();
    Ok(out)
}

/// Claim name -> its element id (a UUID), for the claim holder tie-break.
///
/// # Errors
/// Returns [`ViewError`] if a tracking file fails to parse.
pub fn claim_ids(root: &Path) -> Result<HashMap<String, String>, ViewError> {
    let model = Model::build(root)?;
    Ok(model
        .items
        .iter()
        .filter(|(_, i)| i.type_name == "Claim")
        .map(|(n, i)| (n.clone(), i.attrs.get("id").cloned().unwrap_or_default()))
        .collect())
}

#[cfg(test)]
mod narrowing_tests {
    use super::*;
    use crate::model::{Edge, ItemInfo};
    use std::collections::{HashMap, HashSet};

    #[test]
    fn issue_resolution_open_vs_resolved() {
        // i1 resolved by a done action; i2 open (resolver action not done); i3 untriaged (no edge).
        let mut items = HashMap::new();
        for n in ["i1", "i2", "i3"] {
            items.insert(n.to_string(), ItemInfo { type_name: "Issue".to_string(), attrs: HashMap::new(), marker: None, file: String::new() });
        }
        items.insert("actDone".to_string(), ItemInfo { type_name: "action".to_string(), attrs: HashMap::new(), marker: None, file: String::new() });
        items.insert("actOpen".to_string(), ItemInfo { type_name: "action".to_string(), attrs: HashMap::new(), marker: None, file: String::new() });
        let edges = vec![
            Edge { kind: "resolves".to_string(), from: "actDone".to_string(), to: "i1".to_string() },
            Edge { kind: "resolves".to_string(), from: "actOpen".to_string(), to: "i2".to_string() },
        ];
        let model = Model { items, edges };
        let done: HashSet<String> = std::iter::once("actDone".to_string()).collect();
        let res = compute_issue_resolution(&model, &done);
        let open: Vec<&str> = res.iter().filter(|i| i.open).map(|i| i.issue.as_str()).collect();
        assert_eq!(open, vec!["i2", "i3"]); // i1 resolved; i2 + i3 open
        let i3 = res.iter().find(|i| i.issue == "i3").unwrap();
        assert!(i3.resolvers.is_empty(), "i3 is untriaged");
    }

    #[test]
    fn issue_resolved_by_accepted_decision() {
        let mut items = HashMap::new();
        items.insert("i9".to_string(), ItemInfo { type_name: "Issue".to_string(), attrs: HashMap::new(), marker: None, file: String::new() });
        let mut dattrs = HashMap::new();
        dattrs.insert("status".to_string(), "accepted".to_string());
        items.insert("d99".to_string(), ItemInfo { type_name: "Decision".to_string(), attrs: dattrs, marker: None, file: String::new() });
        let edges = vec![Edge { kind: "resolves".to_string(), from: "d99".to_string(), to: "i9".to_string() }];
        let model = Model { items, edges };
        let res = compute_issue_resolution(&model, &HashSet::new());
        assert!(!res[0].open, "accepted Decision resolves the issue");
        assert_eq!(res[0].resolvers[0].kind, "decision");
    }

    #[test]
    fn pending_acceptances_are_the_proposed_decisions_only() {
        // issue096: the console rendered the accepted-only scorecard, so it showed everything EXCEPT
        // what needs the human. Only `proposed` is waiting — rejected and retired are settled, and
        // counting them would recreate the same uselessness from the other direction. Retirement is
        // the `#Supersede` EDGE, not a status value (D0398/issue396): d0004 still READS proposed and
        // is absent because d0005 retired it.
        let with_status = |ty: &str, status: &str| {
            let mut a = HashMap::new();
            a.insert("status".to_string(), status.to_string());
            ItemInfo { type_name: ty.to_string(), attrs: a, marker: None, file: String::new() }
        };
        let mut items = HashMap::new();
        items.insert("d0002".to_string(), with_status("Decision", "DecisionStatus::proposed"));
        items.insert("d0001".to_string(), with_status("Decision", "DecisionStatus::accepted"));
        items.insert("d0003".to_string(), with_status("Decision", "DecisionStatus::rejected"));
        items.insert("d0004".to_string(), with_status("Decision", "DecisionStatus::proposed"));
        items.insert("d0005".to_string(), with_status("Decision", "DecisionStatus::proposed"));
        // A non-Decision carrying the same attribute must not leak in.
        items.insert("someStory".to_string(), with_status("Story", "DecisionStatus::proposed"));
        let edges = vec![Edge { kind: "supersede".to_string(), from: "d0005".to_string(), to: "d0004".to_string() }];
        let model = Model { items, edges };
        assert_eq!(proposed_decisions(&model), vec!["d0002".to_string(), "d0005".to_string()]);

        // And the empty case returns an EMPTY list rather than anything absent: orient always emits
        // the field, because a field that vanishes when empty is indistinguishable from one nobody
        // computed (the D0138 lesson).
        let mut only_accepted = HashMap::new();
        only_accepted.insert("d0001".to_string(), with_status("Decision", "DecisionStatus::accepted"));
        assert!(proposed_decisions(&Model { items: only_accepted, edges: Vec::new() }).is_empty());
    }

    #[test]
    fn a_task_depending_on_a_proposed_decision_is_blocked_and_unblocks_by_itself() {
        // issue112: the frontier is auto-followed (D0052), so ranking an item that cannot be started
        // points the next contributor at a wall. Distinct from superseded — this is WAITING, not
        // retired, and it must clear with no edit once the human answers.
        let with_status = |ty: &str, status: &str| {
            let mut a = HashMap::new();
            a.insert("status".to_string(), status.to_string());
            ItemInfo { type_name: ty.to_string(), attrs: a, marker: None, file: String::new() }
        };
        let mut items = HashMap::new();
        items.insert("dPending".to_string(), with_status("Decision", "DecisionStatus::proposed"));
        items.insert("dSettled".to_string(), with_status("Decision", "DecisionStatus::accepted"));
        let edges = vec![
            Edge { kind: "dependson".to_string(), from: "blockedTask".to_string(), to: "dPending".to_string() },
            Edge { kind: "dependson".to_string(), from: "freeTask".to_string(), to: "dSettled".to_string() },
        ];
        let model = Model { items: items.clone(), edges: edges.clone() };
        let blocked = blocked_by(&model);
        assert!(blocked.contains("blockedTask"), "{blocked:?}");
        assert!(!blocked.contains("freeTask"), "an ACCEPTED decision blocks nothing: {blocked:?}");

        // Accepting the decision unblocks the task with no other edit — nothing is stored.
        let mut accepted = items;
        accepted.insert("dPending".to_string(), with_status("Decision", "DecisionStatus::accepted"));
        assert!(blocked_by(&Model { items: accepted, edges }).is_empty());
    }

    #[test]
    fn a_story_chartered_by_a_proposed_decision_is_blocked_until_it_is_accepted() {
        // issue552: dcWorkspaceLayeringIsGuarded was chartered by a HELD Decision and ranked ready;
        // the charter is the authority the item acts under, so it blocks exactly as #DependsOn does.
        let with_status = |status: &str| {
            let mut a = HashMap::new();
            a.insert("status".to_string(), status.to_string());
            ItemInfo { type_name: "Decision".to_string(), attrs: a, marker: None, file: String::new() }
        };
        let mut items = HashMap::new();
        items.insert("dHeld".to_string(), with_status("DecisionStatus::proposed"));
        items.insert("dStanding".to_string(), with_status("DecisionStatus::accepted"));
        let edges = vec![
            Edge { kind: "charteredby".to_string(), from: "storyHeld".to_string(), to: "dHeld".to_string() },
            Edge { kind: "charteredby".to_string(), from: "storyFree".to_string(), to: "dStanding".to_string() },
        ];
        let blocked = blocked_by(&Model { items: items.clone(), edges: edges.clone() });
        assert!(blocked.contains("storyHeld"), "{blocked:?}");
        assert!(!blocked.contains("storyFree"), "a charter that STANDS blocks nothing: {blocked:?}");
        let mut accepted = items;
        accepted.insert("dHeld".to_string(), with_status("DecisionStatus::accepted"));
        assert!(blocked_by(&Model { items: accepted, edges }).is_empty(), "acceptance alone unblocks it");
    }

    #[test]
    fn an_item_depending_on_an_undone_item_is_blocked_and_says_what_it_waits_on() {
        // issue549: A #DependsOn B with B undone lists A blocked on B; B done frees A; a superseded B
        // blocks A and says so; an edge to a Decision is not this filter's business.
        let plain = |ty: &str| ItemInfo { type_name: ty.to_string(), attrs: HashMap::new(), marker: None, file: String::new() };
        let mut items = HashMap::new();
        for (n, t) in [("a", "action"), ("b", "action"), ("c", "action"), ("gone", "action"), ("d1", "Decision")] {
            items.insert(n.to_string(), plain(t));
        }
        let edges = vec![
            Edge { kind: "dependson".to_string(), from: "a".to_string(), to: "b".to_string() },
            Edge { kind: "dependson".to_string(), from: "c".to_string(), to: "gone".to_string() },
            Edge { kind: "supersede".to_string(), from: "d1".to_string(), to: "gone".to_string() },
            Edge { kind: "dependson".to_string(), from: "b".to_string(), to: "d1".to_string() },
        ];
        let model = Model { items, edges };
        let is_item = |n: &str| n != "d1";
        let done: HashSet<String> = HashSet::new();
        let blocked = blocked_by_items(&model, &done, &is_item);
        assert_eq!(
            blocked,
            vec![
                ("a".to_string(), "b".to_string(), "not done".to_string()),
                ("c".to_string(), "gone".to_string(), "superseded".to_string()),
            ],
            "b's Decision edge is the acceptance filter's, not this one's"
        );
        let done: HashSet<String> = std::iter::once("b".to_string()).collect();
        let blocked = blocked_by_items(&model, &done, &is_item);
        assert_eq!(blocked.len(), 1, "b done frees a; the superseded edge still blocks c: {blocked:?}");
        assert_eq!(blocked[0].0, "c");
    }

}

#[cfg(test)]
mod declared_tests {
    use super::is_declared;

    /// THE CONTROL for issue177. Both directions, because the interesting failure is the FALSE NEGATIVE:
    /// my first scanner stopped at `:` and so kept the `;` from a bare `action foo;`, which made every
    /// backlog action look undeclared and broke `keel show item` for the most common item in this model.
    /// The repository root, found from the crate manifest: a member's cwd under `cargo test` is its
    /// own directory two levels down, so `..` and `src/...` no longer name this repo (sprint 714, 718).
    fn repo_root() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .find(|a| a.join(".git").exists())
            .expect("a member crate sits inside the keel repository")
            .to_path_buf()
    }

    #[test]
    fn a_bare_action_and_a_typed_part_are_both_declared() {
        // The repo root, not `.`: a unit test's cwd is the CRATE dir, which has no `.tracking` at
        // all - the scan would return false for everything and the test would pass vacuously in the
        // false-negative direction while failing in the true one.
        let root = &repo_root();
        assert!(is_declared(root, "dcUnknownNameFails"), "a bare `action foo;` must resolve");
        assert!(is_declared(root, "hardening1Story"), "a typed `part foo : Story` declaration must resolve");
        assert!(!is_declared(root, "."), "`.` is a path, not an item - the bug that started issue177");
        assert!(!is_declared(root, "definitely-not-an-item-anywhere"));
    }
}
