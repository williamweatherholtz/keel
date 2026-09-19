//! The `keel` verbs this member owns (D0479, sprint 750): `cmd_check`, `cmd_orphans`, `cmd_query1`, `cmd_mint`, `cmd_whats_next`, `cmd_ls`.
//!
//! Moved out of `keel-cli/src/main.rs` by `scripts/extract_verbs.py`: the binary is argument dispatch, and a
//! verb body lives with the member whose reach it needs (`python scripts/verb_homes.py --by-member`).
//! Each `cmd_*` takes the arguments after its verb and returns the process exit code.
//
// These were the binary's private items, written under the same lint set; only the lints that fire on
// PUBLIC visibility are new here, and they ask for ceremony a verb body does not owe: its exit code goes
// straight back to `main`, and its `Err` and panics are the ones the body already documents in place.
#![allow(clippy::must_use_candidate, clippy::missing_errors_doc, clippy::missing_panics_doc)]

use crate::orient;
use crate::corpus::collect_sysml;
use crate::validate::check_files;
use keel_args::{positional_arg, root_arg};
use keel_parser::parser_verbs::cmd_spec_version;
use std::path::{Path, PathBuf};

pub fn cmd_check(args: &[String]) -> i32 {
    if args.iter().any(|a| a == "--spec-version") {
        return cmd_spec_version(args);
    }
    if args.is_empty() {
        eprintln!("usage: keel gate check FILE [FILE...]  |  keel gate check --spec-version [--no-fetch]");
        return 2;
    }
    let files: Vec<PathBuf> = args.iter().map(PathBuf::from).collect();
    let report = check_files(&files);

    for err in &report.errors {
        println!("ERROR: {} — {}", err.file.display(), err.message);
    }
    if report.is_clean() {
        println!("{} file(s) checked clean.", files.len());
        0
    } else {
        eprintln!(
            "{} file(s) checked — {} error(s).",
            files.len(),
            report.errors.len()
        );
        1
    }
}


pub fn cmd_orphans(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel orphans [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    match crate::algo::orphans(&root) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(e) => {
            eprintln!("orphans error: {e}");
            1
        }
    }
}


// Name + optional root: `keel <name> <arg> [ROOT]`.
pub fn cmd_query1(args: &[String], usage: &str, f: fn(&std::path::Path, &str) -> String) -> i32 {
    let arg = match positional_arg(args, &format!("keel {usage} <name> [ROOT]"), "an item name") {
        Ok(a) => a,
        Err(code) => return code,
    };
    let root = match root_arg(args, &format!("keel {usage} <name> [ROOT]"), &[], 1) {
        Ok(r) => r,
        Err(code) => return code,
    };
    // AN UNRESOLVABLE NAME IS A FAILURE, NOT AN EMPTY RESULT (issue177). Every command routed through
    // here used to exit 0 for a name that does not exist, answering `{upstream: [], downstream: []}` -
    // which a script reads as "no relations", the reassuring wrong answer. `report`, `render` and `arch`
    // already exit nonzero on an unknown argument; these six did not, so the CLI was inconsistent with
    // itself on the one interface D0093 makes the automation substrate.
    if !crate::queries::is_declared(&root, arg) {
        eprintln!(
            "keel {usage}: no item named `{arg}` is declared in this model.
               An unknown name exits nonzero rather than answering with an empty result, because an empty              result reads as `this item has no relations` (issue177)."
        );
        return 1;
    }
    println!("{}", f(&root, arg));
    0
}


/// `keel record mint [N]` (us019/issue170) — engine-minted v4 UUIDs, one per line, nothing else on stdout,
/// composable into any authoring script.
///
/// Exists so no authoring path depends on an AI generating identity by hand: two hand-minted ids
/// were mangled before guard 38 existed, and manual diligence is not a control (D0047). What this
/// prints is tested against guard 38's OWN shape predicate, so mint and guard stay one truth.
pub fn cmd_mint(args: &[String]) -> i32 {
    const USAGE: &str = "keel record mint [N]   (N >= 1, default 1)";
    let n: u64 = match args {
        [] => 1,
        [a] => {
            if a.starts_with('-') {
                // the positional_arg convention (issue179): a leading dash is never a count
                eprintln!("error: `{a}` looks like a flag, not a count.");
                eprintln!("usage: {USAGE}");
                return 2;
            }
            match a.parse::<u64>() {
                Ok(n) if n >= 1 => n,
                _ => {
                    eprintln!("error: `{a}` is not a count of at least 1.");
                    eprintln!("usage: {USAGE}");
                    return 2;
                }
            }
        }
        _ => {
            eprintln!("usage: {USAGE}");
            return 2;
        }
    };
    let mut out = String::new();
    for _ in 0..n {
        out.push_str(&crate::ident::gen_uuid());
        out.push('\n');
    }
    print!("{out}");
    0
}


pub fn cmd_whats_next(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel show whats-next [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    // issue239/issue247: whats-next used to print nothing and exit 0 whether the frontier was
    // genuinely empty or a filter had failed to compute — identical output for COMPUTED-EMPTY and
    // COULD-NOT-COMPUTE, on the one answer the AI auto-follows (D0052). Now the two are distinct:
    // a failed computation REFUSES rather than answering with silence.
    // The frontier half only (issue439): the suspect walk and the burndown are orient's, not this list's.
    let (ready, compute_failures, outstanding) = orient::ready(&root);
    if !compute_failures.is_empty() {
        eprintln!("whats-next: COULD-NOT-COMPUTE — refusing to print a frontier that may be wrong:");
        for r in &compute_failures {
            eprintln!("  {r}");
        }
        eprintln!("  This is NOT an empty frontier. Fix the model read, then re-run.");
        return 1;
    }
    if ready.is_empty() {
        eprintln!("whats-next: COMPUTED-EMPTY — no task is ready (computed over {outstanding} outstanding item(s)). This is an answer, not a failure.");
    }
    for task in ready {
        println!("{task}");
    }
    0
}


pub fn cmd_ls(args: &[String]) -> i32 {
    let root = match root_arg(args, "keel ls [ROOT]", &[], 0) {
        Ok(r) => r,
        Err(code) => return code,
    };
    let dir = root.join(".tracking");
    for p in collect_sysml(&dir) {
        println!("{}", p.display());
    }
    0
}


/// Header kept at the top of a generated `activation.toml` — the file must explain itself, because the
/// consequence of editing it wrongly (a control silently off) is not obvious from its contents.
pub const ACTIVATION_HEADER: &str = "\
# Process activation (D0138) — which processes THIS project has adopted.
#
# What activating a process does: turns on its whole unit (skill + declared rules + guards), as defined
# by the engine from each process's own `assert constraint` declarations. Deactivating one stops its guards running,
# and `keel gate guard` then REPORTS each as NOT ACTIVE rather than skipping it silently.
#
# DELETE THIS FILE to return to \"everything is active\", which is also the behaviour when no file
# exists — so an existing project that never declares one is unaffected.
#
# CORE guards (identity, provenance, vocabulary, rootedness, well-formedness) are in no unit and CANNOT
# be deactivated here. Activation exists to stop enforcing procedures you have not adopted; it is not a
# switch that makes truthfulness optional.
#
# Edit by hand, or use `keel activate <process>` / `keel deactivate <process>`.
";


/// Write `activation.toml` with both active sets stated exactly.
///
/// BOTH sections are always written, even when one is unchanged (D0164). Writing only the section being
/// edited would leave the other absent, and absent means EVERYTHING ACTIVE — so deactivating one
/// viewpoint would silently re-activate every process the project had turned off. A partial write of a
/// contract whose absence has meaning is a data-loss bug, not a convenience.
/// Rewrite ONLY the `active = [...]` line inside `[section]`, leaving every other byte of the
/// manifest untouched — comments, `charteredBy`, blank lines, ordering, line endings.
///
/// Regenerating the file from a template instead was issue293: one `deactivate`+`activate` round
/// trip deleted `charteredBy = "d0226"` with the comment explaining it. Returns `None` when the
/// section or its `active` line is absent, so the caller can fall back to generating a fresh file
/// rather than silently writing a manifest with a section missing (absence has meaning here).
pub fn replace_active_line(src: &str, section: &str, items: &[String]) -> Option<String> {
    let want = format!("[{section}]");
    let rendered = items.iter().map(|p| format!("\"{p}\"")).collect::<Vec<_>>().join(", ");
    let (mut in_section, mut replaced) = (false, false);
    let mut out = String::with_capacity(src.len() + rendered.len());
    for line in src.split_inclusive('\n') {
        let trimmed = line.trim_end();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_section = trimmed == want;
        } else if in_section && !replaced && trimmed.starts_with("active") && trimmed.contains('=') {
            let eol = &line[trimmed.len()..]; // preserve LF vs CRLF vs no trailing newline
            out.push_str("active = [");
            out.push_str(&rendered);
            out.push(']');
            out.push_str(eol);
            replaced = true;
            continue;
        }
        out.push_str(line);
    }
    replaced.then_some(out)
}


/// The `active` list the manifest currently RECORDS for a section, read from the file rather than
/// recomputed from the engine's view of it.
///
/// The second half of issue293: the process list was rebuilt from `unit_names()`, which only knows
/// SWITCHABLE processes, so every `[always]` one (`decision-authoring`) vanished on any write. An
/// absent section means everything is active, but a PRESENT list naming 10 of 11 marks the 11th
/// INACTIVE — so omitting a process from a list that exists deactivates it by silence.
pub fn recorded_active(root: &Path, section: &str) -> Option<Vec<String>> {
    let src = std::fs::read_to_string(root.join(".engine/contracts/activation.toml")).ok()?;
    let want = format!("[{section}]");
    let mut in_section = false;
    for line in src.lines() {
        let trimmed = line.trim_end();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_section = trimmed == want;
        } else if in_section && trimmed.starts_with("active") {
            let list = trimmed.split_once('[')?.1.rsplit_once(']')?.0;
            return Some(
                list.split(',')
                    .map(|s| s.trim().trim_matches('"').to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
            );
        }
    }
    None
}


pub fn write_activation(root: &Path, processes: &[String], viewpoints: &[String]) -> std::io::Result<()> {
    // JOIN `.engine/contracts` HERE, as the original did. Taking a pre-joined directory instead was my
    // regression and it wrote the manifest to the repo root, where `Activation::load` never looks - so
    // `keel deactivate` reported success and changed nothing. Caught by round-tripping the command
    // against the surfaces it is supposed to affect rather than by reading its output, which said
    // "deactivated" either way.
    let dir = root.join(".engine/contracts");
    std::fs::create_dir_all(&dir)?;
    // PRESERVE FIRST (issue293): an existing manifest is EDITED, never regenerated. Only a missing
    // or unparseable one falls through to the template below.
    let path = dir.join("activation.toml");
    if let Ok(existing) = std::fs::read_to_string(&path) {
        if let Some(edited) = replace_active_line(&existing, "processes", processes)
            .and_then(|s| replace_active_line(&s, "viewpoints", viewpoints))
        {
            return std::fs::write(&path, edited);
        }
    }
    std::fs::write(
        dir.join("activation.toml"),
        format!(
            "{ACTIVATION_HEADER}
[processes]
active = [{}]

[viewpoints]
active = [{}]
",
            processes.iter().map(|p| format!("\"{p}\"")).collect::<Vec<_>>().join(", "),
            viewpoints.iter().map(|v| format!("\"{v}\"")).collect::<Vec<_>>().join(", "),
        ),
    )
}


/// Activate or deactivate one VIEWPOINT, writing both manifest sections (D0164).
///
/// Split out of `cmd_activation` to keep that function within the line budget, and because the two
/// namespaces genuinely differ: a process switch turns guards on and off, a viewpoint switch turns a LENS
/// on and off. Bundling them into one branchy function hid that.
pub fn switch_viewpoint(
    root: &Path,
    act: &crate::activation::Activation,
    mode: &str,
    target: &str,
    all: &[String],
    mut set: Vec<String>,
) -> i32 {
    let materialising = act.active_viewpoints.is_none();
    if mode == "activate" {
        if !set.iter().any(|v| v == target) {
            set.push(target.to_string());
        }
    } else {
        set.retain(|v| v != target);
    }
    set.sort();
    // The PROCESS section must be rewritten too, unchanged: absence means everything active, so omitting
    // it would silently re-activate every process the project had turned off.
    let procs: Vec<String> = recorded_active(root, "processes")
        .unwrap_or_else(|| act.unit_names().into_iter().filter(|p| act.is_process_active(p)).collect());
    if let Err(e) = write_activation(root, &procs, &set) {
        eprintln!("error writing .engine/contracts/activation.toml: {e}");
        return 1;
    }
    if materialising {
        println!("No viewpoint manifest existed (all were active), so one was written with the current");
        println!("effective state before applying this change.");
    }
    println!("{mode}d viewpoint `{target}`. Active viewpoints: {} of {}", set.len(), all.len());
    println!("Read it back: keel activation | keel gate guard");
    0
}


/// A decision's OPTION letters (a fork) and its title plus decision text, for read-back ratification (D0201 B).
pub fn decision_options_and_title(root: &Path, dec: &str) -> (Vec<String>, String) {
    let mut letters = Vec::new();
    let mut title = String::new();
    for p in crate::corpus::collect_sysml(&root.join(".engine").join("decisions")) {
        let Ok(text) = std::fs::read_to_string(&p) else { continue };
        if !text.contains(&format!("part {dec} : Decision")) {
            continue;
        }
        let rel = p.strip_prefix(root).unwrap_or(&p).to_string_lossy().replace('\\', "/");
        letters = crate::textscan::fork_options(root, &rel).into_iter().map(|(l, _)| l).collect();
        // the title AND the decision text: quoting the decision's own words is a read-back of it
        for key in [":>> title = \"", ":>> decision = \""] {
            if let Some(i) = text.find(key) {
                title.push_str(text.get(i + key.len()..).and_then(|r| r.split('"').next()).unwrap_or(""));
                title.push(' ');
            }
        }
        break;
    }
    (letters, title)
}
