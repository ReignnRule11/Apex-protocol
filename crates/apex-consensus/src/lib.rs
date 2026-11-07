//! Consensus-related primitives only. This crate does not implement BFT consensus.
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
pub fn has_supermajority(
    signed: VotingPower,
    total: VotingPower,
) -> Result<bool, QuorumError> {
    if total.0 == 0 {
        return Err(QuorumError::ZeroTotal);
    }
    if signed.0 > total.0 {
        return Err(QuorumError::SignedPowerExceedsTotal);
    }

    Ok(u128::from(signed.0) * 3 > u128::from(total.0) * 2)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
