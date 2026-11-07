//! APEX-owned EVM integration boundary.
//!
//! The full-width state model is an immutable, version-bound read view. It is
//! not yet wired to revm's Database trait or a persistent protocol state backend.

use std::collections::BTreeMap;
use std::fmt;

use revm::{
    database_interface::{DBErrorMarker, Database},
    primitives::{Address, B256, Bytes, StorageKey, StorageValue, U256},
    state::{AccountInfo, Bytecode},
};

pub const REVM_CANDIDATE_VERSION: &str = "43.0.3";

/// Fixed-width 20-byte EVM account address.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EvmAddress(pub [u8; 20]);

/// Unsigned 256-bit integer in big-endian in-memory form.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct U256Word(pub [u8; 32]);

impl U256Word {
    pub const ZERO: Self = Self([0; 32]);

    pub const fn from_be_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn to_be_bytes(self) -> [u8; 32] {
        self.0
    }

    pub const fn from_u64(value: u64) -> Self {
        let source = value.to_be_bytes();
        let mut bytes = [0_u8; 32];
        let mut i = 0;
        while i < 8 {
            bytes[24 + i] = source[i];
            i += 1;
        }
        Self(bytes)
    }
}

/// 32-byte code-content identifier. Hash construction and verification remain
/// protocol decisions and are not performed by this state-model prototype.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EvmCodeHash(pub [u8; 32]);

/// Accounts in the map are present; absence represents a nonexistent account.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvmAccount {
    pub nonce: u64,
    pub balance: U256Word,
    pub code_hash: EvmCodeHash,
}

/// Caller-supplied bounds. No consensus limits are silently selected here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotLimits {
    pub max_accounts: usize,
    pub max_code_bytes: usize,
    pub max_storage_slots: usize,
    pub max_block_hashes: usize,
}

/// Immutable EVM state view anchored to one internal parent-state version.
/// The version is not a block height, state root or wire-format value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvmStateSnapshot {
    parent_state_version: u64,
    accounts: BTreeMap<EvmAddress, EvmAccount>,
    code: BTreeMap<EvmCodeHash, Vec<u8>>,
    storage: BTreeMap<(EvmAddress, U256Word), U256Word>,
    block_hashes: BTreeMap<u64, [u8; 32]>,
    limits: SnapshotLimits,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvmStateError {
    ResourceLimitExceeded(&'static str),
    CodeMissing(EvmCodeHash),
    StorageOwnerMissing(EvmAddress),
}

impl fmt::Display for EvmStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ResourceLimitExceeded(resource) => {
                write!(f, "EVM snapshot exceeds configured {resource} limit")
            }
            Self::CodeMissing(hash) => {
                write!(f, "account references unavailable code hash {hash:?}")
            }
            Self::StorageOwnerMissing(address) => {
                write!(f, "storage is attached to absent account {address:?}")
            }
        }
    }
}

impl std::error::Error for EvmStateError {}

impl EvmStateSnapshot {
    /// Validates and freezes one complete read snapshot.
    ///
    /// Code hashes are checked for referential presence, not recomputed. The
    /// selected fork's hash function and canonical code rules remain open.
    pub fn try_new(
        parent_state_version: u64,
        accounts: BTreeMap<EvmAddress, EvmAccount>,
        code: BTreeMap<EvmCodeHash, Vec<u8>>,
        storage: BTreeMap<(EvmAddress, U256Word), U256Word>,
        block_hashes: BTreeMap<u64, [u8; 32]>,
        limits: SnapshotLimits,
    ) -> Result<Self, EvmStateError> {
        if accounts.len() > limits.max_accounts {
            return Err(EvmStateError::ResourceLimitExceeded("account"));
        }
        if storage.len() > limits.max_storage_slots {
            return Err(EvmStateError::ResourceLimitExceeded("storage-slot"));
        }
        if block_hashes.len() > limits.max_block_hashes {
            return Err(EvmStateError::ResourceLimitExceeded("block-hash"));
        }

        let total_code_bytes = code
            .values()
            .try_fold(0_usize, |total, bytes| total.checked_add(bytes.len()));
        if total_code_bytes.is_none_or(|total| total > limits.max_code_bytes) {
            return Err(EvmStateError::ResourceLimitExceeded("code-byte"));
        }

        for account in accounts.values() {
            if !code.contains_key(&account.code_hash) {
                return Err(EvmStateError::CodeMissing(account.code_hash));
            }
        }
        for key in storage.keys() {
            if !accounts.contains_key(&key.0) {
                return Err(EvmStateError::StorageOwnerMissing(key.0));
            }
        }

        Ok(Self {
            parent_state_version,
            accounts,
            code,
            storage,
            block_hashes,
            limits,
        })
    }

    pub const fn parent_state_version(&self) -> u64 {
        self.parent_state_version
    }

    pub fn database_view(&self) -> EvmDatabaseView<'_> {
        EvmDatabaseView { snapshot: self }
    }
}

/// Read-only APEX database boundary over one immutable snapshot.
///
/// This does not yet implement revm's Database trait. Engine-facing
/// conversions are deferred until the exact adapter and fork semantics are
/// reviewed; the underlying state representation remains engine-independent.
#[derive(Clone, Copy, Debug)]
pub struct EvmDatabaseView<'a> {
    snapshot: &'a EvmStateSnapshot,
}

impl<'a> EvmDatabaseView<'a> {
    pub const fn parent_state_version(&self) -> u64 {
        self.snapshot.parent_state_version
    }

    pub fn basic(&self, address: EvmAddress) -> Option<&'a EvmAccount> {
        self.snapshot.accounts.get(&address)
    }

    pub fn code_by_hash(&self, code_hash: EvmCodeHash) -> Option<&'a [u8]> {
        self.snapshot.code.get(&code_hash).map(Vec::as_slice)
    }

    /// An absent slot, including one under an absent account, reads as zero.
    pub fn storage(&self, address: EvmAddress, key: U256Word) -> U256Word {
        self.snapshot
            .storage
            .get(&(address, key))
            .copied()
            .unwrap_or(U256Word::ZERO)
    }

    pub fn block_hash(&self, number: u64) -> Option<&'a [u8; 32]> {
        self.snapshot.block_hashes.get(&number)
    }
}

/// Error surfaced by the revm database adapter. Missing or malformed code is
/// fatal to execution; it is never silently replaced with empty bytecode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RevmDatabaseError {
    CodeMissing(EvmCodeHash),
    InvalidBytecode(String),
}

impl fmt::Display for RevmDatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CodeMissing(hash) => write!(f, "database code hash not found: {hash:?}"),
            Self::InvalidBytecode(message) => write!(f, "database bytecode is invalid: {message}"),
        }
    }
}

impl std::error::Error for RevmDatabaseError {}
impl DBErrorMarker for RevmDatabaseError {}

impl Database for EvmDatabaseView<'_> {
    type Error = RevmDatabaseError;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        let mut address_bytes = [0_u8; 20];
        address_bytes.copy_from_slice(address.as_slice());
        Ok(
            EvmDatabaseView::basic(self, EvmAddress(address_bytes)).map(|account| AccountInfo {
                balance: U256::from_be_bytes(account.balance.0),
                nonce: account.nonce,
                code_hash: B256::from(account.code_hash.0),
                account_id: None,
                code: None,
            }),
        )
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        let mut hash_bytes = [0_u8; 32];
        hash_bytes.copy_from_slice(code_hash.as_slice());
        let hash = EvmCodeHash(hash_bytes);
        let code = EvmDatabaseView::code_by_hash(self, hash)
            .ok_or(RevmDatabaseError::CodeMissing(hash))?;
        Bytecode::new_raw_checked(Bytes::copy_from_slice(code))
            .map_err(|error| RevmDatabaseError::InvalidBytecode(error.to_string()))
    }

    fn storage(
        &mut self,
        address: Address,
        index: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        let mut address_bytes = [0_u8; 20];
        address_bytes.copy_from_slice(address.as_slice());
        let key = U256Word(index.to_be_bytes::<32>());
        let value = EvmDatabaseView::storage(self, EvmAddress(address_bytes), key);
        Ok(U256::from_be_bytes(value.0))
    }

    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        Ok(self
            .snapshot
            .block_hashes
            .get(&number)
            .copied()
            .map(B256::from)
            .unwrap_or(B256::ZERO))
    }
}


/// One account replacement in a deterministic state delta. Updates are
/// ordered by address and carry the expected prior value for conflict checks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvmAccountUpdate {
    pub address: EvmAddress,
    pub before: Option<EvmAccount>,
    pub after: Option<EvmAccount>,
}

/// One code-blob replacement in a deterministic state delta.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvmCodeUpdate {
    pub code_hash: EvmCodeHash,
    pub before: Option<Vec<u8>>,
    pub after: Option<Vec<u8>>,
}

/// One storage-slot replacement in a deterministic state delta.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvmStorageUpdate {
    pub address: EvmAddress,
    pub key: U256Word,
    pub before: Option<U256Word>,
    pub after: Option<U256Word>,
}

/// APEX-owned state changes produced by one execution against a single parent
/// snapshot. Each update list must be strictly increasing by its full key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvmStateDelta {
    pub parent_state_version: u64,
    pub account_updates: Vec<EvmAccountUpdate>,
    pub code_updates: Vec<EvmCodeUpdate>,
    pub storage_updates: Vec<EvmStorageUpdate>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvmStateApplyError {
    StaleParent { expected: u64, actual: u64 },
    VersionOverflow,
    UnorderedAccountUpdates,
    UnorderedCodeUpdates,
    UnorderedStorageUpdates,
    AccountBeforeMismatch(EvmAddress),
    CodeBeforeMismatch(EvmCodeHash),
    StorageBeforeMismatch(EvmAddress, U256Word),
    InvalidResultingSnapshot(EvmStateError),
}

impl fmt::Display for EvmStateApplyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StaleParent { expected, actual } => {
                write!(f, "EVM delta parent version {expected} is stale; current version is {actual}")
            }
            Self::VersionOverflow => write!(f, "EVM state version cannot be incremented"),
            Self::UnorderedAccountUpdates => write!(f, "account updates must be strictly ordered"),
            Self::UnorderedCodeUpdates => write!(f, "code updates must be strictly ordered"),
            Self::UnorderedStorageUpdates => write!(f, "storage updates must be strictly ordered"),
            Self::AccountBeforeMismatch(address) => {
                write!(f, "account before-state mismatch at {address:?}")
            }
            Self::CodeBeforeMismatch(hash) => {
                write!(f, "code before-state mismatch at {hash:?}")
            }
            Self::StorageBeforeMismatch(address, key) => {
                write!(f, "storage before-state mismatch at {address:?} slot {key:?}")
            }
            Self::InvalidResultingSnapshot(error) => {
                write!(f, "resulting EVM snapshot is invalid: {error}")
            }
        }
    }
}

impl std::error::Error for EvmStateApplyError {}

/// Process-local reference store for atomic full-width EVM state application.
/// It is intentionally not persistent, concurrent or crash-safe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvmStateStore {
    snapshot: EvmStateSnapshot,
}

impl EvmStateStore {
    pub fn new(snapshot: EvmStateSnapshot) -> Self {
        Self { snapshot }
    }

    pub fn snapshot(&self) -> &EvmStateSnapshot {
        &self.snapshot
    }

    /// Validates the parent version, ordering and every before-state before
    /// constructing a replacement snapshot. The store is changed only after
    /// all updates and resulting snapshot invariants have passed.
    pub fn apply(&mut self, delta: &EvmStateDelta) -> Result<u64, EvmStateApplyError> {
        let actual_version = self.snapshot.parent_state_version;
        if delta.parent_state_version != actual_version {
            return Err(EvmStateApplyError::StaleParent {
                expected: delta.parent_state_version,
                actual: actual_version,
            });
        }
        let next_version = actual_version
            .checked_add(1)
            .ok_or(EvmStateApplyError::VersionOverflow)?;

        let mut previous_account = None;
        for update in &delta.account_updates {
            if previous_account.is_some_and(|address| address >= update.address) {
                return Err(EvmStateApplyError::UnorderedAccountUpdates);
            }
            previous_account = Some(update.address);
            if self.snapshot.accounts.get(&update.address).cloned() != update.before {
                return Err(EvmStateApplyError::AccountBeforeMismatch(update.address));
            }
        }

        let mut previous_code = None;
        for update in &delta.code_updates {
            if previous_code.is_some_and(|hash| hash >= update.code_hash) {
                return Err(EvmStateApplyError::UnorderedCodeUpdates);
            }
            previous_code = Some(update.code_hash);
            if self.snapshot.code.get(&update.code_hash).cloned() != update.before {
                return Err(EvmStateApplyError::CodeBeforeMismatch(update.code_hash));
            }
        }

        let mut previous_storage = None;
        for update in &delta.storage_updates {
            let key = (update.address, update.key);
            if previous_storage.is_some_and(|previous| previous >= key) {
                return Err(EvmStateApplyError::UnorderedStorageUpdates);
            }
            previous_storage = Some(key);
            if self.snapshot.storage.get(&key).copied() != update.before {
                return Err(EvmStateApplyError::StorageBeforeMismatch(update.address, update.key));
            }
        }

        let mut accounts = self.snapshot.accounts.clone();
        let mut code = self.snapshot.code.clone();
        let mut storage = self.snapshot.storage.clone();
        for update in &delta.account_updates {
            match &update.after {
                Some(account) => {
                    accounts.insert(update.address, account.clone());
                }
                None => {
                    accounts.remove(&update.address);
                }
            }
        }
        for update in &delta.code_updates {
            match &update.after {
                Some(bytes) => {
                    code.insert(update.code_hash, bytes.clone());
                }
                None => {
                    code.remove(&update.code_hash);
                }
            }
        }
        for update in &delta.storage_updates {
            let key = (update.address, update.key);
            match update.after {
                Some(value) => {
                    storage.insert(key, value);
                }
                None => {
                    storage.remove(&key);
                }
            }
        }

        let next = EvmStateSnapshot::try_new(
            next_version,
            accounts,
            code,
            storage,
            self.snapshot.block_hashes.clone(),
            self.snapshot.limits,
        )
        .map_err(EvmStateApplyError::InvalidResultingSnapshot)?;

        self.snapshot = next;
        Ok(next_version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIMITS: SnapshotLimits = SnapshotLimits {
        max_accounts: 8,
        max_code_bytes: 128,
        max_storage_slots: 16,
        max_block_hashes: 256,
    };

    fn address(last: u8) -> EvmAddress {
        let mut bytes = [0_u8; 20];
        bytes[19] = last;
        EvmAddress(bytes)
    }

    fn snapshot() -> EvmStateSnapshot {
        let code_hash = EvmCodeHash([7; 32]);
        EvmStateSnapshot::try_new(
            42,
            BTreeMap::from([(
                address(1),
                EvmAccount {
                    nonce: 9,
                    balance: U256Word([0x80; 32]),
                    code_hash,
                },
            )]),
            BTreeMap::from([(code_hash, vec![0x60, 0x00])]),
            BTreeMap::from([((address(1), U256Word::from_u64(3)), U256Word([0x55; 32]))]),
            BTreeMap::from([(41, [0x41; 32])]),
            LIMITS,
        )
        .unwrap()
    }

    #[test]
    fn revm_database_trait_reads_full_width_snapshot_values() {
        let snapshot = snapshot();
        let mut database = snapshot.database_view();
        let evm_address = Address::from_slice(&address(1).0);

        let account = Database::basic(&mut database, evm_address)
            .unwrap()
            .expect("present account");
        assert_eq!(account.nonce, 9);
        assert_eq!(account.balance, U256::from_be_bytes([0x80; 32]));
        assert_eq!(account.code_hash, B256::from([7; 32]));
        assert!(Database::code_by_hash(&mut database, account.code_hash).is_ok());
        assert_eq!(
            Database::storage(&mut database, evm_address, U256::from(3_u8)).unwrap(),
            U256::from_be_bytes([0x55; 32])
        );
        assert_eq!(
            Database::block_hash(&mut database, 41).unwrap(),
            B256::from([0x41; 32])
        );
        assert_eq!(
            Database::block_hash(&mut database, 999).unwrap(),
            B256::ZERO
        );
        assert!(
            Database::basic(&mut database, Address::from_slice(&address(2).0))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn revm_database_rejects_missing_and_malformed_code() {
        let hash = EvmCodeHash([8; 32]);
        let snapshot = EvmStateSnapshot::try_new(
            3,
            BTreeMap::new(),
            BTreeMap::from([(hash, vec![0xef, 0x01])]),
            BTreeMap::new(),
            BTreeMap::new(),
            LIMITS,
        )
        .unwrap();
        let mut database = snapshot.database_view();
        assert!(matches!(
            Database::code_by_hash(&mut database, B256::from(hash.0)),
            Err(RevmDatabaseError::InvalidBytecode(_))
        ));
        assert!(matches!(
            Database::code_by_hash(&mut database, B256::from([9; 32])),
            Err(RevmDatabaseError::CodeMissing(EvmCodeHash(bytes))) if bytes == [9; 32]
        ));
    }



    fn empty_delta(parent_state_version: u64) -> EvmStateDelta {
        EvmStateDelta {
            parent_state_version,
            account_updates: Vec::new(),
            code_updates: Vec::new(),
            storage_updates: Vec::new(),
        }
    }

    #[test]
    fn state_delta_applies_full_width_updates_atomically() {
        let mut store = EvmStateStore::new(snapshot());
        let before = store.snapshot().accounts.get(&address(1)).unwrap().clone();
        let mut after = before.clone();
        after.balance = U256Word([0x81; 32]);
        let delta = EvmStateDelta {
            parent_state_version: 42,
            account_updates: vec![EvmAccountUpdate {
                address: address(1),
                before: Some(before),
                after: Some(after),
            }],
            code_updates: Vec::new(),
            storage_updates: vec![EvmStorageUpdate {
                address: address(1),
                key: U256Word::from_u64(3),
                before: Some(U256Word([0x55; 32])),
                after: Some(U256Word([0x66; 32])),
            }],
        };

        assert_eq!(store.apply(&delta), Ok(43));
        let view = store.snapshot().database_view();
        assert_eq!(view.parent_state_version(), 43);
        assert_eq!(view.basic(address(1)).unwrap().balance, U256Word([0x81; 32]));
        assert_eq!(
            view.storage(address(1), U256Word::from_u64(3)),
            U256Word([0x66; 32])
        );
    }

    #[test]
    fn stale_delta_and_replay_are_rejected_without_mutation() {
        let mut store = EvmStateStore::new(snapshot());
        let delta = empty_delta(42);
        assert_eq!(store.apply(&delta), Ok(43));
        let before = store.snapshot().clone();
        assert_eq!(
            store.apply(&delta),
            Err(EvmStateApplyError::StaleParent {
                expected: 42,
                actual: 43
            })
        );
        assert_eq!(store.snapshot(), &before);
    }

    #[test]
    fn late_before_state_mismatch_leaves_all_domains_unchanged() {
        let mut store = EvmStateStore::new(snapshot());
        let account_before = store.snapshot().accounts.get(&address(1)).unwrap().clone();
        let mut account_after = account_before.clone();
        account_after.balance = U256Word::from_u64(123);
        let before = store.snapshot().clone();
        let delta = EvmStateDelta {
            parent_state_version: 42,
            account_updates: vec![EvmAccountUpdate {
                address: address(1),
                before: Some(account_before),
                after: Some(account_after),
            }],
            code_updates: Vec::new(),
            storage_updates: vec![EvmStorageUpdate {
                address: address(1),
                key: U256Word::from_u64(3),
                before: Some(U256Word::from_u64(999)),
                after: None,
            }],
        };
        assert!(matches!(
            store.apply(&delta),
            Err(EvmStateApplyError::StorageBeforeMismatch(_, _))
        ));
        assert_eq!(store.snapshot(), &before);
    }

    #[test]
    fn removing_referenced_code_is_rejected_atomically() {
        let mut store = EvmStateStore::new(snapshot());
        let hash = EvmCodeHash([7; 32]);
        let before = store.snapshot().clone();
        let delta = EvmStateDelta {
            parent_state_version: 42,
            account_updates: Vec::new(),
            code_updates: vec![EvmCodeUpdate {
                code_hash: hash,
                before: Some(vec![0x60, 0x00]),
                after: None,
            }],
            storage_updates: Vec::new(),
        };
        assert!(matches!(
            store.apply(&delta),
            Err(EvmStateApplyError::InvalidResultingSnapshot(EvmStateError::CodeMissing(_)))
        ));
        assert_eq!(store.snapshot(), &before);
    }

    #[test]
    fn unordered_duplicate_updates_are_rejected() {
        let mut store = EvmStateStore::new(snapshot());
        let account = store.snapshot().accounts.get(&address(1)).unwrap().clone();
        let delta = EvmStateDelta {
            parent_state_version: 42,
            account_updates: vec![
                EvmAccountUpdate {
                    address: address(1),
                    before: Some(account.clone()),
                    after: Some(account.clone()),
                },
                EvmAccountUpdate {
                    address: address(1),
                    before: Some(account.clone()),
                    after: Some(account),
                },
            ],
            code_updates: Vec::new(),
            storage_updates: Vec::new(),
        };
        assert_eq!(
            store.apply(&delta),
            Err(EvmStateApplyError::UnorderedAccountUpdates)
        );
    }

    #[test]
    fn state_version_overflow_does_not_mutate_snapshot() {
        let current = snapshot();
        let max = EvmStateSnapshot {
            parent_state_version: u64::MAX,
            ..current
        };
        let mut store = EvmStateStore::new(max);
        let before = store.snapshot().clone();
        assert_eq!(
            store.apply(&empty_delta(u64::MAX)),
            Err(EvmStateApplyError::VersionOverflow)
        );
        assert_eq!(store.snapshot(), &before);
    }

    #[test]
    fn full_width_account_storage_and_code_are_preserved() {
        let snapshot = snapshot();
        let view = snapshot.database_view();
        let account = view.basic(address(1)).unwrap();
        assert_eq!(account.nonce, 9);
        assert_eq!(account.balance, U256Word([0x80; 32]));
        assert_eq!(account.code_hash, EvmCodeHash([7; 32]));
        assert_eq!(
            view.code_by_hash(account.code_hash),
            Some(&[0x60, 0x00][..])
        );
        assert_eq!(
            view.storage(address(1), U256Word::from_u64(3)),
            U256Word([0x55; 32])
        );
    }

    #[test]
    fn addresses_are_full_width_and_distinct() {
        let mut left = [0_u8; 20];
        left[0] = 0x80;
        let mut right = [0_u8; 20];
        right[19] = 0x80;
        assert_ne!(EvmAddress(left), EvmAddress(right));
    }

    #[test]
    fn absent_account_and_storage_slot_are_not_fabricated() {
        let snapshot = snapshot();
        let view = snapshot.database_view();
        assert!(view.basic(address(2)).is_none());
        assert_eq!(
            view.storage(address(1), U256Word::from_u64(99)),
            U256Word::ZERO
        );
        assert_eq!(
            view.storage(address(2), U256Word::from_u64(3)),
            U256Word::ZERO
        );
    }

    #[test]
    fn snapshot_is_bound_to_one_parent_version() {
        assert_eq!(snapshot().database_view().parent_state_version(), 42);
    }

    #[test]
    fn missing_account_code_is_rejected() {
        let result = EvmStateSnapshot::try_new(
            1,
            BTreeMap::from([(
                address(1),
                EvmAccount {
                    nonce: 0,
                    balance: U256Word::ZERO,
                    code_hash: EvmCodeHash([9; 32]),
                },
            )]),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
            LIMITS,
        );
        assert_eq!(
            result,
            Err(EvmStateError::CodeMissing(EvmCodeHash([9; 32])))
        );
    }

    #[test]
    fn orphaned_storage_is_rejected() {
        let result = EvmStateSnapshot::try_new(
            1,
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([((address(1), U256Word::ZERO), U256Word::from_u64(1))]),
            BTreeMap::new(),
            LIMITS,
        );
        assert_eq!(result, Err(EvmStateError::StorageOwnerMissing(address(1))));
    }

    #[test]
    fn resource_limits_fail_closed() {
        let mut limits = LIMITS;
        limits.max_accounts = 0;
        let result = EvmStateSnapshot::try_new(
            1,
            BTreeMap::from([(
                address(1),
                EvmAccount {
                    nonce: 0,
                    balance: U256Word::ZERO,
                    code_hash: EvmCodeHash([7; 32]),
                },
            )]),
            BTreeMap::from([(EvmCodeHash([7; 32]), vec![])]),
            BTreeMap::new(),
            BTreeMap::new(),
            limits,
        );
        assert_eq!(result, Err(EvmStateError::ResourceLimitExceeded("account")));
    }

    #[test]
    fn code_bytes_are_bounded_with_checked_total() {
        let mut limits = LIMITS;
        limits.max_code_bytes = 1;
        let result = EvmStateSnapshot::try_new(
            1,
            BTreeMap::new(),
            BTreeMap::from([(EvmCodeHash([7; 32]), vec![0x60, 0x00])]),
            BTreeMap::new(),
            BTreeMap::new(),
            limits,
        );
        assert_eq!(
            result,
            Err(EvmStateError::ResourceLimitExceeded("code-byte"))
        );
    }
}

#[cfg(test)]
mod revm_harness_tests {
    use revm::{
        ExecuteCommitEvm, ExecuteEvm, MainBuilder, MainContext,
        bytecode::opcode,
        context::{Context, TxEnv},
        context_interface::result::{ExecutionResult, Output},
        database::CacheDB,
        database_interface::EmptyDB,
        primitives::{Bytes, StorageValue, TxKind, U256},
    };

    const INIT_CODE: &[u8] = &[
        opcode::PUSH1,
        0x01,
        opcode::PUSH1,
        0x17,
        opcode::PUSH1,
        0x1f,
        opcode::CODECOPY,
        opcode::PUSH0,
        opcode::MLOAD,
        opcode::PUSH0,
        opcode::SSTORE,
    ];
    const RETURN_RUNTIME: &[u8] = &[
        opcode::PUSH1,
        0x02,
        opcode::PUSH1,
        0x15,
        opcode::PUSH0,
        opcode::CODECOPY,
        opcode::PUSH1,
        0x02,
        opcode::PUSH0,
        opcode::RETURN,
    ];
    const RUNTIME_BYTECODE: &[u8] = &[opcode::PUSH0, opcode::SLOAD];

    fn execute_storage_fixture() -> U256 {
        let init_code: Bytes = [INIT_CODE, RETURN_RUNTIME, RUNTIME_BYTECODE, &[0x42]]
            .concat()
            .into();
        let context = Context::mainnet().with_db(CacheDB::<EmptyDB>::default());
        let mut evm = context.build_mainnet();
        let deployment = evm
            .transact_commit(
                TxEnv::builder()
                    .kind(TxKind::Create)
                    .data(init_code)
                    .build()
                    .expect("valid fixture deployment transaction"),
            )
            .expect("fixture deployment executes");
        let ExecutionResult::Success {
            output: Output::Create(_, Some(contract_address)),
            ..
        } = deployment
        else {
            panic!("fixture contract deployment did not succeed: {deployment:#?}");
        };
        let call = evm
            .transact(
                TxEnv::builder()
                    .kind(TxKind::Call(contract_address))
                    .data(Bytes::new())
                    .nonce(1)
                    .build()
                    .expect("valid fixture call transaction"),
            )
            .expect("fixture call executes");
        assert!(matches!(call.result, ExecutionResult::Success { .. }));
        call.state
            .get(&contract_address)
            .expect("contract state is present in execution result")
            .storage
            .get::<StorageValue>(&U256::ZERO)
            .expect("slot zero is included in execution state diff")
            .present_value()
    }

    #[test]
    fn revm_deploys_contract_and_executes_storage_read() {
        assert_eq!(execute_storage_fixture(), U256::from(0x42_u8));
    }

    #[test]
    fn identical_fresh_contexts_produce_identical_execution_output() {
        let first = execute_storage_fixture();
        let second = execute_storage_fixture();
        assert_eq!(first, second);
        assert_eq!(first, U256::from(0x42_u8));
    }
}
