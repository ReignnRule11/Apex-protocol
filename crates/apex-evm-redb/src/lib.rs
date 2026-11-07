//! Durable full-width EVM state adapter backed by redb.
//!
//! This crate is a persistence prototype, not a canonical protocol encoding,
//! state commitment, or production database selection. Records are versioned
//! and append-only: one redb write transaction publishes all records for a
//! new snapshot and advances the current-version pointer atomically.

use std::{collections::BTreeMap, fmt, path::Path};

use apex_evm::{
    EvmAccount, EvmAddress, EvmCodeHash, EvmStateApplyError, EvmStateDelta, EvmStateSnapshot,
    EvmStateStore, SnapshotLimits, U256Word,
};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};

const SCHEMA_VERSION: u64 = 1;
const META_SCHEMA_KEY: u8 = 0;
const META_CURRENT_VERSION_KEY: u8 = 1;
const META: TableDefinition<u8, u64> = TableDefinition::new("apex_evm_metadata_v1");
const ACCOUNTS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("apex_evm_accounts_v1");
const CODE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("apex_evm_code_v1");
const STORAGE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("apex_evm_storage_v1");
const BLOCK_HASHES: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("apex_evm_block_hashes_v1");

const ACCOUNT_KEY_LEN: usize = 8 + 20;
const CODE_KEY_LEN: usize = 8 + 32;
const STORAGE_KEY_LEN: usize = 8 + 20 + 32;
const BLOCK_HASH_KEY_LEN: usize = 8 + 8;
const ACCOUNT_VALUE_LEN: usize = 8 + 32 + 32;

/// Errors are explicit and fail closed. A commit error requires reopening the
/// database because the caller cannot safely infer whether it committed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PersistentEvmStateError {
    Backend(String),
    UnsupportedSchema(u64),
    CorruptMetadata,
    CorruptKey(&'static str),
    CorruptRecord(&'static str),
    State(String),
    Apply(EvmStateApplyError),
    ReopenRequired,
}

impl fmt::Display for PersistentEvmStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Backend(message) => write!(f, "redb backend error: {message}"),
            Self::UnsupportedSchema(version) => {
                write!(f, "unsupported EVM state schema version {version}")
            }
            Self::CorruptMetadata => write!(f, "EVM state metadata is incomplete or corrupt"),
            Self::CorruptKey(domain) => write!(f, "malformed {domain} storage key"),
            Self::CorruptRecord(domain) => write!(f, "malformed {domain} storage record"),
            Self::State(message) => write!(f, "invalid EVM state snapshot: {message}"),
            Self::Apply(error) => write!(f, "EVM state delta rejected: {error}"),
            Self::ReopenRequired => {
                write!(
                    f,
                    "store requires close and reopen after an ambiguous commit error"
                )
            }
        }
    }
}

impl std::error::Error for PersistentEvmStateError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Lifecycle {
    Ready,
    ReopenRequired,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CommitFault {
    BeforeCommit,
    AcknowledgementLost,
}

/// Single-process redb persistence for complete full-width EVM snapshots.
///
/// Every applied delta writes a complete versioned snapshot and updates the
/// current-version pointer in one ACID transaction. Historical versions are
/// retained; compaction and garbage collection are intentionally deferred.
/// The adapter relies on redb's default exclusive-file lock and Immediate
/// durability. It is not a multi-process writer or consensus commit protocol.
pub struct RedbEvmStateStore {
    db: Database,
    state: EvmStateStore,
    lifecycle: Lifecycle,
    #[cfg(test)]
    commit_fault: Option<CommitFault>,
}

impl RedbEvmStateStore {
    /// Open an existing database or initialize it from the supplied genesis
    /// snapshot. Existing state is never overwritten by the initial snapshot.
    /// Resource limits are supplied by the caller and are not protocol values.
    pub fn open_or_initialize(
        path: impl AsRef<Path>,
        initial: EvmStateSnapshot,
        limits: SnapshotLimits,
    ) -> Result<Self, PersistentEvmStateError> {
        let db = Database::create(path).map_err(backend)?;
        let write = db.begin_write().map_err(backend)?;
        let metadata = {
            let table = write.open_table(META).map_err(backend)?;
            let schema = table
                .get(META_SCHEMA_KEY)
                .map_err(backend)?
                .map(|v| v.value());
            let version = table
                .get(META_CURRENT_VERSION_KEY)
                .map_err(backend)?
                .map(|v| v.value());
            match (schema, version) {
                (Some(schema), Some(version)) => {
                    if schema != SCHEMA_VERSION {
                        return Err(PersistentEvmStateError::UnsupportedSchema(schema));
                    }
                    Some(version)
                }
                (None, None) => None,
                _ => return Err(PersistentEvmStateError::CorruptMetadata),
            }
        };

        if metadata.is_none() {
            if initial.limits() != limits {
                return Err(PersistentEvmStateError::State(
                    "initial snapshot limits differ from supplied limits".into(),
                ));
            }
            write_snapshot_records(&write, &initial)?;
            {
                let mut table = write.open_table(META).map_err(backend)?;
                table
                    .insert(META_SCHEMA_KEY, SCHEMA_VERSION)
                    .map_err(backend)?;
                table
                    .insert(META_CURRENT_VERSION_KEY, initial.parent_state_version())
                    .map_err(backend)?;
            }
        }
        write.commit().map_err(backend)?;

        let snapshot = load_current_snapshot(&db, limits)?;
        Ok(Self {
            db,
            state: EvmStateStore::new(snapshot),
            lifecycle: Lifecycle::Ready,
            #[cfg(test)]
            commit_fault: None,
        })
    }

    pub fn snapshot(&self) -> Result<&EvmStateSnapshot, PersistentEvmStateError> {
        self.ensure_ready()?;
        Ok(self.state.snapshot())
    }

    /// Apply a delta and persist the complete replacement snapshot atomically.
    ///
    /// The in-memory state is advanced only after redb acknowledges the commit.
    /// If commit returns an error, the instance is latched and must be dropped
    /// and reopened to reconcile the durable current-version pointer.
    pub fn apply(&mut self, delta: &EvmStateDelta) -> Result<u64, PersistentEvmStateError> {
        self.ensure_ready()?;
        let mut candidate = self.state.clone();
        let next_version = candidate
            .apply(delta)
            .map_err(PersistentEvmStateError::Apply)?;
        let candidate_snapshot = candidate.snapshot().clone();

        let write = self.db.begin_write().map_err(backend)?;
        write_snapshot_records(&write, &candidate_snapshot)?;
        {
            let mut meta = write.open_table(META).map_err(backend)?;
            meta.insert(META_CURRENT_VERSION_KEY, next_version)
                .map_err(backend)?;
        }

        #[cfg(test)]
        if let Some(fault) = self.commit_fault.take() {
            match fault {
                CommitFault::BeforeCommit => {
                    drop(write);
                    self.lifecycle = Lifecycle::ReopenRequired;
                    return Err(PersistentEvmStateError::Backend(
                        "injected failure before commit".into(),
                    ));
                }
                CommitFault::AcknowledgementLost => {
                    if let Err(error) = write.commit() {
                        self.lifecycle = Lifecycle::ReopenRequired;
                        return Err(backend(error));
                    }
                    self.lifecycle = Lifecycle::ReopenRequired;
                    return Err(PersistentEvmStateError::Backend(
                        "injected lost commit acknowledgement".into(),
                    ));
                }
            }
        }

        if let Err(error) = write.commit() {
            self.lifecycle = Lifecycle::ReopenRequired;
            return Err(backend(error));
        }

        self.state = candidate;
        Ok(next_version)
    }

    /// Run redb's integrity verification. This is a storage-engine check, not
    /// a protocol state-root or semantic correctness proof.
    pub fn check_integrity(&mut self) -> Result<(), PersistentEvmStateError> {
        self.ensure_ready()?;
        self.db.check_integrity().map(|_| ()).map_err(backend)
    }

    fn ensure_ready(&self) -> Result<(), PersistentEvmStateError> {
        if self.lifecycle == Lifecycle::Ready {
            Ok(())
        } else {
            Err(PersistentEvmStateError::ReopenRequired)
        }
    }
}

fn write_snapshot_records(
    write: &redb::WriteTransaction,
    snapshot: &EvmStateSnapshot,
) -> Result<(), PersistentEvmStateError> {
    let version = snapshot.parent_state_version().to_be_bytes();

    {
        let mut table = write.open_table(ACCOUNTS).map_err(backend)?;
        for (address, account) in snapshot.accounts() {
            let mut key = [0_u8; ACCOUNT_KEY_LEN];
            key[..8].copy_from_slice(&version);
            key[8..].copy_from_slice(&address.0);
            let mut value = [0_u8; ACCOUNT_VALUE_LEN];
            value[..8].copy_from_slice(&account.nonce.to_be_bytes());
            value[8..40].copy_from_slice(&account.balance.0);
            value[40..].copy_from_slice(&account.code_hash.0);
            table
                .insert(key.as_slice(), value.as_slice())
                .map_err(backend)?;
        }
    }
    {
        let mut table = write.open_table(CODE).map_err(backend)?;
        for (hash, bytes) in snapshot.code_blobs() {
            let mut key = [0_u8; CODE_KEY_LEN];
            key[..8].copy_from_slice(&version);
            key[8..].copy_from_slice(&hash.0);
            table
                .insert(key.as_slice(), bytes.as_slice())
                .map_err(backend)?;
        }
    }
    {
        let mut table = write.open_table(STORAGE).map_err(backend)?;
        for ((address, slot), value) in snapshot.storage_slots() {
            let mut key = [0_u8; STORAGE_KEY_LEN];
            key[..8].copy_from_slice(&version);
            key[8..28].copy_from_slice(&address.0);
            key[28..].copy_from_slice(&slot.0);
            table
                .insert(key.as_slice(), value.0.as_slice())
                .map_err(backend)?;
        }
    }
    {
        let mut table = write.open_table(BLOCK_HASHES).map_err(backend)?;
        for (number, hash) in snapshot.block_hashes() {
            let mut key = [0_u8; BLOCK_HASH_KEY_LEN];
            key[..8].copy_from_slice(&version);
            key[8..].copy_from_slice(&number.to_be_bytes());
            table
                .insert(key.as_slice(), hash.as_slice())
                .map_err(backend)?;
        }
    }
    Ok(())
}

fn load_current_snapshot(
    db: &Database,
    limits: SnapshotLimits,
) -> Result<EvmStateSnapshot, PersistentEvmStateError> {
    let read = db.begin_read().map_err(backend)?;
    let (schema, version) = {
        let meta = read.open_table(META).map_err(backend)?;
        let schema = meta
            .get(META_SCHEMA_KEY)
            .map_err(backend)?
            .ok_or(PersistentEvmStateError::CorruptMetadata)?
            .value();
        let version = meta
            .get(META_CURRENT_VERSION_KEY)
            .map_err(backend)?
            .ok_or(PersistentEvmStateError::CorruptMetadata)?
            .value();
        (schema, version)
    };
    if schema != SCHEMA_VERSION {
        return Err(PersistentEvmStateError::UnsupportedSchema(schema));
    }
    let prefix = version.to_be_bytes();

    let mut accounts = BTreeMap::new();
    {
        let table = read.open_table(ACCOUNTS).map_err(backend)?;
        for entry in table.iter().map_err(backend)? {
            let (key, value) = entry.map_err(backend)?;
            let key = key.value();
            if key.len() != ACCOUNT_KEY_LEN {
                return Err(PersistentEvmStateError::CorruptKey("account"));
            }
            if key[..8] != prefix {
                continue;
            }
            let bytes = value.value();
            if bytes.len() != ACCOUNT_VALUE_LEN {
                return Err(PersistentEvmStateError::CorruptRecord("account"));
            }
            let mut address = [0_u8; 20];
            address.copy_from_slice(&key[8..]);
            let mut balance = [0_u8; 32];
            balance.copy_from_slice(&bytes[8..40]);
            let mut code_hash = [0_u8; 32];
            code_hash.copy_from_slice(&bytes[40..]);
            accounts.insert(
                EvmAddress(address),
                EvmAccount {
                    nonce: u64::from_be_bytes(
                        bytes[..8]
                            .try_into()
                            .map_err(|_| PersistentEvmStateError::CorruptRecord("account"))?,
                    ),
                    balance: U256Word(balance),
                    code_hash: EvmCodeHash(code_hash),
                },
            );
        }
    }

    let mut code = BTreeMap::new();
    {
        let table = read.open_table(CODE).map_err(backend)?;
        for entry in table.iter().map_err(backend)? {
            let (key, value) = entry.map_err(backend)?;
            let key = key.value();
            if key.len() != CODE_KEY_LEN {
                return Err(PersistentEvmStateError::CorruptKey("code"));
            }
            if key[..8] != prefix {
                continue;
            }
            let mut hash = [0_u8; 32];
            hash.copy_from_slice(&key[8..]);
            code.insert(EvmCodeHash(hash), value.value().to_vec());
        }
    }

    let mut storage = BTreeMap::new();
    {
        let table = read.open_table(STORAGE).map_err(backend)?;
        for entry in table.iter().map_err(backend)? {
            let (key, value) = entry.map_err(backend)?;
            let key = key.value();
            if key.len() != STORAGE_KEY_LEN {
                return Err(PersistentEvmStateError::CorruptKey("storage"));
            }
            if key[..8] != prefix {
                continue;
            }
            let bytes = value.value();
            if bytes.len() != 32 {
                return Err(PersistentEvmStateError::CorruptRecord("storage"));
            }
            let mut address = [0_u8; 20];
            address.copy_from_slice(&key[8..28]);
            let mut slot = [0_u8; 32];
            slot.copy_from_slice(&key[28..]);
            let mut word = [0_u8; 32];
            word.copy_from_slice(bytes);
            storage.insert((EvmAddress(address), U256Word(slot)), U256Word(word));
        }
    }

    let mut block_hashes = BTreeMap::new();
    {
        let table = read.open_table(BLOCK_HASHES).map_err(backend)?;
        for entry in table.iter().map_err(backend)? {
            let (key, value) = entry.map_err(backend)?;
            let key = key.value();
            if key.len() != BLOCK_HASH_KEY_LEN {
                return Err(PersistentEvmStateError::CorruptKey("block-hash"));
            }
            if key[..8] != prefix {
                continue;
            }
            let bytes = value.value();
            if bytes.len() != 32 {
                return Err(PersistentEvmStateError::CorruptRecord("block-hash"));
            }
            let number = u64::from_be_bytes(
                key[8..]
                    .try_into()
                    .map_err(|_| PersistentEvmStateError::CorruptKey("block-hash"))?,
            );
            let mut hash = [0_u8; 32];
            hash.copy_from_slice(bytes);
            block_hashes.insert(number, hash);
        }
    }

    EvmStateSnapshot::try_new(version, accounts, code, storage, block_hashes, limits)
        .map_err(|error| PersistentEvmStateError::State(error.to_string()))
}

fn backend(error: impl fmt::Display) -> PersistentEvmStateError {
    PersistentEvmStateError::Backend(error.to_string())
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

    fn initial_snapshot() -> EvmStateSnapshot {
        let hash = EvmCodeHash([0x77; 32]);
        EvmStateSnapshot::try_new(
            12,
            BTreeMap::from([(
                address(1),
                EvmAccount {
                    nonce: 7,
                    balance: U256Word([0x81; 32]),
                    code_hash: hash,
                },
            )]),
            BTreeMap::from([(hash, vec![0x60, 0x00])]),
            BTreeMap::from([((address(1), U256Word::from_u64(5)), U256Word([0x55; 32]))]),
            BTreeMap::from([(11, [0x11; 32])]),
            LIMITS,
        )
        .unwrap()
    }

    fn empty_delta(version: u64) -> EvmStateDelta {
        EvmStateDelta {
            parent_state_version: version,
            account_updates: Vec::new(),
            code_updates: Vec::new(),
            storage_updates: Vec::new(),
        }
    }

    #[test]
    fn initializes_and_reopens_complete_full_width_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("evm.redb");
        {
            let store =
                RedbEvmStateStore::open_or_initialize(&path, initial_snapshot(), LIMITS).unwrap();
            assert_eq!(store.snapshot().unwrap().parent_state_version(), 12);
        }
        let store =
            RedbEvmStateStore::open_or_initialize(&path, initial_snapshot(), LIMITS).unwrap();
        let snapshot = store.snapshot().unwrap();
        assert_eq!(snapshot.accounts().count(), 1);
        assert_eq!(snapshot.code_blobs().count(), 1);
        assert_eq!(snapshot.storage_slots().count(), 1);
        assert_eq!(snapshot.block_hashes().count(), 1);
        let account = snapshot.accounts().next().unwrap().1;
        assert_eq!(account.balance, U256Word([0x81; 32]));
        assert_eq!(
            snapshot
                .database_view()
                .storage(address(1), U256Word::from_u64(5)),
            U256Word([0x55; 32])
        );
    }

    #[test]
    fn apply_persists_new_version_and_rejects_replay_after_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("evm.redb");
        let mut store =
            RedbEvmStateStore::open_or_initialize(&path, initial_snapshot(), LIMITS).unwrap();
        let delta = empty_delta(12);
        assert_eq!(store.apply(&delta), Ok(13));
        assert!(matches!(
            store.apply(&delta),
            Err(PersistentEvmStateError::Apply(
                EvmStateApplyError::StaleParent { .. }
            ))
        ));
        drop(store);

        let mut reopened =
            RedbEvmStateStore::open_or_initialize(&path, initial_snapshot(), LIMITS).unwrap();
        assert_eq!(reopened.snapshot().unwrap().parent_state_version(), 13);
        assert_eq!(
            reopened.apply(&delta),
            Err(PersistentEvmStateError::Apply(
                EvmStateApplyError::StaleParent {
                    expected: 12,
                    actual: 13
                }
            ))
        );
    }

    #[test]
    fn successful_full_width_delta_persists_all_changed_domains() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("evm.redb");
        let mut store =
            RedbEvmStateStore::open_or_initialize(&path, initial_snapshot(), LIMITS).unwrap();

        let old_hash = EvmCodeHash([0x77; 32]);
        let new_hash = EvmCodeHash([0x78; 32]);
        let before_account = store
            .snapshot()
            .unwrap()
            .accounts()
            .next()
            .unwrap()
            .1
            .clone();
        let mut after_account = before_account.clone();
        after_account.nonce = 8;
        after_account.balance = U256Word([0x91; 32]);
        after_account.code_hash = new_hash;

        let delta = EvmStateDelta {
            parent_state_version: 12,
            account_updates: vec![apex_evm::EvmAccountUpdate {
                address: address(1),
                before: Some(before_account),
                after: Some(after_account),
            }],
            code_updates: vec![
                apex_evm::EvmCodeUpdate {
                    code_hash: old_hash,
                    before: Some(vec![0x60, 0x00]),
                    after: None,
                },
                apex_evm::EvmCodeUpdate {
                    code_hash: new_hash,
                    before: None,
                    after: Some(vec![0x60, 0x01]),
                },
            ],
            storage_updates: vec![apex_evm::EvmStorageUpdate {
                address: address(1),
                key: U256Word::from_u64(5),
                before: Some(U256Word([0x55; 32])),
                after: Some(U256Word([0x66; 32])),
            }],
        };

        assert_eq!(store.apply(&delta), Ok(13));
        drop(store);

        let reopened =
            RedbEvmStateStore::open_or_initialize(&path, initial_snapshot(), LIMITS).unwrap();
        let snapshot = reopened.snapshot().unwrap();
        let account = snapshot.accounts().next().unwrap().1;
        assert_eq!(account.nonce, 8);
        assert_eq!(account.balance, U256Word([0x91; 32]));
        assert_eq!(account.code_hash, new_hash);
        assert_eq!(snapshot.code_blobs().next().unwrap().1, &vec![0x60, 0x01]);
        assert_eq!(
            snapshot
                .database_view()
                .storage(address(1), U256Word::from_u64(5)),
            U256Word([0x66; 32])
        );
    }

    #[test]
    fn failed_delta_does_not_advance_persisted_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("evm.redb");
        let mut store =
            RedbEvmStateStore::open_or_initialize(&path, initial_snapshot(), LIMITS).unwrap();
        let mut delta = empty_delta(12);
        delta.storage_updates.push(apex_evm::EvmStorageUpdate {
            address: address(1),
            key: U256Word::from_u64(5),
            before: Some(U256Word::from_u64(999)),
            after: None,
        });
        assert!(matches!(
            store.apply(&delta),
            Err(PersistentEvmStateError::Apply(
                EvmStateApplyError::StorageBeforeMismatch(_, _)
            ))
        ));
        assert_eq!(store.snapshot().unwrap().parent_state_version(), 12);
        drop(store);
        let reopened =
            RedbEvmStateStore::open_or_initialize(&path, initial_snapshot(), LIMITS).unwrap();
        assert_eq!(reopened.snapshot().unwrap().parent_state_version(), 12);
    }

    #[test]
    fn existing_database_is_not_overwritten_by_new_initial_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("evm.redb");
        let first = initial_snapshot();
        let store = RedbEvmStateStore::open_or_initialize(&path, first.clone(), LIMITS).unwrap();
        drop(store);
        let replacement = EvmStateSnapshot::try_new(
            999,
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
            LIMITS,
        )
        .unwrap();
        let reopened = RedbEvmStateStore::open_or_initialize(&path, replacement, LIMITS).unwrap();
        assert_eq!(reopened.snapshot().unwrap().parent_state_version(), 12);
        assert_eq!(reopened.snapshot().unwrap().accounts().count(), 1);
    }

    fn rewrite_metadata(path: &std::path::Path, key: u8, value: Option<u64>) {
        let db = Database::create(path).unwrap();
        let write = db.begin_write().unwrap();
        {
            let mut table = write.open_table(META).unwrap();
            match value {
                Some(value) => {
                    table.insert(key, value).unwrap();
                }
                None => {
                    table.remove(key).unwrap();
                }
            }
        }
        write.commit().unwrap();
    }

    #[test]
    fn integrity_check_passes_after_committed_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("evm.redb");
        let mut store =
            RedbEvmStateStore::open_or_initialize(&path, initial_snapshot(), LIMITS).unwrap();
        store.check_integrity().unwrap();
    }

    #[test]
    fn reopen_rejects_incomplete_metadata_without_reinitializing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("evm.redb");
        drop(RedbEvmStateStore::open_or_initialize(&path, initial_snapshot(), LIMITS).unwrap());

        rewrite_metadata(&path, META_CURRENT_VERSION_KEY, None);

        assert_eq!(
            RedbEvmStateStore::open_or_initialize(&path, initial_snapshot(), LIMITS).err(),
            Some(PersistentEvmStateError::CorruptMetadata)
        );
    }

    #[test]
    fn reopen_rejects_unsupported_schema_without_reinitializing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("evm.redb");
        drop(RedbEvmStateStore::open_or_initialize(&path, initial_snapshot(), LIMITS).unwrap());

        rewrite_metadata(&path, META_SCHEMA_KEY, Some(SCHEMA_VERSION + 1));

        assert_eq!(
            RedbEvmStateStore::open_or_initialize(&path, initial_snapshot(), LIMITS).err(),
            Some(PersistentEvmStateError::UnsupportedSchema(
                SCHEMA_VERSION + 1
            ))
        );
    }

    #[test]
    fn reopen_rejects_malformed_current_account_record() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("evm.redb");
        drop(RedbEvmStateStore::open_or_initialize(&path, initial_snapshot(), LIMITS).unwrap());

        let db = Database::create(&path).unwrap();
        let write = db.begin_write().unwrap();
        {
            let mut table = write.open_table(ACCOUNTS).unwrap();
            let mut key = [0_u8; ACCOUNT_KEY_LEN];
            key[..8].copy_from_slice(&12_u64.to_be_bytes());
            key[8..].copy_from_slice(&address(1).0);
            table.insert(key.as_slice(), &[0xAA][..]).unwrap();
        }
        write.commit().unwrap();
        drop(db);

        assert_eq!(
            RedbEvmStateStore::open_or_initialize(&path, initial_snapshot(), LIMITS).err(),
            Some(PersistentEvmStateError::CorruptRecord("account"))
        );
    }

    #[test]
    fn pre_commit_failure_latches_store_and_reopen_keeps_parent_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("evm.redb");
        let mut store =
            RedbEvmStateStore::open_or_initialize(&path, initial_snapshot(), LIMITS).unwrap();
        let delta = empty_delta(12);
        store.commit_fault = Some(CommitFault::BeforeCommit);

        assert_eq!(
            store.apply(&delta),
            Err(PersistentEvmStateError::Backend(
                "injected failure before commit".into()
            ))
        );
        assert_eq!(
            store.snapshot().err(),
            Some(PersistentEvmStateError::ReopenRequired)
        );
        drop(store);

        let mut reopened =
            RedbEvmStateStore::open_or_initialize(&path, initial_snapshot(), LIMITS).unwrap();
        assert_eq!(reopened.snapshot().unwrap().parent_state_version(), 12);
        assert_eq!(reopened.apply(&delta), Ok(13));
        drop(reopened);

        let verified =
            RedbEvmStateStore::open_or_initialize(&path, initial_snapshot(), LIMITS).unwrap();
        assert_eq!(verified.snapshot().unwrap().parent_state_version(), 13);
    }

    #[test]
    fn lost_commit_acknowledgement_latches_and_reopen_reconciles_committed_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("evm.redb");
        let mut store =
            RedbEvmStateStore::open_or_initialize(&path, initial_snapshot(), LIMITS).unwrap();
        let delta = empty_delta(12);
        store.commit_fault = Some(CommitFault::AcknowledgementLost);

        assert_eq!(
            store.apply(&delta),
            Err(PersistentEvmStateError::Backend(
                "injected lost commit acknowledgement".into()
            ))
        );
        assert_eq!(
            store.snapshot().err(),
            Some(PersistentEvmStateError::ReopenRequired)
        );
        drop(store);

        let mut reopened =
            RedbEvmStateStore::open_or_initialize(&path, initial_snapshot(), LIMITS).unwrap();
        assert_eq!(reopened.snapshot().unwrap().parent_state_version(), 13);
        assert_eq!(
            reopened.apply(&delta),
            Err(PersistentEvmStateError::Apply(
                EvmStateApplyError::StaleParent {
                    expected: 12,
                    actual: 13,
                }
            ))
        );
    }

    #[test]
    fn orphaned_future_version_records_are_not_selected_as_current_state() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("evm.redb");
        let store =
            RedbEvmStateStore::open_or_initialize(&path, initial_snapshot(), LIMITS).unwrap();
        let mut candidate = store.state.clone();
        candidate.apply(&empty_delta(12)).unwrap();
        let orphaned_snapshot = candidate.snapshot().clone();
        drop(store);

        let db = Database::create(&path).unwrap();
        let write = db.begin_write().unwrap();
        write_snapshot_records(&write, &orphaned_snapshot).unwrap();
        write.commit().unwrap();
        drop(db);

        let mut reopened =
            RedbEvmStateStore::open_or_initialize(&path, initial_snapshot(), LIMITS).unwrap();
        assert_eq!(reopened.snapshot().unwrap().parent_state_version(), 12);
        assert_eq!(reopened.apply(&empty_delta(12)), Ok(13));
        drop(reopened);

        let verified =
            RedbEvmStateStore::open_or_initialize(&path, initial_snapshot(), LIMITS).unwrap();
        assert_eq!(verified.snapshot().unwrap().parent_state_version(), 13);
    }
}
