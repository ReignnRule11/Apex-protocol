//! Minimal shared protocol primitives. Canonical wire formats are not ratified.
use serde::{Deserialize, Serialize};

/// Block height.
pub type Height = u64;

/// Legacy internal identifier retained for historical prototypes only.
///
/// Production EVM chain identity uses ChainId256. Do not use this type in
/// transaction signing, genesis identity, or EVM execution interfaces.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChainId(pub u64);

/// Production-target EVM chain identifier represented in memory as an
/// unsigned 256-bit big-endian integer.
///
/// This type does not assign a chain ID or define its wire encoding. For the
/// proposed EIP-1559 transaction envelope, the value is encoded as a canonical
/// RLP integer (minimal big-endian bytes, with zero represented by the empty
/// byte string). Genesis policy must assign a unique, immutable non-zero value
/// before a network can be launched.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ChainId256([u8; 32]);

impl ChainId256 {
    /// Constructs a chain identifier from its fixed-width big-endian bytes.
    pub const fn from_be_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the fixed-width big-endian representation.
    pub const fn to_be_bytes(self) -> [u8; 32] {
        self.0
    }

    /// Constructs a chain identifier from a u64 without truncating the
    /// production 256-bit representation.
    pub const fn from_u64(value: u64) -> Self {
        let source = value.to_be_bytes();
        let mut bytes = [0_u8; 32];
        let mut i = 0;
        while i < 8 {
            bytes[24 + i] = source[i];
            i += 1;
        }
        Self(bytes)
    }

    /// Returns true when the identifier is zero. Genesis validation is
    /// responsible for rejecting zero and duplicate network identifiers.
    pub const fn is_zero(self) -> bool {
        let mut i = 0;
        while i < 32 {
            if self.0[i] != 0 {
                return false;
            }
            i += 1;
        }
        true
    }
}

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

    #[test]
    fn chain_id_256_preserves_full_width_and_round_trips() {
        let mut bytes = [0_u8; 32];
        bytes[0] = 0x80;
        bytes[31] = 0x07;
        let chain_id = ChainId256::from_be_bytes(bytes);
        assert_eq!(chain_id.to_be_bytes(), bytes);
        assert!(!chain_id.is_zero());
    }

    #[test]
    fn chain_id_256_from_u64_is_big_endian_without_truncation() {
        let chain_id = ChainId256::from_u64(0x0102_0304_0506_0708);
        let bytes = chain_id.to_be_bytes();
        assert_eq!(&bytes[..24], &[0; 24]);
        assert_eq!(&bytes[24..], &[1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn chain_id_256_reports_zero_without_assigning_a_default() {
        assert!(ChainId256::from_be_bytes([0; 32]).is_zero());
        assert!(!ChainId256::from_u64(1).is_zero());
    }
}
