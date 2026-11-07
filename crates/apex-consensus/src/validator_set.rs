//! Deterministic validator-set and lifecycle primitives.
//!
//! These types validate local data structures only. They do not authenticate
//! validator keys, verify signatures, establish certificate validity, or run BFT.

use std::collections::BTreeSet;
use thiserror::Error;

/// Opaque, fixed-width validator identity. All-zero is reserved as invalid.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ValidatorId(pub [u8; 32]);

impl ValidatorId {
    pub fn is_valid(&self) -> bool {
        self.0.iter().any(|byte| *byte != 0)
    }
}

/// A validator's voting weight. Zero power is never admitted to a set.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Validator {
    pub id: ValidatorId,
    pub voting_power: u64,
}

/// Canonically ordered, validated validator set (ascending opaque identity).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatorSet {
    validators: Vec<Validator>,
    total_voting_power: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum ValidatorSetError {
    #[error("validator set must not be empty")]
    EmptySet,
    #[error("validator identity must not be all zero")]
    InvalidIdentity,
    #[error("validator voting power must be greater than zero")]
    ZeroVotingPower,
    #[error("validator identity is duplicated")]
    DuplicateIdentity,
    #[error("total voting power overflows u64")]
    VotingPowerOverflow,
    #[error("signer is not a member of this validator set")]
    UnknownSigner,
    #[error("signer appears more than once in a certificate")]
    DuplicateSigner,
}

impl ValidatorSet {
    pub fn new(mut validators: Vec<Validator>) -> Result<Self, ValidatorSetError> {
        if validators.is_empty() {
            return Err(ValidatorSetError::EmptySet);
        }
        validators.sort_by_key(|validator| validator.id);
        let mut total = 0_u64;
        let mut previous = None;
        for validator in &validators {
            if !validator.id.is_valid() {
                return Err(ValidatorSetError::InvalidIdentity);
            }
            if validator.voting_power == 0 {
                return Err(ValidatorSetError::ZeroVotingPower);
            }
            if previous == Some(validator.id) {
                return Err(ValidatorSetError::DuplicateIdentity);
            }
            previous = Some(validator.id);
            total = total
                .checked_add(validator.voting_power)
                .ok_or(ValidatorSetError::VotingPowerOverflow)?;
        }
        Ok(Self {
            validators,
            total_voting_power: total,
        })
    }

    pub fn validators(&self) -> &[Validator] {
        &self.validators
    }

    pub fn total_voting_power(&self) -> u64 {
        self.total_voting_power
    }

    /// Counts each listed signer once. Unknown and duplicate identities fail closed.
    /// This proves only arithmetic quorum over supplied identities, not signatures.
    pub fn has_supermajority(&self, signers: &[ValidatorId]) -> Result<bool, ValidatorSetError> {
        let mut seen = BTreeSet::new();
        let mut signed = 0_u64;
        for signer in signers {
            if !seen.insert(*signer) {
                return Err(ValidatorSetError::DuplicateSigner);
            }
            let validator = self
                .validators
                .binary_search_by_key(signer, |validator| validator.id)
                .ok()
                .map(|index| self.validators[index])
                .ok_or(ValidatorSetError::UnknownSigner)?;
            signed = signed
                .checked_add(validator.voting_power)
                .ok_or(ValidatorSetError::VotingPowerOverflow)?;
        }
        Ok(u128::from(signed) * 3 > u128::from(self.total_voting_power) * 2)
    }
}

/// A complete replacement set scheduled for an exact activation height.
/// Activation heights are local schedule data, not authenticated governance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatorSetUpdate {
    pub activation_height: u64,
    pub validator_set: ValidatorSet,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatorSetSchedule {
    initial: ValidatorSet,
    updates: Vec<ValidatorSetUpdate>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum ScheduleError {
    #[error("validator-set activation height must be greater than zero")]
    ZeroActivationHeight,
    #[error("validator-set activation heights must be strictly increasing")]
    NonIncreasingActivationHeight,
}

impl ValidatorSetSchedule {
    pub fn new(
        initial: ValidatorSet,
        updates: Vec<ValidatorSetUpdate>,
    ) -> Result<Self, ScheduleError> {
        let mut previous = 0_u64;
        for update in &updates {
            if update.activation_height == 0 {
                return Err(ScheduleError::ZeroActivationHeight);
            }
            if update.activation_height <= previous {
                return Err(ScheduleError::NonIncreasingActivationHeight);
            }
            previous = update.activation_height;
        }
        Ok(Self { initial, updates })
    }

    /// The update activates at its declared height, not one block earlier/later.
    pub fn validator_set_at(&self, height: u64) -> &ValidatorSet {
        self.updates
            .iter()
            .rev()
            .find(|update| update.activation_height <= height)
            .map(|update| &update.validator_set)
            .unwrap_or(&self.initial)
    }

    pub fn updates(&self) -> &[ValidatorSetUpdate] {
        &self.updates
    }
}

/// Evidence envelope for two conflicting messages attributed to one signer.
/// The digests are caller-supplied; this structure does not authenticate them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EquivocationClaim {
    pub validator: ValidatorId,
    pub height: u64,
    pub round: u32,
    pub step: u8,
    pub first_message_digest: [u8; 32],
    pub second_message_digest: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum EquivocationClaimError {
    #[error("equivocation validator identity is invalid")]
    InvalidIdentity,
    #[error("equivocation claim must contain two distinct message digests")]
    IdenticalMessages,
}

impl EquivocationClaim {
    pub fn new(
        validator: ValidatorId,
        height: u64,
        round: u32,
        step: u8,
        first_message_digest: [u8; 32],
        second_message_digest: [u8; 32],
    ) -> Result<Self, EquivocationClaimError> {
        if !validator.is_valid() {
            return Err(EquivocationClaimError::InvalidIdentity);
        }
        if first_message_digest == second_message_digest {
            return Err(EquivocationClaimError::IdenticalMessages);
        }
        Ok(Self {
            validator,
            height,
            round,
            step,
            first_message_digest,
            second_message_digest,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u8) -> ValidatorId {
        ValidatorId([n; 32])
    }

    fn set(entries: &[(u8, u64)]) -> ValidatorSet {
        ValidatorSet::new(
            entries
                .iter()
                .map(|(raw_id, voting_power)| Validator {
                    id: id(*raw_id),
                    voting_power: *voting_power,
                })
                .collect(),
        )
        .unwrap()
    }

    #[test]
    fn canonical_order_is_independent_of_input_order() {
        let a = set(&[(3, 2), (1, 4), (2, 3)]);
        let b = set(&[(2, 3), (3, 2), (1, 4)]);
        assert_eq!(a, b);
        assert_eq!(a.validators()[0].id, id(1));
    }

    #[test]
    fn rejects_empty_zero_identity_zero_power_and_duplicates() {
        assert_eq!(ValidatorSet::new(vec![]), Err(ValidatorSetError::EmptySet));
        assert_eq!(
            ValidatorSet::new(vec![Validator {
                id: id(0),
                voting_power: 1,
            }]),
            Err(ValidatorSetError::InvalidIdentity)
        );
        assert_eq!(
            ValidatorSet::new(vec![Validator {
                id: id(1),
                voting_power: 0,
            }]),
            Err(ValidatorSetError::ZeroVotingPower)
        );
        assert_eq!(
            ValidatorSet::new(vec![
                Validator {
                    id: id(1),
                    voting_power: 1,
                },
                Validator {
                    id: id(1),
                    voting_power: 2,
                },
            ]),
            Err(ValidatorSetError::DuplicateIdentity)
        );
    }

    #[test]
    fn rejects_total_power_overflow() {
        assert_eq!(
            ValidatorSet::new(vec![
                Validator {
                    id: id(1),
                    voting_power: u64::MAX,
                },
                Validator {
                    id: id(2),
                    voting_power: 1,
                },
            ]),
            Err(ValidatorSetError::VotingPowerOverflow)
        );
    }

    #[test]
    fn quorum_is_strictly_greater_than_two_thirds() {
        let validators = set(&[(1, 1), (2, 1), (3, 1)]);
        assert!(!validators.has_supermajority(&[id(1), id(2)]).unwrap());
        assert!(
            validators
                .has_supermajority(&[id(1), id(2), id(3)])
                .unwrap()
        );
    }

    #[test]
    fn rejects_duplicate_or_unknown_certificate_signers() {
        let validators = set(&[(1, 2), (2, 1)]);
        assert_eq!(
            validators.has_supermajority(&[id(1), id(1)]),
            Err(ValidatorSetError::DuplicateSigner)
        );
        assert_eq!(
            validators.has_supermajority(&[id(9)]),
            Err(ValidatorSetError::UnknownSigner)
        );
    }

    #[test]
    fn schedule_activates_at_exact_height() {
        let first = set(&[(1, 1)]);
        let second = set(&[(2, 1)]);
        let schedule = ValidatorSetSchedule::new(
            first.clone(),
            vec![ValidatorSetUpdate {
                activation_height: 10,
                validator_set: second.clone(),
            }],
        )
        .unwrap();
        assert_eq!(schedule.validator_set_at(9), &first);
        assert_eq!(schedule.validator_set_at(10), &second);
        assert_eq!(schedule.validator_set_at(11), &second);
    }

    #[test]
    fn schedule_rejects_zero_or_non_increasing_heights() {
        let first = set(&[(1, 1)]);
        assert_eq!(
            ValidatorSetSchedule::new(
                first.clone(),
                vec![ValidatorSetUpdate {
                    activation_height: 0,
                    validator_set: first.clone(),
                }],
            ),
            Err(ScheduleError::ZeroActivationHeight)
        );
        assert_eq!(
            ValidatorSetSchedule::new(
                first.clone(),
                vec![
                    ValidatorSetUpdate {
                        activation_height: 5,
                        validator_set: first.clone(),
                    },
                    ValidatorSetUpdate {
                        activation_height: 5,
                        validator_set: first.clone(),
                    },
                ],
            ),
            Err(ScheduleError::NonIncreasingActivationHeight)
        );
    }

    #[test]
    fn equivocation_claim_requires_distinct_messages_and_valid_identity() {
        assert_eq!(
            EquivocationClaim::new(id(1), 4, 0, 2, [7; 32], [7; 32]),
            Err(EquivocationClaimError::IdenticalMessages)
        );
        assert_eq!(
            EquivocationClaim::new(id(0), 4, 0, 2, [7; 32], [8; 32]),
            Err(EquivocationClaimError::InvalidIdentity)
        );
        assert!(EquivocationClaim::new(id(1), 4, 0, 2, [7; 32], [8; 32]).is_ok());
    }
}
