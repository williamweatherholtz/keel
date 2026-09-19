//! The atomic file write every keel writer passes through (issue184 / issue366).
//!
//! A D0479 leaf member: it depends on the parser alone, so the read model's fact cache and the write
//! API share one choke point without either depending on the other (sprint 718).

#![forbid(unsafe_code)]
#![deny(warnings, clippy::all, clippy::pedantic, clippy::nursery)]
// D0074 fail-loud: authority-bearing code has no silent failure paths.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented
)]
// Tests may use unwrap/expect/panic/indexing/asserts freely.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing))]

pub mod fsx;
pub mod test_support;
// The verbs this member owns, out of main.rs (D0479, sprint 750).
pub mod fs_verbs;
pub use fsx::scratch;

/// The character length of the longest path under `dir`, used by `keel init` to warn before a host
/// path limit turns into an opaque `git add` failure (issue313).
#[must_use]
pub fn walk_longest(dir: &std::path::Path) -> usize {
    let mut longest = 0;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = entry.path();
            longest = longest.max(p.to_string_lossy().chars().count());
            if p.is_dir() {
                stack.push(p);
            }
        }
    }
    longest
}
