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

/// Internal account key for the stateful admission prototype; not a wire address.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AccountId(pub u64);

/// Minimal account data read from the candidate parent state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccountState {
    pub nonce: u64,
    pub balance: u128,
}

/// Read-only state view. Implementations must return the same account data for
/// the same parent-state version throughout one validation operation.
pub trait AdmissionState {
    fn account(&self, id: AccountId) -> Option<AccountState>;
}

/// Internal transaction fields for admission; not signed or encoded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransactionAdmission {
    pub chain_id: ChainId,
    pub protocol_version: u32,
    pub sender: AccountId,
    pub nonce: u64,
    pub value: u128,
    pub gas_limit: u64,
    pub max_fee_per_gas: u128,
    pub max_priority_fee_per_gas: u128,
}

/// Caller-supplied network limits; these are not protocol constants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdmissionContext {
    pub chain_id: ChainId,
    pub protocol_version: u32,
    pub block_gas_limit: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum TransactionAdmissionError {
    #[error("transaction chain ID does not match the configured chain")]
    WrongChain,
    #[error("transaction protocol version does not match the configured version")]
    WrongProtocolVersion,
    #[error("transaction gas limit must be greater than zero")]
    ZeroGasLimit,
    #[error("transaction gas limit exceeds the block gas limit")]
    GasLimitExceedsBlock,
    #[error("priority fee per gas exceeds maximum fee per gas")]
    PriorityFeeExceedsMaxFee,
    #[error("maximum fee multiplied by gas limit overflows u128")]
    MaximumFeeOverflow,
    #[error("sender account does not exist in the supplied parent state")]
    UnknownSender,
    #[error("transaction nonce does not equal the sender account nonce")]
    NonceMismatch,
    #[error("transaction value plus maximum fee overflows u128")]
    RequiredBalanceOverflow,
    #[error("sender balance is below value plus maximum fee upper bound")]
    InsufficientBalance,
}

/// Checks static envelope fields and returns a maximum-fee upper bound.
/// It does not charge fees or validate signatures, nonce/state, balances,
/// intrinsic gas, canonical bytes, sender authorization or execution.
pub fn validate_transaction_admission(
    context: AdmissionContext,
    transaction: TransactionAdmission,
) -> Result<u128, TransactionAdmissionError> {
    if transaction.chain_id != context.chain_id {
        return Err(TransactionAdmissionError::WrongChain);
    }
    if transaction.protocol_version != context.protocol_version {
        return Err(TransactionAdmissionError::WrongProtocolVersion);
    }
    if transaction.gas_limit == 0 {
        return Err(TransactionAdmissionError::ZeroGasLimit);
    }
    if transaction.gas_limit > context.block_gas_limit {
        return Err(TransactionAdmissionError::GasLimitExceedsBlock);
    }
    if transaction.max_priority_fee_per_gas > transaction.max_fee_per_gas {
        return Err(TransactionAdmissionError::PriorityFeeExceedsMaxFee);
    }
    u128::from(transaction.gas_limit)
        .checked_mul(transaction.max_fee_per_gas)
        .ok_or(TransactionAdmissionError::MaximumFeeOverflow)
}

/// Performs state-aware admission against a caller-selected parent-state view.
/// Returns the maximum-fee upper bound after checking nonce and affordability.
/// The sender identifier is not authenticated: callers must verify the
/// transaction signature and signing domain before trusting this lookup.
pub fn validate_transaction_against_state<S: AdmissionState>(
    context: AdmissionContext,
    transaction: TransactionAdmission,
    state: &S,
) -> Result<u128, TransactionAdmissionError> {
    let maximum_fee = validate_transaction_admission(context, transaction)?;
    let account = state
        .account(transaction.sender)
        .ok_or(TransactionAdmissionError::UnknownSender)?;

    if transaction.nonce != account.nonce {
        return Err(TransactionAdmissionError::NonceMismatch);
    }

    let required_balance = transaction
        .value
        .checked_add(maximum_fee)
        .ok_or(TransactionAdmissionError::RequiredBalanceOverflow)?;
    if account.balance < required_balance {
        return Err(TransactionAdmissionError::InsufficientBalance);
    }

    Ok(maximum_fee)
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
        assert_eq!(
            validate_proposal_header(expected_parent(), proposal()),
            Ok(())
        );
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

    fn admission_context() -> AdmissionContext {
        AdmissionContext {
            chain_id: ChainId(7),
            protocol_version: 1,
            block_gas_limit: 30_000_000,
        }
    }

    fn admission_tx() -> TransactionAdmission {
        TransactionAdmission {
            chain_id: ChainId(7),
            protocol_version: 1,
            sender: AccountId(1),
            nonce: 4,
            value: 500,
            gas_limit: 21_000,
            max_fee_per_gas: 100,
            max_priority_fee_per_gas: 2,
        }
    }

    #[test]
    fn admits_valid_envelope_and_returns_fee_upper_bound() {
        assert_eq!(
            validate_transaction_admission(admission_context(), admission_tx()),
            Ok(2_100_000)
        );
    }

    #[test]
    fn rejects_transaction_wrong_chain() {
        let mut tx = admission_tx();
        tx.chain_id = ChainId(8);
        assert_eq!(
            validate_transaction_admission(admission_context(), tx),
            Err(TransactionAdmissionError::WrongChain)
        );
    }

    #[test]
    fn rejects_wrong_version() {
        let mut tx = admission_tx();
        tx.protocol_version = 2;
        assert_eq!(
            validate_transaction_admission(admission_context(), tx),
            Err(TransactionAdmissionError::WrongProtocolVersion)
        );
    }

    #[test]
    fn rejects_zero_gas() {
        let mut tx = admission_tx();
        tx.gas_limit = 0;
        assert_eq!(
            validate_transaction_admission(admission_context(), tx),
            Err(TransactionAdmissionError::ZeroGasLimit)
        );
    }

    #[test]
    fn rejects_gas_above_block_limit() {
        let mut tx = admission_tx();
        tx.gas_limit = 30_000_001;
        assert_eq!(
            validate_transaction_admission(admission_context(), tx),
            Err(TransactionAdmissionError::GasLimitExceedsBlock)
        );
    }

    #[test]
    fn rejects_priority_fee_above_max_fee() {
        let mut tx = admission_tx();
        tx.max_priority_fee_per_gas = 101;
        assert_eq!(
            validate_transaction_admission(admission_context(), tx),
            Err(TransactionAdmissionError::PriorityFeeExceedsMaxFee)
        );
    }

    #[test]
    fn rejects_fee_multiplication_overflow() {
        let mut tx = admission_tx();
        tx.gas_limit = u64::MAX;
        tx.max_fee_per_gas = u128::MAX;
        let mut context = admission_context();
        context.block_gas_limit = u64::MAX;
        assert_eq!(
            validate_transaction_admission(context, tx),
            Err(TransactionAdmissionError::MaximumFeeOverflow)
        );
    }

    struct TestState {
        account: Option<AccountState>,
    }

    impl AdmissionState for TestState {
        fn account(&self, id: AccountId) -> Option<AccountState> {
            if id == AccountId(1) {
                self.account
            } else {
                None
            }
        }
    }

    fn funded_state() -> TestState {
        TestState {
            account: Some(AccountState {
                nonce: 4,
                balance: 2_100_500,
            }),
        }
    }

    #[test]
    fn stateful_admission_accepts_matching_nonce_and_sufficient_balance() {
        assert_eq!(
            validate_transaction_against_state(
                admission_context(),
                admission_tx(),
                &funded_state()
            ),
            Ok(2_100_000)
        );
    }

    #[test]
    fn stateful_admission_rejects_unknown_sender() {
        let state = TestState { account: None };
        assert_eq!(
            validate_transaction_against_state(admission_context(), admission_tx(), &state),
            Err(TransactionAdmissionError::UnknownSender)
        );
    }

    #[test]
    fn stateful_admission_rejects_nonce_mismatch() {
        let state = TestState {
            account: Some(AccountState {
                nonce: 5,
                balance: u128::MAX,
            }),
        };
        assert_eq!(
            validate_transaction_against_state(admission_context(), admission_tx(), &state),
            Err(TransactionAdmissionError::NonceMismatch)
        );
    }

    #[test]
    fn stateful_admission_rejects_insufficient_balance() {
        let state = TestState {
            account: Some(AccountState {
                nonce: 4,
                balance: 2_100_499,
            }),
        };
        assert_eq!(
            validate_transaction_against_state(admission_context(), admission_tx(), &state),
            Err(TransactionAdmissionError::InsufficientBalance)
        );
    }

    #[test]
    fn stateful_admission_rejects_value_fee_sum_overflow() {
        let mut tx = admission_tx();
        tx.value = u128::MAX;
        let state = TestState {
            account: Some(AccountState {
                nonce: 4,
                balance: u128::MAX,
            }),
        };
        assert_eq!(
            validate_transaction_against_state(admission_context(), tx, &state),
            Err(TransactionAdmissionError::RequiredBalanceOverflow)
        );
    }

    #[test]
    fn zero_fee_remains_permitted_until_policy_is_ratified() {
        let mut tx = admission_tx();
        tx.max_fee_per_gas = 0;
        tx.max_priority_fee_per_gas = 0;
        assert_eq!(
            validate_transaction_admission(admission_context(), tx),
            Ok(0)
        );
    }
}
