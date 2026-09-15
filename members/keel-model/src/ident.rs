//! Identity = an immutable UUID `id` (Invariant 3): minting one and recognising one.
//!
//! `gen_uuid` is what every write mints; the three predicates are what the id guards and the attestation
//! view recognise. One leaf, so neither the write API nor a view has to reach into guards for the shape of
//! an id (D0479, dcGuardsViewOrientCycleIsBroken).

use std::fmt::Write as _;

/// Generate a cryptographically-random UUID v4 (RFC 4122), 122 bits of OS entropy.
///
/// # Distributed safety (issue075 / D0129)
///
/// The previous construction mixed only clock seconds, sub-second nanos, PID and an in-process
/// counter that was 0 for the first record of every invocation. It had **no host component**, so
/// two machines were not independent sources — and `id` is the engine's identity invariant (items
/// never collide on name precisely because they are distinguished by id, CLAUDE.md §2.3), so a
/// collision corrupts identity itself. With no duplicate-id detector (issue074) it would also be
/// undetectable. Entropy now comes from the OS CSPRNG.
///
/// # Panics
///
/// If the OS CSPRNG is unavailable. That is deliberate: minting a weak identity silently is worse
/// than failing loudly (the honest-gate principle, D0098).
#[must_use]
#[allow(clippy::expect_used)] // deliberate: a weak identity minted silently is worse than a loud abort
pub fn gen_uuid() -> String {
    let mut b = [0u8; 16];
    getrandom::fill(&mut b).expect("OS CSPRNG unavailable — refusing to mint a weak identity");
    b[6] = (b[6] & 0x0f) | 0x40; // version 4
    b[8] = (b[8] & 0x3f) | 0x80; // variant RFC 4122
    let mut s = String::with_capacity(36);
    for (i, byte) in b.iter().enumerate() {
        if matches!(i, 4 | 6 | 8 | 10) {
            s.push('-');
        }
        let _ = write!(s, "{byte:02x}");
    }
    s
}

/// 8-4-4-4-12 groups of `[0-9a-z]`, exactly. Written out rather than regexed because the guard path
/// stays dependency-light, and because the group lengths ARE the specification.
#[must_use]
pub fn uuid_shaped(v: &str) -> bool {
    let groups: Vec<&str> = v.split('-').collect();
    groups.len() == 5
        && [8usize, 4, 4, 4, 12].iter().zip(&groups).all(|(want, g)| {
            g.len() == *want && g.chars().all(|c| c.is_ascii_digit() || c.is_ascii_lowercase())
        })
}

/// 8-4-4-4-12 groups of lowercase hex - shaped AND hexadecimal, any version.
#[must_use]
pub fn uuid_hex_shaped(v: &str) -> bool {
    let groups: Vec<&str> = v.split('-').collect();
    groups.len() == 5
        && [8usize, 4, 4, 4, 12].iter().zip(&groups).all(|(want, g)| {
            g.len() == *want && g.chars().all(|c| c.is_ascii_digit() || matches!(c, 'a'..='f'))
        })
}

/// An RFC 4122 version-4 UUID exactly as `write::gen_uuid` emits one: lowercase hex, version
/// nibble `4`, variant nibble in `89ab` (D0430 / issue454). The fingerprint of an API-written id.
#[must_use]
pub fn is_v4_uuid(v: &str) -> bool {
    uuid_hex_shaped(v) && v.as_bytes().get(14) == Some(&b'4') && matches!(v.as_bytes().get(19), Some(b'8' | b'9' | b'a' | b'b'))
}
