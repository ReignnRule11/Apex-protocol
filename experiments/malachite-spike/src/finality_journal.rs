//! Isolated redb-backed finality journal prototype.
//!
//! The journal atomically stores a finalized identity, opaque application-state
//! payload, and chain tip in one redb write transaction. It is research-only:
//! callers remain responsible for supplying a fully verified identity and a
//! complete deterministic state snapshot/delta.

use std::path::Path;

use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};

use crate::native_finality_bridge::{ApexBlockIdentity, FinalityCommitKey};

const JOURNAL: TableDefinition<&[u8], &[u8]> = TableDefinition::new("apex_finality_journal_v1");
const TIPS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("apex_finality_tips_v1");
const IDENTITY_LEN: usize = 108;
const HEIGHT_KEY_LEN: usize = 44;
const CHAIN_KEY_LEN: usize = 36;

/// Result of a durable finality journal operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JournalCommit {
    /// The identity and state payload were atomically persisted.
    Committed,
    /// The exact identity and payload were already durably present.
    AlreadyCommitted,
}

/// Fail-closed journal errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JournalError {
    InvalidIdentity,
    PayloadTooLarge,
    DatabaseUnavailable,
    /// The write transaction's commit returned an error; outcome is uncertain.
    Ambiguous,
    ConflictingFinality,
    ConflictingPayload,
    HeightDiscontinuity,
    ParentMismatch,
    CorruptJournal,
}

/// A bounded, single-process redb journal. This type does not provide
/// multi-process coordination, backup rollback protection, or power-loss proof.
pub struct FinalityJournal {
    database: Database,
    max_payload_bytes: usize,
}

impl FinalityJournal {
    pub fn create(path: impl AsRef<Path>, max_payload_bytes: usize) -> Result<Self, JournalError> {
        let database = Database::create(path).map_err(|_| JournalError::DatabaseUnavailable)?;
        Ok(Self { database, max_payload_bytes })
    }

    pub fn open(path: impl AsRef<Path>, max_payload_bytes: usize) -> Result<Self, JournalError> {
        let database = Database::open(path).map_err(|_| JournalError::DatabaseUnavailable)?;
        Ok(Self { database, max_payload_bytes })
    }

    /// Atomically append one finalized state payload and advance the chain tip.
    ///
    /// The identity comes from the opaque verified-token commit key. The caller
    /// must pass the complete canonical application state needed for recovery.
    pub fn commit(
        &self,
        key: FinalityCommitKey,
        state_payload: &[u8],
    ) -> Result<JournalCommit, JournalError> {
        let identity = key.identity();
        identity.validate().map_err(|_| JournalError::InvalidIdentity)?;
        if state_payload.len() > self.max_payload_bytes {
            return Err(JournalError::PayloadTooLarge);
        }

        let encoded_identity = encode_identity(identity);
        let height_key = encode_height_key(identity);
        let chain_key = encode_chain_key(identity);

        let write_txn = self.database.begin_write().map_err(|_| JournalError::DatabaseUnavailable)?;
        {
            let mut journal = write_txn.open_table(JOURNAL).map_err(|_| JournalError::DatabaseUnavailable)?;
            let mut tips = write_txn.open_table(TIPS).map_err(|_| JournalError::DatabaseUnavailable)?;

            if let Some(existing) = journal.get(height_key.as_slice()).map_err(|_| JournalError::DatabaseUnavailable)? {
                let existing = existing.value();
                if existing.len() < IDENTITY_LEN {
                    return Err(JournalError::CorruptJournal);
                }
                if existing[..IDENTITY_LEN] != encoded_identity {
                    return Err(JournalError::ConflictingFinality);
                }
                if existing[IDENTITY_LEN..] != *state_payload {
                    return Err(JournalError::ConflictingPayload);
                }
                return Ok(JournalCommit::AlreadyCommitted);
            }

            if let Some(tip) = tips.get(chain_key.as_slice()).map_err(|_| JournalError::DatabaseUnavailable)? {
                let previous = decode_identity(tip.value())?;
                let expected_height = previous.height.checked_add(1).ok_or(JournalError::HeightDiscontinuity)?;
                if identity.height != expected_height {
                    return Err(JournalError::HeightDiscontinuity);
                }
                if identity.parent_hash != previous.block_hash {
                    return Err(JournalError::ParentMismatch);
                }
            }

            let mut record = Vec::with_capacity(IDENTITY_LEN + state_payload.len());
            record.extend_from_slice(&encoded_identity);
            record.extend_from_slice(state_payload);
            journal.insert(height_key.as_slice(), record.as_slice()).map_err(|_| JournalError::DatabaseUnavailable)?;
            tips.insert(chain_key.as_slice(), encoded_identity.as_slice()).map_err(|_| JournalError::DatabaseUnavailable)?;
        }

        write_txn.commit().map_err(|_| JournalError::Ambiguous)?;
        Ok(JournalCommit::Committed)
    }

    /// Return the exact stored payload for a chain height, if present.
    pub fn read_payload(&self, identity: ApexBlockIdentity) -> Result<Option<Vec<u8>>, JournalError> {
        identity.validate().map_err(|_| JournalError::InvalidIdentity)?;
        let read_txn = self.database.begin_read().map_err(|_| JournalError::DatabaseUnavailable)?;
        let journal = read_txn.open_table(JOURNAL).map_err(|_| JournalError::DatabaseUnavailable)?;
        let height_key = encode_height_key(identity);
        let Some(record) = journal.get(height_key.as_slice()).map_err(|_| JournalError::DatabaseUnavailable)? else {
            return Ok(None);
        };
        let bytes = record.value();
        if bytes.len() < IDENTITY_LEN {
            return Err(JournalError::CorruptJournal);
        }
        if bytes[..IDENTITY_LEN] != encode_identity(identity) {
            return Err(JournalError::ConflictingFinality);
        }
        Ok(Some(bytes[IDENTITY_LEN..].to_vec()))
    }

    /// Return the latest committed identity for the chain, if one exists.
    pub fn tip(&self, chain_id: [u8; 32], protocol_version: u32) -> Result<Option<ApexBlockIdentity>, JournalError> {
        if chain_id == [0; 32] || protocol_version == 0 {
            return Err(JournalError::InvalidIdentity);
        }
        let read_txn = self.database.begin_read().map_err(|_| JournalError::DatabaseUnavailable)?;
        let tips = read_txn.open_table(TIPS).map_err(|_| JournalError::DatabaseUnavailable)?;
        let mut key = Vec::with_capacity(CHAIN_KEY_LEN);
        key.extend_from_slice(&chain_id);
        key.extend_from_slice(&protocol_version.to_be_bytes());
        match tips.get(key.as_slice()).map_err(|_| JournalError::DatabaseUnavailable)? {
            Some(value) => Ok(Some(decode_identity(value.value())?)),
            None => Ok(None),
        }
    }
}

fn encode_identity(identity: ApexBlockIdentity) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(IDENTITY_LEN);
    bytes.extend_from_slice(&identity.chain_id);
    bytes.extend_from_slice(&identity.protocol_version.to_be_bytes());
    bytes.extend_from_slice(&identity.height.to_be_bytes());
    bytes.extend_from_slice(&identity.parent_hash);
    bytes.extend_from_slice(&identity.block_hash);
    bytes
}

fn decode_identity(bytes: &[u8]) -> Result<ApexBlockIdentity, JournalError> {
    if bytes.len() != IDENTITY_LEN {
        return Err(JournalError::CorruptJournal);
    }
    let mut chain_id = [0; 32];
    chain_id.copy_from_slice(&bytes[..32]);
    let protocol_version = u32::from_be_bytes(bytes[32..36].try_into().map_err(|_| JournalError::CorruptJournal)?);
    let height = u64::from_be_bytes(bytes[36..44].try_into().map_err(|_| JournalError::CorruptJournal)?);
    let mut parent_hash = [0; 32];
    parent_hash.copy_from_slice(&bytes[44..76]);
    let mut block_hash = [0; 32];
    block_hash.copy_from_slice(&bytes[76..108]);
    let identity = ApexBlockIdentity { chain_id, protocol_version, height, parent_hash, block_hash };
    identity.validate().map_err(|_| JournalError::CorruptJournal)?;
    Ok(identity)
}

fn encode_height_key(identity: ApexBlockIdentity) -> Vec<u8> {
    let mut key = Vec::with_capacity(HEIGHT_KEY_LEN);
    key.extend_from_slice(&identity.chain_id);
    key.extend_from_slice(&identity.protocol_version.to_be_bytes());
    key.extend_from_slice(&identity.height.to_be_bytes());
    key
}

fn encode_chain_key(identity: ApexBlockIdentity) -> Vec<u8> {
    let mut key = Vec::with_capacity(CHAIN_KEY_LEN);
    key.extend_from_slice(&identity.chain_id);
    key.extend_from_slice(&identity.protocol_version.to_be_bytes());
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_finality_bridge::VerifiedNativeFinality;
    use tempfile::NamedTempFile;

    fn verified(height: u64, parent_hash: [u8; 32], block_hash: [u8; 32]) -> FinalityCommitKey {
        VerifiedNativeFinality::test_only(ApexBlockIdentity {
            chain_id: [1; 32], protocol_version: 1, height, parent_hash, block_hash,
        }).commit_key()
    }

    #[test]
    fn exact_retry_is_idempotent_and_payload_is_recoverable_after_reopen() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        let journal = FinalityJournal::create(path, 1024).unwrap();
        let key = verified(1, [0; 32], [2; 32]);
        assert_eq!(journal.commit(key, b"state-one"), Ok(JournalCommit::Committed));
        assert_eq!(journal.commit(key, b"state-one"), Ok(JournalCommit::AlreadyCommitted));
        drop(journal);

        let reopened = FinalityJournal::open(path, 1024).unwrap();
        assert_eq!(reopened.read_payload(key.identity()), Ok(Some(b"state-one".to_vec())));
        assert_eq!(reopened.tip([1; 32], 1), Ok(Some(key.identity())));
    }

    #[test]
    fn different_identity_at_same_height_fails_closed() {
        let file = NamedTempFile::new().unwrap();
        let journal = FinalityJournal::create(file.path(), 1024).unwrap();
        assert_eq!(journal.commit(verified(1, [0; 32], [2; 32]), b"a"), Ok(JournalCommit::Committed));
        assert_eq!(journal.commit(verified(1, [0; 32], [3; 32]), b"b"), Err(JournalError::ConflictingFinality));
    }

    #[test]
    fn next_height_requires_parent_linkage() {
        let file = NamedTempFile::new().unwrap();
        let journal = FinalityJournal::create(file.path(), 1024).unwrap();
        assert_eq!(journal.commit(verified(1, [0; 32], [2; 32]), b"a"), Ok(JournalCommit::Committed));
        assert_eq!(journal.commit(verified(2, [9; 32], [3; 32]), b"b"), Err(JournalError::ParentMismatch));
        assert_eq!(journal.commit(verified(2, [2; 32], [3; 32]), b"b"), Ok(JournalCommit::Committed));
    }

    #[test]
    fn rejects_oversized_payload_and_height_gap() {
        let file = NamedTempFile::new().unwrap();
        let journal = FinalityJournal::create(file.path(), 4).unwrap();
        assert_eq!(journal.commit(verified(1, [0; 32], [2; 32]), b"12345"), Err(JournalError::PayloadTooLarge));
        assert_eq!(journal.commit(verified(2, [0; 32], [3; 32]), b"ok"), Err(JournalError::HeightDiscontinuity));
    }
}
