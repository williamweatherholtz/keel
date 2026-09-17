//! Guard family `identity` - split from `keel-cli/src/guards.rs` by `scripts/split_guards.py` (sprint 733).
//!
//! Each guard's dispatch arm, code and tests sit together; the shared scanners, the runner and the
//! lock predicates are the crate root's (`super`). Nothing here was retyped: the text is guards.rs's,
//! with `crate::` paths pointing at the members and private items opened to the crate.

use super::*;

/// The `identity` family: every guard it dispatches, in `GUARD_NAMES` order, with the tier note each
/// arm carried in `run_one` (sprint 733). The root's union test holds these tables equal to `GUARD_NAMES`.
pub(crate) const FAMILY: Family = Family {
    name: "identity",
    arms: &[
        ("actors", actors),
        ("marker-vocabulary", marker_vocabulary), // hard (D0133/issue077) — an undeclared marker silently blinds a control
        ("duplicate-identity", duplicate_identity), // hard (D0129/issue074) — concurrent allocation lands green without it
        ("engine-lint", engine_lint), // hard import-check + warn missing-id (D0112 phase 1, kernel-free)
        ("sequence-multiplicity", sequence_multiplicity), // warning-only (issue101) — sequences are newly enabled
        ("parser-coverage", parser_coverage), // warning-only (issue102) — what the engine cannot read
        ("base-first-justification", base_first_justification), // warning-only (D0139(B))
        ("edge-endpoints", edge_endpoints), // hard (issue109) — an edge asserting a relationship to nothing
        ("ownership", ownership), // hard (D0108/D0129) — a non-owner overwriting another actor's fields
        ("type-collision", type_collision), // hard (D0128) — a project type shadowing an engine type, silently
        ("attribute-vocabulary", attribute_vocabulary), // hard (issue118) — an undeclared attribute is silently LOST
        ("identity-present", identity_present),
        ("identity-well-formed", identity_well_formed),
        ("enrollment-binding", enrollment_binding), // WARNING-tier (D0191, actor-enrollment unit) — a machine binding naming an unregistered or kindless actor
        ("id-is-a-uuid", id_is_a_uuid), // hard (D0430/issue454) - an id from the cutoff on, or added in the tree, is v4
    ],
};

// ── actors guard (authoredBy/createdBy/judgedBy reference a known ProjectActor) ────────────────

/// Pre-convention actor values (2026-06-10/11) + tool names used as judgedBy before the actor
/// convention; reported as WARN, not a violation. Mirrors `validate_actors.LEGACY_ACTORS`.
pub(crate) const LEGACY_ACTORS: &[&str] = &[
    "user", "demo", "inspect", "claudeOpus", "_test_suspect",
    "validate_schema", "validate_workflows", "validate_instances",
    "validate_tracking", "validate_all", "whats_next",
];

pub(crate) const ACTOR_ATTRS: &[&str] = &["authoredBy", "createdBy", "judgedBy"];

/// The day the actor convention became binding. Every legacy-actor reference in the corpus is dated
/// before it (newest observed: 2026-06-11), so this starts at zero violations; a legacy name on a
/// record dated on or after it is a VIOLATION (D0261). Deriving the verdict from the RECORD'S OWN
/// DATE is what makes this a ratchet rather than a second hand-maintained baseline that drifts.
pub(crate) const LEGACY_ACTOR_CUTOFF: &str = "2026-06-12";

/// The `judgedAt`/`createdAt` date declared on the same line, if any. Item declarations in this
/// corpus are single-line, so the line carries its own date; a line without one is treated as
/// undatable history rather than assumed recent.
pub(crate) fn record_date(line: &str) -> Option<String> {
    for attr in ["judgedAt", "createdAt", "saidAt", "acceptedAt"] {
        if let Some(rest) = line.split(attr).nth(1) {
            let digits: String =
                rest.trim_start_matches([' ', '=', '"']).chars().take(10).collect();
            if digits.len() == 10 && digits.as_bytes().get(4) == Some(&b'-') {
                return Some(digits);
            }
        }
    }
    None
}

pub(crate) fn load_known_actors(root: &Path) -> HashSet<String> {
    let mut known = HashSet::new();
    let Ok(text) = keel_model::corpus::read_to_string(root.join(".tracking").join("actors.sysml")) else {
        return known;
    };
    for line in text.lines() {
        // ^\s*part\s+(\w+)\s*:\s*(?:Person|Actor)\b
        let t = line.trim_start_matches(is_space);
        let Some(after) = t.strip_prefix("part") else { continue };
        let after_ws = after.trim_start_matches(is_space);
        if after_ws.len() == after.len() {
            continue;
        }
        let ident: String = after_ws.chars().take_while(|c| keel_model::algo::is_word(*c)).collect();
        if ident.is_empty() {
            continue;
        }
        let Some(r) = after_ws.strip_prefix(ident.as_str()) else { continue };
        let r = r.trim_start_matches(is_space);
        let Some(r) = r.strip_prefix(':') else { continue };
        let r = r.trim_start_matches(is_space);
        let is_actor = ["Person", "Actor"].iter().any(|kw| {
            r.strip_prefix(kw).is_some_and(|tail| tail.chars().next().is_none_or(|c| !keel_model::algo::is_word(c)))
        });
        if is_actor {
            known.insert(ident);
        }
    }
    known
}

/// Values of `:>> authoredBy|createdBy|judgedBy = "..."` on a line.
pub(crate) fn scan_actor_refs(line: &str) -> Vec<String> {
    let mut vals = Vec::new();
    for chunk in line.split(":>>").skip(1) {
        let c = chunk.trim_start_matches(is_space);
        for attr in ACTOR_ATTRS {
            if let Some(rest) = c.strip_prefix(attr) {
                let rest = rest.trim_start_matches(is_space);
                if let Some(rest) = rest.strip_prefix('=') {
                    let rest = rest.trim_start_matches(is_space);
                    if let Some(rest) = rest.strip_prefix('"') {
                        let val: String = rest.chars().take_while(|c| *c != '"').collect();
                        if !val.is_empty() {
                            vals.push(val);
                        }
                    }
                }
                break; // the chunk started with this attr name; don't test the others
            }
        }
    }
    vals
}

/// Guard: every `authoredBy`/`createdBy`/`judgedBy` value references a known `ProjectActor`
/// (or a tolerated legacy actor). Mirrors `validate_actors.py`.
#[must_use]
pub fn actors(root: &Path) -> GuardReport {
    let known = load_known_actors(root);
    let legacy: HashSet<&str> = LEGACY_ACTORS.iter().copied().collect();
    let mut warnings = Vec::new();
    let mut violations = Vec::new();
    let mut legacy_historic = 0usize;
    let files = keel_model::corpus::collect_sysml(&root.join(".tracking"));
    let scanned = files.len();
    for path in &files {
        let Ok(text) = keel_model::corpus::read_to_string(path) else { continue };
        let rel = relpath(root, path);
        for (i, line) in text.lines().enumerate() {
            for val in scan_actor_refs(line) {
                if known.contains(&val) {
                    continue;
                }
                if legacy.contains(val.as_str()) {
                    // A legacy name in a record dated ON OR AFTER the convention is a VIOLATION,
                    // not tolerated history — the date comes from the record itself, so this needs
                    // no baseline list to drift (D0261). Older ones are COUNTED, not enumerated:
                    // 52 undischargeable lines per run were 54% of the whole warning channel, and
                    // real findings sat unread behind them for four days.
                    if record_date(line).is_some_and(|d| d.as_str() >= LEGACY_ACTOR_CUTOFF) {
                        violations.push(format!(
                            "{rel}:{}: legacy actor \"{val}\" in a record dated on/after {LEGACY_ACTOR_CUTOFF} \
                             — legacy names are tolerated only in history that predates the convention",
                            i + 1
                        ));
                    } else {
                        legacy_historic += 1;
                    }
                } else {
                    violations.push(format!("{rel}:{}: unknown actor \"{val}\" not in ProjectActors", i + 1));
                }
            }
        }
    }
    if legacy_historic > 0 {
        warnings.push(history_line(&format!(
            "{legacy_historic} legacy actor reference(s) in records predating the {LEGACY_ACTOR_CUTOFF}              convention — immutable history, NOT dischargeable (rewriting a judgedBy would falsify              provenance). Counted, not enumerated: a warning nobody can act on trains blindness to              the ones they can. A legacy name dated on/after the cutoff is a violation above."
        )));
    }
    GuardReport { name: "actors", scanned, warnings, violations }
}

/// Guard (issue109): every typed-edge endpoint resolves to a declared item.
///
/// HARD, and it is an honest-state gate rather than a completeness one: a dangling edge does not
/// mean work is unfinished, it means the model asserts a relationship that is not there. `issue060`
/// read as triaged by a resolver declared in no commit; a delivery Story read as chartered by an
/// origin that never existed. Both survived every existing check, because `issues` and `charter`
/// each verify that the EDGE is present and neither resolves its endpoints.
///
/// Both were fixed before this guard was added, so it starts at zero — no grandfathering needed and
/// none granted (issue068 forbids retro-failing work that was correct when written; this work was
/// not correct when written, it was undetected).
#[must_use]
pub fn edge_endpoints(root: &Path) -> GuardReport {
    match keel_view::view::dangling_edge_endpoints_scanned(root) {
        Ok((scanned, bad)) => {
            let violations = bad
                .into_iter()
                .map(|e| format!("{e} — a typed edge must connect two declared items; declare the item or remove the edge, never repoint it at something convenient"))
                .collect();
            GuardReport { name: "edge-endpoints", scanned, warnings: Vec::new(), violations }
        }
        Err(e) => GuardReport { name: "edge-endpoints", scanned: 0, warnings: Vec::new(), violations: vec![format!("error resolving edge endpoints: {e}")] },
    }
}

// ── marker-vocabulary guard (an undeclared/misspelled marker silently blinds a control) ───────────

/// Guard: every metadata marker used must be DECLARED (D0133 / issue077).
///
/// Markers were never type-checked, so a MISSPELLED marker validated clean and silently removed that
/// item from the depending control's view — and a blind guard reports PASS, not a violation. The
/// exposure was concentrated: `#Verify` carries 456 edges and is what `tier-satisfaction`,
/// `sr_verified_pct` and the `verification-trace` guard all key on, so a single typo would report a
/// DELIVERED requirement as unverified. `#DerivedFrom` (37 edges) is load-bearing for the HARD
/// `requirement-rootedness` guard.
///
/// HARD-blocking: a typo that blinds a control makes the model's computed state a lie, which is
/// ill-formed STATE rather than incomplete work — squarely inside the honest-state gate (D0098).
/// Safe to make hard because the check is exact (a declared-name set membership), not heuristic.
#[must_use]
pub fn marker_vocabulary(root: &Path) -> GuardReport {
    // Project-declared markers may be declared ANYWHERE in .engine or .tracking (D0136): a downstream
    // project must be able to declare its OWN vocabulary in its OWN files, without being forced into a
    // frozen-schema (§2.5) change just to keep committing.
    let mut files = keel_model::corpus::collect_sysml(&root.join(".tracking"));
    files.extend(keel_model::corpus::collect_sysml(&root.join(".engine")));
    let declared_texts: Vec<String> = files.iter().filter_map(|p| keel_model::corpus::read_to_string(p).ok()).collect();
    let declared = markers_declared(&declared_texts);
    let mut scanned = 0usize;
    let mut violations = Vec::new();
    for path in &files {
        let Ok(text) = keel_model::corpus::read_to_string(path) else { continue };
        let rel = relpath(root, path);
        for (marker, line) in markers_used(&text) {
            scanned += 1;
            if !declared.contains(&marker) {
                violations.push(format!(
                    "{rel}:{line}: marker `#{marker}` is NOT declared as a `metadata def` — markers are not type-checked, so an undeclared or MISSPELLED marker validates clean and silently removes this item from whatever guard or view depends on it (issue077/D0133). If it is a TYPO, fix the spelling. If it is your project's own marker, declare `metadata def {marker};` in any of your own .engine or .tracking files — you do NOT need to touch frozen schema/core (D0136). The engine's own markers are always valid without declaration."
                ));
            }
        }
    }
    GuardReport { name: "marker-vocabulary", scanned, warnings: Vec::new(), violations }
}

// ── duplicate-identity guard (concurrent allocation otherwise lands GREEN) ─────────────────────

/// Extract the value of a `:>> <attr> = "<value>";` assignment appearing anywhere in `line`.
///
/// Attributes are frequently written inline (`part x : TestResult { :>> id = "…"; :>> outcome = …; }`),
/// so this searches the whole line rather than anchoring at the start.
pub(crate) fn inline_attr(line: &str, attr: &str) -> Option<String> {
    let mut rest = line;
    while let Some(pos) = rest.find(":>>") {
        let after = &rest[pos + 3..];
        let trimmed = after.trim_start();
        if let Some(tail) = trimmed.strip_prefix(attr) {
            let tail = tail.trim_start();
            if let Some(tail) = tail.strip_prefix('=') {
                let tail = tail.trim_start();
                if let Some(tail) = tail.strip_prefix('"') {
                    if let Some(end) = tail.find('"') {
                        return Some(tail[..end].to_owned());
                    }
                }
            }
        }
        rest = after;
    }
    None
}

/// Declaration keywords whose following token is an instance/definition NAME.
pub(crate) const DECL_KEYWORDS: [&str; 6] = ["part", "action", "verification", "requirement", "item", "use case"];

/// Name declared by `line`, if it is an instance declaration (not a reference or an edge).
pub(crate) fn declared_name(line: &str) -> Option<String> {
    // Strip a leading metadata marker prefix (`#ProspectiveChange part d0129 : Decision {`).
    let line = if line.starts_with('#') {
        line.split_once(' ').map_or("", |(_, r)| r).trim_start()
    } else {
        line
    };
    // Edges and successions mention names but declare none.
    for skip in ["first ", "flow ", "satisfy ", "verify ", "allocate ", "then ", "private ", "import ", "doc "] {
        if line.starts_with(skip) {
            return None;
        }
    }
    for kw in DECL_KEYWORDS {
        let Some(rest) = line.strip_prefix(kw) else { continue };
        let rest = rest.trim_start();
        // `part def Foo` / `action def Bar` declare a TYPE — still a name in the package scope.
        let rest = rest.strip_prefix("def ").map_or(rest, str::trim_start);
        let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
        if name.is_empty() {
            continue;
        }
        // Must be followed by a declaration token, not more words (which would make it a phrase).
        let after = rest[name.len()..].trim_start();
        if after.starts_with(':') || after.starts_with(';') || after.starts_with('{') {
            return Some(name);
        }
    }
    None
}

/// Guard: no repeated identity anywhere in the model (issue074 / D0129).
///
/// This is the failure class where the **absence** of a git conflict is the danger. Git protects
/// against concurrent edits to the same LINES; it does not protect against concurrent allocation of
/// the same NAME. Two contributors working offline both mint the next decision number by directory
/// scan (`write.rs::next_decision_number`); because their slugs differ the FILENAMES differ, so git
/// reports no conflict, both land, and the resulting duplicate `package DecisionNNNN` declarations
/// are silently merged by the registry (`keel-parser/src/registry.rs`, `or_default()`). The same shape
/// applies to two `sprintNNN_*` files (both counted by `in_progress_sprints`) and to hand-appended
/// `issueNNN` names. Corruption therefore lands GREEN and is undetectable afterwards.
///
/// Four classes are detected:
/// 1. repeated element `id` — identity itself (CLAUDE.md §2.3), also the backstop for a UUID collision
/// 2. repeated declared item name within one package
/// 3. repeated `package` name across files — the silently-merged case
/// 4. repeated allocated sequence number (decision file `NNNN-`, sprint file `sprintNNN_`)
///
/// Per D0047 a recurrable defect class gets a permanent automated control, not vigilance — and this
/// one bit the engine's own authors during D0129 (two workstreams both allocated `issue071`; nothing
/// warned, because the two claims lived in different files).
#[must_use]
pub fn duplicate_identity(root: &Path) -> GuardReport {
    let mut paths = keel_model::corpus::collect_sysml(&root.join(".tracking"));
    paths.extend(keel_model::corpus::collect_sysml(&root.join(".engine")));
    let scanned = paths.len();
    let files: Vec<(String, String)> = paths
        .iter()
        .filter_map(|p| keel_model::corpus::read_to_string(p).ok().map(|t| (relpath(root, p), t)))
        .collect();

    let (warnings, mut violations) = duplicate_scan(&files);

    // Class 4 — allocated sequence numbers embedded in FILENAMES (no git conflict when slugs differ).
    violations.extend(duplicate_sequence(root, &root.join(".engine").join("decisions"), "", 4));
    violations.extend(duplicate_sequence(root, &root.join(".tracking").join("delivery"), "sprint", 0));

    GuardReport { name: "duplicate-identity", scanned, warnings, violations }
}

/// Pure core of `duplicate_identity`: scan `(relpath, text)` pairs for repeated ids, item names and
/// package names. Returns `(warnings, violations)`.
///
/// The warnings channel is retained but now always empty for ids: issue080's 18 bootstrap duplicates
/// were re-identified by a D0067 migration, so there is no exemption path left and every duplicate
/// fails. The tuple shape is kept because the item-name and package-name scans share this function.
pub(crate) fn duplicate_scan(files: &[(String, String)]) -> (Vec<String>, Vec<String>) {
    let mut violations = Vec::new();
    let warnings = Vec::new();
    let mut ids: HashMap<String, String> = HashMap::new();
    let mut pkgs: HashMap<String, String> = HashMap::new();
    let mut items: HashMap<(String, String), String> = HashMap::new();

    for (rel, text) in files {
        let mut cur_pkg = String::new();

        for (i, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with("//") || line.starts_with('*') || line.starts_with("/*") {
                continue;
            }
            let loc = format!("{rel}:{}", i + 1);

            if let Some(rest) = line.strip_prefix("package ") {
                let name = rest.split_whitespace().next().unwrap_or("").trim_end_matches('{').trim();
                if !name.is_empty() {
                    if cur_pkg.is_empty() {
                        name.clone_into(&mut cur_pkg);
                    }
                    if let Some(prev) = pkgs.insert(name.to_owned(), loc.clone()) {
                        violations.push(format!(
                            "{loc}: duplicate package name `{name}` (also declared at {prev}) — the registry SILENTLY MERGES same-named packages, so this corruption would land green (issue074)"
                        ));
                    }
                }
                continue;
            }

            if let Some(id) = inline_attr(line, "id") {
                if let Some(prev) = ids.insert(id.clone(), loc.clone()) {
                    // No grandfather list any more (issue080 RESOLVED): the 18 bootstrap duplicates
                    // across 26 records were re-identified by a D0067 migration, so every duplicate
                    // from here is a live corruption and fails. Keeping an empty exemption list
                    // around would be an invitation to refill it.
                    violations.push(format!(
                        "{loc}: duplicate element id \"{id}\" (also at {prev}) — identity is the invariant that lets items share a name (§2.3); a collision corrupts it"
                    ));
                }
            }

            if let Some(name) = declared_name(line) {
                let key = (cur_pkg.clone(), name.clone());
                if let Some(prev) = items.insert(key, loc.clone()) {
                    violations.push(format!(
                        "{loc}: duplicate declared name `{name}` in package `{cur_pkg}` (also at {prev}) — concurrent allocation produces no git conflict, so nothing else would warn"
                    ));
                }
            }
        }
    }

    (warnings, violations)
}

/// Detect two files in `dir` that claim the same allocated sequence number.
///
/// `prefix` is stripped before reading digits (`sprint163_x` -> `163`); `width` > 0 requires exactly
/// that many leading digits (decision files are zero-padded `0129-`).
pub(crate) fn duplicate_sequence(root: &Path, dir: &Path, prefix: &str, width: usize) -> Vec<String> {
    let mut seen: HashMap<String, String> = HashMap::new();
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else { return out };
    let mut paths: Vec<_> = entries.filter_map(Result::ok).map(|e| e.path()).collect();
    paths.sort();
    for path in paths {
        if path.extension().and_then(|e| e.to_str()) != Some("sysml") {
            continue;
        }
        let Some(stem) = path.file_name().and_then(|n| n.to_str()) else { continue };
        let Some(rest) = stem.strip_prefix(prefix) else { continue };
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        if digits.is_empty() || (width > 0 && digits.len() != width) {
            continue;
        }
        let rel = relpath(root, &path);
        if let Some(prev) = seen.insert(digits.clone(), rel.clone()) {
            out.push(format!(
                "{rel}: sequence number {prefix}{digits} already allocated by {prev} — two contributors allocated it independently; different slugs mean git reported NO conflict (issue074)"
            ));
        }
    }
    out
}

// ── type-collision guard (userDefinedTypedefs, D0128) ────────────────────────

/// A `<kind> def <Name>` declaration line's name.
pub(crate) fn declared_type_name(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let t = trimmed.strip_prefix("abstract ").unwrap_or(trimmed);
    let mut it = t.split_whitespace();
    let first = it.next()?;
    if !first.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    let mut next = it.next()?;
    if next == "case" {
        next = it.next()?; // `use case def X`
    }
    if next != "def" {
        return None;
    }
    let name = it.next()?;
    let name = name.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_');
    (!name.is_empty() && name.chars().next().is_some_and(char::is_alphabetic)).then_some(name)
}

/// Guard: a PROJECT type must not shadow an ENGINE type (D0128 userDefinedTypedefs).
///
/// A project declaring its own domain types in `.tracking/` is supported and wanted — that is the
/// whole point of userDefinedTypedefs, and it already resolves. What is NOT safe is a project
/// declaring a name the engine already defines. Measured before building this: a project
/// `part def Story :> Element` validates CLEAN today and passes every guard, while `Story` is the
/// type `orient` counts work by. Whichever definition wins, the other is silently ignored, and a
/// computed view starts counting something other than what the reader believes — with no diagnostic
/// anywhere. Silence is the defect; the collision itself is easy to fix once seen.
///
/// HARD, and it starts at zero: 91 engine defs against 305 project defs in this repo produce no
/// collision today, so nothing is grandfathered and nothing needs to be (issue068 protects work that
/// was correct when written; there is none to protect here).
///
/// Names only, deliberately. Whether the project MEANT to extend or to replace the engine type is
/// not decidable from the text, and a guard that guessed would be wrong in one direction or the
/// other. Reporting the shadow and letting the author rename is exact.
#[must_use]
pub fn type_collision(root: &Path) -> GuardReport {
    let mut engine: HashMap<String, String> = HashMap::new();
    for path in keel_model::corpus::collect_sysml(&root.join(".engine").join("schema")) {
        let Ok(text) = keel_model::corpus::read_to_string(&path) else { continue };
        let rel = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().replace('\\', "/");
        for (i, line) in text.lines().enumerate() {
            if let Some(n) = declared_type_name(line) {
                engine.entry(n.to_owned()).or_insert_with(|| format!("{rel}:{}", i + 1));
            }
        }
    }
    let mut scanned = 0usize;
    let mut violations = Vec::new();
    for path in keel_model::corpus::collect_sysml(&root.join(".tracking")) {
        let Ok(text) = keel_model::corpus::read_to_string(&path) else { continue };
        let rel = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().replace('\\', "/");
        for (i, line) in text.lines().enumerate() {
            let Some(n) = declared_type_name(line) else { continue };
            scanned += 1;
            if let Some(where_engine) = engine.get(n) {
                violations.push(format!(
                    "{rel}:{}: project type `{n}` SHADOWS the engine type declared at {where_engine} — whichever definition wins, the other is silently ignored and a computed view starts counting something other than what the reader believes. Rename the project type (D0128: projects extend the model, they do not redefine it)",
                    i + 1
                ));
            }
        }
    }
    GuardReport { name: "type-collision", scanned, warnings: Vec::new(), violations }
}

// ── ownership + attestation authority (D0129 srDcAuthorityFromRegistry; mechanizes D0108) ────────

/// Item name -> (`createdBy`, its attribute assignments) parsed from one `.sysml` source.
///
/// Deliberately AST-based rather than diff-line based: a diff hunk does not know which item a
/// changed line belongs to, and guessing from indentation would misattribute an edit — the one
/// thing an ownership check must never do.
/// Is a non-owner diff exactly the sanctioned ACCEPT TRANSFORM (D0205) — the same attribute set
/// with only `status` moving from proposed to accepted?
pub(crate) fn is_accept_transform(old_attrs: &[String], new_attrs: &[String]) -> bool {
    if old_attrs.len() != new_attrs.len() {
        return false;
    }
    let mut status_flip = false;
    let old_set: std::collections::HashSet<&String> = old_attrs.iter().collect();
    let new_set: std::collections::HashSet<&String> = new_attrs.iter().collect();
    for gone in old_set.difference(&new_set) {
        if gone.starts_with("status=") && gone.contains("proposed") {
            status_flip = true;
        } else {
            return false; // some other attribute changed — not the sanctioned transform
        }
    }
    for came in new_set.difference(&old_set) {
        if !(came.starts_with("status=") && came.contains("accepted")) {
            return false;
        }
    }
    status_flip
}

pub(crate) fn items_with_attrs(src: &str, filename: &str) -> HashMap<String, (String, Vec<String>)> {
    let mut out = HashMap::new();
    let Ok(tokens) = keel_parser::tokenize(src, filename) else { return out };
    let Ok(pkg) = keel_parser::parse(tokens, filename) else { return out };
    let mut note = |name: &str, attrs: &[keel_parser::ast::Attribute]| {
        let mut pairs: Vec<String> = attrs
            .iter()
            .map(|a| format!("{}={}", a.name, keel_view::view::attr_value_string(&a.value)))
            .collect();
        pairs.sort();
        let created_by = attrs
            .iter()
            .find(|a| a.name == "createdBy")
            .map(|a| keel_view::view::attr_value_string(&a.value))
            .unwrap_or_default();
        out.insert(name.to_owned(), (created_by, pairs));
    };
    for item in &pkg.items {
        match item {
            keel_parser::ast::Item::Part(p) => note(&p.name, &p.attributes),
            keel_parser::ast::Item::Verification(v) => note(&v.name, &v.attributes),
            keel_parser::ast::Item::UseCase(u) => note(&u.name, &u.attributes),
            keel_parser::ast::Item::ActionUsage(a) => note(&a.name, &a.attributes),
            // Items nested inside a delivery `action def` — where every sprint's gates live, and so
            // the densest concentration of owned fields in the model.
            keel_parser::ast::Item::ActionDef(d) => {
                for p in &d.parts {
                    note(&p.name, &p.attributes);
                }
                for v in &d.verifications {
                    note(&v.name, &v.attributes);
                }
            }
            _ => {}
        }
    }
    out
}

/// The file's content at HEAD, or `None` if it is newly added.
pub(crate) fn head_blob(root: &Path, path: &str) -> Option<String> {
    let out = keel_git::gitx::git()
        .arg("-C")
        .arg(root)
        .args(["show", &format!("HEAD:{path}")])
        .output()
        .ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Guard (D0108/D0129): only an item's OWNER edits its fields.
///
/// D0108's coordination contract — owner-of-record edits fields, a non-owner may ADD items and typed
/// edges or SUPERSEDE, never overwrite in place — was CONVENTION ONLY: absent from every guard and
/// every declared rule, enforced by prose plus a reminder hook. D0047 is explicit that manual
/// vigilance is not a control, and in-place write paths exist that can clobber another actor's item.
///
/// WHAT PASSES, deliberately: adding a new item, adding a typed edge, and superseding all leave every
/// existing item's fields untouched, so they are invisible to this check by construction rather than
/// by an exemption list that could drift. Only a CHANGED attribute on an item someone else created
/// is a violation.
///
/// Compares the staged file against its HEAD blob AST-to-AST. A diff hunk does not know which item a
/// line belongs to; attributing an edit to the wrong owner would be worse than not checking.
#[must_use]
pub fn ownership(root: &Path) -> GuardReport {
    let actor = keel_actor::actor::resolve(root, None).ok();
    let read = ChangeRead::current();
    let all_staged = changed_files(root, read);
    // A GOVERNED MIGRATION is the sanctioned exception, and it needs one because these two controls
    // genuinely collide: D0108 forbids a non-owner editing another actor's fields, while D0067
    // REQUIRES bulk transforms that cross every ownership boundary at once (repairing 26 duplicated
    // ids is exactly that). Blocking a migration would have made D0067 unexecutable; exempting on a
    // flag would have made D0108 optional.
    //
    // The exemption is therefore the same keystone shape `process-change` already uses: the change
    // is permitted only when a transform under `.engine/tools/migrations/` is CO-COMMITTED, which is
    // what D0067 demands anyway (a committed transform, a dry run, reconciled control totals). It
    // cannot be claimed — it has to be in the commit, where a reviewer can read it.
    let migration_co_committed = all_staged.iter().any(|p| p.starts_with(".engine/tools/migrations/"));
    if migration_co_committed {
        return GuardReport {
            name: "ownership",
            scanned: 0,
            warnings: vec![
                "cross-owner edits ALLOWED: a migration transform under .engine/tools/migrations/ is co-committed (D0067). Ownership (D0108) is suspended for this commit and the transform is the record of why.".to_owned(),
                read_line(read),
            ],
            violations: Vec::new(),
        };
    }
    let staged: Vec<String> = all_staged.into_iter()
        .filter(|p| std::path::Path::new(p).extension().is_some_and(|e| e.eq_ignore_ascii_case("sysml")))
        .collect();
    let mut violations = Vec::new();
    let mut scanned = 0usize;
    for path in &staged {
        let Some(before) = head_blob(root, path) else { continue }; // newly added file — all additions
        let Ok(after) = keel_model::corpus::read_to_string(root.join(path)) else { continue };
        let old = items_with_attrs(&before, path);
        let new = items_with_attrs(&after, path);
        for (name, (owner, new_attrs)) in &new {
            let Some((old_owner, old_attrs)) = old.get(name) else { continue }; // added item
            scanned += 1;
            if old_attrs == new_attrs {
                continue;
            }
            // D0205 (githubChannel): the ACCEPT TRANSFORM is the one sanctioned non-owner edit — a
            // recording channel (the GitHub Action, the serve endpoint) flips a Decision's status
            // from proposed to accepted on the human's authenticated gesture. Recognized MECHANICALLY:
            // the ONLY attribute that changed is `status`, exactly proposed -> accepted. Anything
            // else a non-owner touches (title, rationale, a second attr riding along) still violates.
            // The acceptance EVENT items the same write appends are ADDS and were always permitted.
            if is_accept_transform(old_attrs, new_attrs) {
                continue;
            }
            let owner = if old_owner.is_empty() { owner } else { old_owner };
            if owner.is_empty() {
                continue; // no recorded owner — nothing to enforce against, and inventing one is worse
            }
            match &actor {
                Some(a) if a == owner => {}
                Some(a) => violations.push(format!(
                    "{path}: '{name}' is owned by '{owner}' and its fields were edited by '{a}' — D0108: a non-owner ADDS items and typed edges or SUPERSEDES, never overwrites in place. Author a superseding item, or have the owner make the change."
                )),
                None => violations.push(format!(
                    "{path}: '{name}' (owned by '{owner}') had fields edited, but this machine has no bound actor, so the edit cannot be attributed. Run `keel actor set <id>` — provenance is never defaulted (D0129)."
                )),
            }
        }
    }
    GuardReport { name: "ownership", scanned, warnings: vec![read_line(read)], violations }
}

/// Guard (issue166): every id-bearing declaration actually carries an `:>> id`.
///
/// HARD, and it closes an invariant that was unguarded. §1.3 makes identity an immutable UUID so items
/// never collide on name — and `keel gate validate` passed with an `Issue` missing its `id` entirely. The only
/// existing coverage was `engine-lint`, which is `.engine`-scoped by design, and the demoted python
/// tracking validator (D0132), which fails correct files and so cannot be relied on. `duplicate-identity`
/// catches two items SHARING an id and says nothing about an item having none.
///
/// A TEXT SCAN, not a model walk: an item with no identity is exactly the thing the model layer cannot
/// see clearly, and this needs to run at commit speed. Measured at ~60ms over 8738 declarations, against
/// a 156ms model build it does not perform.
///
/// STARTS AT ZERO with no grandfather line, because the corpus was measured first: 8738 id-bearing
/// declarations across `.tracking` and `.engine`, none missing an id. A forward-only exemption would have
/// been ceremony over an empty set.
#[must_use]
pub fn identity_present(root: &Path) -> GuardReport {
    let mut files = keel_model::corpus::collect_sysml(&root.join(".tracking"));
    files.extend(keel_model::corpus::collect_sysml(&root.join(".engine")));
    let mut scanned = 0usize;
    let mut violations = Vec::new();
    for path in &files {
        let Ok(text) = keel_model::corpus::read_to_string(path) else { continue };
        let rel = relpath(root, path);
        let decls = id_bearing_decls(&text);
        for (name, ty, line, body) in decls {
            scanned += 1;
            if !body.contains(":>> id") {
                violations.push(format!(
                    "{rel}:{line}: {name} : {ty} carries NO `:>> id` - identity is an immutable UUID (section 1.3); an item without one cannot be referenced, superseded or attested against"
                ));
            }
        }
    }
    GuardReport { name: "identity-present", scanned, warnings: Vec::new(), violations }
}

#[must_use]
/// Guard 38: an `id` must be SHAPED like a UUID — 8-4-4-4-12 of `[0-9a-z]` (issue170/D0168).
///
/// Guard 37 checks an id is PRESENT and `duplicate-identity` checks two items do not SHARE one. The
/// middle property — that the string is an identifier at all — was enforced by nothing, and an id of
/// literally `not-a-uuid-at-all` passed `keel gate validate` and all 37 guards. A malformed id is still
/// UNIQUE, so it collides with nothing and every view resolves it happily; the damage is silent and
/// surfaces only when something outside this repo tries to join on it.
///
/// SHAPE, NOT STRICT HEX, and the corpus is why: 78 ids deliberately carry a mnemonic suffix
/// (`…-000000000i01` for the intake process steps), which is UUID-shaped but not hexadecimal. Those are
/// intentional and readable, and a guard that failed them would be demanding a migration nobody asked
/// for. Shape catches every real defect — the two mangled ids that prompted this, and the 15 historical
/// ones below — while leaving a deliberate convention alone.
///
/// THE GRANDFATHER SET IS AN EXPLICIT LIST, not a date. Guard 36's first version keyed its exemption on
/// a date and thereby exempted the very defect it existed for. Fifteen named strings cannot absorb a
/// sixteenth: a new malformed id fails, no matter when it is written. They are not REWRITTEN because
/// section 1.3 makes identity immutable — an id is wrong here, and changing it would be a second wrong.
pub fn identity_well_formed(root: &Path) -> GuardReport {
    /// Ids that predate the guard. Malformed (7- and 9-character first groups) and immutable.
    const GRANDFATHERED: [&str; 15] = [
        "be4dae8-5f6a-4b7c-def8-9a0b1c2d3e4f",
        "cf5ebl9-7b8c-4d9e-efa0-1c2d3e4f5a6b",
        "d0105r001-0001-4001-9001-516273841001",
        "d0105r002-0002-4002-9002-516273841002",
        "d0105r003-0003-4003-9003-516273841003",
        "d0105r004-0004-4004-9004-516273841004",
        "d0105r005-0005-4005-9005-516273841005",
        "d0105r006-0006-4006-9006-516273841006",
        "d0105r007-0007-4007-9007-516273841007",
        "d0105r008-0008-4008-9008-516273841008",
        "da6fcm0-8c9d-4e0f-fab1-2d3e4f5a6b7c",
        "da7gdp2-0e1f-4a2b-bcd3-4f5a6b7c8d9e",
        "eb5ebf9-6a7b-4c8d-efa9-0b1c2d3e4f5a",
        "eb8heq3-1f2a-4b3c-cde4-5a6b7c8d9e0f",
        "fc6fcn1-9d0e-4f1a-abc2-3e4f5a6b7c8d",
    ];
    let mut files = keel_model::corpus::collect_sysml(&root.join(".tracking"));
    files.extend(keel_model::corpus::collect_sysml(&root.join(".engine")));
    let mut scanned = 0usize;
    let mut violations = Vec::new();
    for path in &files {
        let Ok(text) = keel_model::corpus::read_to_string(path) else { continue };
        let rel = relpath(root, path);
        for (n, raw) in text.lines().enumerate() {
            let line = raw.trim_start();
            if line.starts_with("//") {
                continue;
            }
            for value in id_values(line) {
                scanned += 1;
                if uuid_shaped(&value) || GRANDFATHERED.contains(&value.as_str()) {
                    continue;
                }
                violations.push(format!(
                    "{rel}:{}: id \"{value}\" is not shaped like a UUID (8-4-4-4-12 of [0-9a-z]) - section 1.3 makes identity an immutable UUID, and a malformed id is still UNIQUE, so nothing else in the model will ever notice",
                    n + 1
                ));
            }
        }
    }
    GuardReport { name: "identity-well-formed", scanned, warnings: Vec::new(), violations }
}

/// Guard 69: an id written from the cutoff on, or added in the working tree, is a v4 UUID (D0430 / issue454).
///
/// Guard 38 checks an id is SHAPED like a UUID - `[0-9a-z]` groups - and the control map claimed
/// "every id is a well-formed v4 UUID" on its strength. On 2026-09-10 a verifier subagent hand-wrote
/// six `TestResult`s around the write API with ids such as `eh5h6i7g-8f9e-0j1h-2i3d-4e5f6g7h8i9d`,
/// and validate and all 68 guards accepted them. `write::gen_uuid` emits RFC 4122 v4 and every id
/// dated that day in the tree is v4, so the v4 shape is the fingerprint that separates an API-written
/// record from a typed one.
///
/// TWO FORWARD CLAUSES, ONE `HISTORY` LINE. (1) An item whose recorded date is on or after the cutoff
/// and whose id is not v4 is a violation. (2) Any non-v4 id on a line ADDED in the working tree
/// relative to HEAD is a violation whatever its date or lack of one - the fabrication issue454 saw
/// was staged, not committed, and an id that did not exist at HEAD is new no matter what it says
/// about itself; this is the clause a date ratchet alone lacks (guard 36's first version exempted
/// the defect it existed for). Everything else that fails the shape - 5,515 hex-but-not-v4 ids
/// (v5s and sequence-shaped ids never typed by hand) and 88 not-hex ones (mnemonic suffixes, the
/// earlier typed ids) on 2026-09-10 - is history, counted by class and never enumerated (D0261):
/// identity is immutable (section 1.3), so none of them can be corrected. STATED RESIDUAL: a typed id
/// that happens to satisfy the nibbles passes; the guard is a fingerprint, the write ledger (D0424) is
/// the proof.
#[must_use]
pub fn id_is_a_uuid(root: &Path) -> GuardReport {
    /// The day the write API's shape became binding on every id (D0430).
    const CUTOFF: &str = "2026-09-10";
    let census = match keel_view::view::id_shape_census(root, CUTOFF) {
        Ok(c) => c,
        Err(e) => {
            return GuardReport { name: "id-is-a-uuid", scanned: 0, warnings: Vec::new(), violations: vec![format!("error building the model: {e}")] };
        }
    };
    let mut violations = Vec::new();
    let mut reported: HashSet<String> = HashSet::new();
    for (item, file, id, date) in &census.forward {
        reported.insert(id.clone());
        violations.push(format!(
            "{file}: {item} is dated {date} and its id \"{id}\" is not an RFC 4122 v4 UUID (8-4-4-4-12 lowercase hex, version nibble 4, variant in 89ab) - the write API emits v4, so a record dated on/after {CUTOFF} with another shape was typed around it (D0430/issue454)"
        ));
    }
    // `git diff HEAD` lists no UNTRACKED file, and a new sprint record is exactly that - the probe
    // (sprint 655) put a typed id in a new file and the diff clause counted it as history. Every line
    // of an untracked `.sysml` under the model roots is an added line.
    let diff = git_stdout(root, &["diff", "HEAD", "-U0", "--", ".tracking", ".engine"]);
    let mut added = added_non_v4_ids(&diff);
    for rel in git_stdout(root, &["ls-files", "--others", "--exclude-standard", "--", ".tracking", ".engine"]).lines() {
        let rel = rel.trim();
        if !std::path::Path::new(rel).extension().is_some_and(|e| e.eq_ignore_ascii_case("sysml")) {
            continue;
        }
        if let Ok(text) = keel_model::corpus::read_to_string(root.join(rel)) {
            added.extend(non_v4_ids_in(rel, &text));
        }
    }
    for (file, id) in added {
        if reported.insert(id.clone()) {
            violations.push(format!(
                "{file}: id \"{id}\" is ADDED in the working tree and is not an RFC 4122 v4 UUID - an id that did not exist at HEAD is new whatever its record says, and the write API would have emitted v4 (D0430/issue454)"
            ));
        }
    }
    let history = census.history_not_hex + census.history_hex_not_v4;
    let warnings = if history == 0 {
        Vec::new()
    } else {
        vec![history_line(&format!(
            "{history} ids on records dated before {CUTOFF} or carrying no date fail the v4 shape - {} hex but not v4 (v5s, sequence-shaped), {} not hex - immutable (section 1.3), counted not enumerated (D0261)",
            census.history_hex_not_v4, census.history_not_hex
        ))]
    };
    GuardReport { name: "id-is-a-uuid", scanned: census.scanned, warnings, violations }
}

/// `(file, id)` for every `:>> id = "..."` on an ADDED line of a unified diff whose value is not v4.
/// Pure over the diff text so the clause is testable without a repository.
pub(crate) fn added_non_v4_ids(diff: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut file = String::new();
    for line in diff.lines() {
        if let Some(rest) = line.strip_prefix("+++ ") {
            file = rest.strip_prefix("b/").unwrap_or(rest).to_string();
            continue;
        }
        if !line.starts_with('+') || line.starts_with("+++") {
            continue;
        }
        out.extend(non_v4_ids_in(&file, &line[1..]));
    }
    out
}

/// `(file, id)` for every non-v4 `:>> id = "..."` in `text`, comment lines skipped.
pub(crate) fn non_v4_ids_in(file: &str, text: &str) -> Vec<(String, String)> {
    text.lines()
        .map(str::trim_start)
        .filter(|l| !l.starts_with("//"))
        .flat_map(id_values)
        .filter(|id| !is_v4_uuid(id))
        .map(|id| (file.to_string(), id))
        .collect()
}

/// Guard 44 (D0191, WARNING tier, owned by the `actor-enrollment` unit).
///
/// When a machine binding (`.keel/actor`) exists, its name resolves to a registered `Person`, or to
/// an `Actor` carrying a declared kind. Until now NOTHING validated the binding file — a name that is unregistered or
/// kindless surfaced only when some downstream write refused. An absent binding scans zero: binding
/// is per-machine and optional until a write needs an actor.
#[must_use]
pub fn enrollment_binding(root: &Path) -> GuardReport {
    let Some(bound) = keel_model::corpus::read_to_string(root.join(keel_actor::actor::BINDING_PATH))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    else {
        return GuardReport { name: "enrollment-binding", scanned: 0, warnings: Vec::new(), violations: Vec::new() };
    };
    let actors = keel_model::corpus::read_to_string(root.join(".tracking").join("actors.sysml")).unwrap_or_default();
    let mut warnings = Vec::new();
    let decl = actors.lines().find_map(|line| {
        let l = line.trim_start();
        l.strip_prefix("part ")
            .and_then(|r| r.split_once(':'))
            .filter(|(n, _)| n.trim() == bound)
            .map(|(_, after)| after.trim_start().to_string())
    });
    match decl {
        None => warnings.push(format!(
            "machine binding `.keel/actor` names `{bound}`, which is NOT a registered actor — enroll it (`keel enroll`) or rebind (`keel actor set`) before it strands a write (D0191)"
        )),
        Some(after) if after.starts_with("Person") => {}
        Some(after) => {
            // An Actor must carry a kind; single-line and block forms both keep the kind within
            // the declaring region, so scan from the declaration to the next `part `.
            let region = actors
                .split_once(&format!("part {bound}"))
                .map(|(_, rest)| rest.split("\npart ").next().unwrap_or(rest).to_string())
                .unwrap_or_default();
            if !region.contains("ActorKind::") {
                warnings.push(format!(
                    "machine binding `.keel/actor` names `{bound}` ({}), which declares NO kind — an actor whose kind is unstated defeats the human/AI attestation distinction (D0106/D0191)",
                    after.split_whitespace().next().unwrap_or("?")
                ));
            }
        }
    }
    GuardReport { name: "enrollment-binding", scanned: 1, warnings, violations: Vec::new() }
}

/// Every `:>> id = "…"` value on one line. A line may carry several: the sprint records declare an item
/// and its result on one line each, and a per-line regex-free scan must not stop at the first.
pub(crate) fn id_values(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(i) = rest.find(":>> id") {
        rest = &rest[i + ":>> id".len()..];
        let Some(after_eq) = rest.split_once('=') else { break };
        let mut chars = after_eq.1.trim_start().chars();
        if chars.next() != Some('"') {
            continue;
        }
        let tail = chars.as_str();
        if let Some(end) = tail.find('"') {
            out.push(tail[..end].to_string());
            rest = &tail[end..];
        } else {
            break;
        }
    }
    out
}

/// `(name, type, 1-based line, body-up-to-the-next-declaration)` for each id-bearing declaration.
///
/// The body stops at the NEXT declaration so a member can never borrow its sibling's id — the bug that
/// would make this guard pass a file where one item has two ids and its neighbour none.
pub(crate) fn id_bearing_decls(text: &str) -> Vec<(String, String, usize, String)> {
    let starts: Vec<(usize, usize, String, String)> = text
        .lines()
        .enumerate()
        .filter_map(|(i, raw)| {
            let line = raw.trim_start();
            if line.starts_with("//") {
                return None;
            }
            let after_marker = line.strip_prefix('#').map_or(line, |r| {
                r.split_once(char::is_whitespace).map_or("", |(_, rest)| rest.trim_start())
            });
            for kw in ["part ", "verification ", "requirement ", "use case "] {
                if let Some(rest) = after_marker.strip_prefix(kw) {
                    let (name, rest) = rest.split_once(':')?;
                    let ty: String =
                        rest.trim_start().chars().take_while(char::is_ascii_alphanumeric).collect();
                    if ENGINE_ID_TYPES.contains(&ty.as_str()) && rest.contains('{') {
                        return Some((i, i + 1, name.trim().to_string(), ty));
                    }
                }
            }
            None
        })
        .collect();
    let lines: Vec<&str> = text.lines().collect();
    starts
        .iter()
        .enumerate()
        .map(|(k, (idx, line_no, name, ty))| {
            let end = starts.get(k + 1).map_or(lines.len(), |n| n.0);
            let body = lines.get(*idx..end).unwrap_or_default().join("\n");
            (name.clone(), ty.clone(), *line_no, body)
        })
        .collect()
}

/// Every `(name, attributes)` pair in a package, including those nested inside an `action def` body —
/// which is where the backlog and every sprint record actually live, so a top-level-only walk would
/// inspect almost nothing.
pub(crate) fn named_attr_bearers(pkg: &keel_parser::ast::Package) -> Vec<(&str, &[keel_parser::ast::Attribute])> {
    use keel_parser::ast::Item;
    let mut out: Vec<(&str, &[keel_parser::ast::Attribute])> = Vec::new();
    for item in &pkg.items {
        match item {
            Item::Part(p) => out.push((p.name.as_str(), &p.attributes)),
            Item::Verification(v) => out.push((v.name.as_str(), &v.attributes)),
            Item::ActionDef(d) => {
                for p in &d.parts {
                    out.push((p.name.as_str(), &p.attributes));
                }
                for v in &d.verifications {
                    out.push((v.name.as_str(), &v.attributes));
                }
            }
            _ => {}
        }
    }
    out
}

/// WARNING-level: a PROJECT-declared marker with no recorded justification (D0139(B)).
///
/// D0139 requires that a custom `metadata def` edge be a last resort carrying a recorded justification
/// naming the base constructs considered and why each fails. That rule exists because a documented
/// PREFERENCE demonstrably does not work here: `sysmlv2-syntax-notes.md:16` already recorded `:>` as the
/// idiomatic derivation form, and `#DerivedFrom` was used 37 times anyway. D0047 is explicit that manual
/// vigilance is not a control.
///
/// Scope is deliberately narrow. The engine's own 17 markers are GRANDFATHERED — forward-only, the
/// issue068 rule — and D0140 has since supplied kernel-verified justifications for the two that needed
/// them. So this fires only on markers a PROJECT declares from here on, of which there are currently
/// zero. A guard that reports nothing today and blocks a bad habit tomorrow is the intended shape.
///
/// TEXT-BASED, and the reason is worth stating rather than hiding: the AST does not capture `doc`
/// clauses or comments, so there is nothing structural to inspect. Capturing doc text is a separate
/// parser increment; until it lands, a text scan is the honest option, and it is scoped to the lines
/// immediately around the declaration rather than the whole file.
pub(crate) fn base_first_justification(root: &Path) -> GuardReport {
    let engine: HashSet<&str> = engine_markers().iter().map(String::as_str).collect();
    let mut warnings = Vec::new();
    let mut scanned = 0usize;
    for dir in [root.join(".tracking"), root.join(".engine")] {
        if !dir.is_dir() {
            continue;
        }
        for path in keel_model::corpus::collect_sysml(&dir) {
            let Ok(text) = keel_model::corpus::read_to_string(&path) else { continue };
            let rel = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().replace('\\', "/");
            let lines: Vec<&str> = text.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                let Some(rest) = line.trim_start().strip_prefix("metadata def ") else { continue };
                // Strip a trailing `// comment` BEFORE taking the name. Without this,
                // `metadata def View;   // marks a computed artifact` yields a name containing the whole
                // comment, so an ENGINE marker fails the grandfather check and is reported as a project
                // marker — which is exactly what happened on the first run.
                let decl = rest.split("//").next().unwrap_or(rest);
                let name = decl.trim_end_matches([';', ' ', '{']).trim();
                if name.is_empty() || engine.contains(name) {
                    continue; // engine markers are grandfathered (issue068)
                }
                scanned += 1;
                // A justification is a `doc` clause on the declaration, or comment lines directly above
                // it. Both are how this repo actually documents its markers today.
                let has_doc = line.contains("doc ")
                    || lines
                        .get(i.saturating_sub(3)..i)
                        .unwrap_or_default()
                        .iter()
                        .any(|l| {
                            let t = l.trim_start();
                            t.starts_with("//") && t.len() > 8
                        });
                if !has_doc {
                    warnings.push(format!(
                        "{rel}:{}: project marker `#{name}` has no recorded justification — D0139(B) requires naming the base SysML v2 constructs considered and why each fails, because a custom edge is a last resort and an undocumented one is dialect nobody can audit",
                        i + 1
                    ));
                }
            }
        }
    }
    GuardReport { name: "base-first-justification", scanned, warnings, violations: Vec::new() }
}

/// WARNING-level: statements the parser could not read, grouped by leading token (issue102).
///
/// The parser recognises a fixed statement set and skips the rest — silently, until now. Measured with
/// an undeclared target, `ref e : NoSuchType`, `port p : NoSuchPortDef`, `assert constraint c :
/// NoSuchConstraint` and `connect ghostA.p to ghostB.p` ALL validate clean, while the control
/// (`part x : NoSuchType`) correctly produces a diagnostic. So those statements are not merely
/// unresolved — they are invisible.
///
/// This is the safety property that makes the rest of the base-first pass survivable. Every construct
/// D0139 converts toward is currently in that invisible set, so a conversion landing before the reader
/// would make its edges parse clean and vanish while every guard reported green — the failure mode this
/// project exists to prevent, and the one issue027 already fixed for items dropped outside a package.
///
/// Reports a per-lead-token count rather than one line per statement: the engine's own schema skips 29
/// statements today, and a 29-line warning block every run would train its reader to ignore the guard.
/// The counts are what shows a conversion going wrong — a lead token appearing where it did not before.
pub(crate) fn parser_coverage(root: &Path) -> GuardReport {
    let mut by_lead: BTreeMap<String, usize> = BTreeMap::new();
    let mut scanned = 0usize;
    for dir in [root.join(".tracking"), root.join(".engine")] {
        if !dir.is_dir() {
            continue;
        }
        for path in keel_model::corpus::collect_sysml(&dir) {
            let Ok(pkg) = keel_model::corpus::parse_pkg(&path) else { continue };
            scanned += 1;
            for sk in &pkg.skipped {
                *by_lead.entry(sk.lead.clone()).or_default() += 1;
            }
        }
    }
    let total: usize = by_lead.values().sum();
    let mut warnings = Vec::new();
    if total > 0 {
        // Rank by count and show only the head. The tail is dominated by element NAMES — a skipped
        // `use case <name> : T` reports its own name as the lead token — which produces dozens of
        // singletons that bury the kinds worth acting on. Collapsing them keeps the guard readable,
        // which is the difference between a control someone reads and one they learn to scroll past.
        let mut ranked: Vec<(&String, &usize)> = by_lead.iter().collect();
        ranked.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
        let head: Vec<String> = ranked.iter().take(8).map(|(l, n)| format!("{l}×{n}")).collect();
        let tail: usize = ranked.iter().skip(8).map(|(_, n)| **n).sum();
        let tail_note =
            if tail > 0 { format!(", and {tail} more across {} kind(s)", ranked.len() - 8) } else { String::new() };
        warnings.push(format!(
            "{total} statement(s) across {scanned} file(s) are SKIPPED by keel-parser and therefore invisible to every guard and view: {}{tail_note}. A base-first conversion onto any of these would parse clean and lose its edges (issue102/D0139)",
            head.join(", ")
        ));
    }
    // issue231 (process-value panel, formal-methods lens): the RATCHET. The parser is the TCB of
    // every guard and view, and a warning whose count can grow silently is a control people learn
    // to scroll past. With a committed baseline declared, EXCEEDING it is a violation — never mere
    // presence (the D0132 all-or-nothing lesson): the 7 legacy successions stay a warning, a NEW
    // skipped class goes red. Shrinking below baseline warns to ratchet the baseline DOWN, so the
    // bound only ever tightens. Absent contract = ratchet not adopted, stated (D0136).
    let mut violations = Vec::new();
    let baseline_path = root.join(".engine").join("contracts").join("parser-coverage-baseline.toml");
    match keel_model::corpus::read_to_string(&baseline_path) {
        Ok(text) => match text.parse::<toml::Value>().ok().and_then(|v| v.get("skipped").and_then(toml::Value::as_integer)) {
            Some(baseline) => {
                let baseline = usize::try_from(baseline).unwrap_or(0);
                if total > baseline {
                    violations.push(format!(
                        "skipped-statement count {total} EXCEEDS the committed baseline {baseline} — a new statement class became invisible to every guard and view (issue231 ratchet). Either teach keel-parser the construct, or raise the baseline IN THE SAME COMMIT with the reason (a visible diff, never silent growth)."
                    ));
                } else if total < baseline {
                    warnings.push(format!(
                        "skipped-statement count {total} is BELOW the baseline {baseline} — ratchet it down in parser-coverage-baseline.toml so the bound keeps what the parser gained (issue231)."
                    ));
                }
            }
            None => violations.push("parser-coverage-baseline.toml exists but has no integer `skipped` key — a malformed ratchet must fail loud, not silently un-adopt".to_string()),
        },
        Err(_) => warnings.push("no parser-coverage baseline declared (.engine/contracts/parser-coverage-baseline.toml) — the skipped count can grow without a red gate (issue231 ratchet not adopted; D0136: absence is a state, stated)".to_string()),
    }
    GuardReport { name: "parser-coverage", scanned, warnings, violations }
}

/// WARNING-level: every multi-valued feature assignment `:>> f = (a, b, c)` in the model (issue101).
///
/// Enabling the sequence form (issue095) removed a crude safety property: `(` used to be a parse error
/// EVERYWHERE, so a sequence could not be written into a single-valued attribute. Now it can, and it is
/// accepted silently — `createdBy = ("you", "ghost")` parses clean and the `actors` guard passes, so an
/// unregistered actor slips through. The precise check — reject a sequence where the schema declares no
/// `[*]` multiplicity — is not yet possible: neither the AST nor the registry captures multiplicity.
///
/// So this reports every sequence instead. That is genuinely useful rather than a placeholder, because
/// the model contains ZERO sequences today: any hit is new, and reviewing it is exactly the check that
/// multiplicity metadata will later automate. Deliberately AST-based, not a text scan — grepping for
/// `= (` matches the prose in Decisions and definition-of-done text that discusses the sequence form, which is the
/// self-inflating-census error of issue099 in miniature.
///
/// Retire this guard when multiplicity lands and the exact check replaces it.
pub(crate) fn sequence_multiplicity(root: &Path) -> GuardReport {
    use keel_parser::ast::Value;
    let mut warnings = Vec::new();
    let mut scanned = 0usize;
    for dir in [root.join(".tracking"), root.join(".engine")] {
        if !dir.is_dir() {
            continue;
        }
        for path in keel_model::corpus::collect_sysml(&dir) {
            let Ok(pkg) = keel_model::corpus::parse_pkg(&path) else {
                continue; // parse errors are validate's business, not this guard's
            };
            let rel = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().replace('\\', "/");
            for (item_name, attrs) in named_attr_bearers(&pkg) {
                for a in attrs {
                    if let Value::Seq(items) = &a.value {
                        scanned += 1;
                        warnings.push(format!(
                            "{rel}:{}: {item_name}.{} is a {}-element sequence — confirm the schema declares this feature multi-valued ([*]); a sequence in a single-valued attribute is accepted silently today (issue101)",
                            a.line,
                            a.name,
                            items.len()
                        ));
                    }
                }
            }
        }
    }
    GuardReport { name: "sequence-multiplicity", scanned, warnings, violations: Vec::new() }
}

// ── engine-lint guard (D0112 phase 1: the mechanical .engine instance lints, ported kernel-free) ──

/// Instance types that carry an `:>> id` (identity invariant §2.3). Mirrors `validate_instances._ID_TYPES`.
pub(crate) const ENGINE_ID_TYPES: &[&str] = &[
    "Decision", "AISkill", "Agent", "Process", "ProcessStep", "TestResult", "Brief", "Persona", "Need",
    "Issue", "Story", "Release", "ChangeRequest", "Component", "DesignElement", "Test", "Viewpoint",
    "Indicator", "Measurement",
    // Intake (D0166). A type absent from this list has its IDENTITY UNCHECKED - engine-lint never asks
    // whether it carries an `:>> id` - so a new item type is only half-registered until it is here.
    "Statement", "UserStory",
];

/// Count `part|verification|requirement <name> : <IdType>` declarations in `text`.
///
/// Line-based mirror of `validate_instances.warn_missing_ids`.
pub(crate) fn count_tracked_instances(text: &str) -> usize {
    text.lines()
        .filter(|line| {
            let t = line.trim_start();
            ["part ", "verification ", "requirement "].iter().any(|kw| {
                t.strip_prefix(kw)
                    .and_then(|rest| rest.split_once(':'))
                    .map(|(_, after)| after.trim_start().split(|c: char| !c.is_alphanumeric()).next().unwrap_or(""))
                    .is_some_and(|ty| ENGINE_ID_TYPES.contains(&ty))
            })
        })
        .count()
}

/// The two mechanical `.engine`-instance lints, ported kernel-free (D0112 phase 1).
///
/// (1) HARD — every `.engine/decisions/*.sysml` must `import EngineWork` (the `Decision` type lives
/// there). (2) WARN — every tracked instance (`part|verification|requirement <name> : <IdType>`) should
/// carry an `:>> id` (§2.3). The first kernel-free step of retiring the JVM from the `.engine` path;
/// parity with the python lints by VERDICT, not byte-identical text.
#[must_use]
pub fn engine_lint(root: &Path) -> GuardReport {
    let mut warnings = Vec::new();
    let mut violations = Vec::new();
    let decisions_dir = root.join(".engine").join("decisions");
    let decision_files = keel_model::corpus::collect_sysml(&decisions_dir);
    // (1) HARD: import-EngineWork on every decision file.
    for path in &decision_files {
        if let Ok(text) = keel_model::corpus::read_to_string(path) {
            if !text.contains("import EngineWork") {
                violations.push(format!(
                    "{}: Decision file missing 'import EngineWork' — the Decision type lives in EngineWork (D0112)",
                    relpath(root, path)
                ));
            }
        }
    }
    // (2) WARN: missing-id across the .engine instance set (decisions/processes/views + registry + template).
    let mut inst_files: Vec<PathBuf> = Vec::new();
    for sub in ["decisions", "processes", "views"] {
        inst_files.extend(keel_model::corpus::collect_sysml(&root.join(".engine").join(sub)));
    }
    inst_files.push(root.join(".engine").join("skills").join("skills-registry.sysml"));
    inst_files.push(root.join(".engine").join("docs").join("tracking-template.sysml"));
    for path in &inst_files {
        let Ok(text) = keel_model::corpus::read_to_string(path) else { continue };
        let inst = count_tracked_instances(&text);
        let ids = text.matches(":>> id =").count();
        if inst > ids {
            warnings.push(format!("{}: {} tracked instance(s) missing :>> id (§2.3)", relpath(root, path), inst - ids));
        }
    }
    GuardReport { name: "engine-lint", scanned: decision_files.len() + inst_files.len(), warnings, violations }
}

/// Guard (issue118): every `:>> name =` on an ENGINE-typed element names an attribute that type
/// actually declares.
///
/// # Why this is not covered by anything else
///
/// The engine checked MARKERS (D0133) and ENUM MEMBERS, but never attribute NAMES. Proven by probe:
/// a `CodeElement` authored with `codeHsah` and `riskClas` passed `validate`, all 32 guards and the
/// fast edit gate — 329 files reported clean — while silently losing its risk classification, so the
/// element sat in the audit frontier as `correctness` when its author had written `dataLoss`. That is
/// the whole "new schema elements are not being picked up" surprise: the author believes a fact was
/// recorded, every gate agrees the model is honest, and the fact is not there.
///
/// # What it deliberately does NOT judge
///
/// A type the engine schema does not declare is a PROJECT type (D0136/sprint 298), whose attributes
/// the engine cannot know. Those are skipped entirely — judging them would recreate the issue090
/// lockout, where a binary's opinion about a project's own vocabulary blocked every commit.
#[must_use]
pub fn attribute_vocabulary(root: &Path) -> GuardReport {
    let mut files = keel_model::corpus::collect_sysml(&root.join(".tracking"));
    files.extend(keel_model::corpus::collect_sysml(&root.join(".engine")));
    let mut scanned = 0usize;
    let mut violations = Vec::new();
    for path in &files {
        let Ok(text) = keel_model::corpus::read_to_string(path) else { continue };
        let rel = relpath(root, path);
        // The type of the element whose braces we are inside, with the depth it opened at.
        let mut stack: Vec<(String, String, i32)> = Vec::new();
        let mut depth: i32 = 0;
        for (i, raw) in text.lines().enumerate() {
            let line = strip_string_literals(raw);
            let t = line.trim_start();
            if t.starts_with("//") {
                continue;
            }
            // THE ELEMENT DECLARED ON THIS LINE, if any. Records in this repo are overwhelmingly
            // authored on ONE line — `verification x : Test { :>> id = ...; :>> method = ...; }` —
            // so a scanner that only credits `:>>` to a previously-pushed stack frame misses them
            // all. The first cut did exactly that: 8088 of 41036 assignments scanned, and it
            // reported PASS over the 80% it never looked at, which is precisely the silent-coverage
            // failure this guard exists to prevent.
            let mut line_owner: Option<(String, String)> = None;
            if let Some((decl, rest)) = t.split_once(':') {
                let decl_t = decl.trim().trim_start_matches('#');
                let is_usage = ["part", "verification", "occurrence", "requirement", "action", "item", "use case"]
                    .iter()
                    .any(|k| decl_t.starts_with(k) && !decl_t.contains(" def "));
                if is_usage && !rest.starts_with('>') {
                    let name: String = decl_t.split_whitespace().nth(1).unwrap_or("").to_string();
                    let ty: String =
                        rest.trim().chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
                    if !ty.is_empty() && line.contains('{') {
                        line_owner = Some((name.clone(), ty.clone()));
                        // Push only if the block stays OPEN past this line; a single-line record
                        // opens and closes here and must not linger on the stack.
                        if line.matches('{').count() > line.matches('}').count() {
                            stack.push((name, ty, depth));
                        }
                    }
                }
            }
            // EVERY `:>>` on the line, not just a line-leading one.
            for (pos, _) in line.match_indices(":>>") {
                let attr: String = line[pos + 3..]
                    .trim_start()
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                let owner = line_owner.as_ref().map(|(n, ty)| (n, ty)).or_else(|| stack.last().map(|(n, ty, _)| (n, ty)));
                let Some((owner, ty)) = owner else { continue };
                let Some(declared) = keel_schema::schema::declared_attrs_in(root, ty) else { continue };
                scanned += 1;
                if !attr.is_empty() && !declared.contains(&attr) {
                    let hint = nearest_attr(&attr, &declared).map_or_else(
                        || format!("`{ty}` declares: {}", sorted_list(&declared)),
                        |n| format!("did you mean `{n}`?"),
                    );
                    violations.push(format!(
                        "{rel}:{}: `{owner} : {ty}` sets `{attr}`, which `{ty}` does not declare — {hint} An undeclared attribute is accepted silently and the value is simply LOST: the element computes as though it were never authored (issue118).",
                        i + 1
                    ));
                }
            }
            depth += i32::try_from(line.matches('{').count()).unwrap_or(0)
                - i32::try_from(line.matches('}').count()).unwrap_or(0);
            while stack.last().is_some_and(|(_, _, d)| depth <= *d) {
                stack.pop();
            }
        }
    }
    GuardReport { name: "attribute-vocabulary", scanned, warnings: Vec::new(), violations }
}

pub(crate) fn sorted_list(s: &HashSet<String>) -> String {
    let mut v: Vec<&str> = s.iter().map(String::as_str).collect();
    v.sort_unstable();
    v.join(", ")
}

#[cfg(test)]
mod parallel_tests {
    use super::{added_non_v4_ids, is_v4_uuid, non_v4_ids_in, uuid_shaped};
    use keel_model::ident::uuid_hex_shaped;

    /// D0430/issue454: the known positive is an id the write API emitted this session; the known
    /// negatives are the verifier's typed id (shape only), a v5 and a sequence-shaped hex id.
    #[test]
    fn is_v4_uuid_accepts_the_api_shape_and_refuses_shape_alone() {
        // known positive: an id add-task wrote this session; known negatives: the verifier's typed id
        // (shaped, not hex), a v5, a wrong variant nibble, uppercase hex.
        assert!(is_v4_uuid("5a6bb71c-66d8-497b-90c1-0b544f47c758"));
        assert!(uuid_shaped("eh5h6i7g-8f9e-0j1h-2i3d-4e5f6g7h8i9d"));
        assert!(!is_v4_uuid("eh5h6i7g-8f9e-0j1h-2i3d-4e5f6g7h8i9d"));
        assert!(!is_v4_uuid("005f7385-0a6e-5aea-a0a5-68a8287434ee"));
        assert!(!is_v4_uuid("0069b38f-a2cd-44f8-1750-8c6ea975cd34"));
        assert!(!is_v4_uuid("5A6BB71C-66D8-497B-90C1-0B544F47C758"));
        assert!(uuid_hex_shaped("005f7385-0a6e-5aea-a0a5-68a8287434ee"));
        assert!(!uuid_hex_shaped("eh5h6i7g-8f9e-0j1h-2i3d-4e5f6g7h8i9d"));
    }

    #[test]
    fn added_non_v4_ids_reads_only_added_lines_of_the_diff() {
        let diff = concat!(
            "diff --git a/.tracking/x.sysml b/.tracking/x.sysml\n",
            "--- a/.tracking/x.sysml\n",
            "+++ b/.tracking/x.sysml\n",
            "@@ -1,0 +2,3 @@\n",
            "+    part a : Issue { :>> id = \"5a6bb71c-66d8-497b-90c1-0b544f47c758\"; }\n",
            "+    part b : Issue { :>> id = \"eh5h6i7g-8f9e-0j1h-2i3d-4e5f6g7h8i9d\"; }\n",
            "+    // :>> id = \"zzzzzzzz-zzzz-zzzz-zzzz-zzzzzzzzzzzz\" in a comment is not an id\n",
            "-    part c : Issue { :>> id = \"a608di4d-1f56-4c8a-bc91-1f48f5i4h111\"; }\n",
        );
        assert_eq!(added_non_v4_ids(diff), vec![(".tracking/x.sysml".to_string(), "eh5h6i7g-8f9e-0j1h-2i3d-4e5f6g7h8i9d".to_string())]);
        assert!(added_non_v4_ids("").is_empty());
        // an untracked file is scanned whole, comments skipped
        let fresh = "package Q {\n    // :>> id = \"zzzzzzzz-zzzz-zzzz-zzzz-zzzzzzzzzzzz\"\n    part u : Issue { :>> id = \"0069b38f-a2cd-44f8-1750-8c6ea975cd34\"; }\n    part v : Issue { :>> id = \"5a6bb71c-66d8-497b-90c1-0b544f47c758\"; }\n}\n";
        assert_eq!(non_v4_ids_in(".tracking/new.sysml", fresh), vec![(".tracking/new.sysml".to_string(), "0069b38f-a2cd-44f8-1750-8c6ea975cd34".to_string())]);
    }

    /// Every guard that ran is timed, so the receipt's per-guard `ms` is a measured profile (issue455).
    /// The durations are the RUN's, so this holds with other `run_all`s live on other test threads.
    #[test]
    fn run_all_timed_names_every_guard_that_ran() {
        let root = std::path::Path::new("../..");
        if !root.join(".tracking").is_dir() {
            return; // not the self-build tree
        }
        let (reports, timed) = super::run_all_timed(root);
        let ran = reports.iter().filter(|r| !r.warnings.iter().any(|w| w.starts_with("NOT ACTIVE"))).count();
        assert_eq!(timed.len(), ran, "one duration per guard that ran");
        for (name, _) in &timed {
            assert!(super::GUARD_NAMES.contains(name), "{name} is not a guard");
        }
    }

    /// dcGuardsRunInParallelAndTimed: the reports come back in `GUARD_NAMES` order, one per enforced
    /// guard, with every inactive one present as its NOT ACTIVE report - exactly what the serial loop
    /// returned. Run against this repository, whose activation set is the real one; a thread finishing
    /// order must never show through.
    #[test]
    fn run_all_reports_are_in_guard_names_order() {
        let root = std::path::Path::new("../..");
        if !root.join(".tracking").is_dir() {
            return; // not the self-build tree
        }
        let act = keel_model::activation::Activation::load(root);
        let expected: Vec<&str> = super::GUARD_NAMES
            .iter()
            .copied()
            .filter(|n| match act.guard_state(n) {
                keel_model::activation::GuardState::Inactive(_) => true,
                _ => super::run_one(n, root).is_some(),
            })
            .collect();
        let got: Vec<&str> = super::run_all(root).iter().map(|r| r.name).collect();
        assert_eq!(got, expected, "run_all must return reports in GUARD_NAMES order regardless of thread timing");
    }

    /// The same tree judged twice yields the same verdict per guard: a guard's answer is a function of
    /// the tree, not of which thread ran it beside which.
    #[test]
    fn run_all_is_deterministic_across_two_runs() {
        let root = std::path::Path::new("../..");
        if !root.join(".tracking").is_dir() {
            return;
        }
        let a: Vec<(String, usize, usize)> = super::run_all(root).iter().map(|r| (r.name.to_string(), r.warnings.len(), r.violations.len())).collect();
        let b: Vec<(String, usize, usize)> = super::run_all(root).iter().map(|r| (r.name.to_string(), r.warnings.len(), r.violations.len())).collect();
        assert_eq!(a, b);
    }
}

#[cfg(test)]
mod attribute_vocabulary_tests {
    use super::*;

    fn probe(body: &str) -> GuardReport {
        let dir = keel_fs::scratch(&format!("keel-attrvocab-{}", body.len()));
        let tracking = dir.join(".tracking");
        std::fs::create_dir_all(&tracking).expect("scratch dir");
        std::fs::write(tracking.join("probe.sysml"), body).expect("write probe");
        let r = attribute_vocabulary(&dir);
        let _ = std::fs::remove_dir_all(&dir);
        r
    }

    #[test]
    fn a_misspelled_attribute_is_caught_with_the_nearest_name() {
        let r = probe("part p : Claim {\n    :>> claimedItm = \"x\";\n}\n");
        assert_eq!(r.violations.len(), 1, "{:?}", r.violations);
        assert!(r.violations[0].contains("did you mean `claimedItem`?"), "{}", r.violations[0]);
    }

    #[test]
    fn correctly_spelled_attributes_pass_including_inherited_ones() {
        let r = probe("part p : Claim {\n    :>> id = \"x\";\n    :>> claimedItem = \"y\";\n}\n");
        assert!(r.violations.is_empty(), "id is inherited from Element: {:?}", r.violations);
    }

    #[test]
    fn single_line_records_are_scanned() {
        // THE REGRESSION THAT MATTERS. The first cut only credited `:>>` to a stack frame pushed on
        // an EARLIER line, so single-line records — the dominant form in this repo — were invisible:
        // 8088 of 41036 assignments scanned, reporting PASS over the 80% it never read.
        let r = probe("part p : Claim { :>> id = \"x\"; :>> claimedItm = \"y\"; }\n");
        assert_eq!(r.scanned, 2, "both assignments on the one line must be scanned");
        assert_eq!(r.violations.len(), 1, "{:?}", r.violations);
    }

    #[test]
    fn a_project_declared_type_is_never_judged() {
        // Judging a vocabulary the engine cannot know is how issue090 blocked every commit.
        let r = probe("part p : SomeProjectType {\n    :>> whateverTheyLike = \"x\";\n}\n");
        assert!(r.violations.is_empty(), "{:?}", r.violations);
        assert_eq!(r.scanned, 0, "an unknown type is skipped, not scanned");
    }

    #[test]
    fn prose_naming_an_attribute_is_not_an_assignment() {
        let r = probe("// :>> notReal = \"in a comment\";\npart p : Claim { :>> id = \"x\"; }\n");
        assert!(r.violations.is_empty(), "{:?}", r.violations);
    }
}

#[cfg(test)]
mod accept_transform_tests {
    use super::is_accept_transform;

    /// D0205: the sanctioned non-owner edit is EXACTLY status proposed->accepted; anything else —
    /// the reverse flip, a rider attribute, a different field — still violates ownership.
    #[test]
    fn only_the_forward_status_flip_is_sanctioned() {
        let old = vec!["status=proposed".to_string(), "title=x".to_string()];
        let fwd = vec!["status=accepted".to_string(), "title=x".to_string()];
        assert!(is_accept_transform(&old, &fwd));
        let rev_old = vec!["status=accepted".to_string(), "title=x".to_string()];
        let rev_new = vec!["status=proposed".to_string(), "title=x".to_string()];
        assert!(!is_accept_transform(&rev_old, &rev_new), "reverse flip is not sanctioned");
        let rider = vec!["status=accepted".to_string(), "title=CHANGED".to_string()];
        assert!(!is_accept_transform(&old, &rider), "a rider edit is not sanctioned");
        assert!(!is_accept_transform(&old, &old), "no change is not a transform");
    }
}
