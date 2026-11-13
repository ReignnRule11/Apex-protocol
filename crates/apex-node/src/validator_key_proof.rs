//! Ed25519 proof-of-possession checks for genesis candidate validators.
//!
//! This verifies that each candidate controls the private key corresponding to
//! its committed validator identity. It does not grant membership, authenticate
//! governance, prevent nonce replay across invocations, or activate consensus.
//! The caller must issue unpredictable one-time nonces and durably consume them.

use crate::genesis_commitment::{GenesisCommitmentError, GenesisProtocolConfig};
use apex_consensus::validator_set::ValidatorId;
use ed25519_dalek::{Signature, VerifyingKey};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use thiserror::Error;

const ID_DOMAIN: &[u8] = b"APEX::ED25519_VALIDATOR_ID::V1\\0";
const PROOF_DOMAIN: &[u8] = b"APEX::ED25519_VALIDATOR_KEY_PROOF::V1\\0";

/// A validator's signed response to a fresh genesis-scoped challenge.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatorKeyProof {
    pub validator_id: ValidatorId,
    pub public_key: [u8; 32],
    pub signature: [u8; 64],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum ValidatorKeyProofError {
    #[error("proof nonce must not be all zero")]
    ZeroNonce,
    #[error("proof count must exactly match the genesis candidate validator count")]
    ProofCountMismatch,
    #[error("proof references a validator outside the committed candidate set")]
    UnknownValidator,
    #[error("more than one proof was supplied for the same validator")]
    DuplicateValidator,
    #[error("public key is not a valid Ed25519 verifying key")]
    InvalidPublicKey,
    #[error("public key does not derive the committed validator identity")]
    IdentityMismatch,
    #[error("Ed25519 proof-of-possession signature is invalid")]
    InvalidSignature,
    #[error("genesis commitment could not be computed")]
    InvalidGenesis,
}

impl From<GenesisCommitmentError> for ValidatorKeyProofError {
    fn from(_: GenesisCommitmentError) -> Self {
        Self::InvalidGenesis
    }
}

/// Derive the protocol validator identity from a raw Ed25519 public key.
pub fn validator_id_for_public_key(public_key: &[u8; 32]) -> ValidatorId {
    let mut hasher = Sha256::new();
    hasher.update(ID_DOMAIN);
    hasher.update(public_key);
    ValidatorId(hasher.finalize().into())
}

/// Canonical message that a candidate must sign for the supplied genesis and
/// one-time challenge nonce. All fields are fixed-width and big-endian.
pub fn validator_key_proof_message(
    genesis: &GenesisProtocolConfig,
    nonce: &[u8; 32],
    validator_id: ValidatorId,
) -> Result<Vec<u8>, ValidatorKeyProofError> {
    if nonce.iter().all(|byte| *byte == 0) {
        return Err(ValidatorKeyProofError::ZeroNonce);
    }
    let commitment = genesis.commitment()?;
    let mut message = Vec::with_capacity(PROOF_DOMAIN.len() + 32 + 4 + 32 + 32 + 32);
    message.extend_from_slice(PROOF_DOMAIN);
    message.extend_from_slice(&genesis.chain_id.to_be_bytes());
    message.extend_from_slice(&genesis.protocol_version.to_be_bytes());
    message.extend_from_slice(&commitment.digest);
    message.extend_from_slice(nonce);
    message.extend_from_slice(&validator_id.0);
    Ok(message)
}

/// Verify one proof for every validator committed in the genesis candidate set.
///
/// The caller is responsible for generating a cryptographically unpredictable
/// nonce and recording it as consumed before accepting the result. This pure
/// verifier intentionally has no hidden clock, randomness, or replay database.
pub fn verify_validator_key_proofs(
    genesis: &GenesisProtocolConfig,
    nonce: &[u8; 32],
    proofs: &[ValidatorKeyProof],
) -> Result<(), ValidatorKeyProofError> {
    if nonce.iter().all(|byte| *byte == 0) {
        return Err(ValidatorKeyProofError::ZeroNonce);
    }
    let validators = genesis.validators.validators();
    if proofs.len() != validators.len() {
        return Err(ValidatorKeyProofError::ProofCountMismatch);
    }

    let mut seen = BTreeSet::new();
    for proof in proofs {
        if !seen.insert(proof.validator_id) {
            return Err(ValidatorKeyProofError::DuplicateValidator);
        }
        if validators
            .binary_search_by_key(&proof.validator_id, |validator| validator.id)
            .is_err()
        {
            return Err(ValidatorKeyProofError::UnknownValidator);
        }
        if validator_id_for_public_key(&proof.public_key) != proof.validator_id {
            return Err(ValidatorKeyProofError::IdentityMismatch);
        }
        let verifying_key = VerifyingKey::from_bytes(&proof.public_key)
            .map_err(|_| ValidatorKeyProofError::InvalidPublicKey)?;
        let signature = Signature::from_bytes(&proof.signature);
        let message = validator_key_proof_message(genesis, nonce, proof.validator_id)?;
        verifying_key
            .verify_strict(&message, &signature)
            .map_err(|_| ValidatorKeyProofError::InvalidSignature)?;
    }

    if validators.iter().any(|validator| !seen.contains(&validator.id)) {
        return Err(ValidatorKeyProofError::ProofCountMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genesis_commitment::GenesisProtocolConfig;
    use apex_consensus::validator_set::{Validator, ValidatorSet};
    use apex_types::ChainId256;
    use ed25519_dalek::{Signer, SigningKey};

    fn fixture() -> (GenesisProtocolConfig, [SigningKey; 2], [u8; 32]) {
        let keys = [
            SigningKey::from_bytes(&[11; 32]),
            SigningKey::from_bytes(&[22; 32]),
        ];
        let validators = ValidatorSet::new(
            keys.iter()
                .map(|key| Validator {
                    id: validator_id_for_public_key(&key.verifying_key().to_bytes()),
                    voting_power: 1,
                })
                .collect(),
        )
        .unwrap();
        (
            GenesisProtocolConfig {
                chain_id: ChainId256::from_be_bytes([9; 32]),
                protocol_version: 1,
                validators,
            },
            keys,
            [0xA5; 32],
        )
    }

    fn proof(
        genesis: &GenesisProtocolConfig,
        nonce: &[u8; 32],
        key: &SigningKey,
    ) -> ValidatorKeyProof {
        let public_key = key.verifying_key().to_bytes();
        let validator_id = validator_id_for_public_key(&public_key);
        let message = validator_key_proof_message(genesis, nonce, validator_id).unwrap();
        ValidatorKeyProof {
            validator_id,
            public_key,
            signature: key.sign(&message).to_bytes(),
        }
    }

    #[test]
    fn verifies_complete_genesis_scoped_key_possession_set() {
        let (genesis, keys, nonce) = fixture();
        let proofs = keys
            .iter()
            .map(|key| proof(&genesis, &nonce, key))
            .collect::<Vec<_>>();
        assert_eq!(verify_validator_key_proofs(&genesis, &nonce, &proofs), Ok(()));
    }

    #[test]
    fn rejects_zero_nonce_and_incomplete_proof_set() {
        let (genesis, keys, nonce) = fixture();
        let proofs = vec![proof(&genesis, &nonce, &keys[0])];
        assert_eq!(
            verify_validator_key_proofs(&genesis, &[0; 32], &proofs),
            Err(ValidatorKeyProofError::ZeroNonce)
        );
        assert_eq!(
            verify_validator_key_proofs(&genesis, &nonce, &proofs),
            Err(ValidatorKeyProofError::ProofCountMismatch)
        );
    }

    #[test]
    fn rejects_duplicate_validator_proofs() {
        let (genesis, keys, nonce) = fixture();
        let first = proof(&genesis, &nonce, &keys[0]);
        assert_eq!(
            verify_validator_key_proofs(&genesis, &nonce, &[first.clone(), first]),
            Err(ValidatorKeyProofError::DuplicateValidator)
        );
    }

    #[test]
    fn rejects_public_key_identity_substitution() {
        let (genesis, keys, nonce) = fixture();
        let mut first = proof(&genesis, &nonce, &keys[0]);
        first.public_key = keys[1].verifying_key().to_bytes();
        let second = proof(&genesis, &nonce, &keys[1]);
        assert_eq!(
            verify_validator_key_proofs(&genesis, &nonce, &[first, second]),
            Err(ValidatorKeyProofError::IdentityMismatch)
        );
    }

    #[test]
    fn rejects_signature_replay_across_genesis_or_nonce() {
        let (genesis, keys, nonce) = fixture();
        let proofs = keys
            .iter()
            .map(|key| proof(&genesis, &nonce, key))
            .collect::<Vec<_>>();
        assert_eq!(
            verify_validator_key_proofs(&genesis, &[0xA6; 32], &proofs),
            Err(ValidatorKeyProofError::InvalidSignature)
        );

        let mut changed_genesis = genesis.clone();
        changed_genesis.protocol_version += 1;
        assert_eq!(
            verify_validator_key_proofs(&changed_genesis, &nonce, &proofs),
            Err(ValidatorKeyProofError::InvalidSignature)
        );
    }

    #[test]
    fn rejects_signature_tampering() {
        let (genesis, keys, nonce) = fixture();
        let mut proofs = keys
            .iter()
            .map(|key| proof(&genesis, &nonce, key))
            .collect::<Vec<_>>();
        proofs[0].signature[0] ^= 0x80;
        assert_eq!(
            verify_validator_key_proofs(&genesis, &nonce, &proofs),
            Err(ValidatorKeyProofError::InvalidSignature)
        );
    }
}
