//! Guard family `acceptance` - split from `keel-cli/src/guards.rs` by `scripts/split_guards.py` (sprint 733).
//!
//! Each guard's dispatch arm, code and tests sit together; the shared scanners, the runner and the
//! lock predicates are the crate root's (`super`). Nothing here was retyped: the text is guards.rs's,
//! with `crate::` paths pointing at the members and private items opened to the crate.

use super::*;

/// The `acceptance` family: every guard it dispatches, in `GUARD_NAMES` order, with the tier note each
/// arm carried in `run_one` (sprint 733). The root's union test holds these tables equal to `GUARD_NAMES`.
pub(crate) const FAMILY: Family = Family {
    name: "acceptance",
    arms: &[
        ("evidence-cited", evidence_cited),
        ("acceptance-events", acceptance_events),
        ("attestation-substance", attestation_substance), // hard (D0130/issue083) — a confirmation must attest something
        ("confirmation-authenticity", confirmation_authenticity), // hard (D0106/issue059) — rule-sourced
        ("attestation-authority", attestation_authority), // hard (D0092) — a human-only verdict judged by an AI
        ("impossible-evidence-date", impossible_evidence_dates),
        ("claim-ancestry", claim_ancestry), // issue229: claimedAt bounded by the introducing commit (D0013 applied to claims)
        ("judgment-request-quality", judgment_request_quality), // D0207: a fork must earn the ask
        ("acceptance-binds-to-text", acceptance_binds_to_text), // hard (issue341/D0308) - the text signed is the text carried
        ("plan-covers-step", plan_covers_step), // hard (D0396) - a PLAN-COVERED acceptance whose plan no longer holds
        ("consent-scope", consent_scope), // hard (D0439/issue460) - standing consent accepted nothing its text put outside it
        ("direction-cited", direction_cited), // hard (D0463/issue428) - the human's quoted direction links the Statement holding it
    ],
};

// ── acceptance-events guard (accepted Decision has a passing acceptance event) ─────────────────

/// Guard: an accepted Decision's acceptance event - and a rejected Decision's rejection event
/// (issue526) - must be HUMAN-judged (D0106/issue059).
///
/// The enforceable slice of strict process-boundedness (a sign-off is never AI-fabricated). Rule-sourced
/// from `confirmationAuthenticityRule` and `rejectionAuthenticityRule` (the CONTRACT pattern). D0106's
/// conversational parse-first part is inherently un-gatable at commit and stays reminder-enforced.
#[must_use]
pub fn confirmation_authenticity(root: &Path) -> GuardReport {
    let mut report = GuardReport { name: "confirmation-authenticity", scanned: 0, warnings: Vec::new(), violations: Vec::new() };
    for (rule, verdict, event) in [
        ("confirmationAuthenticityRule", "accepted", "acceptance"),
        ("rejectionAuthenticityRule", "rejected", "rejection"),
    ] {
        match keel_view::view::rule_violations_opt(root, rule) {
            Ok(Some((scanned, bad))) => {
                report.scanned += scanned;
                report.violations.extend(bad.into_iter().map(|d| {
                    format!("{d}: {verdict} but its {event} event is not human-judged — a sign-off must be a real human attestation, never AI-fabricated (D0106/D0016)")
                }));
            }
            // D0136/issue090: an ABSENT rule means the project has not ADOPTED this control —
            // it has not violated it. Warn (never silent, so deleting a rule to dodge the gate is
            // visible) and pass; a MALFORMED rule still fails via Err below.
            Ok(None) => report.warnings.push(format!("declared rule `{rule}` is not present — this control is NOT ADOPTED by this project, so nothing was checked (D0136/issue090)")),
            Err(e) => report.violations.push(format!("error reading {rule}: {e}")),
        }
    }
    // D0192 OPTION A substance half: when the attestation policy DECLARES a recording delegation for
    // acceptances, a delegated record must actually quote the human's conversational words. Sourced
    // from `delegatedAcceptanceSubstanceRule` (CONTRACT pattern, forward-only per the rule's cutoff).
    // A declared delegation with no substance rule is warned — the policy promised a check that is
    // not adopted — and no declared delegation means nothing is demanded here.
    if let Some(delegation) = keel_model::activation::recording_delegation(root, "decisionAcceptance") {
        match keel_view::view::rule_violations_opt(root, "delegatedAcceptanceSubstanceRule") {
            Ok(Some((_, bad))) => {
                for d in bad {
                    report.violations.push(format!(
                        "{d}: accepted on/after the delegation cutoff but its acceptance event neither quotes the human's words (a single-quoted span) nor cites their gesture — a record made under the {delegation} recording delegation must carry its channel evidence"
                    ));
                }
            }
            Ok(None) => report.warnings.push(format!(
                "attestation-policy declares recording delegation {delegation} for decisionAcceptance but `delegatedAcceptanceSubstanceRule` is not declared — the delegation's substance check is NOT ADOPTED (D0192)"
            )),
            Err(e) => report.violations.push(format!("error reading delegatedAcceptanceSubstanceRule: {e}")),
        }
    }
    // D0198 OPTION A (quote receipts): the same contract pattern for confirmation FLIPS — when the
    // policy declares the confirmationRecord recording delegation, a human-judged flip after the
    // cutoff must quote the human itself or carry a companion `<test>Attest<N>` record that does.
    if let Some(delegation) = keel_model::activation::recording_delegation(root, "confirmationRecord") {
        match keel_view::view::rule_violations_opt(root, "delegatedConfirmationSubstanceRule") {
            Ok(Some((_, bad))) => {
                for d in bad {
                    report.violations.push(format!(
                        "{d}: a human-judged confirmation flip after the delegation cutoff carries no quote receipt — neither its own text nor a companion <test>Attest<N> record quotes the human's words (D0198 {delegation})"
                    ));
                }
            }
            Ok(None) => report.warnings.push(format!(
                "attestation-policy declares recording delegation {delegation} for confirmationRecord but `delegatedConfirmationSubstanceRule` is not declared — the delegation's substance check is NOT ADOPTED (D0198)"
            )),
            Err(e) => report.violations.push(format!("error reading delegatedConfirmationSubstanceRule: {e}")),
        }
    }
    report
}

/// Guard: every `status=accepted` Decision carries a passing `dNNNNAcceptR1` event (D0066).
#[must_use]
pub fn acceptance_events(root: &Path) -> GuardReport {
    // CONTRACT (D0107): sourced from the declared acceptanceEventRule (single gate source).
    match keel_view::view::rule_violations_opt(root, "acceptanceEventRule") {
        Ok(Some((total, mut missing))) => {
            missing.sort();
            let violations = missing
                .into_iter()
                .map(|d| format!("{d}: accepted but no passing acceptance event (D0066)"))
                .collect();
            GuardReport { name: "acceptance-events", scanned: total, warnings: Vec::new(), violations }
        }
                // D0136/issue090: an ABSENT rule means the project has not ADOPTED this control —
        // it has not violated it. Warn (never silent, so deleting a rule to dodge the gate is
        // visible) and pass; a MALFORMED rule still fails via Err below.
        Ok(None) => GuardReport { name: "acceptance-events", scanned: 0, warnings: vec!["declared rule `acceptanceEventRule` is not present — this control is NOT ADOPTED by this project, so nothing was checked (D0136/issue090)".to_string()], violations: Vec::new() },
Err(e) => GuardReport {
            name: "acceptance-events",
            scanned: 0,
            warnings: Vec::new(),
            violations: vec![format!("error reading decisions: {e}")],
        },
    }
}

// ── sprint-coverage guard (done work is covered by a sprint) ────────────────────────────────────

/// Done tasks predating the sprint discipline (D0064); accepted as historical (never extend).
/// D0232's cutover. Both attestation guards bind FORWARD only: 983 `method=test` results and 38
/// coverage claims predate the convention, and retro-fitting evidence nobody captured would mean
/// inventing it — which is the very failure these guards exist to prevent. Same shape as D0198's
/// quote-receipt cutover.
pub(crate) const EVIDENCE_ENFORCED_FROM: &str = "2026-08-25";

// ── acceptance-binds-to-text (issue341 / D0308): the text signed is the text carried ─────────────

/// The three fields an acceptance is a judgment OF.
pub(crate) const ACCEPTED_FIELDS: [&str; 3] = ["decision", "rationale", "consequences"];

/// The latest `{dec}AcceptR<n>` result's `judgedAgainst` - the SHA the acceptance currently binds to.
pub(crate) fn latest_acceptance_sha(text: &str, dec: &str) -> Option<String> {
    latest_acceptance(text, dec).map(|(_, s)| s)
}

/// The latest `{dec}AcceptR<n>` result: its part name and its `judgedAgainst`.
pub(crate) fn latest_acceptance(text: &str, dec: &str) -> Option<(String, String)> {
    let prefix = format!("part {dec}AcceptR");
    let mut best: Option<(u32, String)> = None;
    for (i, _) in text.match_indices(&prefix) {
        let after = &text[i + prefix.len()..];
        let n: u32 = after.chars().take_while(char::is_ascii_digit).collect::<String>().parse().ok()?;
        let ja = after.find("judgedAgainst = \"")? + 17;
        let sha: String = after[ja..].chars().take_while(char::is_ascii_hexdigit).collect();
        if best.as_ref().is_none_or(|(bn, _)| n > *bn) {
            best = Some((n, sha));
        }
    }
    best.map(|(n, s)| (format!("{dec}AcceptR{n}"), s))
}

/// The commit that first carried `part <result>` in `rel`, if any is committed yet (D0329).
///
/// A re-binding is recorded against HEAD, then lands in the SAME commit as the corrected text (a
/// commit gate cannot let the edit through alone), so its `judgedAgainst` names the tree BEFORE the
/// edit and the text it vouches for is the tree that carried it - the same shape the introducing-commit
/// fallback already covers for a first acceptance. One `git log -S` per drifted Decision, which is rare.
pub(crate) fn result_landing_commit(root: &Path, rel: &str, result: &str) -> Option<String> {
    let out = git_stdout(root, &["log", "--format=%h", "-S", &format!("part {result} "), "--", rel]);
    out.lines().map(str::trim).rfind(|l| !l.is_empty()).map(str::to_string)
}

/// Pure core: which of the accepted fields differ between the text at acceptance and the text at HEAD,
/// compared as string VALUES (see `field_of`).
pub(crate) fn drifted_fields(accepted: &str, head: &str) -> Vec<&'static str> {
    ACCEPTED_FIELDS.iter().copied().filter(|k| field_of(accepted, k) != field_of(head, k)).collect()
}

/// Guard (hard, D0308 / issue341): the text a human signed is the text the tree carries.
///
/// An accepted Decision's decision, rationale and consequences at HEAD must equal its text at the
/// SHA its latest acceptance result binds to - the file at `judgedAgainst`, or,
/// when the acceptance was recorded in the same commit as the Decision (every standing-consent
/// auto-accept), the commit that introduced the file. A difference is a violation naming the Decision,
/// both SHAs and the fields; the remedy is `keel accept <d> --rebind` on a human judgment that the
/// words still hold, never an edit of history. Measured before the guard existed: 17 of 301 accepted
/// Decisions had drifted, all editorially (PROPOSED->ACCEPTED wording inside the text, rollout notes,
/// guard-count renumbering) - recorded as issue371, re-bound, not allowlisted.
#[must_use]
pub fn acceptance_binds_to_text(root: &Path) -> GuardReport {
    let dir = root.join(".engine").join("decisions");
    let files = keel_model::corpus::collect_sysml(&dir);
    if files.is_empty() {
        return GuardReport { name: "acceptance-binds-to-text", scanned: 0, warnings: Vec::new(), violations: Vec::new() };
    }
    // Every accepted Decision IN FORCE with a result: (rel path, decision, sha, head text). A retired
    // Decision - the target of a `#Supersede` edge (D0398) - keeps the standing it had when retired, but
    // its text is no longer what anyone is bound to; the superseder's is, and that one is checked.
    let retired = keel_model::corpus::supersede_targets(root);
    let mut accepted: Vec<(String, String, String, String)> = Vec::new();
    for f in &files {
        let Ok(text) = keel_model::corpus::read_to_string(f) else { continue };
        if !text.contains("DecisionStatus::accepted") {
            continue;
        }
        let Some(i) = text.find("part d0") else { continue };
        let dec: String = text[i + 5..].chars().take_while(|c| c.is_alphanumeric()).collect();
        if retired.contains(&dec) {
            continue;
        }
        let Some(sha) = latest_acceptance_sha(&text, &dec) else { continue };
        let rel = relpath(root, f);
        accepted.push((rel, dec, sha, text));
    }
    // One batch for the texts at the binding SHAs; one `git log` for every file's introducing commit.
    let keys: Vec<String> = accepted.iter().map(|(rel, _, sha, _)| format!("{sha}:{rel}")).collect();
    let blobs = keel_model::gitfacts::batch_cat_blobs(root, &keys);
    let intro_log = git_stdout(root, &["log", "--diff-filter=A", "--format=%h", "--name-only", "--", ".engine/decisions"]);
    let mut introducing: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut current = String::new();
    for line in intro_log.lines() {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }
        if std::path::Path::new(l).extension().is_some_and(|e| e.eq_ignore_ascii_case("sysml")) {
            introducing.entry(l.replace('\\', "/")).or_insert_with(|| current.clone());
        } else {
            current = l.to_string();
        }
    }
    let intro_keys: Vec<String> = accepted
        .iter()
        .filter(|(rel, _, sha, _)| blobs.get(&format!("{sha}:{rel}")).cloned().flatten().is_none())
        .filter_map(|(rel, _, _, _)| introducing.get(rel).map(|c| format!("{c}:{rel}")))
        .collect();
    let intro_blobs = keel_model::gitfacts::batch_cat_blobs(root, &intro_keys);
    let head = git_stdout(root, &["rev-parse", "--short", "HEAD"]).trim().to_string();
    let mut violations = Vec::new();
    let mut warnings = Vec::new();
    let mut scanned = 0usize;
    for (rel, dec, sha, text) in &accepted {
        let at_binding = blobs.get(&format!("{sha}:{rel}")).cloned().flatten();
        let (base_sha, base) = at_binding.map_or_else(
            || {
                introducing.get(rel).map_or_else(
                    || (sha.clone(), None),
                    |c| (format!("{c} (the commit that introduced the file; the acceptance at {sha} predates it)"), intro_blobs.get(&format!("{c}:{rel}")).cloned().flatten()),
                )
            },
            |b| (sha.clone(), Some(b)),
        );
        let Some(base) = base else {
            warnings.push(format!("{dec}: no text found at its acceptance SHA {sha} or at any introducing commit - the binding cannot be checked yet (not yet committed, a shallow clone, or a file renamed since); it will be, at the first commit that holds it"));
            continue;
        };
        scanned += 1;
        let mut drift = drifted_fields(&base, text);
        if !drift.is_empty() {
            // D0329: a re-binding that LANDED WITH the corrected text vouches for the tree that carried
            // it. Compare against the commit where the latest result first appeared; if that text is
            // HEAD's, the binding holds. An uncommitted re-binding is pending, not drift.
            if let Some((result, _)) = latest_acceptance(text, dec) {
                if let Some(landing) = result_landing_commit(root, rel, &result) {
                    let key = format!("{landing}:{rel}");
                    if let Some(Some(at_landing)) = keel_model::gitfacts::batch_cat_blobs(root, std::slice::from_ref(&key)).get(&key) {
                        if drifted_fields(at_landing, text).is_empty() {
                            drift.clear();
                        }
                    }
                } else {
                    warnings.push(format!("{dec}: its latest acceptance ({result}) is not committed yet - the binding is checked at the commit that lands it (D0329)"));
                    drift.clear();
                }
            }
        }
        if !drift.is_empty() {
            violations.push(format!(
                "{dec}: {} changed since the acceptance bound at {base_sha} - the text at {head} (HEAD) is not the text the human signed (issue341/D0308). If the words still hold, re-bind with `keel accept {dec} --rebind --note \"<what changed, why it still holds>\"`; never edit the acceptance.",
                drift.join(", ")
            ));
        }
    }
    violations.sort();
    GuardReport { name: "acceptance-binds-to-text", scanned, warnings, violations }
}

// ── attestation-substance guard (a confirmation that attests nothing) ─────────────────────────────

/// Attestations already thin when this guard landed (2026-08-13), grandfathered to WARNING.
///
/// FORWARD-ONLY per the issue068 lesson: a new guard must never retroactively fail items authored
/// under the process in force at the time. Nine of 234 `method=confirmation` verifications — seven bare
/// "accepted", one empty (`d0129Accept`), one bare actor name (`d0128Accept`). They warn
/// on every run so the debt stays visible; anything NEW is a hard violation. Do not extend this list:
/// a new contentless attestation is a defect to fix, not to grandfather.
pub(crate) const GRANDFATHERED_THIN_ATTESTATIONS: [&str; 9] = [
    "d0118Accept",
    "d0119Accept",
    "d0120Accept",
    "d0121Accept",
    "d0124Accept",
    "d0125Accept",
    "d0127Accept",
    "d0128Accept",
    "d0129Accept",
];

/// Guard: a passing `method=confirmation` must actually attest something (issue083 / D0130).
///
/// `d0129Accept` was authored with an EMPTY `procedureText` and passed every enforced guard — because
/// `acceptance-events` and `confirmation-authenticity` verify that an acceptance EXISTS and is
/// HUMAN-judged, never that it says anything. For a confirmation the attestation text IS the evidence
/// (D0016), so a contentless acceptance is an unsupported claim in the shape of a complete record, on
/// the record type that governs everything downstream.
///
/// HARD-blocking, matching `decision-rationale` (D0103), which applies the same substantive-field test
/// to a Decision's *why*: a contentless attestation is ill-formed STATE, not incomplete work, so it is
/// squarely inside the honest-state gate (D0098) rather than the burndown.
#[must_use]
pub fn attestation_substance(root: &Path) -> GuardReport {
    let grandfathered: HashSet<&str> = GRANDFATHERED_THIN_ATTESTATIONS.iter().copied().collect();
    // COUNTED, not enumerated (D0261). The allowlist is FIXED and anything outside it already
    // violates, so nothing can hide in this number - unlike the per-instance lines, which added no
    // information after the first run and crowded out findings a reader could act on.
    let mut grandfathered_thin = 0usize;
    match keel_view::view::thin_attestations(root) {
        Ok(found) => {
            let mut warnings = Vec::new();
            let mut violations = Vec::new();
            for (name, reason) in &found {
                if grandfathered.contains(name.as_str()) {
                    grandfathered_thin += 1;
                } else {
                    violations.push(format!("{name}: {reason} — a confirmation records a HUMAN's word, so it must say what was attested and to what (D0016/issue083)"));
                }
            }
            if grandfathered_thin > 0 {
                warnings.push(history_line(&format!(
                    "{grandfathered_thin} grandfathered thin attestation(s) (pre-issue083) — counted,                      not enumerated (D0261): the allowlist is fixed, so anything outside it is a                      violation above and nothing can hide in this number"
                )));
            }
            GuardReport { name: "attestation-substance", scanned: found.len(), warnings, violations }
        }
        Err(e) => GuardReport { name: "attestation-substance", scanned: 0, warnings: Vec::new(), violations: vec![format!("error reading attestations: {e}")] },
    }
}

/// Guard (D0092/D0106/D0129): a human-only attestation must be judged by a HUMAN.
///
/// `confirmation-authenticity` already enforces this for Decision ACCEPTANCE. This extends the same
/// rule to the other authority D0129 names: DISPOSITION of a finding at or above the threshold.
///
/// THE THRESHOLD IS LOAD-BEARING AND WAS ALMOST GOT WRONG. D0080 explicitly permits an AI to
/// disposition a LOW finding — this repo contains exactly such a case, `issue043Disp1R1` judged by
/// `claudeOpus`, whose own text says "Low doc-accuracy finding, AI-dispositioned (no human gate for
/// Low, D0080)". A guard requiring a human on every disposition would have failed a correct,
/// documented judgement and forced either a false attestation or a bypass. Medium and above only.
/// The threshold stays in the guard rather than in the policy file because it is a property of the
/// FINDING, not of the actor — the contract answers "who may attest", not "what needs attesting".
///
/// D0146: the check is now against the DECLARED policy (kind AND role) rather than a hardcoded
/// `is a Person`, which is what srDcAuthorityFromRegistry asks for. An absent contract falls back to
/// human-only, so deleting the file cannot disable the check.
#[must_use]
pub fn attestation_authority(root: &Path) -> GuardReport {
    match keel_view::view::ai_judged_high_dispositions(root) {
        Ok((scanned, bad)) => {
            let violations = bad
                .into_iter()
                .map(|(disp, issue, gap)| format!(
                    "{disp}: dispositions '{issue}' (>= Medium) but its judge does not satisfy the declared authority policy — {gap}. See .engine/contracts/attestation-policy.toml [findingDisposition] (D0092/D0146)."
                ))
                .collect();
            GuardReport { name: "attestation-authority", scanned, warnings: Vec::new(), violations }
        }
        Err(e) => GuardReport {
            name: "attestation-authority",
            scanned: 0,
            warnings: Vec::new(),
            violations: vec![format!("error reading dispositions: {e}")],
        },
    }
}

/// Guard (issue144): a judgment may not cite a commit that postdates it.
///
/// HARD, and it is squarely an honest-state gate (D0098): it does not ask whether work is finished, it
/// asks whether a recorded judgment is POSSIBLE. It exists because I stamped a human's
/// `method=confirmation` result — given the day before — against a commit created today, by running a
/// blanket replace of every `PENDING` SHA in a file. Every field remained well-formed, `attestation-*`
/// and `confirmation-authenticity` both passed, and the record silently claimed the human had attested
/// something at a commit they had never seen. §4 forbids fabricating an attestation; this is the control
/// for fabricating one MECHANICALLY, which no amount of care about the original recording prevents.
#[must_use]
pub fn impossible_evidence_dates(root: &Path) -> GuardReport {
    match keel_view::view::impossible_evidence_dates(root) {
        Ok((scanned, violations)) => {
            GuardReport { name: "impossible-evidence-date", scanned, warnings: Vec::new(), violations }
        }
        Err(e) => GuardReport {
            name: "impossible-evidence-date",
            scanned: 0,
            warnings: Vec::new(),
            violations: vec![format!("error reading results: {e}")],
        },
    }
}

/// Guard 52: an AI-judged `method=test` result records WHAT WAS RUN (D0232/issue266).
///
/// MEASURED before it was built: of 5,135 recorded `TestResult`s, exactly ONE recorded what produced
/// it. A result carries outcome, judgedBy, judgedAt and judgedAgainst — so every `pass` in this model
/// is a TESTIMONY, and nothing in the record lets a third party re-derive it. That is the mechanism
/// behind a 3.9% fail rate over 5,120 results: the actor doing the work also authors the verdict.
///
/// AI-JUDGED ONLY, and that is the design rather than an exemption. Governance binds the AI; a
/// HUMAN's word IS the evidence, and demanding a receipt from them would point the control at the
/// wrong party. `method=confirmation` is human by construction (D0106) and never in scope here.
///
/// FORWARD-ONLY from [`EVIDENCE_ENFORCED_FROM`], on the D0198 precedent: 983 `method=test` results
/// predate the convention, and retro-fitting evidence nobody captured would mean inventing it.
pub(crate) fn evidence_cited(root: &Path) -> GuardReport {
    let mut scanned = 0usize;
    let mut violations = Vec::new();
    // The Test declares the METHOD, the result declares the JUDGE — both are needed, so map first.
    let files = keel_model::corpus::collect_sysml(&root.join(".tracking"));
    let mut method_of: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for f in &files {
        let Ok(text) = keel_model::corpus::read_to_string(f) else { continue };
        for cap in text.split("verification ").skip(1) {
            let Some(name) = cap.split([' ', ':']).next() else { continue };
            if let Some(m) = cap.split(":>> method = VerificationMethod::").nth(1) {
                if let Some(kind) = m.split([';', ' ']).next() {
                    method_of.insert(name.to_string(), kind.to_string());
                }
            }
        }
    }
    for f in &files {
        let Ok(text) = keel_model::corpus::read_to_string(f) else { continue };
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if !line.contains(" : TestResult {") {
                continue;
            }
            let Some(judged_at) = field(line, "judgedAt") else { continue };
            if judged_at.as_str() < EVIDENCE_ENFORCED_FROM {
                continue;
            }
            let Some(judged_by) = field(line, "judgedBy") else { continue };
            // A human's attestation needs no receipt.
            if keel_actor::actor::kind_of(root, &judged_by).as_deref() == Some("human") {
                continue;
            }
            // A PROPOSAL (D0312 B) claims nothing yet - it is what an AI records when it has no
            // receipt - so it owes none; the claim is made when a human judges it.
            if line.contains("VerdictKind::proposed") {
                continue;
            }
            let Some(part) =
                line.split(" : TestResult").next().and_then(|s| s.split("part ").nth(1))
            else {
                continue;
            };
            let base = part.trim().rsplit_once('R').map_or_else(|| part.trim(), |(b, _)| b);
            if method_of.get(base).map(String::as_str) != Some("test") {
                continue; // only an EXERCISED claim owes a re-runnable receipt
            }
            scanned += 1;
            // The receipt is a `// RAN:` comment on the result line or immediately above it.
            let has = line.contains("// RAN:")
                || i.checked_sub(1)
                    .and_then(|j| lines.get(j))
                    .is_some_and(|p| p.trim_start().starts_with("// RAN:"));
            if !has {
                violations.push(format!(
                    "{}:{}: {} is an AI-judged method=test result with no `// RAN:` receipt - a pass nobody else can re-derive is a testimony, not a test. Pass --evidence to record result (D0232)",
                    relpath(root, f),
                    i + 1,
                    part.trim()
                ));
            }
        }
    }
    GuardReport { name: "evidence-cited", scanned, warnings: Vec::new(), violations }
}

/// Guard 68: a PLAN-COVERED acceptance still holds against its plan (D0396 / issue-dcSignedPlan).
///
/// A marker Decision accepted at record time under D0375 option C carries a `PLAN-COVERED under dNNNN
/// step '<name>'` acceptance note: the human signed the plan, not this Decision, so the cover is only
/// as good as the plan it cites. If the plan is later un-accepted, re-accepted under standing consent,
/// itself becomes plan-covered, loses its decider, or its text stops naming the step, the cover is a
/// signature that no longer exists. This guard re-runs the same three clauses `plan_cover::assess`
/// applied at record time and FAILS any covered Decision whose plan no longer holds them - so the
/// cover cannot outlive the plan.
pub(crate) fn plan_covers_step(root: &Path) -> GuardReport {
    let mut violations = Vec::new();
    let mut scanned = 0usize;
    for p in keel_model::corpus::collect_sysml(&root.join(".engine").join("decisions")) {
        let Ok(text) = keel_model::corpus::read_to_string(&p) else { continue };
        // the covered Decision's own name and its acceptance note
        let Some(dname) = text.split("part d").nth(1).and_then(|r| r.split(' ').next()).map(|s| format!("d{s}")) else {
            continue;
        };
        let accept_marker = format!("verification {dname}Accept ");
        let Some(after) = text.split(&accept_marker).nth(1) else { continue };
        let Some(note) = after.split(":>> procedureText = \"").nth(1).and_then(|r| r.split('"').next()) else {
            continue;
        };
        if !note.starts_with(crate::plan_cover::TOKEN) {
            continue;
        }
        scanned += 1;
        let Some((plan_id, step)) = crate::plan_cover::parse_note(note) else {
            violations.push(format!("{dname}: carries a {} note that cannot be parsed for its plan and step", crate::plan_cover::TOKEN));
            continue;
        };
        match crate::plan_cover::assess(root, &dname, &plan_id, &step) {
            crate::plan_cover::Cover::Covered { .. } => {}
            crate::plan_cover::Cover::Refused { clause, detail } => {
                violations.push(format!(
                    "{dname}: accepted as PLAN-COVERED by {plan_id} step '{step}', but the plan no longer holds clause ({clause}): {detail}. The cover is a signature that no longer exists - re-accept {dname} with a human's own word, or restore the plan."
                ));
            }
        }
    }
    GuardReport { name: "plan-covers-step", scanned, warnings: Vec::new(), violations }
}

// ── judgment-request-quality guard (a fork must earn the ask, D0207 clause 3) ────────────────────

/// Guard: a PROPOSED fork Decision carries everything a human needs to judge it.
///
/// Their words (D0207): "there's not a strong shortname, not good rationale, not good alternatives
/// or implications (i.e. the 'why'), there's no statement of research. all these should be provided
/// if you're reaching out for my judgment." A fork (a decision enumerating OPTIONs) is the one shape
/// that still reaches out — so before it may even be proposed it must have: a short name leading the
/// title (one word before the colon), a substantive rationale, a RESEARCH statement grounding the
/// choice, and per-option implications (a COST per OPTION). Non-fork decisions auto-accept under the
/// standing consent and are not scanned here. Accepted history is out of scope (status filter).
/// The `dNNNN` names declared in one decision file (`part dNNNN : Decision`), in order.
pub(crate) fn decision_names_in(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|l| l.trim_start().strip_prefix("part "))
        .filter_map(|r| r.split_once(" : Decision").map(|(n, _)| n.trim().to_string()))
        .filter(|n| n.starts_with('d') && n.len() == 5 && n[1..].chars().all(|c| c.is_ascii_digit()))
        .collect()
}

#[must_use]
pub fn judgment_request_quality(root: &Path) -> GuardReport {
    let mut scanned = 0usize;
    let mut violations = Vec::new();
    // D0398: a retired Decision (the target of a #Supersede edge) reaches out to nobody, whatever
    // its status field kept - its quality as a request is no longer anyone's concern.
    let retired = keel_model::corpus::supersede_targets(root);
    for path in keel_model::corpus::collect_sysml(&root.join(".engine").join("decisions")) {
        let Ok(text) = keel_model::corpus::read_to_string(&path) else { continue };
        if !text.contains("status = DecisionStatus::proposed") {
            continue;
        }
        if decision_names_in(&text).iter().all(|d| retired.contains(d.as_str())) {
            continue;
        }
        let rel = relpath(root, &path);
        let options = text.matches("OPTION ").count();
        let distinct_options = {
            let mut toks: Vec<char> = Vec::new();
            let mut rest = text.as_str();
            while let Some(i) = rest.find("OPTION ") {
                rest = &rest[i + 7..];
                if let Some(c) = rest.chars().next().filter(char::is_ascii_uppercase) {
                    if !toks.contains(&c) {
                        toks.push(c);
                    }
                }
            }
            toks.len()
        };
        if distinct_options < 2 {
            continue; // not a fork — auto-accepts under D0207, never reaches out
        }
        scanned += 1;
        let field = |name: &str| -> String {
            let key = format!("{name} = \"");
            text.find(&key).and_then(|i| {
                let s = i + key.len();
                text[s..].find('"').map(|j| text[s..s + j].to_string())
            }).unwrap_or_default()
        };
        let title = field("title");
        let short = title.split(':').next().unwrap_or("").trim();
        if short.is_empty() || short.contains(' ') || short.chars().count() > 28 {
            violations.push(format!(
                "{rel}: fork decision's title does not lead with a strong short name (one word before the colon, <= 28 chars) — got \"{short}\" (D0207: the ask must be recognizable at a glance)"
            ));
        }
        if field("rationale").chars().count() < 200 {
            violations.push(format!(
                "{rel}: fork decision's rationale is under 200 chars — a request for judgment must carry its why (D0207)"
            ));
        }
        let research = text
            .lines()
            .find_map(|l| l.trim_start().strip_prefix("// RESEARCH:"))
            .map_or("", str::trim);
        if research.chars().count() < 40 {
            violations.push(format!(
                "{rel}: fork decision carries no substantive RESEARCH statement (a `// RESEARCH:` line, >= 40 chars) — what was looked at before asking: panel precedent, literature, prior art, or 'none found, and here is where I looked' (D0207; `keel record decision --research \"...\"`)"
            ));
        }
        let costs = text.matches("COST").count();
        if costs < distinct_options {
            violations.push(format!(
                "{rel}: {distinct_options} option(s) but only {costs} COST statement(s) — every alternative carries its implications or the choice is not informed (D0207)"
            ));
        }
        let _ = options;
    }
    GuardReport { name: "judgment-request-quality", scanned, warnings: Vec::new(), violations }
}

// ── claim-ancestry guard (a claim's date is bounded by the commit that introduced it, issue229) ───

/// Guard: `claimedAt` cannot precede its own introducing commit by more than the expiry window.
///
/// issue229 (process-value panel, multi-agent lens): holdership = earliest un-expired `claimedAt`,
/// and `claimedAt` is authored by the claimer — a backdated claim steals holdership deterministically
/// on every clone. This applies the repo's own doctrine (D0013: git ancestry is the clock) to claims:
/// the introducing commit's author date bounds how early the claim may say it was made. The bound is
/// [`keel_write::claim::CLAIM_EXPIRY_DAYS`], because a claim older than the window is stale on arrival —
/// backdating WITHIN the window remains possible and is stated here rather than hidden: the guard
/// narrows the theft window from unbounded to the expiry span. An uncommitted claim has no
/// introducing commit yet and is skipped — it cannot influence another clone until it lands.
#[must_use]
pub fn claim_ancestry(root: &Path) -> GuardReport {
    let claims = match keel_write::claim::claims(root) {
        Ok(c) => c,
        Err(e) => {
            return GuardReport {
                name: "claim-ancestry",
                scanned: 0,
                warnings: Vec::new(),
                violations: vec![format!("error reading claims: {e}")],
            }
        }
    };
    // A SHALLOW clone cannot resolve introduction commits — the oldest visible commit is the clone
    // boundary, not the claim's birth, and judging against it flags honest claims (this exact guard
    // went CI-red on its first push because checkout@v4 defaults to depth 1). Depth-dependent
    // verdicts are the machine-dependence K15 forbids: on a shallow repo this guard SKIPS LOUDLY.
    let shallow = keel_git::gitx::git()
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--is-shallow-repository"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .is_some_and(|o| String::from_utf8_lossy(&o.stdout).trim() == "true");
    if shallow {
        return GuardReport {
            name: "claim-ancestry",
            scanned: 0,
            warnings: vec![
                "SHALLOW clone: introduction commits cannot be resolved, so claim dates were NOT checked here (a depth-dependent verdict would be the K15 machine-dependence this guard exists to prevent). CI checks out full history for this reason.".to_string(),
            ],
            violations: Vec::new(),
        };
    }
    let mut scanned = 0usize;
    let mut violations = Vec::new();
    for c in &claims {
        if c.at.is_empty() {
            continue;
        }
        let intro = keel_git::gitx::git()
            .arg("-C")
            .arg(root)
            .args(["log", "--reverse", "--format=%ad", "--date=short", "-S", &c.name, "--", ".tracking/claims"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| String::from_utf8_lossy(&o.stdout).lines().next().map(str::to_owned));
        let Some(intro_date) = intro.filter(|d| !d.is_empty()) else { continue }; // uncommitted claim
        scanned += 1;
        let lead = keel_view::view::days_between_pub(&c.at, &intro_date);
        if lead > keel_write::claim::CLAIM_EXPIRY_DAYS {
            violations.push(format!(
                "{}: claimedAt {} predates its introducing commit ({intro_date}) by {lead} day(s) — more than the {}-day expiry window. Git ancestry is the clock (D0013); a claim cannot say it was made before it could have influenced any clone (issue229 backdating).",
                c.name, c.at, keel_write::claim::CLAIM_EXPIRY_DAYS
            ));
        }
    }
    GuardReport { name: "claim-ancestry", scanned, warnings: Vec::new(), violations }
}

// ── consent-scope guard (D0439 / issue460: consent accepted nothing its own text put outside it) ──

/// What `consent_scope` found in ONE auto-accepted Decision's text.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ConsentScope {
    /// Inside the consent: no marker part, and the prose names none of the marker vocabulary (or
    /// declares `NOT A PROCESS CHANGE`).
    Inside,
    /// A `#ProspectiveChange` / `#SafetyChange` part auto-accepted - D0337's own rule, defeated.
    MarkerPart(&'static str),
    /// No marker, but the prose names the vocabulary - the issue460 shape.
    MarkerWords(Vec<&'static str>),
}

/// Read a one-line `:>> <name> = "..."` Decision field out of a whole file. The write path emits every
/// field on one line (`sanitize_field` collapses whitespace), so the first `"` after the needle ends it.
pub(crate) fn decision_field(text: &str, name: &str) -> Option<String> {
    let needle = format!(":>> {name} = \"");
    let rest = text.split(&needle).nth(1)?;
    Some(rest.split('"').next()?.to_string())
}

/// Classify one Decision file against the consent's scope, with the write path's own vocabulary
/// (`deck::marker_words`) - the hold in `record decision` and this guard cannot disagree on a word.
pub(crate) fn consent_scope_of(text: &str, dname: &str) -> ConsentScope {
    for m in ["ProspectiveChange", "SafetyChange"] {
        if text.contains(&format!("#{m} part {dname} : Decision")) {
            return ConsentScope::MarkerPart(m);
        }
    }
    let fields: Vec<String> = ["context", "decision", "rationale", "consequences"].iter().filter_map(|f| decision_field(text, f)).collect();
    let refs: Vec<&str> = fields.iter().map(String::as_str).collect();
    keel_model::textscan::marker_text_without_marker(&refs, false).map_or(ConsentScope::Inside, ConsentScope::MarkerWords)
}

/// Guard: standing consent accepted nothing outside its scope (D0337) - read from the RECORDED files,
/// not the write path's intent.
///
/// D0337 scoped the consent to the existing processes: a marker Decision stays proposed. The write path
/// applies that rule, and on 2026-09-10 a Decision whose consequences said `Process-change (D0337)` in
/// so many words auto-accepted because its draft had no `marker:` line and only the marker was read
/// (issue460). This guard reads every AUTO-ACCEPTED Decision (`acceptance_kind` == Auto - the human's
/// own acceptances are theirs to give) and fails one that (a) carries a marker part, or (b) names the
/// marker vocabulary in its prose without `NOT A PROCESS CHANGE`; both dated forward of their cutoff.
/// Same classifier as the `record decision` hold, so a hand-edited or hand-accepted file cannot pass a
/// test the write path would have failed.
///
/// HARD, forward-only. 104 marker Decisions auto-accepted before D0337 existed (d0206..d0336, all dated
/// 2026-09-05 or earlier) and one earlier unmarked prose mention (d0367) are immutable history, counted
/// (D0261).
#[must_use]
pub fn consent_scope(root: &Path) -> GuardReport {
    /// D0337's date: from the day after, a marker Decision reached acceptance only by a human's word.
    const MARKER_CUTOFF: &str = "2026-09-06";
    /// This control's date (D0439): from here on the text is read too.
    const WORDS_CUTOFF: &str = "2026-09-10";
    let mut scanned = 0usize;
    let mut violations = Vec::new();
    let (mut history_parts, mut history_words) = (0usize, 0usize);
    for path in keel_model::corpus::collect_sysml(&root.join(".engine").join("decisions")) {
        let Ok(text) = keel_model::corpus::read_to_string(&path) else { continue };
        let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let Some(nnnn) = stem.get(..4).filter(|s| s.chars().all(|c| c.is_ascii_digit())) else { continue };
        let dname = format!("d{nnnn}");
        if acceptance_kind(&text, &dname) != Some(Acceptance::Auto) {
            continue;
        }
        scanned += 1;
        let created = decision_field(&text, "createdAt").unwrap_or_default();
        let rel = relpath(root, &path);
        match consent_scope_of(&text, &dname) {
            ConsentScope::Inside => {}
            ConsentScope::MarkerPart(m) => {
                if created.as_str() >= MARKER_CUTOFF {
                    violations.push(format!(
                        "{rel}: {dname} carries #{m} and its acceptance is AUTO-ACCEPTED under standing consent - a process or enforcement change the consent does not reach (D0337); it is the human's to accept in their own words (D0289), or stays proposed"
                    ));
                } else {
                    history_parts += 1;
                }
            }
            ConsentScope::MarkerWords(words) => {
                if created.as_str() >= WORDS_CUTOFF {
                    violations.push(format!(
                        "{rel}: {dname} names {} in its own text, carries no marker and is AUTO-ACCEPTED under standing consent - the text put it outside the consent and nothing read the text (issue460/D0439); re-record it with `marker: process-change`, state `{}: <why>` in the text, or a human accepts it in their own words (D0289)",
                        words.join(", "),
                        keel_model::textscan::NOT_A_PROCESS_CHANGE
                    ));
                } else {
                    history_words += 1;
                }
            }
        }
    }
    let mut warnings = Vec::new();
    if history_parts + history_words > 0 {
        warnings.push(history_line(&format!(
            "{history_parts} marker Decisions auto-accepted before {MARKER_CUTOFF} (D0337 did not exist) and {history_words} texts naming the marker vocabulary auto-accepted before {WORDS_CUTOFF} (nothing read the text) - immutable history, counted not enumerated (D0261)"
        )));
    }
    GuardReport { name: "consent-scope", scanned, warnings, violations }
}

// ── direction-cited guard (D0463 / issue428: the human's direction is a Statement, not the agent's rendering) ──

/// D0463's date: a Decision or dated `DoD` created on/after this day that quotes the human links the Statement.
pub(crate) const DIRECTION_CUTOFF: &str = "2026-09-12";

/// The phrases that attribute a quoted span to the human, matched without case. `the human, YYYY-MM-DD:`
/// is read as a sixth form by [`human_spans`].
pub(crate) const DIRECTION_ANCHORS: [&str; 5] = ["their words", "the human's words", "the human said", "human's own words", "the human's own words"];

/// Fold the two escapes a `SysML` string field carries so a span compares as the human typed it: `''`
/// (a quote inside a quoted field, the write path's own escape) and `\"`; whitespace runs become one space.
pub(crate) fn fold_quoted(s: &str) -> String {
    s.replace("''", "'").replace("\\\"", "\"").split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Every span `text` attributes to the human: after an anchor phrase (or `the human, YYYY-MM-DD`), an
/// optional `verbatim`, at most a few of `:` `,` `-` and spaces, then a quoted span - `“...”` closed
/// by the first `”`, or `'...'` closed by the first `'` NOT followed by a letter or digit, so `don't`
/// and `it's` inside the quote do not close it (a possessive `s' ` does, and the shorter span is
/// still a verbatim prefix). Under eight characters is not read as a quote; `their words>` or `their
/// words' shape` (no quote follows) is a mention, not a citation. Returned folded ([`fold_quoted`]),
/// which is also how a Statement's text is compared.
pub(crate) fn human_spans(text: &str) -> Vec<String> {
    let t = fold_quoted(text);
    let lower = t.to_ascii_lowercase();
    let mut starts: Vec<usize> = Vec::new();
    for a in DIRECTION_ANCHORS {
        let mut from = 0;
        while let Some(i) = lower[from..].find(a) {
            starts.push(from + i + a.len());
            from += i + a.len();
        }
    }
    let dated = "the human, ";
    let mut from = 0;
    while let Some(i) = lower[from..].find(dated) {
        let p = from + i + dated.len();
        let is_date = t.get(p..p + 10).is_some_and(|d| d.char_indices().all(|(k, c)| if k == 4 || k == 7 { c == '-' } else { c.is_ascii_digit() }));
        if is_date {
            starts.push(p + 10);
        }
        from = p;
    }
    starts.sort_unstable();
    starts.dedup();
    let mut spans = Vec::new();
    for s in starts {
        let rest = &t[s..];
        let sep = [' ', ':', ',', '-'];
        let skipped = rest.trim_start_matches(sep);
        let skipped = skipped.strip_prefix("verbatim").map_or(skipped, |r| r.trim_start_matches(sep));
        if rest.len() - skipped.len() > 16 {
            continue;
        }
        let close = match (skipped.strip_prefix('“'), skipped.strip_prefix('\'')) {
            (Some(body), _) => body.find('”').map(|i| &body[..i]),
            (None, Some(body)) => body.char_indices().find(|&(i, c)| c == '\'' && !body[i + 1..].chars().next().is_some_and(char::is_alphanumeric)).map(|(i, _)| &body[..i]),
            (None, None) => None,
        };
        let Some(span) = close else { continue };
        if span.chars().count() >= 8 {
            spans.push(span.to_string());
        }
    }
    spans
}

/// A record that may cite the human: `(name, createdAt if the record carries one, the spans it quotes)`.
pub(crate) type DirectionRecord = (String, Option<String>, Vec<String>);

/// Pure core (issue428 / D0463): every record that quotes the human, split by its date against the
/// cutoff. On/after: each span must appear (folded) in the `text` of a Statement the record reaches by
/// a `#DerivedFrom` edge, else `(record, what is missing)` is a violation. Before: counted as
/// grandfathered. Undated (a `DoD` the write path stamped no `createdAt` on): counted apart - the
/// boundary cannot be read, so the clause cannot be applied forward to it.
pub(crate) fn direction_violations<S: std::hash::BuildHasher>(
    records: &[DirectionRecord],
    edges: &[(String, String)],
    statements: &std::collections::HashMap<String, String, S>,
    cutoff: &str,
) -> (Vec<(String, String)>, usize, usize) {
    let mut forward = Vec::new();
    let (mut grandfathered, mut undated) = (0usize, 0usize);
    for (name, created, spans) in records {
        if spans.is_empty() {
            continue;
        }
        match created.as_deref() {
            None => {
                undated += 1;
                continue;
            }
            Some(c) if c < cutoff => {
                grandfathered += 1;
                continue;
            }
            Some(_) => {}
        }
        let linked: Vec<&String> = edges.iter().filter(|(from, _)| from == name).filter_map(|(_, to)| statements.get(to)).collect();
        if linked.is_empty() {
            let first = spans.first().map_or_else(String::new, |s| excerpt(s));
            forward.push((name.clone(), format!("no #DerivedFrom edge to a Statement, and it quotes the human: '{first}'")));
            continue;
        }
        for span in spans {
            if !linked.iter().any(|st| st.contains(span.as_str())) {
                forward.push((name.clone(), format!("no linked Statement's text holds this span verbatim: '{}'", excerpt(span))));
            }
        }
    }
    (forward, grandfathered, undated)
}

/// The first sixty characters of a span, for a message.
pub(crate) fn excerpt(span: &str) -> String {
    let mut s: String = span.chars().take(60).collect();
    if s.len() < span.len() {
        s.push_str("...");
    }
    s
}

/// A `:>> key = "..."` field read from `block`, honouring `\"` inside the value (a Statement's text may
/// carry one; `decision_field` stops at the first `"`, which the write path's one-line Decision fields allow).
pub(crate) fn quoted_field(block: &str, key: &str) -> Option<String> {
    let needle = format!(":>> {key} = \"");
    let rest = &block[block.find(&needle)? + needle.len()..];
    let mut out = String::new();
    let mut escaped = false;
    for c in rest.chars() {
        match (escaped, c) {
            (false, '\\') => {
                escaped = true;
                out.push(c);
            }
            (false, '"') => return Some(out),
            _ => {
                escaped = false;
                out.push(c);
            }
        }
    }
    None
}

/// Guard: a Decision or task `DoD` that quotes the human links a Statement holding their words (issue428).
///
/// stpa-self run 2 (UCA-H1) found `humanDirects` to be a prose channel nothing parses: a Decision may say
/// `the human's words: '...'` in its rationale with no Statement recorded and no `#DerivedFrom` edge, so
/// the tree holds the AGENT's rendering of the direction and nothing to check it against (hazard EHZ3 -
/// words attributed to the human without their record). `keel record statement` (D0216/D0236) is the
/// channel; this guard binds the citing records to it. A record is READ AS CITING when a field carries
/// one of the anchor phrases followed by a quoted span ([`human_spans`]); it HOLDS when every span is
/// found, escapes folded, in the `text` of a Statement it reaches by `#DerivedFrom dependency from
/// <record> to <stNNN>;`. Decisions are read from `.engine/decisions/` (context/decision/rationale/
/// consequences); `DoDs` from every `verification <x>DoD : Test` under `.tracking/` (procedureText).
///
/// HARD, forward-only from D0463's date by `createdAt`. The citing Decisions before it are the agent's
/// renderings already accepted on; they are counted once, not re-litigated (D0261). A `DoD` the write path
/// stamped no date on is counted apart: its boundary cannot be read (issue513 names the missing stamp).
#[must_use]
pub fn direction_cited(root: &Path) -> GuardReport {
    let mut records: Vec<DirectionRecord> = Vec::new();
    let mut edges: Vec<(String, String)> = Vec::new();
    let mut statements: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut scanned = 0usize;
    let mut read_edges = |text: &str| {
        for line in text.lines() {
            if let Some(rest) = line.trim_start().strip_prefix("#DerivedFrom dependency from ") {
                if let Some((from, to)) = rest.trim_end().trim_end_matches(';').split_once(" to ") {
                    edges.push((from.trim().to_string(), to.trim().to_string()));
                }
            }
        }
    };
    for path in keel_model::corpus::collect_sysml(&root.join(".engine").join("decisions")) {
        let Ok(text) = keel_model::corpus::read_to_string(&path) else { continue };
        read_edges(&text);
        let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let Some(nnnn) = stem.get(..4).filter(|s| s.chars().all(|c| c.is_ascii_digit())) else { continue };
        scanned += 1;
        let mut spans = Vec::new();
        for f in ["context", "decision", "rationale", "consequences"] {
            if let Some(v) = quoted_field(&text, f) {
                spans.extend(human_spans(&v));
            }
        }
        records.push((format!("d{nnnn}"), decision_field(&text, "createdAt"), spans));
    }
    for path in keel_model::corpus::collect_sysml(&root.join(".tracking")) {
        let Ok(text) = keel_model::corpus::read_to_string(&path) else { continue };
        read_edges(&text);
        let mut rest = text.as_str();
        while let Some(i) = rest.find("part st") {
            let block = &rest[i..];
            let name: String = block[5..].chars().take_while(|c| c.is_alphanumeric()).collect();
            let end = block.find("\n    }").map_or(block.len(), |e| e + 6);
            if block[..end].contains(": Statement") {
                if let Some(v) = quoted_field(&block[..end], "text") {
                    statements.insert(name.clone(), fold_quoted(&v));
                }
            }
            rest = &rest[i + 5 + name.len()..];
        }
        let mut rest = text.as_str();
        while let Some(i) = rest.find("verification ") {
            let block = &rest[i..];
            let head: &str = block.lines().next().unwrap_or("");
            rest = &rest[i + "verification ".len()..];
            let Some((name, tail)) = head["verification ".len()..].split_once(':') else { continue };
            let name = name.trim();
            if !name.ends_with("DoD") || !tail.trim_start().starts_with("Test") {
                continue;
            }
            // The block: from the head to the end of the line that closes procedureText (the write path
            // emits a DoD on one line; a hand-authored multi-line DoD is read to that line too).
            let Some(procedure) = quoted_field(block, "procedureText") else { continue };
            let close = block.find(":>> procedureText = \"").map_or(0, |s| s + ":>> procedureText = \"".len() + procedure.len());
            let block_end = block[close..].find('\n').map_or(block.len(), |e| close + e);
            let block = &block[..block_end];
            scanned += 1;
            let spans = human_spans(&procedure);
            if !spans.is_empty() {
                records.push((name.to_string(), decision_field(block, "createdAt"), spans));
            }
        }
    }
    let (forward, grandfathered, undated) = direction_violations(&records, &edges, &statements, DIRECTION_CUTOFF);
    let violations: Vec<String> = forward
        .into_iter()
        .map(|(name, what)| format!("{name}: quotes the human and {what} - the tree holds the agent's rendering of their direction with nothing to check it against (issue428/D0463, hazard EHZ3). Record their words with `keel record statement` and author `#DerivedFrom dependency from {name} to <stNNN>;`, or quote nobody."))
        .collect();
    let mut warnings = Vec::new();
    if grandfathered + undated > 0 {
        warnings.push(history_line(&format!(
            "{grandfathered} citing Decision(s) recorded before {DIRECTION_CUTOFF} quote the human with no Statement to check against, and {undated} citing DoD(s) carry no createdAt so the boundary cannot be read for them - immutable history and an unstamped field (issue513), counted not enumerated (D0261)"
        )));
    }
    GuardReport { name: "direction-cited", scanned, warnings, violations }
}

#[cfg(test)]
mod binds_to_text_tests {
    use super::{drifted_fields, latest_acceptance_sha};

    /// D0308: the LATEST acceptance result is the binding; the three accepted fields are compared;
    /// a re-binding to the current text clears the drift while the first acceptance stays in place.
    #[test]
    fn the_latest_acceptance_binds_and_drift_is_per_field() {
        let at_accept = r#"part d1 : Decision { :>> decision = "do X"; :>> rationale = "because"; :>> consequences = "Y"; }
    part d1AcceptR1 : TestResult { :>> judgedAgainst = "aaa1111"; }"#;
        let head = r#"part d1 : Decision { :>> decision = "do X"; :>> rationale = "because"; :>> consequences = "Y. ROLLOUT: landed."; }
    part d1AcceptR1 : TestResult { :>> judgedAgainst = "aaa1111"; }"#;
        assert_eq!(drifted_fields(at_accept, head), vec!["consequences"]);
        assert_eq!(latest_acceptance_sha(head, "d1"), Some("aaa1111".to_string()));
        let rebound = format!("{head}
    part d1AcceptR2 : TestResult {{ :>> judgedAgainst = \"bbb2222\"; :>> notes = \"REBOUND\"; }}");
        assert_eq!(latest_acceptance_sha(&rebound, "d1"), Some("bbb2222".to_string()), "the re-binding is the new baseline");
        assert!(drifted_fields(head, head).is_empty());
        // formatting between `+`-joined segments is not signed text
        let joined_a = ":>> consequences = \"Pros: a, \"\n        + \"b.\";";
        let joined_b = ":>> consequences = \"Pros: a, \"\r\n            + \"b.\";";
        assert!(drifted_fields(joined_a, joined_b).is_empty(), "indentation and line endings between segments are formatting");
        assert_eq!(super::field_of(joined_a, "consequences").as_deref(), Some("Pros: a, b."));
    }
}

#[cfg(test)]
mod claim_ancestry_tests {
    /// issue229 pinned: a claim whose `claimedAt` predates its own introducing commit by more than
    /// the expiry window turns the guard RED; a same-day claim stays green. Scratch git repo so the
    /// intro-commit lookup is real, not mocked.
    #[test]
    fn backdated_claim_is_refused_and_honest_claim_passes() {
        let dir = keel_fs::scratch("keel-claim-ancestry-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".tracking").join("claims")).expect("mkdir");
        let run = |args: &[&str]| {
            let out = keel_git::gitx::git().arg("-C").arg(&dir).args(args).output().expect("git runs");
            assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
        };
        run(&["init", "-q"]);
        run(&["config", "user.email", "t@t"]);
        run(&["config", "user.name", "t"]);
        let claim_file = |claimed_at: &str| {
            format!(
                "package ProjectClaimsT {{\n    part claimT001 : Claim {{\n        :>> id = \"aaaaaaaa-6666-4666-9666-aaaaaaaaaaaa\";\n        :>> claimedItem = \"someItem\";\n        :>> claimedBy = \"tester\";\n        :>> claimedAt = \"{claimed_at}\";\n    }}\n}}\n"
            )
        };
        // Backdated far beyond the window relative to the commit date (today).
        std::fs::write(dir.join(".tracking/claims/tester.sysml"), claim_file("2020-01-01")).expect("write");
        run(&["add", "-A"]);
        run(&["commit", "-q", "-m", "claim"]);
        keel_model::fingerprint::new_epoch();
        let red = super::claim_ancestry(&dir);
        assert_eq!(red.violations.len(), 1, "backdated claim must violate: {:?}", red.warnings);
        assert!(red.violations[0].contains("predates its introducing commit"), "{}", red.violations[0]);
        // An honest claim dated the day it was committed passes.
        let today = {
            let out = keel_git::gitx::git().arg("-C").arg(&dir).args(["log", "-1", "--format=%ad", "--date=short"]).output().expect("git");
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        };
        std::fs::write(dir.join(".tracking/claims/tester.sysml"), claim_file(&today)).expect("write");
        run(&["add", "-A"]);
        run(&["commit", "-q", "-m", "honest claim"]);
        keel_model::fingerprint::new_epoch();
        let green = super::claim_ancestry(&dir);
        assert!(green.violations.is_empty(), "{:?}", green.violations);
    }
}

#[cfg(test)]
mod consent_scope_tests {
    use super::{consent_scope_of, ConsentScope};

    fn decision(dname: &str, marker: &str, consequences: &str) -> String {
        format!(
            "package X {{\n    {marker}part {dname} : Decision {{\n        :>> createdAt = \"2026-09-10\";\n        :>> context = \"ctx\";\n        :>> decision = \"dec\";\n        :>> rationale = \"why\";\n        :>> consequences = \"{consequences}\";\n    }}\n    verification {dname}Accept : Test {{ :>> procedureText = \"AUTO-ACCEPTED under standing consent\"; }}\n    part {dname}AcceptR1 : TestResult {{ :>> verdict = VerdictKind::pass; }}\n}}\n"
        )
    }

    /// D0388 known-positive: the issue460 record - `Process-change (D0337)` in the consequences, no marker.
    #[test]
    fn a_text_naming_the_vocabulary_with_no_marker_is_outside_the_consent() {
        let t = decision("d0432", "", "Process-change (D0337): it waits for the human's word.");
        assert_eq!(consent_scope_of(&t, "d0432"), ConsentScope::MarkerWords(vec!["process-change"]));
        let t = decision("d0500", "#SafetyChange ", "changes the hook set");
        assert_eq!(consent_scope_of(&t, "d0500"), ConsentScope::MarkerPart("SafetyChange"));
    }

    /// D0388 known-negative: a plain Decision, and one that names the vocabulary in passing and says so.
    #[test]
    fn a_plain_text_and_a_declared_mention_are_inside_the_consent() {
        let t = decision("d0501", "", "Adopt the merge; the processes changed nothing.");
        assert_eq!(consent_scope_of(&t, "d0501"), ConsentScope::Inside);
        let t = decision("d0502", "", "Rank 1 lands with its own #ProspectiveChange Decision. NOT A PROCESS CHANGE: this ranks, it changes no process.");
        assert_eq!(consent_scope_of(&t, "d0502"), ConsentScope::Inside);
    }

    /// The real tree: nothing forward of the cutoffs, and the pre-cutoff history is counted, not enumerated.
    #[test]
    fn the_self_build_holds_and_counts_its_history() {
        let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
        let r = super::consent_scope(root);
        assert!(r.violations.is_empty(), "{:?}", r.violations);
        assert!(r.scanned >= 100, "scanned {} auto-accepted Decisions", r.scanned);
        assert_eq!(r.warnings.len(), 1, "one counted-history line: {:?}", r.warnings);
        assert!(r.warnings[0].contains("marker Decisions auto-accepted before"));
    }
}

#[cfg(test)]
mod direction_cited_tests {
    use super::{direction_violations, human_spans, quoted_field};
    use std::collections::HashMap;

    fn statements(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(n, t)| ((*n).to_string(), super::fold_quoted(t))).collect()
    }

    /// The reader: each accepted form yields the span; a mention with no quote, a short span and an
    /// apostrophe inside the quote do what the doc says.
    #[test]
    fn the_anchor_forms_yield_the_quoted_span_and_mentions_do_not() {
        assert_eq!(human_spans("their words: 'why are we losing rigor? it feels wrong'"), vec!["why are we losing rigor? it feels wrong"]);
        assert_eq!(human_spans("The human, 2026-09-09: 'don't stop for that' and then"), vec!["don't stop for that"]);
        assert_eq!(human_spans("the human's words 'kind of seems like keel is still crazy slow'."), vec!["kind of seems like keel is still crazy slow"]);
        assert_eq!(human_spans("Their words: ''I do like C the best'' (nested escape)"), vec!["I do like C the best"]);
        assert_eq!(human_spans("their words, verbatim: “repeated sampling should be an option though” - so it stays"), vec!["repeated sampling should be an option though"]);
        assert!(human_spans("their words, 2026-09-08, the copy-for-Claude text of the published brief").is_empty(), "a dated attribution with no quoted span is a mention");
        assert!(human_spans("keel accept <d> --words '<their words>' --by <human>").is_empty(), "a usage line is not a citation");
        assert!(human_spans("the sprint kept the 'their words' shape.").is_empty(), "a mention with no quote after it");
        assert!(human_spans("their words: 'ok'").is_empty(), "under eight characters is not read as a quote");
        assert!(human_spans("no citation here at all").is_empty());
        assert_eq!(quoted_field(":>> text = \"say \\\"hi\\\" now\"; :>> x = \"y\";", "text").as_deref(), Some("say \\\"hi\\\" now"));
    }

    /// D0388 known-positive: a Decision after the cutoff quoting the human with no Statement is a
    /// violation naming the span; one whose linked Statement holds different words is one too.
    #[test]
    fn a_citing_decision_with_no_statement_is_refused_forward_only() {
        let recs = vec![
            ("d0901".to_string(), Some("2026-09-12".to_string()), human_spans("their words: 'make the brief use caveman please'")),
            ("d0902".to_string(), Some("2026-09-12".to_string()), human_spans("their words: 'something the statement never said'")),
            ("d0903".to_string(), Some("2026-01-01".to_string()), human_spans("their words: 'an old rendering, before the rule'")),
            ("dcOldDoD".to_string(), None, human_spans("The human, 2026-09-01: 'an undated DoD quoting them'")),
        ];
        let edges = vec![("d0902".to_string(), "st900".to_string())];
        let sts = statements(&[("st900", "make it use caveman?")]);
        let (forward, grandfathered, undated) = direction_violations(&recs, &edges, &sts, "2026-09-12");
        assert_eq!(forward.len(), 2, "{forward:?}");
        assert!(forward[0].0 == "d0901" && forward[0].1.contains("no #DerivedFrom edge") && forward[0].1.contains("make the brief use caveman"), "{forward:?}");
        assert!(forward[1].0 == "d0902" && forward[1].1.contains("holds this span verbatim"), "{forward:?}");
        assert_eq!((grandfathered, undated), (1, 1));
    }

    /// D0388 known-negative: the Statement holds the span (escapes folded) and the edge exists - it
    /// passes; a Decision quoting nobody is untouched.
    #[test]
    fn a_citing_decision_linked_to_its_statement_passes_and_a_silent_one_is_untouched() {
        let recs = vec![
            ("d0904".to_string(), Some("2026-09-12".to_string()), human_spans("The human, 2026-09-08: 'make it use caveman?' - so the brief does.")),
            ("d0905".to_string(), Some("2026-09-12".to_string()), human_spans("their words: 'user said ''X'' - is the user asking for a view?'")),
            ("d0906".to_string(), Some("2026-09-12".to_string()), human_spans("no quotation of anyone; a plain rationale")),
        ];
        let edges = vec![("d0904".to_string(), "st110".to_string()), ("d0905".to_string(), "st111".to_string())];
        let sts = statements(&[("st110", "make it use caveman?"), ("st111", "rubrics so lower-model agents can run them as necessary (e.g. user said ''X'' - is the user asking for a view?  if so, quote")]);
        let (forward, grandfathered, undated) = direction_violations(&recs, &edges, &sts, "2026-09-12");
        assert!(forward.is_empty(), "{forward:?}");
        assert_eq!((grandfathered, undated), (0, 0));
    }

    /// The real tree: nothing forward of the cutoff, and the pre-cutoff citing Decisions are counted once.
    #[test]
    fn the_self_build_holds_and_counts_its_history() {
        let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
        let r = super::direction_cited(root);
        assert!(r.violations.is_empty(), "{:?}", r.violations);
        assert!(r.scanned >= 400, "scanned {} Decisions and DoDs", r.scanned);
        assert_eq!(r.warnings.len(), 1, "one counted-history line: {:?}", r.warnings);
        assert!(r.warnings[0].contains("citing Decision(s) recorded before"), "{}", r.warnings[0]);
    }
}
