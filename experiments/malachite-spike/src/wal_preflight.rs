//! Strict, read-only scanner for the pinned Malachite v0.8.2 WAL framing.
//!
//! This module accepts an already-preserved byte slice. It never opens, locks,
//! truncates, repairs, or writes a WAL path. Compressed records are framed but
//! deliberately not decoded: their CRC covers decompressed bytes, so they are
//! reported as unverified rather than treated as valid.

use sha2::{Digest, Sha256};

const HEADER_LEN: usize = 12;
const ENTRY_HEADER_LEN: usize = 13;
const MAX_ENTRY_SIZE: u64 = 1024 * 1024 * 1024;
const WAL_VERSION_V1: u32 = 1;

/// Result of scanning a preserved WAL byte snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalPreflightReport {
    /// SHA-256 of the complete input snapshot, including damaged trailing bytes.
    pub snapshot_sha256: [u8; 32],
    pub snapshot_bytes: u64,
    pub version: Option<u32>,
    pub sequence: Option<u64>,
    pub entries: Vec<WalEntryReport>,
    pub issues: Vec<WalScanIssue>,
}

impl WalPreflightReport {
    /// True only when framing is complete and every record CRC was verified.
    pub fn is_fully_verified(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Metadata for one framed entry. Offsets are byte offsets in the original snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalEntryReport {
    pub index: u64,
    pub offset: u64,
    pub compressed: bool,
    pub payload_bytes: u64,
    pub expected_crc32: u32,
    pub crc_verified: bool,
}

/// A conservative finding. Scanning stops at structural ambiguity, but the
/// report hash always commits to the entire original input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WalScanIssue {
    EmptySnapshot,
    IncompleteHeader { available_bytes: u64 },
    UnsupportedVersion { version: u32 },
    IncompleteEntryHeader { offset: u64, available_bytes: u64 },
    EntryTooLarge { offset: u64, declared_bytes: u64 },
    IncompleteEntryPayload { offset: u64, declared_bytes: u64, available_bytes: u64 },
    CrcMismatch { offset: u64, expected: u32, actual: u32 },
    CompressionNotVerified { offset: u64 },
}

/// Scan an immutable snapshot using the pinned upstream v1 framing.
///
/// The caller is responsible for acquiring the bytes without first invoking
/// Malachite's mutating `Log::open` or default engine replay path.
pub fn scan_wal_snapshot(bytes: &[u8]) -> WalPreflightReport {
    let snapshot_sha256: [u8; 32] = Sha256::digest(bytes).into();
    let mut report = WalPreflightReport {
        snapshot_sha256,
        snapshot_bytes: bytes.len() as u64,
        version: None,
        sequence: None,
        entries: Vec::new(),
        issues: Vec::new(),
    };

    if bytes.is_empty() {
        report.issues.push(WalScanIssue::EmptySnapshot);
        return report;
    }
    if bytes.len() < HEADER_LEN {
        report.issues.push(WalScanIssue::IncompleteHeader {
            available_bytes: bytes.len() as u64,
        });
        return report;
    }

    let version = u32::from_be_bytes(bytes[0..4].try_into().expect("fixed header slice"));
    let sequence = u64::from_be_bytes(bytes[4..12].try_into().expect("fixed header slice"));
    report.version = Some(version);
    report.sequence = Some(sequence);
    if version != WAL_VERSION_V1 {
        report.issues.push(WalScanIssue::UnsupportedVersion { version });
        return report;
    }

    let mut offset = HEADER_LEN;
    while offset < bytes.len() {
        let remaining = bytes.len() - offset;
        if remaining < ENTRY_HEADER_LEN {
            report.issues.push(WalScanIssue::IncompleteEntryHeader {
                offset: offset as u64,
                available_bytes: remaining as u64,
            });
            break;
        }

        // Malachite v0.8.2 interprets any nonzero compression byte as true.
        let compressed = bytes[offset] != 0;
        let payload_len = u64::from_be_bytes(
            bytes[offset + 1..offset + 9].try_into().expect("fixed entry length slice"),
        );
        let expected_crc = u32::from_be_bytes(
            bytes[offset + 9..offset + 13].try_into().expect("fixed entry CRC slice"),
        );

        if payload_len > MAX_ENTRY_SIZE {
            report.issues.push(WalScanIssue::EntryTooLarge {
                offset: offset as u64,
                declared_bytes: payload_len,
            });
            break;
        }

        let payload_start = offset + ENTRY_HEADER_LEN;
        let Ok(payload_len_usize) = usize::try_from(payload_len) else {
            report.issues.push(WalScanIssue::EntryTooLarge {
                offset: offset as u64,
                declared_bytes: payload_len,
            });
            break;
        };
        let Some(payload_end) = payload_start.checked_add(payload_len_usize) else {
            report.issues.push(WalScanIssue::EntryTooLarge {
                offset: offset as u64,
                declared_bytes: payload_len,
            });
            break;
        };
        if payload_end > bytes.len() {
            report.issues.push(WalScanIssue::IncompleteEntryPayload {
                offset: offset as u64,
                declared_bytes: payload_len,
                available_bytes: (bytes.len() - payload_start) as u64,
            });
            break;
        }

        let (crc_verified, actual_crc) = if compressed {
            report.issues.push(WalScanIssue::CompressionNotVerified {
                offset: offset as u64,
            });
            (false, None)
        } else {
            (true, Some(crc32(&bytes[payload_start..payload_end])))
        };

        report.entries.push(WalEntryReport {
            index: report.entries.len() as u64,
            offset: offset as u64,
            compressed,
            payload_bytes: payload_len,
            expected_crc32: expected_crc,
            crc_verified: crc_verified && actual_crc == Some(expected_crc),
        });

        if let Some(actual) = actual_crc {
            if actual != expected_crc {
                report.issues.push(WalScanIssue::CrcMismatch {
                    offset: offset as u64,
                    expected: expected_crc,
                    actual,
                });
                break;
            }
        }
        offset = payload_end;
    }

    report
}

/// IEEE CRC-32, matching crc32fast's standard polynomial and initialization.
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(sequence: u64) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&WAL_VERSION_V1.to_be_bytes());
        bytes.extend_from_slice(&sequence.to_be_bytes());
        bytes
    }

    fn append_entry(bytes: &mut Vec<u8>, compressed: u8, payload: &[u8], crc: u32) {
        bytes.push(compressed);
        bytes.extend_from_slice(&(payload.len() as u64).to_be_bytes());
        bytes.extend_from_slice(&crc.to_be_bytes());
        bytes.extend_from_slice(payload);
    }

    #[test]
    fn empty_and_incomplete_headers_are_reported_without_mutation() {
        assert_eq!(scan_wal_snapshot(&[]).issues, vec![WalScanIssue::EmptySnapshot]);
        let input = vec![0, 0, 0, 1, 0, 0];
        let before = input.clone();
        let report = scan_wal_snapshot(&input);
        assert_eq!(report.issues, vec![WalScanIssue::IncompleteHeader { available_bytes: 6 }]);
        assert_eq!(input, before);
        assert_eq!(report.snapshot_sha256, <[u8; 32]>::from(Sha256::digest(&input)));
    }

    #[test]
    fn header_only_and_multiple_uncompressed_entries_verify() {
        let mut bytes = header(9);
        let empty_report = scan_wal_snapshot(&bytes);
        assert!(empty_report.is_fully_verified());
        assert_eq!(empty_report.sequence, Some(9));

        append_entry(&mut bytes, 0, b"vote", crc32(b"vote"));
        append_entry(&mut bytes, 0, b"proposal", crc32(b"proposal"));
        let report = scan_wal_snapshot(&bytes);
        assert!(report.is_fully_verified());
        assert_eq!(report.entries.len(), 2);
        assert_eq!(report.entries[0].offset, HEADER_LEN as u64);
        assert_eq!(report.entries[1].index, 1);
        assert!(report.entries.iter().all(|entry| entry.crc_verified));
    }

    #[test]
    fn partial_entry_header_and_payload_are_not_repaired() {
        let mut header_tail = header(1);
        header_tail.extend_from_slice(&[0, 0, 0]);
        assert!(matches!(
            scan_wal_snapshot(&header_tail).issues[0],
            WalScanIssue::IncompleteEntryHeader { .. }
        ));

        let mut payload_tail = header(1);
        payload_tail.push(0);
        payload_tail.extend_from_slice(&5_u64.to_be_bytes());
        payload_tail.extend_from_slice(&0_u32.to_be_bytes());
        payload_tail.extend_from_slice(b"xy");
        assert!(matches!(
            scan_wal_snapshot(&payload_tail).issues[0],
            WalScanIssue::IncompleteEntryPayload { declared_bytes: 5, available_bytes: 2, .. }
        ));
    }

    #[test]
    fn crc_mismatch_and_oversized_entry_fail_closed() {
        let mut corrupt = header(2);
        append_entry(&mut corrupt, 0, b"vote", 123);
        assert!(matches!(
            scan_wal_snapshot(&corrupt).issues[0],
            WalScanIssue::CrcMismatch { .. }
        ));

        let mut oversized = header(2);
        oversized.push(0);
        oversized.extend_from_slice(&(MAX_ENTRY_SIZE + 1).to_be_bytes());
        oversized.extend_from_slice(&0_u32.to_be_bytes());
        assert!(matches!(
            scan_wal_snapshot(&oversized).issues[0],
            WalScanIssue::EntryTooLarge { .. }
        ));
    }

    #[test]
    fn compressed_entry_is_framed_but_never_claimed_crc_verified() {
        let mut bytes = header(3);
        append_entry(&mut bytes, 1, b"compressed", 0);
        append_entry(&mut bytes, 0, b"next", crc32(b"next"));
        let report = scan_wal_snapshot(&bytes);
        assert_eq!(report.entries.len(), 2);
        assert!(!report.entries[0].crc_verified);
        assert!(report.entries[1].crc_verified);
        assert_eq!(report.issues, vec![WalScanIssue::CompressionNotVerified { offset: 12 }]);
        assert!(!report.is_fully_verified());
    }

    #[test]
    fn unsupported_version_is_not_interpreted_as_v1() {
        let mut bytes = header(0);
        bytes[3] = 2;
        assert_eq!(
            scan_wal_snapshot(&bytes).issues,
            vec![WalScanIssue::UnsupportedVersion { version: 2 }]
        );
    }
}
