//! Explicit mapping from a validated APEX block context into revm 43.0.3.
//!
//! This adapter is intentionally narrower than revm. It supports only the
//! post-Merge, pre-Cancun SpecId range because APEX currently has no validated
//! difficulty, blob-gas, or Amsterdam slot inputs. No default fork is chosen.

use crate::execution_context::{BlockExecutionContext, ForkId};
use revm::{
    context::{block::BlockEnv, cfg::CfgEnv},
    primitives::{hardfork::SpecId, Address, B256, U256},
};
use std::fmt;

/// An explicit, caller-supplied mapping from an APEX fork profile to a revm
/// Ethereum execution-spec identifier. This is configuration, not ratification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RevmForkBinding {
    fork_id: ForkId,
    spec_id: SpecId,
}

impl RevmForkBinding {
    pub const fn new(fork_id: ForkId, spec_id: SpecId) -> Self {
        Self { fork_id, spec_id }
    }

    pub const fn fork_id(self) -> ForkId {
        self.fork_id
    }

    pub const fn spec_id(self) -> SpecId {
        self.spec_id
    }
}

/// Fail-closed errors when APEX inputs cannot be represented by the current
/// revm environment boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RevmEnvironmentError {
    ForkBindingMismatch { context: ForkId, binding: ForkId },
    ChainIdExceedsRevmU64,
    BaseFeeExceedsRevmU64,
    UnsupportedSpecId(SpecId),
    MissingPrevrandaoForPostMergeSpec(SpecId),
}

impl fmt::Display for RevmEnvironmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ForkBindingMismatch { context, binding } => write!(
                f,
                "revm fork binding {binding:?} does not match active APEX fork {context:?}"
            ),
            Self::ChainIdExceedsRevmU64 => {
                write!(f, "APEX 256-bit chain ID cannot be represented by revm's u64 chain ID")
            }
            Self::BaseFeeExceedsRevmU64 => {
                write!(f, "APEX 256-bit base fee cannot be represented by revm's u64 base fee")
            }
            Self::UnsupportedSpecId(spec) => write!(
                f,
                "revm SpecId {spec:?} is outside the explicitly supported adapter range"
            ),
            Self::MissingPrevrandaoForPostMergeSpec(spec) => write!(
                f,
                "revm SpecId {spec:?} requires an explicit prevrandao value"
            ),
        }
    }
}

impl std::error::Error for RevmEnvironmentError {}

/// Immutable pair of revm configuration and block environment values.
#[derive(Clone, Debug)]
pub struct RevmExecutionEnvironment {
    cfg: CfgEnv,
    block: BlockEnv,
}

impl RevmExecutionEnvironment {
    /// Builds revm inputs from an already validated APEX context and an
    /// explicit fork binding. No implicit mainnet context or latest SpecId is
    /// consulted. The selected SpecId uses revm's mainnet gas table as a
    /// temporary execution-semantics candidate, not as ratified APEX policy.
    pub fn try_from_context(
        context: &BlockExecutionContext,
        binding: RevmForkBinding,
    ) -> Result<Self, RevmEnvironmentError> {
        if context.fork_id() != binding.fork_id {
            return Err(RevmEnvironmentError::ForkBindingMismatch {
                context: context.fork_id(),
                binding: binding.fork_id,
            });
        }

        // revm 43.0.3's CfgEnv chain ID is u64. Reject rather than truncate a
        // valid wider APEX identifier.
        let chain_id_bytes = context.chain_id().to_be_bytes();
        if chain_id_bytes[..24].iter().any(|byte| *byte != 0) {
            return Err(RevmEnvironmentError::ChainIdExceedsRevmU64);
        }
        let mut chain_id_tail = [0_u8; 8];
        chain_id_tail.copy_from_slice(&chain_id_bytes[24..]);
        let chain_id = u64::from_be_bytes(chain_id_tail);

        // revm 43.0.3's BlockEnv basefee is u64. Preserve full-width APEX
        // arithmetic elsewhere; this boundary rejects values it cannot hold.
        let base_fee_bytes = context.base_fee_per_gas().to_be_bytes();
        if base_fee_bytes[..24].iter().any(|byte| *byte != 0) {
            return Err(RevmEnvironmentError::BaseFeeExceedsRevmU64);
        }
        let mut base_fee_tail = [0_u8; 8];
        base_fee_tail.copy_from_slice(&base_fee_bytes[24..]);
        let base_fee = u64::from_be_bytes(base_fee_tail);

        // This bounded adapter intentionally supports only post-Merge,
        // pre-Cancun semantics. Earlier forks require a difficulty input;
        // Cancun+ requires blob-gas context; Amsterdam adds slot semantics.
        let spec = binding.spec_id;
        if (spec as u8) < (SpecId::MERGE as u8) || (spec as u8) >= (SpecId::CANCUN as u8) {
            return Err(RevmEnvironmentError::UnsupportedSpecId(spec));
        }
        let prevrandao = context
            .prevrandao()
            .ok_or(RevmEnvironmentError::MissingPrevrandaoForPostMergeSpec(spec))?;

        let mut cfg = CfgEnv::default();
        cfg.set_spec_and_mainnet_gas_params(spec);
        cfg.chain_id = chain_id;

        let mut block = BlockEnv::default();
        block.number = U256::from(context.block_number());
        block.timestamp = U256::from(context.timestamp());
        block.gas_limit = context.gas_limit();
        block.basefee = base_fee;
        block.beneficiary = Address::from_slice(&context.beneficiary().0);
        block.difficulty = U256::ZERO;
        block.prevrandao = Some(B256::from(prevrandao));
        // No blob-gas inputs exist in BlockExecutionContext yet. Do not keep
        // revm's Default blob environment, which would invent blob context.
        block.blob_excess_gas_and_price = None;
        // Slot semantics are not supported by this adapter range.
        block.slot_num = 0;

        Ok(Self { cfg, block })
    }

    pub const fn cfg(&self) -> &CfgEnv {
        &self.cfg
    }

    pub const fn block(&self) -> &BlockEnv {
        &self.block
    }

    pub fn into_parts(self) -> (CfgEnv, BlockEnv) {
        (self.cfg, self.block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_context::{ForkActivation, ForkActivationSchedule};
    use crate::{EvmAddress, U256Word};
    use apex_types::ChainId256;

    fn fork(byte: u8) -> ForkId {
        ForkId::from_bytes([byte; 32])
    }

    fn context(chain_id: ChainId256, base_fee: U256Word, prevrandao: Option<[u8; 32]>) -> BlockExecutionContext {
        let schedule = ForkActivationSchedule::try_new(vec![ForkActivation::new(0, fork(7), false)]).unwrap();
        BlockExecutionContext::try_from_validated_block(
            chain_id,
            &schedule,
            123,
            1_800_000_123,
            30_000_000,
            base_fee,
            EvmAddress([0xabu8; 20]),
            prevrandao,
        )
        .unwrap()
    }

    fn binding(spec: SpecId) -> RevmForkBinding {
        RevmForkBinding::new(fork(7), spec)
    }

    #[test]
    fn maps_every_supported_context_field_without_mainnet_defaults() {
        let randomness = [0x55; 32];
        let context = context(ChainId256::from_u64(77), U256Word::from_u64(9), Some(randomness));
        let env = RevmExecutionEnvironment::try_from_context(&context, binding(SpecId::SHANGHAI)).unwrap();

        assert_eq!(env.cfg.chain_id, 77);
        assert_eq!(env.cfg.spec, SpecId::SHANGHAI);
        assert_eq!(env.block.number, U256::from(123));
        assert_eq!(env.block.timestamp, U256::from(1_800_000_123));
        assert_eq!(env.block.gas_limit, 30_000_000);
        assert_eq!(env.block.basefee, 9);
        assert_eq!(env.block.beneficiary, Address::from_slice(&[0xabu8; 20]));
        assert_eq!(env.block.prevrandao, Some(B256::from(randomness)));
        assert_eq!(env.block.difficulty, U256::ZERO);
        assert!(env.block.blob_excess_gas_and_price.is_none());
    }

    #[test]
    fn rejects_fork_binding_that_does_not_match_context() {
        let context = context(ChainId256::from_u64(77), U256Word::ZERO, Some([1; 32]));
        let error = RevmExecutionEnvironment::try_from_context(
            &context,
            RevmForkBinding::new(fork(8), SpecId::MERGE),
        )
        .unwrap_err();
        assert!(matches!(error, RevmEnvironmentError::ForkBindingMismatch { .. }));
    }

    #[test]
    fn rejects_chain_id_that_revm_would_truncate() {
        let mut bytes = [0; 32];
        bytes[0] = 1;
        let context = context(ChainId256::from_be_bytes(bytes), U256Word::ZERO, Some([1; 32]));
        assert_eq!(
            RevmExecutionEnvironment::try_from_context(&context, binding(SpecId::MERGE)).unwrap_err(),
            RevmEnvironmentError::ChainIdExceedsRevmU64
        );
    }

    #[test]
    fn rejects_base_fee_that_revm_would_truncate() {
        let mut bytes = [0; 32];
        bytes[0] = 1;
        let context = context(ChainId256::from_u64(1), U256Word::from_be_bytes(bytes), Some([1; 32]));
        assert_eq!(
            RevmExecutionEnvironment::try_from_context(&context, binding(SpecId::MERGE)).unwrap_err(),
            RevmEnvironmentError::BaseFeeExceedsRevmU64
        );
    }

    #[test]
    fn rejects_forks_outside_the_bounded_environment_profile() {
        let context = context(ChainId256::from_u64(1), U256Word::ZERO, Some([1; 32]));
        assert_eq!(
            RevmExecutionEnvironment::try_from_context(&context, binding(SpecId::FRONTIER)).unwrap_err(),
            RevmEnvironmentError::UnsupportedSpecId(SpecId::FRONTIER)
        );
        assert_eq!(
            RevmExecutionEnvironment::try_from_context(&context, binding(SpecId::CANCUN)).unwrap_err(),
            RevmEnvironmentError::UnsupportedSpecId(SpecId::CANCUN)
        );
    }

    #[test]
    fn rejects_post_merge_mapping_without_prevrandao() {
        let context = context(ChainId256::from_u64(1), U256Word::ZERO, None);
        assert_eq!(
            RevmExecutionEnvironment::try_from_context(&context, binding(SpecId::MERGE)).unwrap_err(),
            RevmEnvironmentError::MissingPrevrandaoForPostMergeSpec(SpecId::MERGE)
        );
    }
}
