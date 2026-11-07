//! Authenticated Ed25519 validator identities and strict commit certificates.
//!
//! Experimental protocol profile. It verifies signatures and quorum against an
//! explicitly scheduled authenticated validator set; it is not yet wired into
//! Malachite or the application commit path.
use crate::consensus_message::{ConsensusMessageStep, ConsensusSigningMessage};
use crate::validator_set::{ValidatorId, ValidatorSet};
use ed25519_dalek::{Signature, VerifyingKey};
use sha2_11::{Digest, Sha256};
use std::collections::BTreeSet;
use thiserror::Error;

const VALIDATOR_ID_DOMAIN: &[u8] = b"APEX-VALIDATOR-ID-ED25519-V1";
const VALIDATOR_SET_DOMAIN: &[u8] = b"APEX-VALIDATOR-SET-ED25519-V1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidatorPublicKey(pub [u8; 32]);

pub fn validator_id_from_public_key(
    public_key: ValidatorPublicKey,
) -> Result<ValidatorId, CertificateError> {
    VerifyingKey::from_bytes(&public_key.0).map_err(|_| CertificateError::InvalidPublicKey)?;
    let mut hasher = Sha256::new();
    hasher.update(VALIDATOR_ID_DOMAIN);
    hasher.update([1_u8]);
    hasher.update(public_key.0);
    let digest = hasher.finalize();
    let mut id = [0_u8; 32];
    id.copy_from_slice(&digest);
    if id == [0; 32] {
        return Err(CertificateError::InvalidPublicKey);
    }
    Ok(ValidatorId(id))
}

#[derive(Clone, Debug)]
struct AuthenticatedValidator {
    id: ValidatorId,
    voting_power: u64,
    public_key: ValidatorPublicKey,
    verifying_key: VerifyingKey,
}

#[derive(Clone, Debug)]
pub struct AuthenticatedValidatorSet {
    validators: Vec<AuthenticatedValidator>,
    total_voting_power: u64,
    hash: [u8; 32],
}

impl AuthenticatedValidatorSet {
    pub fn new(
        set: ValidatorSet,
        public_keys: Vec<ValidatorPublicKey>,
    ) -> Result<Self, CertificateError> {
        if public_keys.len() != set.validators().len() {
            return Err(CertificateError::ValidatorKeySetMismatch);
        }
        let mut authenticated = Vec::with_capacity(public_keys.len());
        let mut seen = BTreeSet::new();
        for public_key in public_keys {
            let id = validator_id_from_public_key(public_key)?;
            if !seen.insert(id) {
                return Err(CertificateError::DuplicateValidatorKey);
            }
            let validator = set
                .validators()
                .iter()
                .find(|v| v.id == id)
                .ok_or(CertificateError::ValidatorKeySetMismatch)?;
            let verifying_key = VerifyingKey::from_bytes(&public_key.0)
                .map_err(|_| CertificateError::InvalidPublicKey)?;
            if verifying_key.is_weak() {
                return Err(CertificateError::InvalidPublicKey);
            }
            authenticated.push(AuthenticatedValidator {
                id,
                voting_power: validator.voting_power,
                public_key,
                verifying_key,
            });
        }
        authenticated.sort_by_key(|validator| validator.id);
        let validator_count =
            u32::try_from(authenticated.len()).map_err(|_| CertificateError::TooManyValidators)?;
        let mut hasher = Sha256::new();
        hasher.update(VALIDATOR_SET_DOMAIN);
        hasher.update([1_u8]);
        hasher.update(validator_count.to_be_bytes());
        for validator in &authenticated {
            hasher.update(validator.id.0);
            hasher.update(validator.voting_power.to_be_bytes());
            hasher.update(validator.public_key.0);
        }
        let digest = hasher.finalize();
        let mut hash = [0_u8; 32];
        hash.copy_from_slice(&digest);
        Ok(Self {
            validators: authenticated,
            total_voting_power: set.total_voting_power(),
            hash,
        })
    }

    pub const fn hash(&self) -> [u8; 32] {
        self.hash
    }

    pub const fn total_voting_power(&self) -> u64 {
        self.total_voting_power
    }

    pub fn validator_count(&self) -> usize {
        self.validators.len()
    }
}

#[derive(Clone, Debug)]
pub struct AuthenticatedValidatorSetUpdate {
    pub activation_height: u64,
    pub validator_set: AuthenticatedValidatorSet,
}

#[derive(Clone, Debug)]
pub struct AuthenticatedValidatorSetSchedule {
    initial: AuthenticatedValidatorSet,
    updates: Vec<AuthenticatedValidatorSetUpdate>,
}

impl AuthenticatedValidatorSetSchedule {
    pub fn new(
        initial: AuthenticatedValidatorSet,
        updates: Vec<AuthenticatedValidatorSetUpdate>,
    ) -> Result<Self, CertificateError> {
        let mut previous = 0_u64;
        for update in &updates {
            if update.activation_height == 0 || update.activation_height <= previous {
                return Err(CertificateError::InvalidValidatorSetSchedule);
            }
            previous = update.activation_height;
        }
        Ok(Self { initial, updates })
    }

    pub fn validator_set_at(&self, height: u64) -> &AuthenticatedValidatorSet {
        self.updates
            .iter()
            .rev()
            .find(|update| update.activation_height <= height)
            .map(|update| &update.validator_set)
            .unwrap_or(&self.initial)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommitSignature {
    pub public_key: ValidatorPublicKey,
    pub signature: [u8; 64],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitCertificate {
    pub chain_id: [u8; 32],
    pub protocol_version: u32,
    pub height: u64,
    pub round: u32,
    pub block_hash: [u8; 32],
    pub validator_set_hash: [u8; 32],
    pub signatures: Vec<CommitSignature>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommitContext {
    pub chain_id: [u8; 32],
    pub protocol_version: u32,
    pub height: u64,
    pub round: u32,
    pub block_hash: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum CertificateError {
    #[error("Ed25519 public key encoding is invalid")]
    InvalidPublicKey,
    #[error("public keys do not exactly match the validator set")]
    ValidatorKeySetMismatch,
    #[error("validator public key is duplicated")]
    DuplicateValidatorKey,
    #[error("authenticated validator-set schedule is invalid")]
    InvalidValidatorSetSchedule,
    #[error("validator count exceeds the canonical u32 encoding")]
    TooManyValidators,
    #[error("certificate has more signatures than scheduled validators")]
    TooManySignatures,
    #[error("certificate chain ID or protocol version is invalid")]
    WrongChainContext,
    #[error("certificate height does not match the expected height")]
    WrongHeight,
    #[error("certificate round does not match the expected round")]
    WrongRound,
    #[error("certificate block hash does not match the expected block")]
    WrongBlock,
    #[error("certificate block hash must not be zero")]
    NilBlock,
    #[error("certificate validator-set hash does not match the scheduled set")]
    WrongValidatorSet,
    #[error("certificate contains no signatures")]
    EmptyCertificate,
    #[error("certificate signer is not in the scheduled validator set")]
    UnknownSigner,
    #[error("certificate contains a duplicate signer")]
    DuplicateSigner,
    #[error("Ed25519 signature verification failed")]
    InvalidSignature,
    #[error("certificate voting power does not exceed two-thirds")]
    InsufficientQuorum,
    #[error("certificate voting-power arithmetic overflowed")]
    VotingPowerOverflow,
}

pub fn verify_commit_certificate(
    certificate: &CommitCertificate,
    expected: CommitContext,
    schedule: &AuthenticatedValidatorSetSchedule,
) -> Result<(), CertificateError> {
    if expected.chain_id == [0; 32]
        || expected.protocol_version == 0
        || certificate.chain_id != expected.chain_id
        || certificate.protocol_version != expected.protocol_version
    {
        return Err(CertificateError::WrongChainContext);
    }
    if certificate.height != expected.height {
        return Err(CertificateError::WrongHeight);
    }
    if certificate.round != expected.round {
        return Err(CertificateError::WrongRound);
    }
    if certificate.block_hash == [0; 32] {
        return Err(CertificateError::NilBlock);
    }
    if certificate.block_hash != expected.block_hash {
        return Err(CertificateError::WrongBlock);
    }
    if certificate.signatures.is_empty() {
        return Err(CertificateError::EmptyCertificate);
    }
    let validators = schedule.validator_set_at(expected.height);
    if certificate.validator_set_hash != validators.hash() {
        return Err(CertificateError::WrongValidatorSet);
    }
    if certificate.signatures.len() > validators.validators.len() {
        return Err(CertificateError::TooManySignatures);
    }

    let mut seen = BTreeSet::new();
    let mut signed_power = 0_u64;
    for entry in &certificate.signatures {
        let signer_id = validator_id_from_public_key(entry.public_key)?;
        if !seen.insert(signer_id) {
            return Err(CertificateError::DuplicateSigner);
        }
        let validator = validators
            .validators
            .iter()
            .find(|validator| validator.id == signer_id)
            .ok_or(CertificateError::UnknownSigner)?;
        if validator.public_key != entry.public_key {
            return Err(CertificateError::UnknownSigner);
        }
        let message = ConsensusSigningMessage::new(
            certificate.chain_id,
            certificate.protocol_version,
            certificate.height,
            certificate.round,
            ConsensusMessageStep::Precommit,
            signer_id.0,
            certificate.validator_set_hash,
            certificate.block_hash,
        )
        .map_err(|_| CertificateError::WrongChainContext)?;
        let signature = Signature::from_bytes(&entry.signature);
        validator
            .verifying_key
            .verify_strict(&message.signing_bytes(), &signature)
            .map_err(|_| CertificateError::InvalidSignature)?;
        signed_power = signed_power
            .checked_add(validator.voting_power)
            .ok_or(CertificateError::VotingPowerOverflow)?;
    }
    if u128::from(signed_power) * 3 <= u128::from(validators.total_voting_power) * 2 {
        return Err(CertificateError::InsufficientQuorum);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validator_set::Validator;
    use ed25519_dalek::{Signer, SigningKey};

    fn keys() -> Vec<SigningKey> {
        (1_u8..=3)
            .map(|n| SigningKey::from_bytes(&[n; 32]))
            .collect()
    }

    fn authenticated(keys: &[SigningKey]) -> AuthenticatedValidatorSet {
        let entries = keys
            .iter()
            .map(|key| ValidatorPublicKey(key.verifying_key().to_bytes()))
            .collect::<Vec<_>>();
        let validators = entries
            .iter()
            .map(|key| Validator {
                id: validator_id_from_public_key(*key).unwrap(),
                voting_power: 1,
            })
            .collect();
        AuthenticatedValidatorSet::new(ValidatorSet::new(validators).unwrap(), entries).unwrap()
    }

    fn certificate(
        keys: &[SigningKey],
        set: &AuthenticatedValidatorSet,
        count: usize,
    ) -> CommitCertificate {
        let mut signatures = Vec::new();
        for key in keys.iter().take(count) {
            let public_key = ValidatorPublicKey(key.verifying_key().to_bytes());
            let id = validator_id_from_public_key(public_key).unwrap();
            let message = ConsensusSigningMessage::new(
                [9; 32],
                1,
                12,
                3,
                ConsensusMessageStep::Precommit,
                id.0,
                set.hash(),
                [8; 32],
            )
            .unwrap();
            signatures.push(CommitSignature {
                public_key,
                signature: key.sign(&message.signing_bytes()).to_bytes(),
            });
        }
        CommitCertificate {
            chain_id: [9; 32],
            protocol_version: 1,
            height: 12,
            round: 3,
            block_hash: [8; 32],
            validator_set_hash: set.hash(),
            signatures,
        }
    }

    #[test]
    fn verifies_real_signatures_and_strict_supermajority() {
        let keys = keys();
        let set = authenticated(&keys);
        let schedule = AuthenticatedValidatorSetSchedule::new(set.clone(), vec![]).unwrap();
        let mut cert = certificate(&keys, &set, 3);
        let context = CommitContext {
            chain_id: [9; 32],
            protocol_version: 1,
            height: 12,
            round: 3,
            block_hash: [8; 32],
        };
        assert_eq!(verify_commit_certificate(&cert, context, &schedule), Ok(()));
        cert.signatures.pop();
        assert_eq!(
            verify_commit_certificate(&cert, context, &schedule),
            Err(CertificateError::InsufficientQuorum)
        );
    }

    #[test]
    fn rejects_wrong_chain_height_block_and_set_hash() {
        let keys = keys();
        let set = authenticated(&keys);
        let schedule = AuthenticatedValidatorSetSchedule::new(set.clone(), vec![]).unwrap();
        let cert = certificate(&keys, &set, 3);
        let expected = CommitContext {
            chain_id: [9; 32],
            protocol_version: 1,
            height: 12,
            round: 3,
            block_hash: [8; 32],
        };
        assert_eq!(
            verify_commit_certificate(
                &cert,
                CommitContext {
                    chain_id: [7; 32],
                    ..expected
                },
                &schedule
            ),
            Err(CertificateError::WrongChainContext)
        );
        let mut bad = cert.clone();
        bad.height += 1;
        assert_eq!(
            verify_commit_certificate(&bad, expected, &schedule),
            Err(CertificateError::WrongHeight)
        );
        let mut bad = cert.clone();
        bad.round += 1;
        assert_eq!(
            verify_commit_certificate(&bad, expected, &schedule),
            Err(CertificateError::WrongRound)
        );
        let mut bad = cert.clone();
        bad.block_hash = [6; 32];
        assert_eq!(
            verify_commit_certificate(&bad, expected, &schedule),
            Err(CertificateError::WrongBlock)
        );
        let mut bad = cert.clone();
        bad.block_hash = [0; 32];
        assert_eq!(
            verify_commit_certificate(&bad, expected, &schedule),
            Err(CertificateError::NilBlock)
        );
        let mut bad = cert;
        bad.validator_set_hash = [4; 32];
        assert_eq!(
            verify_commit_certificate(&bad, expected, &schedule),
            Err(CertificateError::WrongValidatorSet)
        );
    }

    #[test]
    fn rejects_tampered_signature_duplicate_and_unknown_signer() {
        let keys = keys();
        let set = authenticated(&keys);
        let schedule = AuthenticatedValidatorSetSchedule::new(set.clone(), vec![]).unwrap();
        let expected = CommitContext {
            chain_id: [9; 32],
            protocol_version: 1,
            height: 12,
            round: 3,
            block_hash: [8; 32],
        };
        let mut bad = certificate(&keys, &set, 3);
        bad.signatures[0].signature[0] ^= 1;
        assert_eq!(
            verify_commit_certificate(&bad, expected, &schedule),
            Err(CertificateError::InvalidSignature)
        );
        let mut bad = certificate(&keys, &set, 3);
        bad.signatures[1] = bad.signatures[0];
        assert_eq!(
            verify_commit_certificate(&bad, expected, &schedule),
            Err(CertificateError::DuplicateSigner)
        );
        let outsider = SigningKey::from_bytes(&[99; 32]);
        let mut bad = certificate(&keys, &set, 3);
        bad.signatures[0].public_key = ValidatorPublicKey(outsider.verifying_key().to_bytes());
        assert_eq!(
            verify_commit_certificate(&bad, expected, &schedule),
            Err(CertificateError::UnknownSigner)
        );
    }

    #[test]
    fn validator_set_hash_binds_keys_power_and_schedule_activation() {
        let keys = keys();
        let first = authenticated(&keys);
        let replacement_keys = vec![SigningKey::from_bytes(&[4; 32])];
        let second = authenticated(&replacement_keys);
        let schedule = AuthenticatedValidatorSetSchedule::new(
            first.clone(),
            vec![AuthenticatedValidatorSetUpdate {
                activation_height: 20,
                validator_set: second.clone(),
            }],
        )
        .unwrap();
        assert_eq!(schedule.validator_set_at(19).hash(), first.hash());
        assert_eq!(schedule.validator_set_at(20).hash(), second.hash());
        assert_ne!(first.hash(), second.hash());
    }
}
