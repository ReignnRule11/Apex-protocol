//! Bounded mapping of an already authenticated type-2 transaction into revm.
//!
//! This module does not decode RLP, verify signatures, recover senders, or
//! establish that its input is authentic. Its caller must supply a sender
//! derived by the separately reviewed authentication pipeline.

use crate::revm_environment::RevmExecutionEnvironment;
use crate::{EvmAddress, U256Word};
use revm::{
    context::TxEnv,
    primitives::{AccessList, AccessListItem, Address, Bytes, TxKind, U256},
};
use std::fmt;

/// Resource bounds are explicit caller policy; no consensus limits are implied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransactionEnvironmentLimits {
    pub max_input_bytes: usize,
    pub max_access_list_addresses: usize,
    pub max_access_list_storage_keys: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Type2AccessListEntry {
    pub address: EvmAddress,
    pub storage_keys: Vec<U256Word>,
}

/// Execution-facing fields from an upstream-authenticated type-2 envelope.
/// This is not a canonical signed envelope and contains no signature or tx ID.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticatedType2Transaction {
    chain_id: u64,
    caller: EvmAddress,
    nonce: u64,
    max_priority_fee_per_gas: u128,
    max_fee_per_gas: u128,
    gas_limit: u64,
    destination: Option<EvmAddress>,
    value: U256Word,
    input: Vec<u8>,
    access_list: Vec<Type2AccessListEntry>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransactionEnvironmentError {
    ZeroChainId,
    ChainIdMismatch { transaction: u64, execution: u64 },
    ZeroGasLimit,
    PriorityFeeExceedsMaxFee,
    InputTooLarge { actual: usize, maximum: usize },
    TooManyAccessListAddresses { actual: usize, maximum: usize },
    TooManyAccessListStorageKeys { actual: usize, maximum: usize },
    AccessListStorageKeyCountOverflow,
}

impl fmt::Display for TransactionEnvironmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroChainId => write!(f, "type-2 transaction chain ID must be nonzero"),
            Self::ChainIdMismatch {
                transaction,
                execution,
            } => write!(
                f,
                "transaction chain ID {transaction} does not match execution chain ID {execution}"
            ),
            Self::ZeroGasLimit => write!(f, "transaction gas limit must be nonzero"),
            Self::PriorityFeeExceedsMaxFee => write!(f, "priority fee exceeds max fee per gas"),
            Self::InputTooLarge { actual, maximum } => write!(
                f,
                "transaction input length {actual} exceeds configured maximum {maximum}"
            ),
            Self::TooManyAccessListAddresses { actual, maximum } => write!(
                f,
                "access-list address count {actual} exceeds configured maximum {maximum}"
            ),
            Self::TooManyAccessListStorageKeys { actual, maximum } => write!(
                f,
                "access-list storage-key count {actual} exceeds configured maximum {maximum}"
            ),
            Self::AccessListStorageKeyCountOverflow => {
                write!(f, "access-list storage-key count overflowed usize")
            }
        }
    }
}

impl std::error::Error for TransactionEnvironmentError {}

impl AuthenticatedType2Transaction {
    /// Caller must have verified the canonical signed envelope and recovered
    /// caller before constructing this execution input. This API does not
    /// itself prove that the authentication step occurred.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        chain_id: u64,
        caller: EvmAddress,
        nonce: u64,
        max_priority_fee_per_gas: u128,
        max_fee_per_gas: u128,
        gas_limit: u64,
        destination: Option<EvmAddress>,
        value: U256Word,
        input: Vec<u8>,
        access_list: Vec<Type2AccessListEntry>,
        limits: TransactionEnvironmentLimits,
    ) -> Result<Self, TransactionEnvironmentError> {
        if chain_id == 0 {
            return Err(TransactionEnvironmentError::ZeroChainId);
        }
        if gas_limit == 0 {
            return Err(TransactionEnvironmentError::ZeroGasLimit);
        }
        if max_priority_fee_per_gas > max_fee_per_gas {
            return Err(TransactionEnvironmentError::PriorityFeeExceedsMaxFee);
        }
        if input.len() > limits.max_input_bytes {
            return Err(TransactionEnvironmentError::InputTooLarge {
                actual: input.len(),
                maximum: limits.max_input_bytes,
            });
        }
        if access_list.len() > limits.max_access_list_addresses {
            return Err(TransactionEnvironmentError::TooManyAccessListAddresses {
                actual: access_list.len(),
                maximum: limits.max_access_list_addresses,
            });
        }
        let key_count = access_list
            .iter()
            .try_fold(0_usize, |n, entry| n.checked_add(entry.storage_keys.len()))
            .ok_or(TransactionEnvironmentError::AccessListStorageKeyCountOverflow)?;
        if key_count > limits.max_access_list_storage_keys {
            return Err(TransactionEnvironmentError::TooManyAccessListStorageKeys {
                actual: key_count,
                maximum: limits.max_access_list_storage_keys,
            });
        }
        Ok(Self {
            chain_id,
            caller,
            nonce,
            max_priority_fee_per_gas,
            max_fee_per_gas,
            gas_limit,
            destination,
            value,
            input,
            access_list,
        })
    }

    pub const fn chain_id(&self) -> u64 {
        self.chain_id
    }

    /// Convert to revm TxEnv only when the transaction and explicit block
    /// execution environment carry the same chain ID. State admission,
    /// affordability and execution remain separate stages.
    pub fn to_revm_tx_env(
        &self,
        environment: &RevmExecutionEnvironment,
    ) -> Result<TxEnv, TransactionEnvironmentError> {
        let execution_chain_id = environment.cfg().chain_id;
        if self.chain_id != execution_chain_id {
            return Err(TransactionEnvironmentError::ChainIdMismatch {
                transaction: self.chain_id,
                execution: execution_chain_id,
            });
        }
        let access_list = self
            .access_list
            .iter()
            .map(|entry| AccessListItem {
                address: Address::from_slice(&entry.address.0),
                storage_keys: entry
                    .storage_keys
                    .iter()
                    .map(|key| U256::from_be_bytes(key.0))
                    .collect(),
            })
            .collect::<AccessList>();

        Ok(TxEnv {
            tx_type: 0x02,
            caller: Address::from_slice(&self.caller.0),
            gas_limit: self.gas_limit,
            gas_price: self.max_fee_per_gas,
            kind: self.destination.map_or(TxKind::Create, |address| {
                TxKind::Call(Address::from_slice(&address.0))
            }),
            value: U256::from_be_bytes(self.value.0),
            data: Bytes::copy_from_slice(&self.input),
            nonce: self.nonce,
            chain_id: Some(self.chain_id),
            access_list,
            gas_priority_fee: Some(self.max_priority_fee_per_gas),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        execution_context::{
            BlockExecutionContext, ForkActivation, ForkActivationSchedule, ForkId,
        },
        revm_environment::{RevmExecutionEnvironment, RevmForkBinding},
    };
    use apex_types::ChainId256;
    use revm::primitives::hardfork::SpecId;

    const LIMITS: TransactionEnvironmentLimits = TransactionEnvironmentLimits {
        max_input_bytes: 1024,
        max_access_list_addresses: 8,
        max_access_list_storage_keys: 16,
    };

    fn environment(chain_id: u64) -> RevmExecutionEnvironment {
        let fork_id = ForkId::from_bytes([0x19; 32]);
        let schedule =
            ForkActivationSchedule::try_new(vec![ForkActivation::new(0, fork_id, false)]).unwrap();
        let block = BlockExecutionContext::try_from_validated_block(
            ChainId256::from_u64(chain_id),
            &schedule,
            1,
            1_800_000_001,
            30_000_000,
            U256Word::from_u64(1),
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

    fn transaction(destination: Option<EvmAddress>) -> AuthenticatedType2Transaction {
        AuthenticatedType2Transaction::try_new(
            77,
            EvmAddress([0x11; 20]),
            4,
            2,
            10,
            50_000,
            destination,
            U256Word::from_u64(123),
            vec![0x60, 0x00],
            vec![Type2AccessListEntry {
                address: EvmAddress([0x22; 20]),
                storage_keys: vec![U256Word::from_u64(3), U256Word::from_u64(4)],
            }],
            LIMITS,
        )
        .unwrap()
    }

    #[test]
    fn maps_type2_call_fields_and_access_list() {
        let env = transaction(Some(EvmAddress([0x33; 20])))
            .to_revm_tx_env(&environment(77))
            .unwrap();
        assert_eq!(env.tx_type, 0x02);
        assert_eq!(env.caller, Address::from_slice(&[0x11; 20]));
        assert_eq!(env.gas_limit, 50_000);
        assert_eq!(env.gas_price, 10);
        assert_eq!(env.gas_priority_fee, Some(2));
        assert_eq!(env.nonce, 4);
        assert_eq!(env.chain_id, Some(77));
        assert_eq!(env.value, U256::from(123));
        assert_eq!(env.data.as_ref(), &[0x60, 0x00]);
        assert_eq!(env.access_list.len(), 1);
        assert_eq!(
            env.access_list[0].storage_keys,
            vec![U256::from(3), U256::from(4)]
        );
    }

    #[test]
    fn maps_contract_creation() {
        let env = transaction(None).to_revm_tx_env(&environment(77)).unwrap();
        assert_eq!(env.kind, TxKind::Create);
    }

    #[test]
    fn rejects_chain_id_mismatch() {
        assert_eq!(
            transaction(None)
                .to_revm_tx_env(&environment(78))
                .unwrap_err(),
            TransactionEnvironmentError::ChainIdMismatch {
                transaction: 77,
                execution: 78
            },
        );
    }

    #[test]
    fn rejects_invalid_fee_ordering_and_zero_gas_limit() {
        assert_eq!(
            AuthenticatedType2Transaction::try_new(
                77,
                EvmAddress([1; 20]),
                0,
                11,
                10,
                1,
                None,
                U256Word::ZERO,
                vec![],
                vec![],
                LIMITS,
            )
            .unwrap_err(),
            TransactionEnvironmentError::PriorityFeeExceedsMaxFee,
        );
        assert_eq!(
            AuthenticatedType2Transaction::try_new(
                77,
                EvmAddress([1; 20]),
                0,
                1,
                10,
                0,
                None,
                U256Word::ZERO,
                vec![],
                vec![],
                LIMITS,
            )
            .unwrap_err(),
            TransactionEnvironmentError::ZeroGasLimit,
        );
    }

    #[test]
    fn enforces_input_and_access_list_bounds() {
        assert!(matches!(
            AuthenticatedType2Transaction::try_new(
                77,
                EvmAddress([1; 20]),
                0,
                1,
                10,
                1,
                None,
                U256Word::ZERO,
                vec![0; 1025],
                vec![],
                LIMITS,
            )
            .unwrap_err(),
            TransactionEnvironmentError::InputTooLarge { .. },
        ));
        assert!(matches!(
            AuthenticatedType2Transaction::try_new(
                77,
                EvmAddress([1; 20]),
                0,
                1,
                10,
                1,
                None,
                U256Word::ZERO,
                vec![],
                vec![Type2AccessListEntry {
                    address: EvmAddress([2; 20]),
                    storage_keys: vec![U256Word::ZERO; 17],
                }],
                LIMITS,
            )
            .unwrap_err(),
            TransactionEnvironmentError::TooManyAccessListStorageKeys { .. },
        ));
    }
}
