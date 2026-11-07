//! Versioned, deterministic protocol encodings.
//! This crate currently specifies only the bounded account-record prototype.

use thiserror::Error;

const ACCOUNT_MAGIC: [u8; 4] = *b"APEX";
const ACCOUNT_SCHEMA_VERSION: u16 = 1;
pub const CANONICAL_ACCOUNT_RECORD_V1_LEN: usize = 38;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccountRecordV1 {
    pub account_id: u64,
    pub nonce: u64,
    pub balance: u128,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum CodecError {
    #[error("account record has an invalid length")]
    InvalidLength,
    #[error("account record magic is invalid")]
    InvalidMagic,
    #[error("account record schema version is unsupported")]
    UnsupportedVersion,
}

/// Encodes magic[4] || schema_version:u16-BE || account_id:u64-BE ||
/// nonce:u64-BE || balance:u128-BE.
pub fn encode_account_record_v1(record: AccountRecordV1) -> [u8; CANONICAL_ACCOUNT_RECORD_V1_LEN] {
    let mut out = [0_u8; CANONICAL_ACCOUNT_RECORD_V1_LEN];
    out[..4].copy_from_slice(&ACCOUNT_MAGIC);
    out[4..6].copy_from_slice(&ACCOUNT_SCHEMA_VERSION.to_be_bytes());
    out[6..14].copy_from_slice(&record.account_id.to_be_bytes());
    out[14..22].copy_from_slice(&record.nonce.to_be_bytes());
    out[22..38].copy_from_slice(&record.balance.to_be_bytes());
    out
}

/// Strictly decodes one complete account-record v1; trailing bytes are rejected.
pub fn decode_account_record_v1(bytes: &[u8]) -> Result<AccountRecordV1, CodecError> {
    if bytes.len() != CANONICAL_ACCOUNT_RECORD_V1_LEN {
        return Err(CodecError::InvalidLength);
    }
    if bytes[..4] != ACCOUNT_MAGIC {
        return Err(CodecError::InvalidMagic);
    }
    let version = u16::from_be_bytes(bytes[4..6].try_into().expect("fixed slice"));
    if version != ACCOUNT_SCHEMA_VERSION {
        return Err(CodecError::UnsupportedVersion);
    }
    Ok(AccountRecordV1 {
        account_id: u64::from_be_bytes(bytes[6..14].try_into().expect("fixed slice")),
        nonce: u64::from_be_bytes(bytes[14..22].try_into().expect("fixed slice")),
        balance: u128::from_be_bytes(bytes[22..38].try_into().expect("fixed slice")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_record_v1_round_trips_fixed_width_values() {
        let record = AccountRecordV1 {
            account_id: u64::MAX,
            nonce: u64::MAX - 1,
            balance: u128::MAX,
        };
        let encoded = encode_account_record_v1(record);
        assert_eq!(encoded.len(), 38);
        assert_eq!(decode_account_record_v1(&encoded), Ok(record));
    }

    #[test]
    fn encoding_is_big_endian_and_versioned() {
        let encoded = encode_account_record_v1(AccountRecordV1 {
            account_id: 0x0102_0304_0506_0708,
            nonce: 9,
            balance: 10,
        });
        assert_eq!(&encoded[..6], &[65, 80, 69, 88, 0, 1]);
        assert_eq!(&encoded[6..14], &[1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn decoder_rejects_trailing_truncated_and_unknown_records() {
        let encoded = encode_account_record_v1(AccountRecordV1 {
            account_id: 1,
            nonce: 2,
            balance: 3,
        });
        let mut trailing = encoded.to_vec();
        trailing.push(0);
        assert_eq!(
            decode_account_record_v1(&trailing),
            Err(CodecError::InvalidLength)
        );
        assert_eq!(
            decode_account_record_v1(&encoded[..37]),
            Err(CodecError::InvalidLength)
        );
        let mut wrong_magic = encoded;
        wrong_magic[0] = 0;
        assert_eq!(
            decode_account_record_v1(&wrong_magic),
            Err(CodecError::InvalidMagic)
        );
        let mut wrong_version = encoded;
        wrong_version[5] = 2;
        assert_eq!(
            decode_account_record_v1(&wrong_version),
            Err(CodecError::UnsupportedVersion)
        );
    }
}
