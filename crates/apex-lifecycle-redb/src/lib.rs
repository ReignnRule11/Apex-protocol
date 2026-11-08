//! Durable lifecycle checkpoint storage backed by redb.
//!
//! The local database provides atomic compare-and-advance and survives ordinary
//! process restarts. It is NOT an anti-rollback root: restoring an older database
//! image can restore an older head. Production startup must compare every
//! network's local head with an independently protected external witness.

use apex_consensus::validator_lifecycle_checkpoint::{
    CheckpointStoreError, LifecycleCheckpointAnchor, MonotonicLifecycleCheckpointStore,
};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use std::path::Path;
use thiserror::Error;

const SCHEMA_VERSION: u64 = 1;
const META: TableDefinition<u8, u64> = TableDefinition::new("apex_lifecycle_meta_v1");
const HEADS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("apex_lifecycle_heads_v1");

#[derive(Debug, Error, Clone, Copy, Eq, PartialEq)]
pub enum LifecycleStoreOpenError {
    #[error("lifecycle database is unavailable")]
    Unavailable,
    #[error("lifecycle database schema is unsupported")]
    UnsupportedSchema,
    #[error("lifecycle database metadata is corrupt")]
    CorruptMetadata,
}

/// A redb-backed, single-process checkpoint store.
///
/// redb's exclusive file lock prevents a second process from opening the same
/// database for writes. Each compare-and-advance is one committed write
/// transaction. Callers must still use an external rollback witness to detect
/// restoration of an older database or replacement of the storage device.
pub struct RedbLifecycleCheckpointStore {
    db: Database,
}

impl RedbLifecycleCheckpointStore {
    /// Open or initialize the database without overwriting existing checkpoint heads.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, LifecycleStoreOpenError> {
        let db = Database::create(path).map_err(|_| LifecycleStoreOpenError::Unavailable)?;
        let write = db
            .begin_write()
            .map_err(|_| LifecycleStoreOpenError::Unavailable)?;
        {
            let mut meta = write
                .open_table(META)
                .map_err(|_| LifecycleStoreOpenError::Unavailable)?;
            let version = meta
                .get(0)
                .map_err(|_| LifecycleStoreOpenError::Unavailable)?
                .map(|value| value.value());
            match version {
                Some(SCHEMA_VERSION) => {}
                Some(_) => return Err(LifecycleStoreOpenError::UnsupportedSchema),
                None => {
                    if meta
                        .iter()
                        .map_err(|_| LifecycleStoreOpenError::Unavailable)?
                        .next()
                        .is_some()
                    {
                        return Err(LifecycleStoreOpenError::CorruptMetadata);
                    }
                    meta.insert(0, SCHEMA_VERSION)
                        .map_err(|_| LifecycleStoreOpenError::Unavailable)?;
                }
            }
        }
        {
            let _ = write
                .open_table(HEADS)
                .map_err(|_| LifecycleStoreOpenError::Unavailable)?;
        }
        write
            .commit()
            .map_err(|_| LifecycleStoreOpenError::Unavailable)?;
        Ok(Self { db })
    }

    /// Verify redb's structural integrity. This does not prove anti-rollback.
    pub fn check_integrity(&mut self) -> Result<(), CheckpointStoreError> {
        self.db
            .check_integrity()
            .map_err(|_| CheckpointStoreError::Unavailable)?;
        Ok(())
    }
}

impl MonotonicLifecycleCheckpointStore for RedbLifecycleCheckpointStore {
    fn load(
        &self,
        network_id: [u8; 32],
    ) -> Result<Option<LifecycleCheckpointAnchor>, CheckpointStoreError> {
        let read = self
            .db
            .begin_read()
            .map_err(|_| CheckpointStoreError::Unavailable)?;
        let table = read
            .open_table(HEADS)
            .map_err(|_| CheckpointStoreError::Unavailable)?;
        let Some(value) = table
            .get(network_id.as_slice())
            .map_err(|_| CheckpointStoreError::Unavailable)?
        else {
            return Ok(None);
        };
        Ok(Some(decode_anchor(value.value())?))
    }

    fn compare_and_advance(
        &self,
        network_id: [u8; 32],
        expected: Option<LifecycleCheckpointAnchor>,
        next: LifecycleCheckpointAnchor,
    ) -> Result<bool, CheckpointStoreError> {
        let write = self
            .db
            .begin_write()
            .map_err(|_| CheckpointStoreError::Unavailable)?;
        {
            let mut table = write
                .open_table(HEADS)
                .map_err(|_| CheckpointStoreError::Unavailable)?;
            let current = table
                .get(network_id.as_slice())
                .map_err(|_| CheckpointStoreError::Unavailable)?
                .map(|value| decode_anchor(value.value()))
                .transpose()?;
            if current != expected {
                return Ok(false);
            }
            let valid = match expected {
                None => next.sequence == 1,
                Some(anchor) => anchor.sequence.checked_add(1) == Some(next.sequence),
            };
            if !valid {
                return Err(CheckpointStoreError::InvalidAdvance);
            }
            let encoded = encode_anchor(next);
            table
                .insert(network_id.as_slice(), encoded.as_slice())
                .map_err(|_| CheckpointStoreError::Unavailable)?;
        }
        write
            .commit()
            .map_err(|_| CheckpointStoreError::Unavailable)?;
        Ok(true)
    }
}

fn encode_anchor(anchor: LifecycleCheckpointAnchor) -> [u8; 40] {
    let mut bytes = [0; 40];
    bytes[..8].copy_from_slice(&anchor.sequence.to_be_bytes());
    bytes[8..].copy_from_slice(&anchor.checkpoint_hash);
    bytes
}

fn decode_anchor(bytes: &[u8]) -> Result<LifecycleCheckpointAnchor, CheckpointStoreError> {
    if bytes.len() != 40 {
        return Err(CheckpointStoreError::Unavailable);
    }
    let sequence = u64::from_be_bytes(
        bytes[..8]
            .try_into()
            .map_err(|_| CheckpointStoreError::Unavailable)?,
    );
    if sequence == 0 {
        return Err(CheckpointStoreError::Unavailable);
    }
    let checkpoint_hash = bytes[8..]
        .try_into()
        .map_err(|_| CheckpointStoreError::Unavailable)?;
    Ok(LifecycleCheckpointAnchor {
        sequence,
        checkpoint_hash,
    })
}

/// Independent witness for a checkpoint head. A production implementation
/// must be backed by a separately protected monotonic service, TPM/HSM-backed
/// state, or equivalent trust domain—not the same database, disk, or backup set.
pub trait ExternalLifecycleRollbackWitness {
    fn trusted_head(
        &self,
        network_id: [u8; 32],
    ) -> Result<Option<LifecycleCheckpointAnchor>, RollbackWitnessError>;
}

#[derive(Debug, Error, Clone, Copy, Eq, PartialEq)]
pub enum RollbackWitnessError {
    #[error("external rollback witness is unavailable")]
    Unavailable,
    #[error("local checkpoint head differs from the independently trusted head")]
    HeadMismatch,
}

/// Fail closed when local storage has been rolled back, replaced, or lost
/// relative to the independently maintained witness.
pub fn verify_external_rollback_witness(
    local: &impl MonotonicLifecycleCheckpointStore,
    witness: &impl ExternalLifecycleRollbackWitness,
    network_id: [u8; 32],
) -> Result<(), RollbackWitnessError> {
    let local_head = local
        .load(network_id)
        .map_err(|_| RollbackWitnessError::Unavailable)?;
    let trusted_head = witness.trusted_head(network_id)?;
    if local_head != trusted_head {
        return Err(RollbackWitnessError::HeadMismatch);
    }
    Ok(())
}

/// Fail-closed startup verification for every configured network.
///
/// Unlike the low-level equality check, startup requires both stores to contain
/// an initialized head. An uninitialized network must be bootstrapped through a
/// separately authorized procedure; startup never silently treats an empty
/// local database and empty witness as trusted genesis.
#[derive(Debug, Error, Clone, Copy, Eq, PartialEq)]
pub enum LifecycleStartupError {
    #[error("at least one network must be configured for lifecycle startup verification")]
    EmptyNetworkSet,
    #[error("configured network ID must not be zero")]
    InvalidNetwork,
    #[error("configured network ID appears more than once")]
    DuplicateNetwork,
    #[error("local lifecycle head is unavailable")]
    LocalUnavailable { network_id: [u8; 32] },
    #[error("independent rollback witness is unavailable")]
    WitnessUnavailable { network_id: [u8; 32] },
    #[error("network has no initialized local and independently witnessed checkpoint")]
    UninitializedNetwork { network_id: [u8; 32] },
    #[error("local lifecycle head differs from the independent witness")]
    HeadMismatch { network_id: [u8; 32] },
}

/// Verify all configured network heads before enabling node services.
///
/// Callers must invoke this gate before starting consensus, signing, networking,
/// or any service that can act on lifecycle state. It performs no recovery,
/// initialization, or mutation. The witness implementation must live in a
/// separately protected trust domain; this function cannot establish that
/// property by itself.
pub fn verify_lifecycle_startup(
    local: &impl MonotonicLifecycleCheckpointStore,
    witness: &impl ExternalLifecycleRollbackWitness,
    network_ids: &[[u8; 32]],
) -> Result<(), LifecycleStartupError> {
    use std::collections::BTreeSet;

    if network_ids.is_empty() {
        return Err(LifecycleStartupError::EmptyNetworkSet);
    }

    let mut seen = BTreeSet::new();
    for network_id in network_ids {
        if *network_id == [0; 32] {
            return Err(LifecycleStartupError::InvalidNetwork);
        }
        if !seen.insert(*network_id) {
            return Err(LifecycleStartupError::DuplicateNetwork);
        }

        let local_head =
            local
                .load(*network_id)
                .map_err(|_| LifecycleStartupError::LocalUnavailable {
                    network_id: *network_id,
                })?;
        let trusted_head = witness.trusted_head(*network_id).map_err(|_| {
            LifecycleStartupError::WitnessUnavailable {
                network_id: *network_id,
            }
        })?;

        let (Some(local_head), Some(trusted_head)) = (local_head, trusted_head) else {
            return Err(LifecycleStartupError::UninitializedNetwork {
                network_id: *network_id,
            });
        };
        if local_head != trusted_head {
            return Err(LifecycleStartupError::HeadMismatch {
                network_id: *network_id,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod startup_tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct Witness {
        heads: Mutex<BTreeMap<[u8; 32], LifecycleCheckpointAnchor>>,
        unavailable: bool,
    }

    impl ExternalLifecycleRollbackWitness for Witness {
        fn trusted_head(
            &self,
            network_id: [u8; 32],
        ) -> Result<Option<LifecycleCheckpointAnchor>, RollbackWitnessError> {
            if self.unavailable {
                return Err(RollbackWitnessError::Unavailable);
            }
            Ok(self.heads.lock().unwrap().get(&network_id).copied())
        }
    }

    fn anchor(sequence: u64, byte: u8) -> LifecycleCheckpointAnchor {
        LifecycleCheckpointAnchor {
            sequence,
            checkpoint_hash: [byte; 32],
        }
    }

    #[test]
    fn startup_requires_at_least_one_network() {
        let dir = tempfile::tempdir().unwrap();
        let local = RedbLifecycleCheckpointStore::open(dir.path().join("local.redb")).unwrap();
        assert_eq!(
            verify_lifecycle_startup(&local, &Witness::default(), &[]),
            Err(LifecycleStartupError::EmptyNetworkSet)
        );
    }

    #[test]
    fn startup_rejects_zero_and_duplicate_network_ids() {
        let dir = tempfile::tempdir().unwrap();
        let local = RedbLifecycleCheckpointStore::open(dir.path().join("local.redb")).unwrap();
        let witness = Witness::default();
        assert_eq!(
            verify_lifecycle_startup(&local, &witness, &[[0; 32]]),
            Err(LifecycleStartupError::InvalidNetwork)
        );
        assert_eq!(
            verify_lifecycle_startup(&local, &witness, &[[1; 32], [1; 32]]),
            Err(LifecycleStartupError::DuplicateNetwork)
        );
    }

    #[test]
    fn startup_accepts_only_matching_initialized_heads_for_every_network() {
        let dir = tempfile::tempdir().unwrap();
        let local = RedbLifecycleCheckpointStore::open(dir.path().join("local.redb")).unwrap();
        let witness = Witness::default();
        for (network, head) in [([1; 32], anchor(4, 4)), ([2; 32], anchor(9, 9))] {
            assert!(local.compare_and_advance(network, None, head).unwrap());
            witness.heads.lock().unwrap().insert(network, head);
        }
        assert_eq!(
            verify_lifecycle_startup(&local, &witness, &[[1; 32], [2; 32]]),
            Ok(())
        );
    }

    #[test]
    fn startup_fails_closed_on_head_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        let local = RedbLifecycleCheckpointStore::open(dir.path().join("local.redb")).unwrap();
        let network = [3; 32];
        assert!(
            local
                .compare_and_advance(network, None, anchor(2, 2))
                .unwrap()
        );
        let witness = Witness::default();
        witness.heads.lock().unwrap().insert(network, anchor(3, 3));
        assert_eq!(
            verify_lifecycle_startup(&local, &witness, &[network]),
            Err(LifecycleStartupError::HeadMismatch {
                network_id: network
            })
        );
    }

    #[test]
    fn startup_fails_closed_when_witness_is_unavailable() {
        let dir = tempfile::tempdir().unwrap();
        let local = RedbLifecycleCheckpointStore::open(dir.path().join("local.redb")).unwrap();
        let network = [4; 32];
        let head = anchor(1, 1);
        assert!(local.compare_and_advance(network, None, head).unwrap());
        let witness = Witness {
            heads: Mutex::new(BTreeMap::from([(network, head)])),
            unavailable: true,
        };
        assert_eq!(
            verify_lifecycle_startup(&local, &witness, &[network]),
            Err(LifecycleStartupError::WitnessUnavailable {
                network_id: network
            })
        );
    }

    #[test]
    fn startup_rejects_empty_local_and_witness_as_uninitialized() {
        let dir = tempfile::tempdir().unwrap();
        let local = RedbLifecycleCheckpointStore::open(dir.path().join("local.redb")).unwrap();
        let network = [5; 32];
        assert_eq!(
            verify_lifecycle_startup(&local, &Witness::default(), &[network]),
            Err(LifecycleStartupError::UninitializedNetwork {
                network_id: network
            })
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    fn anchor(sequence: u64, byte: u8) -> LifecycleCheckpointAnchor {
        LifecycleCheckpointAnchor {
            sequence,
            checkpoint_hash: [byte; 32],
        }
    }

    #[test]
    fn checkpoint_survives_close_and_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("lifecycle.redb");
        let network = [7; 32];
        {
            let store = RedbLifecycleCheckpointStore::open(&path).unwrap();
            assert_eq!(store.load(network).unwrap(), None);
            assert!(
                store
                    .compare_and_advance(network, None, anchor(1, 1))
                    .unwrap()
            );
            assert_eq!(store.load(network).unwrap(), Some(anchor(1, 1)));
        }
        {
            let mut store = RedbLifecycleCheckpointStore::open(&path).unwrap();
            assert_eq!(store.load(network).unwrap(), Some(anchor(1, 1)));
            assert!(
                store
                    .compare_and_advance(network, Some(anchor(1, 1)), anchor(2, 2))
                    .unwrap()
            );
            store.check_integrity().unwrap();
        }
    }

    #[test]
    fn stale_writer_cannot_overwrite_a_newer_head() {
        let dir = tempfile::tempdir().unwrap();
        let store = RedbLifecycleCheckpointStore::open(dir.path().join("lifecycle.redb")).unwrap();
        let network = [8; 32];
        assert!(
            store
                .compare_and_advance(network, None, anchor(1, 1))
                .unwrap()
        );
        assert!(
            store
                .compare_and_advance(network, Some(anchor(1, 1)), anchor(2, 2))
                .unwrap()
        );
        assert!(
            !store
                .compare_and_advance(network, Some(anchor(1, 1)), anchor(2, 3))
                .unwrap()
        );
        assert_eq!(store.load(network).unwrap(), Some(anchor(2, 2)));
    }

    #[test]
    fn invalid_sequence_advance_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let store = RedbLifecycleCheckpointStore::open(dir.path().join("lifecycle.redb")).unwrap();
        assert_eq!(
            store.compare_and_advance([9; 32], None, anchor(2, 2)),
            Err(CheckpointStoreError::InvalidAdvance)
        );
    }

    #[test]
    fn independent_witness_detects_missing_or_rolled_back_local_head() {
        #[derive(Default)]
        struct Witness(Mutex<BTreeMap<[u8; 32], LifecycleCheckpointAnchor>>);
        impl ExternalLifecycleRollbackWitness for Witness {
            fn trusted_head(
                &self,
                network_id: [u8; 32],
            ) -> Result<Option<LifecycleCheckpointAnchor>, RollbackWitnessError> {
                Ok(self.0.lock().unwrap().get(&network_id).copied())
            }
        }

        let dir = tempfile::tempdir().unwrap();
        let store = RedbLifecycleCheckpointStore::open(dir.path().join("lifecycle.redb")).unwrap();
        let witness = Witness::default();
        let network = [10; 32];
        assert_eq!(
            verify_external_rollback_witness(&store, &witness, network),
            Ok(())
        );
        witness.0.lock().unwrap().insert(network, anchor(3, 3));
        assert_eq!(
            verify_external_rollback_witness(&store, &witness, network),
            Err(RollbackWitnessError::HeadMismatch)
        );
    }
}
