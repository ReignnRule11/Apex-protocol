# ADR-0045: Isolated Malachite EngineBuilder Adapter

- **Status:** Accepted for research integration only; production adoption not approved.
- **Date:** 2026-11-07
- **Context:** Phase 20B verified the pinned Malachite v0.8.2 channel API surface, but did not exercise the full builder path. Phase 20C–20F established APEX-side application, signer, validator-set and process-harness primitives without a real BFT engine.

## Decision

Add a typed `build_engine` adapter to the existing isolated `experiments/malachite-spike` workspace. The adapter calls the pinned upstream `EngineBuilder` and configures all required default actor categories: WAL, network, consensus, sync and request channels. It returns the upstream channel set and engine handle.

Do not add Malachite to the production Cargo workspace. Do not instantiate validators or create key/network material as part of this compile-only phase.

## Rationale

- Compile-checks the actual upstream builder call and required codec/context trait bounds rather than merely re-exporting builder symbols.
- Keeps the transitive dependency graph and generated lockfile review isolated.
- Preserves APEX ownership of application validation, finality verification, signer safety, protocol codecs and durable commit semantics.
- Avoids treating upstream test-app behavior or its wire/signature formats as ratified APEX protocol decisions.

## Consequences

- APEX has a compile-checked adapter to the upstream engine construction API.
- No running consensus engine, validator process, socket, application event loop, certificate verifier or finalized-state commit has been demonstrated.
- G2 and G3 remain blocked. A real isolated multi-validator runtime and adversarial network/recovery evidence are required before considering dependency promotion.
- Malachite's alpha maturity, audit status, dependency advisories, license compatibility and operational suitability remain review items.
