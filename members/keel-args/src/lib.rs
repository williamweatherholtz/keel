//! keel-args: the argument helpers every keel verb shares (D0479, sprint 740).
//!
//! `scripts/verb_homes.py` showed the shape: every `cmd_` body in keel-cli/src/main.rs parsed its arguments
//! through nine private fns of main.rs, so no body could leave the binary before they did. They moved here
//! whole by `scripts/extract_seams.py`: [`root_arg`] (the `[ROOT]` positional that refuses an unknown flag,
//! issue133, and a non-project, issue281), [`without_flag_values`], [`refuse_flag_as_path`] (GH#14),
//! [`flag`], [`prose_flag`] and [`prose_args`] (the `-from FILE` form, D0224), [`provenance_date`]
//! (never defaulted, issue182), [`positional_arg`] (issue179) and [`repo_arg`]. The crate depends on
//! keel-git alone, for project discovery; keel-cli re-exports it as `args` and main.rs imports the names.
#![forbid(unsafe_code)]
#![deny(warnings, clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, clippy::todo, clippy::unimplemented)]
#![allow(clippy::implicit_hasher, clippy::too_long_first_doc_paragraph, clippy::module_name_repetitions)]
// Tests may use unwrap/expect/panic/indexing/asserts freely.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing))]

// The verbs this member owns, out of main.rs (D0479, sprint 750).
pub mod args_verbs;

use std::path::PathBuf;

use keel_git::projects::{find_repo_root, require_project};

/// Drop each named flag AND ITS VALUE, leaving only true positionals for [`root_arg`].
///
/// THE CLASS THIS ENDS, third instance in one session: `root_arg` takes the first bare token as ROOT
/// and cannot know which flags consume the token after them, so every command that adds a
/// value-taking flag re-creates the bug. `keel github decider --root X` read X as a login,
/// `keel recall --prompt -` read `-` as a root, and `keel why t --budget 1500` read 1500 as a root —
/// each silently answering about the wrong thing until the issue281 project precondition started
/// refusing outright, which is the only reason the last two were visible at all.
///
/// It is NOT fixed inside `root_arg` because a known flag's following positional is legitimately the
/// root for existing callers (`--explain /r`), so the distinction has to be stated by the caller that
/// knows it.
#[must_use]
pub fn without_flag_values(args: &[String], value_flags: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    let mut skip = false;
    for a in args {
        if skip {
            skip = false;
            continue;
        }
        if let Some(name) = a.strip_prefix("--") {
            if value_flags.contains(&name) {
                skip = true;
                continue;
            }
        }
        out.push(a.clone());
    }
    out
}

/// Resolve a subcommand's optional `[ROOT]` positional, REFUSING an unrecognised flag (issue133).
///
/// `positionals` is how many leading positional arguments the subcommand takes before ROOT (`keel show view
/// <name> [ROOT]` passes 1). `known` lists the flag names the subcommand accepts, without `--`.
///
/// THE DEFECT THIS EXISTS TO END: every parser used to take its first argument as ROOT, so `keel audit
/// --explan` made the root the literal string `--explan`, and the command then failed somewhere
/// downstream about a missing directory — or worse, succeeded against the wrong tree. The other half of
/// the class SKIPPED anything starting with `--`, so an unknown flag was silently ignored and the command
/// ran with the wrong behaviour and said nothing. Both turn a typo into a confident wrong answer instead
/// of an error at the point of the mistake, which is the shape this whole class keeps taking.
///
/// `Err(2)` after printing the usage line; the caller returns that code unchanged.
///
/// # Errors
///
/// An unknown flag, no project found from the current directory, or a root that is not a keel project: the usage line is printed and the code is `2`.
pub fn root_arg(args: &[String], usage: &str, known: &[&str], positionals: usize) -> Result<PathBuf, i32> {
    let mut positional: Vec<&String> = Vec::new();
    for a in args {
        if let Some(name) = a.strip_prefix("--") {
            // A flag's VALUE is consumed by the caller's own parse; only the flag NAME is judged here.
            if !known.contains(&name) {
                eprintln!("error: unknown flag `{a}`");
                eprintln!("usage: {usage}");
                return Err(2);
            }
        } else {
            positional.push(a);
        }
    }
    if let Some(p) = positional.get(positionals) {
        let root = PathBuf::from(p.as_str());
        require_project(&root, usage)?;
        return Ok(root);
    }
    let root = find_repo_root().ok_or_else(|| {
        eprintln!("error: no .engine/ directory found from the current directory upward");
        eprintln!("  (the search stops at the repository boundary — it will not answer for another repo).");
        eprintln!("usage: {usage}");
        2
    })?;
    require_project(&root, usage)?;
    Ok(root)
}

/// Refuse an argument that LOOKS like a flag where a path or a name is expected (GH#14).
///
/// A mistyped or unsupported `--flag` used to be accepted as the ROOT: `keel gate guard --read` gated a
/// directory named `--read`, found nothing, and reported every guard PASS with 0 scanned. Silent
/// mis-parsing plus pass-at-zero produces a GREEN RUN OVER NOTHING, which is worse than an error
/// because it is indistinguishable from a clean tree. The same shape was hit again while building
/// `github ingest`, where a trailing `--at` value was read as the root.
///
/// Returns the exit code to use, or `None` when the argument is fine. `cmd_activation` already did
/// this for process names (issue179); this generalises it to every path-taking entry point.
#[must_use]
pub fn refuse_flag_as_path(arg: Option<&String>, cmd: &str) -> Option<i32> {
    let a = arg?;
    if !a.starts_with("--") {
        return None;
    }
    eprintln!("keel {cmd}: `{a}` looks like a flag, not a path.");
    eprintln!("  It would otherwise be taken as the ROOT — and a root that does not exist scans");
    eprintln!("  NOTHING, so every check would report PASS over an empty tree (GH#14). Refusing");
    eprintln!("  rather than answering green about a directory that is not there.");
    Some(2)
}

/// Parse simple `--key value` flag pairs from a flat args slice.
#[must_use]
pub fn flag(args: &[String], name: &str) -> Option<String> {
    let key = format!("--{name}");
    args.windows(2).find_map(|w| match w {
        [k, v] if *k == key => Some(v.clone()),
        _ => None,
    })
}

/// A PROSE flag: `--<name>-from FILE` read verbatim (trimmed), else `--<name> TEXT`. One reader for
/// every prose input of `record` (D0224, issue543). The trap this closes: a backtick inside a
/// double-quoted shell argument is command substitution, so the shell RUNS the command the prose
/// merely names - it fired into `decision` twice, `issue` once, `task` once, and on 2026-09-14 a
/// FIFTH time into `result`'s evidence, because the file form had been added verb by verb, each
/// after its own occurrence. Refuses both flags at once by name (the caller learns which would
/// have won) and a file it cannot read. `Ok(None)` is neither flag given.
///
/// # Errors
///
/// Both forms of one name at once, or a `-from` file that cannot be read; the message names which.
pub fn prose_flag(args: &[String], name: &str, verb: &str) -> Result<Option<String>, String> {
    let from = flag(args, &format!("{name}-from"));
    let inline = flag(args, name);
    match (from, inline) {
        (Some(_), Some(_)) => Err(format!("{verb}: --{name}-from and --{name} were both given; pass one")),
        (Some(f), None) => match std::fs::read_to_string(&f) {
            Ok(t) => Ok(Some(t.trim().to_string())),
            Err(e) => Err(format!("{verb}: cannot read {f}: {e}")),
        },
        (None, inline) => Ok(inline),
    }
}

/// The argument list with every `--<name>-from FILE` of `names` rewritten into `--<name> TEXT`
/// through `prose_flag`, for a command whose prose is read DOWNSTREAM of its entry - `accept`'s
/// `--note` is read by `fold_words_into_note`, `fold_warnings_into_note` and the channel layer
/// before the command itself reads it (issue546, sprint 713). Normalising the arguments once at
/// the entry means every reader sees one form and none learns about files; the alternative,
/// teaching each of seven reads about `-from`, is the verb-by-verb path D0224 refused. Both forms
/// of one name is refused by name (the error is `prose_flag`'s) and nothing downstream runs.
///
/// # Errors
///
/// The exit code `2` after `prose_flag`'s message, for any name given in both forms or whose file cannot be read.
pub fn prose_args(args: &[String], names: &[&str], verb: &str) -> Result<Vec<String>, i32> {
    let mut out = args.to_vec();
    for name in names {
        let from_key = format!("--{name}-from");
        if !out.contains(&from_key) {
            continue;
        }
        let text = match prose_flag(&out, name, verb) {
            Ok(Some(t)) => t,
            Ok(None) => continue,
            Err(msg) => {
                eprintln!("error: {msg}");
                return Err(2);
            }
        };
        let mut next = Vec::with_capacity(out.len());
        let mut skip_value = false;
        for a in &out {
            if skip_value {
                skip_value = false;
            } else if *a == from_key {
                skip_value = true;
            } else {
                next.push(a.clone());
            }
        }
        next.push(format!("--{name}"));
        next.push(text);
        out = next;
    }
    Ok(out)
}

/// A provenance DATE, refused rather than defaulted (issue182).
///
/// Five write paths read this as `flag(args, ..).unwrap_or_else(|| "2026-01-01".to_owned())`. CLAUDE.md
/// says provenance is never defaulted; that rule was implemented for the ACTOR, where a missing actor
/// makes the write refuse, and the DATE fell back to a false constant. A result written without a date
/// claimed it happened on 2026-01-01, which corrupts any series and feeds guard 36 - whose whole job is
/// catching evidence that cites a date it could not have had.
///
/// REFUSED, not defaulted to today: an AI has no clock it can honestly attest to, and guessing is what
/// produced the constant in the first place. The caller states the date or the write does not happen.
///
/// # Errors
///
/// `2` after the usage line when the flag is absent - the date is stated or the write does not happen.
pub fn provenance_date(args: &[String], flag_name: &str, usage: &str) -> Result<String, i32> {
    flag(args, flag_name).ok_or_else(|| {
        eprintln!("error: --{flag_name} YYYY-MM-DD is required.");
        eprintln!(
"  A provenance date is never defaulted: it used to fall back to 2026-01-01."
        );
        eprintln!("usage: {usage}");
        2
    })
}

/// The first positional argument, REFUSING anything that looks like a flag (issue179).
///
/// `keel init --help` created a directory named `--help` and scaffolded a complete engine into it; 277
/// files reached this repository and were committed before the CRLF warnings gave it away. `cmd_init`
/// read `args.first()` directly and so never passed through `root_arg`, which has rejected unknown
/// flags all along - the bypass was the bug, not the parsing.
///
/// A leading `-` is refused wherever a PATH or a NAME is expected. For `init` the stakes are highest,
/// because its whole job is writing a tree to disk, so any string it accepts is a filesystem mutation.
///
/// # Errors
///
/// `2` after the usage line when there is no first argument or it looks like a flag.
pub fn positional_arg<'a>(args: &'a [String], usage: &str, what: &str) -> Result<&'a String, i32> {
    let Some(first) = args.first() else {
        eprintln!("usage: {usage}");
        return Err(2);
    };
    if first.starts_with('-') {
        eprintln!("error: `{first}` looks like a flag, not {what}.");
        eprintln!("  Refused rather than used: `keel init --help` once created a directory named");
        eprintln!("  `--help` and scaffolded an engine into it (issue179).");
        eprintln!("usage: {usage}");
        return Err(2);
    }
    Ok(first)
}

/// The repo a git-touching subcommand acts on: the first non-flag argument, else the discovered root.
#[must_use]
pub fn repo_arg(rest: &[String]) -> PathBuf {
    rest.iter()
        .find(|a| !a.starts_with("--"))
        .map_or_else(|| find_repo_root().unwrap_or_else(|| PathBuf::from(".")), PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::root_arg;

    #[test]
    fn an_unknown_flag_is_a_mistake_and_never_a_root() {
        // issue133: the whole class. A mistyped flag used to BECOME the root path (or, in the
        // skip-flags variant, be silently ignored), so a typo produced a confident wrong answer
        // somewhere downstream instead of an error where the mistake was made.
        let a = |v: &[&str]| v.iter().map(|s| (*s).to_string()).collect::<Vec<_>>();
        assert_eq!(root_arg(&a(&["--explan"]), "u", &[], 0), Err(2));
        assert_eq!(root_arg(&a(&["--explan"]), "u", &["explain"], 0), Err(2), "a NEAR-MISS of a known flag is still unknown");
        // A REAL project path, because `root_arg` now VALIDATES as well as parses (issue281): it
        // refuses a root that is not a keel project, so a synthetic `/r` no longer reaches the caller.
        // The assertions below still test what they always did — that the positional is FOUND around a
        // declared flag — they just use a root that a caller could really pass.
        let repo = keel_fs::test_support::repo_root();
        let r = repo.to_string_lossy().to_string();
        let found = |v: &[&str], pos: usize| {
            root_arg(&a(v), "u", &["explain"], pos).ok().map(|p| p.to_string_lossy().to_string())
        };
        assert_eq!(found(&["--explain", &r], 0), Some(r.clone()));
        assert_eq!(found(&[&r, "--explain"], 0), Some(r.clone()));
        // `positionals` skips the subcommand's own leading argument (`keel show view <name> [ROOT]`)
        assert_eq!(found(&["decisions", &r], 1), Some(r.clone()));
        // and a leading positional alone leaves ROOT to repo discovery, not to the positional
        assert_ne!(root_arg(&a(&["decisions"]), "u", &[], 1).map(|p| p.to_string_lossy().to_string()), Ok("decisions".to_string()));
        // The new half of the contract: a path that exists but is NOT a project is refused, never
        // answered over. This is the false green issue281 closed.
        let tmp = std::env::temp_dir();
        assert_eq!(root_arg(&a(&[&tmp.to_string_lossy()]), "u", &[], 0), Err(2), "a non-project root is refused");
    }
}
