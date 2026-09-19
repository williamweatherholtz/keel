//! Guard family `surface` - split from `keel-cli/src/guards.rs` by `scripts/split_guards.py` (sprint 733).
//!
//! Each guard's dispatch arm, code and tests sit together; the shared scanners, the runner and the
//! lock predicates are the crate root's (`super`). Nothing here was retyped: the text is guards.rs's,
//! with `crate::` paths pointing at the members and private items opened to the crate.

use super::*;

/// The `surface` family: every guard it dispatches, in `GUARD_NAMES` order, with the tier note each
/// arm carried in `run_one` (sprint 733). The root's union test holds these tables equal to `GUARD_NAMES`.
pub(crate) const FAMILY: Family = Family {
    name: "surface",
    arms: &[
        ("viewpoint-renderer", viewpoint_renderer),
        ("tool-reference", tool_reference), // hard (issue196) — a doc naming a deleted tool strands its follower
        ("question-coverage", question_coverage), // D0161: declared knowledge facts are well-formed; coverage itself stays a view
        ("cli-surface-declared", cli_surface_declared),
        ("cli-reference", cli_reference), // hard (D0471/issue528) - a living doc names only verbs this binary dispatches
        ("source-reference", source_reference), // hard (D0514/issue591) - a living doc's `file.rs:N` citation resolves and holds its identifier
    ],
};

// ── viewpoint-renderer guard (every declared viewpoint names a real renderer) ──────────────────


/// Classify a viewpoint renderer string: `"retired"` (query.py/report.py, a violation), `"planned"`
/// (a tolerated warning), `"ok"` (names a real `keel` subcommand), or `"unknown"` (a violation).
pub(crate) fn classify_renderer(r: &str) -> &'static str {
    if r.contains("query.py") || r.contains("report.py") {
        "retired"
    } else if r.starts_with("(planned") {
        "planned"
    } else if keel_schema::cli_surface::renderer_command(r).is_some_and(|(verb, lens)| {
        // D0273: the lens family collapsed into ONE router, so a renderer now reads
        // `keel show <lens>`. Accepting only the verb would let `keel show frobnicate` pass, which
        // is the same hole with an extra word in it — so when the verb is the router, the LENS is
        // what must resolve.
        if verb == "show" {
            lens.is_some_and(keel_schema::cli_surface::has_lens)
        } else {
            keel_schema::cli_surface::has_command(verb)
        }
    }) {
        "ok"
    } else {
        "unknown"
    }
}

/// Guard (D0056/issue034): every declared Viewpoint's renderer names a real current command
/// (a `keel <subcommand>`), or is explicitly `(planned ...)`.
///
/// A renderer referencing a RETIRED tool (query.py / report.py, D0074) or an unknown command is a
/// violation — it stops the viewpoint registry from drifting to dead renderers (the d0056 finding).
/// A `(planned ...)` renderer is a tolerated WARNING (a declared-but-unbuilt concern).
#[must_use]
pub fn viewpoint_renderer(root: &Path) -> GuardReport {
    // EVERY declared Viewpoint, not just the registry file's (issue139). This guard read one hardcoded
    // filename while the model saw them all, so a viewpoint declared elsewhere never had its renderer
    // checked — proven with a probe whose renderer named no command and which the guard did not see.
    let vps = match keel_view::view::declared_viewpoints(root) {
        Ok(v) => v,
        Err(e) => return GuardReport { name: "viewpoint-renderer", scanned: 0, warnings: Vec::new(), violations: vec![format!("cannot enumerate viewpoints: {e}")] },
    };
    let mut scanned = 0;
    let mut warnings = Vec::new();
    let mut violations = Vec::new();
    // A DEACTIVATED viewpoint's renderer is not required to resolve (D0164), the same way a deactivated
    // process's guards are skipped: a project that declared it does not look through this lens has not
    // violated a rule about the lens. Reported in `keel activation`, never silent.
    let act = keel_model::activation::Activation::load(root);
    for vp in vps {
        if !act.is_viewpoint_active(&vp.name) {
            continue;
        }
        // A Viewpoint with NO renderer is a declared lens nothing can render, so it is judged rather
        // than skipped — the previous text scan only ever saw viewpoints that had the line at all.
        let r = vp.renderer;
        let label = if vp.title.is_empty() { vp.name } else { vp.title };
        scanned += 1;
        if r.trim().is_empty() {
            violations.push(format!("{label}: viewpoint declares NO renderer — a lens nothing can render is a concern claimed and not served"));
            continue;
        }
        match classify_renderer(&r) {
            "retired" => violations.push(format!("{label}: renderer references a RETIRED tool (query.py/report.py, D0074) — '{r}'")),
            "unknown" => violations.push(format!("{label}: renderer names no known keel command — '{r}'")),
            "planned" => warnings.push(format!("{label}: viewpoint declared but renderer is planned/unbuilt — '{r}'")),
            _ => {}
        }
    }
    GuardReport { name: "viewpoint-renderer", scanned, warnings, violations }
}

/// Guard 39: a tool the LIVING doc surface references must EXIST (issue196).
///
/// Sprint 377's closeOut recorded the python deck generator as deleted while the file still sat in
/// `.engine/tools/` with two live references — a claimed deletion nobody ran `ls` against
/// (verify-the-wrong-surface, filesystem edition). Its first dry run found a SECOND stale reference
/// (a retired hook script still named in a skill). The checkable half is mechanical: every
/// `.engine/tools/<file>` mentioned in processes, skills, docs, or CLAUDE.md must resolve on disk.
///
/// SCOPE IS THE LIVING SURFACE ONLY — decisions and `.tracking` are historical records and may name
/// tools that no longer exist, truthfully. The no-tombstones rule applies to what this guard scans;
/// immutability applies to what it does not.
#[must_use]
pub fn tool_reference(root: &Path) -> GuardReport {
    let files = living_doc_files(root);
    let needle = ".engine/tools/";
    let mut scanned = 0usize;
    let mut violations = Vec::new();
    let mut reported = std::collections::BTreeSet::new();
    for path in &files {
        let Ok(text) = keel_model::corpus::read_to_string(path) else { continue };
        let rel = relpath(root, path);
        for (n, line) in text.lines().enumerate() {
            let mut rest = line;
            while let Some(i) = rest.find(needle) {
                let tail = &rest[i..];
                let end = tail
                    .find(|c: char| !(c.is_ascii_alphanumeric() || "._/-".contains(c)))
                    .unwrap_or(tail.len());
                let mut tok = &tail[..end];
                while tok.ends_with('.') || tok.ends_with('-') {
                    tok = &tok[..tok.len() - 1];
                }
                rest = &tail[needle.len()..];
                // a bare directory mention carries no filename; only a file reference is checkable
                if !tok.rsplit('/').next().is_some_and(|f| f.contains('.')) {
                    continue;
                }
                scanned += 1;
                if !root.join(tok).exists() && reported.insert(tok.to_string()) {
                    violations.push(format!(
                        "{rel}:{}: references `{tok}`, which does not exist - a follower hits a dead path, and a claimed deletion that left references is a claim nobody verified",
                        n + 1
                    ));
                }
            }
        }
    }
    GuardReport { name: "tool-reference", scanned, warnings: Vec::new(), violations }
}

/// Guard 74: every `keel <verb>` the LIVING doc surface names is a verb this binary dispatches (issue528/D0471).
///
/// Sprint 701's VERIFIER followed the test-verify skill verbatim; its three gate lines said `KEEL
/// validate .`, `KEEL check-engine .`, `KEEL guard --no-receipt .` — top-level verbs D0452 folded under
/// `gate` — so each exited 2 with a usage dump, and the receipt came back with no gate lines and
/// `DISCREPANCIES: NONE`: a control (D0425) whose procedure had silently stopped naming real commands.
/// `cli-surface-declared` holds facts = help = dispatch; nothing held the docs to any of the three.
///
/// WHAT IS A COMMAND REFERENCE. `keel`, `KEEL`, `keel.exe` or a copy (`keel-serve.exe`), then a verb.
/// Inside code — a fenced block or a backtick span — every such token is one. In prose (a `.sysml`
/// string, a `.toml` comment, a sentence) `keel` is also the project's name ("keel is a system", "the
/// keel write API"), so a prose token counts only when it carries a flag or a root argument (`Run keel
/// gate --fast .`, `keel suite --touched`). A prose mention of a bare verb is skipped, not judged.
///
/// WHAT IS CHECKED. The verb is in `cli_surface::COMMAND_NAMES`. When the verb is a router that declares
/// sub-verbs (`gate`, `record`, `audit`, `process`, `library`, `github`, `hook`) and the next token is
/// verb-shaped, it is one of that router's declared sub-verbs (`cli_facts`, D0451/D0454); after `show`
/// it is a lens. A flag, a placeholder or a path as the next token is not checked. The message names
/// the spelling that exists when the retired verb survives as a sub-verb or a lens.
///
/// SCOPE IS THE LIVING SURFACE ONLY, as tool-reference's: processes, skills, docs, contracts,
/// workflows, rules, CLAUDE.md, and the output style (its rules bind every turn). Decisions and
/// `.tracking` are history and may truthfully name a verb that no longer exists.
#[must_use]
pub fn cli_reference(root: &Path) -> GuardReport {
    let mut files = living_doc_files(root);
    let styles = root.join(".claude").join("output-styles");
    if styles.is_dir() {
        if let Ok(rd) = std::fs::read_dir(&styles) {
            files.extend(rd.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("md"))));
        }
    }
    let mut scanned = 0usize;
    let mut violations = Vec::new();
    for path in &files {
        let Ok(text) = keel_model::corpus::read_to_string(path) else { continue };
        let rel = relpath(root, path);
        let markdown = path.extension().is_some_and(|x| x.eq_ignore_ascii_case("md"));
        for (n, r) in cli_references(&text, markdown) {
            scanned += 1;
            if let Some(what) = cli_reference_defect(&r) {
                violations.push(format!("{rel}:{n}: names `keel {}`{what} (issue528)", r.phrase()));
            }
        }
    }
    GuardReport { name: "cli-reference", scanned, warnings: Vec::new(), violations }
}

/// Guard 76: every `<file>.rs:N` / `<file>.rs:N-M` citation on the LIVING doc surface resolves to a
/// source file under a workspace member, and the identifier it sits beside is inside the cited range
/// (issue591 / D0514).
///
/// Sprint 736 moved `migrate.rs` to `members/keel-process` while the project-migration skill said
/// `migrate.rs:664-666` for `check_preconditions` - a function that sat at 805-807 before the move and
/// stayed there after it, the prose untouched. `tool-reference` holds `.engine/tools/<file>` names to
/// disk and `cli-reference` holds `keel <verb>` to the dispatch; a line citation was held to nothing,
/// and every D0479 extraction moves files that skills cite.
///
/// WHAT IS A CITATION. A token ending `.rs` followed by `:N` or `:N-M`: a bare basename
/// (`migrate.rs:804-807`) or a path (`members/keel-process/src/migrate.rs:804-807`). A `.py:N` line or
/// a path under `target/` is outside the population - not read, not counted.
///
/// HOW IT RESOLVES. Through the manifest-driven module home `scripts/module_home.py` reads: the corpus
/// is every `.rs` under every `[workspace] member`'s `src/`. A path resolves when the corpus holds it; a
/// bare basename resolves when exactly one corpus file bears it - two is a violation asking for the
/// path, none is a violation naming the file of that name if one exists elsewhere.
///
/// WHAT IS HELD. The range is within the file. When a backticked identifier sits on the line - the
/// nearest one to the citation - or, failing that, the last one on the wrapped line above it, that
/// identifier appears within the cited lines; a miss names the line it is defined on today. A citation
/// with no identifier in reach is counted and held to the file and the range alone - the stated
/// residual: a bare `x.rs:12` can drift within its file unseen, and naming what it cites is what
/// makes it checkable.
///
/// SCOPE IS THE LIVING SURFACE ONLY, as tool-reference's; Decisions and `.tracking` are history and may
/// truthfully cite a line that has since moved.
///
/// A ROOT WITH NO `Cargo.toml` HAS NO CORPUS. The engine docs a scaffolded project receives cite keel's
/// own source, which that project does not hold: `keel init` then `gate guard all` reddened every one
/// of the 23 scaffold-and-gate tests on this guard's first touched run. Such a root scans nothing and
/// passes - the claim is made where the source is, the self-build, and a project cannot hold it.
///
/// A ROOT WITH A `Cargo.toml` THAT IS NOT THE SELF-BUILD holds its own corpus and the engine's shipped
/// docs: an adopter with a Rust workspace was red on citations of `members/keel-process/src/migrate.rs`
/// that only this repository can hold (GH#88, issue612). Outside the self-build a living doc whose
/// path under `.engine/` the embedded engine ships is set aside - not read, not counted - and one
/// warning says how many; the adopter's own claims (`CLAUDE.md`, a project-added file under
/// `.engine/`) are read as before. A claim the engine ships is held where it is made (D0520).
#[must_use]
pub fn source_reference(root: &Path) -> GuardReport {
    if !root.join("Cargo.toml").is_file() {
        return GuardReport { name: "source-reference", scanned: 0, warnings: Vec::new(), violations: Vec::new() };
    }
    let corpus = member_rust_sources(root);
    let self_build = keel_model::corpus::is_self_build(root);
    let mut scanned = 0usize;
    let mut set_aside = 0usize;
    let mut violations = Vec::new();
    for path in &living_doc_files(root) {
        let rel = relpath(root, path);
        if !self_build && is_shipped_engine_doc(&rel) {
            set_aside += 1;
            continue;
        }
        let Ok(text) = keel_model::corpus::read_to_string(path) else { continue };
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            for c in source_citations(line) {
                scanned += 1;
                let above = i.checked_sub(1).and_then(|j| lines.get(j).copied());
                let ident = cited_identifier(line, (c.start, c.end), above);
                if let Some(why) = source_citation_defect(root, &corpus, &c, ident.as_deref()) {
                    violations.push(format!("{rel}:{}: cites `{}`, {why} (issue591)", i + 1, c.token()));
                }
            }
        }
    }
    let mut warnings = Vec::new();
    if set_aside > 0 {
        warnings.push(format!("{set_aside} shipped living doc(s) set aside - the engine's claims, held in the self-build (D0520)"));
    }
    GuardReport { name: "source-reference", scanned, warnings, violations }
}

/// Whether a root-relative living-doc path (`.engine/skills/x/SKILL.md`, either separator) is one the
/// embedded engine ships. `CLAUDE.md` and a project-added file under `.engine/` are not.
fn is_shipped_engine_doc(rel: &str) -> bool {
    let under = rel.replace('\\', "/");
    under.strip_prefix(".engine/").is_some_and(|inner| keel_schema::embedded::ENGINE_DIR.get_file(inner).is_some())
}

/// One `<file>.rs:N[-M]` citation on a doc line, with its byte span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCitation {
    /// The file as written: `migrate.rs` or `members/keel-process/src/migrate.rs`.
    pub file: String,
    pub from: usize,
    pub to: usize,
    pub start: usize,
    pub end: usize,
}

impl SourceCitation {
    fn token(&self) -> String {
        if self.from == self.to {
            format!("{}:{}", self.file, self.from)
        } else {
            format!("{}:{}-{}", self.file, self.from, self.to)
        }
    }
}

/// Every citation on one line, in order. A `.py:N` or any other extension is not one; a path with a
/// `target/` component is a build product, outside the population.
#[must_use]
pub fn source_citations(line: &str) -> Vec<SourceCitation> {
    let path_char = |c: char| c.is_ascii_alphanumeric() || "._/-".contains(c);
    let mut out = Vec::new();
    let mut from = 0usize;
    while let Some(i) = line[from..].find(".rs:").map(|i| from + i) {
        from = i + 4;
        let start = line[..i].char_indices().rev().take_while(|(_, c)| path_char(*c)).last().map_or(i, |(k, _)| k);
        let file = line[start..i + 3].trim_start_matches("./");
        let base = file.rsplit('/').next().unwrap_or_default();
        if base.len() <= 3 || !base.bytes().next().is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_') {
            continue;
        }
        let digits = |s: &str| s.bytes().take_while(u8::is_ascii_digit).count();
        let rest = &line[i + 4..];
        let n1 = digits(rest);
        if n1 == 0 {
            continue;
        }
        let Ok(first) = rest[..n1].parse::<usize>() else { continue };
        let mut end = i + 4 + n1;
        let mut last = first;
        if let Some(r) = rest[n1..].strip_prefix('-') {
            let n2 = digits(r);
            if n2 > 0 {
                if let Ok(v) = r[..n2].parse::<usize>() {
                    last = v;
                    end += 1 + n2;
                }
            }
        }
        // `x.rs:12abc` is not a line number; `x.rs:12.` and `x.rs:12)` are
        if line[end..].starts_with(|c: char| c.is_alphanumeric() || c == '_') {
            continue;
        }
        if file.starts_with("target/") || file.contains("/target/") {
            continue;
        }
        out.push(SourceCitation { file: file.to_string(), from: first, to: last, start, end });
        from = end;
    }
    out
}

/// A backtick span's content when it is shaped like a Rust path or identifier (`check_preconditions`,
/// `Refusal::SelfBuild`, `keel_fs::scratch()`), with a call's `()` and a macro's `!` stripped.
fn identifier_shaped(span: &str) -> Option<&str> {
    let s = span.trim().trim_end_matches("()").trim_end_matches('!');
    let ok = !s.is_empty()
        && s.split("::").all(|seg| {
            seg.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_') && seg.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        });
    ok.then_some(s)
}

/// The identifier a citation sits beside: the nearest identifier-shaped backtick span on the line
/// outside the citation's own span, else the last one on the wrapped line above (a paragraph wraps
/// where it wraps; the identifier and its citation are one sentence).
#[must_use]
pub fn cited_identifier(line: &str, at: (usize, usize), above: Option<&str>) -> Option<String> {
    let spans_of = |l: &str| -> Vec<(usize, usize, String)> {
        backtick_spans(l).into_iter().filter_map(|(a, b)| identifier_shaped(&l[a + 1..b]).map(|s| (a, b, s.to_string()))).collect()
    };
    let (start, end) = at;
    let same: Option<String> = spans_of(line)
        .into_iter()
        .filter(|(a, b, _)| *b < start || end <= *a)
        .min_by_key(|(a, b, _)| if *b < start { start - *b } else { *a - end })
        .map(|(_, _, s)| s);
    same.or_else(|| above.filter(|l| !l.trim().is_empty()).and_then(|l| spans_of(l).pop().map(|(_, _, s)| s)))
}

/// Every `.rs` under every `[workspace] member`'s `src/`, repo-relative with `/` separators, manifest
/// order then path order - the corpus `scripts/module_home.py` resolves a module in (D0486: the one
/// list). A member written as `dir/*` expands to the subdirectories that hold a `Cargo.toml`.
#[must_use]
pub fn member_rust_sources(root: &Path) -> Vec<String> {
    fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        let mut entries: Vec<_> = rd.flatten().map(|e| e.path()).collect();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    let manifest = keel_model::corpus::read_to_string(root.join("Cargo.toml")).unwrap_or_default();
    let mut dirs = Vec::new();
    for m in keel_model::corpus::workspace_members(&manifest) {
        if let Some(parent) = m.strip_suffix("/*") {
            if let Ok(rd) = std::fs::read_dir(root.join(parent)) {
                let mut subs: Vec<_> = rd.flatten().map(|e| e.path()).filter(|p| p.join("Cargo.toml").is_file()).collect();
                subs.sort();
                dirs.extend(subs);
            }
        } else {
            dirs.push(root.join(&m));
        }
    }
    let mut files = Vec::new();
    for d in dirs {
        walk(&d.join("src"), &mut files);
    }
    files.iter().filter_map(|p| p.strip_prefix(root).ok()).map(|p| p.to_string_lossy().replace('\\', "/")).collect()
}

/// Where a cited file is in the corpus: one path, several bearing a bare basename, or none (with the
/// files elsewhere in the corpus that bear the same basename, so the message can say where it went).
#[derive(Debug, PartialEq, Eq)]
pub enum Resolution {
    Path(String),
    Ambiguous(Vec<String>),
    Missing(Vec<String>),
}

/// Resolve a citation's file through the corpus: a path must be in it; a bare basename must be borne
/// by exactly one entry.
#[must_use]
pub fn resolve_citation(corpus: &[String], file: &str) -> Resolution {
    let base = file.rsplit('/').next().unwrap_or(file);
    let bearing: Vec<String> = corpus.iter().filter(|p| p.rsplit('/').next() == Some(base)).cloned().collect();
    if file.contains('/') {
        if corpus.iter().any(|p| p == file) {
            Resolution::Path(file.to_string())
        } else {
            Resolution::Missing(bearing)
        }
    } else {
        match bearing.as_slice() {
            [one] => Resolution::Path(one.clone()),
            [] => Resolution::Missing(Vec::new()),
            _ => Resolution::Ambiguous(bearing),
        }
    }
}

/// `id` as a whole word in `line`: no identifier character on either side.
fn has_word(line: &str, id: &str) -> bool {
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    let mut from = 0usize;
    while let Some(i) = line[from..].find(id).map(|i| from + i) {
        let before = line[..i].chars().next_back().is_some_and(ident);
        let after = line[i + id.len()..].starts_with(ident);
        if !before && !after {
            return true;
        }
        from = i + id.len();
    }
    false
}

/// The 1-based line `id` is defined on in `text` - `fn id`, `struct id`, `const id`, ... - else the
/// first line naming it as a word; `None` when the file never names it.
#[must_use]
pub fn definition_line(text: &str, id: &str) -> Option<usize> {
    const KEYWORDS: [&str; 10] = ["fn", "struct", "enum", "const", "static", "mod", "trait", "type", "union", "macro_rules!"];
    let naming: Vec<(usize, &str)> = text.lines().enumerate().filter(|(_, l)| has_word(l, id)).map(|(i, l)| (i + 1, l)).collect();
    naming
        .iter()
        .find(|(_, l)| KEYWORDS.iter().any(|k| l.contains(&format!("{k} {id}"))))
        .or_else(|| naming.first())
        .map(|(n, _)| *n)
}

/// Why a citation is a defect, or `None` when it resolves, its range is within the file and the
/// identifier beside it (when one is in reach) sits inside that range. The text names the line the
/// identifier is on today, so the follower can repoint without opening the file.
#[must_use]
pub fn source_citation_defect(root: &Path, corpus: &[String], c: &SourceCitation, ident: Option<&str>) -> Option<String> {
    let path = match resolve_citation(corpus, &c.file) {
        Resolution::Path(p) => p,
        Resolution::Ambiguous(many) => {
            return Some(format!("a bare name {} source files bear - cite the path: {}", many.len(), many.iter().map(|p| format!("`{p}`")).collect::<Vec<_>>().join(", ")));
        }
        Resolution::Missing(elsewhere) => {
            let today = match elsewhere.as_slice() {
                [] => String::new(),
                [one] => format!(" - today the file is `{one}`"),
                many => format!(" - files of that name today: {}", many.iter().map(|p| format!("`{p}`")).collect::<Vec<_>>().join(", ")),
            };
            return Some(format!("which resolves to no source file under a workspace member's src/{today}"));
        }
    };
    let Ok(text) = keel_model::corpus::read_to_string(root.join(&path)) else {
        return Some(format!("and `{path}` cannot be read"));
    };
    let total = text.lines().count();
    if c.from == 0 || c.to < c.from {
        return Some(format!("whose range {}-{} is not a range of lines", c.from, c.to));
    }
    if c.to > total {
        return Some(format!("whose range runs past the end of `{path}` ({total} lines)"));
    }
    let id = ident?;
    let last = id.rsplit("::").next().unwrap_or(id);
    if text.lines().skip(c.from - 1).take(c.to - c.from + 1).any(|l| has_word(l, last)) {
        return None;
    }
    Some(definition_line(&text, last).map_or_else(
        || format!("beside `{id}`, which `{path}` does not name at all"),
        |n| format!("beside `{id}`, which is not within lines {}-{} of `{path}` - today it is at line {n}", c.from, c.to),
    ))
}

/// The `.md` / `.sysml` / `.toml` files of the living doc surface: `.engine/{processes,skills,docs,
/// contracts,workflows,rules}` and `CLAUDE.md`. Shared by tool-reference, cli-reference and
/// source-reference so the three guards mean the same thing by "living".
pub(crate) fn living_doc_files(root: &Path) -> Vec<std::path::PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| {
                x.eq_ignore_ascii_case("md") || x.eq_ignore_ascii_case("sysml") || x.eq_ignore_ascii_case("toml")
            }) {
                out.push(p);
            }
        }
    }
    let mut files = Vec::new();
    for base in ["processes", "skills", "docs", "contracts", "workflows", "rules"] {
        walk(&root.join(".engine").join(base), &mut files);
    }
    let claude = root.join("CLAUDE.md");
    if claude.exists() {
        files.push(claude);
    }
    files
}

/// One `keel <verb> [next]` token the guard judges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliReference {
    pub verb: String,
    /// The token after the verb, if the line has one (`--fast`, `.`, `validate`, `<lens>`).
    pub next: Option<String>,
}

impl CliReference {
    fn phrase(&self) -> String {
        match (&self.next, cli_reference_sub_verb(self)) {
            (Some(n), Some(_)) => format!("{} {n}", self.verb),
            _ => self.verb.clone(),
        }
    }
}

/// The next token when it is verb-shaped and follows a router that declares sub-verbs (or `show`).
pub(crate) fn cli_reference_sub_verb(r: &CliReference) -> Option<&str> {
    let n = r.next.as_deref()?;
    let verb_shaped = n.starts_with(|c: char| c.is_ascii_lowercase()) && n.chars().all(|c| c.is_ascii_lowercase() || c == '-');
    if !verb_shaped {
        return None;
    }
    if r.verb == "show" || !router_sub_verbs(&r.verb).is_empty() {
        Some(n)
    } else {
        None
    }
}

/// The sub-verbs a command fact declares in its invocation (D0451); empty for a plain command.
pub(crate) fn router_sub_verbs(verb: &str) -> Vec<String> {
    keel_schema::cli_facts::command_facts()
        .find(|f| f.name == verb)
        .map(|f| keel_schema::cli_facts::sub_verbs_of(f.invocation))
        .unwrap_or_default()
}

/// Why a reference is a defect, or `None` when the binary dispatches it. The text names the spelling
/// that exists today when the retired verb survives as a sub-verb or a lens.
#[must_use]
pub fn cli_reference_defect(r: &CliReference) -> Option<String> {
    let now = |verb: &str| -> String {
        if keel_schema::cli_surface::has_lens(verb) {
            return format!("; today it is `keel show {verb}`");
        }
        let routers: Vec<&str> = keel_schema::cli_facts::command_facts()
            .filter(|f| keel_schema::cli_facts::sub_verbs_of(f.invocation).iter().any(|s| s == verb))
            .map(|f| f.name)
            .collect();
        match routers.as_slice() {
            [] => String::new(),
            [one] => format!("; today it is `keel {one} {verb}`"),
            many => format!("; today it is a sub-verb of {}", many.join(", ")),
        }
    };
    if !keel_schema::cli_surface::has_command(&r.verb) {
        return Some(format!(
            ", which this binary does not dispatch - a follower's command exits 2 with a usage dump{}",
            now(&r.verb)
        ));
    }
    let sub = cli_reference_sub_verb(r)?;
    let declared = if r.verb == "show" {
        keel_schema::cli_surface::has_lens(sub)
    } else {
        router_sub_verbs(&r.verb).iter().any(|s| s == sub)
    };
    if declared {
        None
    } else if r.verb == "show" {
        Some(format!(", and `{sub}` is not a lens `keel show` lists{}", now(sub)))
    } else {
        Some(format!(", and `{sub}` is not a sub-verb `{}` declares in .engine/cli/commands.sysml{}", r.verb, now(sub)))
    }
}

/// Every command reference in `text` with its 1-based line.
///
/// Inside code always; in prose only when the phrase carries a flag or a root argument. `markdown` turns
/// on fence tracking; a `.sysml` or `.toml` file has no fences, only backtick spans in strings and comments.
#[must_use]
pub fn cli_references(text: &str, markdown: bool) -> Vec<(usize, CliReference)> {
    let mut out = Vec::new();
    let mut fenced = false;
    for (i, line) in text.lines().enumerate() {
        if markdown && line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        let spans = backtick_spans(line);
        let mut from = 0usize;
        while let Some((at, after_bin)) = find_keel_token(line, from) {
            from = after_bin;
            let rest = &line[after_bin..];
            let ws = rest.len() - rest.trim_start_matches([' ', '\t']).len();
            if ws == 0 {
                continue;
            }
            let rest = &rest[ws..];
            let verb_len = rest.find(|c: char| !(c.is_ascii_lowercase() || c == '-')).unwrap_or(rest.len());
            if verb_len == 0 || !rest.starts_with(|c: char| c.is_ascii_lowercase()) {
                continue;
            }
            let verb = &rest[..verb_len];
            // a verb ending in a letter-run glued to `'s`, `-`… is prose ("engine's"): the char after must not be a word char
            if rest[verb_len..].starts_with(|c: char| c.is_alphanumeric() || c == '_' || c == '\'') {
                continue;
            }
            // a root argument `.` is its own token: `keel itself.` ends a sentence, `keel gate .` names a root
            let spaced = rest[verb_len..].starts_with([' ', '\t']);
            let tail = rest[verb_len..].trim_start_matches([' ', '\t']);
            let next = tail
                .split(|c: char| c.is_whitespace() || c == '`' || c == '"' || c == ')' || c == ']' || c == ';' || c == ',')
                .next()
                .filter(|t| !t.is_empty())
                .map(str::to_string);
            let in_code = fenced || spans.iter().any(|(a, b)| *a < at && at < *b);
            let command_shaped = spaced && next.as_deref().is_some_and(|n| n.starts_with('-') || n == ".");
            if in_code || command_shaped {
                out.push((i + 1, CliReference { verb: verb.to_string(), next }));
            }
        }
    }
    out
}

/// Byte ranges of the backtick spans on one line, `(open, close)`.
pub(crate) fn backtick_spans(line: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut from = 0usize;
    while let Some(a) = line[from..].find('`') {
        let a = from + a;
        let Some(b) = line[a + 1..].find('`') else { break };
        let b = a + 1 + b;
        out.push((a, b));
        from = b + 1;
    }
    out
}

/// The next `keel` / `KEEL` / `keel.exe` / `keel-<copy>.exe` binary token at or after `from` that is
/// not part of a longer word or a dotted name (`.keel/`, `keel-cli`): returns `(start, end)`.
pub(crate) fn find_keel_token(line: &str, from: usize) -> Option<(usize, usize)> {
    let mut search = from;
    while let Some(i) = line[search..].find("keel").or_else(|| line[search..].find("KEEL")).map(|i| search + i) {
        // both spellings may occur; take the earlier of the two
        let i = match (line[search..].find("keel"), line[search..].find("KEEL")) {
            (Some(a), Some(b)) => search + a.min(b),
            _ => i,
        };
        search = i + 4;
        let before = line[..i].chars().next_back();
        if before.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == '-') {
            continue;
        }
        let mut end = i + 4;
        let rest = &line[end..];
        if let Some(r) = rest.strip_prefix('-') {
            // a copy of the binary: keel-serve.exe, keel-land.exe - only with the .exe suffix
            let name_len = r.find(|c: char| !c.is_ascii_lowercase()).unwrap_or(r.len());
            if name_len > 0 && r[name_len..].starts_with(".exe") {
                end += 1 + name_len + 4;
            } else {
                continue;
            }
        } else if rest.starts_with(".exe") {
            end += 4;
        } else if rest.starts_with(|c: char| c.is_alphanumeric() || c == '_' || c == '.' || c == '-' || c == '/') {
            continue;
        }
        return Some((i, end));
    }
    None
}

// ── question-coverage guard (declared knowledge facts are well-formed, D0161 part 3ii) ────────────

/// Guard: declared knowledge facts are well-formed (D0161 part 3ii).
///
/// A `Question` carries its text; an `Alias` carries its term and maps to an existing element.
/// WELL-FORMEDNESS only — coverage itself stays `keel knowledge question-coverage`,
/// because gating on coverage would make the cheapest fix deleting the question (D0098). Zero
/// declared facts = zero scanned, green: an absent `.knowledge/` is the feature unplugged (D0161
/// part 3i), never a violation. Unit-owned by `knowledge-graph-memory`, so deactivating that process
/// drops exactly this check.
#[must_use]
pub fn question_coverage(root: &Path) -> GuardReport {
    match keel_view::view::knowledge_wellformedness(root) {
        Ok((scanned, violations)) => GuardReport { name: "question-coverage", scanned, warnings: Vec::new(), violations },
        Err(e) => GuardReport {
            name: "question-coverage",
            scanned: 0,
            warnings: Vec::new(),
            violations: vec![format!("error reading knowledge facts: {e}")],
        },
    }
}

/// The comparison, pure: authored facts vs the Rust mirror vs the dispatch inventory.
///
/// BOTH WAYS on every edge. Returns violations only - there is no advisory shape here, because any disagreement
/// means `keel --help` describes a surface that does not exist or hides one that does.
#[must_use]
pub fn cli_surface_violations(
    authored: &[AuthoredCliFact],
    mirror: &[keel_schema::cli_facts::CliFact],
    commands: &[&str],
    lenses: &[&str],
) -> Vec<String> {
    use std::collections::BTreeMap;
    let mut out = Vec::new();
    // issue548: two facts IN FORCE declaring one command name were collapsed into this map silently, the
    // later one winning. A superseder shares its target's name by design, but the target is dropped by
    // `parse_cli_facts` before it reaches here, so a duplicate at this point is two live declarations.
    let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
    for f in authored {
        if let Some(first) = seen.insert(f.name.as_str(), f.part.as_str()) {
            out.push(format!("`{}` is declared by two CliCommand facts in force (`{first}` and `{}`) - retire one with a #Supersede edge (D0108)", f.name, f.part));
        }
    }
    let by_name: BTreeMap<&str, &AuthoredCliFact> = authored.iter().map(|f| (f.name.as_str(), f)).collect();
    let mirror_by: BTreeMap<&str, &keel_schema::cli_facts::CliFact> = mirror.iter().map(|f| (f.name, f)).collect();
    // authored <-> mirror. The invocation is compared too (issue548): it is the field `--help` renders as
    // the command's shape, and it was the one field the two homes could disagree on unseen.
    for f in authored {
        match mirror_by.get(f.name.as_str()) {
            None => out.push(format!("`{}` is an authored CliCommand fact with no entry in cli_facts.rs - the help cannot describe it", f.name)),
            Some(m) => {
                for (what, a, b) in [("family", f.family.as_str(), m.family), ("effect", f.effect.as_str(), m.effect), ("stability", f.stability.as_str(), m.stability), ("invocation", f.invocation.as_str(), m.invocation), ("synopsis", f.synopsis.as_str(), m.synopsis)] {
                    if a != b {
                        out.push(format!("`{}` {what} differs: facts say `{a}`, cli_facts.rs says `{b}` - one home, the .sysml; regenerate the mirror", f.name));
                    }
                }
            }
        }
    }
    for m in mirror {
        if !by_name.contains_key(m.name) {
            out.push(format!("`{}` is in cli_facts.rs but has no authored CliCommand fact - the help describes a command the model does not declare", m.name));
        }
    }
    // authored <-> dispatch, by kind
    for f in authored {
        let is_lens = f.family == "lens";
        let dispatched = if is_lens { lenses.contains(&f.name.as_str()) } else { commands.contains(&f.name.as_str()) };
        if !dispatched {
            out.push(format!("`{}` is declared as a {} but nothing dispatches it", f.name, if is_lens { "show lens" } else { "command" }));
        }
    }
    for c in commands {
        if by_name.get(c).is_none_or(|f| f.family == "lens") {
            out.push(format!("`{c}` is dispatched as a command but has no CliCommand fact (D0271: every command is an authored fact)"));
        }
    }
    for l in lenses {
        if by_name.get(l).is_none_or(|f| f.family != "lens") {
            out.push(format!("`{l}` is dispatched as a show lens but has no CliCommand fact with family `lens`"));
        }
    }
    out
}

/// Every `dNNNN` / `DNNNN` token in `text`, lowercased, in order, deduplicated.
///
/// A token is four digits behind a `d`/`D` with no identifier character on either side, so `D0356`
/// and `d0345` are cited and `dcOneClick`, `d03561` and `keeld0001` are not.
#[must_use]
pub fn decision_ids_cited(text: &str) -> Vec<String> {
    let b = text.as_bytes();
    let ident = |i: usize| b.get(i).is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_');
    let mut out: Vec<String> = Vec::new();
    for (i, c) in b.iter().enumerate() {
        if !matches!(c, b'd' | b'D') || (i > 0 && ident(i - 1)) {
            continue;
        }
        let Some(digits) = b.get(i + 1..i + 5) else { continue };
        if digits.iter().all(u8::is_ascii_digit) && !ident(i + 5) {
            let id = format!("d{}", String::from_utf8_lossy(digits));
            if !out.contains(&id) {
                out.push(id);
            }
        }
    }
    out
}

/// The Decision ids that exist in this tree: `.engine/decisions/NNNN-*.sysml` -> `dNNNN`.
pub(crate) fn decision_ids_present(root: &Path) -> BTreeSet<String> {
    keel_model::corpus::collect_sysml(&root.join(".engine").join("decisions"))
        .iter()
        .filter_map(|p| p.file_name().and_then(|n| n.to_str()))
        .filter_map(|n| n.get(..4).filter(|d| d.bytes().all(|c| c.is_ascii_digit())))
        .map(|d| format!("d{d}"))
        .collect()
}

/// issue423: a `CliCommand` synopsis that cites a Decision cites a LIVE one.
///
/// Pure. `synopses` is `(home, command, synopsis)` for both fact homes; `present` is the ids with a
/// file under `.engine/decisions`; `retired` maps each `#Supersede` target to the Decision that
/// retired it (D0398: a retired Decision is out of every scorecard, so a synopsis pointing a reader
/// at it points them at authority the tree has withdrawn). A cited id that is retired, or that has
/// no file, is a violation naming the home, the command, the id and - when retired - the superseder.
#[must_use]
pub fn synopsis_citation_violations(
    synopses: &[(&str, &str, &str)],
    present: &BTreeSet<String>,
    retired: &BTreeMap<String, String>,
) -> Vec<String> {
    let mut out = Vec::new();
    for (home, command, synopsis) in synopses {
        for id in decision_ids_cited(synopsis) {
            if let Some(by) = retired.get(&id) {
                out.push(format!("`{command}` synopsis in {home} cites {id}, which is RETIRED - superseded by {by} (D0398); cite {by} or the Decision in force"));
            } else if !present.contains(&id) {
                out.push(format!("`{command}` synopsis in {home} cites {id}, and no Decision file under .engine/decisions carries that number"));
            }
        }
    }
    out
}

/// Guard: the CLI surface is an authored fact, held equal to the dispatch and to the help.
///
/// (D0271, issue344.) `.engine/cli/commands.sysml` is the home; `cli_facts::CLI_FACTS` mirrors it and renders
/// `keel --help`; `cli_surface::COMMAND_NAMES` / `LENS_NAMES` are the dispatch. Any of the three
/// disagreeing is a violation. An absent facts file is a violation too: a project on this engine
/// vintage ships the file, and a tree without it is a tree whose help describes nothing.
#[must_use]
pub fn cli_surface_declared(root: &Path) -> GuardReport {
    let path = root.join(".engine").join("cli").join("commands.sysml");
    let Ok(text) = keel_model::corpus::read_to_string(&path) else {
        return GuardReport {
            name: "cli-surface-declared",
            scanned: 0,
            warnings: Vec::new(),
            violations: vec![format!("{} is absent - the CLI facts (D0271) are not in this tree; `keel migrate` resyncs it", path.display())],
        };
    };
    let authored = parse_cli_facts(&text);
    let mut violations = cli_surface_violations(&authored, &keel_schema::cli_facts::CLI_FACTS, &keel_schema::cli_surface::COMMAND_NAMES, &keel_schema::cli_surface::LENS_NAMES);
    // issue423: what a synopsis CITES is held to the tree too - both homes, so a stale citation
    // cannot survive in the one the drift check happens not to compare.
    //
    // ONLY WHERE THE CITED DECISIONS LIVE (issue433, D0419). The Decisions a synopsis cites are the
    // ENGINE's, in the engine's own `.engine/decisions`. A downstream project's `.engine/decisions`
    // holds THAT project's Decisions - `keel init` ships none - so the same check read every scaffold
    // as citing twelve missing Decisions and turned CI red for six pushes (the two tests that build
    // a project and land in it). The engine's channel is recognised by D0271 being in it - the
    // Decision that made the surface an authored fact; a tree without it cannot resolve what the
    // synopses cite, and `keel-cli/Cargo.toml` alone does not say so (a test fixture shaped like the
    // self-build carries no Decisions either). Where the citations cannot resolve, they are not read.
    let present = decision_ids_present(root);
    if present.contains("d0271") {
        let mut synopses: Vec<(&str, &str, &str)> = authored.iter().map(|f| (".engine/cli/commands.sysml", f.name.as_str(), f.synopsis.as_str())).collect();
        synopses.extend(keel_schema::cli_facts::CLI_FACTS.iter().map(|f| ("cli_facts.rs", f.name, f.synopsis)));
        let retired: BTreeMap<String, String> = keel_model::corpus::supersede_edges(root).into_iter().map(|(from, to)| (to, from)).collect();
        violations.extend(synopsis_citation_violations(&synopses, &present, &retired));
    }
    GuardReport { name: "cli-surface-declared", scanned: authored.len(), warnings: Vec::new(), violations }
}

#[cfg(test)]
mod cli_reference_tests {
    use super::{cli_reference, cli_reference_defect, cli_references};

    /// D0388 pair, chosen before the real tree was read. Known positive: a doc line naming a verb the
    /// binary no longer dispatches fails naming the line. Known negative: the current spellings pass.
    #[test]
    fn a_retired_verb_in_code_fails_naming_its_line_and_the_current_spelling_passes() {
        let root = keel_fs::scratch("keel-cliref-guard");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".engine").join("skills")).expect("mkdir");
        std::fs::create_dir_all(root.join(".tracking")).expect("mkdir");
        std::fs::write(
            root.join(".engine").join("skills").join("s.md"),
            "run `keel guard .` first\nthen `keel gate guard .` and `keel show why`\n",
        )
        .expect("write");
        std::fs::write(root.join(".tracking").join("h.sysml"), "// history: `keel guard .` once existed\n").expect("write");
        let report = cli_reference(&root);
        assert_eq!(report.scanned, 3, "{:?}", report.violations);
        assert_eq!(report.violations.len(), 1, "{:?}", report.violations);
        assert!(report.violations[0].starts_with(".engine/skills/s.md:1: names `keel guard`"), "{}", report.violations[0]);
        assert!(report.violations[0].contains("today it is `keel gate guard`"), "{}", report.violations[0]);
    }

    /// The token shapes: a fence, an uppercase spelling, a copy of the binary, a prose command with a
    /// flag, and the project's name in a sentence, which is not a command at all.
    #[test]
    fn code_and_flagged_prose_are_references_and_the_project_name_in_a_sentence_is_not() {
        let md = "```\nKEEL validate .\n```\nkeel is also a system, and the keel write API is one channel.\n`./target/release/keel-serve.exe show orient .`\nThe keel engine's contract. Run it for keel itself. Then `keel gate .`\n";
        let refs = cli_references(md, true);
        let verbs: Vec<&str> = refs.iter().map(|(_, r)| r.verb.as_str()).collect();
        assert_eq!(verbs, vec!["validate", "show", "gate"], "{refs:?}");
        assert_eq!(refs[0].0, 2);
        assert_eq!(refs[1].1.next.as_deref(), Some("orient"));
        let sysml = ":>> actionText = \"Run keel gate --fast . after the last write; keel record verbs only; a keel process unit\";\n";
        let refs = cli_references(sysml, false);
        assert_eq!(refs.len(), 1, "{refs:?}");
        assert_eq!(refs[0].1.verb, "gate");
        assert_eq!(refs[0].1.next.as_deref(), Some("--fast"));
    }

    /// The second level: a router's next token is held to its declared sub-verbs, a lens to the lens
    /// list, and a flag or placeholder is not judged.
    #[test]
    fn a_router_is_held_to_its_declared_sub_verbs_and_a_lens_to_the_lens_list() {
        let refs = |s: &str| cli_references(&format!("`{s}`\n"), true).remove(0).1;
        assert_eq!(cli_reference_defect(&refs("keel gate validate .")), None);
        assert_eq!(cli_reference_defect(&refs("keel gate --fast")), None);
        assert_eq!(cli_reference_defect(&refs("keel record gate-result --file F")), None);
        assert_eq!(cli_reference_defect(&refs("keel show <lens> [ROOT]")), None);
        assert_eq!(cli_reference_defect(&refs("keel render report assurance")), None, "render declares no sub-verbs");
        assert_eq!(cli_reference_defect(&refs("keel activate stpa-self")), None, "activate declares no sub-verbs");
        let d = cli_reference_defect(&refs("keel gate frobnicate .")).expect("undeclared sub-verb");
        assert!(d.contains("`frobnicate` is not a sub-verb `gate` declares"), "{d}");
        let d = cli_reference_defect(&refs("keel show nothing-here")).expect("unknown lens");
        assert!(d.contains("not a lens"), "{d}");
        let d = cli_reference_defect(&refs("keel knowledge question-coverage")).expect("retired top-level lens");
        assert!(d.contains("today it is `keel show knowledge`"), "{d}");
        let d = cli_reference_defect(&refs("keel workspace")).expect("never a verb");
        assert!(d.ends_with("usage dump"), "{d}");
    }
}

#[cfg(test)]
mod source_reference_tests {
    use super::{cited_identifier, resolve_citation, source_citations, source_reference, Resolution};

    /// A workspace of two members: `keel-a` holds `migrate.rs` with `check_preconditions` defined on
    /// line 5, both hold a `lib.rs`. `.engine/skills/s.md` is the living surface; `.tracking` is history.
    fn workspace(name: &str) -> std::path::PathBuf {
        let root = keel_fs::scratch(name);
        let _ = std::fs::remove_dir_all(&root);
        for m in ["keel-a", "keel-b"] {
            std::fs::create_dir_all(root.join("members").join(m).join("src")).expect("mkdir");
            std::fs::write(root.join("members").join(m).join("src").join("lib.rs"), "pub mod x;\n").expect("write");
        }
        std::fs::write(root.join("Cargo.toml"), "[workspace]\nmembers = [\n    \"members/keel-a\",\n    \"members/keel-b\",\n]\n").expect("write");
        std::fs::write(
            root.join("members/keel-a/src/migrate.rs"),
            "//! doc\n\n/// refuses a self-build\n#[must_use]\npub fn check_preconditions(root: &Path) -> Result<(), Refusal> {\n    if root.join(\"keel-cli\").is_dir() {\n        return Err(Refusal::SelfBuild);\n    }\n    Ok(())\n}\n",
        )
        .expect("write");
        std::fs::create_dir_all(root.join(".engine").join("skills")).expect("mkdir");
        std::fs::create_dir_all(root.join(".tracking")).expect("mkdir");
        root
    }

    /// D0388 pair, chosen before the real tree was read. KNOWN-POSITIVE: the issue591 shape - the
    /// identifier on the line above, the citation a stale range in a bare basename - is red naming the
    /// doc line and the line the function is on today. KNOWN-NEGATIVE: the same citation repointed by
    /// path to the lines that hold the identifier is green; `target/` and `.py` are outside the population.
    #[test]
    fn a_stale_range_is_red_naming_todays_line_and_the_repointed_path_is_green() {
        let root = workspace("keel-srcref-pair");
        std::fs::write(
            root.join(".engine/skills/s.md"),
            "`check_preconditions` refuses any tree holding `keel-cli/Cargo.toml`\nas a self-build (`migrate.rs:1-2`), so this is the surface.\n",
        )
        .expect("write");
        std::fs::write(root.join(".tracking/h.sysml"), "// history: `check_preconditions` at migrate.rs:1-2 once\n").expect("write");
        let red = source_reference(&root);
        assert_eq!(red.scanned, 1, "{:?}", red.violations);
        assert_eq!(red.violations.len(), 1, "{:?}", red.violations);
        assert!(red.violations[0].starts_with(".engine/skills/s.md:2: cites `migrate.rs:1-2`, beside `check_preconditions`"), "{}", red.violations[0]);
        assert!(red.violations[0].contains("not within lines 1-2 of `members/keel-a/src/migrate.rs` - today it is at line 5"), "{}", red.violations[0]);

        std::fs::write(
            root.join(".engine/skills/s.md"),
            "`check_preconditions` refuses any tree holding `keel-cli/Cargo.toml`\nas a self-build (`members/keel-a/src/migrate.rs:5-8`).\nA build product `target/debug/build/out.rs:3` and a script `module_home.py:40` are not source.\n",
        )
        .expect("write");
        let green = source_reference(&root);
        assert_eq!(green.scanned, 1, "{:?}", green.violations);
        assert!(green.violations.is_empty(), "{:?}", green.violations);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The resolution classes: a moved file's old path names where it is today, a bare name two crates
    /// bear asks for the path, an unknown name is missing, and a range past the file's end is red even
    /// with no identifier in reach.
    #[test]
    fn a_moved_path_names_todays_file_a_shared_basename_asks_for_the_path_and_a_range_is_held_to_the_file() {
        let root = workspace("keel-srcref-classes");
        std::fs::write(
            root.join(".engine/skills/s.md"),
            "see keel-cli/src/migrate.rs:5 for `check_preconditions`\nand lib.rs:1 for the root; ghost.rs:9 is nowhere\nand migrate.rs:40-41 is past the end\n",
        )
        .expect("write");
        let r = source_reference(&root);
        assert_eq!(r.scanned, 4, "{:?}", r.violations);
        assert_eq!(r.violations.len(), 4, "{:#?}", r.violations);
        assert!(r.violations[0].contains("`keel-cli/src/migrate.rs:5`, which resolves to no source file") && r.violations[0].contains("today the file is `members/keel-a/src/migrate.rs`"), "{}", r.violations[0]);
        assert!(r.violations[1].contains("`lib.rs:1`, a bare name 2 source files bear") && r.violations[1].contains("`members/keel-a/src/lib.rs`, `members/keel-b/src/lib.rs`"), "{}", r.violations[1]);
        assert!(r.violations[2].contains("`ghost.rs:9`, which resolves to no source file") && !r.violations[2].contains("today"), "{}", r.violations[2]);
        assert!(r.violations[3].contains("`migrate.rs:40-41`, whose range runs past the end of `members/keel-a/src/migrate.rs` (10 lines)"), "{}", r.violations[3]);
        assert_eq!(resolve_citation(&["a/src/x.rs".to_string()], "x.rs"), Resolution::Path("a/src/x.rs".to_string()));
        assert_eq!(resolve_citation(&["a/src/x.rs".to_string()], "b/src/x.rs"), Resolution::Missing(vec!["a/src/x.rs".to_string()]));
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The token shapes and the identifier's reach: a range or a line, a path or a name, a citation in
    /// a backtick span, and the nearest identifier on the line beating the one on the line above.
    #[test]
    fn citations_are_read_in_their_shapes_and_the_identifier_is_the_nearest_in_reach() {
        let cites = |l: &str| source_citations(l).iter().map(|c| (c.file.clone(), c.from, c.to)).collect::<Vec<_>>();
        assert_eq!(cites("at `migrate.rs:664-666` and members/keel-process/src/migrate.rs:805, ./x.rs:3."), vec![("migrate.rs".to_string(), 664, 666), ("members/keel-process/src/migrate.rs".to_string(), 805, 805), ("x.rs".to_string(), 3, 3)]);
        assert!(cites("target/release/build/x.rs:3, facts.py:12, guards.rs: the module, x.rs:12abc, .rs:4").is_empty());
        let line = "`Refusal::SelfBuild` is returned by `check_preconditions` (`migrate.rs:805-807`) before `keel_git::gitx::git()` runs";
        let c = &source_citations(line)[0];
        assert_eq!(cited_identifier(line, (c.start, c.end), Some("`other_fn` above")).as_deref(), Some("check_preconditions"));
        let wrapped = "as a self-build (migrate.rs:664-666), so this is the surface";
        let c = &source_citations(wrapped)[0];
        assert_eq!(cited_identifier(wrapped, (c.start, c.end), Some("`check_preconditions` refuses any tree holding `keel-cli/Cargo.toml`")).as_deref(), Some("check_preconditions"));
        assert_eq!(cited_identifier(wrapped, (c.start, c.end), Some("")), None);
        assert_eq!(cited_identifier(wrapped, (c.start, c.end), None), None);
    }

    /// The live tree: every citation on the living surface resolves and holds its identifier - the
    /// two project-migration lines among them (issue591's known-positive, repointed in sprint 743).
    #[test]
    fn the_living_surface_cites_source_that_resolves() {
        let r = source_reference(&crate::test_repo_root());
        assert!(r.scanned >= 2, "the project-migration skill and process cite migrate.rs: {}", r.scanned);
        assert!(r.violations.is_empty(), "{:#?}", r.violations);
    }

    /// A scaffolded project: the engine docs it received cite keel's source, and there is no
    /// `Cargo.toml` at its root to resolve them against. Nothing is scanned and the guard is green;
    /// the same doc under a root that HAS a manifest is scanned and red. This is the shape that
    /// reddened 23 scaffold-and-gate tests on the guard's first touched run.
    #[test]
    fn a_root_without_a_workspace_manifest_scans_nothing() {
        let root = workspace("keel-srcref-noworkspace");
        std::fs::write(root.join(".engine/skills/s.md"), "`check_preconditions` refuses a self-build (`members/keel-process/src/migrate.rs:804-807`).\n").expect("write");
        let with = source_reference(&root);
        assert_eq!((with.scanned, with.violations.len()), (1, 1), "{:?}", with.violations);
        std::fs::remove_file(root.join("Cargo.toml")).expect("rm");
        let without = source_reference(&root);
        assert_eq!((without.scanned, without.violations.len()), (0, 0), "{:?}", without.violations);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// D0388 pair for issue612 (GH#88), chosen before the tree was read. An adopter with a Rust
    /// workspace holds the shipped project-migration skill, which cites keel's own `migrate.rs`, and
    /// its own `.engine/docs/own.md`, which cites a stale line of its own member. KNOWN-POSITIVE: the
    /// shipped doc is set aside with one warning; the adopter's stale claim is the only violation.
    /// KNOWN-NEGATIVE: the same root with `keel-cli/Cargo.toml` is the self-build and reads both.
    #[test]
    fn outside_the_self_build_a_shipped_doc_is_set_aside_and_the_adopters_own_claim_is_read() {
        let root = workspace("keel-srcref-adopter");
        let skill = keel_schema::embedded::engine_text("skills/project-migration/SKILL.md").expect("the engine ships the skill");
        assert!(skill.contains("members/keel-process/src/migrate.rs:"), "the fixture relies on the shipped skill citing migrate.rs");
        std::fs::create_dir_all(root.join(".engine/skills/project-migration")).expect("mkdir");
        std::fs::write(root.join(".engine/skills/project-migration/SKILL.md"), skill).expect("write");
        std::fs::create_dir_all(root.join(".engine/docs")).expect("mkdir");
        std::fs::write(root.join(".engine/docs/own.md"), "our `check_preconditions` (`members/keel-a/src/migrate.rs:1-2`) is stale here\n").expect("write");

        let adopter = source_reference(&root);
        assert_eq!(adopter.violations.len(), 1, "{:#?}", adopter.violations);
        assert!(adopter.violations[0].starts_with(".engine/docs/own.md:1: cites `members/keel-a/src/migrate.rs:1-2`"), "{}", adopter.violations[0]);
        assert_eq!(adopter.warnings, vec!["1 shipped living doc(s) set aside - the engine's claims, held in the self-build (D0520)".to_string()]);
        assert_eq!(adopter.scanned, 1, "only the adopter's own citation is counted");

        std::fs::create_dir_all(root.join("keel-cli")).expect("mkdir");
        std::fs::write(root.join("keel-cli/Cargo.toml"), "[package]\nname = \"keel-cli\"\n").expect("write");
        let self_build = source_reference(&root);
        let shipped_cites: usize = skill.lines().map(|l| source_citations(l).len()).sum();
        assert_eq!(shipped_cites, 1, "the shipped skill cites migrate.rs once today");
        assert_eq!(self_build.violations.len(), 1 + shipped_cites, "{:#?}", self_build.violations);
        assert!(self_build.warnings.is_empty(), "{:?}", self_build.warnings);
        assert_eq!(self_build.scanned, 1 + shipped_cites, "{}", self_build.scanned);
        let _ = std::fs::remove_dir_all(&root);
    }
}

#[cfg(test)]
mod cli_surface_declared_tests {
    use super::*;

    fn fact(name: &str, family: &str, effect: &str) -> String {
        format!(":>> name = \"{name}\"; :>> family = \"{family}\"; :>> effect = CliEffect::{effect}; :>> stability = CliStability::stable; :>> synopsis = \"s\";")
    }
    fn line(name: &str, family: &str, effect: &str) -> String {
        format!("    part x : CliCommand {{ :>> id = \"i\"; {} }}", fact(name, family, effect))
    }
    fn mirror(name: &'static str, family: &'static str, effect: &'static str) -> keel_schema::cli_facts::CliFact {
        keel_schema::cli_facts::CliFact { name, family, effect, stability: "stable", invocation: "", synopsis: "s" }
    }

    #[test]
    fn a_dispatched_command_with_no_fact_is_a_violation() {
        let authored = parse_cli_facts(&line("orient", "orientation", "reads"));
        let m = [mirror("orient", "orientation", "reads")];
        let v = cli_surface_violations(&authored, &m, &["orient", "ghost"], &[]);
        assert_eq!(v.len(), 1, "{v:?}");
        assert!(v[0].contains("`ghost` is dispatched"), "{v:?}");
    }

    #[test]
    fn a_fact_nothing_dispatches_is_a_violation_and_so_is_a_lens_declared_as_a_command() {
        let text = format!("{}\n{}", line("orient", "orientation", "reads"), line("suspect", "orientation", "reads"));
        let authored = parse_cli_facts(&text);
        let m = [mirror("orient", "orientation", "reads"), mirror("suspect", "orientation", "reads")];
        let v = cli_surface_violations(&authored, &m, &["orient"], &["suspect"]);
        // `suspect` is a lens in the dispatch but declared as a command: undispatched as a command AND
        // the lens has no `lens` fact - two violations, both true.
        assert_eq!(v.len(), 2, "{v:?}");
    }

    #[test]
    fn a_mirror_that_drifts_from_the_facts_is_caught_field_by_field() {
        let authored = parse_cli_facts(&line("status", "orientation", "reads"));
        let m = [mirror("status", "orientation", "writes")];
        let v = cli_surface_violations(&authored, &m, &["status"], &[]);
        assert_eq!(v.len(), 1, "{v:?}");
        assert!(v[0].contains("effect differs") && v[0].contains("`reads`") && v[0].contains("`writes`"), "{v:?}");
    }

    /// issue547, the probe pair (D0388). KNOWN-POSITIVE: `cliX` superseded by `cliX2` of the same command
    /// name parses to ONE fact, the superseder's, and a mirror equal to it is clean. KNOWN-NEGATIVE: the
    /// same two lines with no edge are two facts in force of one name - a violation naming both parts.
    #[test]
    fn a_superseded_fact_is_not_in_force_and_two_live_facts_of_one_name_are_a_violation() {
        let old = format!("    part cliX : CliCommand {{ :>> id = \"a\"; {} }}", fact("x", "governance", "writes")).replace(":>> synopsis = \"s\";", ":>> synopsis = \"old\";");
        let new = format!("    part cliX2 : CliCommand {{ :>> id = \"b\"; {} }}", fact("x", "governance", "writes"));
        let with_edge = format!("{old}\n{new}\n    #Supersede dependency from cliX2 to cliX;\n");
        let authored = parse_cli_facts(&with_edge);
        assert_eq!(authored.len(), 1, "{authored:?}");
        assert_eq!((authored[0].part.as_str(), authored[0].synopsis.as_str()), ("cliX2", "s"));
        let m = [mirror("x", "governance", "writes")];
        assert!(cli_surface_violations(&authored, &m, &["x"], &[]).is_empty());

        let without = parse_cli_facts(&format!("{old}\n{new}\n"));
        assert_eq!(without.len(), 2);
        let v = cli_surface_violations(&without, &m, &["x"], &[]);
        assert!(v.iter().any(|l| l.contains("two CliCommand facts in force") && l.contains("`cliX`") && l.contains("`cliX2`")), "{v:?}");
    }

    /// issue548, the probe pair: a mirror whose INVOCATION differs from the fact is a violation naming the
    /// field (positive); the same mirror with the fact's invocation is clean (negative).
    #[test]
    fn an_invocation_that_drifts_between_the_two_homes_is_caught() {
        let text = format!("    part cliA : CliCommand {{ :>> id = \"i\"; {} :>> invocation = \"<d> (--words TEXT | --note TEXT)\"; }}", fact("accept", "governance", "writes"));
        let authored = parse_cli_facts(&text);
        let drifted = [keel_schema::cli_facts::CliFact { invocation: "<d> --note TEXT", ..mirror("accept", "governance", "writes") }];
        let v = cli_surface_violations(&authored, &drifted, &["accept"], &[]);
        assert_eq!(v.len(), 1, "{v:?}");
        assert!(v[0].contains("invocation differs") && v[0].contains("--words"), "{v:?}");
        let level = [keel_schema::cli_facts::CliFact { invocation: "<d> (--words TEXT | --note TEXT)", ..mirror("accept", "governance", "writes") }];
        assert!(cli_surface_violations(&authored, &level, &["accept"], &[]).is_empty());
    }

    #[test]
    fn the_live_facts_mirror_and_dispatch_agree() {
        let text = keel_model::corpus::read_to_string(crate::test_repo_root().join(".engine/cli/commands.sysml")).expect("the facts ship with the engine");
        let authored = parse_cli_facts(&text);
        assert_eq!(authored.len(), keel_schema::cli_facts::CLI_FACTS.len(), "every fact parsed");
        let v = cli_surface_violations(&authored, &keel_schema::cli_facts::CLI_FACTS, &keel_schema::cli_surface::COMMAND_NAMES, &keel_schema::cli_surface::LENS_NAMES);
        assert!(v.is_empty(), "{v:#?}");
    }

    /// issue423, the probe pair named in the definition of done (D0388): the KNOWN-POSITIVE is a synopsis citing
    /// D0353 - the push gate, retired by D0356 - and the KNOWN-NEGATIVE is the corrected `suite`
    /// synopsis in the live facts, which cites `D0356`. The retired citation names its superseder; an
    /// id with no file is its own class; an id that is present and live is not a violation.
    #[test]
    fn a_synopsis_citing_a_retired_or_absent_decision_is_a_violation_and_the_live_suite_synopsis_is_not() {
        let suite = keel_schema::cli_facts::CLI_FACTS.iter().find(|f| f.name == "suite").expect("the suite fact");
        assert!(suite.synopsis.contains("D0356"), "the known-negative cites D0356: {}", suite.synopsis);
        // Every Decision the LIVE synopsis cites is present: the known-negative is "cites only what is
        // in force", not "cites exactly D0356" - hardcoding {d0271, d0356} here went red in CI the moment
        // the synopsis gained a citation (D0421 on cab7cac, issue438).
        let mut present: BTreeSet<String> = decision_ids_cited(suite.synopsis).into_iter().collect();
        present.insert("d0271".to_string());
        let retired: BTreeMap<String, String> = std::iter::once(("d0353".to_string(), "d0356".to_string())).collect();
        let v = synopsis_citation_violations(
            &[
                ("fixture", "gate", "gates the push on the last green suite (D0353)"),
                ("cli_facts.rs", "suite", suite.synopsis),
                ("fixture", "ghost", "declared under d0999 which was never recorded"),
                ("fixture", "help", "renders from the facts (D0271)"),
            ],
            &present,
            &retired,
        );
        assert_eq!(v.len(), 2, "{v:#?}");
        assert!(v[0].contains("`gate`") && v[0].contains("d0353") && v[0].contains("RETIRED") && v[0].contains("d0356"), "{v:?}");
        assert!(v[1].contains("`ghost`") && v[1].contains("d0999") && v[1].contains("no Decision file"), "{v:?}");
    }

    #[test]
    fn decision_ids_are_cited_as_whole_tokens_only() {
        assert_eq!(decision_ids_cited("D0356 then d0345, D0356 again"), vec!["d0356", "d0345"]);
        assert!(decision_ids_cited("dcOneClick keeld0001 d03561 D035 d-0356").is_empty());
        assert_eq!(decision_ids_cited("(D0201 B)"), vec!["d0201"]);
    }

    /// The live tree: every id either fact home cites has a file and is not retired.
    #[test]
    fn the_live_synopses_cite_only_live_decisions() {
        let root = Path::new("../..");
        let text = keel_model::corpus::read_to_string(crate::test_repo_root().join(".engine/cli/commands.sysml")).expect("the facts ship with the engine");
        let authored = parse_cli_facts(&text);
        let mut synopses: Vec<(&str, &str, &str)> = authored.iter().map(|f| ("commands.sysml", f.name.as_str(), f.synopsis.as_str())).collect();
        synopses.extend(keel_schema::cli_facts::CLI_FACTS.iter().map(|f| ("cli_facts.rs", f.name, f.synopsis)));
        let retired: BTreeMap<String, String> = keel_model::corpus::supersede_edges(root).into_iter().map(|(from, to)| (to, from)).collect();
        assert!(retired.contains_key("d0353"), "the retired set holds the push gate");
        let v = synopsis_citation_violations(&synopses, &decision_ids_present(root), &retired);
        assert!(v.is_empty(), "{v:#?}");
    }

    /// issue433: the citation check reads only a tree that holds the engine's Decision channel
    /// (D0271 present). Known-negative: the engine's facts under `.engine/cli` with NO Decisions -
    /// the shape `keel init` ships and the shape the self-build-like fixtures take - yields no
    /// citation violation. Known-positive: the same tree with a `0271-` file yields the twelve.
    #[test]
    fn citations_are_read_only_where_the_engines_decisions_live() {
        let root = std::env::temp_dir().join(format!("kcite{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".engine").join("cli")).expect("mkdir");
        std::fs::create_dir_all(root.join(".engine").join("decisions")).expect("mkdir");
        std::fs::copy("../../.engine/cli/commands.sysml", root.join(".engine").join("cli").join("commands.sysml")).expect("copy the facts");
        let cites = |r: &GuardReport| r.violations.iter().filter(|v| v.contains(" cites ")).count();
        let without = cli_surface_declared(&root);
        assert_eq!(cites(&without), 0, "no Decisions in the tree: nothing to hold the citations against: {:#?}", without.violations);
        std::fs::write(root.join(".engine").join("decisions").join("0271-cliSurfaceIsAnAuthoredFact.sysml"), "package Decision0271 {}\n").expect("write");
        let with = cli_surface_declared(&root);
        assert!(cites(&with) >= 12, "the engine's channel is present and every other cited Decision is absent: {} citation violation(s)", cites(&with));
        let _ = std::fs::remove_dir_all(&root);
    }
}

#[cfg(test)]
mod viewpoint_enumeration_tests {
    use super::*;

    /// issue139: the guard must judge EVERY declared Viewpoint, not the ones in one hardcoded filename.
    /// Before the fix a viewpoint in any other `.engine/views` file was invisible — the guard reported
    /// "32 scanned, 0 violations" with a probe in the tree whose renderer named no command at all. This
    /// asserts the FAILING direction, because a guard only ever tested green is a guard nobody tested.
    #[test]
    fn a_viewpoint_outside_the_registry_file_is_still_judged() {
        let dir = keel_fs::scratch("keel_vp_enum_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".engine/views")).unwrap();
        std::fs::create_dir_all(dir.join(".tracking")).unwrap();
        let vp = |name: &str, id: &str, title: &str, renderer: &str| {
            format!(
                "package P{name} {{
    part {name} : Viewpoint {{
        :>> id = \"{id}\";
        :>> title = \"{title}\";
        :>> renderer = \"{renderer}\";
    }}
}}
"
            )
        };
        // one in the registry file with a real renderer, one in ANOTHER file with a command that does
        // not exist — the exact shape that used to pass.
        std::fs::write(dir.join(".engine/views/viewpoint-registry.sysml"), vp("goodVP", "aaaaaaaa-1111-4111-8111-111111111111", "good", "keel show orient")).unwrap();
        std::fs::write(dir.join(".engine/views/other.sysml"), vp("strayVP", "bbbbbbbb-2222-4222-8222-222222222222", "stray", "keel no-such-command")).unwrap();

        let r = viewpoint_renderer(&dir);
        assert_eq!(r.scanned, 2, "both viewpoints must be scanned, wherever they are declared");
        assert_eq!(r.violations.len(), 1, "the stray viewpoint's unknown renderer must be a violation; got {:?}", r.violations);
        assert!(r.violations[0].contains("stray"), "the violation must name the stray viewpoint; got {:?}", r.violations);

        // and the enumeration both readers share agrees
        let rows = keel_view::view::declared_viewpoints(&dir).unwrap();
        assert_eq!(rows.len(), 2, "one answer to what viewpoints exist");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
