# ADR-0002: Rust Implementation

- **Status:** Accepted
- **Decision:** Use Rust for protocol and node implementation.
- **Rationale:** Strong type system, memory-safety properties and mature async ecosystem.
- **Consequences:** Dependency auditing, reproducible builds, unsafe-code policy and toolchain management are required. Rust alone does not guarantee protocol security.

## Minimum supported Rust version

The workspace MSRV is raised from Rust 1.85 to Rust 1.90 in Phase 10 to use the exact redb 4.5.0 release. The upstream redb 4.5.0 manifest explicitly declares Rust 1.90. This is an explicit compatibility decision, not an incidental transitive dependency change.

The toolchain is pinned to 1.90.0 in `rust-toolchain.toml`, and CI installs the same version. Revisit this floor only through a reviewed ADR and a complete dependency/workspace test. Malachite remains unintegrated: its upstream workspace currently declares Rust 1.88, which is no longer the blocking MSRV constraint, but consensus adoption still requires its independent security, compatibility and operational gates.

## Dependency policy

Pin consensus-critical and state-persistence dependencies deliberately. Audit exact release source, license, MSRV, changelog, dependency tree and recovery semantics. A passing build does not establish the correctness or security of a storage engine.
