//! Consensus primitives and engine-neutral protocol boundary checks.
//!
//! This crate does not implement BFT consensus, authenticate validator votes,
//! verify commit certificates, execute transactions, or establish finality.
//! Engine adapters must treat all engine-provided data as untrusted until the
//! selected engine's verification path has completed.

pub mod application_boundary;
pub mod certificate;
pub mod consensus_message;
pub mod validator_lifecycle;
pub mod validator_lifecycle_commitment;
pub mod validator_lifecycle_checkpoint;
pub mod validator_set;

use apex_types::{ChainId, Hash32, Height};
use std::collections::{BTreeMap, BTreeSet};
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
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
    pub recipient: AccountId,
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
    #[error("recipient account does not exist in the supplied parent state")]
    UnknownRecipient,
    #[error("sender nonce cannot be incremented")]
    NonceOverflow,
    #[error("recipient balance overflows u128 after value transfer")]
    RecipientBalanceOverflow,
    #[error("transaction identifier is duplicated within the block")]
    DuplicateTransactionId,
    #[error("cumulative block gas limit arithmetic overflows u64")]
    CumulativeBlockGasOverflow,
    #[error("cumulative transaction gas limit exceeds the block gas limit")]
    CumulativeBlockGasLimitExceeded,
    #[error("aggregate maximum-fee escrow overflows u128")]
    AggregateMaximumFeeOverflow,
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

/// Complete account values proposed by a pure transaction transition plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlannedAccountUpdate {
    pub account: AccountId,
    pub before: AccountState,
    pub after: AccountState,
}

/// A validated, deterministic plan. It is not persisted or committed by this crate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StateTransitionPlan {
    pub sender: PlannedAccountUpdate,
    pub recipient: Option<PlannedAccountUpdate>,
    pub maximum_fee_escrow: u128,
}

/// Builds a pure transfer/nonce transition plan against one parent-state view.
///
/// The sender is debited by value plus the maximum-fee upper bound (escrow).
/// The recipient receives value. Actual fee settlement/refund is deliberately
/// deferred. The caller must atomically persist all updates or persist none.
pub fn plan_transaction_transition<S: AdmissionState>(
    context: AdmissionContext,
    transaction: TransactionAdmission,
    state: &S,
) -> Result<StateTransitionPlan, TransactionAdmissionError> {
    let maximum_fee = validate_transaction_against_state(context, transaction, state)?;
    let sender_before = state
        .account(transaction.sender)
        .ok_or(TransactionAdmissionError::UnknownSender)?;
    let next_nonce = sender_before
        .nonce
        .checked_add(1)
        .ok_or(TransactionAdmissionError::NonceOverflow)?;
    let sender_debit = if transaction.sender == transaction.recipient {
        maximum_fee
    } else {
        transaction
            .value
            .checked_add(maximum_fee)
            .ok_or(TransactionAdmissionError::RequiredBalanceOverflow)?
    };
    let sender_after = AccountState {
        nonce: next_nonce,
        balance: sender_before.balance - sender_debit,
    };

    if transaction.sender == transaction.recipient {
        return Ok(StateTransitionPlan {
            sender: PlannedAccountUpdate {
                account: transaction.sender,
                before: sender_before,
                after: sender_after,
            },
            recipient: None,
            maximum_fee_escrow: maximum_fee,
        });
    }

    let recipient_before = state
        .account(transaction.recipient)
        .ok_or(TransactionAdmissionError::UnknownRecipient)?;
    let recipient_balance = recipient_before
        .balance
        .checked_add(transaction.value)
        .ok_or(TransactionAdmissionError::RecipientBalanceOverflow)?;
    let recipient_after = AccountState {
        nonce: recipient_before.nonce,
        balance: recipient_balance,
    };

    Ok(StateTransitionPlan {
        sender: PlannedAccountUpdate {
            account: transaction.sender,
            before: sender_before,
            after: sender_after,
        },
        recipient: Some(PlannedAccountUpdate {
            account: transaction.recipient,
            before: recipient_before,
            after: recipient_after,
        }),
        maximum_fee_escrow: maximum_fee,
    })
}

/// Temporary caller-supplied transaction identity used only for duplicate
/// detection inside one candidate block. This is not a transaction hash,
/// signature, replay-protection primitive, or canonical protocol identifier.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TransactionId(pub u64);

/// One ordered transaction entry in a candidate block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlockTransaction {
    pub id: TransactionId,
    pub admission: TransactionAdmission,
}

/// A complete pure account-state plan for one ordered candidate block.
/// Account updates are sorted by AccountId, independent of map iteration
/// behavior. Transaction execution order remains the caller-provided order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockTransitionPlan {
    pub account_updates: Vec<PlannedAccountUpdate>,
    pub transaction_count: usize,
    pub total_gas_limit: u64,
    pub total_maximum_fee_escrow: u128,
}

struct OverlayState<'a, S> {
    parent: &'a S,
    current: &'a BTreeMap<AccountId, AccountState>,
}

impl<S: AdmissionState> AdmissionState for OverlayState<'_, S> {
    fn account(&self, id: AccountId) -> Option<AccountState> {
        self.current
            .get(&id)
            .copied()
            .or_else(|| self.parent.account(id))
    }
}

/// Plans all transactions in caller-supplied order against an in-memory
/// overlay. A later transaction observes all earlier planned nonce, balance
/// and recipient changes. Any error returns no plan and never mutates parent
/// state. This is planning only: no signature checks, execution, persistence,
/// canonical transaction IDs, receipts, state roots, or fee settlement.
pub fn plan_block_transition<S: AdmissionState>(
    context: AdmissionContext,
    transactions: &[BlockTransaction],
    parent_state: &S,
) -> Result<BlockTransitionPlan, TransactionAdmissionError> {
    let mut seen_ids = BTreeSet::new();
    let mut current = BTreeMap::new();
    let mut original = BTreeMap::new();
    let mut total_gas_limit = 0_u64;
    let mut total_maximum_fee_escrow = 0_u128;

    for block_transaction in transactions {
        if !seen_ids.insert(block_transaction.id) {
            return Err(TransactionAdmissionError::DuplicateTransactionId);
        }

        total_gas_limit = total_gas_limit
            .checked_add(block_transaction.admission.gas_limit)
            .ok_or(TransactionAdmissionError::CumulativeBlockGasOverflow)?;
        if total_gas_limit > context.block_gas_limit {
            return Err(TransactionAdmissionError::CumulativeBlockGasLimitExceeded);
        }

        let overlay = OverlayState {
            parent: parent_state,
            current: &current,
        };
        let transaction_plan =
            plan_transaction_transition(context, block_transaction.admission, &overlay)?;

        total_maximum_fee_escrow = total_maximum_fee_escrow
            .checked_add(transaction_plan.maximum_fee_escrow)
            .ok_or(TransactionAdmissionError::AggregateMaximumFeeOverflow)?;

        let sender = transaction_plan.sender;
        original.entry(sender.account).or_insert(sender.before);
        current.insert(sender.account, sender.after);

        if let Some(recipient) = transaction_plan.recipient {
            original
                .entry(recipient.account)
                .or_insert(recipient.before);
            current.insert(recipient.account, recipient.after);
        }
    }

    let account_updates = current
        .iter()
        .map(|(account, after)| PlannedAccountUpdate {
            account: *account,
            before: original[account],
            after: *after,
        })
        .collect();

    Ok(BlockTransitionPlan {
        account_updates,
        transaction_count: transactions.len(),
        total_gas_limit,
        total_maximum_fee_escrow,
    })
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
            recipient: AccountId(2),
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
        sender: Option<AccountState>,
        recipient: Option<AccountState>,
    }

    impl AdmissionState for TestState {
        fn account(&self, id: AccountId) -> Option<AccountState> {
            match id {
                AccountId(1) => self.sender,
                AccountId(2) => self.recipient,
                _ => None,
            }
        }
    }

    fn funded_state() -> TestState {
        TestState {
            sender: Some(AccountState {
                nonce: 4,
                balance: 2_100_500,
            }),
            recipient: Some(AccountState {
                nonce: 0,
                balance: 1_000,
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
        let state = TestState {
            sender: None,
            recipient: None,
        };
        assert_eq!(
            validate_transaction_against_state(admission_context(), admission_tx(), &state),
            Err(TransactionAdmissionError::UnknownSender)
        );
    }

    #[test]
    fn stateful_admission_rejects_nonce_mismatch() {
        let state = TestState {
            sender: Some(AccountState {
                nonce: 5,
                balance: u128::MAX,
            }),
            recipient: Some(AccountState {
                nonce: 0,
                balance: 1_000,
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
            sender: Some(AccountState {
                nonce: 4,
                balance: 2_100_499,
            }),
            recipient: Some(AccountState {
                nonce: 0,
                balance: 1_000,
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
            sender: Some(AccountState {
                nonce: 4,
                balance: u128::MAX,
            }),
            recipient: Some(AccountState {
                nonce: 0,
                balance: 1_000,
            }),
        };
        assert_eq!(
            validate_transaction_against_state(admission_context(), tx, &state),
            Err(TransactionAdmissionError::RequiredBalanceOverflow)
        );
    }

    #[test]
    fn transition_plan_debits_sender_advances_nonce_and_credits_recipient() {
        let state = funded_state();
        let plan =
            plan_transaction_transition(admission_context(), admission_tx(), &state).unwrap();
        assert_eq!(plan.maximum_fee_escrow, 2_100_000);
        assert_eq!(plan.sender.before.nonce, 4);
        assert_eq!(plan.sender.after.nonce, 5);
        assert_eq!(plan.sender.after.balance, 0);
        let recipient = plan.recipient.unwrap();
        assert_eq!(recipient.before.balance, 1_000);
        assert_eq!(recipient.after.balance, 1_500);
        // Planning is pure: the supplied parent view remains unchanged.
        assert_eq!(state.sender.unwrap().nonce, 4);
        assert_eq!(state.recipient.unwrap().balance, 1_000);
    }

    #[test]
    fn transition_plan_rejects_missing_recipient_without_mutating_parent() {
        let mut state = funded_state();
        state.recipient = None;
        assert_eq!(
            plan_transaction_transition(admission_context(), admission_tx(), &state),
            Err(TransactionAdmissionError::UnknownRecipient)
        );
        assert_eq!(state.sender.unwrap().nonce, 4);
    }

    #[test]
    fn transition_plan_handles_self_transfer_without_double_update() {
        let state = funded_state();
        let mut tx = admission_tx();
        tx.recipient = tx.sender;
        tx.value = 100;
        let plan = plan_transaction_transition(admission_context(), tx, &state).unwrap();
        assert!(plan.recipient.is_none());
        assert_eq!(plan.sender.after.nonce, 5);
        assert_eq!(plan.sender.after.balance, 500);
    }

    #[test]
    fn transition_plan_rejects_recipient_balance_overflow() {
        let mut state = funded_state();
        state.recipient = Some(AccountState {
            nonce: 0,
            balance: u128::MAX,
        });
        assert_eq!(
            plan_transaction_transition(admission_context(), admission_tx(), &state),
            Err(TransactionAdmissionError::RecipientBalanceOverflow)
        );
    }

    struct MapState(BTreeMap<AccountId, AccountState>);

    impl AdmissionState for MapState {
        fn account(&self, id: AccountId) -> Option<AccountState> {
            self.0.get(&id).copied()
        }
    }

    fn block_state() -> MapState {
        MapState(BTreeMap::from([
            (
                AccountId(1),
                AccountState {
                    nonce: 0,
                    balance: 10_000_000,
                },
            ),
            (
                AccountId(2),
                AccountState {
                    nonce: 0,
                    balance: 1_000,
                },
            ),
            (
                AccountId(3),
                AccountState {
                    nonce: 0,
                    balance: 2_000,
                },
            ),
        ]))
    }

    fn block_tx(id: u64, sender: u64, recipient: u64, nonce: u64, value: u128) -> BlockTransaction {
        BlockTransaction {
            id: TransactionId(id),
            admission: TransactionAdmission {
                chain_id: ChainId(7),
                protocol_version: 1,
                sender: AccountId(sender),
                recipient: AccountId(recipient),
                nonce,
                value,
                gas_limit: 21_000,
                max_fee_per_gas: 1,
                max_priority_fee_per_gas: 0,
            },
        }
    }

    #[test]
    fn block_plan_applies_sequential_nonces_and_merges_recipient_credits() {
        let state = block_state();
        let transactions = [
            block_tx(1, 1, 2, 0, 100),
            block_tx(2, 1, 2, 1, 200),
            block_tx(3, 1, 3, 2, 300),
        ];
        let plan = plan_block_transition(admission_context(), &transactions, &state).unwrap();

        assert_eq!(plan.transaction_count, 3);
        assert_eq!(plan.total_gas_limit, 63_000);
        assert_eq!(plan.total_maximum_fee_escrow, 63_000);
        assert_eq!(plan.account_updates.len(), 3);
        assert_eq!(plan.account_updates[0].account, AccountId(1));
        assert_eq!(plan.account_updates[0].before.nonce, 0);
        assert_eq!(plan.account_updates[0].after.nonce, 3);
        assert_eq!(plan.account_updates[0].after.balance, 9_936_400);
        assert_eq!(plan.account_updates[1].before.balance, 1_000);
        assert_eq!(plan.account_updates[1].after.balance, 1_300);
        assert_eq!(plan.account_updates[2].before.balance, 2_000);
        assert_eq!(plan.account_updates[2].after.balance, 2_300);
        assert_eq!(state.account(AccountId(1)).unwrap().nonce, 0);
        assert_eq!(state.account(AccountId(2)).unwrap().balance, 1_000);
    }

    #[test]
    fn block_plan_rejects_duplicate_opaque_transaction_ids() {
        let state = block_state();
        let transactions = [block_tx(9, 1, 2, 0, 100), block_tx(9, 1, 3, 1, 100)];
        assert_eq!(
            plan_block_transition(admission_context(), &transactions, &state),
            Err(TransactionAdmissionError::DuplicateTransactionId)
        );
    }

    #[test]
    fn block_plan_enforces_cumulative_gas_limit() {
        let state = block_state();
        let mut context = admission_context();
        context.block_gas_limit = 30_000;
        let transactions = [block_tx(1, 1, 2, 0, 100), block_tx(2, 1, 3, 1, 100)];
        assert_eq!(
            plan_block_transition(context, &transactions, &state),
            Err(TransactionAdmissionError::CumulativeBlockGasLimitExceeded)
        );
    }

    #[test]
    fn block_plan_failure_on_late_transaction_returns_no_plan_and_preserves_parent() {
        let state = block_state();
        let transactions = [block_tx(1, 1, 2, 0, 100), block_tx(2, 1, 3, 99, 100)];
        assert_eq!(
            plan_block_transition(admission_context(), &transactions, &state),
            Err(TransactionAdmissionError::NonceMismatch)
        );
        assert_eq!(state.account(AccountId(1)).unwrap().nonce, 0);
        assert_eq!(state.account(AccountId(2)).unwrap().balance, 1_000);
        assert_eq!(state.account(AccountId(3)).unwrap().balance, 2_000);
    }

    #[test]
    fn block_plan_is_deterministic_for_identical_ordered_inputs() {
        let state = block_state();
        let transactions = [block_tx(4, 1, 2, 0, 123), block_tx(5, 1, 3, 1, 456)];
        let first = plan_block_transition(admission_context(), &transactions, &state).unwrap();
        let second = plan_block_transition(admission_context(), &transactions, &state).unwrap();
        assert_eq!(first, second);
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
