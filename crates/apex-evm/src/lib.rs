//! APEX-owned EVM integration boundary.
//!
//! The dependency is pinned for reproducibility. The test-only execution
//! harness demonstrates deterministic local EVM execution using revm's
//! documented API; it is not a transaction processor, node API, consensus
//! implementation, production state adapter, or approved fork configuration.

use revm as _;

/// Exact upstream crate version under the Phase 19 compatibility spike.
pub const REVM_CANDIDATE_VERSION: &str = "43.0.3";

#[cfg(test)]
mod tests {
    use revm::{
        ExecuteCommitEvm, ExecuteEvm, MainBuilder, MainContext,
        bytecode::opcode,
        context::{Context, TxEnv},
        context_interface::result::{ExecutionResult, Output},
        database::CacheDB,
        database_interface::EmptyDB,
        primitives::{Bytes, StorageValue, TxKind, U256},
    };

    // Minimal init code: copy the final byte into memory and store it in slot 0.
    const INIT_CODE: &[u8] = &[
        opcode::PUSH1,
        0x01,
        opcode::PUSH1,
        0x17,
        opcode::PUSH1,
        0x1f,
        opcode::CODECOPY,
        opcode::PUSH0,
        opcode::MLOAD,
        opcode::PUSH0,
        opcode::SSTORE,
    ];
    const RETURN_RUNTIME: &[u8] = &[
        opcode::PUSH1,
        0x02,
        opcode::PUSH1,
        0x15,
        opcode::PUSH0,
        opcode::CODECOPY,
        opcode::PUSH1,
        0x02,
        opcode::PUSH0,
        opcode::RETURN,
    ];
    const RUNTIME_BYTECODE: &[u8] = &[opcode::PUSH0, opcode::SLOAD];

    /// Deploys a tiny contract into a fresh in-memory EVM, then calls it.
    ///
    /// This deliberately uses revm's mainnet context defaults as a local
    /// fixture only. APEX must replace those defaults with a ratified,
    /// explicit block/fork context before consensus-critical execution.
    fn execute_storage_fixture() -> U256 {
        let expected_value = 0x42_u8;
        let init_code: Bytes = [
            INIT_CODE,
            RETURN_RUNTIME,
            RUNTIME_BYTECODE,
            &[expected_value],
        ]
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
        let storage = call
            .state
            .get(&contract_address)
            .expect("contract state is present in execution result")
            .storage
            .get::<StorageValue>(&U256::ZERO)
            .expect("slot zero is included in execution state diff");

        storage.present_value()
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
