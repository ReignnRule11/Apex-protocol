# Phase 20G2D26 — Independently Operated Lifecycle Witness

## Objective

Replace the test-only witness boundary with a concrete, separately deployable
service that durably maintains monotonic checkpoint heads and a production
client using strict mutual TLS.

## Implementation

- Add apex-lifecycle-witness, a real HTTP service backed by the existing
  redb lifecycle checkpoint store.
- Expose read-head and atomic compare-and-advance endpoints.
- Reject malformed network IDs, invalid sequence transitions, stale writers,
  unauthorized requests, and storage failures.
- Require a high-entropy bearer token and bind only to loopback. Public TLS
  termination and mutual client authentication are delegated to an
  independently operated reverse proxy.
- Add apex-lifecycle-witness-client, a real HTTPS client implementing both
  MonotonicLifecycleCheckpointStore and ExternalLifecycleRollbackWitness.
- Require HTTPS, a client certificate/private key, a dedicated witness CA,
  and a bearer token. No plaintext or invalid-certificate fallback exists.
- Add service/router integration tests using a real temporary redb database;
  no in-memory witness implementation is used by the service test.
- Document mTLS, host separation, independent backup controls, bootstrap,
  credential handling, and operational verification.

## Trust model

The service is only an independent witness when deployed on a separately
administered host with independent storage, credentials, and backup controls.
The example Caddy configuration requires and validates client certificates.
A separate high-entropy API token provides defense in depth and prevents
unauthenticated local callers from bypassing the reverse proxy.

The service cannot prevent a privileged witness-host administrator from
restoring an old database image. Independent backup immutability, access
separation, monitoring, and restore testing remain deployment requirements.

## Startup integration boundary

The remote client can now be supplied to the existing startup verification
gate and checkpoint acceptance path. This phase does not wire the gate into
apex-node; that binary is currently a placeholder, and the consensus crate
explicitly does not implement BFT consensus. A later node-runtime phase must
consume the remote witness client before enabling any consensus, signer, or
network services.

## Release posture

This phase is not evidence of a deployed witness, a live validator network,
BFT correctness, or mainnet readiness. G0–G5 remain blocked; G6/G7 remain not
started; overall release remains NO-GO.
