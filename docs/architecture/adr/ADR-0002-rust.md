# ADR-0002: Rust Implementation

- Status: Accepted as design direction
- Decision: Use Rust for protocol and node implementation.
- Rationale: Strong type system, memory-safety properties and mature async ecosystem.
- Consequences: Dependency auditing, reproducible builds, unsafe-code policy and toolchain pinning are required. Rust alone does not guarantee protocol security.
