//! Deterministic account-state commitments and membership proofs.
//! This remains an account-only prototype, not the complete APEX state root.

use apex_codec::{AccountRecordV1, encode_account_record_v1};
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

/// RFC 6962-style recursive tree shape; APEX domains make this APEX-specific.
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

fn wrap_root(tree_root: Hash32, leaf_count: usize) -> Hash32 {
    sha256(&[
        ROOT_DOMAIN,
        &1_u16.to_be_bytes(),
        &(leaf_count as u64).to_be_bytes(),
        &tree_root.0,
    ])
}

/// Computes the Phase 14 account-only root. Its byte-level construction is unchanged.
pub fn account_state_root_v1(accounts: &BTreeMap<AccountId, AccountState>) -> Hash32 {
    let leaves: Vec<Hash32> = accounts
        .iter()
        .map(|(id, state)| account_leaf_hash(*id, *state))
        .collect();
    wrap_root(merkle_root(&leaves), leaves.len())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SiblingSide {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProofSibling {
    pub hash: Hash32,
    /// The sibling's position relative to the running subtree.
    pub side: SiblingSide,
}

/// Inclusion proof for one canonical account record in an account_state_root_v1 root.
/// Index and count are authenticated indirectly by the exact proof shape and root wrapper.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountMembershipProof {
    pub account: AccountRecordV1,
    pub leaf_index: u64,
    pub leaf_count: u64,
    /// Bottom-up path from the account leaf to the unwrapped tree root.
    pub siblings: Vec<ProofSibling>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProofError {
    EmptyTree,
    IndexOutOfRange,
    CountOutOfRange,
    InvalidShape,
    RootMismatch,
    InvalidNonMembership,
    CountMismatch,
}

fn collect_proof(leaves: &[Hash32], index: usize, out: &mut Vec<ProofSibling>) {
    if leaves.len() <= 1 {
        return;
    }
    let split = largest_power_of_two_less_than(leaves.len());
    if index < split {
        collect_proof(&leaves[..split], index, out);
        out.push(ProofSibling {
            hash: merkle_root(&leaves[split..]),
            side: SiblingSide::Right,
        });
    } else {
        collect_proof(&leaves[split..], index - split, out);
        out.push(ProofSibling {
            hash: merkle_root(&leaves[..split]),
            side: SiblingSide::Left,
        });
    }
}

fn expected_sides(index: usize, count: usize, out: &mut Vec<SiblingSide>) {
    if count <= 1 {
        return;
    }
    let split = largest_power_of_two_less_than(count);
    if index < split {
        expected_sides(index, split, out);
        out.push(SiblingSide::Right);
    } else {
        expected_sides(index - split, count - split, out);
        out.push(SiblingSide::Left);
    }
}

/// Builds a proof for an existing account. Account order is numeric AccountId order.
pub fn account_membership_proof_v1(
    accounts: &BTreeMap<AccountId, AccountState>,
    account_id: AccountId,
) -> Option<AccountMembershipProof> {
    let index = accounts.keys().position(|id| *id == account_id)?;
    let (id, state) = accounts.get_key_value(&account_id)?;
    let leaves: Vec<Hash32> = accounts
        .iter()
        .map(|(id, state)| account_leaf_hash(*id, *state))
        .collect();
    let mut siblings = Vec::new();
    collect_proof(&leaves, index, &mut siblings);
    Some(AccountMembershipProof {
        account: AccountRecordV1 {
            account_id: id.0,
            nonce: state.nonce,
            balance: state.balance,
        },
        leaf_index: index as u64,
        leaf_count: leaves.len() as u64,
        siblings,
    })
}

/// Verifies a membership proof against the exact Phase 14 root construction.
/// Rejects impossible indices/counts, non-canonical path lengths/directions and tampering.
pub fn verify_account_membership_proof_v1(
    proof: &AccountMembershipProof,
    expected_root: Hash32,
) -> Result<(), ProofError> {
    let count = usize::try_from(proof.leaf_count).map_err(|_| ProofError::CountOutOfRange)?;
    let index = usize::try_from(proof.leaf_index).map_err(|_| ProofError::IndexOutOfRange)?;
    if count == 0 {
        return Err(ProofError::EmptyTree);
    }
    if index >= count {
        return Err(ProofError::IndexOutOfRange);
    }

    let mut sides = Vec::new();
    expected_sides(index, count, &mut sides);
    if sides.len() != proof.siblings.len()
        || sides
            .iter()
            .zip(&proof.siblings)
            .any(|(expected, actual)| *expected != actual.side)
    {
        return Err(ProofError::InvalidShape);
    }

    let mut current = sha256(&[LEAF_DOMAIN, &encode_account_record_v1(proof.account)]);
    for sibling in &proof.siblings {
        current = match sibling.side {
            SiblingSide::Left => node_hash(sibling.hash, current),
            SiblingSide::Right => node_hash(current, sibling.hash),
        };
    }
    let root = wrap_root(current, count);
    if root != expected_root {
        return Err(ProofError::RootMismatch);
    }
    Ok(())
}

/// Authenticated absence proof for an AccountId in the numerically ordered account set.
/// Neighbor inclusion proofs bind the predecessor/successor to the same root and count.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccountNonMembershipProof {
    Empty {
        query: AccountId,
    },
    BeforeFirst {
        query: AccountId,
        successor: AccountMembershipProof,
    },
    AfterLast {
        query: AccountId,
        predecessor: AccountMembershipProof,
    },
    Between {
        query: AccountId,
        predecessor: AccountMembershipProof,
        successor: AccountMembershipProof,
    },
}

/// Builds an ordered-neighbor non-membership proof. Returns None if the account exists.
pub fn account_non_membership_proof_v1(
    accounts: &BTreeMap<AccountId, AccountState>,
    query: AccountId,
) -> Option<AccountNonMembershipProof> {
    if accounts.contains_key(&query) {
        return None;
    }
    if accounts.is_empty() {
        return Some(AccountNonMembershipProof::Empty { query });
    }
    let predecessor = accounts
        .range(..query)
        .next_back()
        .and_then(|(id, _)| account_membership_proof_v1(accounts, *id));
    let successor = accounts
        .range((std::ops::Bound::Excluded(query), std::ops::Bound::Unbounded))
        .next()
        .and_then(|(id, _)| account_membership_proof_v1(accounts, *id));
    match (predecessor, successor) {
        (None, Some(successor)) => {
            Some(AccountNonMembershipProof::BeforeFirst { query, successor })
        }
        (Some(predecessor), None) => {
            Some(AccountNonMembershipProof::AfterLast { query, predecessor })
        }
        (Some(predecessor), Some(successor)) => Some(AccountNonMembershipProof::Between {
            query,
            predecessor,
            successor,
        }),
        (None, None) => None,
    }
}

/// Verifies absence using authenticated adjacent neighbors in the ordered account set.
/// The expected root must itself come from a trusted consensus/finality context.
pub fn verify_account_non_membership_proof_v1(
    proof: &AccountNonMembershipProof,
    expected_root: Hash32,
) -> Result<(), ProofError> {
    match proof {
        AccountNonMembershipProof::Empty { .. } => {
            if account_state_root_v1(&BTreeMap::new()) == expected_root {
                Ok(())
            } else {
                Err(ProofError::RootMismatch)
            }
        }
        AccountNonMembershipProof::BeforeFirst { query, successor } => {
            verify_account_membership_proof_v1(successor, expected_root)?;
            if successor.leaf_index != 0 || successor.account.account_id <= query.0 {
                return Err(ProofError::InvalidNonMembership);
            }
            Ok(())
        }
        AccountNonMembershipProof::AfterLast { query, predecessor } => {
            verify_account_membership_proof_v1(predecessor, expected_root)?;
            if predecessor.leaf_index.checked_add(1) != Some(predecessor.leaf_count)
                || predecessor.account.account_id >= query.0
            {
                return Err(ProofError::InvalidNonMembership);
            }
            Ok(())
        }
        AccountNonMembershipProof::Between {
            query,
            predecessor,
            successor,
        } => {
            verify_account_membership_proof_v1(predecessor, expected_root)?;
            verify_account_membership_proof_v1(successor, expected_root)?;
            if predecessor.leaf_count != successor.leaf_count {
                return Err(ProofError::CountMismatch);
            }
            if predecessor.leaf_index.checked_add(1) != Some(successor.leaf_index)
                || predecessor.account.account_id >= query.0
                || successor.account.account_id <= query.0
            {
                return Err(ProofError::InvalidNonMembership);
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(nonce: u64, balance: u128) -> AccountState {
        AccountState { nonce, balance }
    }

    #[test]
    fn phase_14_root_vector_is_unchanged() {
        let accounts = BTreeMap::from([
            (AccountId(9), account(2, 900)),
            (AccountId(1), account(0, 100)),
            (AccountId(5), account(7, 500)),
        ]);
        assert_eq!(
            account_state_root_v1(&accounts).0,
            [
                0xfa, 0x4b, 0x00, 0xc9, 0xb8, 0xee, 0x50, 0xbd, 0xb2, 0xb1, 0x71, 0x09, 0xe6, 0x99,
                0xdf, 0x69, 0xb2, 0x9e, 0x23, 0xcf, 0x78, 0x73, 0x7e, 0x53, 0x67, 0x47, 0x97, 0x91,
                0x0f, 0x31, 0x92, 0xd5,
            ]
        );
    }

    #[test]
    fn inclusion_proofs_verify_for_every_index_across_tree_shapes() {
        for count in 1..=17_u64 {
            let accounts: BTreeMap<_, _> = (0..count)
                .map(|i| (AccountId(i * 3 + 1), account(i, u128::from(i) * 101)))
                .collect();
            let root = account_state_root_v1(&accounts);
            for id in accounts.keys() {
                let proof = account_membership_proof_v1(&accounts, *id).unwrap();
                assert_eq!(verify_account_membership_proof_v1(&proof, root), Ok(()));
            }
        }
    }

    #[test]
    fn absent_account_has_no_membership_proof() {
        let accounts = BTreeMap::from([(AccountId(1), account(0, 10))]);
        assert!(account_membership_proof_v1(&accounts, AccountId(2)).is_none());
        assert!(account_membership_proof_v1(&BTreeMap::new(), AccountId(1)).is_none());
    }

    #[test]
    fn verifier_rejects_tampered_account_sibling_index_and_root() {
        let accounts: BTreeMap<_, _> = (0..5_u64)
            .map(|i| (AccountId(i + 1), account(i, u128::from(i + 10))))
            .collect();
        let root = account_state_root_v1(&accounts);
        let original = account_membership_proof_v1(&accounts, AccountId(3)).unwrap();

        let mut changed_account = original.clone();
        changed_account.account.balance += 1;
        assert_eq!(
            verify_account_membership_proof_v1(&changed_account, root),
            Err(ProofError::RootMismatch)
        );

        let mut changed_sibling = original.clone();
        changed_sibling.siblings[0].hash.0[0] ^= 1;
        assert_eq!(
            verify_account_membership_proof_v1(&changed_sibling, root),
            Err(ProofError::RootMismatch)
        );

        let mut changed_index = original.clone();
        changed_index.leaf_index = 99;
        assert_eq!(
            verify_account_membership_proof_v1(&changed_index, root),
            Err(ProofError::IndexOutOfRange)
        );

        assert_eq!(
            verify_account_membership_proof_v1(&original, Hash32([0; 32])),
            Err(ProofError::RootMismatch)
        );
    }

    #[test]
    fn verifier_rejects_wrong_path_direction_and_length() {
        let accounts: BTreeMap<_, _> = (0..6_u64)
            .map(|i| (AccountId(i), account(i, u128::from(i))))
            .collect();
        let root = account_state_root_v1(&accounts);
        let original = account_membership_proof_v1(&accounts, AccountId(5)).unwrap();

        let mut wrong_side = original.clone();
        wrong_side.siblings[0].side = match wrong_side.siblings[0].side {
            SiblingSide::Left => SiblingSide::Right,
            SiblingSide::Right => SiblingSide::Left,
        };
        assert_eq!(
            verify_account_membership_proof_v1(&wrong_side, root),
            Err(ProofError::InvalidShape)
        );

        let mut short_path = original.clone();
        short_path.siblings.pop();
        assert_eq!(
            verify_account_membership_proof_v1(&short_path, root),
            Err(ProofError::InvalidShape)
        );
    }

    #[test]
    fn verifier_rejects_empty_tree_and_zero_count_proofs() {
        let proof = AccountMembershipProof {
            account: AccountRecordV1 {
                account_id: 1,
                nonce: 0,
                balance: 0,
            },
            leaf_index: 0,
            leaf_count: 0,
            siblings: vec![],
        };
        assert_eq!(
            verify_account_membership_proof_v1(&proof, account_state_root_v1(&BTreeMap::new())),
            Err(ProofError::EmptyTree)
        );
    }
    #[test]
    fn non_membership_proofs_verify_empty_and_all_order_boundaries() {
        let empty = BTreeMap::new();
        let empty_root = account_state_root_v1(&empty);
        let empty_proof = account_non_membership_proof_v1(&empty, AccountId(7)).unwrap();
        assert_eq!(
            verify_account_non_membership_proof_v1(&empty_proof, empty_root),
            Ok(())
        );

        let accounts = BTreeMap::from([
            (AccountId(10), account(0, 100)),
            (AccountId(20), account(1, 200)),
            (AccountId(30), account(2, 300)),
        ]);
        let root = account_state_root_v1(&accounts);

        let before = account_non_membership_proof_v1(&accounts, AccountId(5)).unwrap();
        assert!(matches!(
            before,
            AccountNonMembershipProof::BeforeFirst { .. }
        ));
        assert_eq!(
            verify_account_non_membership_proof_v1(&before, root),
            Ok(())
        );

        let between = account_non_membership_proof_v1(&accounts, AccountId(15)).unwrap();
        assert!(matches!(between, AccountNonMembershipProof::Between { .. }));
        assert_eq!(
            verify_account_non_membership_proof_v1(&between, root),
            Ok(())
        );

        let after = account_non_membership_proof_v1(&accounts, AccountId(40)).unwrap();
        assert!(matches!(after, AccountNonMembershipProof::AfterLast { .. }));
        assert_eq!(verify_account_non_membership_proof_v1(&after, root), Ok(()));
    }

    #[test]
    fn non_membership_generation_refuses_existing_account() {
        let accounts = BTreeMap::from([(AccountId(10), account(0, 100))]);
        assert!(account_non_membership_proof_v1(&accounts, AccountId(10)).is_none());
    }

    #[test]
    fn non_membership_verifier_rejects_non_adjacent_or_reordered_neighbors() {
        let accounts = BTreeMap::from([
            (AccountId(10), account(0, 100)),
            (AccountId(20), account(1, 200)),
            (AccountId(30), account(2, 300)),
        ]);
        let root = account_state_root_v1(&accounts);
        let p10 = account_membership_proof_v1(&accounts, AccountId(10)).unwrap();
        let p30 = account_membership_proof_v1(&accounts, AccountId(30)).unwrap();

        let gap = AccountNonMembershipProof::Between {
            query: AccountId(15),
            predecessor: p10.clone(),
            successor: p30.clone(),
        };
        assert_eq!(
            verify_account_non_membership_proof_v1(&gap, root),
            Err(ProofError::InvalidNonMembership)
        );

        let reversed = AccountNonMembershipProof::Between {
            query: AccountId(25),
            predecessor: p30,
            successor: p10,
        };
        assert_eq!(
            verify_account_non_membership_proof_v1(&reversed, root),
            Err(ProofError::InvalidNonMembership)
        );
    }

    #[test]
    fn non_membership_verifier_rejects_wrong_query_and_wrong_root() {
        let accounts = BTreeMap::from([
            (AccountId(10), account(0, 100)),
            (AccountId(20), account(1, 200)),
        ]);
        let root = account_state_root_v1(&accounts);
        let proof = account_non_membership_proof_v1(&accounts, AccountId(15)).unwrap();
        assert_eq!(
            verify_account_non_membership_proof_v1(&proof, Hash32([0; 32])),
            Err(ProofError::RootMismatch)
        );
        let tampered_query = match proof {
            AccountNonMembershipProof::Between {
                predecessor,
                successor,
                ..
            } => AccountNonMembershipProof::Between {
                query: AccountId(10),
                predecessor,
                successor,
            },
            _ => unreachable!(),
        };
        assert_eq!(
            verify_account_non_membership_proof_v1(&tampered_query, root),
            Err(ProofError::InvalidNonMembership)
        );
    }

    #[test]
    fn non_membership_verifier_rejects_mismatched_neighbor_counts() {
        let accounts_a = BTreeMap::from([
            (AccountId(10), account(0, 100)),
            (AccountId(20), account(1, 200)),
            (AccountId(30), account(2, 300)),
        ]);
        let accounts_b = BTreeMap::from([
            (AccountId(10), account(0, 100)),
            (AccountId(20), account(1, 200)),
            (AccountId(30), account(2, 300)),
            (AccountId(40), account(3, 400)),
        ]);
        let root = account_state_root_v1(&accounts_a);
        let predecessor = account_membership_proof_v1(&accounts_a, AccountId(10)).unwrap();
        let successor = account_membership_proof_v1(&accounts_b, AccountId(20)).unwrap();
        let proof = AccountNonMembershipProof::Between {
            query: AccountId(15),
            predecessor,
            successor,
        };
        assert_eq!(
            verify_account_non_membership_proof_v1(&proof, root),
            Err(ProofError::RootMismatch)
        );
    }
}
