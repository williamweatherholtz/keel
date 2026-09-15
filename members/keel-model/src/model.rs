//! The tracking model every reader runs over (D0074/D0075): items, typed edges, and the memoized
//! build from the parsed corpus. Out of `view/mod.rs` in sprint 718 (D0479) so the write API and the
//! frontier can read the graph without depending on a renderer.
//!
//! Regenerable cache, never truth (§2.1): the model is a projection of the `.sysml` files, rebuilt
//! whenever their fingerprint changes.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use keel_parser::ast::{Item, Package, Value};

#[derive(Debug, thiserror::Error)]
pub enum ViewError {
    #[error("view file not found: {0}")]
    NotFound(String),
    #[error("reading view file {0}: {1}")]
    Io(String, std::io::Error),
    #[error("invalid view TOML {0}: {1}")]
    Toml(String, Box<toml::de::Error>),
    #[error("parsing tracking file {0}: {1}")]
    Track(String, String),
    #[error("view '{view}' references unknown edge kind '{edge}' (known: {known})")]
    UnknownEdge { view: String, edge: String, known: String },
    #[error("unknown render mode '{0}' (expected: graph, table, review)")]
    UnknownMode(String),
    #[error("unknown report '{0}' (expected: assurance, traceability, quality-debt, flow, governance, friction)")]
    UnknownReport(String),
    #[error("invalid critique policy: {0}")]
    Policy(String),
    #[error("unknown element '{0}' (no authored item by that name)")]
    UnknownElement(String),
    #[error("section needs exactly one seed: a view name or an element name")]
    BadSection,
    #[error("element '{0}' is a {1}, not a Need (a boundary seed must be a Need)")]
    NotANeed(String, String),
}

#[derive(Clone)]
pub struct ItemInfo {
    pub type_name: String,
    pub attrs: HashMap<String, String>,
    pub marker: Option<String>,
    /// Repo-relative source file (forward-slashed) — powers the `newlyAdded` git-temporal rule scope
    /// (D0105). Empty for items constructed in tests / without a known source.
    pub file: String,
}

#[derive(Clone)]
pub struct Edge {
    pub kind: String,
    pub from: String,
    pub to: String,
}

/// The computed `displayLabel` view (schema §2.3 — declared but historically unbuilt; D0126). The human
/// label for an element: its authored `title` when present + non-blank, else the immutable `name`
/// identifier. Titles may duplicate (that is fine — identity is the `name`); `displayLabel` is never
/// stored, always computed here so every surface labels elements the same way.
#[must_use]
pub fn display_label(name: &str, info: &ItemInfo) -> String {
    match info.attrs.get("title") {
        Some(t) if !t.trim().is_empty() => t.clone(),
        _ => name.to_string(),
    }
}

#[derive(Clone)]
pub struct Model {
    pub items: HashMap<String, ItemInfo>,
    pub edges: Vec<Edge>,
}

/// The directories whose `.sysml` files ARE the model.
///
/// Authored instances live in `.tracking` and in the `.engine` INSTANCE dirs. Parsing is syntactic
/// (no import resolution), so `.engine` instance files parse standalone. Schema files are the
/// vocabulary rather than instances, and are excluded.
///
/// Shared rather than inlined in `Model::build`, because a check that resolves names against the
/// model must scan exactly what the model contains. Walking `.engine` wholesale instead made
/// `edge-endpoints` fire on `docs/tracking-template.sysml` — an authoring EXAMPLE whose
/// `exampleNeed` placeholders are undeclared on purpose. Four violations against a file the model
/// never loads is how a new guard teaches its reader to ignore it.
///
/// issue104: `.engine/workflows` belongs here. The six workflow definitions are the one place this
/// repo models behaviour in the base language (an `action def` with successions), so omitting them
/// left their flow edges nowhere to land.
#[must_use]
pub fn model_dirs(root: &Path) -> [std::path::PathBuf; 8] {
    [
        root.join(".tracking"),
        root.join(".knowledge"), // D0161: declared Questions/Aliases - absent dir = nothing declared
        root.join(".engine").join("decisions"),
        root.join(".engine").join("processes"),
        root.join(".engine").join("views"),
        root.join(".engine").join("skills"),
        root.join(".engine").join("rules"), // D0105: declared EdgeRule/ElementRule instances
        root.join(".engine").join("workflows"),
    ]
}

/// Known edge kinds (canonical, lowercase), DERIVED from the schema — never restated here.
///
/// This was a hardcoded list, and it had drifted from the schema in BOTH directions (issue119): it
/// rejected `derivedFrom`, `covers`, `dispositions` and `specialize`, which the schema declares and
/// which 166 edges in the model use, while accepting three kinds the schema never declared. A user
/// who read the schema and declared a viewpoint over `derivedFrom` was told the schema's own
/// vocabulary was unknown. Deriving makes that class of drift unrepresentable.
#[must_use]
pub fn known_edges() -> &'static std::collections::HashSet<String> {
    static K: std::sync::LazyLock<std::collections::HashSet<String>> =
        std::sync::LazyLock::new(keel_schema::schema::edge_kinds);
    &K
}

pub fn value_to_string(v: &Value) -> String {
    match v {
        Value::Str(s) | Value::Ident(s) => s.clone(),
        Value::Int(n) => n.to_string(),
        Value::EnumLit { member, .. } => member.clone(),
        // A multi-valued assignment rendered as a single scalar is a lossy view by nature. Joining
        // with ", " keeps every element VISIBLE rather than showing the first and dropping the rest,
        // which is the failure a caller would not notice. Callers needing the elements individually —
        // reference resolution, edge building — must match on `Value::Seq` directly; this function is
        // for display and for attribute lookups that are single-valued by schema.
        Value::Seq(items) => items.iter().map(value_to_string).collect::<Vec<_>>().join(", "),
    }
}

#[must_use]
pub fn edge_kind_from_marker(marker: &str) -> String {
    let m = marker.trim_start_matches('#');
    if m.is_empty() {
        "dependency".to_string()
    } else {
        m.to_lowercase()
    }
}

/// Process-global memo of the last-built model, keyed by a content fingerprint (perf: a serve
/// page-load burst fires ~8 views that each call `Model::build`; without this they each re-parse all
/// ~260 files — slow on I/O-heavy hosts, e.g. Windows Defender scanning each read). Regenerable cache,
/// never truth (§2.1) — invalidated automatically when any file changes.
static MODEL_CACHE: std::sync::Mutex<Option<(u64, std::sync::Arc<Model>)>> = std::sync::Mutex::new(None);
/// Serializes BUILDS so a concurrent cold burst does ONE parse (others wait, then hit the cache).
static MODEL_BUILD_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

impl Model {
    /// The cached model if its fingerprint matches `fp` - a shared handle, never a clone of the
    /// graph (dcOneCorpusPerProcess: forty guards used to copy ~1,160 files' worth of items each).
    fn cached_model(fp: u64) -> Option<std::sync::Arc<Self>> {
        MODEL_CACHE.lock().ok().and_then(|g| g.as_ref().filter(|(c, _)| *c == fp).map(|(_, m)| std::sync::Arc::clone(m)))
    }

    /// Build the model, MEMOIZED by content fingerprint (see [`MODEL_CACHE`]). A burst of concurrent
    /// callers on an unchanged model shares one parse; the cache invalidates on any file change.
    ///
    /// # Errors
    /// Returns [`ViewError::Track`] naming the first file that fails to parse.
    pub fn build(root: &Path) -> Result<std::sync::Arc<Self>, ViewError> {
        keel_perf::perf::add(&keel_perf::perf::BUILD_CALLS, 1);
        let fp = crate::fingerprint::of(root);
        if let Some(m) = Self::cached_model(fp) {
            keel_perf::perf::add(&keel_perf::perf::CACHE_HITS, 1);
            return Ok(m);
        }
        let _bl = MODEL_BUILD_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(m) = Self::cached_model(fp) {
            keel_perf::perf::add(&keel_perf::perf::CACHE_HITS, 1);
            return Ok(m); // another thread built it while we waited
        }
        let model = std::sync::Arc::new(keel_perf::perf::timed(&keel_perf::perf::PARSE_NANOS, || Self::build_uncached(root))?);
        if let Ok(mut g) = MODEL_CACHE.lock() {
            *g = Some((fp, std::sync::Arc::clone(&model)));
        }
        Ok(model)
    }

    /// The RETIRED set: every target of a `#Supersede` edge (D0384 option A / D0398, 2026-09-09).
    ///
    /// The edge is the ONLY authored mark of retirement - `DecisionStatus` has no `superseded` member
    /// since D0398 - so a Decision's standing is its `status` read TOGETHER with this set: a `proposed`
    /// Decision in it is not waiting, an `accepted` one in it is not in force. A `#SupersedeClause`
    /// edge reverses one clause and does NOT retire (d0149 -> d0129 leaves never-rebase standing).
    #[must_use]
    pub fn retired(&self) -> HashSet<String> {
        self.edges.iter().filter(|e| e.kind == "supersede").map(|e| e.to.clone()).collect()
    }

    /// The Decisions that STAND with `status` (`proposed` / `accepted` / `rejected`): they read that
    /// member and are not retired. Every queue, scorecard and governance scope reads its Decisions
    /// through this one predicate, so a retired Decision cannot reach a human's queue by any of them
    /// (issue396: d0353 sat on the queue for a day after d0356 retired it).
    ///
    /// Matches the enum-path suffix as well as the bare member: the authored value is
    /// `DecisionStatus::proposed`, and matching the full path would silently stop working if the enum
    /// were renamed, while the bare word alone would also catch a `counterproposed`.
    #[must_use]
    pub fn standing(&self, status: &str) -> HashSet<&String> {
        let retired = self.retired();
        let suffix = format!("::{status}");
        self.items
            .iter()
            .filter(|(n, i)| i.type_name == "Decision" && !retired.contains(n.as_str()))
            .filter(|(_, i)| i.attrs.get("status").is_some_and(|s| s.ends_with(&suffix) || s == status))
            .map(|(n, _)| n)
            .collect()
    }

    fn build_uncached(root: &Path) -> Result<Self, ViewError> {
        let n = model_dirs(root).iter().map(|d| crate::corpus::collect_sysml(d).len()).sum();
        Self::build_with_workers(root, Self::parse_workers(n))
    }

    /// The pool width for `n` files: sized as `guards::run_in_parallel` sizes its own, never wider
    /// than the work.
    fn parse_workers(n: usize) -> usize {
        std::thread::available_parallelism().map_or(4, std::num::NonZeroUsize::get).min(n.max(1))
    }

    /// The model from the corpus, parsed and ingested file by file across `workers` threads (`1` is
    /// the serial build) and REPLAYED in declaration order.
    ///
    /// issue443: the parse was the one serial floor every guard waited on. `corpus::parsed` makes each
    /// file an independent unit, and a file's ingest is a sequence of puts (last file wins) and
    /// put-if-absents (first wins) whose effect depends only on the order the sequences are applied in -
    /// so each worker records its file's sequence and the calling thread applies them in `paths` order.
    /// Items, edges and every view are what the serial build produced.
    ///
    /// # Errors
    /// Returns [`ViewError::Track`] naming the first file that fails to parse.
    pub fn build_with_workers(root: &Path, workers: usize) -> Result<Self, ViewError> {
        let paths: Vec<_> = model_dirs(root).iter().flat_map(|d| crate::corpus::collect_sysml(d)).collect();
        let ingested = keel_perf::perf::phase("model:parse-files", || Self::parse_all(root, &paths, workers));
        let mut items: HashMap<String, ItemInfo> = HashMap::new();
        let mut edges: Vec<Edge> = Vec::new();
        keel_perf::perf::phase("model:ingest", || -> Result<(), ViewError> {
            for file in ingested {
                let file = file?;
                for op in file.ops {
                    match op {
                        ItemOp::Put(name, info) => {
                            items.insert(name, info);
                        }
                        ItemOp::PutIfAbsent(name, info) => {
                            items.entry(name).or_insert(info);
                        }
                    }
                }
                edges.extend(file.edges);
            }
            Ok(())
        })?;
        // `resultof` edges: a TestResult named `<test>R<n>` records a run of Test `<test>` (gate or
        // DoD). The link is by naming convention, not a typed edge — derive it so result leaves
        // connect to their Test (which is itself `contains`-linked to its def).
        let resultofs: Vec<Edge> = items
            .iter()
            .filter(|(_, info)| info.type_name == "TestResult")
            .filter_map(|(name, _)| {
                let test = strip_result_suffix(name)?;
                items.contains_key(test).then(|| Edge { kind: "resultof".to_string(), from: name.clone(), to: test.to_string() })
            })
            .collect();
        edges.extend(resultofs);
        Ok(Self { items, edges })
    }

    /// One file's contribution to the model: its item ops and edges in the order `ingest` produced them.
    fn ingest_file(root: &Path, path: &Path) -> Result<FileIngest, ViewError> {
        let name = path.display().to_string();
        let pkg = crate::corpus::parsed(path).map_err(|e| match e {
            crate::corpus::ParseFailure::Io(e) => ViewError::Io(name.clone(), e),
            crate::corpus::ParseFailure::Lex(m) | crate::corpus::ParseFailure::Parse(m) => ViewError::Track(name.clone(), m),
        })?;
        // Repo-relative, forward-slashed path — matches `git diff --name-only` for `newlyAdded` scope.
        let rel = path.strip_prefix(root).unwrap_or(path).display().to_string().replace('\\', "/");
        let mut out = FileIngest { ops: Vec::new(), edges: Vec::new() };
        Self::ingest(&pkg, &mut out.ops, &mut out.edges, &rel);
        Ok(out)
    }

    /// Every path parsed and ingested, returned in `paths` order, across `workers` threads.
    ///
    /// A host that refuses a thread gets that worker's share done on the calling thread - slower,
    /// same answer. A slot no worker filled is reported as a failure of that file rather than
    /// silently dropped: a model missing a file it was asked for is a lie every view would repeat.
    fn parse_all(root: &Path, paths: &[std::path::PathBuf], workers: usize) -> Vec<Result<FileIngest, ViewError>> {
        if workers <= 1 || paths.len() <= 1 {
            return paths.iter().map(|p| Self::ingest_file(root, p)).collect();
        }
        let next = std::sync::atomic::AtomicUsize::new(0);
        let slots: Vec<std::sync::Mutex<Option<Result<FileIngest, ViewError>>>> = paths.iter().map(|_| std::sync::Mutex::new(None)).collect();
        std::thread::scope(|s| {
            for _ in 0..workers {
                let work = || loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let Some(path) = paths.get(i) else { break };
                    let done = Self::ingest_file(root, path);
                    if let Some(Ok(mut slot)) = slots.get(i).map(std::sync::Mutex::lock) {
                        *slot = Some(done);
                    }
                };
                // 16 MiB, as the guard pool: the parser recurses on nesting depth, and a spawned
                // thread's default stack is a quarter of the main thread's on some hosts.
                if std::thread::Builder::new().stack_size(16 * 1024 * 1024).spawn_scoped(s, work).is_err() {
                    work();
                }
            }
        });
        slots
            .into_iter()
            .zip(paths)
            .map(|(slot, path)| {
                slot.into_inner()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .unwrap_or_else(|| Err(ViewError::Track(path.display().to_string(), "parsed by no worker".to_string())))
            })
            .collect()
    }

    fn ingest(pkg: &Package, items: &mut Vec<ItemOp>, edges: &mut Vec<Edge>, file: &str) {
        for item in &pkg.items {
            match item {
                Item::Part(p) => add_item(items, &p.name, p.type_name.as_deref(), &p.attributes, p.marker.as_deref(), file),
                Item::Verification(v) => add_item(items, &v.name, v.type_name.as_deref(), &v.attributes, None, file),
                // issue102 construct 2/6: use case USAGES were skipped entirely, so all 56 were absent
                // from the model — no type, no attributes, unreachable by any view or trace. Ingested
                // exactly like any other typed item; the model keys on `type_name`, so they surface as
                // `UseCase` rather than as parts.
                Item::UseCase(u) => add_item(items, &u.name, u.type_name.as_deref(), &u.attributes, None, file),
                // D0143: a typed action usage is ingested like any other typed item. The Model keys on
                // `type_name`, so a retyped Process still surfaces as `Process`.
                Item::ActionUsage(a) => add_item(items, &a.name, a.type_name.as_deref(), &a.attributes, None, file),
                Item::ActionDecl(a) => add_item_typed(items, &a.name, "action", file),
                Item::ActionDef(ad) => {
                    add_item_typed(items, &ad.name, "ActionDef", file);
                    // `contains` edges: a def structurally owns its nested parts/verifications/actions.
                    // This containment is real structure the flat item map loses; the diagram draws it
                    // so the nested children connect to their def instead of floating.
                    for p in &ad.parts {
                        add_item(items, &p.name, p.type_name.as_deref(), &p.attributes, p.marker.as_deref(), file);
                        edges.push(Edge { kind: "contains".to_string(), from: ad.name.clone(), to: p.name.clone() });
                    }
                    for v in &ad.verifications {
                        add_item(items, &v.name, v.type_name.as_deref(), &v.attributes, None, file);
                        edges.push(Edge { kind: "contains".to_string(), from: ad.name.clone(), to: v.name.clone() });
                    }
                    for a in &ad.actions {
                        add_item_typed(items, &a.name, "action", file);
                        edges.push(Edge { kind: "contains".to_string(), from: ad.name.clone(), to: a.name.clone() });
                    }
                    for s in &ad.successions {
                        let kind = if s.is_ordering_only { "ordering" } else { "succession" };
                        edges.push(Edge { kind: kind.to_string(), from: s.first.clone(), to: s.then.clone() });
                    }
                    // `flow from A.out to B.in` (issue102): emit at the granularity the model actually
                    // has. Endpoints are dotted feature paths, and the model knows the ROOT (an action
                    // in this def) but not its ports, so the edge connects the roots. Taking the root
                    // rather than the whole path is what makes the edge resolvable at all; the full
                    // path stays in the AST for any consumer that later gains feature resolution.
                    for f in &ad.flows {
                        let root = |s: &str| s.split('.').next().unwrap_or(s).to_string();
                        edges.push(Edge { kind: "flow".to_string(), from: root(&f.from), to: root(&f.to) });
                    }
                }
                Item::Satisfy(e) => edges.push(Edge { kind: "satisfy".to_string(), from: e.need.clone(), to: e.by.clone() }),
                Item::Allocate(e) => edges.push(Edge { kind: "allocate".to_string(), from: e.sr.clone(), to: e.to.clone() }),
                Item::Dependency(d) => edges.push(Edge { kind: edge_kind_from_marker(&d.marker), from: d.from.clone(), to: d.to.clone() }),
                Item::Succession(s) => {
                    let kind = if s.is_ordering_only { "ordering" } else { "succession" };
                    edges.push(Edge { kind: kind.to_string(), from: s.first.clone(), to: s.then.clone() });
                }
                Item::Import(_) | Item::TypeDef(_) | Item::EnumDef(_) => {}
            }
        }
        // `contains` for Process -> its ProcessSteps. Steps are authored as siblings of the Process
        // in the same package (not AST-nested), so link by co-membership: every ProcessStep in this
        // package belongs to the Process(es) declared in it.
        let processes: Vec<&str> = pkg
            .items
            .iter()
            .filter_map(|i| match i {
                Item::Part(p) if p.type_name.as_deref() == Some("Process") => Some(p.name.as_str()),
                _ => None,
            })
            .collect();
        if !processes.is_empty() {
            for item in &pkg.items {
                if let Item::Part(p) = item {
                    if p.type_name.as_deref() == Some("ProcessStep") {
                        for proc in &processes {
                            edges.push(Edge { kind: "contains".to_string(), from: (*proc).to_string(), to: p.name.clone() });
                        }
                    }
                }
            }
        }
    }
}

/// One step of a file's ingest, recorded where it is produced and applied in declaration order
/// (issue443). `Put` is the typed item with its attributes - the last file to declare a name wins;
/// `PutIfAbsent` is a bare declaration (`action x;`, an `action def`) that never overwrites a typed one.
enum ItemOp {
    Put(String, ItemInfo),
    PutIfAbsent(String, ItemInfo),
}

/// What one file contributes to the model, in the order `Model::ingest` produced it.
struct FileIngest {
    ops: Vec<ItemOp>,
    edges: Vec<Edge>,
}

fn add_item(items: &mut Vec<ItemOp>, name: &str, type_name: Option<&str>, attributes: &[keel_parser::ast::Attribute], marker: Option<&str>, file: &str) {
    let attrs = attributes.iter().map(|a| (a.name.clone(), value_to_string(&a.value))).collect();
    items.push(ItemOp::Put(name.to_string(), ItemInfo { type_name: type_name.unwrap_or("").to_string(), attrs, marker: marker.map(str::to_string), file: file.to_string() }));
}

fn add_item_typed(items: &mut Vec<ItemOp>, name: &str, type_name: &str, file: &str) {
    items.push(ItemOp::PutIfAbsent(name.to_string(), ItemInfo { type_name: type_name.to_string(), attrs: HashMap::new(), marker: None, file: file.to_string() }));
}

/// Strip a `R<digits>` result suffix: `storyDiagramRenderFixDoDR1` -> `storyDiagramRenderFixDoD`.
/// Returns `None` when the name does not end in `R` followed by one or more digits.
#[must_use]
pub fn strip_result_suffix(name: &str) -> Option<&str> {
    let idx = name.rfind('R')?;
    let (head, tail) = name.split_at(idx);
    let digits = tail.get(1..)?;
    (!head.is_empty() && !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())).then_some(head)
}

#[must_use]
pub fn has_outgoing(edges: &[Edge], name: &str, kind: &str) -> bool {
    edges.iter().any(|e| e.from == name && e.kind == kind)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn pooled_build_is_the_serial_build() {
        // issue443: the corpus parses and ingests across a pool and the model must be what the serial
        // build produced - the same items with the same fields and source files, the edges in the same
        // order, and the same winner where two files declare one name (last typed declaration wins,
        // a bare `action x;` never overwrites). Seven files whose sorted order differs from the order
        // they were written in; two of them collide on `shared`.
        let dir = std::env::temp_dir().join(format!("keel_ppar_{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        let tracking = dir.join(".tracking");
        std::fs::create_dir_all(&tracking).unwrap();
        for (i, tag) in ["g", "a", "f", "b", "e", "c", "d"].iter().enumerate() {
            let body = format!(
                concat!(
                    "package P{tag} {{\n",
                    "    part n{tag} : Need {{ :>> id = \"{i}\"; :>> title = \"{tag}\"; }}\n",
                    "    part sr{tag} : SystemRequirement {{ :>> id = \"{i}0\"; }}\n",
                    "    part shared : Need {{ :>> title = \"from {tag}\"; }}\n",
                    "    action def D{tag} {{ action shared; action step{tag}; }}\n",
                    "    satisfy n{tag} by sr{tag};\n",
                    "}}\n"
                ),
                tag = tag,
                i = i
            );
            std::fs::write(tracking.join(format!("{tag}.sysml")), body).unwrap();
        }
        let serial = Model::build_with_workers(&dir, 1).unwrap();
        let pooled = Model::build_with_workers(&dir, 4).unwrap();
        let edges = |m: &Model| m.edges.iter().map(|e| format!("{} {} {}", e.kind, e.from, e.to)).collect::<Vec<_>>();
        assert_eq!(edges(&pooled), edges(&serial));
        assert!(edges(&serial).len() >= 7 * 3, "{:?}", edges(&serial));
        assert_eq!(pooled.items.len(), serial.items.len());
        for (name, want) in &serial.items {
            let got = &pooled.items[name];
            assert_eq!((&got.type_name, &got.attrs, &got.marker, &got.file), (&want.type_name, &want.attrs, &want.marker, &want.file), "{name}");
        }
        // The collision resolves as it always did: the last file in sorted order (g.sysml) wins the typed
        // `shared`, and the bare `action shared;` declarations never displaced it.
        assert_eq!(pooled.items["shared"].type_name, "Need");
        assert_eq!(pooled.items["shared"].attrs["title"], "from g");
        assert_eq!(pooled.items["shared"].file, ".tracking/g.sysml");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn standing_is_the_status_minus_the_edge_and_a_clause_reversal_keeps_its_target() {
        // D0398 (D0384 option A): `#Supersede` retires its target WHOLE; `#SupersedeClause` reverses one
        // clause and leaves the target in force. Both targets keep `status = accepted` — the field is
        // never rewritten — so the reader's predicate is status AND no incoming whole-supersede edge.
        let with_status = |status: &str| {
            let mut a = HashMap::new();
            a.insert("status".to_string(), status.to_string());
            ItemInfo { type_name: "Decision".to_string(), attrs: a, marker: None, file: String::new() }
        };
        let mut items = HashMap::new();
        for d in ["dWhole", "dClause", "dNewer", "dRetiredWhileProposed"] {
            items.insert(d.to_string(), with_status(if d == "dRetiredWhileProposed" { "DecisionStatus::proposed" } else { "DecisionStatus::accepted" }));
        }
        let edges = vec![
            Edge { kind: "supersede".to_string(), from: "dNewer".to_string(), to: "dWhole".to_string() },
            Edge { kind: "supersedeclause".to_string(), from: "dNewer".to_string(), to: "dClause".to_string() },
            Edge { kind: "supersede".to_string(), from: "dNewer".to_string(), to: "dRetiredWhileProposed".to_string() },
        ];
        let model = Model { items, edges };
        let retired = model.retired();
        assert!(retired.contains("dWhole") && retired.contains("dRetiredWhileProposed") && !retired.contains("dClause"));
        let accepted = model.standing("accepted");
        assert!(accepted.contains(&"dClause".to_string()), "a clause reversal leaves its target in force");
        assert!(accepted.contains(&"dNewer".to_string()));
        assert!(!accepted.contains(&"dWhole".to_string()), "retired whole: out of the accepted set whatever the field says");
        assert!(model.standing("proposed").is_empty(), "a proposed Decision retired by the edge is not waiting on anyone");
    }

}
