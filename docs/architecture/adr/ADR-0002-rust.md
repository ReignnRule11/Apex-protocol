# ADR-0002: Rust Implementation

- **Status:** Accepted as design direction; MSRV increase remains an open decision.
- **Decision:** Use Rust for protocol and node implementation.
- **Rationale:** Strong type system, memory-safety properties and mature async ecosystem.
- **Consequences:** Dependency auditing, reproducible builds, unsafe-code policy and toolchain management are required. Rust alone does not guarantee protocol security.

## Minimum supported Rust version

The workspace currently declares Rust 1.85 in the root `Cargo.toml`. This is the project's declared MSRV, not an assurance that every future dependency supports it. The Malachite upstream workspace currently declares Rust 1.88, so it cannot be added without a deliberate MSRV decision.

Any MSRV change must:
1. Be reflected in `Cargo.toml`, `rust-toolchain.toml`, CI and contributor documentation.
2. Be tested against the complete workspace and locked dependency graph.
3. Be recorded as a reviewed compatibility decision rather than changed only to satisfy one dependency.
4. Consider contributor and deployment environments.

Until that decision is made, keep Malachite out of the dependency graph and retain the engine-neutral integration boundary.
