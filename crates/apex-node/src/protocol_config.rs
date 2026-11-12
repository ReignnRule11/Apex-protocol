//! Strict parsing and structural validation for node protocol configuration.
//!
//! This configuration is an operator-supplied candidate validator set. It is
//! not a genesis commitment, governance decision, authenticated key registry,
//! or proof that any validator controls a signing key.

use apex_consensus::validator_set::{Validator, ValidatorId, ValidatorSet, ValidatorSetError};
use apex_types::ChainId256;
use std::env;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeProtocolConfig {
    pub chain_id: ChainId256,
    pub protocol_version: u32,
    pub validators: ValidatorSet,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum NodeProtocolConfigError {
    #[error("required environment variable {0} is missing")]
    Missing(&'static str),
    #[error("chain ID must be exactly 64 hexadecimal characters")]
    InvalidChainId,
    #[error("chain ID must not be zero")]
    ZeroChainId,
    #[error("protocol version must be a positive u32")]
    InvalidProtocolVersion,
    #[error("validator list must not be empty")]
    EmptyValidatorList,
    #[error("each validator entry must use 64-hex-character-id:positive-voting-power")]
    InvalidValidatorEntry,
    #[error("validator set is structurally invalid: {0}")]
    InvalidValidatorSet(ValidatorSetError),
}

impl NodeProtocolConfig {
    /// Read required protocol identity and candidate validator-set settings.
    pub fn from_env() -> Result<Self, NodeProtocolConfigError> {
        let chain_id = env::var("APEX_CHAIN_ID")
            .map_err(|_| NodeProtocolConfigError::Missing("APEX_CHAIN_ID"))?;
        let protocol_version = env::var("APEX_PROTOCOL_VERSION")
            .map_err(|_| NodeProtocolConfigError::Missing("APEX_PROTOCOL_VERSION"))?;
        let validators = env::var("APEX_VALIDATORS")
            .map_err(|_| NodeProtocolConfigError::Missing("APEX_VALIDATORS"))?;
        Self::parse(&chain_id, &protocol_version, &validators)
    }

    /// Parse a 256-bit chain ID, positive protocol version and CSV validator set.
    ///
    /// Validator entries use `<64 hex chars>:<u64 voting power>`.
    pub fn parse(
        chain_id: &str,
        protocol_version: &str,
        validators: &str,
    ) -> Result<Self, NodeProtocolConfigError> {
        if chain_id.len() != 64 {
            return Err(NodeProtocolConfigError::InvalidChainId);
        }
        let chain_bytes: [u8; 32] = hex::decode(chain_id)
            .map_err(|_| NodeProtocolConfigError::InvalidChainId)?
            .try_into()
            .map_err(|_| NodeProtocolConfigError::InvalidChainId)?;
        let chain_id = ChainId256::from_be_bytes(chain_bytes);
        if chain_id.is_zero() {
            return Err(NodeProtocolConfigError::ZeroChainId);
        }

        let protocol_version = protocol_version
            .parse::<u32>()
            .ok()
            .filter(|version| *version > 0)
            .ok_or(NodeProtocolConfigError::InvalidProtocolVersion)?;

        if validators.is_empty() {
            return Err(NodeProtocolConfigError::EmptyValidatorList);
        }
        let entries = validators
            .split(',')
            .map(|entry| {
                let (identity, power) = entry
                    .split_once(':')
                    .ok_or(NodeProtocolConfigError::InvalidValidatorEntry)?;
                if identity.len() != 64 {
                    return Err(NodeProtocolConfigError::InvalidValidatorEntry);
                }
                let identity: [u8; 32] = hex::decode(identity)
                    .map_err(|_| NodeProtocolConfigError::InvalidValidatorEntry)?
                    .try_into()
                    .map_err(|_| NodeProtocolConfigError::InvalidValidatorEntry)?;
                let voting_power = power
                    .parse::<u64>()
                    .ok()
                    .filter(|power| *power > 0)
                    .ok_or(NodeProtocolConfigError::InvalidValidatorEntry)?;
                Ok(Validator {
                    id: ValidatorId(identity),
                    voting_power,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let validators =
            ValidatorSet::new(entries).map_err(NodeProtocolConfigError::InvalidValidatorSet)?;

        Ok(Self {
            chain_id,
            protocol_version,
            validators,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_config() -> NodeProtocolConfig {
        NodeProtocolConfig::parse(
            &"01".repeat(32),
            "1",
            &format!("{}:10,{}:20", "02".repeat(32), "03".repeat(32)),
        )
        .unwrap()
    }

    #[test]
    fn parses_full_width_chain_and_canonical_validator_set() {
        let config = valid_config();
        assert_eq!(config.chain_id.to_be_bytes(), [1; 32]);
        assert_eq!(config.protocol_version, 1);
        assert_eq!(config.validators.validators().len(), 2);
        assert_eq!(config.validators.total_voting_power(), 30);
        assert_eq!(config.validators.validators()[0].id, ValidatorId([2; 32]));
    }

    #[test]
    fn rejects_zero_or_malformed_chain_identity() {
        assert_eq!(
            NodeProtocolConfig::parse(&"00".repeat(32), "1", &format!("{}:1", "01".repeat(32))),
            Err(NodeProtocolConfigError::ZeroChainId)
        );
        assert_eq!(
            NodeProtocolConfig::parse("abcd", "1", &format!("{}:1", "01".repeat(32))),
            Err(NodeProtocolConfigError::InvalidChainId)
        );
    }

    #[test]
    fn rejects_zero_or_malformed_protocol_version() {
        for version in ["0", "-1", "4294967296", "v1"] {
            assert_eq!(
                NodeProtocolConfig::parse(
                    &"01".repeat(32),
                    version,
                    &format!("{}:1", "02".repeat(32))
                ),
                Err(NodeProtocolConfigError::InvalidProtocolVersion)
            );
        }
    }

    #[test]
    fn rejects_empty_malformed_zero_power_and_duplicate_validators() {
        let id = "02".repeat(32);
        assert_eq!(
            NodeProtocolConfig::parse(&"01".repeat(32), "1", ""),
            Err(NodeProtocolConfigError::EmptyValidatorList)
        );
        assert!(matches!(
            NodeProtocolConfig::parse(&"01".repeat(32), "1", "bad"),
            Err(NodeProtocolConfigError::InvalidValidatorEntry)
        ));
        assert!(matches!(
            NodeProtocolConfig::parse(&"01".repeat(32), "1", &format!("{id}:0")),
            Err(NodeProtocolConfigError::InvalidValidatorEntry)
        ));
        assert_eq!(
            NodeProtocolConfig::parse(&"01".repeat(32), "1", &format!("{id}:1,{id}:2")),
            Err(NodeProtocolConfigError::InvalidValidatorSet(
                ValidatorSetError::DuplicateIdentity
            ))
        );
    }

    #[test]
    fn rejects_zero_identity_and_voting_power_overflow() {
        assert_eq!(
            NodeProtocolConfig::parse(&"01".repeat(32), "1", &format!("{}:1", "00".repeat(32))),
            Err(NodeProtocolConfigError::InvalidValidatorSet(
                ValidatorSetError::InvalidIdentity
            ))
        );
        assert_eq!(
            NodeProtocolConfig::parse(
                &"01".repeat(32),
                "1",
                &format!("{}:{},{}:1", "02".repeat(32), u64::MAX, "03".repeat(32))
            ),
            Err(NodeProtocolConfigError::InvalidValidatorSet(
                ValidatorSetError::VotingPowerOverflow
            ))
        );
    }
}
