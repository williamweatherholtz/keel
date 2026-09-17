//! Guard family `sprints` - split from `keel-cli/src/guards.rs` by `scripts/split_guards.py` (sprint 733).
//!
//! Each guard's dispatch arm, code and tests sit together; the shared scanners, the runner and the
//! lock predicates are the crate root's (`super`). Nothing here was retyped: the text is guards.rs's,
//! with `crate::` paths pointing at the members and private items opened to the crate.

use super::*;

/// The `sprints` family: every guard it dispatches, in `GUARD_NAMES` order, with the tier note each
/// arm carried in `run_one` (sprint 733). The root's union test holds these tables equal to `GUARD_NAMES`.
pub(crate) const FAMILY: Family = Family {
    name: "sprints",
    arms: &[
        ("sprint-coverage", sprint_coverage),
        ("ceremony", ceremony),
        ("charter", charter),
        ("priority-inversion", priority_inversion), // warning-only (D0130/issue084) — recorded order vs recorded severity
        ("retro-backlog", retro_backlog), // warning-only (D0130/issue085) — a retro finding must not terminate in prose
        ("stale-gate-prose", stale_gate_prose),
        ("scaffold-placeholder", scaffold_placeholder), // hard (dcSprintScaffold) — an unfilled skeleton is not a record
        ("sprint-closure", sprint_closure),
    ],
};

/// `<task>` from a `part <task>DoDR<n> : TestResult { ...pass }` part name.
pub(crate) fn strip_dodr(name: &str) -> Option<String> {
    let pos = name.find("DoDR")?;
    let after = &name[pos + "DoDR".len()..];
    if after.is_empty() || !after.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let task = &name[..pos];
    if task.is_empty() {
        None
    } else {
        Some(task.to_string())
    }
}

/// Done tasks declared in the backlog: `part <task>DoDR<n> : TestResult { ...VerdictKind::pass }`.
pub(crate) fn done_tasks(backlog: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    for (idx, _) in backlog.match_indices("part ") {
        let after = &backlog[idx + "part ".len()..];
        let name: String = after.chars().take_while(|c| keel_model::algo::is_word(*c)).collect();
        if let Some(task) = strip_dodr(&name) {
            let stmt_end = backlog[idx..].find('}').map_or(backlog.len(), |e| idx + e);
            let stmt = &backlog[idx..stmt_end];
            if stmt.contains(": TestResult") && stmt.contains("VerdictKind::pass") {
                out.insert(task);
            }
        }
    }
    out
}

pub(crate) fn delivery_blob(root: &Path) -> String {
    keel_model::corpus::collect_sysml(&root.join(".tracking").join("delivery"))
        .iter()
        .filter_map(|p| keel_model::corpus::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Guard: every done backlog task is covered by a sprint (its name appears in a delivery file)
/// or is grandfathered. Mirrors `validate_sprint_coverage.py` (D0064/issue020).
#[must_use]
pub fn sprint_coverage(root: &Path) -> GuardReport {
    let backlog = keel_model::corpus::read_to_string(root.join(".tracking").join("backlog.sysml")).unwrap_or_default();
    let done = done_tasks(&backlog);
    let blob = delivery_blob(root);
    let grandfathered: HashSet<&str> = GRANDFATHERED.iter().copied().collect();
    let mut uncovered: Vec<String> = done
        .iter()
        .filter(|t| !blob.contains(t.as_str()) && !grandfathered.contains(t.as_str()))
        .cloned()
        .collect();
    uncovered.sort();
    let violations = uncovered
        .into_iter()
        .map(|t| format!("{t}: done but not covered by any sprint (D0064/issue020)"))
        .collect();
    GuardReport { name: "sprint-coverage", scanned: done.len(), warnings: Vec::new(), violations }
}

/// The tasks declared inside a delivery file's `action def` block, and the `TestResult` names
/// present in that same file. A task is STAMPED when some result name starts with the task name
/// (`storyFooDoDR1` / `storyFooR1` both stamp `storyFoo` — both spellings are in the corpus).
pub(crate) fn sprint_tasks_and_results(src: &str) -> (Vec<String>, Vec<String>) {
    let tasks = src
        .lines()
        .filter_map(|l| l.trim().strip_prefix("action ")?.strip_suffix(';'))
        .filter(|t| !t.contains(' ') && !t.contains(':'))
        .map(str::to_string)
        .collect();
    let results = src
        .split("part ")
        .skip(1)
        .filter(|seg| seg.starts_with(char::is_alphanumeric))
        .filter_map(|seg| {
            let name = seg.split_whitespace().next()?;
            seg.split_once(" : ")
                .filter(|(_, rest)| rest.starts_with("TestResult"))
                .map(|_| name.to_string())
        })
        .collect();
    (tasks, results)
}

/// Guard: a sprint the work has MOVED ON FROM may not carry an unstamped task (D0260).
///
/// Sprint 483's story was finished and verified, its result never appended, and the frontier
/// therefore served finished work as ready for three weeks — the one miss in 496 sprints, found
/// only when D0258's priority-assessment step first read the frontier's head item by item.
///
/// The check is a RATCHET, not a new burden: all 496 sprint files already stamp every task, so
/// this guard starts at zero violations and exists to keep a perfect record perfect. It is
/// scoped to avoid the issue272 failure — blocking legitimate work to prevent an illegitimate
/// state. The HIGHEST-numbered sprint is exempt, because an in-progress sprint has unstamped
/// tasks by definition and gating it would make the guard a lockout. Opening sprint N+1 is the
/// objective, self-declared event that says N is no longer in progress.
#[must_use]
pub fn sprint_closure(root: &Path) -> GuardReport {
    let files = keel_model::corpus::collect_sysml(&root.join(".tracking").join("delivery"));
    let number = |p: &Path| -> Option<u32> {
        p.file_name()?
            .to_str()?
            .strip_prefix("sprint")?
            .split(|c: char| !c.is_ascii_digit())
            .next()?
            .parse()
            .ok()
    };
    // The in-progress exemption: the single highest sprint number present.
    let newest = files.iter().filter_map(|p| number(p)).max();
    let mut violations = Vec::new();
    let mut scanned = 0usize;
    for path in &files {
        let Ok(src) = keel_model::corpus::read_to_string(path) else { continue };
        if !src.contains("action def ") {
            continue;
        }
        if number(path).is_some() && number(path) == newest {
            continue; // in progress — see doc comment
        }
        let (tasks, results) = sprint_tasks_and_results(&src);
        scanned += tasks.len();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
        for t in tasks {
            if !results.iter().any(|r| r.starts_with(&t)) {
                violations.push(format!(
                    "{name}: task `{t}` has no TestResult, but work has moved on to a later sprint \
                     — an unstamped task is served as READY forever, so finished work is \
                     indistinguishable from open work (D0260/sprint483)"
                ));
            }
        }
    }
    violations.sort();
    GuardReport { name: "sprint-closure", scanned, warnings: Vec::new(), violations }
}

// ── ceremony guard (gate ordering + retro-scan evidence) ───────────────────────────────────────
// The gate order is `keel_model::orient::gate_order(root)`, read from the workflow chain the process steps
// bind (D0435) - never a constant here.
// The scan wording is `keel_write::write::RETRO_SCAN_EVIDENCE`, ONE home shared with the write that
// refuses a retro result without it (issue566) - this guard reads a Retro already on the tree.
pub(crate) const CEREMONY_GRANDFATHERED: &[&str] = &["sprint11_nativeSpikes"];

/// Gate names (of `order`) with a `verification <…{G}Gate>` declaration in the text.
pub(crate) fn gates_defined(text: &str, order: &[String]) -> HashSet<String> {
    let mut out = HashSet::new();
    for (idx, _) in text.match_indices("verification ") {
        let after = &text[idx + "verification ".len()..];
        let name: String = after.chars().take_while(|c| keel_model::algo::is_word(*c)).collect();
        for g in order {
            if name.ends_with(&format!("{g}Gate")) {
                out.insert(g.clone());
            }
        }
    }
    out
}

/// Gate names (of `order`) with a WRITTEN `part <…{G}Gate…R\d+> : TestResult` of any verdict
/// (`orient::gate_has_result`, D0437 / issue544). The guard checks SEQUENCE, not done-ness: an AI-judged
/// inspect gate lands `VerdictKind::proposed` under D0312 B and was still recorded in its turn; reading
/// only `pass` here made every AI-run sprint red at its Implement gate (issue470), and reading `pass`
/// and `proposed` made sprint708's honestly FAILED closeOut read as unrecorded and its Retro as out of
/// turn (issue544) - the red landed on the one record that told the truth (D0098).
pub(crate) fn gates_recorded(text: &str, order: &[String]) -> HashSet<String> {
    order.iter().filter(|g| keel_model::textscan::gate_has_result(text, g)).cloned().collect()
}

/// Ordering violations: a recorded gate while an earlier DEFINED gate is unrecorded.
pub(crate) fn ordering_violations(order: &[String], defined: &HashSet<String>, recorded: &HashSet<String>) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (i, g) in order.iter().enumerate() {
        if !recorded.contains(g) {
            continue;
        }
        for earlier in order.iter().take(i) {
            if defined.contains(earlier) && !recorded.contains(earlier) {
                out.push((g.clone(), earlier.clone()));
            }
        }
    }
    out
}

/// The retro Test's NAME when Retro is recorded but that Test's `procedureText` records no
/// avoidable-issue scan (issue011); `None` when it does, or when no Retro is recorded. Anchors on
/// the `verification …RetroGate… : Test` declaration (not any `RetroGate` substring, which can
/// appear in other gates' prose). The predicate is `keel_write::write::retro_scan_recorded`, the one the
/// write refuses on (issue566) - the surface this reads is the Test, so the message names the Test.
pub(crate) fn retro_scan_missing(text: &str, recorded: &HashSet<String>) -> Option<String> {
    if !recorded.contains("Retro") {
        return None;
    }
    for (idx, _) in text.match_indices("verification ") {
        let after = &text[idx + "verification ".len()..];
        let name: String = after.chars().take_while(|c| keel_model::algo::is_word(*c)).collect();
        if !name.contains("RetroGate") {
            continue;
        }
        let rest = after.strip_prefix(name.as_str())?;
        let pt = rest.find("procedureText")?;
        let after_pt = &rest[pt..];
        let q = after_pt.find('"')?;
        let body: String = after_pt[q + 1..].chars().take_while(|c| *c != '"').collect();
        return (!keel_write::write::retro_scan_recorded(&body)).then_some(name);
    }
    None // no retro verification declaration found
}

/// Guard: ceremony gates are recorded in order, and a recorded Retro carries its scan evidence.
///
/// Within a delivery file, no ceremony gate is recorded (`pass` or `proposed`, D0437) while an earlier
/// DEFINED gate is unrecorded; a recorded Retro records avoidable-issue scan evidence. Mirrors
/// `validate_ceremony.py`. Whether a recorded gate is PASSED is orient's question, not this guard's.
/// The order is the workflow chain the process steps bind (D0435); a tree that binds no gate has no
/// ceremony order, and the guard WARNS that the delivery records are unenforceable-by-step rather
/// than enforcing a sequence nobody declared.
#[must_use]
pub fn ceremony(root: &Path) -> GuardReport {
    let files = keel_model::corpus::collect_sysml(&root.join(".tracking").join("delivery"));
    let mut warnings = Vec::new();
    let mut violations = Vec::new();
    let order = keel_model::orient::gate_order(root);
    if order.is_empty() {
        warnings.push(format!(
            "no ProcessStep binds a `gate:<phase>` check, so no ceremony order is declared - {} delivery record(s) are unenforceable-by-step (D0435)",
            files.len()
        ));
        return GuardReport { name: "ceremony", scanned: files.len(), warnings, violations };
    }
    let grandfathered: HashSet<&str> = CEREMONY_GRANDFATHERED.iter().copied().collect();
    for path in &files {
        let Ok(text) = keel_model::corpus::read_to_string(path) else { continue };
        let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let recorded = gates_recorded(&text, &order);
        let mut defined = gates_defined(&text, &order);
        defined.extend(recorded.iter().cloned());
        let viols = ordering_violations(&order, &defined, &recorded);
        if !viols.is_empty() {
            let detail = viols.iter().map(|(g, e)| format!("{g} recorded but {e} (earlier) unrecorded")).collect::<Vec<_>>().join("; ");
            if grandfathered.contains(stem.as_str()) {
                warnings.push(history_line(&format!("{stem}: {detail} (grandfathered, pre-issue010)")));
            } else {
                violations.push(format!("{stem}: {detail}"));
            }
        }
        if let Some(test) = retro_scan_missing(&text, &recorded).filter(|_| !grandfathered.contains(stem.as_str())) {
            // issue566: the message names the surface read - the Test's procedureText - never the result.
            violations.push(format!(
                "{stem}: Retro Test {test} records no avoidable-issue scan in its procedureText (issue011; one of: {})",
                keel_write::write::RETRO_SCAN_EVIDENCE.join(" | ")
            ));
        }
    }
    GuardReport { name: "ceremony", scanned: files.len(), warnings, violations }
}

/// Guard: every newly-added delivery Story declares its `#CharteredBy` edge (D0068).
///
/// CONTRACT (D0107): sourced from the declared `charterRule` (an `EdgeRule` with `newlyAdded` git scope)
/// — the single gate source; the bespoke charter predicate was retired after parity (sprints 178-183).
#[must_use]
pub fn charter(root: &Path) -> GuardReport {
    match keel_view::view::rule_violations_opt(root, "charterRule") {
        Ok(Some((scanned, uncharted))) => {
            let violations = uncharted
                .into_iter()
                .map(|s| format!("Story '{s}' has no #CharteredBy edge — a delivery Story must charter to its originating Decision/Need/Requirement, or (a research spike) to an Issue/proposed-Decision (D0068/issue055)"))
                .collect();
            GuardReport { name: "charter", scanned, warnings: Vec::new(), violations }
        }
                // D0136/issue090: an ABSENT rule means the project has not ADOPTED this control —
        // it has not violated it. Warn (never silent, so deleting a rule to dodge the gate is
        // visible) and pass; a MALFORMED rule still fails via Err below.
        Ok(None) => GuardReport { name: "charter", scanned: 0, warnings: vec!["declared rule `charterRule` is not present — this control is NOT ADOPTED by this project, so nothing was checked (D0136/issue090)".to_string()], violations: Vec::new() },
Err(e) => GuardReport { name: "charter", scanned: 0, warnings: Vec::new(), violations: vec![format!("error reading charter rule: {e}")] },
    }
}

/// Guard: no comment may claim a gate is PENDING when its acceptance result PASSES (issue140).
///
/// Prose frozen at authoring time contradicting the record beside it is the D0018 defect inside the model
/// files, and it is not harmless: `keel-viewer.sysml` carried `PENDING human acceptance of N-18 — NOT yet
/// accepted` on the line ABOVE N-18's passing acceptance result, and I believed the comment over the
/// record and published a false claim in a critique. A reader has no reason to distrust a comment sitting
/// next to the thing it describes.
///
/// PRECISE, not heuristic: fires only when a comment within three lines of a PASSING `*Accept*R*`
/// `TestResult` claims the opposite. A comment saying PENDING beside a gate that has NOT been signed is
/// correct and is left alone, which is what keeps this from firing on honest work-in-progress.
#[must_use]
pub fn stale_gate_prose(root: &Path) -> GuardReport {
    const CLAIMS: [&str; 3] = ["PENDING", "NOT yet accepted", "proposed/unaccepted"];
    let mut violations = Vec::new();
    let mut scanned = 0;
    for dir in [".tracking", ".engine", ".knowledge"] {
        for f in keel_model::corpus::collect_sysml(&root.join(dir)) {
            let Ok(text) = keel_model::corpus::read_to_string(&f) else { continue };
            let lines: Vec<&str> = text.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                // A passing acceptance RESULT: the record a nearby comment must not contradict.
                let is_pass_result = line.contains(": TestResult") && line.contains("VerdictKind::pass") && line.contains("Accept");
                if !is_pass_result {
                    continue;
                }
                scanned += 1;
                let lo = i.saturating_sub(3);
                let hi = (i + 4).min(lines.len());
                // `.get` rather than a slice: a panic inside a GUARD would take the whole gate down and
                // report nothing, which is the worst possible failure mode for a control.
                for probe in lines.get(lo..hi).unwrap_or(&[]) {
                    let t = probe.trim_start();
                    if !t.starts_with("//") {
                        continue;
                    }
                    if let Some(claim) = CLAIMS.iter().find(|c| t.contains(**c)) {
                        violations.push(format!(
                            "{}:{}: a comment says `{claim}` within three lines of a PASSING acceptance result — the prose contradicts the record it sits beside, and a reader has no reason to distrust it (issue140). State what IS; delete the stale note rather than annotating it",
                            relpath(root, &f),
                            i + 1
                        ));
                        break;
                    }
                }
            }
        }
    }
    violations.sort();
    violations.dedup();
    GuardReport { name: "stale-gate-prose", scanned, warnings: Vec::new(), violations }
}

// ── retro-backlog guard (a retro finding that terminates in prose) ────────────────────────────────

/// Violations for staged sprint records whose RETRO names findings that this commit does not track.
///
/// # Why this is the third shape of the check, and why the first two were both wrong
///
/// The first version was satisfied when the commit CO-STAGED any tracked file — and every commit
/// stages issues.sysml for something, so five retros reached zero items (issue189). The second
/// version, the fix for that, required the retro's text to NAME an item — and every sprint file names
/// the task it delivered, so the check was satisfied by construction; it also examined a retro only
/// if the text contained the literal tokens AVOIDABLE-ISSUE or LESSON:, so a retro that said FINDING
/// was never examined at all. Six findings on 2026-09-01 went through it that way (issue335).
///
/// This version asks the question the guard was always for: does THIS COMMIT add a tracked item that
/// THIS RETRO names? `staged_added_items` is the set of `part issueNNN` / `action dcX;` declarations
/// in the staged diff's added lines; a retro is clean when its own text names one of them, or when it
/// carries an explicit no-item justification. Every retro gate is examined — a `method = analyze`
/// gate titled retro IS a findings record, whatever words it uses.
///
/// # The fourth shape (issue364, D0293): a justification must name what tracks the finding
///
/// The third shape let a retro carry "no new item - already tracked" and pass with nothing checked -
/// the phrase was a substring match consulting no item. Two retros in two days said a finding was
/// tracked elsewhere when nothing tracked it, and both passed; the human's question found them. Now a
/// justified retro must NAME an item that exists (added by this commit, or already in the tree) - an
/// `issueNNN`, a `dcTask`, or a `dNNNN` Decision. Whether the named item actually covers the finding
/// stays the reader's judgment; that the claim points at something real is the guard's.
pub(crate) fn retro_backlog_violations(added_items: &[String], known_items: &[String], sprint_texts: &[(String, String)]) -> Vec<String> {
    let mut out = Vec::new();
    for (path, text) in sprint_texts {
        for retro in retro_texts(text) {
            let lower = retro.to_lowercase();
            let named = named_items(&retro);
            let exists = |n: &String| added_items.iter().any(|a| a == n) || known_items.iter().any(|k| k == n);
            if let Some(phrase) = RETRO_NO_ITEM_JUSTIFICATIONS.iter().find(|j| lower.contains(*j)) {
                if named.iter().any(exists) {
                    continue;
                }
                out.push(format!(
                    "{path}: the retro says '{phrase}' but names no existing item that tracks the finding ({}) — a justification must point at something real, written as the tree names it: `issueNNN`, `dcTaskName`, `d0NNN` (or `D0NNN`) - the item that carries it (D0131/D0293/issue424; issue364: two retros claimed 'already tracked' about untracked findings and passed)",
                    if named.is_empty() { "it names no item at all".to_string() } else { format!("it names {}, none of which exists", named.join(", ")) }
                ));
                continue;
            }
            if named.iter().any(|n| added_items.iter().any(|a| a == n)) {
                continue;
            }
            out.push(format!(
                "{path}: the retro records findings but this commit ADDS no tracked item the retro names ({}) — a finding must become a tracked, prioritized item in the SAME commit, or the retro must say why none is needed (D0131; issue335: naming an existing task is what every retro does, and is not tracking a finding)",
                if named.is_empty() { "it names no item at all".to_string() } else { format!("it names {}, none of which this commit adds", named.join(", ")) }
            ));
        }
    }
    out
}

/// `part issueNNN` and `action dcX;` declarations ADDED by the staged diff.
pub(crate) fn added_items(root: &Path, read: ChangeRead) -> Vec<String> {
    let has_head = !git_stdout(root, &["rev-parse", "--verify", "-q", "HEAD"]).trim().is_empty();
    let mut text = if read == ChangeRead::WorkingTree && has_head {
        git_stdout(root, &["diff", "HEAD", "-U0", "--", ".tracking"])
    } else {
        git_stdout(root, &["diff", "--cached", "-U0", "--", ".tracking"])
    };
    if read == ChangeRead::WorkingTree {
        // An untracked record under .tracking is added whole: every line of it is a `+` line.
        for p in git_stdout(root, &["ls-files", "--others", "--exclude-standard", "--full-name", "-z", "--", ".tracking"]).split('\0').filter(|p| !p.is_empty()) {
            if let Ok(body) = keel_model::corpus::read_to_string(root.join(p)) {
                for l in body.lines() {
                    text.push('+');
                    text.push_str(l);
                    text.push('\n');
                }
            }
        }
    }
    let mut items = Vec::new();
    for line in text.lines().filter(|l| l.starts_with('+') && !l.starts_with("+++")) {
        let l = line[1..].trim_start();
        if let Some(rest) = l.strip_prefix("part ") {
            if rest.starts_with("issue") {
                items.push(rest.chars().take_while(char::is_ascii_alphanumeric).collect());
            }
        } else if let Some(rest) = l.strip_prefix("action ") {
            if rest.starts_with("dc") {
                items.push(rest.chars().take_while(char::is_ascii_alphanumeric).collect());
            }
        }
    }
    items
}

/// Guard: a sprint retro's findings must become tracked items, not prose (issue085 / D0130).
///
/// Sprint 247's retro named three avoidable issues; only one had a control, and the other two were
/// written into CLAUDE.md prose and the AI's own memory — OUTSIDE the model, carrying no severity, no
/// priority, no resolver and no id, invisible to orient and the burndown, inside the very ceremony
/// meant to prevent recurrence. That is the prose-shadow-truth D0018 forbids.
///
/// Git-diff-aware, heuristic and WARNING-level — the `doc-sync` (D0113) shape. It is satisfied either
/// by co-recording a tracked item or by SAYING why none is needed, so what it really enforces is that
/// the choice is explicit. Reads the working tree, which equals the index for staged-and-unmodified
/// files (the pre-commit case); a partially-staged sprint file could be misread, which is one more
/// reason this warns rather than blocks.
#[must_use]
pub fn retro_backlog(root: &Path) -> GuardReport {
    let read = ChangeRead::current();
    let changed = changed_files(root, read);
    let sprint_texts: Vec<(String, String)> = changed
        .iter()
        .filter(|p| p.contains(".tracking/delivery/sprint") && std::path::Path::new(p).extension().is_some_and(|e| e.eq_ignore_ascii_case("sysml")))
        .filter_map(|p| keel_model::corpus::read_to_string(root.join(p)).ok().map(|t| (p.clone(), t)))
        // A staged sprint whose RETRO text is unchanged from HEAD is not this commit's retro (D0331):
        // correcting a past sprint's receipt (issue283) re-staged sprint469 and the guard judged a
        // 2026-08-29 retro by a rule written afterwards. Only a retro that moved is this commit's to answer for.
        .filter(|(p, t)| {
            let at_head = git_stdout(root, &["show", &format!("HEAD:{p}")]);
            at_head.is_empty() || retro_texts(&at_head) != retro_texts(t)
        })
        .collect();
    let scanned = sprint_texts.len();
    let added = added_items(root, read);
    // What a justification may point at: every declared task, every Issue (open or done), every Decision.
    let mut known: Vec<String> = declared_task_names(root).into_iter().collect();
    known.extend(keel_view::view::all_issue_names(root, &HashSet::new()).unwrap_or_default());
    if let Ok(rd) = std::fs::read_dir(root.join(".engine").join("decisions")) {
        known.extend(rd.flatten().filter_map(|e| e.file_name().to_str().and_then(|f| f.get(..4)).filter(|n| n.chars().all(|c| c.is_ascii_digit())).map(|n| format!("d{n}"))));
    }
    GuardReport { name: "retro-backlog", scanned, warnings: vec![read_line(read)], violations: retro_backlog_violations(&added, &known, &sprint_texts) }
}

// ── priority-inversion guard (recorded order disagreeing with recorded severity) ──────────────────

/// Guard: a ready item outranks work that resolves a >= High Issue (issue084 / D0130).
///
/// D0052 makes backlog DECLARATION ORDER the priority and requires the AI to auto-follow the ranked
/// frontier, but nothing compared recorded ORDER against recorded SEVERITY — so a mis-ordered backlog
/// looked exactly like a curated one. It was mis-ordered: `keelArchViews` (issue069, Low) ranked FIRST
/// because an earlier session appended it to the end of a COMPLETED block, while
/// `dcStaleKernelInstanceGate` (issue081, High) ranked 14th.
///
/// WARNING-level and never blocking: priority IS a human judgment and deferring a High item behind an
/// enabler can be entirely correct. The point is to make the trade-off visible rather than leave it to
/// whoever last appended to the file. A compute error IS a violation.
///
/// D0429 (issue345): "if that is deliberate say so" now names where - a `#PrioritizedBy` dependency
/// from the item to the Statement or Decision that ranks it. An item carrying one is not warned on
/// (`keel show priority` lists it under `recorded` with the citation); an edge to any other type IS a
/// violation, because a rank cited to a work item records nothing.
#[must_use]
pub fn priority_inversion(root: &Path) -> GuardReport {
    match keel_view::priority::priority_inversions(root) {
        Ok(inv) => {
            let warnings = inv
                .pairs
                .iter()
                .map(|(lower, high, sev)| {
                    format!("{lower} outranks {high}, whose computed class is {sev} (a resolved Issue's severity, or a finding retros keep naming as already tracked - D0311; `keel show priority` shows which) — if that is deliberate record it: `#PrioritizedBy dependency from {lower} to <stNNN|dNNNN>;` citing the Statement or Decision that ranks it (D0429); otherwise reorder the backlog (D0052: declaration order IS priority; reordering is how you reprioritize)")
                })
                .collect();
            GuardReport { name: "priority-inversion", scanned: inv.pairs.len() + inv.recorded.len() + inv.violations.len(), warnings, violations: inv.violations }
        }
        Err(e) => GuardReport { name: "priority-inversion", scanned: 0, warnings: Vec::new(), violations: vec![format!("error computing priority inversions: {e}")] },
    }
}

/// Guard 40: no `.sysml` in the model carries the scaffold's FILL-ME token (dcSprintScaffold).
///
/// `keel record sprint` writes every judgment-bearing text as [`keel_write::scaffold::PLACEHOLDER`] so the
/// skeleton is honest about being unfilled — and THIS guard is what makes that honesty enforceable:
/// an unfilled scaffold cannot pass a gate or be committed, by construction rather than diligence.
/// Also in the fast per-edit tier (`keel gate --fast`), so the rejection lands at edit time.
#[must_use]
pub fn scaffold_placeholder(root: &Path) -> GuardReport {
    let mut files = keel_model::corpus::collect_sysml(&root.join(".tracking"));
    files.extend(keel_model::corpus::collect_sysml(&root.join(".engine")));
    let mut scanned = 0usize;
    let mut violations = Vec::new();
    for path in &files {
        let Ok(text) = keel_model::corpus::read_to_string(path) else { continue };
        scanned += 1;
        let rel = relpath(root, path);
        for (n, line) in text.lines().enumerate() {
            if line.contains(keel_write::scaffold::PLACEHOLDER) {
                violations.push(format!(
                    "{rel}:{}: unfilled scaffold text — fill it in; a placeholder is not a recorded judgment",
                    n + 1
                ));
            }
        }
    }
    GuardReport { name: "scaffold-placeholder", scanned, warnings: Vec::new(), violations }
}

#[cfg(test)]
mod retro_backlog_tests {
    #[test]
    fn retro_backlog_fails_when_a_finding_is_neither_tracked_in_this_commit_nor_justified() {
        use super::retro_backlog_violations as check;
        // A sprint file whose RETRO gate carries the given text. The DoD line names the delivered
        // task, as every real one does — which is what made the second shape of this guard vacuous.
        let sprint = |t: &str| {
            vec![(
                ".tracking/delivery/sprint999_x.sysml".to_string(),
                format!(
                    "package S {{
verification storyDoD : Test {{ :>> method = VerificationMethod::test; :>> procedureText = \"DELIVERED: dcTheWork.\"; }}
                     verification xRetroGate : Test {{ :>> title = \"retro gate\"; :>> method = VerificationMethod::analyze; :>> procedureText = \"{t}\"; }}
}}
"
                ),
            )]
        };
        let nothing_added: Vec<String> = Vec::new();
        let added_issue073 = vec!["issue073".to_string()];

        // THREE SHAPES OF THIS GUARD, and the two earlier ones are kept here as regressions.
        // Shape 1 (pre-issue189): co-staging a tracked file satisfied it — every commit stages one.
        // Shape 2 (D0172): the retro's text had to NAME an item — every sprint file names the task it
        //   delivered, and the check only ran on the tokens AVOIDABLE-ISSUE / LESSON: (issue335).
        // Shape 3 (D0279): this commit must ADD an item the retro's own text names, or say why not.
        assert_eq!(check(&nothing_added, &[], &sprint("AVOIDABLE-ISSUE 1: piping hung the kernel.")).len(), 1);
        // The sprint-513 case: FINDING, not LESSON — shape 2 never looked. Shape 3 does.
        assert_eq!(check(&nothing_added, &[], &sprint("FINDING: piping hung the kernel.")).len(), 1);
        // Naming the delivered task is what every retro does; it tracks nothing.
        assert_eq!(check(&nothing_added, &[], &sprint("FINDING: piping hung the kernel. Delivered dcTheWork.")).len(), 1);
        // Naming an item THIS COMMIT ADDS -> clean.
        assert!(check(&added_issue073, &[], &sprint("FINDING: piping hung the kernel - tracked as issue073.")).is_empty());
        // Explicitly justified as needing none -> clean. The obligation is a STATED choice.
        assert!(check(&nothing_added, &["dcPreBashAdvisory".to_string()], &sprint("AVOIDABLE-ISSUE 1: x — no new item, already guarded by dcPreBashAdvisory.")).is_empty());
        // A retro with no findings language still gets examined; it names nothing and justifies
        // nothing, so it is a violation — a retro that records no finding and no reason is exactly
        // the empty ceremony D0131 exists to prevent.
        assert_eq!(check(&nothing_added, &[], &sprint("WELL: everything went fine.")).len(), 1);
    }
}

#[cfg(test)]
mod scaffold_placeholder_tests {
    use super::scaffold_placeholder;
    use keel_write::scaffold::{sprint, test_support::temp_root, PLACEHOLDER};

    /// Guard 40 rejects the scaffold until it is filled — the whole point of the marker.
    #[test]
    fn the_placeholder_guard_rejects_an_unfilled_scaffold_and_passes_a_filled_one() {
        let root = temp_root("guard");
        let path = sprint(&root, 999, "guardRun", "d9997", 2, "claudeOpus5").expect("scaffold");
        let report = scaffold_placeholder(&root);
        assert!(!report.violations.is_empty(), "an unfilled scaffold must be rejected");
        let filled = std::fs::read_to_string(&path).expect("read").replace(PLACEHOLDER, "filled in");
        std::fs::write(&path, filled).expect("fill");
        assert!(scaffold_placeholder(&root).violations.is_empty(), "a filled scaffold passes");
    }
}

#[cfg(test)]
mod retro_tie_tests {
    use super::{named_items, retro_backlog_violations, retro_texts};

    /// A sprint file whose RETRO gate carries `finding`. The `DoD` line names the delivered task — as
    /// every real sprint file does — which is exactly what defeated the previous shape of this guard.
    fn sprint_with_retro(finding: &str) -> String {
        format!(
            "package S {{
             verification storyXDoD : Test {{ :>> method = VerificationMethod::test; :>> procedureText = \"DELIVERED BACKLOG ITEMS: dcDeliveredThing.\"; }}
             verification xRetroGate : Test {{ :>> title = \"Sprint X retro gate\"; :>> method = VerificationMethod::analyze; :>> procedureText = \"{finding}\"; }}
             }}
"
        )
    }

    /// THE HOLE THAT LET SIX FINDINGS THROUGH IN ONE DAY (issue335). The retro says FINDING, not
    /// LESSON — so the old vocabulary gate never examined it — and the file names the delivered task,
    /// so the old naming check was satisfied by construction. Both were wrong; this must FAIL.
    #[test]
    fn a_finding_in_any_words_with_no_new_item_is_a_violation() {
        let text = sprint_with_retro("TWO FINDINGS. (1) the guard checks that an edge EXISTS, not that the resolver fits. (2) the same per-instance repair recurred.");
        let v = retro_backlog_violations(&[], &[], &[("s.sysml".to_string(), text)]);
        assert_eq!(v.len(), 1, "a retro with findings and no NEW tracked item must be a violation: {v:?}");
        assert!(v[0].contains("names no item at all"), "{v:?}");
    }

    /// issue189's hole, kept as a regression: naming the DELIVERED task is what every retro does and
    /// tracks nothing. Only an item this commit ADDS counts.
    #[test]
    fn naming_the_delivered_task_does_not_track_a_finding() {
        let text = sprint_with_retro("FINDING: the counter was wrong. This sprint delivered dcDeliveredThing, which is unrelated.");
        let v = retro_backlog_violations(&[], &[], &[("s.sysml".to_string(), text)]);
        assert_eq!(v.len(), 1, "an EXISTING task's name must not satisfy the check: {v:?}");
        assert!(v[0].contains("dcDeliveredThing") && v[0].contains("none of which this commit adds"), "{v:?}");
    }

    /// The satisfying condition: the retro names an item and THIS COMMIT adds it.
    #[test]
    fn a_finding_whose_named_item_this_commit_adds_is_clean() {
        let text = sprint_with_retro("FINDING: the counter was wrong; recorded as issue188 with a resolver.");
        let added = vec!["issue188".to_string()];
        assert!(retro_backlog_violations(&added, &[], &[("s.sysml".to_string(), text)]).is_empty());
        let text = sprint_with_retro("LESSON: shell mangling again - now tracked as dcAuthorViaWriteTool.");
        let added = vec!["dcAuthorViaWriteTool".to_string()];
        assert!(retro_backlog_violations(&added, &[], &[("s.sysml".to_string(), text)]).is_empty());
    }

    /// The obligation is a STATED choice, not always-an-item: an explicit justification is clean.
    #[test]
    fn an_explicit_no_item_justification_is_clean_when_it_names_the_item_that_tracks_it() {
        // Naming an EXISTING item (here a known task) beside the justification is what makes the claim
        // checkable; the phrase alone is no longer enough (issue364, D0293).
        let text = sprint_with_retro("FINDING: a one-off typo; no new item - dcPostEditGate already catches this class.");
        let known = vec!["dcPostEditGate".to_string()];
        assert!(retro_backlog_violations(&[], &known, &[("s.sysml".to_string(), text)]).is_empty());
        // ...and a Decision counts as the tracking item too.
        let text = sprint_with_retro("no new item - already tracked: the stale help is recorded in d0283.");
        let known = vec!["d0283".to_string()];
        assert!(retro_backlog_violations(&[], &known, &[("s.sysml".to_string(), text)]).is_empty());
    }

    /// issue364, second shape: a retro whose text contains the word "verification" was cut in half by the
    /// old extraction and never examined - the guard passed a retro it had not read.
    #[test]
    fn a_retro_that_mentions_verification_is_still_examined() {
        let text = sprint_with_retro("FINDING: the verification of X was skipped; no item named anywhere here.");
        assert_eq!(retro_texts(&text).len(), 1, "the retro must be extracted whole: {:?}", retro_texts(&text));
        let v = retro_backlog_violations(&[], &[], &[("s.sysml".to_string(), text)]);
        assert_eq!(v.len(), 1, "and examined: {v:?}");
    }

    /// THE CONTROL for issue364: 'already tracked' must point at something real.
    #[test]
    fn already_tracked_with_no_named_item_is_a_violation() {
        let text = sprint_with_retro("no new item - already tracked: the finding belongs to the verification revamp.");
        let v = retro_backlog_violations(&[], &["dcVerificationByAuthority".to_string()], &[("s.sysml".to_string(), text)]);
        assert_eq!(v.len(), 1, "{v:?}");
        assert!(v[0].contains("'already tracked'") && v[0].contains("names no item at all"), "{v:?}");
    }

    /// ...and naming an item that does NOT exist is the same violation, said differently.
    #[test]
    fn already_tracked_naming_a_nonexistent_item_is_a_violation() {
        let text = sprint_with_retro("no new item - already tracked in issue999, which covers it.");
        let v = retro_backlog_violations(&[], &["issue001".to_string()], &[("s.sysml".to_string(), text)]);
        assert_eq!(v.len(), 1, "{v:?}");
        assert!(v[0].contains("issue999") && v[0].contains("none of which exists"), "{v:?}");
    }

    /// The two retros that motivated the fix, re-scanned: sprint 526's named a real item (issue336) and
    /// so passes the FORM check - the guard cannot judge that issue336 does not cover the finding, and
    /// says so in its doc; sprint 527's named dcVerificationByAuthority and passes likewise. What both
    /// would have FAILED is the shape they took in the human's reading: a claim with no item at all.
    #[test]
    fn a_justification_that_names_a_real_item_passes_form_and_leaves_relevance_to_the_reader() {
        let text = sprint_with_retro("no new item - already tracked: that gap belongs to issue336.");
        assert!(retro_backlog_violations(&[], &["issue336".to_string()], &[("s.sysml".to_string(), text)]).is_empty());
    }

    /// Only the RETRO gate is examined — a `DoD` or review gate mentioning a finding-like word is not a
    /// findings record, and a sprint with no retro gate yields nothing to check.
    #[test]
    fn only_retro_gates_are_examined() {
        let no_retro = "package S {
verification storyXDoD : Test { :>> method = VerificationMethod::test; :>> procedureText = \"FINDING: not a retro.\"; }
}
";
        assert!(retro_texts(no_retro).is_empty());
        assert!(retro_backlog_violations(&[], &[], &[("s.sysml".to_string(), no_retro.to_string())]).is_empty());
    }

    /// Word boundaries on the item tokens, as before.
    #[test]
    fn prose_lookalikes_do_not_count_as_items() {
        assert!(named_items("the dc motor issue was discussed at length").is_empty());
        assert!(named_items("reproduced changes").is_empty());
        assert_eq!(named_items("tracked as dcFooBar"), vec!["dcFooBar"]);
        assert_eq!(named_items("see issue123"), vec!["issue123"]);
        assert!(named_items("tissue42 is not an item").is_empty());
    }

    /// issue424: a Decision named the way every document in this repository names one - `D0388` - is
    /// read as naming `d0388`, the item's real name, so the exists check hits the tree. Only the
    /// Decision form folds: `Issue 42` and `DC` in prose are words, and `ID0388` is not a boundary.
    #[test]
    fn a_decision_named_in_uppercase_is_read_as_its_real_name() {
        assert_eq!(named_items("already tracked by D0388 and its probe rule"), vec!["d0388"]);
        assert_eq!(named_items("d0388 twice: D0388"), vec!["d0388", "d0388"]);
        assert!(named_items("Issue 42 and the DC team and ID0388").is_empty());
        assert_eq!(named_items("tracked in issue424 (D0131)"), vec!["issue424", "d0131"]);
    }

    /// The case the guard refused at sprint 627, constructed both ways (D0388 pair): the same retro,
    /// whose only named item is written `D0388`, PASSES when d0388 exists and FAILS when it does not -
    /// and the refusal quotes the forms it accepts in the case it accepts them.
    #[test]
    fn a_retro_justified_by_an_uppercase_decision_passes_when_it_exists_and_fails_when_it_does_not() {
        let text = sprint_with_retro("no new item - already tracked by D0388 and its probe rule.");
        let known = vec!["d0388".to_string()];
        assert!(retro_backlog_violations(&[], &known, &[("s.sysml".to_string(), text.clone())]).is_empty(), "d0388 exists: the uppercase form names it");
        let v = retro_backlog_violations(&[], &["d0387".to_string()], &[("s.sysml".to_string(), text)]);
        assert_eq!(v.len(), 1, "{v:?}");
        assert!(v[0].contains("d0388") && v[0].contains("none of which exists"), "{v:?}");
        assert!(v[0].contains("`d0NNN` (or `D0NNN`)") && v[0].contains("`issueNNN`") && v[0].contains("`dcTaskName`"), "the refusal quotes the accepted forms: {v:?}");
    }
}

#[cfg(test)]
mod change_read_tests {
    use super::{is_read, read_line, ChangeRead, GuardReport};

    /// D0388 known-positive: the note is neither actionable nor history, and the report names it.
    #[test]
    fn the_read_note_is_folded_into_the_summary_not_counted_as_a_warning() {
        let r = GuardReport { name: "process-change", scanned: 1, warnings: vec![read_line(ChangeRead::WorkingTree)], violations: Vec::new() };
        assert!(is_read(&r.warnings[0]));
        assert_eq!(r.actionable().count(), 0, "the note is not a warning a reader acts on");
        assert_eq!(r.history().count(), 0, "nor counted history");
        assert_eq!(r.read_mode(), Some("working tree"));
        assert!(r.ok());
    }

    /// D0388 known-negative: a real warning beside the note still counts as one, and a report with no
    /// note has no read mode - the guards that read no diff say nothing about one.
    #[test]
    fn a_real_warning_still_counts_and_a_noteless_report_has_no_read_mode() {
        let r = GuardReport {
            name: "doc-sync",
            scanned: 0,
            warnings: vec!["a doc claim moved".to_owned(), read_line(ChangeRead::Index)],
            violations: Vec::new(),
        };
        assert_eq!(r.actionable().count(), 1);
        assert_eq!(r.read_mode(), Some("index"));
        let plain = GuardReport { name: "charter", scanned: 3, warnings: Vec::new(), violations: Vec::new() };
        assert_eq!(plain.read_mode(), None);
    }

    /// The labels are the two words the summary line and the receipt key carry.
    #[test]
    fn the_two_labels_are_distinct_words() {
        assert_eq!(ChangeRead::Index.label(), "index");
        assert_eq!(ChangeRead::WorkingTree.label(), "working tree");
        assert_ne!(read_line(ChangeRead::Index), read_line(ChangeRead::WorkingTree));
    }
}
