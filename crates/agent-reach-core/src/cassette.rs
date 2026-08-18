//! Recorded responses, so the inner development loop does not need the network.
//!
//! Every search endpoint available to this project without a key is rate
//! limited, and we do not disguise the client to get around that — so
//! measurement is a scarce resource. A harness that re-queries live on every
//! iteration exhausts its own ability to measure within the hour; that is not a
//! prediction, it is what happened. Recording each response the first time and
//! replaying it afterwards makes the inner loop free, fast and deterministic.
//!
//! Off by default. With `AGENT_REACH_CASSETTE` unset every call behaves exactly
//! as before, so the production path is untouched.
//!
//! Failures are recorded too, status code and all. A `429` replayed from disk is
//! how the "throttled, not measured" path gets tested without waiting for a real
//! endpoint to refuse us.
//!
//! **Limit, stated plainly:** the key includes the query, so changing the query
//! ladder produces new keys that miss and go to the network. The cassette makes
//! repetition cheap; it does not make exploration free.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// One recorded exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    /// HTTP status, or the process exit code for a subprocess backend.
    pub status: u16,
    pub body: String,
}

/// Where recordings live, or `None` when the cassette is off.
pub fn dir() -> Option<PathBuf> {
    std::env::var_os("AGENT_REACH_CASSETTE").map(PathBuf::from)
}

/// Filename for an exchange: a readable slug plus a hash of the full key.
///
/// The slug alone would collide and cannot round-trip punctuation; the hash
/// alone would make the directory unreadable when someone needs to see which
/// query went wrong. Both, and the directory stays greppable.
pub fn key(parts: &[&str]) -> String {
    let joined = parts.join("\u{1f}");

    // FNV-1a. A cassette filename needs to be stable and collision-resistant
    // enough for a few thousand entries — not cryptographic. Reaching for a
    // hashing crate here would add a dependency for no gain.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in joined.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    let slug: String = joined
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .chars()
        .take(48)
        .collect();

    format!("{slug}-{hash:016x}.json")
}

/// Replay a recorded exchange, if the cassette is on and holds this key.
pub fn load(key: &str) -> Option<Recording> {
    let path = dir()?.join(key);
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

/// Record an exchange. Silent when the cassette is off.
///
/// A write failure is not propagated: recording is an optimisation, and a
/// read-only directory must not turn a working search into a failed one.
pub fn save(key: &str, recording: &Recording) {
    let Some(dir) = dir() else { return };
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    if let Ok(json) = serde_json::to_string_pretty(recording) {
        let _ = std::fs::write(dir.join(key), json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_is_stable_and_readable() {
        let a = key(&["github", "search", "rust http client"]);
        assert_eq!(a, key(&["github", "search", "rust http client"]));
        assert!(a.starts_with("github-search-rust-http-client-"), "{a}");
        assert!(a.ends_with(".json"));
    }

    #[test]
    fn key_separates_fields_that_would_otherwise_merge() {
        // Without a separator, ("ab","c") and ("a","bc") would hash alike and
        // two different queries would share one recording.
        assert_ne!(key(&["ab", "c"]), key(&["a", "bc"]));
    }

    #[test]
    fn key_is_a_legal_filename() {
        let k = key(&["ddg", "search", "hızlı arama: x/y?z=1&w"]);
        let illegal = ['/', '\\', ':', '?', '*', '<', '>', '|'];
        assert!(!k.contains(illegal), "{k}");
    }

    #[test]
    fn load_is_none_when_cassette_is_off() {
        // The production path must be untouched when the variable is unset.
        if std::env::var_os("AGENT_REACH_CASSETTE").is_none() {
            assert!(load("anything.json").is_none());
        }
    }
}
