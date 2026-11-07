//! Construction of the revm execution engine from explicit APEX inputs.
//!
//! This module wires the Phase 19L environment mapper and the APEX-owned
//! immutable state snapshot into revm 43.0.3's execution builder. It does not
//! decode/authenticate transactions, execute a block, settle fees, or commit
//! the resulting state delta.

use crate::{EvmDatabaseView, EvmStateSnapshot, revm_environment::RevmExecutionEnvironment};
use revm::{
    Context, MainBuilder, MainnetEvm,
    context::{BlockEnv, CfgEnv, TxEnv},
};

/// Builds a revm EVM using only the supplied APEX snapshot and mapped
/// environment.
///
/// `MainBuilder::build_mainnet` selects revm's standard Ethereum instruction
/// and precompile implementation. It does not create a mainnet context or
/// choose a fork: the exact `CfgEnv` and `BlockEnv` produced by the APEX
/// mapper are installed before the builder is invoked. The caller must supply
/// an authenticated/validated snapshot, block context, and fork binding.
pub fn build_revm_evm<'a>(
    snapshot: &'a EvmStateSnapshot,
    environment: RevmExecutionEnvironment,
) -> MainnetEvm<Context<BlockEnv, TxEnv, CfgEnv, EvmDatabaseView<'a>>> {
    let (cfg, block) = environment.into_parts();
    let spec = *cfg.spec();

    Context::<BlockEnv, TxEnv, CfgEnv, EvmDatabaseView<'a>>::new(snapshot.database_view(), spec)
        .with_cfg(cfg)
        .with_block(block)
        .build_mainnet()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EvmAddress, SnapshotLimits, U256Word,
        execution_context::{
            BlockExecutionContext, ForkActivation, ForkActivationSchedule, ForkId,
        },
        revm_environment::{RevmExecutionEnvironment, RevmForkBinding},
    };
    use apex_types::ChainId256;
    use revm::{
        context_interface::{Block, JournalTr},
        primitives::hardfork::SpecId,
    };

    fn environment() -> RevmExecutionEnvironment {
        let fork_id = ForkId::from_bytes([0x19; 32]);
        let schedule =
            ForkActivationSchedule::try_new(vec![ForkActivation::new(0, fork_id, false)]).unwrap();
        let block = BlockExecutionContext::try_from_validated_block(
            ChainId256::from_u64(77),
            &schedule,
            123,
            1_800_000_123,
            30_000_000,
            U256Word::from_u64(9),
            EvmAddress([0xabu8; 20]),
            Some([0x55; 32]),
        )
        .unwrap();
        RevmExecutionEnvironment::try_from_context(
            &block,
            RevmForkBinding::new(fork_id, SpecId::SHANGHAI),
        )
        .unwrap()
    }

    fn empty_snapshot() -> EvmStateSnapshot {
        EvmStateSnapshot::try_new(
            42,
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            SnapshotLimits {
                max_accounts: 8,
                max_code_bytes: 128,
                max_storage_slots: 16,
                max_block_hashes: 256,
            },
        )
        .unwrap()
    }

    #[test]
    fn builder_installs_the_exact_apex_cfg_and_block_environment() {
        let snapshot = empty_snapshot();
        let evm = build_revm_evm(&snapshot, environment());

        assert_eq!(evm.ctx.cfg.chain_id, 77);
        assert_eq!(*evm.ctx.cfg.spec(), SpecId::SHANGHAI);
        assert_eq!(evm.ctx.block.number(), 123_u64.into());
        assert_eq!(evm.ctx.block.timestamp(), 1_800_000_123);
        assert_eq!(evm.ctx.block.gas_limit(), 30_000_000);
        assert_eq!(evm.ctx.block.basefee(), 9);
        assert_eq!(
            evm.ctx.block.beneficiary(),
            revm::primitives::Address::from_slice(&[0xabu8; 20])
        );
        assert_eq!(
            evm.ctx.block.prevrandao(),
            Some(revm::primitives::B256::from([0x55; 32]))
        );
        assert_eq!(evm.ctx.journaled_state.db().parent_state_version(), 42);
    }
}
