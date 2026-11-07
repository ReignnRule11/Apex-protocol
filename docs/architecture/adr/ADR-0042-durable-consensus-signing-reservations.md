# ADR-0042: Durable Consensus Signing Reservations

**Status:** Accepted for prototype only; production signer NOT approved.
**Date:** 2026-11-07

## Context

Consensus signing safety requires that a validator never signs conflicting messages for the same chain, protocol version, height, round, step and signer identity. The application boundary in Phase 20C does not provide signer anti-double-sign or restart safety. Consensus algorithms and signing domains remain unratified.

## Decision

Introduce an engine-neutral signing safety store. A caller must reserve exact canonical message bytes under a complete signing coordinate before invoking an external signing backend. The store permits exact-byte retries and rejects different bytes at the same coordinate. The redb adapter commits the reservation transaction before returning success.

The storage key is a fixed-width 81-byte encoding: 32-byte chain ID, 4-byte protocol version, 8-byte height, 4-byte round, 1-byte step, and 32-byte signer ID. Numeric fields are big-endian.

## Deliberate non-decisions

This ADR does not choose Ed25519, BLS, secp256k1 or another suite; does not define key derivation, public-key identity, certificate formats, canonical message encoding, or Malachite signing semantics. It does not authorize production validator keys.

## Consequences and residual risks

A crash after durable reservation but before signing can safely retry only identical bytes. A crash after signing but before broadcast also cannot reserve a conflicting message. Exact-byte equality avoids relying on an unratified hash function for conflict detection.

The prototype uses redb's exclusive file lock and is single-process. Backup rollback can erase reservations; production requires rollback-resistant external anchoring or protected monotonic state. No HSM/KMS, encrypted key store, key rotation, validator transitions, signer process, engine integration or multi-process support is included.

## References

- Phase 20C application boundary: `docs/protocol/phase-20c-application-engine-boundary.md`
- Phase 20D record: `docs/protocol/phase-20d-durable-signer-safety.md`
- Cryptographic domain separation constraint: `docs/architecture/adr/ADR-0006-cryptographic-domains.md`
