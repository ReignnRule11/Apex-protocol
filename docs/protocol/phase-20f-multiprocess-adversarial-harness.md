# Phase 20F — Multi-Process Adversarial Harness

**Status:** Baseline implementation submitted for exact-head CI validation.
**Base main:** `aa7c37b9451235e69780fa7902968d1fd157e33f`
**Release impact:** G2 and G3 remain BLOCKED. Production remains NO-GO.

## Objective

Introduce reproducible process-level fault injection before connecting a consensus engine, validator keys or production network. Exercise independent process lifecycle, deterministic message delivery faults, append-only journal recovery and replay behavior.

## Architecture

The isolated Cargo workspace at `experiments/consensus-harness` is deliberately excluded from the production workspace.

- A coordinator integration test starts four independent operating-system worker processes.
- Each worker has its own stdin/stdout command channel and separate append-only journal file.
- Worker command framing is capped at 2,048 bytes; encoded payloads are capped at 1,024 hex characters (512 bytes); journal records are capped at 10,000.
- A successful append is written and `sync_all` completes before the worker acknowledges it.
- Journal startup validates contiguous sequence numbers and payload encoding; malformed persisted data fails worker startup.
- Delivery order is controlled by the test coordinator, making partition, drop, delay and reordering scenarios deterministic and reproducible.

## Implemented scenarios

1. Four independent processes accept a common ordered log.
2. A partitioned process misses a record; out-of-order delivery is rejected as a gap; replay after healing catches it up.
3. Delayed and dropped records are withheld and later delivered in order.
4. A conflicting payload at an already accepted sequence is rejected without overwriting the journal.
5. A worker process stops and restarts against its own journal and recovers its accepted prefix.
6. A replacement process starts empty and catches up by replaying the accepted history.
7. Repeated deterministic scenarios produce byte-identical journals.
8. Oversized payloads are rejected and do not mutate process state.

## CI

The dedicated `Four-process adversarial harness` job runs Rust 1.91 formatting, Clippy with warnings denied, and all isolated harness tests. The root workspace and Malachite research jobs remain separate.

## Explicit limits

This harness is not a BFT protocol or consensus engine. It does not invoke Malachite, sign or authenticate votes, verify certificates, prove quorum agreement, implement a network stack, simulate cryptographic Byzantine identities, or demonstrate safety/liveness. The worker journal is a test fixture, not the production signer WAL or a consensus WAL. The deterministic coordinator models delivery faults, not kernel/network timing, packet loss or partitions on real sockets. Restart recovery proves only the fixture's own append-only file behavior. No production validators, keys, funds or network are enabled.

## Exit criteria

1. Four distinct worker processes are observed by the integration test.
2. Fault schedules deterministically cover gap rejection, delayed/dropped delivery, recovery, conflict rejection, restart and replacement replay.
3. Repeated runs produce identical journal bytes.
4. Oversized payloads fail closed without state mutation.
5. Isolated format, Clippy and test jobs pass on the exact PR head.
6. Documentation clearly states that BFT correctness and authenticated P2P remain unimplemented.

**Decision:** Process harness baseline GO when exact-head CI passes; engine-backed consensus and production validator operation remain NO-GO.
