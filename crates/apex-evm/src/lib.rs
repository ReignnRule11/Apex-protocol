//! APEX-owned EVM integration boundary.
//!
//! The full-width state model is an immutable, version-bound read view. It is
//! not yet wired to revm's Database trait or a persistent protocol state backend.

use std::collections::BTreeMap;
use std::fmt;

use revm as _;

pub const REVM_CANDIDATE_VERSION: &str = "43.0.3";

/// Fixed-width 20-byte EVM account address.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EvmAddress(pub [u8; 20]);

/// Unsigned 256-bit integer in big-endian in-memory form.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct U256Word(pub [u8; 32]);

impl U256Word {
    pub const ZERO: Self = Self([0; 32]);

    pub const fn from_be_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn to_be_bytes(self) -> [u8; 32] {
        self.0
    }

    pub const fn from_u64(value: u64) -> Self {
        let source = value.to_be_bytes();
        let mut bytes = [0_u8; 32];
        let mut i = 0;
        while i < 8 {
            bytes[24 + i] = source[i];
            i += 1;
        }
        Self(bytes)
    }
}

/// 32-byte code-content identifier. Hash construction and verification remain
/// protocol decisions and are not performed by this state-model prototype.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EvmCodeHash(pub [u8; 32]);

/// Accounts in the map are present; absence represents a nonexistent account.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvmAccount {
    pub nonce: u64,
    pub balance: U256Word,
    pub code_hash: EvmCodeHash,
}

/// Caller-supplied bounds. No consensus limits are silently selected here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotLimits {
    pub max_accounts: usize,
    pub max_code_bytes: usize,
    pub max_storage_slots: usize,
    pub max_block_hashes: usize,
}

/// Immutable EVM state view anchored to one internal parent-state version.
/// The version is not a block height, state root or wire-format value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvmStateSnapshot {
    parent_state_version: u64,
    accounts: BTreeMap<EvmAddress, EvmAccount>,
    code: BTreeMap<EvmCodeHash, Vec<u8>>,
    storage: BTreeMap<(EvmAddress, U256Word), U256Word>,
    block_hashes: BTreeMap<u64, [u8; 32]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvmStateError {
    ResourceLimitExceeded(&'static str),
    CodeMissing(EvmCodeHash),
    StorageOwnerMissing(EvmAddress),
}

impl fmt::Display for EvmStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ResourceLimitExceeded(resource) => {
                write!(f, "EVM snapshot exceeds configured {resource} limit")
            }
            Self::CodeMissing(hash) => write!(f, "account references unavailable code hash {hash:?}"),
            Self::StorageOwnerMissing(address) => {
                write!(f, "storage is attached to absent account {address:?}")
            }
        }
    }
}

impl std::error::Error for EvmStateError {}

impl EvmStateSnapshot {
    /// Validates and freezes one complete read snapshot.
    ///
    /// Code hashes are checked for referential presence, not recomputed. The
    /// selected fork's hash function and canonical code rules remain open.
    pub fn try_new(
        parent_state_version: u64,
        accounts: BTreeMap<EvmAddress, EvmAccount>,
        code: BTreeMap<EvmCodeHash, Vec<u8>>,
        storage: BTreeMap<(EvmAddress, U256Word), U256Word>,
        block_hashes: BTreeMap<u64, [u8; 32]>,
        limits: SnapshotLimits,
    ) -> Result<Self, EvmStateError> {
        if accounts.len() > limits.max_accounts {
            return Err(EvmStateError::ResourceLimitExceeded("account"));
        }
        if storage.len() > limits.max_storage_slots {
            return Err(EvmStateError::ResourceLimitExceeded("storage-slot"));
        }
        if block_hashes.len() > limits.max_block_hashes {
            return Err(EvmStateError::ResourceLimitExceeded("block-hash"));
        }

        let total_code_bytes = code.values().try_fold(0_usize, |total, bytes| {
            total.checked_add(bytes.len())
        });
        if total_code_bytes.is_none_or(|total| total > limits.max_code_bytes) {
            return Err(EvmStateError::ResourceLimitExceeded("code-byte"));
        }

        for account in accounts.values() {
            if !code.contains_key(&account.code_hash) {
                return Err(EvmStateError::CodeMissing(account.code_hash));
            }
        }
        for (address, _) in storage.keys() {
            if !accounts.contains_key(address) {
                return Err(EvmStateError::StorageOwnerMissing(*address));
            }
        }

        Ok(Self {
            parent_state_version,
            accounts,
            code,
            storage,
            block_hashes,
        })
    }

    pub const fn parent_state_version(&self) -> u64 {
        self.parent_state_version
    }

    pub fn database_view(&self) -> EvmDatabaseView<'_> {
        EvmDatabaseView { snapshot: self }
    }
}

/// Read-only APEX database boundary over one immutable snapshot.
///
/// This does not yet implement revm's Database trait. Engine-facing
/// conversions are deferred until the exact adapter and fork semantics are
/// reviewed; the underlying state representation remains engine-independent.
#[derive(Clone, Copy, Debug)]
pub struct EvmDatabaseView<'a> {
    snapshot: &'a EvmStateSnapshot,
}

impl<'a> EvmDatabaseView<'a> {
    pub const fn parent_state_version(&self) -> u64 {
        self.snapshot.parent_state_version
    }

    pub fn basic(&self, address: EvmAddress) -> Option<&'a EvmAccount> {
        self.snapshot.accounts.get(&address)
    }

    pub fn code_by_hash(&self, code_hash: EvmCodeHash) -> Option<&'a [u8]> {
        self.snapshot.code.get(&code_hash).map(Vec::as_slice)
    }

    /// An absent slot, including one under an absent account, reads as zero.
    pub fn storage(&self, address: EvmAddress, key: U256Word) -> U256Word {
        self.snapshot
            .storage
            .get(&(address, key))
            .copied()
            .unwrap_or(U256Word::ZERO)
    }

    pub fn block_hash(&self, number: u64) -> Option<&'a [u8; 32]> {
        self.snapshot.block_hashes.get(&number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIMITS: SnapshotLimits = SnapshotLimits {
        max_accounts: 8,
        max_code_bytes: 128,
        max_storage_slots: 16,
        max_block_hashes: 256,
    };

    fn address(last: u8) -> EvmAddress {
        let mut bytes = [0_u8; 20];
        bytes[19] = last;
        EvmAddress(bytes)
    }

    fn snapshot() -> EvmStateSnapshot {
        let code_hash = EvmCodeHash([7; 32]);
        EvmStateSnapshot::try_new(
            42,
            BTreeMap::from([(
                address(1),
                EvmAccount {
                    nonce: 9,
                    balance: U256Word([0x80; 32]),
                    code_hash,
                },
            )]),
            BTreeMap::from([(code_hash, vec![0x60, 0x00])]),
            BTreeMap::from([((address(1), U256Word::from_u64(3)), U256Word([0x55; 32]))]),
            BTreeMap::from([(41, [0x41; 32])]),
            LIMITS,
        )
        .unwrap()
    }

    #[test]
    fn full_width_account_storage_and_code_are_preserved() {
        let snapshot = snapshot();
        let view = snapshot.database_view();
        let account = view.basic(address(1)).unwrap();
        assert_eq!(account.nonce, 9);
        assert_eq!(account.balance, U256Word([0x80; 32]));
        assert_eq!(account.code_hash, EvmCodeHash([7; 32]));
        assert_eq!(view.code_by_hash(account.code_hash), Some(&[0x60, 0x00][..]));
        assert_eq!(
            view.storage(address(1), U256Word::from_u64(3)),
            U256Word([0x55; 32])
        );
    }

    #[test]
    fn addresses_are_full_width_and_distinct() {
        let mut left = [0_u8; 20];
        left[0] = 0x80;
        let mut right = [0_u8; 20];
        right[19] = 0x80;
        assert_ne!(EvmAddress(left), EvmAddress(right));
    }

    #[test]
    fn absent_account_and_storage_slot_are_not_fabricated() {
        let snapshot = snapshot();
        let view = snapshot.database_view();
        assert!(view.basic(address(2)).is_none());
        assert_eq!(view.storage(address(1), U256Word::from_u64(99)), U256Word::ZERO);
        assert_eq!(view.storage(address(2), U256Word::from_u64(3)), U256Word::ZERO);
    }

    #[test]
    fn snapshot_is_bound_to_one_parent_version() {
        assert_eq!(snapshot().database_view().parent_state_version(), 42);
    }

    #[test]
    fn missing_account_code_is_rejected() {
        let result = EvmStateSnapshot::try_new(
            1,
            BTreeMap::from([(
                address(1),
                EvmAccount {
                    nonce: 0,
                    balance: U256Word::ZERO,
                    code_hash: EvmCodeHash([9; 32]),
                },
            )]),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
            LIMITS,
        );
        assert_eq!(result, Err(EvmStateError::CodeMissing(EvmCodeHash([9; 32]))));
    }

    #[test]
    fn orphaned_storage_is_rejected() {
        let result = EvmStateSnapshot::try_new(
            1,
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([((address(1), U256Word::ZERO), U256Word::from_u64(1))]),
            BTreeMap::new(),
            LIMITS,
        );
        assert_eq!(result, Err(EvmStateError::StorageOwnerMissing(address(1))));
    }

    #[test]
    fn resource_limits_fail_closed() {
        let mut limits = LIMITS;
        limits.max_accounts = 0;
        let result = EvmStateSnapshot::try_new(
            1,
            BTreeMap::from([(
                address(1),
                EvmAccount {
                    nonce: 0,
                    balance: U256Word::ZERO,
                    code_hash: EvmCodeHash([7; 32]),
                },
            )]),
            BTreeMap::from([(EvmCodeHash([7; 32]), vec![])]),
            BTreeMap::new(),
            BTreeMap::new(),
            limits,
        );
        assert_eq!(result, Err(EvmStateError::ResourceLimitExceeded("account")));
    }

    #[test]
    fn code_bytes_are_bounded_with_checked_total() {
        let mut limits = LIMITS;
        limits.max_code_bytes = 1;
        let result = EvmStateSnapshot::try_new(
            1,
            BTreeMap::new(),
            BTreeMap::from([(EvmCodeHash([7; 32]), vec![0x60, 0x00])]),
            BTreeMap::new(),
            BTreeMap::new(),
            limits,
        );
        assert_eq!(result, Err(EvmStateError::ResourceLimitExceeded("code-byte")));
    }
}

#[cfg(test)]
mod revm_harness_tests {
    use revm::{
        ExecuteCommitEvm, ExecuteEvm, MainBuilder, MainContext,
        bytecode::opcode,
        context::{Context, TxEnv},
        context_interface::result::{ExecutionResult, Output},
        database::CacheDB,
        database_interface::EmptyDB,
        primitives::{Bytes, StorageValue, TxKind, U256},
    };

    const INIT_CODE: &[u8] = &[
        opcode::PUSH1, 0x01, opcode::PUSH1, 0x17, opcode::PUSH1, 0x1f,
        opcode::CODECOPY, opcode::PUSH0, opcode::MLOAD, opcode::PUSH0, opcode::SSTORE,
    ];
    const RETURN_RUNTIME: &[u8] = &[
        opcode::PUSH1, 0x02, opcode::PUSH1, 0x15, opcode::PUSH0, opcode::CODECOPY,
        opcode::PUSH1, 0x02, opcode::PUSH0, opcode::RETURN,
    ];
    const RUNTIME_BYTECODE: &[u8] = &[opcode::PUSH0, opcode::SLOAD];

    fn execute_storage_fixture() -> U256 {
        let init_code: Bytes = [INIT_CODE, RETURN_RUNTIME, RUNTIME_BYTECODE, &[0x42]]
            .concat()
            .into();
        let context = Context::mainnet().with_db(CacheDB::<EmptyDB>::default());
        let mut evm = context.build_mainnet();
        let deployment = evm
            .transact_commit(
                TxEnv::builder()
                    .kind(TxKind::Create)
                    .data(init_code)
                    .build()
                    .expect("valid fixture deployment transaction"),
            )
            .expect("fixture deployment executes");
        let ExecutionResult::Success {
            output: Output::Create(_, Some(contract_address)),
            ..
        } = deployment
        else {
            panic!("fixture contract deployment did not succeed: {deployment:#?}");
        };
        let call = evm
            .transact(
                TxEnv::builder()
                    .kind(TxKind::Call(contract_address))
                    .data(Bytes::new())
                    .nonce(1)
                    .build()
                    .expect("valid fixture call transaction"),
            )
            .expect("fixture call executes");
        assert!(matches!(call.result, ExecutionResult::Success { .. }));
        call.state
            .get(&contract_address)
            .expect("contract state is present in execution result")
            .storage
            .get::<StorageValue>(&U256::ZERO)
            .expect("slot zero is included in execution state diff")
            .present_value()
    }

    #[test]
    fn revm_deploys_contract_and_executes_storage_read() {
        assert_eq!(execute_storage_fixture(), U256::from(0x42_u8));
    }

    #[test]
    fn identical_fresh_contexts_produce_identical_execution_output() {
        let first = execute_storage_fixture();
        let second = execute_storage_fixture();
        assert_eq!(first, second);
        assert_eq!(first, U256::from(0x42_u8));
    }
}
