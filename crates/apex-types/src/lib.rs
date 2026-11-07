//! Minimal shared protocol primitives. Canonical wire formats are not ratified.
use serde::{Deserialize, Serialize};

/// Block height.
pub type Height = u64;

/// Network identifier; no production value has been assigned.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChainId(pub u64);

/// 32-byte commitment.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Hash32(pub [u8; 32]);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn commitment_is_32_bytes() {
        assert_eq!(Hash32([0; 32]).0.len(), 32);
    }
}
