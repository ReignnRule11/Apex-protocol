//! Conversion of revm's execution state into APEX-owned, parent-bound deltas.
//!
//! This adapter deliberately rejects fork-sensitive account deletion until an
//! explicit, ratified fork context is supplied by the block pipeline.

use std::collections::BTreeMap;
use std::fmt;

use revm::state::EvmState;

use crate::{
    EvmAccount, EvmAccountUpdate, EvmAddress, EvmCodeHash, EvmCodeUpdate, EvmStateDelta,
    EvmStateSnapshot, EvmStorageUpdate, U256Word,
};

/// Fail-closed errors while translating revm state into an APEX delta.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RevmStateDiffError {
    /// A changed account refers to code that revm did not provide.
    ChangedCodeUnavailable(EvmAddress, EvmCodeHash),
    /// Supplied bytecode does not hash to the account's declared code hash.
    CodeHashMismatch(EvmAddress, EvmCodeHash),
    /// The same content hash was associated with different bytes.
    CodeHashCollision(EvmCodeHash),
    /// revm's original storage value disagrees with the selected parent view.
    StorageParentMismatch(EvmAddress, U256Word),
    /// Account deletion semantics require a fork-specific rule not yet ratified.
    ForkSensitiveAccountDeletion(EvmAddress),
}

impl fmt::Display for RevmStateDiffError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChangedCodeUnavailable(address, hash) => {
                write!(f, "changed code unavailable for {address:?} ({hash:?})")
            }
            Self::CodeHashMismatch(address, hash) => {
                write!(f, "bytecode hash mismatch for {address:?} ({hash:?})")
            }
            Self::CodeHashCollision(hash) => {
                write!(
                    f,
                    "different bytecode supplied for existing code hash {hash:?}"
                )
            }
            Self::StorageParentMismatch(address, key) => {
                write!(
                    f,
                    "revm original storage disagrees with parent at {address:?} slot {key:?}"
                )
            }
            Self::ForkSensitiveAccountDeletion(address) => {
                write!(
                    f,
                    "account deletion for {address:?} requires a ratified fork context"
                )
            }
        }
    }
}

impl std::error::Error for RevmStateDiffError {}

/// Extract a deterministic APEX delta from a successful revm state result.
///
/// Only changed account information and changed storage slots are emitted.
/// The supplied snapshot is the exact immutable parent used by revm; the
/// returned delta carries its StateVersion and must still pass
/// `EvmStateStore::apply` or the persistent adapter's equivalent checks.
///
/// This function does not settle fees, create receipts, select a fork, or
/// authorize a transaction. Reverted/failed execution results must not be
/// passed here as committed state.
pub fn extract_revm_state_delta(
    parent: &EvmStateSnapshot,
    state: &EvmState,
) -> Result<EvmStateDelta, RevmStateDiffError> {
    let mut account_updates = BTreeMap::new();
    let mut code_updates = BTreeMap::new();
    let mut storage_updates = BTreeMap::new();

    for (revm_address, account) in state {
        let mut address_bytes = [0_u8; 20];
        address_bytes.copy_from_slice(revm_address.as_slice());
        let address = EvmAddress(address_bytes);

        if account.is_selfdestructed() {
            return Err(RevmStateDiffError::ForkSensitiveAccountDeletion(address));
        }

        if account.is_changed() {
            let mut hash_bytes = [0_u8; 32];
            hash_bytes.copy_from_slice(account.info.code_hash.as_slice());
            let code_hash = EvmCodeHash(hash_bytes);
            let before = parent.accounts.get(&address).cloned();

            if let Some(bytecode) = account.info.code.as_ref() {
                let bytes = bytecode.original_bytes().to_vec();
                if bytecode.hash_slow().as_slice() != code_hash.0 {
                    return Err(RevmStateDiffError::CodeHashMismatch(address, code_hash));
                }
                match parent.code.get(&code_hash) {
                    Some(existing) if existing != &bytes => {
                        return Err(RevmStateDiffError::CodeHashCollision(code_hash));
                    }
                    Some(_) => {}
                    None => {
                        code_updates.insert(
                            code_hash,
                            EvmCodeUpdate {
                                code_hash,
                                before: None,
                                after: Some(bytes),
                            },
                        );
                    }
                }
            } else if before.as_ref().map(|value| value.code_hash) != Some(code_hash) {
                return Err(RevmStateDiffError::ChangedCodeUnavailable(
                    address, code_hash,
                ));
            }

            let after = EvmAccount {
                nonce: account.info.nonce,
                balance: U256Word(account.info.balance.to_be_bytes::<32>()),
                code_hash,
            };
            account_updates.insert(
                address,
                EvmAccountUpdate {
                    address,
                    before,
                    after: Some(after),
                },
            );
        }

        for (revm_key, slot) in account.changed_storage_slots() {
            let key = U256Word(revm_key.to_be_bytes::<32>());
            let parent_key = (address, key);
            let parent_value = parent.storage.get(&parent_key).copied();
            let parent_word = parent_value.unwrap_or(U256Word::ZERO);
            let original = U256Word(slot.original_value().to_be_bytes::<32>());
            if original != parent_word {
                return Err(RevmStateDiffError::StorageParentMismatch(address, key));
            }

            let present = U256Word(slot.present_value().to_be_bytes::<32>());
            storage_updates.insert(
                parent_key,
                EvmStorageUpdate {
                    address,
                    key,
                    before: parent_value,
                    after: if present == U256Word::ZERO {
                        None
                    } else {
                        Some(present)
                    },
                },
            );
        }
    }

    Ok(EvmStateDelta {
        parent_state_version: parent.parent_state_version(),
        account_updates: account_updates.into_values().collect(),
        code_updates: code_updates.into_values().collect(),
        storage_updates: storage_updates.into_values().collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EvmStateStore, SnapshotLimits};
    use revm::{
        primitives::{Address, U256},
        state::{Account, AccountInfo, Bytecode, EvmStorageSlot, TransactionId},
    };

    const LIMITS: SnapshotLimits = SnapshotLimits {
        max_accounts: 8,
        max_code_bytes: 128,
        max_storage_slots: 16,
        max_block_hashes: 16,
    };

    fn address() -> (Address, EvmAddress) {
        let mut bytes = [0_u8; 20];
        bytes[19] = 7;
        (Address::from_slice(&bytes), EvmAddress(bytes))
    }

    fn parent_and_state() -> (EvmStateSnapshot, EvmState) {
        let (revm_address, address) = address();
        let bytecode = Bytecode::default();
        let hash = EvmCodeHash(bytecode.hash_slow().0);
        let code = bytecode.original_bytes().to_vec();
        let parent = EvmStateSnapshot::try_new(
            17,
            BTreeMap::from([(
                address,
                EvmAccount {
                    nonce: 1,
                    balance: U256Word::from_u64(100),
                    code_hash: hash,
                },
            )]),
            BTreeMap::from([(hash, code)]),
            BTreeMap::new(),
            BTreeMap::new(),
            LIMITS,
        )
        .unwrap();

        let info = AccountInfo::new(U256::from(100_u64), 1, bytecode.hash_slow(), bytecode);
        let mut account = Account::from(info).with_touched_mark();
        account.storage.insert(
            U256::ZERO,
            EvmStorageSlot::new_changed(U256::ZERO, U256::from(0x42_u8), TransactionId::ZERO),
        );
        let mut state = EvmState::default();
        state.insert(revm_address, account);
        (parent, state)
    }

    #[test]
    fn extracts_revm_storage_change_and_applies_parent_bound_delta() {
        let (parent, state) = parent_and_state();
        let delta = extract_revm_state_delta(&parent, &state).unwrap();
        assert_eq!(delta.parent_state_version, 17);
        assert!(delta.account_updates.is_empty());
        assert_eq!(delta.storage_updates.len(), 1);

        let mut store = EvmStateStore::new(parent);
        assert_eq!(store.apply(&delta), Ok(18));
        assert_eq!(
            store
                .snapshot()
                .database_view()
                .storage(address().1, U256Word::ZERO),
            U256Word::from_u64(0x42)
        );
    }

    #[test]
    fn rejects_storage_diff_from_a_different_parent() {
        let (parent, mut state) = parent_and_state();
        let (revm_address, _) = address();
        let account = state.get_mut(&revm_address).unwrap();
        account.storage.insert(
            U256::ZERO,
            EvmStorageSlot::new_changed(U256::from(1_u8), U256::from(0x42_u8), TransactionId::ZERO),
        );
        assert!(matches!(
            extract_revm_state_delta(&parent, &state),
            Err(RevmStateDiffError::StorageParentMismatch(_, _))
        ));
    }

    #[test]
    fn rejects_selfdestruct_until_fork_semantics_are_explicit() {
        let (parent, mut state) = parent_and_state();
        let (revm_address, address) = address();
        state.get_mut(&revm_address).unwrap().mark_selfdestruct();
        assert_eq!(
            extract_revm_state_delta(&parent, &state),
            Err(RevmStateDiffError::ForkSensitiveAccountDeletion(address))
        );
    }
}
