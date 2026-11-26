// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! SHA-256 digests: spec hashes, data fingerprints and Score hashes (ADR-0013).

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest as _, Sha256};

/// A SHA-256 digest, written as 64 lowercase hexadecimal digits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sha256Digest([u8; 32]);

impl Sha256Digest {
    /// The all-zero digest, a placeholder for "not computed yet".
    pub const ZERO: Self = Self([0; 32]);

    /// The SHA-256 digest of `bytes`.
    #[must_use]
    pub fn of(bytes: &[u8]) -> Self {
        Self(Sha256::digest(bytes).into())
    }

    /// The 32 digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// SHA-256 computed incrementally: the digest of everything passed to
/// [`update`](Self::update), in order, without holding it all at once (the value-based data
/// fingerprint, RFC-0008).
#[derive(Clone, Default)]
pub struct Sha256Hasher(Sha256);

impl Sha256Hasher {
    /// A hasher that has seen nothing yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds `bytes` to the input.
    pub fn update(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }

    /// The digest of the whole input.
    #[must_use]
    pub fn finish(self) -> Sha256Digest {
        Sha256Digest(self.0.finalize().into())
    }
}

impl fmt::Debug for Sha256Hasher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Sha256Hasher")
    }
}

impl fmt::Display for Sha256Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// The error for text that is not 64 lowercase hexadecimal digits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("a SHA-256 digest is 64 lowercase hexadecimal digits")]
pub struct InvalidDigest;

impl FromStr for Sha256Digest {
    type Err = InvalidDigest;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let hex = text.as_bytes();
        if hex.len() != 64 {
            return Err(InvalidDigest);
        }
        let mut bytes = [0; 32];
        let (pairs, _) = hex.as_chunks::<2>();
        for (byte, [high, low]) in bytes.iter_mut().zip(pairs) {
            *byte = (nibble(*high)? << 4) | nibble(*low)?;
        }
        Ok(Self(bytes))
    }
}

fn nibble(digit: u8) -> Result<u8, InvalidDigest> {
    match digit {
        b'0'..=b'9' => Ok(digit - b'0'),
        b'a'..=b'f' => Ok(digit - b'a' + 10),
        _ => Err(InvalidDigest),
    }
}

impl Serialize for Sha256Digest {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Sha256Digest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = <std::borrow::Cow<'de, str>>::deserialize(deserializer)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_of_empty_input_matches_the_standard_vector() {
        assert_eq!(
            Sha256Digest::of(b"").to_string(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn an_incremental_digest_equals_the_digest_of_the_whole_input() {
        let mut hasher = Sha256Hasher::new();
        hasher.update(b"ab");
        hasher.update(b"");
        hasher.update(b"c");
        assert_eq!(hasher.finish(), Sha256Digest::of(b"abc"));
    }

    #[test]
    fn hex_round_trips_and_rejects_bad_text() {
        let digest = Sha256Digest::of(b"abc");
        let text = digest.to_string();
        assert_eq!(text.parse::<Sha256Digest>(), Ok(digest));
        assert_eq!(
            text.to_uppercase().parse::<Sha256Digest>(),
            Err(InvalidDigest)
        );
        assert_eq!("abc".parse::<Sha256Digest>(), Err(InvalidDigest));
        let json = serde_json::to_string(&digest).unwrap();
        assert_eq!(serde_json::from_str::<Sha256Digest>(&json).unwrap(), digest);
    }
}
