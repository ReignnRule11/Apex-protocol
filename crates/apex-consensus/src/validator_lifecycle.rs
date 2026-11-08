//! Executable, policy-neutral validator lifecycle reference state machine.
//!
//! This in-memory model enforces local transition ordering and state-domain
//! separation. It is not a governance authority, durable registry, consensus
//! validator-set updater, canonical wire codec, or production authorization
//! system. Membership confirmation requires an injected proof verifier.

use std::collections::BTreeMap;
use thiserror::Error;

/// Chain/operator/validator namespace for one lifecycle record.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct LifecycleScope {
    pub chain_id: [u8; 32],
    pub protocol_version: u32,
    pub operator_id: [u8; 32],
    pub validator_id: [u8; 32],
}

pub type TransitionRequestId = [u8; 32];
pub type ProtocolTransitionId = [u8; 32];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationStatus {
    Unregistered,
    Submitted,
    UnderReview,
    RemediationRequired,
    Rejected,
    Withdrawn,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EligibilityStatus {
    Unknown,
    Eligible,
    Ineligible,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GovernanceDisposition {
    Undecided,
    Approved,
    Denied,
    Appealed,
    Suspended,
    Revoked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConsensusMembership {
    Inactive,
    ActivationPending {
        transition_id: ProtocolTransitionId,
    },
    Active {
        activation_transition_id: ProtocolTransitionId,
        activation_height: u64,
    },
    ExitPending {
        transition_id: ProtocolTransitionId,
    },
    InactiveAfterExit {
        exit_transition_id: ProtocolTransitionId,
        exit_height: u64,
    },
}

/// Application, eligibility, governance and consensus are separate domains.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LifecycleState {
    pub application: ApplicationStatus,
    pub eligibility: EligibilityStatus,
    pub disposition: GovernanceDisposition,
    pub membership: ConsensusMembership,
    pub evidence_revision: u64,
    pub policy_revision: u64,
    pub revision: u64,
}

impl Default for LifecycleState {
    fn default() -> Self {
        Self {
            application: ApplicationStatus::Unregistered,
            eligibility: EligibilityStatus::Unknown,
            disposition: GovernanceDisposition::Undecided,
            membership: ConsensusMembership::Inactive,
            evidence_revision: 0,
            policy_revision: 0,
            revision: 0,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LifecycleEvent {
    Submit {
        evidence_revision: u64,
        policy_revision: u64,
    },
    StartReview,
    RequireRemediation,
    ResubmitEvidence {
        evidence_revision: u64,
    },
    AssessEligibility {
        result: EligibilityStatus,
        policy_revision: u64,
    },
    Approve {
        decision_id: [u8; 32],
        policy_revision: u64,
    },
    Deny {
        decision_id: [u8; 32],
        policy_revision: u64,
    },
    Appeal {
        appeal_id: [u8; 32],
    },
    Withdraw,
    ProposeActivation {
        transition_id: ProtocolTransitionId,
    },
    ConfirmActivation {
        transition_id: ProtocolTransitionId,
        committed_height: u64,
        proof: Vec<u8>,
    },
    ProposeExit {
        transition_id: ProtocolTransitionId,
    },
    ConfirmExit {
        transition_id: ProtocolTransitionId,
        committed_height: u64,
        proof: Vec<u8>,
    },
    SuspendDisposition {
        decision_id: [u8; 32],
    },
    RevokeDisposition {
        decision_id: [u8; 32],
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransitionEnvelope {
    pub request_id: TransitionRequestId,
    pub scope: LifecycleScope,
    pub expected_revision: u64,
    pub event: LifecycleEvent,
}

/// Production callers must verify a real protocol commit, not an API status.
pub trait ProtocolTransitionVerifier {
    fn verify_committed_transition(
        &self,
        scope: LifecycleScope,
        transition_id: ProtocolTransitionId,
        committed_height: u64,
        proof: &[u8],
    ) -> Result<(), ProtocolProofError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum ProtocolProofError {
    #[error("protocol transition proof is invalid")]
    Invalid,
    #[error("protocol transition verification is unavailable")]
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum LifecycleError {
    #[error("chain, operator and validator identities must be non-zero")]
    InvalidScope,
    #[error("protocol version must be non-zero")]
    InvalidProtocolVersion,
    #[error("transition request ID must be non-zero")]
    InvalidRequestId,
    #[error("expected lifecycle revision is stale")]
    StaleRevision,
    #[error("lifecycle revision overflow")]
    RevisionOverflow,
    #[error("request ID was reused for a different transition envelope")]
    RequestIdConflict,
    #[error("invalid application transition")]
    InvalidApplicationTransition,
    #[error("invalid eligibility transition")]
    InvalidEligibilityTransition,
    #[error("invalid governance disposition transition")]
    InvalidDispositionTransition,
    #[error("invalid consensus membership transition")]
    InvalidMembershipTransition,
    #[error("evidence revision must be non-zero and strictly increase")]
    InvalidEvidenceRevision,
    #[error("policy revision must be non-zero")]
    InvalidPolicyRevision,
    #[error("decision, appeal or protocol transition ID must be non-zero")]
    InvalidReference,
    #[error("committed height must be non-zero")]
    InvalidCommittedHeight,
    #[error("protocol transition proof must not be empty")]
    EmptyProtocolProof,
    #[error("protocol transition proof verification failed: {0}")]
    ProtocolProof(ProtocolProofError),
    #[error("history revision does not match its append position")]
    HistoryInvariant,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleRecord {
    pub envelope: TransitionEnvelope,
    pub resulting_state: LifecycleState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplyOutcome {
    Applied {
        revision: u64,
    },
    Duplicate {
        original_revision: u64,
    },
}

/// In-memory reference ledger; not durable, distributed or authenticated.
#[derive(Clone, Debug, Default)]
pub struct LifecycleLedger {
    states: BTreeMap<LifecycleScope, LifecycleState>,
    history: BTreeMap<LifecycleScope, Vec<LifecycleRecord>>,
    requests: BTreeMap<TransitionRequestId, TransitionEnvelope>,
}

impl LifecycleLedger {
    pub fn state(&self, scope: LifecycleScope) -> LifecycleState {
        self.states.get(&scope).copied().unwrap_or_default()
    }

    pub fn history(&self, scope: LifecycleScope) -> &[LifecycleRecord] {
        self.history
            .get(&scope)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn apply(
        &mut self,
        envelope: TransitionEnvelope,
        verifier: &impl ProtocolTransitionVerifier,
    ) -> Result<ApplyOutcome, LifecycleError> {
        validate_scope(envelope.scope)?;
        if is_zero(&envelope.request_id) {
            return Err(LifecycleError::InvalidRequestId);
        }

        if let Some(existing) = self.requests.get(&envelope.request_id) {
            if existing != &envelope {
                return Err(LifecycleError::RequestIdConflict);
            }
            let revision = self
                .history(existing.scope)
                .iter()
                .find(|record| record.envelope.request_id == envelope.request_id)
                .map(|record| record.resulting_state.revision)
                .ok_or(LifecycleError::HistoryInvariant)?;
            return Ok(ApplyOutcome::Duplicate {
                original_revision: revision,
            });
        }

        let current = self.state(envelope.scope);
        if envelope.expected_revision != current.revision {
            return Err(LifecycleError::StaleRevision);
        }

        let mut next = current;
        apply_event(&envelope, &mut next, verifier)?;
        next.revision = current
            .revision
            .checked_add(1)
            .ok_or(LifecycleError::RevisionOverflow)?;

        let record = LifecycleRecord {
            envelope: envelope.clone(),
            resulting_state: next,
        };
        self.states.insert(envelope.scope, next);
        self.history
            .entry(envelope.scope)
            .or_default()
            .push(record);
        self.requests.insert(envelope.request_id, envelope);

        Ok(ApplyOutcome::Applied {
            revision: next.revision,
        })
    }
}

fn apply_event(
    envelope: &TransitionEnvelope,
    state: &mut LifecycleState,
    verifier: &impl ProtocolTransitionVerifier,
) -> Result<(), LifecycleError> {
    match &envelope.event {
        LifecycleEvent::Submit {
            evidence_revision,
            policy_revision,
        } => {
            if state.application != ApplicationStatus::Unregistered {
                return Err(LifecycleError::InvalidApplicationTransition);
            }
            require_evidence(*evidence_revision)?;
            require_policy(*policy_revision)?;
            state.application = ApplicationStatus::Submitted;
            state.evidence_revision = *evidence_revision;
            state.policy_revision = *policy_revision;
        }
        LifecycleEvent::StartReview => {
            if !matches!(
                state.application,
                ApplicationStatus::Submitted | ApplicationStatus::RemediationRequired
            ) {
                return Err(LifecycleError::InvalidApplicationTransition);
            }
            state.application = ApplicationStatus::UnderReview;
        }
        LifecycleEvent::RequireRemediation => {
            if state.application != ApplicationStatus::UnderReview {
                return Err(LifecycleError::InvalidApplicationTransition);
            }
            state.application = ApplicationStatus::RemediationRequired;
            state.eligibility = EligibilityStatus::Unknown;
        }
        LifecycleEvent::ResubmitEvidence { evidence_revision } => {
            if state.application != ApplicationStatus::RemediationRequired {
                return Err(LifecycleError::InvalidApplicationTransition);
            }
            require_evidence(*evidence_revision)?;
            if *evidence_revision <= state.evidence_revision {
                return Err(LifecycleError::InvalidEvidenceRevision);
            }
            state.evidence_revision = *evidence_revision;
            state.eligibility = EligibilityStatus::Unknown;
            state.application = ApplicationStatus::UnderReview;
        }
        LifecycleEvent::AssessEligibility {
            result,
            policy_revision,
        } => {
            if state.application != ApplicationStatus::UnderReview
                || *result == EligibilityStatus::Unknown
            {
                return Err(LifecycleError::InvalidEligibilityTransition);
            }
            require_policy(*policy_revision)?;
            if *policy_revision != state.policy_revision {
                return Err(LifecycleError::InvalidPolicyRevision);
            }
            state.eligibility = *result;
            if *result == EligibilityStatus::Ineligible {
                state.disposition = GovernanceDisposition::Undecided;
            }
        }
        LifecycleEvent::Approve {
            decision_id,
            policy_revision,
        }
        | LifecycleEvent::Deny {
            decision_id,
            policy_revision,
        } => {
            require_reference(decision_id)?;
            require_policy(*policy_revision)?;
            if state.eligibility != EligibilityStatus::Eligible
                || *policy_revision != state.policy_revision
                || !matches!(
                    state.disposition,
                    GovernanceDisposition::Undecided | GovernanceDisposition::Appealed
                )
            {
                return Err(LifecycleError::InvalidDispositionTransition);
            }
            state.disposition = if matches!(&envelope.event, LifecycleEvent::Approve { .. }) {
                GovernanceDisposition::Approved
            } else {
                GovernanceDisposition::Denied
            };
        }
        LifecycleEvent::Appeal { appeal_id } => {
            require_reference(appeal_id)?;
            if state.disposition != GovernanceDisposition::Denied {
                return Err(LifecycleError::InvalidDispositionTransition);
            }
            state.disposition = GovernanceDisposition::Appealed;
        }
        LifecycleEvent::Withdraw => {
            if !matches!(
                state.application,
                ApplicationStatus::Submitted
                    | ApplicationStatus::UnderReview
                    | ApplicationStatus::RemediationRequired
            ) {
                return Err(LifecycleError::InvalidApplicationTransition);
            }
            state.application = ApplicationStatus::Withdrawn;
        }
        LifecycleEvent::ProposeActivation { transition_id } => {
            require_reference(transition_id)?;
            if state.eligibility != EligibilityStatus::Eligible
                || state.disposition != GovernanceDisposition::Approved
                || state.membership != ConsensusMembership::Inactive
            {
                return Err(LifecycleError::InvalidMembershipTransition);
            }
            state.membership = ConsensusMembership::ActivationPending {
                transition_id: *transition_id,
            };
        }
        LifecycleEvent::ConfirmActivation {
            transition_id,
            committed_height,
            proof,
        } => {
            verify_membership_transition(
                envelope.scope,
                *transition_id,
                *committed_height,
                proof,
                state.membership
                    == (ConsensusMembership::ActivationPending {
                        transition_id: *transition_id,
                    }),
                verifier,
            )?;
            state.membership = ConsensusMembership::Active {
                activation_transition_id: *transition_id,
                activation_height: *committed_height,
            };
        }
        LifecycleEvent::ProposeExit { transition_id } => {
            require_reference(transition_id)?;
            if !matches!(state.membership, ConsensusMembership::Active { .. }) {
                return Err(LifecycleError::InvalidMembershipTransition);
            }
            state.membership = ConsensusMembership::ExitPending {
                transition_id: *transition_id,
            };
        }
        LifecycleEvent::ConfirmExit {
            transition_id,
            committed_height,
            proof,
        } => {
            verify_membership_transition(
                envelope.scope,
                *transition_id,
                *committed_height,
                proof,
                state.membership
                    == (ConsensusMembership::ExitPending {
                        transition_id: *transition_id,
                    }),
                verifier,
            )?;
            state.membership = ConsensusMembership::InactiveAfterExit {
                exit_transition_id: *transition_id,
                exit_height: *committed_height,
            };
        }
        LifecycleEvent::SuspendDisposition { decision_id } => {
            require_reference(decision_id)?;
            if !matches!(
                state.disposition,
                GovernanceDisposition::Approved | GovernanceDisposition::Suspended
            ) {
                return Err(LifecycleError::InvalidDispositionTransition);
            }
            state.disposition = GovernanceDisposition::Suspended;
        }
        LifecycleEvent::RevokeDisposition { decision_id } => {
            require_reference(decision_id)?;
            if !matches!(
                state.disposition,
                GovernanceDisposition::Approved | GovernanceDisposition::Suspended
            ) {
                return Err(LifecycleError::InvalidDispositionTransition);
            }
            // Governance revocation never directly rewrites consensus membership.
            state.disposition = GovernanceDisposition::Revoked;
        }
    }
    Ok(())
}

fn verify_membership_transition(
    scope: LifecycleScope,
    transition_id: ProtocolTransitionId,
    committed_height: u64,
    proof: &[u8],
    expected_state: bool,
    verifier: &impl ProtocolTransitionVerifier,
) -> Result<(), LifecycleError> {
    require_reference(&transition_id)?;
    if committed_height == 0 {
        return Err(LifecycleError::InvalidCommittedHeight);
    }
    if proof.is_empty() {
        return Err(LifecycleError::EmptyProtocolProof);
    }
    if !expected_state {
        return Err(LifecycleError::InvalidMembershipTransition);
    }
    verifier
        .verify_committed_transition(scope, transition_id, committed_height, proof)
        .map_err(LifecycleError::ProtocolProof)
}

fn validate_scope(scope: LifecycleScope) -> Result<(), LifecycleError> {
    if is_zero(&scope.chain_id)
        || is_zero(&scope.operator_id)
        || is_zero(&scope.validator_id)
    {
        return Err(LifecycleError::InvalidScope);
    }
    if scope.protocol_version == 0 {
        return Err(LifecycleError::InvalidProtocolVersion);
    }
    Ok(())
}

fn is_zero(value: &[u8; 32]) -> bool {
    value.iter().all(|byte| *byte == 0)
}

fn require_reference(value: &[u8; 32]) -> Result<(), LifecycleError> {
    if is_zero(value) {
        Err(LifecycleError::InvalidReference)
    } else {
        Ok(())
    }
}

fn require_evidence(value: u64) -> Result<(), LifecycleError> {
    if value == 0 {
        Err(LifecycleError::InvalidEvidenceRevision)
    } else {
        Ok(())
    }
}

fn require_policy(value: u64) -> Result<(), LifecycleError> {
    if value == 0 {
        Err(LifecycleError::InvalidPolicyRevision)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestVerifier {
        accept: bool,
    }

    impl ProtocolTransitionVerifier for TestVerifier {
        fn verify_committed_transition(
            &self,
            _scope: LifecycleScope,
            _transition_id: ProtocolTransitionId,
            _committed_height: u64,
            _proof: &[u8],
        ) -> Result<(), ProtocolProofError> {
            if self.accept {
                Ok(())
            } else {
                Err(ProtocolProofError::Invalid)
            }
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

    fn envelope(
        request: u8,
        revision: u64,
        event: LifecycleEvent,
    ) -> TransitionEnvelope {
        TransitionEnvelope {
            request_id: [request; 32],
            scope: scope(),
            expected_revision: revision,
            event,
        }
    }

    fn apply(
        ledger: &mut LifecycleLedger,
        request: u8,
        event: LifecycleEvent,
    ) -> Result<ApplyOutcome, LifecycleError> {
        let revision = ledger.state(scope()).revision;
        ledger.apply(
            envelope(request, revision, event),
            &TestVerifier { accept: true },
        )
    }

    fn reach_eligible(ledger: &mut LifecycleLedger) {
        apply(
            ledger,
            1,
            LifecycleEvent::Submit {
                evidence_revision: 1,
                policy_revision: 1,
            },
        )
        .unwrap();
        apply(ledger, 2, LifecycleEvent::StartReview).unwrap();
        apply(
            ledger,
            3,
            LifecycleEvent::AssessEligibility {
                result: EligibilityStatus::Eligible,
                policy_revision: 1,
            },
        )
        .unwrap();
        apply(
            ledger,
            4,
            LifecycleEvent::Approve {
                decision_id: [4; 32],
                policy_revision: 1,
            },
        )
        .unwrap();
    }

    #[test]
    fn application_eligibility_approval_and_membership_are_separate() {
        let mut ledger = LifecycleLedger::default();
        reach_eligible(&mut ledger);
        let state = ledger.state(scope());

        assert_eq!(state.application, ApplicationStatus::UnderReview);
        assert_eq!(state.eligibility, EligibilityStatus::Eligible);
        assert_eq!(state.disposition, GovernanceDisposition::Approved);
        assert_eq!(state.membership, ConsensusMembership::Inactive);
    }

    #[test]
    fn activation_requires_pending_transition_and_verified_commit() {
        let mut ledger = LifecycleLedger::default();
        reach_eligible(&mut ledger);
        apply(
            &mut ledger,
            5,
            LifecycleEvent::ProposeActivation {
                transition_id: [5; 32],
            },
        )
        .unwrap();

        let failed = ledger.apply(
            envelope(
                6,
                5,
                LifecycleEvent::ConfirmActivation {
                    transition_id: [5; 32],
                    committed_height: 10,
                    proof: vec![9],
                },
            ),
            &TestVerifier { accept: false },
        );
        assert_eq!(
            failed,
            Err(LifecycleError::ProtocolProof(ProtocolProofError::Invalid))
        );
        assert_eq!(
            ledger.state(scope()).membership,
            ConsensusMembership::ActivationPending {
                transition_id: [5; 32],
            }
        );

        apply(
            &mut ledger,
            7,
            LifecycleEvent::ConfirmActivation {
                transition_id: [5; 32],
                committed_height: 10,
                proof: vec![9],
            },
        )
        .unwrap();
        assert_eq!(
            ledger.state(scope()).membership,
            ConsensusMembership::Active {
                activation_transition_id: [5; 32],
                activation_height: 10,
            }
        );
    }

    #[test]
    fn revocation_does_not_rewrite_consensus_membership() {
        let mut ledger = LifecycleLedger::default();
        reach_eligible(&mut ledger);
        apply(
            &mut ledger,
            5,
            LifecycleEvent::ProposeActivation {
                transition_id: [5; 32],
            },
        )
        .unwrap();
        apply(
            &mut ledger,
            6,
            LifecycleEvent::ConfirmActivation {
                transition_id: [5; 32],
                committed_height: 10,
                proof: vec![9],
            },
        )
        .unwrap();
        apply(
            &mut ledger,
            7,
            LifecycleEvent::RevokeDisposition {
                decision_id: [7; 32],
            },
        )
        .unwrap();

        assert_eq!(
            ledger.state(scope()).disposition,
            GovernanceDisposition::Revoked
        );
        assert_eq!(
            ledger.state(scope()).membership,
            ConsensusMembership::Active {
                activation_transition_id: [5; 32],
                activation_height: 10,
            }
        );
    }

    #[test]
    fn duplicate_request_is_idempotent_and_conflicting_reuse_fails() {
        let mut ledger = LifecycleLedger::default();
        let first = envelope(
            1,
            0,
            LifecycleEvent::Submit {
                evidence_revision: 1,
                policy_revision: 1,
            },
        );

        assert_eq!(
            ledger.apply(first.clone(), &TestVerifier { accept: true }),
            Ok(ApplyOutcome::Applied { revision: 1 })
        );
        assert_eq!(
            ledger.apply(first, &TestVerifier { accept: true }),
            Ok(ApplyOutcome::Duplicate {
                original_revision: 1,
            })
        );

        let conflict = envelope(1, 1, LifecycleEvent::StartReview);
        assert_eq!(
            ledger.apply(conflict, &TestVerifier { accept: true }),
            Err(LifecycleError::RequestIdConflict)
        );
        assert_eq!(ledger.history(scope()).len(), 1);
    }

    #[test]
    fn stale_revision_fails_without_mutation() {
        let mut ledger = LifecycleLedger::default();
        apply(
            &mut ledger,
            1,
            LifecycleEvent::Submit {
                evidence_revision: 1,
                policy_revision: 1,
            },
        )
        .unwrap();

        let stale = envelope(2, 0, LifecycleEvent::StartReview);
        assert_eq!(
            ledger.apply(stale, &TestVerifier { accept: true }),
            Err(LifecycleError::StaleRevision)
        );
        assert_eq!(ledger.state(scope()).revision, 1);
    }

    #[test]
    fn evidence_revision_must_advance_after_remediation() {
        let mut ledger = LifecycleLedger::default();
        apply(
            &mut ledger,
            1,
            LifecycleEvent::Submit {
                evidence_revision: 1,
                policy_revision: 1,
            },
        )
        .unwrap();
        apply( &mut ledger, 2, LifecycleEvent::StartReview).unwrap();
        apply(&mut ledger, 3, LifecycleEvent::RequireRemediation).unwrap();

        let result = apply(
            &mut ledger,
            4,
            LifecycleEvent::ResubmitEvidence {
                evidence_revision: 1,
            },
        );
        assert_eq!(result, Err(LifecycleError::InvalidEvidenceRevision));
        assert_eq!(ledger.state(scope()).evidence_revision, 1);
    }

    #[test]
    fn exit_requires_its_own_verified_protocol_transition() {
        let mut ledger = LifecycleLedger::default();
        reach_eligible(&mut ledger);
        apply(
            &mut ledger,
            5,
            LifecycleEvent::ProposeActivation {
                transition_id: [5; 32],
            },
        )
        .unwrap();
        apply(
            &mut ledger,
            6,
            LifecycleEvent::ConfirmActivation {
                transition_id: [5; 32],
                committed_height: 10,
                proof: vec![9],
            },
        )
        .unwrap();
        apply(
            &mut ledger,
            7,
            LifecycleEvent::ProposeExit {
                transition_id: [8; 32],
            },
        )
        .unwrap();
        apply(
            &mut ledger,
            8,
            LifecycleEvent::ConfirmExit {
                transition_id: [8; 32],
                committed_height: 12,
                proof: vec![8],
            },
        )
        .unwrap();

        assert_eq!(
            ledger.state(scope()).membership,
            ConsensusMembership::InactiveAfterExit {
                exit_transition_id: [8; 32],
                exit_height: 12,
            }
        );
    }

    #[test]
    fn histories_are_append_only_and_revision_ordered() {
        let mut ledger = LifecycleLedger::default();
        reach_eligible(&mut ledger);
        let history = ledger.history(scope());

        assert_eq!(history.len(), 4);
        for (index, record) in history.iter().enumerate() {
            assert_eq!(record.resulting_state.revision, index as u64 + 1);
        }
    }

    #[test]
    fn scopes_are_isolated() {
        let mut ledger = LifecycleLedger::default();
        apply(
            &mut ledger,
            1,
            LifecycleEvent::Submit {
                evidence_revision: 1,
                policy_revision: 1,
            },
        )
        .unwrap();

        let other = LifecycleScope {
            validator_id: [4; 32],
            ..scope()
        };
        assert_eq!(ledger.state(other), LifecycleState::default());
    }
}
