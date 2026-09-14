//! Pure scanners over `.sysml` TEXT - no model, no git, no filesystem.
//!
//! Shared by guards, orient and the views, so it sits below all three (D0479, dcGuardsViewOrientCycleIsBroken):
//! the marker scan behind `marker-vocabulary`, the retro-text and item-name extraction behind `retro-backlog`
//! and the priority view's retro citations, and the ceremony predicates (`gate_passed` and its two
//! siblings) that orient, the flow view and the ceremony guard all read.

use std::collections::HashSet;

/// Remove `SysML` string literals from a line, so markers QUOTED IN PROSE are not mistaken for edges.
///
/// Essential, not cosmetic: `procedureText` fields legitimately discuss markers (`#Marker dependency
/// from a to b`, `#Kind dependency`, `#Changes dependency`), and a naive scan reports each as an
/// undeclared marker. Those three alone would have produced 9 false violations on a hard guard.
pub(crate) fn strip_string_literals(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_str = false;
    for c in line.chars() {
        if c == '"' {
            in_str = !in_str;
            continue;
        }
        if !in_str {
            out.push(c);
        }
    }
    out
}

/// Syntactic positions in which a `#Marker` names a real edge or a marked item.
const MARKER_FOLLOWERS: [&str; 5] = ["dependency", "part", "item", "verification", "requirement"];

/// The ENGINE's own marker algebra — always valid, known to the binary (D0136 / issue089).
///
/// These are the markers the engine's OWN guards and views consume: `#Verify` drives
/// tier-satisfaction and verification-trace, `#DerivedFrom` drives the hard requirement-rootedness
/// guard, `#Resolves` drives issue triage, and so on. They are part of the engine's CONTRACT, so the
/// binary must know them intrinsically rather than requiring each project to re-declare them.
///
/// Why this exists: D0133 shipped `marker-vocabulary` as a HARD guard in the BINARY whose passing
/// condition was `metadata def` lines in the PROJECT's schema files. `include_dir!` embeds `.engine`
/// at build time, so a v0.2.0 binary carried the declarations — but an existing downstream project
/// keeps its own on-disk `.engine/schema/`, which upgrading the binary never touches. Reproduced: a
/// pre-v0.2.0 schema plus a v0.2.0 binary yields **566 violations and every commit blocked**, on the
/// engine's own shipped files. Worse, the obvious remedy meant editing FROZEN `schema/core`, so the
/// guard forced every downstream project into a frozen-schema sign-off just to keep committing.
/// The engine's own marker vocabulary, DERIVED from the `metadata def`s in the schema baked into
/// this binary — never restated as a literal list.
///
/// It was a hardcoded 17-entry list and had already fallen behind: `Controls`, `Feedback` and
/// `Restructure` shipped with the codeaudit module and were never added (issue120). Deriving keeps
/// the property this list exists for — the vocabulary travels WITH the binary, so upgrading the
/// binary against an older on-disk `.engine/` cannot produce the issue090 lockout — while removing
/// the second place that had to be remembered.
#[must_use]
pub fn engine_markers() -> &'static HashSet<String> {
    static M: std::sync::LazyLock<HashSet<String>> =
        std::sync::LazyLock::new(|| crate::schema::VOCAB.markers.clone());
    &M
}

/// Marker names USED in real syntactic positions in `text`, as `(marker, 1-based line)`.
pub(crate) fn markers_used(text: &str) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line = strip_string_literals(raw);
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue; // a comment may legitimately name a marker
        }
        for (pos, _) in line.match_indices('#') {
            let rest = &line[pos + 1..];
            let name: String = rest.chars().take_while(|c| c.is_alphanumeric()).collect();
            if name.is_empty() {
                continue;
            }
            let after = rest[name.len()..].trim_start();
            if MARKER_FOLLOWERS.iter().any(|f| after.starts_with(f)) {
                out.push((name, i + 1));
            }
        }
    }
    out
}

/// Marker names DECLARED as `metadata def <Name>;` in `texts`.
pub(crate) fn markers_declared(texts: &[String]) -> HashSet<String> {
    // The engine's own algebra is always valid — a project must never have to re-declare it (D0136).
    let mut out: HashSet<String> = engine_markers().clone();
    for text in texts {
        for raw in text.lines() {
            let line = raw.trim();
            if line.starts_with("//") {
                continue;
            }
            if let Some(rest) = line.strip_prefix("metadata def ") {
                let name: String = rest.trim().chars().take_while(|c| c.is_alphanumeric()).collect();
                if !name.is_empty() {
                    out.insert(name);
                }
            }
        }
    }
    out
}

/// Phrases by which a retro EXPLICITLY justifies raising no tracked item.
///
/// The obligation is not "always create an item" — sometimes a control already exists, and adding a
/// duplicate is noise. The obligation is that the choice is STATED rather than left silent, so a
/// reader can tell a considered decision from an omission.
pub(crate) const RETRO_NO_ITEM_JUSTIFICATIONS: &[&str] = &["no new item", "no item needed", "already tracked", "no further item"];

/// Every tracked-item NAME this retro's own text mentions — `dcCamelCase` and `issueNNN` tokens.
///
/// The RETRO's text, not the whole sprint file: a sprint file legitimately names the task it
/// delivered in its `DoD` line, and that name satisfied the old check for every retro ever written.
pub(crate) fn named_items(text: &str) -> Vec<String> {
    /// `dc` is followed by an uppercase letter; `issue` by a digit.
    type NextOk = fn(char) -> bool;
    let bytes = text.as_bytes();
    let boundary =
        |i: usize| i.checked_sub(1).and_then(|j| bytes.get(j)).is_none_or(|b| !b.is_ascii_alphanumeric());
    let mut out = Vec::new();
    // A Decision (`d0289`) tracks a finding as legitimately as a task or an Issue does - a "won't do"
    // IS a Decision (Invariant 4) - so a retro may name one to justify raising nothing else. The
    // Decision is matched in BOTH cases - `d0388` as the file names it, `D0388` as CLAUDE.md, every
    // commit message, every DoD and this guard's own refusal text write it - and returned as the
    // item's real name (`d0388`) so the exists check still hits the tree (issue424: sprint 627's
    // 'already tracked by D0388' was refused as naming no item at all). Only the Decision form is
    // case-folded: `Issue` and `DC` in prose are words, not items, and stay unmatched.
    let pairs: [(&str, NextOk); 4] =
        [("dc", |c| c.is_ascii_uppercase()), ("issue", |c| c.is_ascii_digit()), ("d0", |c| c.is_ascii_digit()), ("D0", |c| c.is_ascii_digit())];
    for (needle, ok_next) in pairs {
        let mut from = 0;
        while let Some(rel) = text[from..].find(needle) {
            let st = from + rel;
            let rest = &text[st + needle.len()..];
            if boundary(st) && rest.starts_with(ok_next) {
                let mut name: String = text[st..].chars().take_while(char::is_ascii_alphanumeric).collect();
                if needle == "D0" {
                    name.replace_range(..1, "d");
                }
                out.push(name);
            }
            from = st + needle.len();
        }
    }
    out
}

/// The `procedureText` of every RETRO gate in a sprint file — the `method = analyze` verifications
/// whose title says retro. Returns the texts; a file with no retro gate yields none.
///
/// Blocks start at a LINE that begins with `verification ` — never at the word inside a string. The
/// first version split the whole file on the word, so a retro whose text mentioned "verification"
/// (the commonest word in this repository) was cut in half and silently not examined: the guard passed
/// a retro it had not read. Found by this guard's own arming test on 2026-09-03 (issue364, second shape).
pub(crate) fn retro_texts(sprint_file: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut blocks: Vec<String> = Vec::new();
    for line in sprint_file.lines() {
        if line.trim_start().starts_with("verification ") {
            blocks.push(String::new());
        }
        if let Some(b) = blocks.last_mut() {
            b.push_str(line);
            b.push('\n');
        }
    }
    for block in blocks {
        let head = block.split('{').next().unwrap_or("");
        let is_retro = block.contains("VerificationMethod::analyze") && head.to_ascii_lowercase().contains("retro");
        if !is_retro {
            continue;
        }
        if let Some(i) = block.find("procedureText = \"") {
            let rest = &block[i + 17..];
            if let Some(j) = rest.find("\";") {
                out.push(rest[..j].to_string());
            }
        }
    }
    out
}

/// True if a `part <...><Gate>Gate<...>R<n> : TestResult` with `outcome = pass`
/// exists for the given canonical gate name in `text`.
pub(crate) fn gate_passed(text: &str, gate: &str) -> bool {
    gate_outcome_in(text, gate, &["VerdictKind::pass"])
}

/// True if `<gate>Gate` has a RECORDED `TestResult` - `pass` OR `proposed` (D0312 B). This is the
/// ceremony guard's ORDER reader (D0437): a gate an AI judged without a replayable receipt has been
/// recorded in sequence even though it is not yet passed. Sequence is the guard's concern; done-ness
/// stays [`gate_passed`]'s, so `advance`, orient and the suspect algebra still read a proposed gate as
/// not passed.
pub(crate) fn gate_recorded(text: &str, gate: &str) -> bool {
    gate_outcome_in(text, gate, &["VerdictKind::pass", "VerdictKind::proposed"])
}

/// True if `<gate>Gate` has ANY written `TestResult` - every `VerdictKind` member, `fail` included
/// (issue544, completing D0437's clause). This is the ceremony guard's ORDER reader: sequence asks
/// whether the record was written in its turn, not what it says, and a gate honestly recorded `fail`
/// (sprint708's closeOut, CI red on issue542) was written in its turn. [`gate_recorded`] stays the
/// flow view's finish reader, which refuses `fail` on purpose; done-ness stays [`gate_passed`]'s.
/// The five members are listed, not `pass` negated, so a sixth verdict is a visible edit here.
pub(crate) fn gate_has_result(text: &str, gate: &str) -> bool {
    gate_outcome_in(
        text,
        gate,
        &["VerdictKind::pass", "VerdictKind::proposed", "VerdictKind::fail", "VerdictKind::inconclusive", "VerdictKind::error"],
    )
}

/// A `part ...<gate>Gate...R<n> : TestResult { ... outcome = <one of `outcomes`> }` declaration exists.
fn gate_outcome_in(text: &str, gate: &str, outcomes: &[&str]) -> bool {
    let needle = format!("{gate}Gate");
    for (idx, _) in text.match_indices(&needle) {
        let line_start = text[..idx].rfind('\n').map_or(0, |n| n + 1);
        let stmt_end = text[idx..].find('}').map_or(text.len(), |e| idx + e);
        let stmt = &text[line_start..stmt_end];
        if stmt.contains("part ") && stmt.contains(": TestResult")
            && outcomes.iter().any(|o| stmt.contains(o))
        {
            return true;
        }
    }
    false
}
