# ADR-0047: Experimental authenticated consensus certificates

- **Status:** Accepted for isolated implementation and testing only; not a ratified protocol decision.
- **Date:** 2026-11-07
- **Context:** Phase 20G2A established deterministic signing bytes but did not authenticate signer identities, signatures or quorum certificates.
- **Decision:** Pin ed25519-dalek 3.0.0 for strict Ed25519 verification. Derive validator IDs from a domain-separated SHA-256 hash of an algorithm-tagged public key. Hash authenticated validator sets canonically over sorted IDs, voting powers and public keys. Verify precommit signatures against the exact scheduled validator set and require strictly greater than two-thirds of its voting power.
- **Consequences:** The consensus crate now has a real cryptographic certificate-verification primitive. It rejects malformed/weak keys, unknown/duplicate signers, mismatched context/set hashes, invalid signatures and insufficient quorum.
- **Explicit non-decisions:** No key custody or signing service, no authenticated governance path for validator-set schedule updates, no canonical certificate wire codec, no Malachite runtime integration, no application commit integration, and no production finality claim.
- **Safety:** This experimental profile is not ratified. Do not connect production validator keys or funds. G2, G3 and all release gates remain blocked pending runtime integration, recovery/fault testing and independent review.
