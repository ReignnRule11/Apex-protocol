# APEX Protocol

**APEX** is a research-stage Layer-1 blockchain infrastructure project. The current repository establishes protocol documentation and a Rust workspace foundation; it is not a production blockchain, public testnet, or mainnet.

## Design direction

- Independent proof-of-stake BFT consensus, with safety assumptions and validator economics specified before implementation.
- Rust implementation and deterministic state-transition rules.
- EVM-compatible execution as a planned capability, using a pinned and tested EVM revision rather than a custom virtual machine.
- Full transaction-data availability in the initial design.
- Optional Bitcoin checkpoint anchoring for externally verifiable checkpoint evidence. Bitcoin anchoring is not APEX consensus and does not inherit Bitcoin's security.
- Hybrid monetary policy: a proposed hard supply ceiling, scheduled issuance, and a fee market with base-fee burning and proposer priority fees. Parameters remain undecided.

## Repository status

**Stage:** specification and foundation. Consensus, execution, networking, validator operations, and anchoring are not yet implemented. No token is being offered and no investment or return is promised.

## Planned repository map

- `docs/protocol/` — protocol, consensus, execution, economics, anchoring, and governance specifications.
- `docs/architecture/adr/` — architecture decision records.
- `docs/threat-model/` — security assumptions and threat analysis.
- `crates/` — Rust workspace crates, introduced incrementally.
- `genesis/` — development and testnet configuration templates; not production parameters.

## Engineering principles

1. Safety before throughput claims.
2. Deterministic execution and canonical encoding.
3. Explicit trust assumptions and adversarial testing.
4. No invented consensus algorithm: select and document a mature BFT design before implementation.
5. No mainnet or decentralization claims without independent validators, audits, and operational evidence.

## Development

The workspace is being established incrementally. Once the Rust crates are added, use the repository's documented toolchain and run formatting, linting, and tests before proposing changes.

## License

No license has been selected yet. All rights are reserved unless and until a license is added.
