# Lifecycle Witness Deployment Runbook

## Required topology

Deploy apex-lifecycle-witness on a host that is not administered by the
validator's node operator. Use separate cloud accounts or equivalent
administrative boundaries where possible. The witness host must have its own
persistent volume, backup policy, OS identities, and audit trail.

The Rust process listens on loopback only. Caddy (or an equivalently reviewed
mTLS reverse proxy) terminates TLS and requires a client certificate issued by
the dedicated validator-client CA.

## Build and run

Build the service from a reviewed, pinned commit using the repository's
Rust 1.91.0 toolchain:

```sh
cargo build --release -p apex-lifecycle-witness
```

Create a dedicated service account and persistent directory. Set ownership
and permissions so only the witness service account can write the database:

```sh
install -d -o apex-witness -g apex-witness -m 0700 /var/lib/apex-witness
```

Configure a systemd unit with:
- User=apex-witness
- Group=apex-witness
- Environment=APEX_WITNESS_DB_PATH=/var/lib/apex-witness/lifecycle.redb
- Environment=APEX_WITNESS_LISTEN=127.0.0.1:8787
- EnvironmentFile=/etc/apex-witness/witness.env (mode 0600, root-owned; contains APEX_WITNESS_API_TOKEN generated from at least 32 random bytes)
- NoNewPrivileges=true
- ProtectSystem=strict
- ProtectHome=true
- ReadWritePaths=/var/lib/apex-witness
- PrivateTmp=true

Do not run the service as root. Do not share the witness database or backup
credentials with validator hosts.

## mTLS

Provision a dedicated client CA and issue a short-lived client certificate to
each authorized validator. Configure the reverse proxy to require and verify
client certificates. The witness server certificate must chain to the CA
provided to the validator client. Rotate and revoke credentials through a
documented operational process. Do not place private keys in source control,
container images, CI logs, or the witness database.

## Validator integration

Construct HttpLifecycleWitnessClient::from_pem with:
1. The HTTPS witness endpoint.
2. The validator's client certificate chain and private key in PEM format.
3. The CA certificate used to validate the witness server.
4. The bearer token configured on the witness service.

The constructor rejects HTTP endpoints and does not disable certificate
validation. The client implements both MonotonicLifecycleCheckpointStore
and ExternalLifecycleRollbackWitness. Keep the local checkpoint store and
remote witness as separate objects and use the remote object for witness
updates and startup comparisons.

Do not call the blocking client from an async executor thread. If the node
runtime is asynchronous, isolate these operations on a blocking worker and
await the result before enabling consensus or signing.

## Acceptance checks

Before authorizing a validator:
- Confirm an unauthorized client certificate is rejected at the proxy.
- Confirm an authorized client can read a known network head.
- Confirm a first write succeeds only at sequence 1.
- Confirm a stale expected head returns conflict and does not overwrite state.
- Confirm sequence skips are rejected.
- Stop the witness service and confirm validator startup fails closed.
- Restore an older local validator database and confirm it disagrees with the
  current remote witness and startup is rejected.
- Restore an older witness backup in a controlled exercise and confirm the
  incident process detects and escalates the witness rollback; this service
  alone cannot detect rollback of its own database.

Record certificate fingerprints, deployment commit, service binary digest,
database location, backup policy, and test evidence in the operations record.

## Recovery

Never automatically reconcile a mismatch. Stop validator services, preserve
both stores and logs, and invoke the separately authorized recovery procedure.
No recovery or genesis bootstrap endpoint is exposed by this service.
