//! Explicit, validated block execution context for deterministic EVM calls.
//!
//! This module defines an APEX-owned context boundary. It deliberately does
//! not choose a production chain ID, fork ID, activation schedule, or revm
//! fork. Those values must come from validated genesis and block data.

use apex_types::ChainId256;
use std::fmt;

use crate::{EvmAddress, U256Word};

/// Opaque identifier for one protocol-approved EVM execution profile.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ForkId([u8; 32]);

impl ForkId {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn to_bytes(self) -> [u8; 32] {
        self.0
    }

    pub const fn is_zero(self) -> bool {
        let mut i = 0;
        while i < 32 {
            if self.0[i] != 0 {
                return false;
            }
            i += 1;
        }
        true
    }
}

/// One configured fork activation. The identifier is opaque; this crate does
/// not infer opcode or precompile semantics from a name or block number.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ForkActivation {
    activation_height: u64,
    fork_id: ForkId,
    requires_prevrandao: bool,
}

impl ForkActivation {
    pub const fn new(activation_height: u64, fork_id: ForkId, requires_prevrandao: bool) -> Self {
        Self {
            activation_height,
            fork_id,
            requires_prevrandao,
        }
    }

    pub const fn activation_height(self) -> u64 {
        self.activation_height
    }

    pub const fn fork_id(self) -> ForkId {
        self.fork_id
    }

    pub const fn requires_prevrandao(self) -> bool {
        self.requires_prevrandao
    }
}

/// Validated, ordered fork activation schedule supplied by protocol
/// configuration. The first entry must activate at genesis (height zero).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForkActivationSchedule {
    activations: Vec<ForkActivation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionContextError {
    ZeroChainId,
    EmptyForkSchedule,
    FirstForkMustActivateAtGenesis,
    ZeroForkId(u64),
    ForkActivationsNotStrictlyIncreasing,
    ZeroGasLimit,
    MissingPrevrandao { height: u64, fork_id: ForkId },
}

impl fmt::Display for ExecutionContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroChainId => write!(f, "execution chain ID must be non-zero"),
            Self::EmptyForkSchedule => write!(f, "fork activation schedule cannot be empty"),
            Self::FirstForkMustActivateAtGenesis => {
                write!(f, "first fork profile must activate at genesis height zero")
            }
            Self::ZeroForkId(height) => {
                write!(f, "fork profile at height {height} has a zero identifier")
            }
            Self::ForkActivationsNotStrictlyIncreasing => {
                write!(f, "fork activation heights must be strictly increasing")
            }
            Self::ZeroGasLimit => write!(f, "block gas limit must be non-zero"),
            Self::MissingPrevrandao { height, fork_id } => write!(
                f,
                "block at height {height} requires prevrandao for fork {fork_id:?}"
            ),
        }
    }
}

impl std::error::Error for ExecutionContextError {}

impl ForkActivationSchedule {
    pub fn try_new(activations: Vec<ForkActivation>) -> Result<Self, ExecutionContextError> {
        let first = activations
            .first()
            .ok_or(ExecutionContextError::EmptyForkSchedule)?;
        if first.activation_height != 0 {
            return Err(ExecutionContextError::FirstForkMustActivateAtGenesis);
        }

        let mut previous_height = None;
        for activation in &activations {
            if activation.fork_id.is_zero() {
                return Err(ExecutionContextError::ZeroForkId(
                    activation.activation_height,
                ));
            }
            if previous_height.is_some_and(|height| height >= activation.activation_height) {
                return Err(ExecutionContextError::ForkActivationsNotStrictlyIncreasing);
            }
            previous_height = Some(activation.activation_height);
        }
        Ok(Self { activations })
    }

    /// Returns the most recent activation at or before the supplied height.
    pub fn active_at(&self, block_height: u64) -> ForkActivation {
        self.activations
            .iter()
            .rev()
            .find(|activation| activation.activation_height <= block_height)
            .copied()
            .expect("validated schedule always has a genesis activation")
    }

    pub fn activations(&self) -> &[ForkActivation] {
        &self.activations
    }
}

/// Complete APEX-owned execution inputs for one validated block.
///
/// Fields are private and exposed read-only. Construction requires a
/// non-zero chain ID, a validated fork schedule, explicit block parameters,
/// and fork-required randomness. No host clock, process randomness, or
/// implicit Ethereum-mainnet defaults are consulted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlockExecutionContext {
    chain_id: ChainId256,
    fork_id: ForkId,
    block_number: u64,
    timestamp: u64,
    gas_limit: u64,
    base_fee_per_gas: U256Word,
    beneficiary: EvmAddress,
    prevrandao: Option<[u8; 32]>,
}

impl BlockExecutionContext {
    #[allow(clippy::too_many_arguments)]
    pub fn try_from_validated_block(
        chain_id: ChainId256,
        schedule: &ForkActivationSchedule,
        block_number: u64,
        timestamp: u64,
        gas_limit: u64,
        base_fee_per_gas: U256Word,
        beneficiary: EvmAddress,
        prevrandao: Option<[u8; 32]>,
    ) -> Result<Self, ExecutionContextError> {
        if chain_id.is_zero() {
            return Err(ExecutionContextError::ZeroChainId);
        }
        if gas_limit == 0 {
            return Err(ExecutionContextError::ZeroGasLimit);
        }
        let active = schedule.active_at(block_number);
        if active.requires_prevrandao && prevrandao.is_none() {
            return Err(ExecutionContextError::MissingPrevrandao {
                height: block_number,
                fork_id: active.fork_id,
            });
        }
        Ok(Self {
            chain_id,
            fork_id: active.fork_id,
            block_number,
            timestamp,
            gas_limit,
            base_fee_per_gas,
            beneficiary,
            prevrandao,
        })
    }

    pub const fn chain_id(&self) -> ChainId256 {
        self.chain_id
    }

    pub const fn fork_id(&self) -> ForkId {
        self.fork_id
    }

    pub const fn block_number(&self) -> u64 {
        self.block_number
    }

    pub const fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub const fn gas_limit(&self) -> u64 {
        self.gas_limit
    }

    pub const fn base_fee_per_gas(&self) -> U256Word {
        self.base_fee_per_gas
    }

    pub const fn beneficiary(&self) -> EvmAddress {
        self.beneficiary
    }

    pub const fn prevrandao(&self) -> Option<[u8; 32]> {
        self.prevrandao
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fork(byte: u8) -> ForkId {
        ForkId::from_bytes([byte; 32])
    }

    fn schedule() -> ForkActivationSchedule {
        ForkActivationSchedule::try_new(vec![
            ForkActivation::new(0, fork(1), false),
            ForkActivation::new(100, fork(2), true),
            ForkActivation::new(250, fork(3), true),
        ])
        .unwrap()
    }

    fn context_at(height: u64, prevrandao: Option<[u8; 32]>) -> BlockExecutionContext {
        BlockExecutionContext::try_from_validated_block(
            ChainId256::from_u64(77),
            &schedule(),
            height,
            1_800_000_000,
            30_000_000,
            U256Word::from_u64(7),
            EvmAddress([9; 20]),
            prevrandao,
        )
        .unwrap()
    }

    #[test]
    fn schedule_selects_fork_at_activation_boundaries() {
        let schedule = schedule();
        assert_eq!(schedule.active_at(0).fork_id(), fork(1));
        assert_eq!(schedule.active_at(99).fork_id(), fork(1));
        assert_eq!(schedule.active_at(100).fork_id(), fork(2));
        assert_eq!(schedule.active_at(249).fork_id(), fork(2));
        assert_eq!(schedule.active_at(250).fork_id(), fork(3));
    }

    #[test]
    fn schedule_rejects_empty_or_non_genesis_start() {
        assert_eq!(
            ForkActivationSchedule::try_new(Vec::new()),
            Err(ExecutionContextError::EmptyForkSchedule)
        );
        assert_eq!(
            ForkActivationSchedule::try_new(vec![ForkActivation::new(1, fork(1), false)]),
            Err(ExecutionContextError::FirstForkMustActivateAtGenesis)
        );
    }

    #[test]
    fn schedule_rejects_zero_or_non_increasing_fork_entries() {
        assert_eq!(
            ForkActivationSchedule::try_new(vec![ForkActivation::new(0, fork(0), false)]),
            Err(ExecutionContextError::ZeroForkId(0))
        );
        assert_eq!(
            ForkActivationSchedule::try_new(vec![
                ForkActivation::new(0, fork(1), false),
                ForkActivation::new(100, fork(2), false),
                ForkActivation::new(100, fork(3), false),
            ]),
            Err(ExecutionContextError::ForkActivationsNotStrictlyIncreasing)
        );
    }

    #[test]
    fn context_captures_explicit_block_fields_and_active_fork() {
        let randomness = [0xabu8; 32];
        let context = context_at(100, Some(randomness));
        assert_eq!(context.chain_id(), ChainId256::from_u64(77));
        assert_eq!(context.fork_id(), fork(2));
        assert_eq!(context.block_number(), 100);
        assert_eq!(context.timestamp(), 1_800_000_000);
        assert_eq!(context.gas_limit(), 30_000_000);
        assert_eq!(context.base_fee_per_gas(), U256Word::from_u64(7));
        assert_eq!(context.beneficiary(), EvmAddress([9; 20]));
        assert_eq!(context.prevrandao(), Some(randomness));
    }

    #[test]
    fn context_rejects_zero_chain_id_and_zero_gas_limit() {
        let schedule = schedule();
        assert_eq!(
            BlockExecutionContext::try_from_validated_block(
                ChainId256::from_u64(0),
                &schedule,
                1,
                1,
                1,
                U256Word::ZERO,
                EvmAddress([0; 20]),
                None,
            ),
            Err(ExecutionContextError::ZeroChainId)
        );
        assert_eq!(
            BlockExecutionContext::try_from_validated_block(
                ChainId256::from_u64(1),
                &schedule,
                1,
                1,
                0,
                U256Word::ZERO,
                EvmAddress([0; 20]),
                None,
            ),
            Err(ExecutionContextError::ZeroGasLimit)
        );
    }

    #[test]
    fn context_requires_randomness_when_active_profile_requires_it() {
        assert_eq!(
            BlockExecutionContext::try_from_validated_block(
                ChainId256::from_u64(1),
                &schedule(),
                100,
                1,
                1,
                U256Word::ZERO,
                EvmAddress([0; 20]),
                None,
            ),
            Err(ExecutionContextError::MissingPrevrandao {
                height: 100,
                fork_id: fork(2)
            })
        );
    }

    #[test]
    fn prevrandao_is_not_required_before_its_activation() {
        let context = context_at(99, None);
        assert_eq!(context.fork_id(), fork(1));
        assert_eq!(context.prevrandao(), None);
    }
}
