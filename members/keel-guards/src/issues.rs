//! Guard family `issues` - split from `keel-cli/src/guards.rs` by `scripts/split_guards.py` (sprint 733).
//!
//! Each guard's dispatch arm, code and tests sit together; the shared scanners, the runner and the
//! lock predicates are the crate root's (`super`). Nothing here was retyped: the text is guards.rs's,
//! with `crate::` paths pointing at the members and private items opened to the crate.

use super::*;

/// The `issues` family: every guard it dispatches, in `GUARD_NAMES` order, with the tier note each
/// arm carried in `run_one` (sprint 733). The root's union test holds these tables equal to `GUARD_NAMES`.
pub(crate) const FAMILY: Family = Family {
    name: "issues",
    arms: &[
        ("issues", issues),
        ("resolver-kind", resolver_kind),
        ("untrusted-routing", untrusted_routing),
        ("control-defect-registry", control_defect_registry),
        ("untrusted-taint", untrusted_taint), // hard (issue347/D0314) - the untrusted label travels through derivation
    ],
};

/// Guard: work descended from an UNTRUSTED utterance must be routed through a Decision (D0264).
///
/// An issue on a public tracker is an instruction from an unauthenticated stranger. Triaging it is
/// fine — reading is not obeying — but routing it straight to an implementation task means the
/// project acts on a stranger's instruction with nobody having agreed to it. That is prompt
/// injection with a filing form, and the defence is not detection but ROUTING: untrusted input may
/// produce a plan and a proposed Decision, and a human accepts before anything is built.
///
/// SCOPE, deliberately narrow. This fires only on a story that HAS been routed (`#Implicates`) and
/// whose targets contain no Decision. An unrouted story is untriaged, not a violation — the guard
/// must not punish work-in-progress, which is how a control gets bypassed instead of obeyed.
#[must_use]
pub fn untrusted_routing(root: &Path) -> GuardReport {
    let blob = keel_model::corpus::collect_sysml(&root.join(".tracking"))
        .iter()
        .filter_map(|p| keel_model::corpus::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n");
    // Statements whose recorded tier is `untrusted`.
    let untrusted: HashSet<String> = blob
        .split("part ")
        .skip(1)
        .filter(|seg| seg.contains("SourceTrust::untrusted"))
        .filter_map(|seg| seg.split_whitespace().next().map(str::to_string))
        .collect();
    // Stories deriving from one of them.
    let mut tainted: HashSet<String> = HashSet::new();
    for line in blob.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("#DerivedFrom dependency from ") {
            if let Some((story, target)) = rest.split_once(" to ") {
                if untrusted.contains(target.trim_end_matches(';').trim()) {
                    tainted.insert(story.trim().to_string());
                }
            }
        }
    }
    // Of those, the ones ROUTED somewhere, and whether any target is a Decision.
    let mut routed: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
    for line in blob.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("#Implicates dependency from ") {
            if let Some((story, target)) = rest.split_once(" to ") {
                let story = story.trim().to_string();
                if tainted.contains(&story) {
                    routed.entry(story).or_default().push(target.trim_end_matches(';').trim().to_string());
                }
            }
        }
    }
    let is_decision = |t: &String| {
        t.len() >= 5 && t.starts_with('d') && t[1..5].chars().all(|c| c.is_ascii_digit())
    };
    let violations: Vec<String> = routed
        .iter()
        .filter(|(_, targets)| !targets.iter().any(is_decision))
        .map(|(story, targets)| {
            format!(
                "{story} descends from an UNTRUSTED utterance and is routed to {targets:?} with no \
                 Decision among them — untrusted input may PLAN, and a human accepts before anything \
                 is implemented (D0264). Propose a Decision and route the story to it."
            )
        })
        .collect();
    // SCANNED is the population POLICED - untrusted utterances - not the subset currently in
    // violation. Reporting the tainted-story count instead read 0 on a tree holding four untrusted
    // statements, which the liveness meta-test rightly rejected: a guard whose population is always
    // zero has no signal distinguishing "nothing to police" from "mis-aimed" (issue180).
    GuardReport { name: "untrusted-routing", scanned: untrusted.len(), warnings: Vec::new(), violations }
}

/// Guard: nothing transitively derived from an UNTRUSTED utterance reaches a privileged sink without
/// a human having accepted it on the way.
///
/// Guard 56 (`untrusted-routing`) enforces the first hop: an untrusted story must be routed to a
/// Decision. It does not follow the label further. A `UserStory` derived from an untrusted
/// `Statement`, a `Need` derived from that story, a task chartered by a Decision that story
/// implicated - three hops and the label is gone, and the task builds (the `CaMeL` finding, D0282).
///
/// This guard computes the TAINT CLOSURE over the derivation edges - `#DerivedFrom` (target to
/// source), `#Implicates` (story to target), `#CharteredBy` and `#Resolves` (item to what it serves)
/// - from every `SourceTrust::untrusted` utterance, and fails when a tainted item is
///
/// - a Decision that is ACCEPTED, but only automatically (standing consent, D0291) - an instruction
///   from an unauthenticated stranger became an accepted direction with nobody reading it; or
/// - a task that is DONE (a passing result) - built on that instruction.
///
/// Trust is conferred at exactly one point: a Decision a HUMAN accepted - their quoted word, the
/// console, a terminal. Propagation stops there; what descends from it is the human's direction. A
/// proposed Decision is a plan, and planning is what untrusted input may do (D0264), so it is never a
/// violation on its own.
///
/// STATED RESIDUAL: a guard, contract or hook edit is not a model item and carries no edge, so a
/// tainted task that changes the enforcement surface is caught here only as a done task; the keystone
/// lock (D0209) is what requires its Decision.
#[must_use]
pub fn untrusted_taint(root: &Path) -> GuardReport {
    let blob = keel_model::corpus::collect_sysml(&root.join(".tracking"))
        .iter()
        .filter_map(|p| keel_model::corpus::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n");
    let deciders: HashSet<String> = keel_github::github::deciders(root).into_keys().collect();
    let (scanned, sources) = untrusted_sources(&blob, &deciders);
    let edges = taint_edges(&blob);
    let accepted = decision_acceptances(root);
    let done = keel_model::done::done_names(root);
    let violations = taint_violations(&sources, &edges, &accepted, &done);
    GuardReport { name: "untrusted-taint", scanned, warnings: Vec::new(), violations }
}

/// The untrusted utterances (count) and the ones whose speaker is NOT a declared decider (sources).
///
/// `sourceTrust` is a proxy from repository visibility and stays as recorded (D0264): the human's
/// own GitHub issues on this public repository carry `untrusted`. But `saidBy` is the GitHub login
/// the API attributed the words to, and `github-actors.toml` is the committed table that maps a
/// login to the human whose judgment it is (D0219) - the same table the decision channel trusts to
/// record an acceptance. A decider's own words are the human's direction, not a stranger's
/// instruction: trust is conferred at the source. The first live run found exactly this - three
/// violations, every one an issue the decider filed themself.
pub(crate) fn untrusted_sources(blob: &str, deciders: &HashSet<String>) -> (usize, Vec<String>) {
    let mut scanned = 0usize;
    let mut sources = Vec::new();
    for seg in blob.split("part ").skip(1).filter(|seg| seg.contains("SourceTrust::untrusted")) {
        let Some(name) = seg.split_whitespace().next() else { continue };
        scanned += 1;
        let said_by = seg.split(":>> saidBy = \"").nth(1).and_then(|r| r.split('"').next()).unwrap_or("");
        if !deciders.contains(said_by) {
            sources.push(name.to_string());
        }
    }
    (scanned, sources)
}

/// Every derivation edge as (from, to) in the DIRECTION TAINT FLOWS, with the edge's kind for the path.
pub(crate) fn taint_edges(blob: &str) -> Vec<(String, String, &'static str)> {
    let mut out = Vec::new();
    for line in blob.lines() {
        let t = line.trim();
        // (marker, taint flows from the edge's `to` to its `from`)
        for (marker, reversed) in [("#DerivedFrom", true), ("#Implicates", false), ("#CharteredBy", true), ("#Resolves", true)] {
            let Some(rest) = t.strip_prefix(marker).and_then(|r| r.strip_prefix(" dependency from ")) else { continue };
            let Some((a, b)) = rest.split_once(" to ") else { continue };
            let a = a.trim().to_string();
            let b = b.trim_end_matches(';').trim().to_string();
            if reversed { out.push((b, a, marker)) } else { out.push((a, b, marker)) }
        }
    }
    out
}

/// Every accepted Decision under `.engine/decisions`, with how it was accepted.
pub(crate) fn decision_acceptances(root: &Path) -> HashMap<String, Acceptance> {
    let mut out = HashMap::new();
    for path in keel_model::corpus::collect_sysml(&root.join(".engine").join("decisions")) {
        let Ok(text) = keel_model::corpus::read_to_string(&path) else { continue };
        let Some(name) = path.file_name().and_then(|n| n.to_str()).and_then(|n| n.get(..4)).filter(|n| n.chars().all(|c| c.is_ascii_digit())) else { continue };
        let dname = format!("d{name}");
        if let Some(kind) = acceptance_kind(&text, &dname) {
            out.insert(dname, kind);
        }
    }
    out
}

/// The closure and its verdicts. Breadth-first from every source, carrying the path; a HUMAN-accepted
/// Decision is where trust is conferred and the walk stops; an AUTO-accepted Decision or a DONE task
/// reached with the label still on it is a violation naming the whole path.
pub(crate) fn taint_violations(
    sources: &[String],
    edges: &[(String, String, &'static str)],
    accepted: &HashMap<String, Acceptance>,
    done: &HashSet<String>,
) -> Vec<String> {
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    for src in sources {
        let mut seen: HashSet<&str> = HashSet::new();
        let mut queue: std::collections::VecDeque<(String, Vec<String>)> = std::collections::VecDeque::new();
        queue.push_back((src.clone(), vec![src.clone()]));
        while let Some((node, path)) = queue.pop_front() {
            for (from, to, kind) in edges.iter().filter(|(f, _, _)| *f == node) {
                if !seen.insert(to.as_str()) {
                    continue;
                }
                let mut p = path.clone();
                p.push(format!("{kind}-> {to}"));
                let _ = from;
                match accepted.get(to) {
                    Some(Acceptance::Human) => continue, // trust conferred here; nothing below is tainted
                    Some(Acceptance::Auto) => {
                        out.entry(to.clone()).or_insert_with(|| format!(
                            "{to} is an ACCEPTED Decision descended from the UNTRUSTED utterance {src} and its acceptance was automatic (standing consent) - an unauthenticated stranger's instruction became direction with nobody reading it; a human accepts it in their own words (D0289) or it stays proposed. Path: {}",
                            p.join(" ")
                        ));
                        // nobody read it, so nothing below it is the human's: keep walking
                    }
                    None => {}
                }
                if done.contains(to) {
                    out.entry(to.clone()).or_insert_with(|| format!(
                        "{to} is DONE and descends from the UNTRUSTED utterance {src} with no human acceptance on the path - built on an unauthenticated stranger's instruction (D0264). Route the chain through a Decision a human accepts. Path: {}",
                        p.join(" ")
                    ));
                }
                queue.push_back((to.clone(), p));
            }
        }
    }
    out.into_values().collect()
}

/// Guard (D0278): the control-defect registry names real controls and still-open Issues.
///
/// # Why the registry needs its own guard
///
/// `.engine/contracts/control-defects.toml` makes a defective control announce itself beside its own
/// verdict, which only works while the entries are true. Two ways it rots, both silent:
///
/// - A typo'd or retired CONTROL NAME. The entry then belongs to nothing, so the announcement never
///   prints and a known-broken guard goes back to handing out unqualified greens — the exact state
///   the registry was built to end, restored without anyone noticing.
/// - An Issue that has since been RESOLVED. The announcement then keeps qualifying verdicts that are
///   now sound, which is how a true warning becomes noise and then becomes scrolled past (D0214).
///   The process says the entry is removed in the same commit that resolves the Issue; this is what
///   makes that a rule rather than a hope.
///
/// The direction is checked too, because `note()` picks its wording from it: a mistyped direction
/// would tell the reader a green is untrustworthy when the defect is over-reporting, or worse, the
/// reverse.
#[must_use]
pub fn control_defect_registry(root: &Path) -> GuardReport {
    let entries = keel_schema::control_defects::load(root);
    if entries.is_empty() {
        return GuardReport { name: "control-defect-registry", scanned: 0, warnings: Vec::new(), violations: Vec::new() };
    }
    let done = keel_model::done::done_names(root);
    let open: std::collections::HashSet<String> = match keel_view::view::open_issue_names(root, &done) {
        Ok(v) => v.into_iter().collect(),
        Err(e) => {
            return GuardReport {
                name: "control-defect-registry",
                scanned: entries.len(),
                warnings: Vec::new(),
                violations: vec![format!("cannot read issue resolution to check the registry: {e}")],
            };
        }
    };
    // Every Issue this project HAS, open or closed. The registry ships with the engine into every
    // `keel init` project (the defects are the ENGINE's guards, so the note is true downstream), but
    // the Issues it cites live in the engine repository's own .tracking. A downstream project sees
    // the entry and lacks the Issue — found by init_smoke going red on a fresh scaffold. So an id
    // that is ABSENT here is "tracked upstream" and a warning; an id that is PRESENT and CLOSED is
    // the rot the guard exists to catch and stays a violation.
    let known: std::collections::HashSet<String> =
        keel_view::view::all_issue_names(root, &done).unwrap_or_default().into_iter().collect();
    let mut violations = Vec::new();
    let mut warnings = Vec::new();
    for (control, d) in &entries {
        if !GUARD_NAMES.contains(&control.as_str()) {
            violations.push(format!(
                "control-defects.toml names `{control}`, which is not a guard in this binary — the defect note would never print, so a control known to be broken would silently go back to giving unqualified verdicts"
            ));
        }
        if d.direction != "over" && d.direction != "under" {
            violations.push(format!(
                "`{control}`: direction `{}` is neither `over` nor `under` — the note's wording is chosen from it, so a wrong value tells the reader to distrust the wrong half of the verdict",
                d.direction
            ));
        }
        if !known.contains(&d.issue) {
            warnings.push(format!(
                "`{control}` is registered against {}, which this project does not hold — the defect is the engine's and is tracked in the engine repository; the note still prints, and this entry is removed by the engine's next resync when it resolves upstream",
                d.issue
            ));
        } else if !open.contains(&d.issue) {
            violations.push(format!(
                "`{control}` is registered against {}, which is RESOLVED — the entry should have been removed in that same commit (D0278). A note that outlives its defect is how a true warning becomes noise",
                d.issue
            ));
        }
    }
    GuardReport { name: "control-defect-registry", scanned: entries.len(), warnings, violations }
}

/// D0304's date: an Issue created on or after it must be NAMED by its resolver's text.
pub(crate) const ISSUE_NAMING_CUTOFF: &str = "2026-09-04";

/// Guard: every Issue carries a `#Resolves` edge (D0077).
///
/// An untriaged issue (no resolver) is a violation — it has no resolving work/Decision and can
/// never compute as resolved. Enforcement (hook wiring + inclusion in the `guard all` set) is
/// turned on once IRL-d backfill triages the existing issues; until then the guard is runnable
/// but not gating.
#[must_use]
pub fn issues(root: &Path) -> GuardReport {
    // CONTRACT (D0107): sourced from the declared issuesTriagedRule (single gate source), not a bespoke predicate.
    match keel_view::view::rule_violations_opt(root, "issuesTriagedRule") {
        Ok(Some((total, untriaged))) => {
            let mut violations: Vec<String> = untriaged
                .into_iter()
                .map(|i| format!("{i}: untriaged — no #Resolves edge (D0077; link a resolving action or Decision)"))
                .collect();
            let mut warnings = Vec::new();
            // issue333 / D0304: an edge that EXISTS is not a triage - the resolver must NAME the issue.
            // Forward-only from D0304's date (the issue068 pattern): 144 of 387 historical resolutions
            // never name their issue and re-triaging them is a sitting review the human may choose
            // (D0204), never owed; they are counted once so the number cannot hide. NOTE issue352: this
            // cutoff is the ENGINE's adoption date and is retroactive for a downstream project that
            // migrates into it - dcForwardOnlyCutoffIsTheProjectsOwn is the open fix for that class.
            match keel_view::view::unnamed_resolutions(root, ISSUE_NAMING_CUTOFF) {
                Ok((_, forward, historical)) => {
                    for (issue, resolver) in forward {
                        violations.push(format!(
                            "{issue}: its resolver `{resolver}` never names it (title, DoD text or decision) - an edge that exists is not a triage; a resolver that will close the issue says so (issue333/D0304). Name the issue in the resolver's DoD, or re-triage to the item that resolves it."
                        ));
                    }
                    if historical > 0 {
                        warnings.push(history_line(&format!(
                            "{historical} resolution(s) recorded before {ISSUE_NAMING_CUTOFF} do not name their issue - history, forward-only from D0304; re-triaging them is a pull-audit the human may choose (D0204), never owed"
                        )));
                    }
                }
                Err(e) => violations.push(format!("error reading resolutions: {e}")),
            }
            GuardReport { name: "issues", scanned: total, warnings, violations }
        }
                // D0136/issue090: an ABSENT rule means the project has not ADOPTED this control —
        // it has not violated it. Warn (never silent, so deleting a rule to dodge the gate is
        // visible) and pass; a MALFORMED rule still fails via Err below.
        Ok(None) => GuardReport { name: "issues", scanned: 0, warnings: vec!["declared rule `issuesTriagedRule` is not present — this control is NOT ADOPTED by this project, so nothing was checked (D0136/issue090)".to_string()], violations: Vec::new() },
Err(e) => GuardReport { name: "issues", scanned: 0, warnings: Vec::new(), violations: vec![format!("error reading issues: {e}")] },
    }
}

/// Guard: a `#Resolves` resolver must be WORK or a mooting `Decision` (D0077/issue136).
///
/// `guard issues` checks only that an Issue HAS a resolving edge, and its own message already says the
/// resolver should be "a resolving action or Decision" — unchecked, so triage passed on an edge pasted
/// from the line above it, pointing an unrelated `SystemRequirement` at an Issue about a flag parser.
/// A nominally-triaged Issue is worse than an untriaged one: it reports as handled.
///
/// Valid resolvers: a declared `action` (work that will close it), or a `Decision` (which moots it —
/// "we won't do X" is a first-class resolution, §1.4). A requirement, Need, Test or Story cannot resolve
/// anything: none of them is an act, so none can ever compute as complete against the Issue.
///
/// A BESPOKE PREDICATE, and D0107 is the precedent that makes that the right call rather than a
/// shortcut: this cannot be an `EdgeRule`, because `objectType` filters by declared item TYPE and an
/// `action` is not a typed element — the 127 legitimate action resolvers would all fail. Extending the
/// rule language to express "action OR Decision" is the larger change; the constraint is checked here
/// meanwhile, and the rule keeps `objectType = "*"` because that is honestly all it can say.
#[must_use]
pub fn resolver_kind(root: &Path) -> GuardReport {
    let actions = declared_task_names(root);
    match keel_view::view::resolves_edges(root) {
        Ok(edges) => {
            let scanned = edges.len();
            let violations = edges
                .into_iter()
                .filter(|(from, _, ty)| !resolver_kind_holds(&actions, from, ty))
                .map(|(from, to, ty)| {
                    let what = if ty.is_empty() { "not a declared action and not a typed item".to_string() } else { format!("a {ty}") };
                    format!(
                        "{to}: #Resolves comes from {from}, which is {what} — a resolver must be a declared action (work that closes it) or a Decision (which moots it); \
                         until it is, the Issue reports as TRIAGED while nothing is on the hook for it (issue136)"
                    )
                })
                .collect();
            GuardReport { name: "resolver-kind", scanned, warnings: Vec::new(), violations }
        }
        Err(e) => GuardReport { name: "resolver-kind", scanned: 0, warnings: Vec::new(), violations: vec![format!("error reading #Resolves edges: {e}")] },
    }
}

// THE ONE PREDICATE behind `resolver-kind` is keel_model::resolvers' (sprint 737, D0508); re-exported so `guards::issues::resolver_kind_holds` resolves.
pub use keel_model::resolvers::resolver_kind_holds;

#[cfg(test)]
mod taint_tests {
    use super::{acceptance_kind, taint_edges, taint_violations, untrusted_sources, Acceptance};
    use std::collections::{HashMap, HashSet};

    /// A decider's own GitHub issue is recorded `untrusted` (visibility) but its speaker is the human:
    /// counted in the population, not a taint source. A stranger's is.
    #[test]
    fn a_declared_deciders_own_words_are_not_a_taint_source() {
        let blob = "part st1 : Statement { :>> saidBy = \"williamweatherholtz\"; :>> sourceTrust = SourceTrust::untrusted; }\npart st2 : Statement { :>> saidBy = \"stranger\"; :>> sourceTrust = SourceTrust::untrusted; }\n";
        let deciders: HashSet<String> = std::iter::once("williamweatherholtz".to_string()).collect();
        let (scanned, sources) = untrusted_sources(blob, &deciders);
        assert_eq!(scanned, 2);
        assert_eq!(sources, vec!["st2".to_string()]);
    }

    const CHAIN: &str = "#DerivedFrom dependency from us9 to st9;\n#DerivedFrom dependency from n9 to us9;\n#CharteredBy dependency from dcBuildIt to d0900;\n#Implicates dependency from us9 to d0900;\n";

    fn set(names: &[&str]) -> HashSet<String> {
        names.iter().map(std::string::ToString::to_string).collect()
    }

    /// The `DoD` fixture: `Statement(untrusted)` -> story -> Decision -> task, task done, Decision
    /// auto-accepted: red, naming the path. The same chain with the Decision HUMAN-accepted: green.
    #[test]
    fn three_hops_do_not_lose_the_label_and_a_human_acceptance_confers_trust() {
        let edges = taint_edges(CHAIN);
        assert_eq!(edges.len(), 4);
        let sources = vec!["st9".to_string()];
        let mut accepted = HashMap::new();
        accepted.insert("d0900".to_string(), Acceptance::Auto);
        let v = taint_violations(&sources, &edges, &accepted, &set(&["dcBuildIt"]));
        assert_eq!(v.len(), 2, "{v:?}");
        assert!(v.iter().any(|m| m.starts_with("d0900 is an ACCEPTED") && m.contains("st9 #DerivedFrom-> us9 #Implicates-> d0900")), "{v:?}");
        assert!(v.iter().any(|m| m.starts_with("dcBuildIt is DONE") && m.contains("#CharteredBy-> dcBuildIt")), "{v:?}");
        accepted.insert("d0900".to_string(), Acceptance::Human);
        assert!(taint_violations(&sources, &edges, &accepted, &set(&["dcBuildIt"])).is_empty(), "the human's acceptance is where trust is conferred");
    }

    /// A proposed Decision is a plan; planning is what untrusted input may do. A task not yet done is
    /// work in progress, not a violation - the guard must not punish the route it prescribes.
    #[test]
    fn a_plan_and_unbuilt_work_are_not_violations() {
        let edges = taint_edges(CHAIN);
        assert!(taint_violations(&["st9".to_string()], &edges, &HashMap::new(), &HashSet::new()).is_empty());
    }

    /// The acceptance Test's own text says whether anyone read it.
    #[test]
    fn auto_and_human_acceptances_are_told_apart_by_the_record() {
        let auto = "verification d0900Accept : Test { :>> procedureText = \"AUTO-ACCEPTED under standing consent (D0207).\"; }\npart d0900AcceptR1 : TestResult { :>> outcome = VerdictKind::pass; }";
        assert_eq!(acceptance_kind(auto, "d0900"), Some(Acceptance::Auto));
        let human = "verification d0900Accept : Test { :>> procedureText = \"their words: 'i do like C the best'\"; }\npart d0900AcceptR1 : TestResult { :>> outcome = VerdictKind::pass; }";
        assert_eq!(acceptance_kind(human, "d0900"), Some(Acceptance::Human));
        assert_eq!(acceptance_kind("verification d0900Accept : Test { }", "d0900"), None, "no passing result: proposed");
    }
}
