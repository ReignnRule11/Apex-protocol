//! Canonical genesis protocol encoding and commitment.
//!
//! Commits to protocol identity and a structurally validated candidate
//! validator set. It does not authenticate validator keys or authorize
//! membership, governance, consensus, or network activation.

use apex_consensus::validator_set::ValidatorSet;
use apex_types::ChainId256;
use sha2::{Digest, Sha256};
use thiserror::Error;

const MAGIC: &[u8; 4] = b"APEX";
const SCHEMA_VERSION: u16 = 1;
const DOMAIN: &[u8] = b"APEX::GENESIS_PROTOCOL_COMMITMENT::V1\0";
const HEADER_LEN: usize = 4 + 2 + 32 + 4 + 4;
const VALIDATOR_LEN: usize = 32 + 8;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenesisProtocolConfig {
    pub chain_id: ChainId256,
    pub protocol_version: u32,
    pub validators: ValidatorSet,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenesisProtocolCommitment {
    pub canonical_bytes: Vec<u8>,
    pub digest: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum GenesisCommitmentError {
    #[error("genesis chain ID must not be zero")]
    ZeroChainId,
    #[error("genesis protocol version must be positive")]
    InvalidProtocolVersion,
    #[error("validator count exceeds the canonical u32 limit")]
    TooManyValidators,
}

impl GenesisProtocolConfig {
    /// Layout: magic[4] || schema:u16-BE || chain_id[32] ||
    /// protocol_version:u32-BE || count:u32-BE ||
    /// repeated(validator_id[32] || voting_power:u64-BE).
    /// ValidatorSet guarantees ascending identity order and rejects invalid sets.
    pub fn encode_canonical(&self) -> Result<Vec<u8>, GenesisCommitmentError> {
        if self.chain_id.is_zero() {
            return Err(GenesisCommitmentError::ZeroChainId);
        }
        if self.protocol_version == 0 {
            return Err(GenesisCommitmentError::InvalidProtocolVersion);
        }
        let validators = self.validators.validators();
        let count = u32::try_from(validators.len())
            .map_err(|_| GenesisCommitmentError::TooManyValidators)?;
        let capacity = HEADER_LEN
            .checked_add(
                validators
                    .len()
                    .checked_mul(VALIDATOR_LEN)
                    .ok_or(GenesisCommitmentError::TooManyValidators)?,
            )
            .ok_or(GenesisCommitmentError::TooManyValidators)?;
        let mut out = Vec::with_capacity(capacity);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&SCHEMA_VERSION.to_be_bytes());
        out.extend_from_slice(&self.chain_id.to_be_bytes());
        out.extend_from_slice(&self.protocol_version.to_be_bytes());
        out.extend_from_slice(&count.to_be_bytes());
        for validator in validators {
            out.extend_from_slice(&validator.id.0);
            out.extend_from_slice(&validator.voting_power.to_be_bytes());
        }
        Ok(out)
    }

    /// SHA-256(DOMAIN || canonical_bytes), with a protocol-specific domain.
    pub fn commitment(&self) -> Result<GenesisProtocolCommitment, GenesisCommitmentError> {
        let canonical_bytes = self.encode_canonical()?;
        let mut hasher = Sha256::new();
        hasher.update(DOMAIN);
        hasher.update(&canonical_bytes);
        let digest: [u8; 32] = hasher.finalize().into();
        Ok(GenesisProtocolCommitment {
            canonical_bytes,
            digest,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apex_consensus::validator_set::{Validator, ValidatorId};

    fn config(chain: [u8; 32], version: u32, entries: &[(u8, u64)]) -> GenesisProtocolConfig {
        let validators = ValidatorSet::new(
            entries
                .iter()
                .map(|(id, voting_power)| Validator {
                    id: ValidatorId([*id; 32]),
                    voting_power: *voting_power,
                })
                .collect(),
        )
        .unwrap();
        GenesisProtocolConfig {
            chain_id: ChainId256::from_be_bytes(chain),
            protocol_version: version,
            validators,
        }
    }

    #[test]
    fn canonical_encoding_uses_fixed_width_big_endian_fields() {
        let value = config([1; 32], 0x0102_0304, &[(2, 9), (1, 7)]);
        let bytes = value.encode_canonical().unwrap();
        assert_eq!(&bytes[..6], b"APEX\0\x01");
        assert_eq!(&bytes[6..38], &[1; 32]);
        assert_eq!(&bytes[38..42], &[1, 2, 3, 4]);
        assert_eq!(&bytes[42..46], &[0, 0, 0, 2]);
        assert_eq!(&bytes[46..78], &[1; 32]);
        assert_eq!(&bytes[78..86], &7_u64.to_be_bytes());
        assert_eq!(&bytes[86..118], &[2; 32]);
        assert_eq!(&bytes[118..126], &9_u64.to_be_bytes());
        assert_eq!(bytes.len(), HEADER_LEN + 2 * VALIDATOR_LEN);
    }

    #[test]
    fn canonical_bytes_and_commitment_ignore_input_order() {
        let a = config([1; 32], 1, &[(3, 2), (1, 4), (2, 3)]);
        let b = config([1; 32], 1, &[(2, 3), (3, 2), (1, 4)]);
        assert_eq!(a.encode_canonical().unwrap(), b.encode_canonical().unwrap());
        assert_eq!(a.commitment().unwrap(), b.commitment().unwrap());
    }

    #[test]
    fn commitment_changes_for_each_committed_field() {
        let base = config([1; 32], 1, &[(1, 4), (2, 3)]).commitment().unwrap();
        let chain = config([2; 32], 1, &[(1, 4), (2, 3)]).commitment().unwrap();
        let version = config([1; 32], 2, &[(1, 4), (2, 3)]).commitment().unwrap();
        let power = config([1; 32], 1, &[(1, 5), (2, 3)]).commitment().unwrap();
        assert_ne!(base.digest, chain.digest);
        assert_ne!(base.digest, version.digest);
        assert_ne!(base.digest, power.digest);
    }

    #[test]
    fn rejects_zero_chain_and_protocol_version() {
        assert_eq!(
            config([0; 32], 1, &[(1, 1)]).encode_canonical(),
            Err(GenesisCommitmentError::ZeroChainId)
        );
        assert_eq!(
            config([1; 32], 0, &[(1, 1)]).encode_canonical(),
            Err(GenesisCommitmentError::InvalidProtocolVersion)
        );
    }

    #[test]
    fn digest_is_domain_separated_sha256() {
        let commitment = config([1; 32], 1, &[(1, 1)]).commitment().unwrap();
        let mut hasher = Sha256::new();
        hasher.update(DOMAIN);
        hasher.update(&commitment.canonical_bytes);
        let expected: [u8; 32] = hasher.finalize().into();
        assert_eq!(commitment.digest, expected);
    }
}
