//! Deterministic account-state commitment prototype.
//! This is not yet the complete APEX state root: AccountId is a u64 prototype
//! identifier and contract storage, code, and system accounts are not modeled.

use apex_codec::{encode_account_record_v1, AccountRecordV1};
use apex_consensus::{AccountId, AccountState};
use apex_types::Hash32;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

const LEAF_DOMAIN: &[u8] = b"APEX/STATE/ACCOUNT/LEAF/V1";
const NODE_DOMAIN: &[u8] = b"APEX/STATE/MERKLE/NODE/V1";
const EMPTY_DOMAIN: &[u8] = b"APEX/STATE/MERKLE/EMPTY/V1";
const ROOT_DOMAIN: &[u8] = b"APEX/STATE/ROOT/V1";

fn sha256(parts: &[&[u8]]) -> Hash32 {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part);
    }
    Hash32(hasher.finalize().into())
}

fn account_leaf_hash(account_id: AccountId, state: AccountState) -> Hash32 {
    let encoded = encode_account_record_v1(AccountRecordV1 {
        account_id: account_id.0,
        nonce: state.nonce,
        balance: state.balance,
    });
    sha256(&[LEAF_DOMAIN, &encoded])
}

fn node_hash(left: Hash32, right: Hash32) -> Hash32 {
    sha256(&[NODE_DOMAIN, &left.0, &right.0])
}

fn largest_power_of_two_less_than(n: usize) -> usize {
    debug_assert!(n > 1);
    let mut power = 1_usize;
    while power < n {
        power <<= 1;
    }
    power >> 1
}

/// RFC 6962-style recursive tree shape; input order is consensus-relevant.
fn merkle_root(leaves: &[Hash32]) -> Hash32 {
    match leaves.len() {
        0 => sha256(&[EMPTY_DOMAIN]),
        1 => leaves[0],
        n => {
            let split = largest_power_of_two_less_than(n);
            node_hash(merkle_root(&leaves[..split]), merkle_root(&leaves[split..]))
        }
    }
}

/// Computes the prototype root for a fixed logical account map.
/// Accounts are ordered by numeric AccountId (BTreeMap order). The commitment
/// binds schema version, caller-supplied state version, account count and tree root.
/// StateVersion is not a block height or a consensus-finality assertion.
pub fn account_state_root_v1(
    state_version: u64,
    accounts: &BTreeMap<AccountId, AccountState>,
) -> Hash32 {
    let leaves: Vec<Hash32> = accounts
        .iter()
        .map(|(id, state)| account_leaf_hash(*id, *state))
        .collect();
    let tree_root = merkle_root(&leaves);
    sha256(&[
        ROOT_DOMAIN,
        &1_u16.to_be_bytes(),
        &state_version.to_be_bytes(),
        &(leaves.len() as u64).to_be_bytes(),
        &tree_root.0,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(nonce: u64, balance: u128) -> AccountState {
        AccountState { nonce, balance }
    }

    #[test]
    fn empty_state_has_stable_nonzero_root() {
        let empty = BTreeMap::new();
        let a = account_state_root_v1(0, &empty);
        let b = account_state_root_v1(0, &empty);
        assert_eq!(a, b);
        assert_ne!(a, Hash32([0; 32]));
    }

    #[test]
    fn insertion_order_does_not_change_root() {
        let a = BTreeMap::from([
            (AccountId(9), account(2, 900)),
            (AccountId(1), account(0, 100)),
            (AccountId(5), account(7, 500)),
        ]);
        let b = BTreeMap::from([
            (AccountId(5), account(7, 500)),
            (AccountId(9), account(2, 900)),
            (AccountId(1), account(0, 100)),
        ]);
        assert_eq!(account_state_root_v1(4, &a), account_state_root_v1(4, &b));
    }

    #[test]
    fn account_or_version_change_changes_root() {
        let base = BTreeMap::from([(AccountId(1), account(0, 100))]);
        let changed = BTreeMap::from([(AccountId(1), account(0, 101))]);
        assert_ne!(account_state_root_v1(4, &base), account_state_root_v1(4, &changed));
        assert_ne!(account_state_root_v1(4, &base), account_state_root_v1(5, &base));
    }

    #[test]
    fn different_leaf_counts_and_empty_state_are_distinguished() {
        let one = BTreeMap::from([(AccountId(1), account(0, 0))]);
        let two = BTreeMap::from([
            (AccountId(1), account(0, 0)),
            (AccountId(2), account(0, 0)),
        ]);
        assert_ne!(account_state_root_v1(0, &BTreeMap::new()), account_state_root_v1(0, &one));
        assert_ne!(account_state_root_v1(0, &one), account_state_root_v1(0, &two));
    }

    #[test]
    fn tree_shape_handles_non_power_of_two_leaf_counts() {
        let leaves: Vec<Hash32> = (0..7_u8).map(|n| sha256(&[b"leaf", &[n]])).collect();
        assert_eq!(merkle_root(&leaves), merkle_root(&leaves));
        assert_ne!(merkle_root(&leaves[..6]), merkle_root(&leaves));
    }
}
