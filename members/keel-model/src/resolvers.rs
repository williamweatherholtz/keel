//! The resolver predicate (issue136/issue558): which item may be the source of a `#Resolves` edge. The
//! `resolver-kind` guard applies it at commit and `record issue --resolver` applies it before it writes,
//! so the two cannot disagree. Descended from keel-guards in sprint 737 (D0508) so the item member reads
//! it without depending on the guards; keel-guards re-exports both names at their old paths.

use std::collections::HashSet;
use std::path::Path;

/// All `action <name>;` task names declared in .tracking/{backlog,delivery} (not `action def`).
#[must_use]
pub fn declared_task_names(root: &Path) -> HashSet<String> {
    let mut names = HashSet::new();
    for sub in ["backlog.sysml", "delivery"] {
        let base = root.join(".tracking").join(sub);
        let files = if base.is_dir() { crate::corpus::collect_sysml(&base) } else { vec![base] };
        for f in files {
            let Ok(text) = crate::corpus::read_to_string(&f) else { continue };
            for line in text.lines() {
                let t = line.trim_start();
                if let Some(rest) = t.strip_prefix("action ") {
                    if rest.starts_with("def ") {
                        continue;
                    }
                    let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
                    if !name.is_empty() {
                        names.insert(name);
                    }
                }
            }
        }
    }
    names
}

/// THE ONE PREDICATE behind `resolver-kind`: a `#Resolves` source is a declared action or a `Decision`.
///
/// An action is work that closes the issue; a Decision moots it. `record issue --resolver` reads this
/// same function before it writes the edge, so the write refuses exactly what the commit gate would
/// (issue558): the first triage of issue556 pointed at a Story, the write printed "triaged on
/// arrival", and the pre-commit guard was the first thing to say otherwise.
#[must_use]
pub fn resolver_kind_holds<S: std::hash::BuildHasher>(actions: &HashSet<String, S>, from: &str, ty: &str) -> bool {
    actions.contains(from) || ty == "Decision"
}
