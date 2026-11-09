# Phase 20G2D26 — Independently Operated Lifecycle Witness

## Objective

Replace the test-only witness boundary with a concrete, separately deployable
service that durably maintains monotonic checkpoint heads. The witness is a
different trust domain from the validator's local lifecycle database.

## Implementation

- Add `apex-lifecycle-witness`, a real HTTP service backed by the existing
  redb lifecycle checkpoint store.
- Expose read-head and atomic compare-and-advance endpoints.
- Reject malformed network IDs, invalid sequence transitions, stale writers,
  and storage failures.
- Bind only to loopback. Public TLS termination and mutual client
  authentication are delegated to an independently operated reverse proxy.
- Add an actual service/router integration test using a real temporary redb
  database; no in-memory witness implementation is used by the service test.
- Document mTLS, host separation, independent backup controls, bootstrap, and
  operational verification.

## Trust model

The service is only an independent witness when deployed on a separately
administered host with independent storage, credentials, and backup controls.
The example Caddy configuration requires and validates client certificates.
The service itself does not expose a public plaintext listener.

The service cannot prevent a privileged witness-host administrator from
restoring an old database image. Independent backup immutability, access
separation, monitoring, and restore testing remain deployment requirements.

## Startup integration boundary

This phase creates the concrete service endpoint and durable witness API. It
does not wire the existing startup gate into `apex-node`; that binary is
currently a placeholder, and the consensus crate explicitly does not
implement BFT consensus. A later node-runtime phase must consume the remote
witness client before enabling any consensus, signer, or network services.

## Release posture

This phase is not evidence of a deployed witness, a live validator network,
BFT correctness, or mainnet readiness. G0–G5 remain blocked; G6/G7 remain not
started; overall release remains **NO-GO**.
