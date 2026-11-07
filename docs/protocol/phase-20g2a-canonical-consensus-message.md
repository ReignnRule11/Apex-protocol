# Phase 20G2A — Canonical Consensus Signing Preimage

**Status:** Implementation submitted for exact-head CI.
**Base main:** 63fe22b1064654b5b09113e5d0e5da33b58e1420
**Scope:** APEX-owned preimage encoding only; no signing or verification.

## Objective

Establish deterministic, domain-separated bytes for consensus signing intents before connecting a cryptographic signer or Malachite runtime.

## Implementation

- Add apex-consensus::consensus_message.
- Encode a dedicated domain tag and encoding version.
- Bind chain ID, protocol version, height, round, consensus step, signer identity, validator-set hash and block hash.
- Use fixed-width fields and big-endian integer encoding.
- Permit a zero block hash only for nil prevote/precommit; reject nil proposals.
- Test fixed width, domain separation, coordinate sensitivity, nil semantics and invalid zero context.

## Security boundaries

This module constructs preimage bytes only. It does not create or verify signatures, bind signer identity to a public key, select a signature suite, authenticate validator-set hashes, derive block hashes, parse Malachite messages, verify certificates or establish finality.

The profile is experimental and not a ratified APEX protocol decision. No production key may be connected to it.

## Exit criteria

1. Exact-head root workspace format, Clippy and tests pass.
2. Existing isolated Malachite API and upstream tests remain green.
3. Every signed coordinate is bound in deterministic bytes.
4. Documentation preserves the distinction between encoding and authentication.

## Next subphase

20G2B must add a pinned cryptographic signature backend, public-key-to-validator identity binding, and strict commit-certificate verification against the scheduled validator set. Only then should a concrete isolated Malachite application/runtime be started.
