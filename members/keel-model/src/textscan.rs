//! Pure scanners over `.sysml` TEXT - no model, no git; the one file read is [`fork_options`], which opens
//! the Decision it is asked about (fork detection descended from the console's deck in sprint 740).
//!
//! Shared by guards, orient and the views, so it sits below all three (D0479, dcGuardsViewOrientCycleIsBroken):
//! the marker scan behind `marker-vocabulary`, the retro-text and item-name extraction behind `retro-backlog`
//! and the priority view's retro citations, and the ceremony predicates (`gate_passed` and its two
//! siblings) that orient, the flow view and the ceremony guard all read.

use std::collections::HashSet;
use std::path::Path;

/// Remove `SysML` string literals from a line, so markers QUOTED IN PROSE are not mistaken for edges.
///
/// Essential, not cosmetic: `procedureText` fields legitimately discuss markers (`#Marker dependency
/// from a to b`, `#Kind dependency`, `#Changes dependency`), and a naive scan reports each as an
/// undeclared marker. Those three alone would have produced 9 false violations on a hard guard.
#[must_use]
pub fn strip_string_literals(line: &str) -> String {
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
        std::sync::LazyLock::new(|| keel_schema::schema::VOCAB.markers.clone());
    &M
}

/// Marker names USED in real syntactic positions in `text`, as `(marker, 1-based line)`.
#[must_use]
pub fn markers_used(text: &str) -> Vec<(String, usize)> {
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
#[must_use]
pub fn markers_declared(texts: &[String]) -> HashSet<String> {
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
pub const RETRO_NO_ITEM_JUSTIFICATIONS: &[&str] = &["no new item", "no item needed", "already tracked", "no further item"];

/// Every tracked-item NAME this retro's own text mentions — `dcCamelCase` and `issueNNN` tokens.
///
/// The RETRO's text, not the whole sprint file: a sprint file legitimately names the task it
/// delivered in its `DoD` line, and that name satisfied the old check for every retro ever written.
pub fn named_items(text: &str) -> Vec<String> {
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
#[must_use]
pub fn retro_texts(sprint_file: &str) -> Vec<String> {
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
#[must_use]
pub fn gate_passed(text: &str, gate: &str) -> bool {
    gate_outcome_in(text, gate, &["VerdictKind::pass"])
}

/// True if `<gate>Gate` has a RECORDED `TestResult` - `pass` OR `proposed` (D0312 B). This is the
/// ceremony guard's ORDER reader (D0437): a gate an AI judged without a replayable receipt has been
/// recorded in sequence even though it is not yet passed. Sequence is the guard's concern; done-ness
/// stays [`gate_passed`]'s, so `advance`, orient and the suspect algebra still read a proposed gate as
/// not passed.
#[must_use]
pub fn gate_recorded(text: &str, gate: &str) -> bool {
    gate_outcome_in(text, gate, &["VerdictKind::pass", "VerdictKind::proposed"])
}

/// True if `<gate>Gate` has ANY written `TestResult` - every `VerdictKind` member, `fail` included
/// (issue544, completing D0437's clause). This is the ceremony guard's ORDER reader: sequence asks
/// whether the record was written in its turn, not what it says, and a gate honestly recorded `fail`
/// (sprint708's closeOut, CI red on issue542) was written in its turn. [`gate_recorded`] stays the
/// flow view's finish reader, which refuses `fail` on purpose; done-ness stays [`gate_passed`]'s.
/// The five members are listed, not `pass` negated, so a sixth verdict is a visible edit here.
#[must_use]
pub fn gate_has_result(text: &str, gate: &str) -> bool {
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

/// The marker vocabulary as WORDS in prose (issue460 / D0337).
///
/// `process-change`, `safety-change`, and the two edge names the markers emit. Matched whole-word and
/// case-insensitively, so `#ProspectiveChange`, `Process-change (D0337)` and `safety-change` all read;
/// `processes changed` does not.
pub const MARKER_WORDS: [&str; 4] = ["process-change", "safety-change", "prospectivechange", "safetychange"];

/// The author's stated way out, on the `NOT A FORK` pattern.
///
/// A Decision that mentions the marker vocabulary in passing - describing another Decision's marker, a
/// guard's name - and changes no process says so in these words, and the record carries the assertion.
pub const NOT_A_PROCESS_CHANGE: &str = "NOT A PROCESS CHANGE";

/// Which marker words a Decision's prose fields carry, by name - distinct, in vocabulary order.
///
/// One classifier for two readers (issue460): the write path holds an unmarked draft whose text names
/// the vocabulary, and guard `consent-scope` reads the recorded file with the same function, so a
/// hand-edited Decision cannot pass a test the write path would have failed.
#[must_use]
pub fn marker_words(fields: &[&str]) -> Vec<&'static str> {
    let lower = format!(" {} ", fields.join(" ").to_lowercase());
    MARKER_WORDS
        .iter()
        .copied()
        .filter(|w| {
            lower.match_indices(w).any(|(i, _)| {
                let before = lower[..i].chars().last().is_none_or(|c| !c.is_alphanumeric());
                let after = lower[i + w.len()..].chars().next().is_none_or(|c| !c.is_alphanumeric());
                before && after
            })
        })
        .collect()
}

/// Does this Decision's text say in words what its draft did not say with a marker - and not declare
/// otherwise?
///
/// `Some(words)` when the fields name the vocabulary, no marker was given and no field carries
/// [`NOT_A_PROCESS_CHANGE`]. On 2026-09-10 D0432's consequences read 'Process-change (D0337)' while its
/// draft had no `marker:` line, and it AUTO-ACCEPTED under standing consent - the text declared itself
/// outside the consent and nothing read the text (issue460).
#[must_use]
pub fn marker_text_without_marker(fields: &[&str], has_marker: bool) -> Option<Vec<&'static str>> {
    if has_marker || fields.iter().any(|f| f.contains(NOT_A_PROCESS_CHANGE)) {
        return None;
    }
    let w = marker_words(fields);
    (!w.is_empty()).then_some(w)
}

/// Extract `OPTION X (short label)` enumerations from a Decision file's text. Two or more make the
/// decision a FORK; fewer yield an empty list and the ordinary single Sign button.
pub fn fork_options(root: &Path, rel_file: &str) -> Vec<(String, String)> {
    let Ok(text) = std::fs::read_to_string(root.join(rel_file)) else { return Vec::new() };
    let mut out = Vec::new();
    let mut rest = text.as_str();
    while let Some(pos) = rest.find("OPTION ") {
        rest = &rest[pos + 7..];
        let Some(tok) = rest.chars().next().filter(char::is_ascii_uppercase) else { continue };
        let Some(open) = rest.find('(') else { continue };
        if rest[1..open].trim().is_empty() {
            if let Some(close) = rest.find(')') {
                let label = rest[open + 1..close].trim().to_string();
                if !label.is_empty() && !out.iter().any(|(t, _)| *t == tok.to_string()) {
                    out.push((tok.to_string(), label));
                }
            }
        }
    }
    if out.len() >= 2 { out } else { Vec::new() }
}

/// The words that mark a Decision as WEIGHING alternatives rather than stating one course (D0322,
/// issue373). Each is a signal; two distinct signals in the `decision` field is the disguised-fork shape.
const FORK_SIGNALS: [(&str, &[&str]); 8] = [
    ("alternative", &["alternative", "alternatives"]),
    ("either", &["either"]),
    ("or-we-could", &["or we could", "we could instead", "could instead"]),
    ("option", &["option", "options"]),
    ("versus", &["versus", " vs ", " vs. "]),
    ("trade-off", &["trade-off", "tradeoff", "trade-offs", "tradeoffs"]),
    ("enumeration", &["(a)", "(b)", " a: ", " b: ", " a) ", " b) "]),
    ("recommend", &["recommend", "recommends", "recommended", "recommendation"]),
];

/// The author's stated way out: a Decision that weighs alternatives in passing and IS a decision says
/// so, in these words, and the record carries the assertion.
pub const NOT_A_FORK: &str = "NOT A FORK";

/// Which fork signals a Decision's `decision` text carries, by name - the disguised-fork detector
/// (D0322 / issue373, found by stpa-self run 1 as UCA-R1).
///
/// `fork_options` reads ONE lexical shape, `OPTION X (label)`; a Decision that weighs alternatives in
/// prose without it auto-accepted under standing consent, and the author controlled the lexicon that
/// decided whether the human was asked. This reads the words that weigh: two distinct signals hold the
/// Decision proposed. Measured before the threshold was set (the D0102 way): over 109 auto-accepted
/// Decisions, four carry two signals - three describe the surfacing process itself (D0207, D0269,
/// D0288, false positives an author answers with `NOT A FORK`) and one (D0292) recommended a routing
/// and executed it, which is the class this exists to catch. No auto-accepted Decision carries three.
#[must_use]
pub fn fork_signals(decision_text: &str) -> Vec<&'static str> {
    let lower = format!(" {} ", decision_text.to_lowercase());
    let word = |w: &str| -> bool {
        // whole-word for alphabetic tokens; the punctuation-bearing ones match as written
        if w.chars().all(|c| c.is_ascii_alphabetic() || c == '-') {
            lower.match_indices(w).any(|(i, _)| {
                let before = lower[..i].chars().last().is_none_or(|c| !c.is_alphanumeric());
                let after = lower[i + w.len()..].chars().next().is_none_or(|c| !c.is_alphanumeric());
                before && after
            })
        } else {
            lower.contains(w)
        }
    };
    FORK_SIGNALS.iter().filter(|(_, words)| words.iter().any(|w| word(w))).map(|(name, _)| *name).collect()
}

/// Is this Decision a fork in substance without the marker - and not declared otherwise?
#[must_use]
pub fn disguised_fork(decision_text: &str) -> Option<Vec<&'static str>> {
    if decision_text.contains(NOT_A_FORK) {
        return None;
    }
    let s = fork_signals(decision_text);
    (s.len() >= 2).then_some(s)
}

#[cfg(test)]
mod fork_shape_tests {
    use super::{disguised_fork, fork_signals};

    /// The D0292 shape: a routing RECOMMENDED with per-item OPTIONS, then executed under consent - held.
    #[test]
    fn a_recommendation_naming_options_is_a_disguised_fork() {
        let d = "Route as recommended. Eleven Issues with resolver tasks; a deactivated process's skill is deployed stating its state, option B.";
        assert_eq!(disguised_fork(d), Some(vec!["option", "recommend"]));
    }

    /// A genuine decision that mentions the rejected alternative in passing carries ONE signal: it
    /// auto-accepts as before. The author's `NOT A FORK` also clears a two-signal text.
    #[test]
    fn one_signal_in_passing_is_a_decision_and_the_author_may_say_so() {
        assert_eq!(fork_signals("Adopt the merge; the alternative of a never-overwrite list would freeze the engine's sections."), vec!["alternative"]);
        assert!(disguised_fork("Adopt the merge; the alternative would freeze the engine's sections.").is_none());
        assert!(disguised_fork("The surfacing page carries the options and my recommendation. NOT A FORK: this defines the page, it chooses nothing.").is_none());
        assert!(fork_signals("optional fields and a recommender system").is_empty(), "whole words only: `optional`, `recommender` are not the signals");
    }

    /// Enumerated courses with a verdict phrase read as weighing.
    #[test]
    fn enumerated_courses_are_a_signal() {
        assert_eq!(disguised_fork("Two ways: (a) refuse at record time; (b) warn at the gate. Either works; we take (a)."), Some(vec!["either", "enumeration"]));
    }
}

#[cfg(test)]
mod marker_text_tests {
    use super::{marker_text_without_marker, marker_words};

    /// The issue460 shape: the consequences say `Process-change (D0337)`, the draft has no marker - held.
    #[test]
    fn prose_naming_the_vocabulary_without_a_marker_is_a_mismatch() {
        let c = "CLAUDE.md names the run. Process-change (D0337): proposed until the human's word.";
        assert_eq!(marker_text_without_marker(&["ctx", "dec", "why", c], false), Some(vec!["process-change"]));
        assert_eq!(marker_words(&["each lands with its own #ProspectiveChange Decision", "a SafetyChange edge"]), vec!["prospectivechange", "safetychange"]);
    }

    /// The same text WITH the marker is the D0337 path's business, not this one's; a text naming neither
    /// is a plain Decision; and the author's `NOT A PROCESS CHANGE` clears a mention in passing.
    #[test]
    fn a_marked_draft_a_plain_text_and_a_declared_mention_are_not_mismatches() {
        let c = "Process-change (D0337): proposed until the human's word.";
        assert!(marker_text_without_marker(&[c], true).is_none());
        assert!(marker_text_without_marker(&["Adopt the merge; the processes changed nothing here."], false).is_none());
        assert!(marker_text_without_marker(&["Rank 1 lands with its own #ProspectiveChange Decision. NOT A PROCESS CHANGE: this Decision ranks, it changes no process."], false).is_none());
        assert!(marker_words(&["safety-changes", "reprocess-change"]).is_empty(), "whole words only");
    }
}

#[cfg(test)]
mod fork_options_tests {
    use super::fork_options;

    /// issue223: a Decision enumerating `OPTION X (label)` choices is a FORK — the deck must offer
    /// one sign button per option, because a bare Sign tap on a fork cannot bind (D0192's tap was
    /// solicited and wasted). Pinned to D0192's actual text shape.
    #[test]
    fn fork_options_parse_the_d0192_shape() {
        let dir = keel_fs::scratch("keel-deck-forkcheck");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(
            dir.join("d.sysml"),
            ":>> decision = \"PROPOSED, two options costed - accepting this Decision means choosing ONE: \
             OPTION A (amend the requirement): supersede srK06 ... OPTION B (close the path): acceptances \
             become human-gesture-only ...\";",
        )
        .expect("write");
        let opts = fork_options(&dir, "d.sysml");
        assert_eq!(
            opts,
            vec![
                ("A".to_string(), "amend the requirement".to_string()),
                ("B".to_string(), "close the path".to_string())
            ]
        );
        // A single option is NOT a fork: the ordinary Sign button stays.
        std::fs::write(dir.join("one.sysml"), "OPTION A (only one)").expect("write");
        assert!(fork_options(&dir, "one.sysml").is_empty());
        // No options at all.
        std::fs::write(dir.join("none.sysml"), "an ordinary decision text").expect("write");
        assert!(fork_options(&dir, "none.sysml").is_empty());
    }
}
