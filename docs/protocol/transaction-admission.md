# Transaction Admission — Phase 4

**Status:** limited static preflight implemented; full transaction validity is not implemented.

## Implemented checks

The validate_transaction_admission function in crates/apex-consensus accepts an internal TransactionAdmission and caller-provided AdmissionContext. It checks:

1. Transaction chain ID equals configured chain ID.
2. Transaction protocol version equals configured version.
3. Gas limit is greater than zero.
4. Gas limit does not exceed the supplied block gas limit.
5. Priority fee per gas does not exceed maximum fee per gas.
6. Gas limit multiplied by maximum fee per gas fits in u128.

On success it returns gas_limit multiplied by max_fee_per_gas as a maximum-fee upper bound. This is not a fee charged, debited, burned or paid to a proposer.

## Deliberate non-goals

The function does not verify signatures, derive or authenticate sender identity, check nonce against account state, check balances, calculate intrinsic gas, validate recipient/calldata/access lists, enforce transaction ordering, decode canonical bytes, execute EVM code or settle fees. The nonce field is carried as data only and is not checked. Zero maximum fee is accepted because no minimum-fee policy has been approved.

The structures are in-memory Rust types, not a canonical wire format. A successful preflight result is not proof that a transaction is valid or executable.

## Required next work

- Ratify canonical transaction encoding and signing domain.
- Specify sender authentication and signature verification with a reviewed algorithm.
- Define account nonce and balance semantics against deterministic parent state.
- Specify intrinsic gas, transaction ordering, block gas accounting and fee settlement.
- Add stateful admission tests, replay tests, malformed-input fuzzing and cross-implementation vectors.
- Integrate execution only after an ADR pins the EVM implementation and fork rules.

## Test coverage

Unit tests cover valid admission, chain/version mismatch, zero gas, gas above block limit, priority-fee ordering, multiplication overflow and the currently permitted zero-fee case. They do not test signatures, account state or execution.
