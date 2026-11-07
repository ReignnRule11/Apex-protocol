# ADR-0046: Experimental canonical consensus signing preimage

- **Status:** Accepted for isolated implementation only; not a ratified protocol decision.
- **Date:** 2026-11-07
- **Context:** Durable signer reservations require exact deterministic message bytes, but APEX has not selected a cryptographic signature suite or finalized its consensus wire protocol.
- **Decision:** Introduce a fixed-width, big-endian signing preimage with a dedicated domain tag and encoding version. Bind chain ID, protocol version, height, round, step, signer identity, validator-set hash and block hash. Permit nil block hashes only for prevote/precommit messages.
- **Consequences:** The encoding provides an APEX-owned preimage for future signer integration. It does not authenticate signers, validate validator-set hashes, sign or verify messages, or establish finality. Signature suite, public-key identity mapping, validator-set hash construction and final wire profile remain open.
- **Safety:** No production key or validator may use this experimental profile. G2 remains blocked pending a real signature backend and commit-certificate verifier.
