# ADR-0082 — Independent Lifecycle Witness Service

- **Status:** Proposed
- **Date:** 2026-11-09
- **Related phase:** 20G2D26
- **Supersedes:** None

## Context

The lifecycle checkpoint store is durable locally but cannot detect a restore
of an older local database without an independently protected head. The
previous external witness interface had no concrete service or transport.

## Decision

Provide a deployable witness service backed by redb with an HTTP API for
reading and atomically advancing network-scoped heads. The process binds only
to loopback and must be fronted by a separately administered mTLS reverse
proxy. Each request must also carry a high-entropy bearer token. A strict
HTTPS client uses mutual TLS, a dedicated witness CA, and the token, and
implements both the monotonic checkpoint store and external witness traits.

The validator and witness must use separate hosts, credentials, storage, and
backup administration.

## Consequences

- A concrete persistent service replaces an in-memory-only witness
  integration boundary for deployments that provision it.
- The client rejects plaintext endpoints and does not disable certificate
  validation.
- The service rejects stale compare-and-advance operations atomically.
- The mTLS reverse proxy, API token, and independent host are part of the
  security boundary and must be provisioned and tested; the repository does
  not claim that they are deployed.
- Witness-host privileged compromise or restoration of old witness backups
  remains a risk unless independently controlled backup immutability and
  monitoring are implemented.
- The service does not activate genesis, reconcile divergence, run consensus,
  or make the current node binary production-ready.
