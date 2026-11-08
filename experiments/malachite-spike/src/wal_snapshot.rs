//! Read-only acquisition of a quiesced Malachite WAL snapshot.
//!
//! This module deliberately does not claim to create an atomic filesystem
//! snapshot. The caller must stop all WAL writers and externally establish
//! exclusive quiescence before acknowledging the source. The opened file
//! descriptor is stamped before and after the bounded read; detectable
//! metadata drift rejects the acquisition.

use std::{
    fs::{File, OpenOptions},
    io::{self, Read},
    path::Path,
    time::SystemTime,
};

use sha2::{Digest, Sha256};

use crate::wal_preflight::{WalPreflightReport, scan_wal_snapshot};

/// Explicit caller attestation that all writers are stopped and the WAL will
/// remain quiescent for the duration of acquisition.
///
/// This is an operational assertion, not a lock, cryptographic proof, or
/// substitute for a storage-level snapshot primitive.
#[derive(Clone, Copy, Debug)]
pub struct QuiescedWalSource {
    _private: (),
}

impl QuiescedWalSource {
    /// Acknowledge that the caller has independently stopped all WAL writers.
    ///
    /// Do not call this while a Malachite engine or any other process can
    /// append, truncate, reset, replace, or otherwise mutate the source WAL.
    pub fn acknowledge_exclusive_quiescence() -> Self {
        Self { _private: () }
    }
}

/// Evidence retained from one bounded read of the opened WAL file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalSnapshot {
    bytes: Vec<u8>,
    sha256: [u8; 32],
}

impl WalSnapshot {
    /// Exact bytes read from the opened file descriptor.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// SHA-256 over every byte in this snapshot.
    pub fn sha256(&self) -> [u8; 32] {
        self.sha256
    }

    /// Number of bytes captured.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Run the strict framing scanner over the captured bytes.
    pub fn preflight(&self) -> WalPreflightReport {
        scan_wal_snapshot(&self.bytes)
    }
}

/// Fail-closed acquisition errors.
#[derive(Debug)]
pub enum SnapshotAcquisitionError {
    Io(io::Error),
    SnapshotTooLarge { observed_at_least: u64, max_bytes: u64 },
    SourceChangedDuringRead,
    LengthMismatch { metadata_bytes: u64, read_bytes: u64 },
}

impl From<io::Error> for SnapshotAcquisitionError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FileStamp {
    len: u64,
    modified: Option<SystemTime>,
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
    #[cfg(unix)]
    changed_seconds: i64,
    #[cfg(unix)]
    changed_nanoseconds: i64,
}

impl FileStamp {
    fn read(file: &File) -> io::Result<Self> {
        let metadata = file.metadata()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            Ok(Self {
                len: metadata.len(),
                modified: metadata.modified().ok(),
                device: metadata.dev(),
                inode: metadata.ino(),
                changed_seconds: metadata.ctime(),
                changed_nanoseconds: metadata.ctime_nsec(),
            })
        }
        #[cfg(not(unix))]
        {
            Ok(Self {
                len: metadata.len(),
                modified: metadata.modified().ok(),
            })
        }
    }
}

/// Acquire bytes from an already-quiesced WAL without invoking Malachite WAL
/// APIs. The caller supplies a hard byte limit to bound memory consumption.
///
/// The file is opened read-only. File identity, length, modification time and
/// (on Unix) device/inode/change-time are compared around the read. This
/// detects common replacement and mutation races, but cannot detect every
/// possible same-metadata rewrite; external writer quiescence remains
/// mandatory. No lock, truncate, repair, sync, or write is performed.
pub fn capture_quiesced_wal_snapshot(
    path: impl AsRef<Path>,
    _quiescence: QuiescedWalSource,
    max_bytes: u64,
) -> Result<WalSnapshot, SnapshotAcquisitionError> {
    let mut file = OpenOptions::new().read(true).open(path)?;
    let before = FileStamp::read(&file)?;
    if before.len > max_bytes {
        return Err(SnapshotAcquisitionError::SnapshotTooLarge {
            observed_at_least: before.len,
            max_bytes,
        });
    }

    let read_limit = max_bytes.saturating_add(1);
    let mut bytes = Vec::with_capacity(
        usize::try_from(before.len.min(max_bytes)).unwrap_or(0),
    );
    file.by_ref().take(read_limit).read_to_end(&mut bytes)?;

    if bytes.len() as u64 > max_bytes {
        return Err(SnapshotAcquisitionError::SnapshotTooLarge {
            observed_at_least: bytes.len() as u64,
            max_bytes,
        });
    }

    let after = FileStamp::read(&file)?;
    if before != after {
        return Err(SnapshotAcquisitionError::SourceChangedDuringRead);
    }
    if bytes.len() as u64 != after.len {
        return Err(SnapshotAcquisitionError::LengthMismatch {
            metadata_bytes: after.len,
            read_bytes: bytes.len() as u64,
        });
    }

    let sha256 = Sha256::digest(&bytes).into();
    Ok(WalSnapshot { bytes, sha256 })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn captures_read_only_bytes_and_preflights_them() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("wal");
        let mut source = File::create(&path).expect("create");
        source.write_all(&1_u32.to_be_bytes()).expect("version");
        source.write_all(&7_u64.to_be_bytes()).expect("sequence");
        source.sync_all().expect("sync test fixture");
        drop(source);

        let before = std::fs::read(&path).expect("read original");
        let snapshot = capture_quiesced_wal_snapshot(
            &path,
            QuiescedWalSource::acknowledge_exclusive_quiescence(),
            1024,
        )
        .expect("capture");
        assert_eq!(snapshot.bytes(), before);
        assert_eq!(snapshot.sha256(), <[u8; 32]>::from(Sha256::digest(&before)));
        assert!(snapshot.preflight().is_fully_verified());
        assert_eq!(std::fs::read(&path).expect("read after"), before);
    }

    #[test]
    fn rejects_source_larger_than_explicit_limit() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("wal");
        std::fs::write(&path, [0_u8; 16]).expect("fixture");
        let result = capture_quiesced_wal_snapshot(
            &path,
            QuiescedWalSource::acknowledge_exclusive_quiescence(),
            8,
        );
        assert!(matches!(
            result,
            Err(SnapshotAcquisitionError::SnapshotTooLarge {
                observed_at_least: 16,
                max_bytes: 8
            })
        ));
    }

    #[test]
    fn empty_snapshot_is_captured_but_preflight_rejects_it() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("wal");
        std::fs::write(&path, []).expect("fixture");
        let snapshot = capture_quiesced_wal_snapshot(
            &path,
            QuiescedWalSource::acknowledge_exclusive_quiescence(),
            8,
        )
        .expect("capture");
        assert!(snapshot.is_empty());
        assert!(!snapshot.preflight().is_fully_verified());
    }
}
