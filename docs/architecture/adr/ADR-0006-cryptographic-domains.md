# ADR-0006: Cryptographic Domain Separation

- **Status:** Accepted as a design constraint; algorithms remain open
- **Decision:** APEX must use separate, explicitly domain-separated cryptographic contexts for transaction authorization, consensus votes, block/transaction commitments and checkpoint commitments. Key roles must be separable and independently rotatable.
- **Rationale:** Reusing signatures or hashes across protocol contexts can create cross-protocol confusion and replay risks.
- **Consequences:** Every signed or hashed object must include a versioned domain tag and relevant chain/network identifier. Exact algorithms, key formats, aggregation and address derivation require a separate reviewed ADR after consensus and EVM compatibility requirements are mapped.
