//! Guard family `decisions` - split from `keel-cli/src/guards.rs` by `scripts/split_guards.py` (sprint 733).
//!
//! Each guard's dispatch arm, code and tests sit together; the shared scanners, the runner and the
//! lock predicates are the crate root's (`super`). Nothing here was retyped: the text is guards.rs's,
//! with `crate::` paths pointing at the members and private items opened to the crate.

use super::*;

/// The `decisions` family: every guard it dispatches, in `GUARD_NAMES` order, with the tier note each
/// arm carried in `run_one` (sprint 733). The root's union test holds these tables equal to `GUARD_NAMES`.
pub(crate) const FAMILY: Family = Family {
    name: "decisions",
    arms: &[
        ("requirement-rootedness", requirement_rootedness),
        ("decision-rationale", decision_rationale), // hard (D0103)
        ("decision-requirement-link", decision_requirement_link), // warning-only member of GUARD_NAMES (D0102)
        ("verification-trace", verification_trace), // warning-only (D0130/issue082) — delivered work whose requirement is untraced
        ("decision-scaffolding", decision_scaffolding), // WARNING-tier (D0188, composed with D0180) — an accepted promise chartering no work
        ("decision-amends-process", decision_amends_process), // WARNING-tier (issue298/D0244)
    ],
};

/// Guard: requirement-rootedness (D0098/D0099, issue047).
///
/// A declared `#Capability` (a user-facing feature) must carry a `#DerivedFrom` edge to a Need. An
/// HONESTY gate: shipping a capability whose driving Need is unstated is a traceability lie-of-omission.
/// UNMARKED work is exempt — decision-driven engine evolution is legitimate (D0064), so this never
/// floods (it binds only what is opted-in via the marker). The full charter-source balance is the
/// non-blocking `keel rootedness` burndown.
#[must_use]
pub fn requirement_rootedness(root: &Path) -> GuardReport {
    // CONTRACT (D0107): sourced from the declared capabilityRootednessRule (single gate source).
    match keel_view::view::rule_violations_opt(root, "capabilityRootednessRule") {
        Ok(Some((scanned, gaps))) => {
            let violations = gaps
                .into_iter()
                .map(|c| format!("{c}: #Capability with no #DerivedFrom edge to a Need — state the driving Need (D0099)"))
                .collect();
            GuardReport { name: "requirement-rootedness", scanned, warnings: Vec::new(), violations }
        }
                // D0136/issue090: an ABSENT rule means the project has not ADOPTED this control —
        // it has not violated it. Warn (never silent, so deleting a rule to dodge the gate is
        // visible) and pass; a MALFORMED rule still fails via Err below.
        Ok(None) => GuardReport { name: "requirement-rootedness", scanned: 0, warnings: vec!["declared rule `capabilityRootednessRule` is not present — this control is NOT ADOPTED by this project, so nothing was checked (D0136/issue090)".to_string()], violations: Vec::new() },
Err(e) => GuardReport { name: "requirement-rootedness", scanned: 0, warnings: Vec::new(), violations: vec![format!("error computing rootedness: {e}")] },
    }
}

// ── decision-amends-process (issue298 / D0244): an amendment reaches the definition ──────────────

/// One process definition's lexical anchors: its file, and the identifiers a Decision would use to
/// speak about it - the `Process` action name, the file stem, and every `ProcessStep` name.
pub(crate) struct ProcessAnchors {
    file: String,
    names: Vec<String>,
}

pub(crate) fn process_anchors(root: &Path) -> Vec<ProcessAnchors> {
    let dir = root.join(".engine").join("processes");
    let Ok(rd) = std::fs::read_dir(&dir) else { return Vec::new() };
    let mut out = Vec::new();
    for entry in rd.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "sysml") {
            continue;
        }
        let Ok(text) = keel_model::corpus::read_to_string(&path) else { continue };
        let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let mut names = vec![stem];
        for line in text.lines() {
            let l = line.trim();
            if let Some(rest) = l.strip_prefix("action ") {
                if rest.contains(": Process ") || rest.contains(": Process{") || rest.contains(": ProcessStep") {
                    if let Some(name) = rest.split(|c: char| c == ':' || c.is_whitespace()).next() {
                        if !name.is_empty() {
                            names.push(name.to_string());
                        }
                    }
                }
            }
        }
        out.push(ProcessAnchors { file: format!(".engine/processes/{}", path.file_name().unwrap_or_default().to_string_lossy()), names });
    }
    out.sort_by(|a, b| a.file.cmp(&b.file));
    out
}

/// Does `text` contain `ident` as a whole identifier (not as a substring of a longer name)?
pub(crate) fn names_identifier(text: &str, ident: &str) -> bool {
    text.match_indices(ident).any(|(i, _)| {
        let before = text[..i].chars().next_back();
        let after = text[i + ident.len()..].chars().next();
        !before.is_some_and(|c| c.is_alphanumeric() || c == '_') && !after.is_some_and(|c| c.is_alphanumeric() || c == '_')
    })
}

/// Pure core (issue298): for each staged Decision, every process it names by identifier whose
/// definition file is NOT also staged yields one warning naming the Decision, the identifier and the
/// file it should have reached.
pub(crate) fn amendment_warnings(decision_texts: &[(String, String)], staged: &[String], anchors: &[ProcessAnchors]) -> Vec<String> {
    let mut out = Vec::new();
    for (path, text) in decision_texts {
        for a in anchors {
            if staged.iter().any(|s| s == &a.file) {
                continue;
            }
            // An anchor must LOOK like an identifier - camelCase or hyphenated. A process whose name is a
            // plain English word (`migration`, `dor`) would fire on every sentence using the word: the
            // first commit carrying this guard warned on "step 5 of the migration plan".
            let mut hit: Vec<&str> = a
                .names
                .iter()
                .filter(|n| n.len() > 3 && (n.contains('-') || n.chars().any(char::is_uppercase)) && names_identifier(text, n))
                .map(String::as_str)
                .collect();
            hit.sort_unstable();
            hit.dedup();
            if hit.is_empty() {
                continue;
            }
            out.push(format!(
                "{path} names `{}` of {} and this commit does not touch that definition - if the Decision AMENDS the step, edit the process file in the same commit so the definition carries it (D0244: a design may amend a process, never quietly disagree with one; issue298 is D0243 doing exactly this); if it only CITES the step, this is noise to read past",
                hit.join("`, `"),
                a.file
            ));
        }
    }
    out
}

/// Guard (WARNING-tier, D0102 promote-once-low-noise): a staged Decision that names a process or one
/// of its steps by identifier while that process definition is unchanged in the same commit.
///
/// THE CLASS (issue298): D0243 changed what knowledge-graph-memory's steps 1 and 2 MEAN - `.knowledge/`
/// optional, seeding corpus-derived - and never touched the process file, so the keystone lock never
/// fired: the guarded artifact was not edited, the change was made AROUND the control. D0244 wrote the
/// rule ("a design may amend a process; it may not quietly disagree with one") and carried D0243's
/// amendment into the definition by hand. This guard watches for the next one.
///
/// WHAT IT CAN AND CANNOT SEE, measured over the 296 Decisions in this tree before it was written:
/// 17 name a process step by identifier and 5 of those did not touch the file (D0242 among them - a
/// real amendment). 24 mention steps by NUMBER ("steps 1-2", "step 5 of the plan", "STPA step 2") and
/// two thirds of those are not amendments at all, so numbers are not an anchor. D0243 itself says
/// "the source process's steps 1-2" and names nothing - the identifier rule would NOT have caught it.
/// That residual is stated here rather than hidden behind a passing check: the durable fix for the
/// unnamed shape is an `#Amends` edge a Decision must carry, which is a schema change and a fork.
#[must_use]
pub fn decision_amends_process(root: &Path) -> GuardReport {
    let read = ChangeRead::current();
    let staged = changed_files(root, read);
    let decision_texts: Vec<(String, String)> = staged
        .iter()
        .filter(|p| is_decision_file(p))
        .map(|p| (p.clone(), changed_text(root, p, read)))
        // A staged Decision whose PROSE is unchanged from HEAD - an appended acceptance result, a
        // re-binding (D0308) - amends nothing; the first re-bind of seventeen Decisions produced
        // twenty-five citation warnings, all noise. Only text that moved can amend.
        .filter(|(p, staged_text)| {
            let at_head = git_stdout(root, &["show", &format!("HEAD:{p}")]);
            at_head.is_empty() || ["decision", "rationale", "consequences", "context"].iter().any(|k| field_of(&at_head, k) != field_of(staged_text, k))
        })
        .collect();
    let anchors = process_anchors(root);
    let mut warnings = amendment_warnings(&decision_texts, &staged, &anchors);
    warnings.push(read_line(read));
    GuardReport { name: "decision-amends-process", scanned: decision_texts.len(), warnings, violations: Vec::new() }
}

/// Guard (D0103): every Decision must carry a substantive `context` + `rationale` (the why).
///
/// Not just the schema-present (possibly blank) fields — a recorded decision without its why is ill-formed
/// state. HARD honest-state gate: a Decision whose `context` or `rationale` is blank/trivial (trimmed < 20
/// chars) is a violation. Precise (no false positives), and all current decisions pass — no flood.
#[must_use]
pub fn decision_rationale(root: &Path) -> GuardReport {
    // CONTRACT (D0107): sourced from the declared decisionRationaleRule (single gate source).
    match keel_view::view::rule_violations_opt(root, "decisionRationaleRule") {
        Ok(Some((total, weak))) => {
            let violations = weak
                .into_iter()
                .map(|d| format!("{d}: blank/trivial context or rationale (D0103 — a Decision must state a substantive why; >=20 chars each)"))
                .collect();
            GuardReport { name: "decision-rationale", scanned: total, warnings: Vec::new(), violations }
        }
                // D0136/issue090: an ABSENT rule means the project has not ADOPTED this control —
        // it has not violated it. Warn (never silent, so deleting a rule to dodge the gate is
        // visible) and pass; a MALFORMED rule still fails via Err below.
        Ok(None) => GuardReport { name: "decision-rationale", scanned: 0, warnings: vec!["declared rule `decisionRationaleRule` is not present — this control is NOT ADOPTED by this project, so nothing was checked (D0136/issue090)".to_string()], violations: Vec::new() },
Err(e) => GuardReport { name: "decision-rationale", scanned: 0, warnings: Vec::new(), violations: vec![format!("error reading decision rationale: {e}")] },
    }
}

/// Guard (D0102/issue052): an accepted Decision that names a Need/SystemRequirement in its prose but
/// carries NO typed edge to it — a governance/derivation link that should be typed, not prose.
///
/// WARNING-level: it RUNS in `GUARD_NAMES` (visible on every commit, not ignorable) but emits warnings,
/// never violations, so it does not block (D0102 — warning first; promotable to a hard gate once proven
/// low-noise by moving the warnings to violations). A compute error IS a violation.
#[must_use]
pub fn decision_requirement_link(root: &Path) -> GuardReport {
    match keel_view::view::decision_requirement_prose_links(root) {
        Ok(pairs) => {
            let warnings = pairs
                .iter()
                .map(|(d, r)| format!("{d} names {r} in prose but has no typed edge to it (D0102 — link via #DependsOn/#Supersede/#DerivedFrom/satisfy/derive)"))
                .collect();
            GuardReport { name: "decision-requirement-link", scanned: pairs.len(), warnings, violations: Vec::new() }
        }
        Err(e) => GuardReport { name: "decision-requirement-link", scanned: 0, warnings: Vec::new(), violations: vec![format!("error computing decision-requirement links: {e}")] },
    }
}

// ── verification-trace guard (delivered work whose requirement carries no verification) ───────────

/// Guard: a DELIVERED verification names a `SystemRequirement` in prose but never `#Verify`-links it.
///
/// Closes issue082 (D0130). Sprint 247 delivered six SRs with passing `DoD` `TestResult`s and CI green,
/// yet `tier-satisfaction` reported all six UNVERIFIED — because an SR is verified only when a Test
/// `#Verify`-links TO IT, and the `DoD` Tests linked to the backlog ACTION instead. So the model could
/// not distinguish *requirement not yet delivered* from *requirement delivered but its verification was
/// never traced upward*, and the AI then reported `sr_verified_pct` to the human as though it meant
/// functional verification. This makes that specific, previously-invisible state visible.
///
/// WARNING-level and non-blocking, on two independent grounds: completeness is honest-state burndown
/// that must never gate a commit (D0098), and prose-name matching is a heuristic, so it follows the
/// D0102 promote-once-low-noise pattern. A compute error IS a violation.
#[must_use]
pub fn verification_trace(root: &Path) -> GuardReport {
    match keel_view::view::untraced_verification_links(root) {
        Ok(pairs) => {
            let warnings = pairs
                .iter()
                .map(|(v, sr)| {
                    format!("{v} PASSED and names {sr} in its procedure, but no #Verify edge reaches {sr} — the work is verified, the REQUIREMENT is not (issue082); author `#Verify dependency from {v} to {sr};`")
                })
                .collect();
            GuardReport { name: "verification-trace", scanned: pairs.len(), warnings, violations: Vec::new() }
        }
        Err(e) => GuardReport { name: "verification-trace", scanned: 0, warnings: Vec::new(), violations: vec![format!("error computing verification trace: {e}")] },
    }
}

/// Guard 42: an accepted `#ProspectiveChange` Decision is reachable by a tracked-item edge (D0188).
///
/// WARNING-TIER by D0188's composed rule with D0180: promotion to hard is a recorded review citing
/// the fire-ledger evidence window, never a default. FORWARD-ONLY: the boundary is D0188's own
/// recorded acceptance date, read from the model (never hardcoded); the 64 historical gaps are not
/// retro-failed. THE LANDING-SPRINT GRACE: the most recently accepted violator is exempt — in this
/// repo's practice acceptance and the chartered work land together, but a multi-contributor project
/// may accept in one integration and charter in the next.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn decision_scaffolding(root: &Path) -> GuardReport {
    let mut files = keel_model::corpus::collect_sysml(&root.join(".tracking"));
    files.extend(keel_model::corpus::collect_sysml(&root.join(".engine")));
    let mut texts: Vec<(String, String)> = Vec::new();
    for f in &files {
        if let Ok(t) = keel_model::corpus::read_to_string(f) {
            texts.push((relpath(root, f), t));
        }
    }
    // The forward-only boundary: d0188's own acceptance date. Absent (downstream trees without the
    // decision) → the guard has no boundary and passes with zero scanned — adoption is by decision.
    let boundary = texts
        .iter()
        .find_map(|(_, t)| {
            let i = t.find("part d0188AcceptR")?;
            let j = t[i..].find("judgedAt = \"")? + i + "judgedAt = \"".len();
            t.get(j..j + 10).map(str::to_string)
        });
    let Some(boundary) = boundary else {
        return GuardReport { name: "decision-scaffolding", scanned: 0, warnings: Vec::new(), violations: Vec::new() };
    };
    // Accepted #ProspectiveChange decisions with their own acceptance dates.
    let mut candidates: Vec<(String, String)> = Vec::new(); // (decision, acceptedAt)
    for (_, t) in &texts {
        for line in t.lines() {
            let l = line.trim_start();
            let Some(rest) = l.strip_prefix("#ProspectiveChange part ") else { continue };
            let Some((name, tail)) = rest.split_once(':') else { continue };
            if !tail.trim_start().starts_with("Decision") {
                continue;
            }
            let name = name.trim().to_string();
            if !t.contains("DecisionStatus::accepted") {
                continue;
            }
            let accepted_at = t
                .find(&format!("part {name}AcceptR"))
                .and_then(|i| {
                    let j = t[i..].find("judgedAt = \"")? + i + "judgedAt = \"".len();
                    t.get(j..j + 10).map(str::to_string)
                })
                .unwrap_or_default();
            if !accepted_at.is_empty() && accepted_at.as_str() >= boundary.as_str() {
                candidates.push((name, accepted_at));
            }
        }
    }
    let scanned = candidates.len();
    // Reachability: any inbound tracked-item edge (charteredby/derivedfrom/resolves) or satisfy. The
    // targets are collected ONCE (issue520): the per-candidate rescan of every corpus line, with two
    // string allocations per line, put this guard at 4645-12463 ms per fire and led 14 of the stop
    // hook's 15 slow fires. One pass, one set, one lookup per candidate; the verdicts are unchanged.
    let targets = inbound_edge_targets(&texts);
    let mut bare: Vec<(String, String)> = unreached(candidates, &targets);
    // Landing-sprint grace: the newest violator by acceptance date is exempt.
    bare.sort_by(|a, b| a.1.cmp(&b.1));
    if !bare.is_empty() {
        bare.pop();
    }
    let mut warnings: Vec<String> = bare
        .into_iter()
        .map(|(d, at)| {
            format!(
                "{d} (accepted {at}): an accepted #ProspectiveChange Decision with NO inbound tracked-item edge — it promises process change but charters no work (D0188). Add a #CharteredBy/#DerivedFrom/#Resolves edge from the item that delivers it, or record why none is needed."
            )
        })
        .collect();
    // D0303 OPTION C (issue331): a Decision is ONE clause. The guard can see whether a Decision is
    // chartered but not which clause an edge covers - so a compound Decision half-delivered read as
    // covered (d0252, nine days). The human chose the convention over a schema change: from the day
    // after acceptance, a Decision whose text enumerates clauses is a violation naming the count and
    // the fix (one Decision per clause, edges between them); the ones before are counted once.
    let (forward, grandfathered) = compound_decisions(&texts, COMPOUND_DECISION_CUTOFF);
    let mut violations: Vec<String> = forward
        .into_iter()
        .map(|(d, n)| format!("{d}: a COMPOUND Decision - its text enumerates {n} clauses - and decision-scaffolding cannot see which clause an edge charters, so partial delivery would be invisible (issue331). D0303 option C: record one Decision per clause, with #DependsOn edges between them, and let each be chartered on its own."))
        .collect();
    violations.sort();
    if grandfathered > 0 {
        warnings.push(history_line(&format!(
            "{grandfathered} compound Decision(s) recorded before {COMPOUND_DECISION_CUTOFF} enumerate several clauses - grandfathered under D0303 option C; their partial delivery is not visible to this guard, and re-splitting history is the D0129 class, so they are counted, not reported"
        )));
    }
    // D0469 (issue444): a gate was asked for at a fixture's price - D0421 said "seconds" and the first
    // live set cost 528 s. A PROPOSED marked Decision that names the land/push path in its decision text
    // carries a MEASURED: token in its rationale (the run, the host, the seconds on a live set) or is
    // named here. Accepted Decisions are the human's word already; the rule is inert until D0469 is
    // human-accepted (D0337) - an auto-accept confers nothing (D0291).
    if texts.iter().any(|(_, t)| acceptance_kind(t, "d0469") == Some(Acceptance::Human)) {
        warnings.extend(unmeasured_path_decisions(&texts).into_iter().map(|(d, words)| format!(
            "{d}: a PROPOSED process-change Decision whose text names the land/push path ({words}) with no MEASURED: token in its rationale - the human would be asked to accept a control at an unmeasured price (issue444: D0421 said seconds, the first live set cost 528 s). Run the control on a live set and put MEASURED: <run>, <host>, <seconds> in the rationale before the ask (D0469)."
        )));
    }
    GuardReport { name: "decision-scaffolding", scanned, warnings, violations }
}

/// D0303 option C's date: a Decision created on or after it is one clause.
pub(crate) const COMPOUND_DECISION_CUTOFF: &str = "2026-09-05";

/// Pure core (issue331 / D0303 C): every Decision whose `decision` text enumerates two or more clauses
/// (`(1) ... (2) ...`, `(a) ... (b) ...`, or `clause A ... clause B`), split by its `createdAt`
/// against the cutoff into `(forward violators as (name, clause count), grandfathered count)`.
pub(crate) fn compound_decisions(texts: &[(String, String)], cutoff: &str) -> (Vec<(String, usize)>, usize) {
    let mut forward = Vec::new();
    let mut grandfathered = 0usize;
    for (_, t) in texts {
        let mut rest = t.as_str();
        while let Some(i) = rest.find("part d0") {
            let block = &rest[i..];
            let name: String = block[5..].chars().take_while(|c| c.is_alphanumeric()).collect();
            let end = block.find("\n    }").map_or(block.len(), |e| e + 6);
            let body = &block[..end];
            rest = &rest[i + 5 + name.len()..];
            if !body.contains(": Decision") {
                continue;
            }
            let field = |key: &str| -> &str {
                body.find(&format!("{key} = \"")).map_or("", |s| {
                    let v = &body[s + key.len() + 4..];
                    v.find('"').map_or(v, |e| &v[..e])
                })
            };
            let decision = field("decision");
            let clauses = clause_count(decision);
            if clauses < 2 {
                continue;
            }
            if field("createdAt") >= cutoff {
                forward.push((name, clauses));
            } else {
                grandfathered += 1;
            }
        }
    }
    (forward, grandfathered)
}

/// How many enumerated clauses a decision text carries: the largest of `(N)` numerals, `(x)` letters,
/// and `clause X` markers that appear in sequence from the first.
pub(crate) fn clause_count(text: &str) -> usize {
    let numeric = (1..=9).take_while(|n| text.contains(&format!("({n})"))).count();
    let lettered = ('a'..='i').take_while(|c| text.contains(&format!("({c})"))).count();
    let clause = ('A'..='I').take_while(|c| text.contains(&format!("clause {c}"))).count();
    numeric.max(lettered).max(clause)
}

/// Pure core (issue520): the inbound tracked-item edge targets of a corpus, read once - the `X` of
/// every `#CharteredBy` / `#DerivedFrom` / `#Resolves dependency from ... to X;` line and the `X` of
/// every `satisfy X by ...` line, each read at the start of its line. `decision_scaffolding` answers a
/// candidate's reachability with one lookup in this set instead of rescanning the corpus per candidate.
pub(crate) fn inbound_edge_targets(texts: &[(String, String)]) -> HashSet<String> {
    let mut out = HashSet::new();
    for (_, t) in texts {
        for line in t.lines() {
            let l = line.trim_start();
            if l.starts_with("#CharteredBy dependency from ")
                || l.starts_with("#DerivedFrom dependency from ")
                || l.starts_with("#Resolves dependency from ")
            {
                if let Some((_, to)) = l.trim_end().trim_end_matches(';').rsplit_once(" to ") {
                    out.insert(to.to_string());
                }
            } else if let Some(rest) = l.strip_prefix("satisfy ") {
                if let Some((subject, _)) = rest.split_once(" by ") {
                    out.insert(subject.to_string());
                }
            }
        }
    }
    out
}

/// The candidates no inbound edge reaches, before the landing-sprint grace - `(decision, acceptedAt)`
/// in candidate order. Separated from the grace so the known positive (a bare Decision beside a
/// chartered one is named) can be pinned: through the public guard a lone bare Decision is always the
/// newest violator and the grace exempts it.
pub(crate) fn unreached(candidates: Vec<(String, String)>, targets: &HashSet<String>) -> Vec<(String, String)> {
    candidates.into_iter().filter(|(d, _)| !targets.contains(d.as_str())).collect()
}

/// The words that put a Decision on the land/push path (D0469, issue444): whole words, any case.
pub(crate) const PATH_WORDS: [&str; 4] = ["land", "push", "refuse", "gate"];

/// Pure core (D0469 / issue444): every PROPOSED `#ProspectiveChange` Decision whose `decision` text
/// names one of [`PATH_WORDS`] as a whole word and whose `rationale` carries no `MEASURED:` token, as
/// `(name, the words matched)`. A Decision with a passing `AcceptR` in its file is not proposed.
pub(crate) fn unmeasured_path_decisions(texts: &[(String, String)]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (_, t) in texts {
        for line in t.lines() {
            let Some(rest) = line.trim_start().strip_prefix("#ProspectiveChange part ") else { continue };
            let Some((name, tail)) = rest.split_once(':') else { continue };
            if !tail.trim_start().starts_with("Decision") {
                continue;
            }
            let name = name.trim();
            if acceptance_kind(t, name).is_some() {
                continue;
            }
            let Some(start) = t.find(&format!("part {name} : Decision")) else { continue };
            let body = &t[start..];
            let body = body.find("\n    }").map_or(body, |e| &body[..e]);
            let field = |key: &str| -> &str {
                body.find(&format!("{key} = \"")).map_or("", |s| {
                    let v = &body[s + key.len() + 4..];
                    v.find('"').map_or(v, |e| &v[..e])
                })
            };
            let decision = field("decision").to_ascii_lowercase();
            let mut words: Vec<&str> = decision
                .split(|c: char| !c.is_ascii_alphanumeric())
                .filter(|w| PATH_WORDS.contains(w))
                .collect();
            words.sort_unstable();
            words.dedup();
            if words.is_empty() || field("rationale").contains("MEASURED:") {
                continue;
            }
            out.push((name.to_string(), words.join("/")));
        }
    }
    out.sort();
    out
}

#[cfg(test)]
mod amendment_tests {
    use super::{amendment_warnings, names_identifier, ProcessAnchors};

    fn kg() -> Vec<ProcessAnchors> {
        vec![ProcessAnchors {
            file: ".engine/processes/knowledge-graph-memory.sysml".to_string(),
            names: vec!["knowledge-graph-memory".to_string(), "knowledgeGraphMemory".to_string(), "kgInjection".to_string(), "kgQuestions".to_string()],
        }]
    }

    /// issue298, the D0242 shape: a Decision names `kgInjection`; the process file is not staged; one
    /// warning names the Decision, the identifier and the file.
    #[test]
    fn a_decision_naming_a_step_without_the_definition_warns() {
        let d = (".engine/decisions/0242-x.sysml".to_string(), "The kgInjection step now pushes before the model thinks.".to_string());
        let w = amendment_warnings(std::slice::from_ref(&d), std::slice::from_ref(&d.0), &kg());
        assert_eq!(w.len(), 1, "{w:?}");
        assert!(w[0].contains("kgInjection") && w[0].contains("knowledge-graph-memory.sysml") && w[0].contains("D0244"), "{}", w[0]);
        // the same commit touching the definition is the amendment reaching it: silent
        let staged = vec![d.0.clone(), ".engine/processes/knowledge-graph-memory.sysml".to_string()];
        assert!(amendment_warnings(&[d], &staged, &kg()).is_empty());
    }

    /// The stated residual, armed so it cannot be forgotten: D0243's own sentence names NOTHING and is
    /// not caught. If this test ever fails, the residual has been closed and the doc must say so.
    #[test]
    fn the_unnamed_shape_is_a_known_miss() {
        let d = (".engine/decisions/0243-x.sysml".to_string(), "as the source process's steps 1-2 imply - rejected; .knowledge/ becomes an optional refinement".to_string());
        let staged = vec![d.0.clone()];
        assert!(amendment_warnings(std::slice::from_ref(&d), &staged, &kg()).is_empty(), "the identifier rule does not see an unnamed process");
    }

    /// A process named by a plain English word is not an anchor: `migration` in "step 5 of the
    /// migration plan" is prose, and the first commit carrying this guard warned on exactly that.
    #[test]
    fn a_plain_word_process_name_is_not_an_anchor() {
        let anchors = vec![ProcessAnchors { file: ".engine/processes/migration.sysml".to_string(), names: vec!["migration".to_string(), "migration".to_string(), "mgExpand".to_string()] }];
        let d = (".engine/decisions/0281-x.sysml".to_string(), "This is step 5 of the migration plan.".to_string());
        assert!(amendment_warnings(std::slice::from_ref(&d), std::slice::from_ref(&d.0), &anchors).is_empty());
        let d = (".engine/decisions/0281-y.sysml".to_string(), "The mgExpand step now also copies the pin.".to_string());
        let w = amendment_warnings(std::slice::from_ref(&d), std::slice::from_ref(&d.0), &anchors);
        assert_eq!(w.len(), 1, "a camelCase step name still anchors: {w:?}");
    }

    /// Whole identifiers only: `kgInjectionProvenOnTheTrap` (a backlog task) must not read as the
    /// step `kgInjection`, or every sprint that delivered the step would warn.
    #[test]
    fn a_longer_name_containing_the_step_is_not_the_step() {
        assert!(!names_identifier("dcKgInjectionProvenOnTheTrap kgInjectionProvenOnTheTrap", "kgInjection"));
        assert!(names_identifier("the kgInjection step", "kgInjection"));
        assert!(names_identifier("(kgInjection)", "kgInjection"));
    }
}

#[cfg(test)]
mod compound_decision_tests {
    use super::{clause_count, compound_decisions};

    /// D0303 C armed against the broken predicate (D0253): a two-clause Decision created after the
    /// cutoff is reported with its count; one before is counted; a single-clause one is neither.
    #[test]
    fn a_compound_decision_is_reported_forward_only() {
        assert_eq!(clause_count("(1) first. (2) second. (3) third."), 3);
        assert_eq!(clause_count("clause A says x; clause B says y"), 2);
        assert_eq!(clause_count("one thing, said once"), 0);
        assert_eq!(clause_count("(2) alone does not enumerate"), 0);
        let mk = |name: &str, created: &str, decision: &str| format!(
            "package X {{\n    part {name} : Decision {{\n        :>> id = \"x\";\n        :>> title = \"t\";\n        :>> createdAt = \"{created}\";\n        :>> decision = \"{decision}\";\n    }}\n}}\n"
        );
        let texts = vec![
            ("a".to_string(), mk("d0901", "2026-09-10", "(1) build the thing; (2) also the other thing")),
            ("b".to_string(), mk("d0902", "2026-01-01", "(1) old; (2) compound; grandfathered")),
            ("c".to_string(), mk("d0903", "2026-09-10", "one clause, one Decision")),
        ];
        let (forward, grandfathered) = compound_decisions(&texts, "2026-09-05");
        assert_eq!(forward, vec![("d0901".to_string(), 2)]);
        assert_eq!(grandfathered, 1);
    }
}

#[cfg(test)]
mod inbound_edge_tests {
    use super::{decision_scaffolding, inbound_edge_targets, unreached};

    const EDGES: &str = "package P {\n    #CharteredBy dependency from storyA to d0901;\n    #DerivedFrom dependency from us9 to st9;\n        #Resolves dependency from dcFix to issue903;;\n    satisfy nNeed by srReq;\n    :>> decision = \"#CharteredBy dependency from s to d0904;\";\n    #CharteredBy dependency from storyB to d0905 ;\n    dependency from x to d0906;\n}\n";

    /// D0388 pair on the pure core: the four edge shapes are read at the start of a line (with a
    /// trailing `;;` and a nested indent); a prose line quoting an edge and an untyped dependency are
    /// not targets, and a head followed by a space before its `;` reaches no name (the set holds the
    /// bytes as written, which is what the per-candidate `ends_with` also saw).
    #[test]
    fn the_targets_are_the_edge_heads_read_once() {
        let texts = vec![("a.sysml".to_string(), EDGES.to_string())];
        let t = inbound_edge_targets(&texts);
        for hit in ["d0901", "st9", "issue903", "nNeed"] {
            assert!(t.contains(hit), "{hit} missing from {t:?}");
        }
        for miss in ["d0904", "d0905", "d0906", "storyA", "srReq"] {
            assert!(!t.contains(miss), "{miss} wrongly in {t:?}");
        }
        assert!(inbound_edge_targets(&[]).is_empty());
        let cands = vec![("d0901".to_string(), "2026-09-10".to_string()), ("d0904".to_string(), "2026-09-11".to_string())];
        assert_eq!(unreached(cands, &t), vec![("d0904".to_string(), "2026-09-11".to_string())], "the bare one is named, the chartered one is not");
    }

    fn decision(name: &str, accepted_at: &str) -> String {
        format!(
            "    #ProspectiveChange part {name} : Decision {{ :>> id = \"00000000-0000-4000-8000-00000000{}\"; :>> title = \"t\"; :>> status = DecisionStatus::accepted; }}\n    part {name}AcceptR : TestResult {{ :>> outcome = VerdictKind::pass; :>> judgedAt = \"{accepted_at}\"; :>> judgedBy = \"human\"; }}\n",
            &name[1..]
        )
    }

    fn run(tag: &str, body: &str) -> Vec<String> {
        let root = std::env::temp_dir().join(format!("keel-inbound-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".tracking")).expect("mkdir");
        let boundary = "    part d0188AcceptR : TestResult { :>> outcome = VerdictKind::pass; :>> judgedAt = \"2026-01-01\"; }\n";
        std::fs::write(root.join(".tracking/fx.sysml"), format!("package Fx {{\n{boundary}{body}}}\n")).expect("write");
        let r = decision_scaffolding(&root);
        let _ = std::fs::remove_dir_all(&root);
        assert!(r.violations.is_empty(), "{:?}", r.violations);
        r.warnings
    }

    /// Through the public guard: two bare accepted Decisions beside a chartered one - the OLDER bare
    /// one is named and the newest is exempt under the landing-sprint grace (known positive); one
    /// bare Decision that is the newest violator names nothing (known negative).
    #[test]
    fn the_guard_reads_the_same_verdicts_from_the_set() {
        let chartered = format!("{}    #CharteredBy dependency from story to d0901;\n", decision("d0901", "2026-09-10"));
        let w = run("pos", &format!("{chartered}{}{}", decision("d0902", "2026-09-11"), decision("d0903", "2026-09-12")));
        assert_eq!(w.len(), 1, "{w:?}");
        assert!(w[0].starts_with("d0902 (accepted 2026-09-11)"), "{w:?}");
        let w = run("neg", &format!("{chartered}{}", decision("d0902", "2026-09-11")));
        assert!(w.is_empty(), "{w:?}");
    }
}

#[cfg(test)]
mod unmeasured_path_tests {
    use super::unmeasured_path_decisions;

    fn decision(name: &str, decision: &str, rationale: &str, accepted: bool) -> String {
        let accept = if accepted {
            format!("        verification {name}Accept : Test {{ :>> method = VerificationMethod::confirmation; }}\n        part {name}AcceptR : TestResult {{ :>> outcome = VerdictKind::pass; :>> judgedAt = \"2026-09-10\"; :>> judgedBy = \"human\"; }}\n")
        } else {
            String::new()
        };
        format!(
            "package X {{\n    #ProspectiveChange part {name} : Decision {{\n        :>> id = \"x\";\n        :>> title = \"t\";\n        :>> createdAt = \"2026-09-13\";\n        :>> decision = \"{decision}\";\n        :>> rationale = \"{rationale}\";\n    }}\n{accept}}}\n"
        )
    }

    /// D0388 trio from the definition of done: the guard warns on a proposed Decision with the path words and no token
    /// (known positive); it is silent on D0421's shape as corrected - path words, MEASURED: token - and on
    /// a proposed Decision with no path words (known negatives). An accepted Decision with the words and
    /// no token is the human's word already and is not named; the match is a whole word, any case.
    #[test]
    fn a_proposed_path_decision_without_a_measured_cost_is_named() {
        let texts = vec![
            ("p".to_string(), decision("d0901", "keel land will refuse the push until the touched set passes", "the set of one runs in about two seconds", false)),
            ("n1".to_string(), decision("d0902", "keel land refuses the push until the touched set passes", "MEASURED: 5fbe410, this host, 528 s wall on the first live set", false)),
            ("n2".to_string(), decision("d0903", "retro findings are Issues, not Decisions", "empirical findings name a defect", false)),
            ("acc".to_string(), decision("d0904", "the push GATE refuses a red tree", "no token, but accepted", true)),
            ("sub".to_string(), decision("d0905", "the landing and the pushed tree and the gateway and what refuses are not the words", "no token", false)),
            ("case".to_string(), decision("d0906", "Land and PUSH, in any case", "no token", false)),
        ];
        assert_eq!(
            unmeasured_path_decisions(&texts),
            vec![("d0901".to_string(), "land/push/refuse".to_string()), ("d0906".to_string(), "land/push".to_string())]
        );
        assert!(unmeasured_path_decisions(&[]).is_empty());
    }
}
