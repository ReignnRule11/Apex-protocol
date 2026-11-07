# Consensus Integration Spike — Phase 3 and Phase 20A

**Status:** Engine-neutral proposal-header boundary implemented and unit-tested; no engine dependency or running consensus implementation.
**Review date:** 2026-11-07
**Phase 20A:** Candidate revalidated; isolated integration spike not yet started.

## Objective

Integrate a mature BFT engine without coupling protocol validity, execution or validator-key custody to an unreviewed upstream API. The spike must end with a reproducible multi-validator harness and documented go/no-go decision, not merely a demo that emits blocks.

## Current facts

- APEX uses edition 2024 and declares Rust 1.91.0.
- Malachite v0.8.2 declares Rust 1.88; the current APEX toolchain is compatible.
- Research baseline: tag v0.8.2, exact commit a453b32155f4ba9f959724279b52f601bae92e09, Apache-2.0.
- Upstream labels Malachite alpha, under heavy development and not externally audited.
- No Malachite crate is present in APEX's dependency graph.
- APEX proposal preflight checks chain ID, protocol version, sequential height and parent linkage only. It does not authenticate a proposer, validate transactions, verify a certificate or establish finality.

## Implemented engine-neutral boundary

- Shared apex-types is consumed by apex-consensus.
- ExpectedParent and ProposalHeader are typed in-memory structures.
- validate_proposal_header checks chain/version/height/parent and rejects parent-height overflow.
- Unit tests cover valid and rejection paths.

These are not canonical wire messages. No consensus signature suite, canonical vote codec, block hash algorithm or validator authentication has been selected.

## Phase 20A decision

The former MSRV blocker is closed by APEX Rust 1.91.0. Pin Malachite v0.8.2 at the exact commit above for research only. Do not add it to the production workspace or connect validator keys yet: alpha maturity, audit absence, dependency/API, signer, persistence and recovery review remain outstanding.

Phase 20B may create a separate isolated dependency/API spike only after crate selection, feature flags, license notices, dependency graph and upstream test commands are documented. Keep engine types out of protocol/application crates.

## Required adapter responsibilities

1. Bind requests and persisted records to APEX chain ID, protocol version and height.
2. Validate proposals deterministically, including canonical structure and full application validity before returning acceptance.
3. Accept only verified engine-finalized decisions; bind them to expected chain, height, parent and proposal identity.
4. Persist finalized application state and commit metadata atomically or with a formally specified recovery protocol.
5. Derive validator-set changes deterministically from committed protocol state with unambiguous activation heights.
6. Use a dedicated consensus key domain, chain-bound signing context, durable anti-double-sign state and explicit key rotation/recovery.
7. Bound inbound messages, queues, peer counts, proposal size, timeouts, storage growth and shutdown behavior.

## Required tests before production adoption

- Pinned upstream test suite and exact dependency graph.
- At least four independent validator processes.
- Conflicting proposals, equivocation, dropped/delayed/reordered messages and partitions.
- Crash/restart, backup/restore, validator-set changes and anti-double-sign persistence.
- No commit of an application-invalid block.
- Deterministic replay from identical genesis with byte-identical full-state commitments.
- Oversized/malformed messages and resource exhaustion with bounded memory/queues.
- Reproducible CI commands, logs, toolchain, lockfile and known failures.
- Independent security review.

## Go/no-go

**Go to the next isolated spike** only after the exact crate set, feature selection, dependency/license/advisory review and API map are recorded.

**No-go for production** while the engine is alpha/unaudited, signer safety is unproven, recovery semantics are incomplete, or adversarial tests and independent review remain outstanding. Reopen ADR-0004 and compare CometBFT or another maintained engine if any gate fails.

## References

- Malachite v0.8.2: https://github.com/circlefin/malachite/releases/tag/v0.8.2
- Exact source commit: https://github.com/circlefin/malachite/commit/a453b32155f4ba9f959724279b52f601bae92e09
- Upstream manifest: https://github.com/circlefin/malachite/blob/v0.8.2/code/Cargo.toml
- Upstream license: https://github.com/circlefin/malachite/blob/v0.8.2/LICENSE
- APEX evaluation: ./consensus-engine-evaluation.md
- APEX adapter contract: ./consensus-adapter-contract.md
