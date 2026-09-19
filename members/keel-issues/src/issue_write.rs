//! `keel record issue` (D0480, sprint 737): the triaged Issue and its `#Resolves` edge, sliced whole out of
//! keel-write's write.rs by `scripts/extract_issues.py`, plus `triage_holds` - the severity and
//! resolver-kind checks that sat inline in main.rs's `cmd_record_issue` (issue558) and are the item
//! process's own. keel-cli's `write` wrapper module re-exports `NewIssue` and `record_issue`, so
//! `crate::write::record_issue` still resolves.

use std::path::Path;

use keel_model::ident::gen_uuid;
use keel_model::model::Model;
use keel_model::resolvers::{declared_task_names, resolver_kind_holds};
use keel_write::write::{per_actor_file, reject_injected_output, sanitize_field, with_file_lock, write_atomic, WriteError};

/// A new `Issue`, with the triage that makes it well-formed on arrival.
pub struct NewIssue<'a> {
    pub title: &'a str,
    pub description: &'a str,
    /// `Critical` | `High` | `Medium` | `Low`.
    pub severity: &'a str,
    /// An EXISTING item that resolves it — the `#Resolves` edge is authored with the Issue.
    pub resolver: &'a str,
    pub related_task: Option<&'a str>,
    pub date: &'a str,
    pub author: &'a str,
    /// Optional engine marker prefix, e.g. `ProcessDefect`.
    pub marker: Option<&'a str>,
    /// Discovered in production use rather than by an internal check.
    pub in_field: bool,
}

/// Next free `issueNNN` name in the issues file.
fn next_issue_number(text: &str) -> u32 {
    let mut max = 0u32;
    let mut from = 0usize;
    while let Some(hit) = text[from..].find("part issue") {
        let at = from + hit + "part issue".len();
        let digits: String = text[at..].chars().take_while(char::is_ascii_digit).collect();
        if let Ok(n) = digits.parse::<u32>() {
            max = max.max(n);
        }
        from = at;
    }
    max + 1
}

/// `keel record issue` — author a TRIAGED Issue plus its `#Resolves` edge, in one call.
///
/// D0108 clause 5 MANDATES that conflicting conclusions across contributors be recorded as an Issue
/// for human adjudication, and there was no command that records an Issue at all — `record decision`
/// existed, `record issue` did not. A sanctioned path with no implementation is the friction that
/// guarantees non-compliance (D0054): the rule was reachable only by hand-editing a 1300-line file.
///
/// The `#Resolves` edge is written WITH the Issue rather than left for later, because the `issues`
/// guard fails on an untriaged Issue — so a command that produced one would hand the caller a red
/// gate as its output. Triage-on-arrival is what makes "green in one call" true.
///
/// # Errors
/// `WriteError::Io` on filesystem errors; `WriteError::TaskNotFound` if the issues file has no
/// package close to insert before.
pub fn record_issue(root: &Path, n: &NewIssue) -> Result<(String, String), WriteError> {
    // issue255: same refusal as record_decision — an Issue is a governance record too.
    reject_injected_output(&[("title", n.title), ("description", n.description)])?;
    // issue185 + issue210: the legacy issues.sysml serves as the ALLOCATION mutex (it always exists)
    // even though the new record lands in the author's per-actor file - two same-machine writers
    // must not race the number scan.
    with_file_lock(&root.join(".tracking").join("issues.sysml"), || record_issue_locked(root, n))
}

fn record_issue_locked(root: &Path, n: &NewIssue) -> Result<(String, String), WriteError> {
    // issue210: the number allocates over ALL of .tracking (per-actor files included), the record
    // lands in the AUTHOR's file. A `part issueNNNDispM` declaration also drives the max, which can
    // only skip numbers ahead, never collide - and the duplicate-identity guard backstops collisions
    // from offline clones: its class 5 reads an allocated name across every package, because the
    // per-actor files are separate packages and class 2 alone landed one such merge green (issue624).
    let mut all_text = String::new();
    for f in keel_model::corpus::collect_sysml(&root.join(".tracking")) {
        if let Ok(s) = std::fs::read_to_string(&f) {
            all_text.push_str(&s);
        }
    }
    let path = per_actor_file(root, "issues", n.author)?;
    let text = std::fs::read_to_string(&path)?;
    let num = next_issue_number(&all_text);
    let name = format!("issue{num:03}");
    let uuid = gen_uuid();
    let s = sanitize_field;
    let marker = n.marker.map_or_else(String::new, |m| format!("#{m} "));
    let related = n
        .related_task
        .map_or_else(String::new, |t| format!("        :>> relatedTask = \"{}\";\n", s(t)));
    let block = format!(
        "\n    {marker}part {name} : Issue {{\n\
         \x20       :>> id = \"{uuid}\";\n\
         \x20       :>> title = \"{title}\";\n\
         \x20       :>> createdAt = \"{date}\";\n\
         \x20       :>> createdBy = \"{author}\";\n\
         \x20       :>> description = \"{desc}\";\n\
         \x20       :>> discoveredInField = {in_field};\n\
         {related}\
         \x20       :>> severity = Severity::{sev};\n\
         \x20   }}\n\
         \x20   #Resolves dependency from {resolver} to {name};\n",
        title = s(n.title),
        date = s(n.date),
        author = s(n.author),
        desc = s(n.description),
        in_field = n.in_field,
        sev = s(n.severity),
        resolver = s(n.resolver),
    );
    let close = text
        .rfind('}')
        .ok_or_else(|| WriteError::TaskNotFound("issues file (no package close)".to_owned()))?;
    let mut out = String::with_capacity(text.len() + block.len());
    out.push_str(text[..close].trim_end());
    out.push('\n');
    out.push_str(&block);
    out.push_str(&text[close..]);
    write_atomic(&path, out)?;
    Ok((name, format!(".tracking/issues-{}.sysml", n.author)))
}

#[cfg(test)]
mod issue_tests {
    use super::next_issue_number;

    #[test]
    fn issue_numbering_takes_the_max_not_the_count() {
        // Counting would collide the moment an issue is ever removed or numbered out of order, and a
        // duplicate id is its own corruption class (issue074). Max+1 is stable under both.
        assert_eq!(next_issue_number("part issue001 : Issue"), 2);
        assert_eq!(next_issue_number("part issue001\npart issue007\npart issue003"), 8);
        assert_eq!(next_issue_number("no issues here"), 1);
        // A mention inside prose must not drive the counter — the self-referential-corpus trap that
        // inflated the marker census in issue099. `part issue` is the declaration form.
        assert_eq!(next_issue_number("description = \"see issue900 for context\""), 1);
    }
}

/// Why `record issue` refuses a triage before anything is written (issue558, D0077).
#[derive(Debug)]
pub enum TriageRefusal {
    /// `--severity` outside `Critical | High | Medium | Low`.
    Severity(String),
    /// The resolver is declared nowhere in the model.
    UnknownResolver(String),
    /// The resolver exists but is neither a declared action nor a `Decision`; carries the kind it is.
    ResolverKind { resolver: String, kind: String },
    /// The model could not be read to check the resolver.
    Model(String),
}

impl std::fmt::Display for TriageRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Severity(s) => write!(f, "--severity must be Critical | High | Medium | Low (got '{s}')"),
            Self::UnknownResolver(r) => write!(
                f,
                "resolver '{r}' is declared nowhere in the model.\n  Authoring the edge anyway would make this Issue read as TRIAGED by something that does not\n  exist (issue109). Declare the resolving action or Decision first, then re-run."
            ),
            Self::ResolverKind { resolver, kind } => write!(
                f,
                "resolver '{resolver}' is a {kind}, not a declared action or a Decision.\n  A resolver is the work that closes the issue or the Decision that moots it; the\n  `resolver-kind` guard refuses any other #Resolves source at commit (issue136/issue558),\n  so writing the edge here would only move that refusal to the gate. Name the action\n  (a backlog item or sprint task) or the Decision, then re-run."
            ),
            Self::Model(e) => write!(f, "cannot read the model to check the resolver: {e}"),
        }
    }
}

/// The item process's own checks on a triage, in the order the command applied them: the severity is one
/// of the four, and the resolver is what the `resolver-kind` guard accepts, read through the guard's own
/// predicate (`keel_model::resolvers::resolver_kind_holds`) so the write refuses exactly what the commit
/// gate would (issue558): the first triage of issue556 pointed at a Story, the write printed "triaged on
/// arrival", and the pre-commit guard was the first thing to say otherwise.
///
/// # Errors
/// [`TriageRefusal`] naming what failed; nothing is written.
pub fn triage_holds(root: &Path, severity: &str, resolver: &str) -> Result<(), TriageRefusal> {
    if !["Critical", "High", "Medium", "Low"].contains(&severity) {
        return Err(TriageRefusal::Severity(severity.to_string()));
    }
    let actions = declared_task_names(root);
    if actions.contains(resolver) {
        return Ok(());
    }
    let model = Model::build(root).map_err(|e| TriageRefusal::Model(e.to_string()))?;
    match model.items.get(resolver) {
        None => Err(TriageRefusal::UnknownResolver(resolver.to_string())),
        Some(item) if !resolver_kind_holds(&actions, resolver, &item.type_name) => {
            Err(TriageRefusal::ResolverKind { resolver: resolver.to_string(), kind: item.type_name.clone() })
        }
        Some(_) => Ok(()),
    }
}

#[cfg(test)]
mod triage_tests {
    use super::{triage_holds, TriageRefusal};
    use std::path::Path;

    #[test]
    fn a_severity_outside_the_four_is_refused_before_the_model_is_read() {
        // A root that does not exist: the severity check must fire first, or this reads a model.
        let err = triage_holds(Path::new("Z:/no/such/root"), "medium", "anything").unwrap_err();
        assert!(matches!(err, TriageRefusal::Severity(ref s) if s == "medium"), "{err}");
        assert!(err.to_string().contains("Critical | High | Medium | Low"));
    }

    #[test]
    fn the_refusal_text_names_the_guard_whose_predicate_it_applied() {
        let e = TriageRefusal::ResolverKind { resolver: "st1".into(), kind: "Story".into() }.to_string();
        assert!(e.contains("is a Story, not a declared action or a Decision"), "{e}");
        assert!(e.contains("resolver-kind"), "{e}");
    }
}
