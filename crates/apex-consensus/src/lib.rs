//! Consensus primitives and engine-neutral protocol boundary checks.
//!
//! This crate does not implement BFT consensus, authenticate validator votes,
//! verify commit certificates, execute transactions, or establish finality.
//! Engine adapters must treat all engine-provided data as untrusted until the
//! selected engine's verification path has completed.

use apex_types::{ChainId, Hash32, Height};
use thiserror::Error;

/// Voting power represented as an unsigned integer.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VotingPower(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum QuorumError {
    #[error("total voting power must be greater than zero")]
    ZeroTotal,
    #[error("signed voting power cannot exceed total voting power")]
    SignedPowerExceedsTotal,
}

/// Returns true only when signed power is strictly greater than two-thirds
/// of total power. This arithmetic helper is not a consensus implementation.
pub fn has_supermajority(signed: VotingPower, total: VotingPower) -> Result<bool, QuorumError> {
    if total.0 == 0 {
        return Err(QuorumError::ZeroTotal);
    }
    if signed.0 > total.0 {
        return Err(QuorumError::SignedPowerExceedsTotal);
    }

    Ok(u128::from(signed.0) * 3 > u128::from(total.0) * 2)
}

/// Minimal parent context against which a proposed block header is checked.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExpectedParent {
    pub chain_id: ChainId,
    pub protocol_version: u32,
    pub height: Height,
    pub block_hash: Hash32,
}

/// Header fields needed for deterministic chain/height/parent preflight.
/// This is not a canonical wire format and does not define a block hash.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProposalHeader {
    pub chain_id: ChainId,
    pub protocol_version: u32,
    pub height: Height,
    pub parent_hash: Hash32,
}

/// Fail-closed errors for proposal identity and linkage checks.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum ProposalHeaderError {
    #[error("proposal chain ID does not match the configured chain")]
    WrongChain,
    #[error("proposal protocol version does not match the configured version")]
    WrongProtocolVersion,
    #[error("parent height cannot be incremented")]
    ParentHeightOverflow,
    #[error("proposal height is not the next height after the expected parent")]
    WrongHeight,
    #[error("proposal parent hash does not match the expected parent")]
    WrongParent,
}

/// Checks only chain identity, protocol version, sequential height and parent
/// linkage. It does not authenticate the proposer, validate transactions,
/// verify a BFT certificate, or establish finality.
pub fn validate_proposal_header(
    expected: ExpectedParent,
    proposal: ProposalHeader,
) -> Result<(), ProposalHeaderError> {
    if proposal.chain_id != expected.chain_id {
        return Err(ProposalHeaderError::WrongChain);
    }
    if proposal.protocol_version != expected.protocol_version {
        return Err(ProposalHeaderError::WrongProtocolVersion);
    }

    let next_height = expected
        .height
        .checked_add(1)
        .ok_or(ProposalHeaderError::ParentHeightOverflow)?;
    if proposal.height != next_height {
        return Err(ProposalHeaderError::WrongHeight);
    }
    if proposal.parent_hash != expected.block_hash {
        return Err(ProposalHeaderError::WrongParent);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expected_parent() -> ExpectedParent {
        ExpectedParent {
            chain_id: ChainId(7),
            protocol_version: 1,
            height: 10,
            block_hash: Hash32([0x11; 32]),
        }
    }

    fn proposal() -> ProposalHeader {
        ProposalHeader {
            chain_id: ChainId(7),
            protocol_version: 1,
            height: 11,
            parent_hash: Hash32([0x11; 32]),
        }
    }

    #[test]
    fn requires_strictly_more_than_two_thirds() {
        assert!(!has_supermajority(VotingPower(2), VotingPower(3)).unwrap());
        assert!(has_supermajority(VotingPower(3), VotingPower(4)).unwrap());
    }

    #[test]
    fn rejects_zero_total() {
        assert_eq!(
            has_supermajority(VotingPower(0), VotingPower(0)),
            Err(QuorumError::ZeroTotal)
        );
    }

    #[test]
    fn rejects_signed_power_above_total() {
        assert_eq!(
            has_supermajority(VotingPower(5), VotingPower(4)),
            Err(QuorumError::SignedPowerExceedsTotal)
        );
    }

    #[test]
    fn arithmetic_handles_maximum_u64_values() {
        assert!(has_supermajority(VotingPower(u64::MAX), VotingPower(u64::MAX)).unwrap());
    }

    #[test]
    fn accepts_matching_chain_version_height_and_parent() {
        assert_eq!(validate_proposal_header(expected_parent(), proposal()), Ok(()));
    }

    #[test]
    fn rejects_wrong_chain() {
        let mut candidate = proposal();
        candidate.chain_id = ChainId(8);
        assert_eq!(
            validate_proposal_header(expected_parent(), candidate),
            Err(ProposalHeaderError::WrongChain)
        );
    }

    #[test]
    fn rejects_wrong_protocol_version() {
        let mut candidate = proposal();
        candidate.protocol_version = 2;
        assert_eq!(
            validate_proposal_header(expected_parent(), candidate),
            Err(ProposalHeaderError::WrongProtocolVersion)
        );
    }

    #[test]
    fn rejects_non_sequential_height() {
        let mut candidate = proposal();
        candidate.height = 12;
        assert_eq!(
            validate_proposal_header(expected_parent(), candidate),
            Err(ProposalHeaderError::WrongHeight)
        );
    }

    #[test]
    fn rejects_wrong_parent_hash() {
        let mut candidate = proposal();
        candidate.parent_hash = Hash32([0x22; 32]);
        assert_eq!(
            validate_proposal_header(expected_parent(), candidate),
            Err(ProposalHeaderError::WrongParent)
        );
    }

    #[test]
    fn rejects_height_overflow() {
        let mut parent = expected_parent();
        parent.height = Height::MAX;
        assert_eq!(
            validate_proposal_header(parent, proposal()),
            Err(ProposalHeaderError::ParentHeightOverflow)
        );
    }
}
