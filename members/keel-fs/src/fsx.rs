//! Write a file atomically: temp file beside the target, parse check for `.sysml`, then rename.
//! Out of `write.rs` in sprint 718 (D0479) so the read model's caches and the write API share it.

/// Write `content` to `path` ATOMICALLY: a sibling temp file, then a rename over the target (issue184).
///
/// `std::fs::write` truncates and then writes, so the target is momentarily EMPTY and then progressively
/// filled. A death in between - a kill, an OOM, a watchdog exit from another thread - leaves the
/// authoritative record truncated. Invariant 1 makes these files the TRUTH, and every one of the 21 write
/// sites in this crate reached that truth non-atomically.
///
/// THE DANGEROUS CASE IS NOT THE OBVIOUS ONE. A truncated file fails the parser, so the gate converts
/// corruption into a red gate. The case that survives is a PARTIAL write that still parses - these files
/// are lists of independent items, so a prefix is often syntactically complete once a closing brace
/// happens to land, and that file passes the gate with items silently missing.
///
/// The temp file is a SIBLING, not in a temp directory, because rename is only atomic within a
/// filesystem. On Windows `fs::rename` fails if the target exists, so the target is removed first - a
/// narrow window that is still strictly better than truncate-then-fill, and the temp file survives a
/// failure at that point rather than the original being gone.
///
/// # Errors
/// Returns the underlying [`std::io::Error`] if the temp write, the removal or the rename fails - or,
/// for a `.sysml` target, an `InvalidData` error when the content would not parse: the write is
/// REFUSED and nothing on disk changes (issue366 / D0305). A record command that reports success over
/// a file `validate` rejects is worse than the failure it hides, so this is the one choke point every
/// API writer of model text passes through and the one place the refusal can live.
pub fn write_atomic(path: &std::path::Path, content: impl AsRef<str>) -> std::io::Result<()> {
    if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("sysml")) {
        let name = path.display().to_string();
        let parsed = keel_parser::tokenize(content.as_ref(), &name)
            .map_err(|e| e.to_string())
            .and_then(|t| keel_parser::parse(t, &name).map(|_| ()).map_err(|e| e.to_string()));
        if let Err(e) = parsed {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("refusing to write {name}: the result would not parse - {e} (issue366: a write API never reports success over a file validate rejects)"),
            ));
        }
    }
    let tmp = path.with_extension(format!(
        "{}.keel-tmp",
        path.extension().map_or_else(String::new, |e| e.to_string_lossy().to_string())
    ));
    std::fs::write(&tmp, content.as_ref())?;
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    match std::fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(e) => {
            // Leave the temp file in place on failure: it holds the ONLY copy of the new content, and
            // deleting it here would turn a failed write into a lost write.
            Err(e)
        }
    }
}

