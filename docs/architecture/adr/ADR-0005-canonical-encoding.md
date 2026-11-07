# ADR-0005: Canonical Protocol Encoding

- **Status:** Accepted for the account-record v1 prototype only; general protocol codec remains open.
- **Date:** 2026-11-07

## Decision

Define one strict fixed-width encoding for the current prototype account record:

| Field | Width | Encoding |
|---|---:|---|
| Magic | 4 bytes | ASCII `APEX` |
| Schema version | 2 bytes | unsigned big-endian; value 1 |
| AccountId | 8 bytes | unsigned big-endian |
| Nonce | 8 bytes | unsigned big-endian |
| Balance | 16 bytes | unsigned big-endian |

Total length: 38 bytes. Decoder MUST reject truncated input, trailing bytes, wrong magic and unknown schema version. No optional fields or alternate encodings exist in v1.

## Scope limitation

This decision does not specify transaction, block, validator, signature, EVM storage, code, receipt or genesis encodings. AccountId remains a u64 prototype identifier. Serde serialization and the redb private storage layout are not protocol encodings.

## Rationale

Fixed-width fields, explicit endianness and exact-length decoding reduce ambiguity for the small logical account model currently implemented. This is deliberately narrower than choosing a general-purpose serialization format.

## Upgrade policy

A new incompatible account-record encoding requires a new schema version and explicit protocol activation decision. Unknown versions fail closed. A storage-engine migration is not permission to change protocol bytes.

## Required follow-up

Cross-language vectors, fuzzing of malformed/truncated/oversized input, code review, and expansion of the logical state model are required before this becomes a production protocol commitment. ADR remains open for general transaction and block encoding.
