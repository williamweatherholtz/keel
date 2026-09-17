"""split_guards.py - the sprint 733 transform: keel-cli/src/guards.rs becomes member keel-guards.

not-an-instrument: it is a one-shot codemod (the migration skill's gate 1); its reconciliation totals are the
transform checking itself before it writes, not a measure of the project - the guard-source lock and the
coverage test in members/keel-guards/src/lib.rs are the sensors the split answers to.

One committed script, dry-run by default (D0479 one extraction per sprint; the migration skill's gate 1).
It reads guards.rs as a sequence of TOP-LEVEL ITEMS (a brace-matched scan that skips strings, chars
and comments, so a `{` inside a format string is not a block), assigns every item a HOME - a family
module or the crate root - and writes members/keel-guards/src/{lib.rs,<family>.rs}. Nothing is
retyped: every item's text is carried byte-for-byte except the two mechanical rewrites the report
counts (`crate::<module>` paths per MODULE_MAP; a private top-level item in a family file becomes
`pub(crate)` so the root and the sibling families can still name it through `use super::*`).

    python scripts/split_guards.py inventory      # every top-level item, its kind, name, span, home
    python scripts/split_guards.py plan           # the per-family assignment + the reconciliation totals
    python scripts/split_guards.py --apply        # write the member; guards.rs is NOT deleted here
                                                  # (git mv + the re-export layer are the sprint's hands)

The reconciliation (migration skill gate 2) is printed either way and the apply REFUSES when it does
not balance: items before == items after, functions before == after, test modules before == after,
and every guard name in run_one's match lands in exactly one family table.
"""
from __future__ import annotations

import hashlib
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
SRC = REPO / "keel-cli" / "src" / "guards.rs"
DST = REPO / "members" / "keel-guards" / "src"

# ── the family table: guard name -> family module ─────────────────────────────────────────────
# Defined this sprint (the DoD's "families guards.md already groups by" was a TIER grouping; the
# retro records that). A guard is in exactly one family; the union is held equal to GUARD_NAMES by
# the test the apply writes into lib.rs.
FAMILIES: dict[str, list[str]] = {
    "identity": [
        "actors", "duplicate-identity", "identity-present", "identity-well-formed", "id-is-a-uuid",
        "ownership", "enrollment-binding", "edge-endpoints", "type-collision", "attribute-vocabulary",
        "marker-vocabulary", "sequence-multiplicity", "parser-coverage", "engine-lint",
        "base-first-justification",
    ],
    "acceptance": [
        "acceptance-events", "confirmation-authenticity", "attestation-substance", "attestation-authority",
        "acceptance-binds-to-text", "evidence-cited", "impossible-evidence-date", "claim-ancestry",
        "consent-scope", "direction-cited", "plan-covers-step", "judgment-request-quality",
    ],
    "decisions": [
        "decision-rationale", "decision-requirement-link", "decision-amends-process", "decision-scaffolding",
        "requirement-rootedness", "verification-trace",
    ],
    "sprints": [
        "sprint-coverage", "sprint-closure", "ceremony", "charter", "retro-backlog", "stale-gate-prose",
        "scaffold-placeholder", "priority-inversion",
    ],
    "issues": [
        "issues", "resolver-kind", "control-defect-registry", "untrusted-routing", "untrusted-taint",
    ],
    "enforcement": [
        "process-change", "doc-sync", "doc-guard-count", "control-map-reconciled", "control-event-coverage",
        "activation-manifest", "process-skill", "process-applicability", "step-check-resolves", "stpa-currency",
        "instruments-declared", "defect-guard-coverage", "hook-config-integrity", "claude-surface-drift",
        "unit-extras-present", "manifest-key-portability", "manifest-coverage",
    ],
    "surface": [
        "cli-surface-declared", "cli-reference", "tool-reference", "viewpoint-renderer", "question-coverage",
    ],
    "release": [
        "gating-workflow-history", "gate-environment-parity", "release-checksums-published",
        "wrapper-pin-checksummed", "release-recorded", "custom-harness-routed", "working-tree-eol",
    ],
    "critique": ["critique", "critique-rigor", "critic-independence", "assured"],
}

# Runnable through `run_one` but not in GUARD_NAMES (enforced): the union test treats these apart.
RUNNABLE_ONLY = ["critique", "assured", "critique-rigor", "defect-guard-coverage"]

# ── the module map: where each `crate::<module>` of keel-cli lives now ────────────────────────
MODULE_MAP: dict[str, str] = {
    "corpus": "keel_model::corpus", "collect_sysml": "keel_model::corpus::collect_sysml",
    "collect_sysml_uncached": "keel_model::corpus::collect_sysml_uncached",
    "parse_pkg": "keel_model::corpus::parse_pkg", "supersede_targets": "keel_model::corpus::supersede_targets",
    "supersede_edges": "keel_model::corpus::supersede_edges", "CheckError": "keel_model::corpus::CheckError",
    "algo": "keel_model::algo", "activation": "keel_model::activation", "textscan": "keel_model::textscan",
    "orient": "keel_model::orient", "gitfacts": "keel_model::gitfacts", "ident": "keel_model::ident",
    "fingerprint": "keel_model::fingerprint", "done": "keel_model::done", "suspect": "keel_model::suspect",
    "binding": "keel_model::binding", "onboard": "keel_model::onboard", "indexer": "keel_model::indexer",
    "model": "keel_model::model", "queries": "keel_model::queries", "claims": "keel_model::claims",
    "evidence": "keel_model::evidence",
    "gitx": "keel_git::gitx", "eol": "keel_git::eol",
    "view": "keel_view::view", "priority": "keel_view::priority", "govern": "keel_view::govern",
    "pm": "keel_view::pm", "arch": "keel_view::arch", "control_proof": "keel_view::control_proof",
    "cli_facts": "keel_schema::cli_facts", "cli_surface": "keel_schema::cli_surface",
    "schema": "keel_schema::schema", "embedded": "keel_schema::embedded",
    "control_defects": "keel_schema::control_defects", "guard_names": "keel_schema::guard_names",
    "color": "keel_json::color", "json": "keel_json::json",
    "claim": "keel_write::claim", "write": "keel_write::write", "scaffold": "keel_write::scaffold",
    "claude_surface": "keel_write::claude_surface", "reverify": "keel_write::reverify",
    "intake_write": "keel_write::intake_write", "pin_skew": "keel_write::pin_skew",
    "github": "keel_github::github", "actor": "keel_actor::actor", "device": "keel_actor::device",
    "perf": "keel_perf::perf",
    # keel-cli modules the guards reached into whose function descended (sprint 733):
    "suite::is_self_build": "keel_model::corpus::is_self_build",
    "touched::custom_harness_tests": "keel_model::corpus::custom_harness_tests",
    "migrate::resync_text": "keel_schema::embedded::resync_text",
    "migrate::is_portable_engine_tool": "keel_schema::embedded::is_portable_engine_tool",
    "deck::marker_text_without_marker": "keel_model::textscan::marker_text_without_marker",
    "deck::NOT_A_PROCESS_CHANGE": "keel_model::textscan::NOT_A_PROCESS_CHANGE",
    "deck::marker_words": "keel_model::textscan::marker_words",
    # stays `crate::` inside keel-guards (rides with the member):
    # plan_cover, hardening, receipt, contentkey, guards (the re-export of GUARD_NAMES)
}
CRATE_LOCAL = {"plan_cover", "hardening", "receipt", "contentkey", "binary"}  # `binary`: a doc line names it (guards.rs 5874)
MODULE_MAP["guards"] = "crate"  # inside the member the guards ARE the crate root: `crate::guards::x` -> `crate::x`

ITEM_START = re.compile(
    r"^(pub(\([a-z]+\))?\s+)?(fn|const|static|struct|enum|type|trait|mod|use|impl|macro_rules!)\b"
)
NAME_OF = re.compile(
    r"^(?:pub(?:\([a-z]+\))?\s+)?(?:(?:unsafe|const|async)\s+)*(fn|const|static|struct|enum|type|trait|mod|use|impl)\s+(?:<[^>]*>\s*)?([A-Za-z_][A-Za-z0-9_:{}, *]*)"
)


class Item:
    def __init__(self, kind: str, name: str, start: int, end: int, lines: list[str]):
        self.kind, self.name, self.start, self.end, self.lines = kind, name, start, end, lines
        self.home: str | None = None

    @property
    def text(self) -> str:
        return "".join(self.lines)

    def __repr__(self) -> str:
        return f"{self.kind} {self.name} [{self.start}-{self.end}]"


def strip_code(line: str, state: dict) -> str:
    """The line with strings, chars and comments blanked - what the brace counter reads.

    `state["block"]` carries an open `/* */` across lines; a string never spans lines in guards.rs
    except raw strings, carried by `state["raw"]` (the closing hash count)."""
    out = []
    i, n = 0, len(line)
    while i < n:
        if state.get("block"):
            j = line.find("*/", i)
            if j < 0:
                return "".join(out)
            state["block"] = False
            i = j + 2
            continue
        if state.get("raw") is not None:
            close = '"' + "#" * state["raw"]
            j = line.find(close, i)
            if j < 0:
                return "".join(out)
            state["raw"] = None
            i = j + len(close)
            continue
        if state.get("str_open"):
            # a plain string that began on an earlier line: skip to its closing quote
            while i < n:
                if line[i] == "\\":
                    i += 2
                    continue
                if line[i] == '"':
                    i += 1
                    state["str_open"] = False
                    break
                i += 1
            if state.get("str_open"):
                return "".join(out)
            continue
        c = line[i]
        if line.startswith("//", i):
            return "".join(out)
        if line.startswith("/*", i):
            state["block"] = True
            i += 2
            continue
        m = re.match(r'b?r(#*)"', line[i:])
        if m:
            state["raw"] = len(m.group(1))
            i += m.end()
            continue
        if c == '"' or line.startswith('b"', i):
            i += 2 if c == "b" else 1
            while i < n:
                if line[i] == "\\":
                    i += 2
                    continue
                if line[i] == '"':
                    i += 1
                    break
                i += 1
            else:
                state["str_open"] = True  # a plain string continued on the next line
            continue
        if c == "'":
            m2 = re.match(r"'(\\.|\\x[0-9a-fA-F]{2}|\\u\{[0-9a-fA-F]+\}|[^\\'])'", line[i:])
            if m2:
                i += m2.end()
                continue
        out.append(c)
        i += 1
    return "".join(out)


def parse_items(lines: list[str]) -> list[Item]:
    """Top-level items with their leading trivia (doc comments, attributes, comments with no blank line between)."""
    items: list[Item] = []
    i, n = 0, len(lines)
    pending_start: int | None = None  # first line of leading trivia
    state: dict = {}
    while i < n:
        line = lines[i]
        stripped = line.rstrip("\n")
        if stripped.startswith("//!"):
            # the file's own module doc: replaced by the member's, never attached to an item
            pending_start = None
            i += 1
            continue
        if stripped.strip() == "":
            # a blank line ends leading trivia UNLESS it is all plain `//` comment - a section banner
            # (`// ── actors guard ... ──`) sits one blank line above its first item and rides with it
            if pending_start is not None and not all(
                lines[k].strip() == "" or (lines[k].startswith("//") and not lines[k].startswith("///"))
                for k in range(pending_start, i)
            ):
                pending_start = None
            i += 1
            continue
        if stripped.startswith("//") or stripped.startswith("#["):
            if pending_start is None:
                pending_start = i
            i += 1
            continue
        if not ITEM_START.match(stripped):
            raise SystemExit(f"{SRC}:{i + 1}: not a top-level item start: {stripped[:80]!r}")
        start = pending_start if pending_start is not None else i
        m = NAME_OF.match(stripped)
        kind = m.group(1)
        name = m.group(2).strip()
        # span: to the line where depth returns to 0 having seen a brace, or a `;` at depth 0
        depth = 0
        seen_brace = False
        j = i
        state = {}
        while j < n:
            code = strip_code(lines[j], state)
            for ch in code:
                if ch == "{":
                    depth += 1
                    seen_brace = True
                elif ch == "}":
                    depth -= 1
            if depth == 0 and (seen_brace or code.rstrip().endswith(";")):
                break
            j += 1
        if kind in ("fn", "const", "static", "struct", "enum", "type", "trait", "mod"):
            # the bare identifier: `GuardReport {`, `HISTORY_PREFIX: &str`, `nearest_attr<'a>(` all trim to it
            name = re.match(r"[A-Za-z_][A-Za-z0-9_]*", name).group(0)
        if kind == "use":
            name = name.split(";")[0].strip()
        if kind == "impl":
            name = "impl " + stripped[len("impl"):].split("{")[0].strip()
        items.append(Item(kind, name, start + 1, j + 1, lines[start:j + 1]))
        pending_start = None
        i = j + 1
    return items


IDENT = re.compile(r"\b[A-Za-z_][A-Za-z0-9_]*\b")


def guard_arms(items: list[Item]) -> list[tuple[str, str, str]]:
    """`(guard name, function, trailing comment)` for every arm of run_one's match, in arm order."""
    run_one = next(it for it in items if it.kind == "fn" and it.name.startswith("run_one"))
    arms = []
    for line in run_one.lines:
        m = re.match(r'\s*"([a-z0-9-]+)" => Some\(([a-z_0-9]+)\(root\)\),(.*)$', line)
        if m:
            arms.append((m.group(1), m.group(2), m.group(3).strip()))
    return arms


def assign_homes(items: list[Item]) -> tuple[dict[str, str], list[str]]:
    """Every item's home. Guard functions by the family table; a helper, type or impl goes with the ONE
    family that references it (transitively), else stays in the root; a test module goes with the
    single family its `use super::` names, else the root."""
    notes: list[str] = []
    arms = guard_arms(items)
    fn_family: dict[str, str] = {}
    fam_of_guard = {g: f for f, gs in FAMILIES.items() for g in gs}
    for guard, fn, _ in arms:
        fam = fam_of_guard.get(guard)
        if fam is None:
            raise SystemExit(f"run_one arm {guard!r} is in no family")
        fn_family[fn] = fam
    by_name: dict[str, Item] = {}
    for it in items:
        if it.kind in ("fn", "const", "static", "struct", "enum", "type", "trait"):
            by_name[it.name] = it
    home: dict[int, str] = {}
    root_kinds_fixed = set()
    for it in items:
        if it.kind == "fn" and it.name in fn_family:
            home[id(it)] = fn_family[it.name]
        elif it.kind == "use":
            home[id(it)] = "root"
        elif it.kind == "fn" and it.name in ("run_one", "run_all", "run_all_timed"):
            home[id(it)] = "root"
    # reference graph over named items (identifiers in the body, minus the item's own name)
    refs: dict[int, set[str]] = {}
    for it in items:
        body_ids = set(IDENT.findall("".join(l for l in it.lines if not l.lstrip().startswith("//"))))
        refs[id(it)] = {x for x in body_ids if x in by_name and x != it.name}
    # impl blocks belong with their type
    type_of_impl: dict[int, str] = {}
    for it in items:
        if it.kind == "impl":
            m = re.search(r"\bfor\s+([A-Za-z_][A-Za-z0-9_]*)|impl(?:<[^>]*>)?\s+([A-Za-z_][A-Za-z0-9_]*)", it.name)
            t = (m.group(1) or m.group(2)) if m else None
            if t and t in by_name:
                type_of_impl[id(it)] = t
    referrers_of = {
        id(it): [o for o in items if o is not it and o.kind != "mod" and it.name in refs[id(o)]] for it in items
    }
    # an impl's referrers are its type's: the pair moves together
    for it in items:
        if it.kind == "impl" and id(it) in type_of_impl:
            referrers_of[id(it)] = []

    def decide(it: Item, referrers: list[Item]) -> str:
        homes = {home[id(o)] for o in referrers if id(o) in home}
        return homes.pop() if len(homes) == 1 else "root"

    undecided = [it for it in items if id(it) not in home and it.kind != "mod"]
    while undecided:
        progressed = False
        for it in list(undecided):
            if it.kind == "impl":
                t = type_of_impl.get(id(it))
                if t and id(by_name[t]) in home:
                    home[id(it)] = home[id(by_name[t])]
                    undecided.remove(it)
                    progressed = True
                elif not t:
                    home[id(it)] = "root"
                    undecided.remove(it)
                    progressed = True
                continue
            referrers = referrers_of[id(it)]
            if not referrers:
                home[id(it)] = "root"  # reached only from outside the file (pub API) or dead
                notes.append(f"{it}: referenced by no named item -> root")
                undecided.remove(it)
                progressed = True
            elif all(id(o) in home for o in referrers):
                home[id(it)] = decide(it, referrers)
                undecided.remove(it)
                progressed = True
        if progressed:
            continue
        # a cycle among helpers (or a type whose impl is its only referrer): decide the one with the most
        # decided referrers from those, then let the rest propagate
        it = max((x for x in undecided if x.kind != "impl"), key=lambda x: sum(id(o) in home for o in referrers_of[id(x)]))
        home[id(it)] = decide(it, referrers_of[id(it)])
        notes.append(f"{it}: in a reference cycle -> {home[id(it)]} (from its decided referrers)")
        undecided.remove(it)
    # a type whose fields another home constructs or reads must stay where every referrer can see it:
    # a private field is module-private, so a struct/enum with any referrer outside its home goes to root.
    # (Handled by the single-home rule above: a type referenced from two homes is already root.)
    # test modules: with the ONE family whose items they name (`use super::{..}` or inline `super::f`);
    # a module naming items of two families, or only root's, stays in the root
    for it in items:
        if it.kind != "mod":
            continue
        named = {home[id(by_name[x])] for x in refs[id(it)]}
        families = named - {"root"}
        home[id(it)] = families.pop() if len(families) == 1 else "root"
        if len(named) > 1:
            notes.append(f"{it}: names items of several homes {sorted(named)} -> {home[id(it)]}")
    for it in items:
        it.home = home[id(it)]
    return {it.name: it.home for it in items}, notes


# The member sits one directory deeper than keel-cli: every manifest-relative path to the repo root
# gains a `..`. Counted in the report like the crate:: rewrites.
DEPTH_REWRITES: list[tuple[str, str]] = [
    ('concat!(env!("CARGO_MANIFEST_DIR"), "/..', 'concat!(env!("CARGO_MANIFEST_DIR"), "/../..'),
    ('env!("CARGO_MANIFEST_DIR")).join("..")', 'env!("CARGO_MANIFEST_DIR")).join("../..")'),
    ('env!("CARGO_MANIFEST_DIR")).parent().unwrap()', 'env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap()'),
    # cargo runs a package's tests from the package dir: the cwd-relative literals gain a `..` too.
    # WRONG FIX, kept as the record of what ran: a deeper literal is still a cwd anchor, and the control
    # (keel-cli/src/touched.rs no_member_test_anchors_on_a_cwd_relative_path) refused the ten it produced;
    # the landed tree resolves them through keel-guards' `test_repo_root()` (manifest ancestors, issue583).
    ('Path::new("..")', 'Path::new("../..")'),
    ('read_to_string("../.engine/', 'read_to_string("../../.engine/'),
    ('std::fs::copy("../.engine/', 'std::fs::copy("../../.engine/'),
    # the hardening census reads keel-cli's own sources; the grandfather test reads the identity family, where its set now lives
    ('read_to_string("src/main.rs")', 'read_to_string("../../keel-cli/src/main.rs")'),
    ('read_to_string("src/serve.rs")', 'read_to_string("../../keel-cli/src/serve.rs")'),
    ('read_to_string("src/guards.rs").expect("guards.rs is readable")', 'read_to_string("src/identity.rs").expect("identity.rs is readable")'),
]


def rewrite_crate_paths(text: str) -> tuple[str, Counter]:
    counts: Counter = Counter()
    for old, new in DEPTH_REWRITES:
        n = text.count(old)
        if n:
            counts["depth:" + old[-24:]] += n
            text = text.replace(old, new)

    def sub(m: re.Match) -> str:
        path = m.group(1)
        # longest key first: `suite::is_self_build` before `suite`
        for key in sorted(MODULE_MAP, key=len, reverse=True):
            if path == key or path.startswith(key + "::"):
                counts[key] += 1
                return MODULE_MAP[key] + path[len(key):]
        head = path.split("::")[0]
        if head in CRATE_LOCAL or head == "GUARD_NAMES":
            counts["crate-local:" + head] += 1
            return "crate::" + path
        counts["UNMAPPED:" + head] += 1
        return "crate::" + path

    return re.sub(r"\bcrate::([A-Za-z_][A-Za-z0-9_:]*)", sub, text), counts


PRIVATE_ITEM = re.compile(r"^(fn|const|static|struct|enum|type|trait)\s", re.M)


def publish_private(text: str) -> tuple[str, int]:
    """A private top-level item in a family file becomes `pub(crate)`; only the item's own header line
    (column 0) is touched, so nothing indented inside a body changes."""
    n = 0
    out = []
    for line in text.splitlines(keepends=True):
        if PRIVATE_ITEM.match(line):
            out.append("pub(crate) " + line)
            n += 1
        else:
            out.append(line)
    return "".join(out), n


def main(argv: list[str]) -> int:
    mode = argv[0] if argv else "plan"
    if mode == "rewrite-file":
        # the four modules that ride with the guards (receipt, contentkey, hardening, plan_cover): the same
        # crate:: and depth rewrites, in place, counted
        for f in argv[1:]:
            p = Path(f)
            text, counts = rewrite_crate_paths(p.read_text(encoding="utf-8", newline=""))
            p.write_text(text, encoding="utf-8", newline="")
            print(f"{f}: {dict(counts)}")
        return 0
    lines = SRC.read_text(encoding="utf-8", newline="").splitlines(keepends=True)
    items = parse_items(lines)
    homes, notes = assign_homes(items)
    arms = guard_arms(items)
    if mode == "inventory":
        for it in items:
            print(f"{it.start:5d}-{it.end:5d} {it.home:12s} {it.kind:6s} {it.name}")
        return 0
    per_home: dict[str, list[Item]] = defaultdict(list)
    for it in items:
        per_home[it.home].append(it)
    before = Counter(it.kind for it in items)
    print("== reconciliation ==")
    print(f"items: {len(items)}  fns: {before['fn']}  test mods: {before['mod']}  uses: {before['use']}")
    covered_lines = sum(len(it.lines) for it in items)
    print(f"lines covered by items: {covered_lines} of {len(lines)} (the rest are blank separators)")
    print(f"run_one arms: {len(arms)}; families: {len(FAMILIES)}; names in tables: {sum(len(v) for v in FAMILIES.values())}")
    table_names = [g for gs in FAMILIES.values() for g in gs]
    arm_names = [a[0] for a in arms]
    missing = sorted(set(arm_names) - set(table_names))
    extra = sorted(set(table_names) - set(arm_names))
    dup = [n for n, c in Counter(table_names).items() if c > 1]
    print(f"arms not in a family: {missing}; table names with no arm: {extra}; duplicates: {dup}")
    for h in ["root"] + list(FAMILIES):
        its = per_home.get(h, [])
        fns = sum(1 for it in its if it.kind == "fn")
        mods = sum(1 for it in its if it.kind == "mod")
        print(f"  {h:12s} items={len(its):4d} fns={fns:3d} tests={mods:2d} lines={sum(len(it.lines) for it in its):5d}")
    for n in notes:
        print("  note:", n)
    balanced = not missing and not extra and not dup and sum(len(v) for v in per_home.values()) == len(items)
    print("BALANCED" if balanced else "NOT BALANCED")
    if mode != "--apply":
        return 0 if balanced else 1
    if not balanced:
        raise SystemExit("apply refused: the reconciliation does not balance")
    DST.mkdir(parents=True, exist_ok=True)
    total_rewrites: Counter = Counter()
    total_pub = 0
    written_fns = 0
    written_mods = 0
    fam_of_guard = {g: f for f, gs in FAMILIES.items() for g in gs}
    guard_index = {g: i for i, g in enumerate(read_guard_names())}
    for fam in FAMILIES:
        its = per_home.get(fam, [])
        # code first in its original order, the test modules after it in theirs: clippy's
        # items_after_test_module flags any item that follows a file's only test module
        ordered = [it for it in its if it.kind != "mod"] + [it for it in its if it.kind == "mod"]
        body = "\n".join(it.text.rstrip("\n") + "\n" for it in ordered)
        body, counts = rewrite_crate_paths(body)
        total_rewrites += counts
        body, n_pub = publish_private(body)
        total_pub += n_pub
        body = apply_post_edits(body, fam)
        # the family's arms in GUARD_NAMES order (the union test holds it), runnable-only last in run_one order
        fam_arms = sorted(
            ((g, f, c) for g, f, c in arms if fam_of_guard[g] == fam),
            key=lambda a: guard_index.get(a[0], len(guard_index) + arm_names.index(a[0])),
        )
        # the family's dispatch table, the arms' comments carried beside them
        table = [
            f"/// The `{fam}` family: every guard it dispatches, in `GUARD_NAMES` order, with the tier note each",
            "/// arm carried in `run_one` (sprint 733). The root's union test holds these tables equal to `GUARD_NAMES`.",
            f"pub(crate) const FAMILY: Family = Family {{",
            f'    name: "{fam}",',
            "    arms: &[",
        ]
        for g, f, c in fam_arms:
            note = f" {c}" if c else ""
            table.append(f'        ("{g}", {f}),{note}')
        table += ["    ],", "};", ""]
        header = (
            f"//! Guard family `{fam}` - split from `keel-cli/src/guards.rs` by `scripts/split_guards.py` (sprint 733).\n"
            "//!\n"
            "//! Each guard's dispatch arm, code and tests sit together; the shared scanners, the runner and the\n"
            "//! lock predicates are the crate root's (`super`). Nothing here was retyped: the text is guards.rs's,\n"
            "//! with `crate::` paths pointing at the members and private items opened to the crate.\n"
            "\n"
            "use super::*;\n"
            "\n"
        )
        (DST / f"{fam}.rs").write_text(header + "\n".join(table) + "\n" + body, encoding="utf-8", newline="\n")
        written_fns += sum(1 for it in its if it.kind == "fn")
        written_mods += sum(1 for it in its if it.kind == "mod")
    # the root: every root item in original order, run_one rewritten over the family tables
    root_items = per_home.get("root", [])
    root_body = ""
    for it in root_items:
        if it.kind == "fn" and it.name.startswith("run_one"):
            root_body += RUN_ONE_TEXT
            continue
        root_body += it.text.rstrip("\n") + "\n\n"
    root_body, counts = rewrite_crate_paths(root_body)
    total_rewrites += counts
    root_body = apply_post_edits(root_body, "root")
    written_fns += sum(1 for it in root_items if it.kind == "fn")
    written_mods += sum(1 for it in root_items if it.kind == "mod")
    mods_decl = "".join(f"mod {fam};\npub use {fam}::*;\n" for fam in FAMILIES)
    families_const = (
        "/// The family tables in family order; `run_one` dispatches through them and the union test holds\n"
        "/// them equal to `GUARD_NAMES` (D0503: the enforced list is the schema member's fact).\n"
        "pub const FAMILIES: &[&Family] = &[" + ", ".join(f"&{fam}::FAMILY" for fam in FAMILIES) + "];\n\n"
        "/// Guards `run_one` dispatches that are NOT enforced members of `GUARD_NAMES` (runnable-only).\n"
        "pub const RUNNABLE_ONLY: [&str; " + str(len(RUNNABLE_ONLY)) + "] = [" + ", ".join(f'"{n}"' for n in RUNNABLE_ONLY) + "];\n\n"
    )
    (DST / "lib.rs").write_text(
        LIB_HEADER + mods_decl + "\n" + FAMILY_TYPE + families_const + root_body + UNION_TEST,
        encoding="utf-8", newline="\n",
    )
    print("== apply ==")
    print(f"wrote {len(FAMILIES)} family files + lib.rs under {DST}")
    print(f"fns written: {written_fns} (before {before['fn']}); test mods written: {written_mods} (before {before['mod']})")
    print(f"crate:: rewrites: {dict(total_rewrites)}")
    print(f"items opened to pub(crate): {total_pub}")
    if written_fns != before["fn"] or written_mods != before["mod"]:
        raise SystemExit("apply wrote an unbalanced set - inspect before building")
    unm = [k for k in total_rewrites if k.startswith("UNMAPPED")]
    if unm:
        print("UNMAPPED crate paths:", unm)
        return 1
    return 0


GUARD_NAMES_RS = REPO / "members" / "keel-schema" / "src" / "guard_names.rs"


def read_guard_names() -> list[str]:
    """The enforced list in its declared order (D0503: the schema member's fact), read, never retyped."""
    text = GUARD_NAMES_RS.read_text(encoding="utf-8")
    m = re.search(r"pub const GUARD_NAMES: \[&str; \d+\] =\s*\[(.*?)\];", text, re.S)
    if not m:
        raise SystemExit(f"GUARD_NAMES not found in {GUARD_NAMES_RS}")
    return re.findall(r'"([a-z0-9-]+)"', m.group(1))


def apply_post_edits(text: str, home: str) -> str:
    """The edits the split MAKES (not moves): each replaces the span from a unique start anchor to the
    first end anchor after it, inclusive, and refuses if the start is not found exactly once."""
    for edit in POST_EDITS:
        if edit["home"] != home:
            continue
        start, end, new = edit["start"], edit["end"], edit["new"]
        if text.count(start) != 1:
            raise SystemExit(f"post-edit {edit['name']}: start anchor found {text.count(start)} times in {home}")
        i = text.index(start)
        j = text.index(end, i) + len(end)
        text = text[:i] + new + text[j:]
    return text


# What changes in meaning with the split, folded here so the transform stays the one authority
# (D0503 holds the lock reshaping; the co-committed Decision names it).
POST_EDITS: list[dict[str, str]] = [
    {
        "name": "lock: guards.rs leaves GUARD_SOURCE_FILES, the member dir joins by prefix",
        "home": "enforcement",
        "start": 'pub(crate) const GUARD_SOURCE_FILES: &[&str] = &["keel-cli/src/guards.rs", ',
        "end": '"members/keel-schema/src/guard_names.rs"];\n',
        "new": (
            'pub(crate) const GUARD_SOURCE_FILES: &[&str] = &["keel-cli/src/adherence.rs", "members/keel-schema/src/guard_names.rs"];\n'
            "\n"
            "/// The directories whose EVERY file is enforcement logic: member keel-guards (sprint 733) - the family\n"
            "/// modules, the runner, the receipt that lets a green guard be skipped (D0371), the content key it is\n"
            "/// keyed on. Locked by prefix so a new family file is inside the lock by construction; the coverage\n"
            "/// test still scans every member for a `-> GuardReport` outside it.\n"
            'pub(crate) const GUARD_SOURCE_DIRS: &[&str] = &["members/keel-guards/src/"];\n'
        ),
    },
    {
        "name": "lock predicate: files or a locked dir prefix",
        "home": "enforcement",
        "start": "    // Guard SOURCE — the enforcement logic itself.\n    GUARD_SOURCE_FILES.contains(&p)\n",
        "end": "    GUARD_SOURCE_FILES.contains(&p)\n",
        "new": (
            "    // Guard SOURCE — the enforcement logic itself.\n"
            "    GUARD_SOURCE_FILES.contains(&p) || GUARD_SOURCE_DIRS.iter().any(|d| p.starts_with(d))\n"
        ),
    },
    {
        "name": "coverage test scans every member; lock assertions name the member; dispatch test reads the tables",
        "home": "root",
        "start": "    /// D0209 clause 2 coverage audit, made executable: every file that DEFINES a guard (`-> GuardReport`)\n",
        "end": "never actually run\"\n            );\n        }\n    }\n",
        "new": '''    /// D0209 clause 2 coverage audit, made executable: every file that DEFINES a guard (`-> GuardReport`)
    /// must sit inside the enforcement-surface lock, so a new guard file cannot be added OUTSIDE it and
    /// thereby be editable without a signed Decision. Scans the real `src/` tree of EVERY workspace member
    /// (sprint 733: the guards are a member, and a guard written into any other member is the same hole)
    /// rather than trusting the hand-list in `GUARD_SOURCE_FILES` / `GUARD_SOURCE_DIRS` -- if the two
    /// diverge, this fails CI, which is the point.
    #[test]
    fn enforcement_surface_covers_every_guard_source() {
        fn collect(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
            let Ok(entries) = std::fs::read_dir(dir) else { return };
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    collect(&p, out);
                } else if p.extension().is_some_and(|x| x == "rs") {
                    out.push(p);
                }
            }
        }
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap();
        let mut files = Vec::new();
        let manifest = keel_model::corpus::read_to_string(repo.join("Cargo.toml")).expect("the workspace manifest");
        let members = keel_model::corpus::workspace_members(&manifest);
        assert!(members.len() >= 12, "the workspace manifest lists its members: {members:?}");
        for member in &members {
            collect(&repo.join(member).join("src"), &mut files);
        }
        assert!(files.iter().any(|f| f.ends_with("identity.rs")), "the scan reached this member's own files");
        let mut uncovered = Vec::new();
        for f in files {
            let Ok(text) = keel_model::corpus::read_to_string(&f) else { continue };
            if !text.contains("-> GuardReport") {
                continue;
            }
            let repo_rel = f.strip_prefix(repo).unwrap().to_string_lossy().replace('\\\\', "/");
            if !is_enforcement_surface(&repo_rel) {
                uncovered.push(repo_rel);
            }
        }
        assert!(
            uncovered.is_empty(),
            "guard-defining file(s) OUTSIDE the enforcement-surface lock (add to GUARD_SOURCE_FILES / GUARD_SOURCE_DIRS): {uncovered:?}"
        );
    }

    #[test]
    fn enforcement_surface_locks_workflows_hooks_and_guard_source() {
        assert!(is_enforcement_surface(".github/workflows/ci.yml"));
        assert!(is_enforcement_surface(".githooks/pre-commit"));
        assert!(is_enforcement_surface("members/keel-guards/src/lib.rs"));
        assert!(is_enforcement_surface("members/keel-guards/src/identity.rs"));
        assert!(is_enforcement_surface("members/keel-guards/src/receipt.rs"));
        assert!(is_enforcement_surface("keel-cli/src/adherence.rs"));
        assert!(is_enforcement_surface("members/keel-schema/src/guard_names.rs"));
        // NOT locked: ordinary source, docs, a workflow-shaped path outside the dir, the old path.
        assert!(!is_enforcement_surface("keel-cli/src/main.rs"));
        assert!(!is_enforcement_surface("keel-cli/src/guards.rs"));
        assert!(!is_enforcement_surface("members/keel-guards/Cargo.toml"));
        assert!(!is_enforcement_surface(".engine/docs/guards.md"));
        assert!(!is_enforcement_surface("README.md"));
    }

    /// Every enforced guard must actually DISPATCH. `run_one` was a hand-written match, so a name could sit
    /// in `GUARD_NAMES` -- counted in the control inventory, listed in `--help`, documented in guards.md --
    /// while `run_one` returned `None` for it and `run_all` silently ran 35 of 36. Since sprint 733 dispatch
    /// is the family tables', so the check reads the tables (no source-text scan, no model build); the
    /// union test in `family_union_tests` holds the converse, that no table names a guard outside the list.
    #[test]
    fn every_enforced_guard_dispatches() {
        for name in GUARD_NAMES {
            assert!(
                super::FAMILIES.iter().flat_map(|f| f.arms.iter()).any(|(n, _)| *n == name),
                "guard `{name}` is in GUARD_NAMES but no family table dispatches it -- it would be counted in the control inventory and never actually run"
            );
        }
    }
''',
    },
]


RUN_ONE_TEXT = '''/// Run one guard by name - `None` if the name is unknown (a name outside every family table).
///
/// Dispatch is the family tables' (sprint 733): each family names its guards beside the code that
/// answers for them, and this function only walks the tables. A guard in one home is in all: the
/// union test below holds the tables equal to `GUARD_NAMES` plus `RUNNABLE_ONLY`.
#[must_use]
pub fn run_one(name: &str, root: &Path) -> Option<GuardReport> {
    FAMILIES.iter().flat_map(|f| f.arms.iter()).find(|(n, _)| *n == name).map(|(_, run)| run(root))
}

'''

FAMILY_TYPE = '''/// A guard: the function that answers for one name.
pub type GuardFn = fn(&Path) -> GuardReport;

/// One dispatch arm: `(guard name, the function that answers for it)`.
pub type Arm = (&'static str, GuardFn);

/// One guard family: its name and, in `GUARD_NAMES` order, the guards it dispatches (sprint 733).
pub struct Family {
    /// The family's module name.
    pub name: &'static str,
    /// The family's dispatch arms.
    pub arms: &'static [Arm],
}

'''

LIB_HEADER = '''//! keel-guards: the forward guards - one module per guard family, the runner and the shared scanners here.
//!
//! The fourth D0479 extraction (sprint 733): `keel-cli/src/guards.rs` split by `scripts/split_guards.py`
//! into family modules, each carrying its guards' dispatch arms, code and tests. keel-cli re-exports this
//! crate as `guards` under its old path, so no caller moved. The enforced list `GUARD_NAMES` is the schema
//! member's fact (D0503), re-exported here; the family tables are held equal to it by a test.
#![forbid(unsafe_code)]
#![deny(warnings, clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, clippy::todo, clippy::unimplemented)]
#![allow(clippy::implicit_hasher, clippy::too_long_first_doc_paragraph, clippy::wildcard_imports, clippy::module_name_repetitions)]
// The family modules are private and glob re-exported: a `pub(crate)` helper in one is reachable from the
// root and its siblings through `use super::*` yet stays out of the crate's public surface, which `pub`
// would not (the glob would export it). So the `pub(crate)` is not redundant, whatever the nursery lint says.
#![allow(clippy::redundant_pub_crate)]
// Tests may use unwrap/expect/panic/indexing/asserts freely.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing))]

pub mod contentkey;
pub mod hardening;
pub mod plan_cover;
pub mod receipt;

'''

UNION_TEST = '''
#[cfg(test)]
mod family_union_tests {
    use super::{FAMILIES, GUARD_NAMES, RUNNABLE_ONLY};

    /// D0503 / sprint 733: a guard in one home is in all. The union of the family tables, ordered by
    /// `GUARD_NAMES`, IS `GUARD_NAMES`; every other arm is a declared runnable-only guard; no name sits
    /// in two families; and each family lists its guards in `GUARD_NAMES` order.
    #[test]
    fn the_family_tables_union_to_guard_names_in_order() {
        let mut seen: Vec<&str> = Vec::new();
        for fam in FAMILIES {
            let mut last: Option<usize> = None;
            for (name, _) in fam.arms {
                assert!(!seen.contains(name), "{name} is dispatched by two families");
                seen.push(name);
                if let Some(pos) = GUARD_NAMES.iter().position(|g| g == name) {
                    assert!(last.is_none_or(|l| l < pos), "family {} lists {name} out of GUARD_NAMES order", fam.name);
                    last = Some(pos);
                } else {
                    assert!(RUNNABLE_ONLY.contains(name), "{name} is in a family table but neither enforced nor declared runnable-only");
                }
            }
        }
        let mut enforced: Vec<&str> = seen.iter().copied().filter(|n| GUARD_NAMES.contains(n)).collect();
        enforced.sort_by_key(|n| GUARD_NAMES.iter().position(|g| g == n));
        assert_eq!(enforced, GUARD_NAMES.to_vec(), "the union of the family tables is not GUARD_NAMES");
        for n in RUNNABLE_ONLY {
            assert!(seen.contains(&n), "runnable-only {n} is in no family table");
        }
    }
}
'''


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
