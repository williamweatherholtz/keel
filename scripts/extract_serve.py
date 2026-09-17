"""extract_serve.py - the sprint 738 transform: the console becomes member keel-serve, ci_runs joins keel-github (D0479).

not-an-instrument: it is a one-shot codemod (the migration skill's gate 1); its reconciliation totals are the
transform checking itself before it writes, not a measure of the project - the release build under
deny(warnings), the serve / deck / launcher integration tests and the rebuild probe are the sensors the
move answers to.

One committed script, dry-run by default (D0479 one extraction per sprint; the migration skill's gate 1).
Eight modules move as git renames and every `crate::<module>::` path in each is rewritten per the map of
the crate it lands in:

    keel-cli/src/{serve,deck,launcher,console_registry,reports,attestation}.rs -> members/keel-serve/src/
    keel-cli/src/ci_runs.rs                                                     -> members/keel-github/src/
    keel-cli/src/verification.rs                                                -> members/keel-view/src/

reports and attestation ride with serve because that is where their readers are: reports reads the guards'
readiness, the view layer and attestation, attestation reads the write API's reverify, the actor and the
workspace - both sit at the console's layer, above every member and below keel-cli. The console's embedded
page moves with the module that `include_str!`s it (keel-cli/assets/console.html -> members/keel-serve/assets/).
One slice leaves a file that stays: the orient computation (OrientReport, compute_orient_state, orient_root,
whats_next_root) out of keel-cli's lib.rs into members/keel-model/src/readiness.rs, so the console reads
`whats_next_root` from the read model and not from the binary crate. keel-cli's lib.rs re-exports every
moved name at its old path, so no caller in main.rs or the tests moved. The reconciliation holds each
moved file's text equal to its source modulo the counted rewrites and the include paths named below, and
each staying file equal to its source minus the slice plus the anchored edits.

Two guard-file lines move with the source they read: hardening.rs's api_surface lens and its route test
read serve.rs and console.html at their new home. members/keel-guards/src is a locked path, so the sprint's
Decision carries the process-change marker (D0209 clause 2).

    python scripts/extract_serve.py            # plan + reconciliation, nothing written
    python scripts/extract_serve.py --apply    # the renames, the rewrites, the slice, the manifests, the re-exports

Idempotent: a tree where members/keel-serve/src/lib.rs exists is reported as already applied.
"""
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
CLI = REPO / "keel-cli"
CLI_SRC = CLI / "src"
MEMBERS = REPO / "members"
SERVE = MEMBERS / "keel-serve"
SERVE_SRC = SERVE / "src"
GITHUB_SRC = MEMBERS / "keel-github" / "src"
VIEW_SRC = MEMBERS / "keel-view" / "src"
MODEL_SRC = MEMBERS / "keel-model" / "src"
GUARDS_SRC = MEMBERS / "keel-guards" / "src"

# `crate::<module>` in a moved file -> the path that resolves from the crate it lands in. A two-segment key
# wins over its one-segment head (the item views are keel-issues', the rest of `view` is keel-view's). A
# module the destination owns stays `crate::`.
SERVE_MAP = {
    "view::dispositions": "keel_issues::views::dispositions",
    "view::open_issues": "keel_issues::views::open_issues",
    "view::intake": "keel_issues::views::intake",
    "view": "keel_view::view",
    "verification": "keel_view::verification",
    "govern": "keel_view::govern",
    "pm": "keel_view::pm",
    "arch": "keel_view::arch",
    "json": "keel_json::json",
    "color": "keel_json::color",
    "write": "keel_write::write",
    "scaffold": "keel_write::scaffold",
    "reverify": "keel_write::reverify",
    "claude_surface": "keel_write::claude_surface",
    "fingerprint": "keel_model::fingerprint",
    "orient": "keel_model::orient",
    "ident": "keel_model::ident",
    "textscan": "keel_model::textscan",
    "done": "keel_model::done",
    "algo": "keel_model::algo",
    "validate_root": "keel_model::validate::validate_root",
    "collect_sysml": "keel_model::corpus::collect_sysml",
    "supersede_targets": "keel_model::corpus::supersede_targets",
    "whats_next_root": "keel_model::readiness::whats_next_root",
    "device": "keel_actor::device",
    "actor": "keel_actor::actor",
    "gitx": "keel_git::gitx",
    "guards": "keel_guards",
    "receipt": "keel_guards::receipt",
    "workspace": "keel_process::workspace",
    "perf": "keel_perf::perf",
    "cli_surface": "keel_schema::cli_surface",
}
SERVE_OWN = {"serve", "deck", "launcher", "console_registry", "reports", "attestation"}
GITHUB_MAP = {
    "color": "keel_json::color",
    "gitx": "keel_git::gitx",
    "collect_sysml": "keel_model::corpus::collect_sysml",
}
GITHUB_OWN = {"github", "ci_runs"}
VIEW_MAP: dict[str, str] = {}
VIEW_OWN = {"view", "verification", "govern", "pm", "arch", "priority", "control_proof"}

# (source, destination, map, own)
MOVES = [
    (CLI_SRC / "serve.rs", SERVE_SRC / "serve.rs", SERVE_MAP, SERVE_OWN),
    (CLI_SRC / "deck.rs", SERVE_SRC / "deck.rs", SERVE_MAP, SERVE_OWN),
    (CLI_SRC / "launcher.rs", SERVE_SRC / "launcher.rs", SERVE_MAP, SERVE_OWN),
    (CLI_SRC / "console_registry.rs", SERVE_SRC / "console_registry.rs", SERVE_MAP, SERVE_OWN),
    (CLI_SRC / "reports.rs", SERVE_SRC / "reports.rs", SERVE_MAP, SERVE_OWN),
    (CLI_SRC / "attestation.rs", SERVE_SRC / "attestation.rs", SERVE_MAP, SERVE_OWN),
    (CLI_SRC / "ci_runs.rs", GITHUB_SRC / "ci_runs.rs", GITHUB_MAP, GITHUB_OWN),
    (CLI_SRC / "verification.rs", VIEW_SRC / "verification.rs", VIEW_MAP, VIEW_OWN),
]
# the page the console embeds moves with it; no text changes
ASSET_MOVES = [(CLI / "assets" / "console.html", SERVE / "assets" / "console.html")]
# edits inside a moved file beyond the crate:: rewrite - the two include_str! paths that were relative to
# keel-cli/src; each anchor occurs exactly once and the length delta is counted in the reconciliation
MOVE_EDITS = {
    CLI_SRC / "serve.rs": [
        ('("main.rs", include_str!("main.rs")),', '("main.rs", include_str!("../../../keel-cli/src/main.rs")),'),
        ('include_str!("../../members/keel-model/src/orient.rs")', 'include_str!("../../keel-model/src/orient.rs")'),
    ],
}
REF = re.compile(r"\bcrate::([A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*)")

LINTS = '''#![forbid(unsafe_code)]
#![deny(warnings, clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, clippy::todo, clippy::unimplemented)]
#![allow(clippy::implicit_hasher, clippy::too_long_first_doc_paragraph, clippy::module_name_repetitions)]
// Tests may use unwrap/expect/panic/indexing/asserts freely.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing))]
'''

CARGO = '''[package]
name = "keel-serve"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "The console (D0094): the localhost server, the obligation deck, the run launcher, the console registry, the scorecard reports and the attestation sampling - everything that serves the engine to a person, above every member and below keel-cli."

[dependencies]
keel-perf = { path = "../keel-perf" }
keel-git = { path = "../keel-git" }
keel-json = { path = "../keel-json" }
keel-actor = { path = "../keel-actor" }
keel-schema = { path = "../keel-schema" }
keel-model = { path = "../keel-model" }
keel-write = { path = "../keel-write" }
keel-view = { path = "../keel-view" }
keel-guards = { path = "../keel-guards" }
keel-process = { path = "../keel-process" }
keel-issues = { path = "../keel-issues" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
tokio = { version = "1", features = ["rt-multi-thread", "net", "macros", "process", "io-util", "time", "sync"] }
axum = "0.7"
async-stream = "0.3"
tokio-stream = "0.1"

[dev-dependencies]
# keel_fs::test_support::repo_root and keel_fs::scratch, the one repository-root walk and scratch dir for tests (issue557).
keel-fs = { path = "../keel-fs" }
tower = { version = "0.5", features = ["util"] }
'''

LIB = '''//! keel-serve: the console - the localhost server (D0094), the obligation deck, the run launcher, the
//! console registry, the scorecard reports and the attestation sampling.
//!
//! The eighth D0479 extraction (sprint 738): six modules out of keel-cli/src, moved by
//! `scripts/extract_serve.py`, with the page the server embeds. keel-cli re-exports each at its old path
//! (`crate::serve`, `crate::deck`, `crate::launcher`, `crate::console_registry`, `crate::reports`,
//! `crate::attestation`), so no caller moved. The crate sits above every member and below keel-cli: it reads
//! the view layer, the guards, the write API, the processes and the item views, and nothing reads it but
//! the binary - so an edit to `serve.rs` rebuilds this crate and keel-cli only.
''' + LINTS + '''
pub mod attestation;
pub mod console_registry;
pub mod deck;
pub mod launcher;
pub mod reports;
pub mod serve;
'''

READINESS_HEAD = '''//! The orient computation (sprint 738, D0479): [`OrientReport`], [`compute_orient_state`], [`orient_root`]
//! and [`whats_next_root`], sliced whole out of keel-cli's lib.rs by `scripts/extract_serve.py` so the
//! console reads `whats_next_root` from the read model rather than from the binary crate. keel-cli
//! re-exports the four at its root, so `keel_cli::orient_root` and the cucumber steps still resolve.

use std::collections::HashSet;
use std::path::Path;

use keel_parser::ast::{ActionDef, Item, Package, Part, Value};

use crate::corpus::{collect_sysml, parse_pkg};
use crate::indexer::{parse_cursor, Cursor};

'''

ORIENT_SLICE = (CLI_SRC / "lib.rs", "// ── orient types", "// ── internal parse helper")

# anchored edits in files that stay: (file, old, new) - each anchor occurs exactly once
EDITS = [
    # keel-cli lib.rs: the six console modules, ci_runs and verification become re-exports; the imports the
    # orient slice used leave with it; the four orient names keep resolving at the root
    (CLI_SRC / "lib.rs",
     "use std::collections::HashSet;\nuse std::path::Path;\n\nuse keel_parser::ast::{ActionDef, Item, Package, Part, Value};\n\n",
     ""),
    (CLI_SRC / "lib.rs", "pub mod verification;\n", "pub use keel_view::verification;\n"),
    (CLI_SRC / "lib.rs", "pub mod ci_runs;\n", "pub use keel_github::ci_runs;\n"),
    (CLI_SRC / "lib.rs", "pub mod attestation;\n", "pub use keel_serve::attestation;\n"),
    (CLI_SRC / "lib.rs", "pub mod deck;\n", "pub use keel_serve::deck;\n"),
    (CLI_SRC / "lib.rs", "pub mod launcher;\n", "pub use keel_serve::launcher;\n"),
    (CLI_SRC / "lib.rs", "pub mod reports;\n", "pub use keel_serve::reports;\n"),
    (CLI_SRC / "lib.rs", "use keel_json::json;\n", ""),
    (CLI_SRC / "lib.rs", "pub mod console_registry;\n", "pub use keel_serve::console_registry;\n"),
    (CLI_SRC / "lib.rs", "pub mod serve;\n",
     "// The console is member keel-serve (D0479, sprint 738): serve, deck, launcher, console_registry, reports, attestation; `crate::serve::` etc. keep resolving.\npub use keel_serve::serve;\n"),
    (CLI_SRC / "lib.rs", ORIENT_SLICE[2],
     "// The orient computation is the read model's (sprint 738, D0479); the four keep resolving at the root.\npub use keel_model::readiness::{compute_orient_state, orient_root, whats_next_root, OrientReport};\n\n" + ORIENT_SLICE[2]),
    # the members that gain a module
    (MODEL_SRC / "lib.rs", "pub mod resolvers;\n", "pub mod resolvers;\npub mod readiness;\n"),
    (VIEW_SRC / "lib.rs", "pub mod view;\n", "pub mod verification;\npub mod view;\n"),
    (GITHUB_SRC / "lib.rs", "pub mod github;\n", "pub mod ci_runs;\npub mod github;\n"),
    (MEMBERS / "keel-github" / "Cargo.toml", "[dependencies]\n\n",
     "[dependencies]\n# ci_runs (sprint 738): the CI-run receipt gate reads the corpus, git and the colour helpers.\nkeel-json = { path = \"../keel-json\" }\nkeel-git = { path = \"../keel-git\" }\nkeel-model = { path = \"../keel-model\" }\nserde_json = \"1\"\n\n"),
    # the manifests
    (REPO / "Cargo.toml", '    "members/keel-issues",\n    "keel-cli",', '    "members/keel-issues",\n    "members/keel-serve",\n    "keel-cli",'),
    (CLI / "Cargo.toml", 'keel-issues = { path = "../members/keel-issues" }\n',
     'keel-issues = { path = "../members/keel-issues" }\n# D0479 console (sprint 738): serve, deck, launcher, console_registry, reports, attestation; re-exported under their old paths in lib.rs.\nkeel-serve = { path = "../members/keel-serve" }\n'),
    (CLI / "Cargo.toml",
     'tokio = { version = "1", features = ["rt-multi-thread", "net", "macros", "process", "io-util", "time", "sync"] }\naxum = "0.7"\nasync-stream = "0.3"\ntokio-stream = "0.1"\n',
     ""),
    (CLI / "Cargo.toml", 'tokio = { version = "1", features = ["rt-multi-thread", "macros"] }\ntower = { version = "0.5", features = ["util"] }\n',
     'tokio = { version = "1", features = ["rt-multi-thread", "macros"] }\n'),
    # the guard lens and its test read the console source where it lives now (locked path - the marked Decision)
    (GUARDS_SRC / "hardening.rs",
     'std::fs::read_to_string(root.join("keel-cli/src/serve.rs")),\n        std::fs::read_to_string(root.join("keel-cli/assets/console.html")),',
     'std::fs::read_to_string(root.join("members/keel-serve/src/serve.rs")),\n        std::fs::read_to_string(root.join("members/keel-serve/assets/console.html")),'),
    (GUARDS_SRC / "hardening.rs", '"keel-cli/src/serve.rs or assets/console.html is not readable', '"members/keel-serve/src/serve.rs or assets/console.html is not readable'),
    (GUARDS_SRC / "hardening.rs",
     'crate::test_repo_root().join("keel-cli/src/serve.rs")).expect("serve.rs is readable");\n        let html = std::fs::read_to_string(crate::test_repo_root().join("keel-cli/assets/console.html"))',
     'crate::test_repo_root().join("members/keel-serve/src/serve.rs")).expect("serve.rs is readable");\n        let html = std::fs::read_to_string(crate::test_repo_root().join("members/keel-serve/assets/console.html"))'),
    # the suspicion manifest names the deliverable's paths per task (D0050)
    (REPO / ".engine" / "deliverable-manifest.txt", "task: serveItemActions | keel-cli/src/serve.rs keel-cli/assets/console.html",
     "task: serveItemActions | members/keel-serve/src/serve.rs members/keel-serve/assets/console.html"),
    (REPO / ".engine" / "deliverable-manifest.txt", "task: rustS7QueryAlign | members/keel-model/src/orient.rs keel-cli/src/lib.rs",
     "task: rustS7QueryAlign | members/keel-model/src/orient.rs members/keel-model/src/readiness.rs"),
    # the code registry's own elements (createdBy claudeFable5, D0108) and two purpose sentences that cite the deck's source
    (REPO / ".tracking" / "architecture" / "code-registry.sysml", ':>> filePath = "keel-cli/src/serve.rs";', ':>> filePath = "members/keel-serve/src/serve.rs";'),
    (REPO / ".tracking" / "architecture" / "code-registry.sysml", ':>> filePath = "keel-cli/src/launcher.rs";', ':>> filePath = "members/keel-serve/src/launcher.rs";'),
    (REPO / ".engine" / "processes" / "obligation-review.sysml", "Deploys `keel deck` (keel-cli/src/deck.rs).", "Deploys `keel deck` (members/keel-serve/src/deck.rs)."),
    (REPO / ".engine" / "skills" / "obligation-review" / "registry.sysml", "Generated by `keel deck` (keel-cli/src/deck.rs)", "Generated by `keel deck` (members/keel-serve/src/deck.rs)"),
]


def read(p: Path) -> str:
    return p.read_text(encoding="utf-8")


def write(p: Path, text: str) -> None:
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(text, encoding="utf-8", newline="\n")


def rewrite(text: str, module_map: dict[str, str], own: set[str]) -> tuple[str, dict[str, int]]:
    """Every `crate::X[::Y]` per the map; the counts per key are the reconciliation."""
    counts: dict[str, int] = {}

    def sub(m: re.Match[str]) -> str:
        path = m.group(1)
        segs = path.split("::")
        for key in ("::".join(segs[:2]), segs[0]):
            if key in module_map:
                counts[key] = counts.get(key, 0) + 1
                return module_map[key] + path[len(key):]
        if segs[0] in own:
            return m.group(0)
        raise SystemExit(f"unmapped crate path in a moved file: crate::{path}")

    return REF.sub(sub, text), counts


def plan_moves() -> list[tuple[Path, Path, str, dict[str, int], int]]:
    out = []
    for src, dst, module_map, own in MOVES:
        text = read(src)
        new, counts = rewrite(text, module_map, own)
        before = len(REF.findall(text))
        after_foreign = sum(counts.values())
        after_own = len(REF.findall(new))
        if before != after_foreign + after_own:
            raise SystemExit(f"{src.name}: {before} crate:: refs before, {after_foreign} rewritten + {after_own} kept")
        delta = sum(n * (len(module_map[k]) - len("crate::" + k)) for k, n in counts.items())
        for old, repl in MOVE_EDITS.get(src, []):
            if new.count(old) != 1:
                raise SystemExit(f"{src.name}: include anchor occurs {new.count(old)} times: {old[:60]!r}")
            new = new.replace(old, repl)
            delta += len(repl) - len(old)
        if len(new) - len(text) != delta or (not counts and not MOVE_EDITS.get(src) and new != text):
            raise SystemExit(f"{src.name}: the rewrite is not the only difference")
        out.append((src, dst, new, counts, before))
    return out


def cut(text: str, start: str, end: str, label: str) -> tuple[str, str]:
    """(text without the slice, the slice). Both anchors occur exactly once; the end is exclusive."""
    if text.count(start) != 1:
        raise SystemExit(f"{label}: start anchor occurs {text.count(start)} times: {start[:50]!r}")
    if text.count(end) != 1:
        raise SystemExit(f"{label}: end anchor occurs {text.count(end)} times: {end[:50]!r}")
    i, j = text.index(start), text.index(end)
    if j <= i:
        raise SystemExit(f"{label}: slice ends before it starts")
    return text[:i] + text[j:], text[i:j]


class Plan:
    def __init__(self) -> None:
        self.files: dict[Path, str] = {}
        self.slices: dict[str, str] = {}
        self.log: list[str] = []

    def text(self, p: Path) -> str:
        if p not in self.files:
            self.files[p] = read(p)
        return self.files[p]

    def slice(self, spec: tuple[Path, str, str], label: str) -> str:
        p, start, end = spec
        before = self.text(p)
        remaining, piece = cut(before, start, end, label)
        if len(remaining) + len(piece) != len(before):
            raise SystemExit(f"{label}: the slice is not conserved")
        self.files[p] = remaining
        self.slices[label] = piece
        self.log.append(f"{p.relative_to(REPO).as_posix()}: {label} out, {len(piece)} chars")
        return piece

    def edit(self, p: Path, old: str, new: str) -> None:
        t = self.text(p)
        if t.count(old) != 1:
            raise SystemExit(f"{p.relative_to(REPO).as_posix()}: anchor occurs {t.count(old)} times: {old[:60]!r}")
        self.files[p] = t.replace(old, new)


def plan() -> tuple[Plan, dict[Path, str]]:
    pl = Plan()
    orient = pl.slice(ORIENT_SLICE, "orient computation")
    for name in ("pub struct OrientReport", "pub fn compute_orient_state", "pub fn whats_next_root", "pub fn orient_root", "fn json_str"):
        if orient.count(name) != 1:
            raise SystemExit(f"orient computation: expected one {name!r} in the slice, found {orient.count(name)}")
    for p, old, new in EDITS:
        pl.edit(p, old, new)
    new_files = {
        SERVE / "Cargo.toml": CARGO,
        SERVE_SRC / "lib.rs": LIB,
        MODEL_SRC / "readiness.rs": READINESS_HEAD + orient.rstrip("\n") + "\n",
    }
    # reconciliation: the slice is in its destination whole
    if orient.strip() not in new_files[MODEL_SRC / "readiness.rs"]:
        raise SystemExit("orient computation: did not carry over whole")
    for src, _dst in ASSET_MOVES:
        if not src.exists():
            raise SystemExit(f"asset missing: {src.relative_to(REPO).as_posix()}")
    return pl, new_files


def main(argv: list[str]) -> int:
    apply = "--apply" in argv
    if (SERVE_SRC / "lib.rs").exists():
        print("already applied: members/keel-serve/src/lib.rs exists")
        return 0
    moves = plan_moves()
    pl, new_files = plan()
    total_refs = 0
    for src, dst, _new, counts, before in moves:
        total_refs += before
        print(f"{src.relative_to(REPO).as_posix()} -> {dst.relative_to(REPO).as_posix()}: {before} crate:: refs, rewritten {dict(sorted(counts.items()))}")
    for line in pl.log:
        print(line)
    print(f"reconciliation: {len(moves)} files moved, {total_refs} crate:: refs each kept or rewritten, {sum(len(v) for v in MOVE_EDITS.values())} include paths re-anchored; {len(ASSET_MOVES)} asset moved; {len(pl.slices)} slice conserved; {len(EDITS)} anchored edits in {len(pl.files)} staying files; {len(new_files)} files written")
    if not apply:
        print("dry run; --apply to write")
        return 0
    SERVE_SRC.mkdir(parents=True, exist_ok=True)
    for src, dst, new, _counts, _before in moves:
        dst.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run(["git", "mv", str(src), str(dst)], cwd=REPO, check=True)
        write(dst, new)
    for src, dst in ASSET_MOVES:
        dst.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run(["git", "mv", str(src), str(dst)], cwd=REPO, check=True)
    for p, text in pl.files.items():
        write(p, text)
        print("edited", p.relative_to(REPO).as_posix())
    for p, text in new_files.items():
        write(p, text)
        print("wrote", p.relative_to(REPO).as_posix())
    print("applied")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
