# Consensus Integration Spike — Phase 3

**Status:** engine-neutral proposal-header boundary implemented and unit-tested; no engine dependency or running consensus implementation.
**Review date:** 2026-11-07

## Objective

Prove that APEX can integrate a mature BFT engine without coupling protocol validity, execution or validator-key custody to an unreviewed upstream API. The spike must end with a reproducible multi-validator test harness and a documented go/no-go decision—not a demo that merely emits blocks.

## Current facts

- APEX is a Rust workspace using edition 2024 and declares `rust-version = "1.85"` in the root manifest.
- The current Malachite upstream workspace declares version `0.8.2` and `rust-version = "1.88"`.
- This is an explicit MSRV mismatch. No Malachite crate is present in APEX's dependency graph.
- Malachite's upstream README describes it as alpha, under heavy development and not externally audited.
- `apex-consensus` now has a pure preflight function that checks chain ID, protocol version, sequential height and parent-hash linkage. It does not authenticate a proposer, validate transactions, verify a commit certificate or establish finality.
- The previous documentation-only revision passed CI. This phase's code revision is subject to its own CI run.

## Implemented in this phase

- Shared `apex-types` is now consumed by `apex-consensus`.
- `ExpectedParent` and `ProposalHeader` provide typed, engine-neutral in-memory structures.
- `validate_proposal_header` performs deterministic chain/version/height/parent checks and rejects parent-height overflow.
- Unit tests cover the valid case and each rejection path.

These types are not canonical serialized protocol messages. No wire format, block hash algorithm, validator authentication or consensus engine API has been selected.

## Decision for this phase

Do not integrate Malachite yet. First decide whether APEX will raise its declared MSRV to at least 1.88 and update CI/toolchain documentation accordingly. Do not lower Malachite's MSRV, patch upstream, or suppress Cargo compatibility warnings to force an integration.

Continue building the engine-neutral boundary in small, testable increments. Do not represent preflight checks as a full application adapter or production consensus implementation.

## Required adapter responsibilities

1. **Application identity:** bind every request and persisted record to APEX chain ID, protocol version and height.
2. **Proposal validation:** validate candidate block structure, parent linkage, canonical transaction ordering and deterministic application preconditions. Proposal acceptance must not depend on local wall-clock or nondeterministic external services.
3. **Finalized commit:** accept only a valid engine-finalized decision; execute the ordered transactions against the correct parent state; persist state and commit metadata atomically or recoverably.
4. **Validator updates:** derive a deterministic validator-set update from committed protocol state; define activation/exit boundaries and prevent ambiguous same-height changes.
5. **Signing boundary:** use a dedicated consensus key domain, chain-bound signing context, durable anti-double-sign state and explicit key rotation/recovery. Never reuse EVM transaction keys by assumption.
6. **Persistence and recovery:** specify write-ahead log/state durability, replay rules, crash points, corruption handling and safe restart behavior.
7. **Operational controls:** bound inbound messages, queues, peer counts, proposal size, timeouts, storage growth and shutdown behavior.

## Test plan and exit criteria

- Build and run the pinned upstream test suite.
- Establish at least four independent validator processes in a local deterministic harness.
- Demonstrate successful block finalization with a documented validator set and voting-power distribution.
- Inject one-third-plus Byzantine voting power and verify the documented safety/liveness behavior without claiming guarantees outside the protocol assumptions.
- Inject equivocation, conflicting proposals, dropped/delayed/reordered messages, partitions, process crashes, restart and validator-set changes.
- Verify no double-sign after restart and no commit of an application-invalid block.
- Replay the same finalized block sequence from identical genesis state and compare state roots byte-for-byte.
- Exercise oversized/malformed messages and resource exhaustion; assert bounded memory/queue behavior.
- Record exact commands, toolchain, dependency lockfile, test logs and known failures.

The spike is not complete until the harness is reproducible in CI and the results are reviewed. A green compile or unit-test suite alone is not an adoption gate.

## Go/no-go criteria

**Go to integration** only if MSRV policy is resolved, exact versions are pinned, the upstream API and dependency graph are reviewed, required tests pass, recovery/signing safety is demonstrated and independent review has no unresolved critical finding.

**No-go** if API churn prevents a stable adapter, signer safety cannot be demonstrated, fault tests expose unresolved safety failures, dependencies are unmaintained or incompatible, or resource bounds cannot be enforced. Reopen ADR-0004 and compare CometBFT or another maintained engine.

## References

- Malachite repository: https://github.com/circlefin/malachite
- Upstream README and maturity statement: https://github.com/circlefin/malachite/blob/main/README.md
- Upstream workspace manifest: https://github.com/circlefin/malachite/blob/main/code/Cargo.toml
- APEX consensus evaluation: ./consensus-engine-evaluation.md
- APEX adapter contract: ./consensus-adapter-contract.md
- APEX threat model: ../threat-model/overview.md
