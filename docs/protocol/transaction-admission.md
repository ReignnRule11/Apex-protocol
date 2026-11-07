# Transaction Admission — Phase 5

**Status:** static preflight and a minimal state-aware admission prototype are implemented. This is not full transaction validity.

## Static checks

`validate_transaction_admission` checks chain ID, protocol version, nonzero gas limit, caller-supplied block gas limit, priority-fee/max-fee ordering, and checked multiplication of gas limit by maximum fee per gas. It returns the maximum-fee upper bound, not a charged or settled fee.

## State-aware checks

`validate_transaction_against_state` first runs static checks, then reads the sender account from a caller-supplied read-only `AdmissionState` view. It rejects a missing account, requires exact equality between transaction nonce and the account nonce, and requires balance to cover:

`transaction.value + (gas_limit × max_fee_per_gas)`

Both multiplication and addition are checked for overflow. The affordability rule is deliberately conservative: it reserves the full maximum-fee upper bound plus value. It does not predict actual gas usage or calculate a refund.

The state provider must supply a stable view of the exact candidate parent state for the duration of validation. A production adapter must bind the view to a versioned state root or immutable snapshot; this prototype trait does not enforce that operational guarantee.

## Trust boundary and limitations

`AccountId(u64)` is an internal placeholder, not a canonical address. The sender field is untrusted until a selected signature algorithm and chain/version-bound signing domain authenticate it. The state lookup must not be treated as proof of sender ownership.

Not implemented: signature verification, sender derivation, canonical transaction decoding, replay protection beyond the unauthenticated nonce comparison, intrinsic gas, recipient/code validation, access-list validation, transaction ordering, cumulative block gas accounting, state mutation, fee debit/refund, receipts, EVM execution, or consensus finality. The state trait does not provide persistence, snapshot isolation, or a state root.

Zero fee remains permitted until the monetary/fee policy is ratified. The block gas limit remains caller-supplied rather than a protocol constant.

## Tests

Tests cover static checks and state-aware success, unknown sender, nonce mismatch, insufficient balance, and value-plus-fee overflow. They do not establish signature validity, persistent-state correctness, cross-node determinism, or execution correctness.

## Next gates

1. Ratify canonical encoding and signing domain.
2. Select reviewed sender/signature algorithms and add verification vectors.
3. Replace the placeholder account identifier with the approved address type.
4. Bind state reads to an immutable, versioned parent-state snapshot.
5. Specify nonce advancement, transaction ordering, intrinsic gas, cumulative block gas and fee settlement.
6. Add replay, property, fuzz, persistence and cross-implementation tests before execution integration.
