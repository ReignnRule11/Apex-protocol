# ADR-0001: Layered Protocol Architecture

- Status: Accepted as design direction
- Decision: Separate consensus, execution, state, data availability, networking, anchoring and governance into explicit protocol layers.
- Rationale: Makes trust boundaries and upgrade responsibilities reviewable.
- Consequences: Cross-layer interfaces and versioning must be specified; layering does not require separate microservices.
