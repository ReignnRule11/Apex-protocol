//! Durable redb adapter for the APEX versioned state-store contract.
//!
//! The on-disk account encoding in this crate is a private storage schema,
//! not a canonical protocol encoding or consensus state commitment.

use apex_consensus::{AccountId, AccountState, PlannedAccountUpdate};
use apex_state::{
    StateStore, StateStoreError, StateVersion, VersionedBlockTransitionPlan, VersionedStateSnapshot,
};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use std::collections::BTreeMap;
use std::path::Path;

const SCHEMA_VERSION: u64 = 1;
const META_SCHEMA_KEY: u8 = 0;
const META_STATE_VERSION_KEY: u8 = 1;
const META: TableDefinition<u8, u64> = TableDefinition::new("apex_metadata_v1");
const ACCOUNTS: TableDefinition<u64, &[u8]> = TableDefinition::new("apex_accounts_v1");

/// Persistent state adapter backed by a single redb database file.
///
/// The adapter uses redb's default exclusive-file locking mode and immediate
/// transaction durability. It is single-process/single-handle in this spike;
/// multi-process writer modes are deliberately not enabled.
pub struct RedbStateStore {
    db: Database,
}

impl RedbStateStore {
    /// Open an existing store, or initialize an empty store with the supplied
    /// genesis state. Existing initialized state is never overwritten.
    pub fn open_or_initialize(
        path: impl AsRef<Path>,
        initial_version: StateVersion,
        initial_accounts: &BTreeMap<AccountId, AccountState>,
    ) -> Result<Self, StateStoreError> {
        let db = Database::create(path).map_err(backend)?;
        let write = db.begin_write().map_err(backend)?;

        let existing = {
            let meta = write.open_table(META).map_err(backend)?;
            let schema = meta
                .get(META_SCHEMA_KEY)
                .map_err(backend)?
                .map(|v| v.value());
            let version = meta
                .get(META_STATE_VERSION_KEY)
                .map_err(backend)?
                .map(|v| v.value());
            match (schema, version) {
                (Some(schema), Some(version)) => {
                    if schema != SCHEMA_VERSION {
                        return Err(StateStoreError::UnsupportedSchema(schema));
                    }
                    Some(version)
                }
                (None, None) => None,
                _ => return Err(StateStoreError::CorruptMetadata),
            }
        };

        if existing.is_none() {
            {
                let mut accounts = write.open_table(ACCOUNTS).map_err(backend)?;
                if accounts.iter().map_err(backend)?.next().is_some() {
                    return Err(StateStoreError::CorruptMetadata);
                }
                for (id, state) in initial_accounts {
                    let bytes = encode_account(*state);
                    accounts.insert(id.0, bytes.as_slice()).map_err(backend)?;
                }
            }
            {
                let mut meta = write.open_table(META).map_err(backend)?;
                meta.insert(META_SCHEMA_KEY, SCHEMA_VERSION)
                    .map_err(backend)?;
                meta.insert(META_STATE_VERSION_KEY, initial_version.0)
                    .map_err(backend)?;
            }
        }

        write.commit().map_err(backend)?;
        Ok(Self { db })
    }

    /// Run redb's integrity verification explicitly. Opening the database
    /// also performs redb's normal recovery procedure.
    pub fn check_integrity(&mut self) -> Result<(), StateStoreError> {
        self.db.check_integrity().map_err(backend)?;
        Ok(())
    }
}

impl StateStore for RedbStateStore {
    fn snapshot(&self) -> Result<VersionedStateSnapshot, StateStoreError> {
        let read = self.db.begin_read().map_err(backend)?;
        let version = {
            let meta = read.open_table(META).map_err(backend)?;
            let schema = meta
                .get(META_SCHEMA_KEY)
                .map_err(backend)?
                .ok_or(StateStoreError::Uninitialized)?
                .value();
            if schema != SCHEMA_VERSION {
                return Err(StateStoreError::UnsupportedSchema(schema));
            }
            meta.get(META_STATE_VERSION_KEY)
                .map_err(backend)?
                .ok_or(StateStoreError::CorruptMetadata)?
                .value()
        };

        let mut accounts = BTreeMap::new();
        {
            let table = read.open_table(ACCOUNTS).map_err(backend)?;
            for entry in table.iter().map_err(backend)? {
                let (key, value) = entry.map_err(backend)?;
                let id = AccountId(key.value());
                let state = decode_account(id, value.value())?;
                if accounts.insert(id, state).is_some() {
                    return Err(StateStoreError::CorruptAccount(id));
                }
            }
        }
        Ok(VersionedStateSnapshot::new(StateVersion(version), accounts))
    }

    fn apply(
        &mut self,
        plan: &VersionedBlockTransitionPlan,
    ) -> Result<StateVersion, StateStoreError> {
        let write = self.db.begin_write().map_err(backend)?;
        let current_version = {
            let meta = write.open_table(META).map_err(backend)?;
            let schema = meta
                .get(META_SCHEMA_KEY)
                .map_err(backend)?
                .ok_or(StateStoreError::Uninitialized)?
                .value();
            if schema != SCHEMA_VERSION {
                return Err(StateStoreError::UnsupportedSchema(schema));
            }
            meta.get(META_STATE_VERSION_KEY)
                .map_err(backend)?
                .ok_or(StateStoreError::CorruptMetadata)?
                .value()
        };

        if plan.parent_version != StateVersion(current_version) {
            return Err(StateStoreError::StaleVersion {
                expected: plan.parent_version,
                actual: StateVersion(current_version),
            });
        }
        let next_version = StateVersion(
            current_version
                .checked_add(1)
                .ok_or(StateStoreError::VersionOverflow)?,
        );

        let mut previous = None;
        {
            let mut accounts = write.open_table(ACCOUNTS).map_err(backend)?;
            for update in &plan.transition.account_updates {
                if previous.is_some_and(|id| id >= update.account) {
                    return Err(StateStoreError::DuplicateOrUnorderedAccountUpdate);
                }
                previous = Some(update.account);
                let stored = accounts
                    .get(update.account.0)
                    .map_err(backend)?
                    .ok_or(StateStoreError::BeforeStateMismatch(update.account))?;
                let current = decode_account(update.account, stored.value())?;
                if current != update.before {
                    return Err(StateStoreError::BeforeStateMismatch(update.account));
                }
            }

            // All account preconditions are checked before the first mutation.
            for PlannedAccountUpdate { account, after, .. } in &plan.transition.account_updates {
                let bytes = encode_account(*after);
                accounts
                    .insert(account.0, bytes.as_slice())
                    .map_err(backend)?;
            }
        }

        {
            let mut meta = write.open_table(META).map_err(backend)?;
            meta.insert(META_STATE_VERSION_KEY, next_version.0)
                .map_err(backend)?;
        }

        // redb defaults to Durability::Immediate: success is returned only
        // after the transaction reaches its documented durability boundary.
        write.commit().map_err(backend)?;
        Ok(next_version)
    }
}

fn encode_account(state: AccountState) -> [u8; 24] {
    let mut bytes = [0_u8; 24];
    bytes[..8].copy_from_slice(&state.nonce.to_le_bytes());
    bytes[8..].copy_from_slice(&state.balance.to_le_bytes());
    bytes
}

fn decode_account(id: AccountId, bytes: &[u8]) -> Result<AccountState, StateStoreError> {
    if bytes.len() != 24 {
        return Err(StateStoreError::CorruptAccount(id));
    }
    let nonce = u64::from_le_bytes(
        bytes[..8]
            .try_into()
            .map_err(|_| StateStoreError::CorruptAccount(id))?,
    );
    let balance = u128::from_le_bytes(
        bytes[8..]
            .try_into()
            .map_err(|_| StateStoreError::CorruptAccount(id))?,
    );
    Ok(AccountState { nonce, balance })
}

fn backend(error: impl std::fmt::Display) -> StateStoreError {
    StateStoreError::Backend(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use apex_consensus::{
        AdmissionContext, AdmissionState, BlockTransaction, TransactionAdmission, TransactionId,
    };
    use apex_state::{StateStore, plan_versioned_block_transition};
    use apex_types::ChainId;
    use std::collections::BTreeMap;

    fn genesis() -> BTreeMap<AccountId, AccountState> {
        BTreeMap::from([
            (
                AccountId(1),
                AccountState {
                    nonce: 0,
                    balance: 1_000_000,
                },
            ),
            (
                AccountId(2),
                AccountState {
                    nonce: 0,
                    balance: 500,
                },
            ),
            (
                AccountId(3),
                AccountState {
                    nonce: 0,
                    balance: 700,
                },
            ),
        ])
    }

    fn transactions() -> [BlockTransaction; 2] {
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

    fn plan(store: &RedbStateStore) -> VersionedBlockTransitionPlan {
        let snapshot = store.snapshot().unwrap();
        plan_versioned_block_transition(
            AdmissionContext {
                chain_id: ChainId(7),
                protocol_version: 1,
                block_gas_limit: 100_000,
            },
            &transactions(),
            &snapshot,
        )
        .unwrap()
    }

    #[test]
    fn durable_apply_survives_close_and_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.redb");
        {
            let mut store =
                RedbStateStore::open_or_initialize(&path, StateVersion(12), &genesis()).unwrap();
            assert_eq!(store.snapshot().unwrap().version(), StateVersion(12));
            assert_eq!(store.apply(&plan(&store)), Ok(StateVersion(13)));
            store.check_integrity().unwrap();
        }
        {
            let store =
                RedbStateStore::open_or_initialize(&path, StateVersion(0), &BTreeMap::new())
                    .unwrap();
            let snapshot = store.snapshot().unwrap();
            assert_eq!(snapshot.version(), StateVersion(13));
            assert_eq!(
                snapshot.account(AccountId(1)).unwrap(),
                AccountState {
                    nonce: 2,
                    balance: 957_700
                }
            );
            assert_eq!(snapshot.account(AccountId(2)).unwrap().balance, 600);
            assert_eq!(snapshot.account(AccountId(3)).unwrap().balance, 900);
        }
    }

    #[test]
    fn stale_plan_is_rejected_and_state_is_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.redb");
        let mut store =
            RedbStateStore::open_or_initialize(&path, StateVersion(12), &genesis()).unwrap();
        let stale = plan(&store);
        let current = plan(&store);
        assert_eq!(store.apply(&current), Ok(StateVersion(13)));
        let before = store.snapshot().unwrap();
        assert!(matches!(
            store.apply(&stale),
            Err(StateStoreError::StaleVersion { .. })
        ));
        assert_eq!(store.snapshot().unwrap(), before);
    }

    #[test]
    fn late_before_state_mismatch_aborts_without_partial_writes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.redb");
        let mut store =
            RedbStateStore::open_or_initialize(&path, StateVersion(12), &genesis()).unwrap();
        let mut invalid = plan(&store);
        invalid.transition.account_updates[1].before.balance += 1;
        let before = store.snapshot().unwrap();
        assert_eq!(
            store.apply(&invalid),
            Err(StateStoreError::BeforeStateMismatch(AccountId(2)))
        );
        assert_eq!(store.snapshot().unwrap(), before);
    }

    #[test]
    fn existing_database_is_never_reinitialized_from_new_genesis_input() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.redb");
        let first = RedbStateStore::open_or_initialize(&path, StateVersion(8), &genesis()).unwrap();
        drop(first);
        let second =
            RedbStateStore::open_or_initialize(&path, StateVersion(99), &BTreeMap::new()).unwrap();
        assert_eq!(second.snapshot().unwrap().version(), StateVersion(8));
        assert_eq!(second.snapshot().unwrap().accounts().len(), 3);
    }

    #[test]
    fn materialized_snapshot_remains_stable_after_later_commit() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.redb");
        let mut store =
            RedbStateStore::open_or_initialize(&path, StateVersion(12), &genesis()).unwrap();
        let old_snapshot = store.snapshot().unwrap();
        let transition = plan(&store);
        store.apply(&transition).unwrap();
        let new_snapshot = store.snapshot().unwrap();

        assert_eq!(old_snapshot.version(), StateVersion(12));
        assert_eq!(old_snapshot.account(AccountId(2)).unwrap().balance, 500);
        assert_eq!(new_snapshot.version(), StateVersion(13));
        assert_eq!(new_snapshot.account(AccountId(2)).unwrap().balance, 600);
    }

    #[test]
    fn closed_database_file_can_be_copied_and_restored_as_a_cold_backup() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.redb");
        let backup = dir.path().join("backup.redb");
        let expected = {
            let mut store =
                RedbStateStore::open_or_initialize(&source, StateVersion(12), &genesis()).unwrap();
            store.apply(&plan(&store)).unwrap();
            store.check_integrity().unwrap();
            let snapshot = store.snapshot().unwrap();
            let RedbStateStore { db } = store;
            db.close().unwrap();
            snapshot
        };

        // This is deliberately a cold copy: the database handle is closed
        // before the file is copied. Copying a live database is unsupported.
        std::fs::copy(&source, &backup).unwrap();
        let restored =
            RedbStateStore::open_or_initialize(&backup, StateVersion(0), &BTreeMap::new()).unwrap();
        assert_eq!(restored.snapshot().unwrap(), expected);
    }

    #[test]
    fn unknown_schema_version_fails_closed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.redb");
        {
            let db = Database::create(&path).unwrap();
            let write = db.begin_write().unwrap();
            {
                let mut meta = write.open_table(META).unwrap();
                meta.insert(META_SCHEMA_KEY, SCHEMA_VERSION + 1).unwrap();
                meta.insert(META_STATE_VERSION_KEY, 12).unwrap();
            }
            write.commit().unwrap();
        }

        assert!(matches!(
            RedbStateStore::open_or_initialize(&path, StateVersion(0), &BTreeMap::new()),
            Err(StateStoreError::UnsupportedSchema(version)) if version == SCHEMA_VERSION + 1
        ));
    }

    #[test]
    fn abrupt_process_exit_recovers_only_committed_state() {
        use std::process::Command;

        if let Ok(path) = std::env::var("APEX_REDB_CRASH_CHILD_PATH") {
            let mode = std::env::var("APEX_REDB_CRASH_CHILD_MODE").unwrap();
            let store =
                RedbStateStore::open_or_initialize(&path, StateVersion(12), &genesis()).unwrap();

            if mode == "uncommitted" {
                let write = store.db.begin_write().unwrap();
                {
                    let mut accounts = write.open_table(ACCOUNTS).unwrap();
                    let changed = encode_account(AccountState {
                        nonce: 999,
                        balance: 999,
                    });
                    accounts.insert(AccountId(1).0, changed.as_slice()).unwrap();
                }
                // The process terminates with a live, uncommitted transaction.
                eprintln!("APEX_REDB_CHILD_ABORT:uncommitted");
                std::process::abort();
            }

            assert_eq!(mode, "committed");
            let mut store = store;
            let transition = plan(&store);
            store.apply(&transition).unwrap();
            // Terminate immediately after the adapter reports commit success.
            eprintln!("APEX_REDB_CHILD_ABORT:committed");
            std::process::abort();
        }

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("crash-recovery.redb");
        drop(RedbStateStore::open_or_initialize(&path, StateVersion(12), &genesis()).unwrap());

        for mode in ["uncommitted", "committed"] {
            let output = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "tests::abrupt_process_exit_recovers_only_committed_state",
                    "--nocapture",
                ])
                .env("APEX_REDB_CRASH_CHILD_PATH", &path)
                .env("APEX_REDB_CRASH_CHILD_MODE", mode)
                .output()
                .unwrap();
            assert!(
                !output.status.success(),
                "child process unexpectedly exited cleanly"
            );
            let marker = format!("APEX_REDB_CHILD_ABORT:{mode}");
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.contains(&marker),
                "child did not reach abort point: {stderr}"
            );
            let recovered =
                RedbStateStore::open_or_initialize(&path, StateVersion(0), &BTreeMap::new())
                    .unwrap();
            let snapshot = recovered.snapshot().unwrap();
            if mode == "uncommitted" {
                assert_eq!(snapshot.version(), StateVersion(12));
                assert_eq!(snapshot.account(AccountId(1)).unwrap().nonce, 0);
                assert_eq!(snapshot.account(AccountId(1)).unwrap().balance, 1_000_000);
            } else {
                assert_eq!(snapshot.version(), StateVersion(13));
                assert_eq!(snapshot.account(AccountId(1)).unwrap().nonce, 2);
                assert_eq!(snapshot.account(AccountId(1)).unwrap().balance, 957_700);
            }
        }
    }
}
