//! Durable single-process signing-intent reservations backed by redb.
//! Stores intent bytes only; it does not hold keys or sign messages.
use apex_signer_safety::{ReservationOutcome, SigningIntent, SigningSafetyError, SigningSafetyStore};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use std::path::Path;

const SCHEMA_VERSION: u64 = 1;
const META: TableDefinition<u8, u64> = TableDefinition::new("apex_signer_meta_v1");
const RESERVATIONS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("apex_signing_reservations_v1");

pub struct RedbSigningSafetyStore { db: Database }

impl RedbSigningSafetyStore {
    /// Opens or initializes the journal; incompatible or partial metadata fails closed.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, SigningSafetyError> {
        let db = Database::create(path).map_err(|_| SigningSafetyError::StorageUnavailable)?;
        let write = db.begin_write().map_err(|_| SigningSafetyError::StorageUnavailable)?;
        {
            let mut meta = write.open_table(META).map_err(|_| SigningSafetyError::StorageUnavailable)?;
            let version = meta.get(0).map_err(|_| SigningSafetyError::StorageUnavailable)?.map(|v| v.value());
            match version {
                Some(SCHEMA_VERSION) => {}
                Some(_) => return Err(SigningSafetyError::UnsupportedSchema),
                None => {
                    if meta.iter().map_err(|_| SigningSafetyError::StorageUnavailable)?.next().is_some() {
                        return Err(SigningSafetyError::CorruptStorage);
                    }
                    meta.insert(0, SCHEMA_VERSION).map_err(|_| SigningSafetyError::StorageUnavailable)?;
                }
            }
        }
        { let _ = write.open_table(RESERVATIONS).map_err(|_| SigningSafetyError::StorageUnavailable)?; }
        write.commit().map_err(|_| SigningSafetyError::StorageUnavailable)?;
        Ok(Self { db })
    }
}

impl SigningSafetyStore for RedbSigningSafetyStore {
    fn reserve(&mut self, intent: &SigningIntent) -> Result<ReservationOutcome, SigningSafetyError> {
        let write = self.db.begin_write().map_err(|_| SigningSafetyError::StorageUnavailable)?;
        let key = intent.coordinate().storage_key();
        let outcome = {
            let mut table = write.open_table(RESERVATIONS).map_err(|_| SigningSafetyError::StorageUnavailable)?;
            match table.get(key.as_slice()).map_err(|_| SigningSafetyError::StorageUnavailable)? {
                Some(existing) if existing.value() == intent.message() => ReservationOutcome::ExistingIdentical,
                Some(_) => return Err(SigningSafetyError::DoubleSignPrevented),
                None => {
                    table.insert(key.as_slice(), intent.message()).map_err(|_| SigningSafetyError::StorageUnavailable)?;
                    ReservationOutcome::Reserved
                }
            }
        };
        // The transaction is committed before Reserved is returned to the signer caller.
        write.commit().map_err(|_| SigningSafetyError::StorageUnavailable)?;
        Ok(outcome)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apex_signer_safety::{ConsensusStep, SignerId, SigningCoordinate};
    use apex_types::ChainId256;
    fn intent(message: &[u8]) -> SigningIntent {
        let signer = SignerId::new([3; 32]).unwrap();
        let coordinate = SigningCoordinate::new(ChainId256::from_u64(77), 1, 12, 0,
            ConsensusStep::Prevote, signer).unwrap();
        SigningIntent::new(coordinate, message.to_vec(), 4096).unwrap()
    }
    #[test]
    fn reservation_survives_reopen_and_conflicts_fail_closed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("signer.redb");
        {
            let mut store = RedbSigningSafetyStore::open(&path).unwrap();
            assert_eq!(store.reserve(&intent(b"vote-a")), Ok(ReservationOutcome::Reserved));
        }
        {
            let mut store = RedbSigningSafetyStore::open(&path).unwrap();
            assert_eq!(store.reserve(&intent(b"vote-a")), Ok(ReservationOutcome::ExistingIdentical));
            assert_eq!(store.reserve(&intent(b"vote-b")), Err(SigningSafetyError::DoubleSignPrevented));
        }
    }
    #[test]
    fn independent_coordinates_do_not_conflict() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("signer.redb");
        let mut store = RedbSigningSafetyStore::open(&path).unwrap();
        assert_eq!(store.reserve(&intent(b"vote-a")), Ok(ReservationOutcome::Reserved));
        let coordinate = SigningCoordinate::new(ChainId256::from_u64(77), 1, 13, 0,
            ConsensusStep::Prevote, SignerId::new([3; 32]).unwrap()).unwrap();
        let next = SigningIntent::new(coordinate, b"vote-b".to_vec(), 4096).unwrap();
        assert_eq!(store.reserve(&next), Ok(ReservationOutcome::Reserved));
    }
}
