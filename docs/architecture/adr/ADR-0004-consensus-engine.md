# ADR-0004: Consensus Engine Candidate

- **Status:** Accepted for prototype evaluation; production adoption conditional
- **Decision:** Malachite is the preferred Rust-native BFT candidate for an integration spike. CometBFT remains the reference/fallback comparator.
- **Context:** APEX requires BFT proof-of-stake consensus and Rust is the selected implementation language. CometBFT is a mature Go implementation; Malachite is Rust-native and Tendermint-style.
- **Rationale:** Malachite best aligns with the Rust architecture and has evidence of active engineering and use as the consensus foundation for Arc. Its upstream README explicitly labels it alpha and not externally audited.
- **Consequences:** APEX must not treat Malachite as production-ready. Pinning, API review, fault injection, recovery tests, resource-bound tests and independent security review are mandatory adoption gates. Reopen this ADR if any gate fails.
- **Non-decisions:** No signature suite, chain ID, codec, economic parameter or performance target is approved by this ADR.
