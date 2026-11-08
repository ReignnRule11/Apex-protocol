//! Exact redb 4.5.0 read-only inspection spike for APEX signer/finality fixtures.
//! Diagnostic only: this module never creates, migrates, repairs, or writes a source DB.
use redb::{ReadOnlyDatabase, ReadableDatabase, ReadableTable, TableDefinition};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};

const META: TableDefinition<u8, u64> = TableDefinition::new("apex_signer_meta_v1");
const RESERVATIONS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("apex_signing_reservations_v1");
const JOURNAL: TableDefinition<&[u8], &[u8]> = TableDefinition::new("apex_finality_journal_v1");
const TIPS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("apex_finality_tips_v1");
const SIGNER_SCHEMA: u64 = 1;
const SIGNER_KEY_LEN: usize = 81;
const FINALITY_KEY_LEN: usize = 44;
const IDENTITY_LEN: usize = 108;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InspectionError {
    Io, SourceTooLarge, DatabaseOpen, MissingTable, UnsupportedSchema,
    CorruptMetadata, RecordLimit, ByteLimit, InvalidSignerRecord,
    InvalidFinalityRecord, FinalityGap, ParentMismatch, TipMismatch,
    MissingTip, UnexpectedTip, SourceChanged,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectionReport {
    pub kind: &'static str,
    pub source_bytes: u64,
    pub source_sha256: [u8; 32],
    pub records: u64,
    pub record_bytes: u64,
    pub report_sha256: [u8; 32],
    /// False for the current finality schema, which has no explicit schema marker.
    pub schema_explicitly_versioned: bool,
}

fn digest(bytes: &[u8]) -> [u8; 32] { Sha256::digest(bytes).into() }

fn source_snapshot(path: &Path, max_source_bytes: u64) -> Result<(u64, [u8; 32]), InspectionError> {
    let bytes = fs::read(path).map_err(|_| InspectionError::Io)?;
    if bytes.len() as u64 > max_source_bytes { return Err(InspectionError::SourceTooLarge); }
    Ok((bytes.len() as u64, digest(&bytes)))
}

fn finish_report(path: &Path, kind: &'static str, before: (u64, [u8; 32]), records: u64,
    record_bytes: u64, schema_explicitly_versioned: bool) -> Result<InspectionReport, InspectionError> {
    let after = source_snapshot(path, before.0)?;
    if before != after { return Err(InspectionError::SourceChanged); }
    let mut canonical = Vec::new();
    canonical.extend_from_slice(b"APEX-REDB-READONLY-INSPECTION-V1");
    canonical.extend_from_slice(kind.as_bytes());
    canonical.extend_from_slice(&before.0.to_be_bytes());
    canonical.extend_from_slice(&before.1);
    canonical.extend_from_slice(&records.to_be_bytes());
    canonical.extend_from_slice(&record_bytes.to_be_bytes());
    canonical.push(u8::from(schema_explicitly_versioned));
    Ok(InspectionReport { kind, source_bytes: before.0, source_sha256: before.1,
        records, record_bytes, report_sha256: digest(&canonical), schema_explicitly_versioned })
}

/// Enumerate and validate the exact signer tables without invoking the mutating store opener.
pub fn inspect_signer(path: impl AsRef<Path>, max_source_bytes: u64, max_records: u64,
    max_record_bytes: u64) -> Result<InspectionReport, InspectionError> {
    let path = path.as_ref();
    let before = source_snapshot(path, max_source_bytes)?;
    let db = ReadOnlyDatabase::open(path).map_err(|_| InspectionError::DatabaseOpen)?;
    let txn = db.begin_read().map_err(|_| InspectionError::DatabaseOpen)?;
    let meta = txn.open_table(META).map_err(|_| InspectionError::MissingTable)?;
    let mut meta_iter = meta.iter().map_err(|_| InspectionError::CorruptMetadata)?;
    let first = meta_iter.next().ok_or(InspectionError::CorruptMetadata)?
        .map_err(|_| InspectionError::CorruptMetadata)?;
    if first.0.value() != 0 || first.1.value() != SIGNER_SCHEMA || meta_iter.next().is_some() {
        return Err(InspectionError::UnsupportedSchema);
    }
    let table = txn.open_table(RESERVATIONS).map_err(|_| InspectionError::MissingTable)?;
    let mut count = 0u64;
    let mut bytes = 0u64;
    for entry in table.iter().map_err(|_| InspectionError::InvalidSignerRecord)? {
        let (key, value) = entry.map_err(|_| InspectionError::InvalidSignerRecord)?;
        let key = key.value();
        let value = value.value();
        if key.len() != SIGNER_KEY_LEN || value.is_empty() || key[..32].iter().all(|b| *b == 0)
            || key[32..36] == [0; 4] || key[49..81].iter().all(|b| *b == 0)
            || !(1..=3).contains(&key[48]) {
            return Err(InspectionError::InvalidSignerRecord);
        }
        count = count.checked_add(1).ok_or(InspectionError::RecordLimit)?;
        bytes = bytes.checked_add((key.len() + value.len()) as u64).ok_or(InspectionError::ByteLimit)?;
        if count > max_records { return Err(InspectionError::RecordLimit); }
        if bytes > max_record_bytes { return Err(InspectionError::ByteLimit); }
    }
    drop(table); drop(txn); drop(db);
    finish_report(path, "signer", before, count, bytes, true)
}

/// Audit every finality row, key/identity binding, contiguous chain and stored tip.
pub fn inspect_finality(path: impl AsRef<Path>, max_source_bytes: u64, max_records: u64,
    max_record_bytes: u64, max_payload_bytes: usize) -> Result<InspectionReport, InspectionError> {
    let path = path.as_ref();
    let before = source_snapshot(path, max_source_bytes)?;
    let db = ReadOnlyDatabase::open(path).map_err(|_| InspectionError::DatabaseOpen)?;
    let txn = db.begin_read().map_err(|_| InspectionError::DatabaseOpen)?;
    let journal = txn.open_table(JOURNAL).map_err(|_| InspectionError::MissingTable)?;
    let tips = txn.open_table(TIPS).map_err(|_| InspectionError::MissingTable)?;
    let mut chains: BTreeMap<Vec<u8>, (u64, [u8; 32])> = BTreeMap::new();
    let mut count = 0u64;
    let mut bytes = 0u64;
    for entry in journal.iter().map_err(|_| InspectionError::InvalidFinalityRecord)? {
        let (key_guard, value_guard) = entry.map_err(|_| InspectionError::InvalidFinalityRecord)?;
        let key = key_guard.value();
        let value = value_guard.value();
        if key.len() != FINALITY_KEY_LEN || value.len() < IDENTITY_LEN
            || value.len() - IDENTITY_LEN > max_payload_bytes {
            return Err(InspectionError::InvalidFinalityRecord);
        }
        let chain = key[..36].to_vec();
        let height = u64::from_be_bytes(key[36..44].try_into().map_err(|_| InspectionError::InvalidFinalityRecord)?);
        let identity = &value[..IDENTITY_LEN];
        if identity[..32] != key[..32] || identity[32..36] != key[32..36]
            || identity[36..44] != key[36..44] || identity[..32].iter().all(|b| *b == 0)
            || identity[32..36] == [0; 4] || height == 0 {
            return Err(InspectionError::InvalidFinalityRecord);
        }
        let mut parent = [0u8; 32]; parent.copy_from_slice(&identity[44..76]);
        let mut block = [0u8; 32]; block.copy_from_slice(&identity[76..108]);
        let expected = match chains.get(&chain) {
            None if height == 1 && parent == [0; 32] => 1,
            None => return Err(InspectionError::FinalityGap),
            Some((last_height, last_block)) if height == last_height + 1 && parent == *last_block => height,
            Some((last_height, _)) if height != last_height + 1 => return Err(InspectionError::FinalityGap),
            Some(_) => return Err(InspectionError::ParentMismatch),
        };
        chains.insert(chain, (expected, block));
        count = count.checked_add(1).ok_or(InspectionError::RecordLimit)?;
        bytes = bytes.checked_add((key.len() + value.len()) as u64).ok_or(InspectionError::ByteLimit)?;
        if count > max_records { return Err(InspectionError::RecordLimit); }
        if bytes > max_record_bytes { return Err(InspectionError::ByteLimit); }
    }
    let mut seen_tips = 0usize;
    for entry in tips.iter().map_err(|_| InspectionError::InvalidFinalityRecord)? {
        let (key_guard, value_guard) = entry.map_err(|_| InspectionError::InvalidFinalityRecord)?;
        let key = key_guard.value();
        let value = value_guard.value();
        if key.len() != 36 || value.len() != IDENTITY_LEN { return Err(InspectionError::InvalidFinalityRecord); }
        let Some((height, block)) = chains.get(key) else { return Err(InspectionError::UnexpectedTip); };
        if value[..36] != key[..] || u64::from_be_bytes(value[36..44].try_into().map_err(|_| InspectionError::InvalidFinalityRecord)?) != *height
            || value[76..108] != block[..] { return Err(InspectionError::TipMismatch); }
        seen_tips += 1;
    }
    if seen_tips != chains.len() { return Err(InspectionError::MissingTip); }
    drop(tips); drop(journal); drop(txn); drop(db);
    finish_report(path, "finality", before, count, bytes, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use redb::{Database, TableDefinition};
    use tempfile::NamedTempFile;

    const TEST_META: TableDefinition<u8, u64> = TableDefinition::new("apex_signer_meta_v1");
    const TEST_RES: TableDefinition<&[u8], &[u8]> = TableDefinition::new("apex_signing_reservations_v1");
    const TEST_JOURNAL: TableDefinition<&[u8], &[u8]> = TableDefinition::new("apex_finality_journal_v1");
    const TEST_TIPS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("apex_finality_tips_v1");

    fn signer_fixture(path: &Path, valid: bool) {
        let db = Database::create(path).unwrap();
        let tx = db.begin_write().unwrap();
        {
            let mut meta = tx.open_table(TEST_META).unwrap();
            meta.insert(0, SIGNER_SCHEMA).unwrap();
        }
        {
            let mut rows = tx.open_table(TEST_RES).unwrap();
            let mut key = [0u8; SIGNER_KEY_LEN];
            key[31] = 7; key[35] = 1; key[43] = 1; key[48] = 2; key[80] = 9;
            if !valid { key[48] = 9; }
            rows.insert(key.as_slice(), b"canonical-vote".as_slice()).unwrap();
        }
        tx.commit().unwrap();
    }

    fn finality_fixture(path: &Path, valid: bool) {
        let db = Database::create(path).unwrap();
        let tx = db.begin_write().unwrap();
        {
            let mut journal = tx.open_table(TEST_JOURNAL).unwrap();
            let mut key = Vec::new();
            key.extend_from_slice(&[1; 32]); key.extend_from_slice(&1u32.to_be_bytes());
            key.extend_from_slice(&1u64.to_be_bytes());
            let mut value = Vec::new();
            value.extend_from_slice(&[1; 32]); value.extend_from_slice(&1u32.to_be_bytes());
            value.extend_from_slice(&1u64.to_be_bytes());
            value.extend_from_slice(&[0; 32]); value.extend_from_slice(&[2; 32]);
            value.extend_from_slice(b"state");
            if !valid { value[44] = 1; }
            journal.insert(key.as_slice(), value.as_slice()).unwrap();
        }
        {
            let mut tips = tx.open_table(TEST_TIPS).unwrap();
            let mut key = Vec::new(); key.extend_from_slice(&[1; 32]); key.extend_from_slice(&1u32.to_be_bytes());
            let mut value = Vec::new();
            value.extend_from_slice(&[1; 32]); value.extend_from_slice(&1u32.to_be_bytes());
            value.extend_from_slice(&1u64.to_be_bytes());
            value.extend_from_slice(&[0; 32]); value.extend_from_slice(&[2; 32]);
            tips.insert(key.as_slice(), value.as_slice()).unwrap();
        }
        tx.commit().unwrap();
    }

    #[test]
    fn signer_inspection_is_bounded_and_source_bytes_are_unchanged() {
        let file = NamedTempFile::new().unwrap();
        signer_fixture(file.path(), true);
        let before = fs::read(file.path()).unwrap();
        let report = inspect_signer(file.path(), 1_000_000, 10, 1_000).unwrap();
        assert_eq!(report.records, 1);
        assert!(report.schema_explicitly_versioned);
        assert_eq!(fs::read(file.path()).unwrap(), before);
        assert_eq!(inspect_signer(file.path(), 1_000_000, 0, 1_000), Err(InspectionError::RecordLimit));
    }

    #[test]
    fn signer_unknown_step_is_rejected_without_repair() {
        let file = NamedTempFile::new().unwrap();
        signer_fixture(file.path(), false);
        let before = fs::read(file.path()).unwrap();
        assert_eq!(inspect_signer(file.path(), 1_000_000, 10, 1_000), Err(InspectionError::InvalidSignerRecord));
        assert_eq!(fs::read(file.path()).unwrap(), before);
    }

    #[test]
    fn finality_inspection_checks_tip_and_does_not_mutate_source() {
        let file = NamedTempFile::new().unwrap();
        finality_fixture(file.path(), true);
        let before = fs::read(file.path()).unwrap();
        let report = inspect_finality(file.path(), 1_000_000, 10, 1_000, 100).unwrap();
        assert_eq!(report.records, 1);
        assert!(!report.schema_explicitly_versioned);
        assert_eq!(fs::read(file.path()).unwrap(), before);
    }

    #[test]
    fn finality_parent_or_identity_corruption_is_rejected() {
        let file = NamedTempFile::new().unwrap();
        finality_fixture(file.path(), false);
        let before = fs::read(file.path()).unwrap();
        assert_eq!(inspect_finality(file.path(), 1_000_000, 10, 1_000, 100), Err(InspectionError::InvalidFinalityRecord));
        assert_eq!(fs::read(file.path()).unwrap(), before);
    }
}
