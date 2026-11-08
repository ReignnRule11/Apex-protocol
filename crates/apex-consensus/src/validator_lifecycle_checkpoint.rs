//! Authenticated, monotonic lifecycle checkpoint acceptance boundary.
//!
//! This module defines the protocol contract only. Production deployments must
//! supply a real signature authenticator and a durable, rollback-resistant
//! compare-and-advance store. The test store below is deliberately in-memory.

use crate::validator_lifecycle::{LifecycleError, LifecycleLedger, ProtocolTransitionVerifier};
use sha2_11::{Digest, Sha256};
use thiserror::Error;

/// Domain-separated checkpoint payload. The signature covers
/// `signing_bytes()`; the checkpoint hash additionally commits to the signature.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticatedLifecycleCheckpoint {
    pub network_id: [u8; 32],
    pub sequence: u64,
    pub commitment: [u8; 32],
    pub previous_checkpoint_hash: [u8; 32],
    pub signature: Vec<u8>,
}

impl AuthenticatedLifecycleCheckpoint {
    /// Canonical fixed-width message that an external signer must authenticate.
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(32 + 32 + 8 + 32 + 32);
        bytes.extend_from_slice(b"APEX/lifecycle-checkpoint/v1");
        bytes.extend_from_slice(&self.network_id);
        bytes.extend_from_slice(&self.sequence.to_be_bytes());
        bytes.extend_from_slice(&self.commitment);
        bytes.extend_from_slice(&self.previous_checkpoint_hash);
        bytes
    }

    /// Stable identifier for chaining the complete signed checkpoint.
    pub fn checkpoint_hash(&self) -> [u8; 32] {
        let mut hash = Sha256::new();
        hash.update(b"APEX/lifecycle-checkpoint-hash/v1");
        hash.update(self.signing_bytes());
        hash.update((self.signature.len() as u64).to_be_bytes());
        hash.update(&self.signature);
        hash.finalize().into()
    }
}

/// External signature verification. Implementations must bind the configured
/// signer/key policy to the network and checkpoint domain.
pub trait LifecycleCheckpointAuthenticator {
    fn authenticate(
        &self,
        signing_bytes: &[u8],
        signature: &[u8],
    ) -> Result<(), CheckpointAuthenticationError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum CheckpointAuthenticationError {
    #[error("checkpoint signature is invalid")]
    Invalid,
    #[error("checkpoint authentication is unavailable")]
    Unavailable,
}

/// Trusted head recorded by the checkpoint store.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LifecycleCheckpointAnchor {
    pub sequence: u64,
    pub checkpoint_hash: [u8; 32],
}

/// Atomic persistence boundary. A production implementation MUST durably
/// compare the complete expected anchor and advance to `next` in one
/// transaction, reject stale/concurrent writers, and protect the head against
/// rollback across process restart, backup restore, and storage replacement.
/// A normal database CAS alone is not sufficient if an attacker can roll back
/// the database and its backups.
pub trait MonotonicLifecycleCheckpointStore {
    fn load(
        &self,
        network_id: [u8; 32],
    ) -> Result<Option<LifecycleCheckpointAnchor>, CheckpointStoreError>;

    fn compare_and_advance(
        &self,
        network_id: [u8; 32],
        expected: Option<LifecycleCheckpointAnchor>,
        next: LifecycleCheckpointAnchor,
    ) -> Result<bool, CheckpointStoreError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum CheckpointStoreError {
    #[error("checkpoint store is unavailable")]
    Unavailable,
    #[error("checkpoint store rejected an invalid sequence advance")]
    InvalidAdvance,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum LifecycleCheckpointAcceptanceError {
    #[error("network ID must not be zero")]
    InvalidNetwork,
    #[error("checkpoint sequence must be non-zero")]
    InvalidSequence,
    #[error("checkpoint signature must not be empty")]
    EmptySignature,
    #[error("lifecycle ledger could not be audited and committed: {0}")]
    Ledger(#[from] LifecycleError),
    #[error("checkpoint commitment does not match the audited ledger")]
    CommitmentMismatch,
    #[error("checkpoint authentication failed: {0}")]
    Authentication(#[from] CheckpointAuthenticationError),
    #[error("checkpoint store failed: {0}")]
    Store(#[from] CheckpointStoreError),
    #[error("checkpoint sequence is not the exact next sequence")]
    NonMonotonicSequence,
    #[error("checkpoint does not extend the currently trusted checkpoint")]
    PreviousCheckpointMismatch,
    #[error("another writer advanced the checkpoint concurrently")]
    ConcurrentAdvance,
}

/// Authenticate, audit and atomically advance the trusted checkpoint head.
///
/// Genesis is sequence 1 with a zero previous hash. Every later checkpoint
/// must extend the exact currently trusted head and increment by one. The
/// store's atomic compare-and-advance is the final race/rollback boundary.
pub fn accept_lifecycle_checkpoint(
    ledger: &LifecycleLedger,
    checkpoint: &AuthenticatedLifecycleCheckpoint,
    authenticator: &impl LifecycleCheckpointAuthenticator,
    store: &impl MonotonicLifecycleCheckpointStore,
    verifier: &impl ProtocolTransitionVerifier,
) -> Result<LifecycleCheckpointAnchor, LifecycleCheckpointAcceptanceError> {
    if checkpoint.network_id == [0; 32] {
        return Err(LifecycleCheckpointAcceptanceError::InvalidNetwork);
    }
    if checkpoint.sequence == 0 {
        return Err(LifecycleCheckpointAcceptanceError::InvalidSequence);
    }
    if checkpoint.signature.is_empty() {
        return Err(LifecycleCheckpointAcceptanceError::EmptySignature);
    }

    authenticator.authenticate(&checkpoint.signing_bytes(), &checkpoint.signature)?;
    let actual = ledger.commitment(verifier)?;
    if actual != checkpoint.commitment {
        return Err(LifecycleCheckpointAcceptanceError::CommitmentMismatch);
    }

    let current = store.load(checkpoint.network_id)?;
    match current {
        None => {
            if checkpoint.sequence != 1 {
                return Err(LifecycleCheckpointAcceptanceError::NonMonotonicSequence);
            }
            if checkpoint.previous_checkpoint_hash != [0; 32] {
                return Err(LifecycleCheckpointAcceptanceError::PreviousCheckpointMismatch);
            }
        }
        Some(anchor) => {
            let next_sequence = anchor
                .sequence
                .checked_add(1)
                .ok_or(LifecycleCheckpointAcceptanceError::NonMonotonicSequence)?;
            if checkpoint.sequence != next_sequence {
                return Err(LifecycleCheckpointAcceptanceError::NonMonotonicSequence);
            }
            if checkpoint.previous_checkpoint_hash != anchor.checkpoint_hash {
                return Err(LifecycleCheckpointAcceptanceError::PreviousCheckpointMismatch);
            }
        }
    }

    let next = LifecycleCheckpointAnchor {
        sequence: checkpoint.sequence,
        checkpoint_hash: checkpoint.checkpoint_hash(),
    };
    if !store.compare_and_advance(checkpoint.network_id, current, next)? {
        return Err(LifecycleCheckpointAcceptanceError::ConcurrentAdvance);
    }
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validator_lifecycle::ProtocolProofError;
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    struct AcceptVerifier;
    impl ProtocolTransitionVerifier for AcceptVerifier {
        fn verify_committed_transition(
            &self, _: crate::validator_lifecycle::LifecycleScope,
            _: [u8; 32], _: u64, _: &[u8],
        ) -> Result<(), ProtocolProofError> { Ok(()) }
    }

    struct AcceptAuthenticator;
    impl LifecycleCheckpointAuthenticator for AcceptAuthenticator {
        fn authenticate(&self, _: &[u8], signature: &[u8])
            -> Result<(), CheckpointAuthenticationError> {
            if signature == b"valid-signature" { Ok(()) }
            else { Err(CheckpointAuthenticationError::Invalid) }
        }
    }

    #[derive(Default)]
    struct MemoryStore(Mutex<BTreeMap<[u8; 32], LifecycleCheckpointAnchor>>);

    impl MonotonicLifecycleCheckpointStore for MemoryStore {
        fn load(&self, network_id: [u8; 32])
            -> Result<Option<LifecycleCheckpointAnchor>, CheckpointStoreError> {
            Ok(self.0.lock().unwrap().get(&network_id).copied())
        }

        fn compare_and_advance(
            &self, network_id: [u8; 32],
            expected: Option<LifecycleCheckpointAnchor>,
            next: LifecycleCheckpointAnchor,
        ) -> Result<bool, CheckpointStoreError> {
            let mut heads = self.0.lock().unwrap();
            if heads.get(&network_id).copied() != expected { return Ok(false); }
            let valid = match expected {
                None => next.sequence == 1,
                Some(anchor) => anchor.sequence.checked_add(1) == Some(next.sequence),
            };
            if !valid { return Err(CheckpointStoreError::InvalidAdvance); }
            heads.insert(network_id, next);
            Ok(true)
        }
    }

    fn checkpoint(sequence: u64, previous: [u8; 32], commitment: [u8; 32])
        -> AuthenticatedLifecycleCheckpoint {
        AuthenticatedLifecycleCheckpoint {
            network_id: [7; 32], sequence, commitment,
            previous_checkpoint_hash: previous,
            signature: b"valid-signature".to_vec(),
        }
    }

    #[test]
    fn accepts_and_persists_authenticated_genesis_checkpoint() {
        let ledger = LifecycleLedger::default();
        let commitment = ledger.commitment(&AcceptVerifier).unwrap();
        let checkpoint = checkpoint(1, [0; 32], commitment);
        let store = MemoryStore::default();
        let accepted = accept_lifecycle_checkpoint(
            &ledger, &checkpoint, &AcceptAuthenticator, &store, &AcceptVerifier,
        ).unwrap();
        assert_eq!(accepted.sequence, 1);
        assert_eq!(store.load([7; 32]).unwrap(), Some(accepted));
    }

    #[test]
    fn rejects_a_checkpoint_with_a_different_commitment() {
        let ledger = LifecycleLedger::default();
        let checkpoint = checkpoint(1, [0; 32], [9; 32]);
        assert_eq!(
            accept_lifecycle_checkpoint(
                &ledger, &checkpoint, &AcceptAuthenticator,
                &MemoryStore::default(), &AcceptVerifier,
            ),
            Err(LifecycleCheckpointAcceptanceError::CommitmentMismatch)
        );
    }

    #[test]
    fn rejects_invalid_authentication_before_advancing_store() {
        struct Reject;
        impl LifecycleCheckpointAuthenticator for Reject {
            fn authenticate(&self, _: &[u8], _: &[u8])
                -> Result<(), CheckpointAuthenticationError> {
                Err(CheckpointAuthenticationError::Invalid)
            }
        }
        let ledger = LifecycleLedger::default();
        let commitment = ledger.commitment(&AcceptVerifier).unwrap();
        let store = MemoryStore::default();
        assert_eq!(
            accept_lifecycle_checkpoint(
                &ledger, &checkpoint(1, [0; 32], commitment),
                &Reject, &store, &AcceptVerifier,
            ),
            Err(LifecycleCheckpointAcceptanceError::Authentication(
                CheckpointAuthenticationError::Invalid
            ))
        );
        assert_eq!(store.load([7; 32]).unwrap(), None);
    }

    #[test]
    fn rejects_skipped_genesis_sequence() {
        let ledger = LifecycleLedger::default();
        let commitment = ledger.commitment(&AcceptVerifier).unwrap();
        assert_eq!(
            accept_lifecycle_checkpoint(
                &ledger, &checkpoint(2, [0; 32], commitment),
                &AcceptAuthenticator, &MemoryStore::default(), &AcceptVerifier,
            ),
            Err(LifecycleCheckpointAcceptanceError::NonMonotonicSequence)
        );
    }

    #[test]
    fn rejects_previous_checkpoint_mismatch() {
        let ledger = LifecycleLedger::default();
        let commitment = ledger.commitment(&AcceptVerifier).unwrap();
        let store = MemoryStore::default();
        let first = checkpoint(1, [0; 32], commitment);
        let anchor = accept_lifecycle_checkpoint(
            &ledger, &first, &AcceptAuthenticator, &store, &AcceptVerifier,
        ).unwrap();
        let second = checkpoint(2, [8; 32], commitment);
        assert_eq!(
            accept_lifecycle_checkpoint(
                &ledger, &second, &AcceptAuthenticator, &store, &AcceptVerifier,
            ),
            Err(LifecycleCheckpointAcceptanceError::PreviousCheckpointMismatch)
        );
        assert_eq!(store.load([7; 32]).unwrap(), Some(anchor));
    }

    #[test]
    fn rejects_stale_sequence_without_mutating_head() {
        let ledger = LifecycleLedger::default();
        let commitment = ledger.commitment(&AcceptVerifier).unwrap();
        let store = MemoryStore::default();
        let first = checkpoint(1, [0; 32], commitment);
        let anchor = accept_lifecycle_checkpoint(
            &ledger, &first, &AcceptAuthenticator, &store, &AcceptVerifier,
        ).unwrap();
        let stale = checkpoint(1, [0; 32], commitment);
        assert_eq!(
            accept_lifecycle_checkpoint(
                &ledger, &stale, &AcceptAuthenticator, &store, &AcceptVerifier,
            ),
            Err(LifecycleCheckpointAcceptanceError::NonMonotonicSequence)
        );
        assert_eq!(store.load([7; 32]).unwrap(), Some(anchor));
    }
}
