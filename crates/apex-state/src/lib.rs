//! Versioned snapshots and an atomic state-application boundary.
//!
//! The in-memory adapter is a reference implementation for tests. Durable
//! adapters live in separate crates and must preserve the same contract.

use apex_consensus::{
    AccountId, AccountState, AdmissionContext, AdmissionState, BlockTransaction,
    BlockTransitionPlan, PlannedAccountUpdate, plan_block_transition,
};
use std::collections::BTreeMap;
use thiserror::Error;

/// Monotonic internal concurrency token; not a block height or state root.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct StateVersion(pub u64);

/// Immutable-by-interface copy of one state version.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionedStateSnapshot {
    version: StateVersion,
    accounts: BTreeMap<AccountId, AccountState>,
}

impl VersionedStateSnapshot {
    pub fn new(version: StateVersion, accounts: BTreeMap<AccountId, AccountState>) -> Self {
        Self { version, accounts }
    }

    pub fn version(&self) -> StateVersion {
        self.version
    }

    pub fn accounts(&self) -> &BTreeMap<AccountId, AccountState> {
        &self.accounts
    }
}

impl AdmissionState for VersionedStateSnapshot {
    fn account(&self, id: AccountId) -> Option<AccountState> {
        self.accounts.get(&id).copied()
    }
}

/// Plan bound to the exact snapshot version used for admission and planning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionedBlockTransitionPlan {
    pub parent_version: StateVersion,
    pub transition: BlockTransitionPlan,
}

pub fn plan_versioned_block_transition(
    context: AdmissionContext,
    transactions: &[BlockTransaction],
    snapshot: &VersionedStateSnapshot,
) -> Result<VersionedBlockTransitionPlan, apex_consensus::TransactionAdmissionError> {
    Ok(VersionedBlockTransitionPlan {
        parent_version: snapshot.version(),
        transition: plan_block_transition(context, transactions, snapshot)?,
    })
}

#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum StateStoreError {
    #[error("plan parent version is stale: expected {expected:?}, current {actual:?}")]
    StaleVersion {
        expected: StateVersion,
        actual: StateVersion,
    },
    #[error("planned before-state does not match account {0:?}")]
    BeforeStateMismatch(AccountId),
    #[error("account updates must be strictly increasing and unique by AccountId")]
    DuplicateOrUnorderedAccountUpdate,
    #[error("state version cannot be incremented")]
    VersionOverflow,
    #[error("persistent state store is not initialized")]
    Uninitialized,
    #[error("persistent state store schema version {0} is not supported")]
    UnsupportedSchema(u64),
    #[error("persistent state metadata is malformed")]
    CorruptMetadata,
    #[error("persisted account {0:?} has an invalid representation")]
    CorruptAccount(AccountId),
    #[error("storage backend error: {0}")]
    Backend(String),
}

/// Implementations must provide consistent snapshots and a linearizable
/// compare-and-swap apply: validate all preconditions, then publish every
/// account update and the next version atomically, or publish nothing.
pub trait StateStore {
    fn snapshot(&self) -> Result<VersionedStateSnapshot, StateStoreError>;
    fn apply(
        &mut self,
        plan: &VersionedBlockTransitionPlan,
    ) -> Result<StateVersion, StateStoreError>;
}

/// Process-local reference adapter. Not durable, crash-safe, or multi-process.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InMemoryStateStore {
    version: StateVersion,
    accounts: BTreeMap<AccountId, AccountState>,
}

impl InMemoryStateStore {
    pub fn new(version: StateVersion, accounts: BTreeMap<AccountId, AccountState>) -> Self {
        Self { version, accounts }
    }
}

impl StateStore for InMemoryStateStore {
    fn snapshot(&self) -> Result<VersionedStateSnapshot, StateStoreError> {
        Ok(VersionedStateSnapshot::new(
            self.version,
            self.accounts.clone(),
        ))
    }

    fn apply(
        &mut self,
        plan: &VersionedBlockTransitionPlan,
    ) -> Result<StateVersion, StateStoreError> {
        if plan.parent_version != self.version {
            return Err(StateStoreError::StaleVersion {
                expected: plan.parent_version,
                actual: self.version,
            });
        }

        let next_version = StateVersion(
            self.version
                .0
                .checked_add(1)
                .ok_or(StateStoreError::VersionOverflow)?,
        );

        let mut previous = None;
        for update in &plan.transition.account_updates {
            if previous.is_some_and(|id| id >= update.account) {
                return Err(StateStoreError::DuplicateOrUnorderedAccountUpdate);
            }
            previous = Some(update.account);
            let current = self
                .accounts
                .get(&update.account)
                .copied()
                .ok_or(StateStoreError::BeforeStateMismatch(update.account))?;
            if current != update.before {
                return Err(StateStoreError::BeforeStateMismatch(update.account));
            }
        }

        // No writes occur until every precondition has passed.
        for PlannedAccountUpdate { account, after, .. } in &plan.transition.account_updates {
            self.accounts.insert(*account, *after);
        }
        self.version = next_version;
        Ok(next_version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apex_consensus::{TransactionAdmission, TransactionId};
    use apex_types::ChainId;

    fn store() -> InMemoryStateStore {
        InMemoryStateStore::new(
            StateVersion(12),
            BTreeMap::from([
                (
                    AccountId(1),
                    AccountState { nonce: 0, balance: 1_000_000 },
                ),
                (AccountId(2), AccountState { nonce: 0, balance: 500 }),
                (AccountId(3), AccountState { nonce: 0, balance: 700 }),
            ]),
        )
    }

    fn context() -> AdmissionContext {
        AdmissionContext {
            chain_id: ChainId(7),
            protocol_version: 1,
            block_gas_limit: 100_000,
        }
    }

    fn txs() -> [BlockTransaction; 2] {
        [0_u64, 1].map(|nonce| BlockTransaction {
            id: TransactionId(nonce + 1),
            admission: TransactionAdmission {
                chain_id: ChainId(7),
                protocol_version: 1,
                sender: AccountId(1),
                recipient: AccountId(nonce + 2),
                nonce,
                value: if nonce == 0 { 100 } else { 200 },
                gas_limit: 21_000,
                max_fee_per_gas: 1,
                max_priority_fee_per_gas: 0,
            },
        })
    }

    fn planned(s: &InMemoryStateStore) -> VersionedBlockTransitionPlan {
        let snapshot = s.snapshot().unwrap();
        plan_versioned_block_transition(context(), &txs(), &snapshot).unwrap()
    }

    #[test]
    fn snapshot_and_plan_are_bound_to_version() {
        let s = store();
        let snapshot = s.snapshot().unwrap();
        let plan = plan_versioned_block_transition(context(), &txs(), &snapshot).unwrap();
        assert_eq!(snapshot.version(), StateVersion(12));
        assert_eq!(plan.parent_version, StateVersion(12));
        assert_eq!(snapshot.account(AccountId(1)).unwrap().nonce, 0);
    }

    #[test]
    fn applies_all_updates_and_advances_version_once() {
        let mut s = store();
        assert_eq!(s.apply(&planned(&s)), Ok(StateVersion(13)));
        let after = s.snapshot().unwrap();
        assert_eq!(
            after.account(AccountId(1)).unwrap(),
            AccountState { nonce: 2, balance: 957_700 }
        );
        assert_eq!(after.account(AccountId(2)).unwrap().balance, 600);
        assert_eq!(after.account(AccountId(3)).unwrap().balance, 900);
    }

    #[test]
    fn stale_and_replayed_plans_are_rejected_without_mutation() {
        let mut s = store();
        let old = planned(&s);
        let current = planned(&s);
        assert_eq!(s.apply(&current), Ok(StateVersion(13)));
        let before = s.snapshot().unwrap();
        assert_eq!(
            s.apply(&old),
            Err(StateStoreError::StaleVersion {
                expected: StateVersion(12),
                actual: StateVersion(13),
            })
        );
        assert_eq!(s.snapshot().unwrap(), before);
        assert_eq!(
            s.apply(&current),
            Err(StateStoreError::StaleVersion {
                expected: StateVersion(12),
                actual: StateVersion(13),
            })
        );
    }

    #[test]
    fn any_before_state_mismatch_prevents_all_writes() {
        let mut s = store();
        let mut p = planned(&s);
        p.transition.account_updates[1].before.balance += 1;
        let before = s.snapshot().unwrap();
        assert_eq!(
            s.apply(&p),
            Err(StateStoreError::BeforeStateMismatch(AccountId(2)))
        );
        assert_eq!(s.snapshot().unwrap(), before);
    }

    #[test]
    fn unordered_updates_are_rejected_without_mutation() {
        let mut s = store();
        let mut p = planned(&s);
        p.transition.account_updates.swap(0, 1);
        let before = s.snapshot().unwrap();
        assert_eq!(
            s.apply(&p),
            Err(StateStoreError::DuplicateOrUnorderedAccountUpdate)
        );
        assert_eq!(s.snapshot().unwrap(), before);
    }

    #[test]
    fn version_overflow_fails_without_mutation() {
        let mut s = InMemoryStateStore::new(StateVersion(u64::MAX), store().accounts);
        let p = planned(&s);
        let before = s.snapshot().unwrap();
        assert_eq!(s.apply(&p), Err(StateStoreError::VersionOverflow));
        assert_eq!(s.snapshot().unwrap(), before);
    }
}
