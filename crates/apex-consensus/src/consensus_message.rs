//! Canonical, domain-separated consensus signing preimages (experimental).
//! This module constructs bytes only; it does not sign or verify them.

use thiserror::Error;

const DOMAIN: &[u8] = b"APEX-CONSENSUS-MSG-V1";
const VERSION: u8 = 1;
const ENCODED_LEN: usize = DOMAIN.len() + 1 + 32 + 4 + 8 + 4 + 1 + 32 + 32 + 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ConsensusMessageStep {
    Proposal = 1,
    Prevote = 2,
    Precommit = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum ConsensusMessageError {
    #[error("chain ID must not be zero")]
    ZeroChainId,
    #[error("protocol version must be non-zero")]
    ZeroProtocolVersion,
    #[error("signer identity must not be zero")]
    ZeroSigner,
    #[error("validator-set hash must not be zero")]
    ZeroValidatorSetHash,
    #[error("proposal must identify a non-zero block hash")]
    NilProposal,
}

/// Fixed-width message fields. Integers are encoded big-endian. A zero block
/// hash denotes a nil prevote/precommit only. This profile is not ratified.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConsensusSigningMessage {
    chain_id: [u8; 32],
    protocol_version: u32,
    height: u64,
    round: u32,
    step: ConsensusMessageStep,
    signer_id: [u8; 32],
    validator_set_hash: [u8; 32],
    block_hash: [u8; 32],
}

impl ConsensusSigningMessage {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        chain_id: [u8; 32],
        protocol_version: u32,
        height: u64,
        round: u32,
        step: ConsensusMessageStep,
        signer_id: [u8; 32],
        validator_set_hash: [u8; 32],
        block_hash: [u8; 32],
    ) -> Result<Self, ConsensusMessageError> {
        if chain_id == [0; 32] {
            return Err(ConsensusMessageError::ZeroChainId);
        }
        if protocol_version == 0 {
            return Err(ConsensusMessageError::ZeroProtocolVersion);
        }
        if signer_id == [0; 32] {
            return Err(ConsensusMessageError::ZeroSigner);
        }
        if validator_set_hash == [0; 32] {
            return Err(ConsensusMessageError::ZeroValidatorSetHash);
        }
        if step == ConsensusMessageStep::Proposal && block_hash == [0; 32] {
            return Err(ConsensusMessageError::NilProposal);
        }
        Ok(Self {
            chain_id,
            protocol_version,
            height,
            round,
            step,
            signer_id,
            validator_set_hash,
            block_hash,
        })
    }

    /// Exact fixed-width signing preimage; no variable-length fields exist.
    pub fn signing_bytes(&self) -> [u8; ENCODED_LEN] {
        let mut out = [0_u8; ENCODED_LEN];
        let mut offset = 0;
        out[offset..offset + DOMAIN.len()].copy_from_slice(DOMAIN);
        offset += DOMAIN.len();
        out[offset] = VERSION;
        offset += 1;
        out[offset..offset + 32].copy_from_slice(&self.chain_id);
        offset += 32;
        out[offset..offset + 4].copy_from_slice(&self.protocol_version.to_be_bytes());
        offset += 4;
        out[offset..offset + 8].copy_from_slice(&self.height.to_be_bytes());
        offset += 8;
        out[offset..offset + 4].copy_from_slice(&self.round.to_be_bytes());
        offset += 4;
        out[offset] = self.step as u8;
        offset += 1;
        out[offset..offset + 32].copy_from_slice(&self.signer_id);
        offset += 32;
        out[offset..offset + 32].copy_from_slice(&self.validator_set_hash);
        offset += 32;
        out[offset..offset + 32].copy_from_slice(&self.block_hash);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(step: ConsensusMessageStep, block: [u8; 32]) -> ConsensusSigningMessage {
        ConsensusSigningMessage::new([1; 32], 1, 7, 2, step, [2; 32], [3; 32], block).unwrap()
    }

    #[test]
    fn encoding_is_fixed_width_and_domain_separated() {
        let bytes = msg(ConsensusMessageStep::Prevote, [4; 32]).signing_bytes();
        assert_eq!(bytes.len(), ENCODED_LEN);
        assert!(bytes.starts_with(DOMAIN));
        assert_eq!(bytes[DOMAIN.len()], VERSION);
    }

    #[test]
    fn step_and_block_hash_are_bound() {
        let base = msg(ConsensusMessageStep::Prevote, [4; 32]).signing_bytes();
        assert_ne!(
            base,
            msg(ConsensusMessageStep::Precommit, [4; 32]).signing_bytes()
        );
        assert_ne!(
            base,
            msg(ConsensusMessageStep::Prevote, [5; 32]).signing_bytes()
        );
    }

    #[test]
    fn nil_votes_are_allowed_but_nil_proposals_are_rejected() {
        assert_eq!(
            msg(ConsensusMessageStep::Prevote, [0; 32])
                .signing_bytes()
                .len(),
            ENCODED_LEN
        );
        assert_eq!(
            ConsensusSigningMessage::new(
                [1; 32],
                1,
                1,
                0,
                ConsensusMessageStep::Proposal,
                [2; 32],
                [3; 32],
                [0; 32]
            ),
            Err(ConsensusMessageError::NilProposal)
        );
    }

    #[test]
    fn rejects_zero_context() {
        assert_eq!(
            ConsensusSigningMessage::new(
                [0; 32],
                1,
                1,
                0,
                ConsensusMessageStep::Prevote,
                [2; 32],
                [3; 32],
                [4; 32]
            ),
            Err(ConsensusMessageError::ZeroChainId)
        );
        assert_eq!(
            ConsensusSigningMessage::new(
                [1; 32],
                0,
                1,
                0,
                ConsensusMessageStep::Prevote,
                [2; 32],
                [3; 32],
                [4; 32]
            ),
            Err(ConsensusMessageError::ZeroProtocolVersion)
        );
    }
}
