# ADR-0003: Optional Bitcoin Checkpoint Anchors

- Status: Accepted as design direction
- Decision: Explore periodic publication of APEX commitments to Bitcoin as an optional evidence layer.
- Rationale: Independent observers may verify a commitment existed no later than its Bitcoin inclusion time, subject to confirmation and reorg policy.
- Consequences: Anchoring cannot replace APEX consensus, prove execution correctness or halt APEX when unavailable.
