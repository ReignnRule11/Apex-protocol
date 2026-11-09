# APEX Lifecycle Witness Service

This is a separately deployable service for lifecycle checkpoint heads. It is
not intended to run on a validator host. It persists heads in redb and uses
atomic compare-and-advance; the service process never accepts a stale writer.

## Runtime configuration

- `APEX_WITNESS_DB_PATH`: durable database path. Default:
  `/var/lib/apex-witness/lifecycle.redb`.
- `APEX_WITNESS_LISTEN`: loopback socket only. Default:
  `127.0.0.1:8787`. Non-loopback addresses are rejected.
- `APEX_WITNESS_API_TOKEN`: required random secret of at least 32 bytes. Every
  API request must include it as a bearer token. The mTLS proxy remains
  mandatory; the token also prevents unauthenticated local callers from
  bypassing the proxy.

The service deliberately has no public plaintext listener. It requires both a
high-entropy bearer token and an independently administered TLS reverse proxy
that requires mutual TLS.

## API

- `GET /v1/networks/{64-lowercase-hex-network-id}/head`
  returns `{"head":null}` before initialization or the current anchor. Every
  request must include `Authorization: Bearer <APEX_WITNESS_API_TOKEN>`.
- `POST /v1/networks/{network-id}/head` accepts
  `{"expected":null,"next":{"sequence":1,"checkpoint_hash":[...32 bytes...]}}`.
  Subsequent updates must supply the exact current `expected` anchor and
  `next.sequence = expected.sequence + 1`.
- HTTP 409 means the expected head was stale.
- HTTP 422 means the requested sequence is invalid.
- HTTP 503 means durable storage is unavailable.

## Trust and deployment requirements

The witness must run on a separate host, under separate administrative
credentials, with separate storage and backup access. The validator's mTLS
client certificate must be issued by a dedicated client CA; the proxy must
require and verify that CA. The proxy's server certificate must be trusted by
the validator. Do not expose port 8787 outside loopback.

A second process, container, database, or disk on the validator host is not an
independent trust domain. Compromise of the witness host, its root account, its
storage administrator, or its signing/CA authority remains in the threat model.

The service does not silently initialize genesis, repair divergence, or accept
rollback. A missing witness head must be initialized only by an explicitly
authorized bootstrap ceremony before validators start. Protect the database
and its backups with independent access controls and tested restore procedures.
A redb file alone cannot resist an administrator restoring an old witness
backup; deployment-level immutability and backup controls are required.

## Example Caddy mTLS reverse proxy

Install Caddy on the witness host. Keep the Rust service bound to loopback and
configure a dedicated client CA. Replace the example host and file paths with
the actual deployment values:

```caddyfile
witness.example.net {
    tls {
        client_auth {
            mode require_and_verify
            trusted_ca_cert_file /etc/caddy/apex-validator-client-ca.pem
        }
    }
    reverse_proxy 127.0.0.1:8787
}
```

Use firewall rules to permit HTTPS only from approved validator egress
addresses where practical. Issue, rotate, revoke, and audit client
certificates operationally. Never commit private keys or production
certificates to this repository.

## Operational checks

Before use, provision the witness host, configure persistent storage and
backups, install the mTLS proxy, verify client-certificate rejection for
unauthorized clients, verify successful access for an authorized validator,
and test backup-restore detection against the validator's local head. The
service binary and integration tests do not prove that a deployment has been
provisioned or that its trust domains are independent.
