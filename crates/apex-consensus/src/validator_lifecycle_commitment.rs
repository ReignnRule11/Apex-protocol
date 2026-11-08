//! Verification wrapper for audited validator-lifecycle ledger commitments.
//!
//! This compares a freshly recomputed local commitment with a caller-supplied
//! expected digest. The expected value's provenance and anti-rollback
//! properties remain the caller's responsibility.

use crate::validator_lifecycle::{
    LifecycleError, LifecycleLedger, ProtocolTransitionVerifier,
};
use thiserror::Error;

/// Result of checking a lifecycle ledger against an expected commitment.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum LifecycleCommitmentVerificationError {
    /// The ledger could not be audited and committed.
    #[error("lifecycle ledger commitment could not be computed: {0}")]
    Ledger(#[from] LifecycleError),
    /// The recomputed commitment differs from the expected digest.
    #[error("lifecycle ledger commitment does not match the expected digest")]
    Mismatch,
}

/// Audits and recomputes the ledger commitment, then compares it with
/// `expected`. No ledger state is mutated.
///
/// This function does not authenticate `expected`, identify its publisher,
/// establish freshness, or prevent an attacker from replacing both history
/// and the expected digest. Callers requiring those properties need a
/// separately authenticated, monotonic external checkpoint.
pub fn verify_lifecycle_commitment(
    ledger: &LifecycleLedger,
    expected: &[u8; 32],
    verifier: &impl ProtocolTransitionVerifier,
) -> Result<(), LifecycleCommitmentVerificationError> {
    let actual = ledger.commitment(verifier)?;
    if &actual != expected {
        return Err(LifecycleCommitmentVerificationError::Mismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validator_lifecycle::{
        LifecycleEvent, LifecycleScope, ProtocolProofError, TransitionEnvelope,
    };

    struct AcceptVerifier;

    impl ProtocolTransitionVerifier for AcceptVerifier {
        fn verify_committed_transition(
            &self,
            _: LifecycleScope,
            _: [u8; 32],
            _: u64,
            _: &[u8],
        ) -> Result<(), ProtocolProofError> {
            Ok(())
        }
    }

    fn scope() -> LifecycleScope {
        LifecycleScope {
            chain_id: [1; 32],
            protocol_version: 1,
            operator_id: [2; 32],
            validator_id: [3; 32],
        }
    }

    fn ledger() -> LifecycleLedger {
        let mut ledger = LifecycleLedger::default();
        ledger
            .apply(
                TransitionEnvelope {
                    request_id: [1; 32],
                    scope: scope(),
                    expected_revision: 0,
                    event: LifecycleEvent::Submit {
                        evidence_revision: 1,
                        policy_revision: 1,
                    },
                },
                &AcceptVerifier,
            )
            .unwrap();
        ledger
    }

    #[test]
    fn accepts_the_exact_recomputed_commitment() {
        let ledger = ledger();
        let expected = ledger.commitment(&AcceptVerifier).unwrap();
        assert_eq!(
            verify_lifecycle_commitment(&ledger, &expected, &AcceptVerifier),
            Ok(())
        );
    }

    #[test]
    fn rejects_a_different_expected_commitment() {
        let ledger = ledger();
        let mut expected = ledger.commitment(&AcceptVerifier).unwrap();
        expected[0] ^= 0x80;
        assert_eq!(
            verify_lifecycle_commitment(&ledger, &expected, &AcceptVerifier),
            Err(LifecycleCommitmentVerificationError::Mismatch)
        );
    }

    #[test]
    fn empty_ledger_has_a_verifiable_commitment() {
        let ledger = LifecycleLedger::default();
        let expected = ledger.commitment(&AcceptVerifier).unwrap();
        assert_eq!(
            verify_lifecycle_commitment(&ledger, &expected, &AcceptVerifier),
            Ok(())
        );
    }
}
