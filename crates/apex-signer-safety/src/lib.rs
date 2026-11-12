//! Consensus signing intent reservations and anti-double-sign policy.
//!
//! This crate does not hold keys or create signatures. A reservation must be
//! durably committed by the backing store before an external signer is called.

use apex_types::ChainId256;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ConsensusStep {
    Proposal = 1,
    Prevote = 2,
    Precommit = 3,
}

impl ConsensusStep {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Stable protocol-level signer identity; not a key or public-key encoding.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SignerId([u8; 32]);

impl SignerId {
    pub fn new(bytes: [u8; 32]) -> Result<Self, SigningSafetyError> {
        if bytes == [0; 32] {
            return Err(SigningSafetyError::ZeroSignerId);
        }
        Ok(Self(bytes))
    }

    pub const fn as_bytes(self) -> [u8; 32] {
        self.0
    }
}

/// Complete consensus coordinate protected against conflicting signatures.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SigningCoordinate {
    chain_id: ChainId256,
    protocol_version: u32,
    height: u64,
    round: u32,
    step: ConsensusStep,
    signer: SignerId,
}

impl SigningCoordinate {
    pub fn new(
        chain_id: ChainId256,
        protocol_version: u32,
        height: u64,
        round: u32,
        step: ConsensusStep,
        signer: SignerId,
    ) -> Result<Self, SigningSafetyError> {
        if chain_id.is_zero() {
            return Err(SigningSafetyError::ZeroChainId);
        }
        if protocol_version == 0 {
            return Err(SigningSafetyError::ZeroProtocolVersion);
        }
        Ok(Self {
            chain_id,
            protocol_version,
            height,
            round,
            step,
            signer,
        })
    }

    pub const fn chain_id(self) -> ChainId256 {
        self.chain_id
    }

    pub const fn protocol_version(self) -> u32 {
        self.protocol_version
    }

    pub const fn height(self) -> u64 {
        self.height
    }

    pub const fn round(self) -> u32 {
        self.round
    }

    pub const fn step(self) -> ConsensusStep {
        self.step
    }

    pub const fn signer(self) -> SignerId {
        self.signer
    }

    /// Fixed-width 81-byte key; numeric fields use big-endian encoding.
    pub fn storage_key(self) -> [u8; 81] {
        let mut key = [0_u8; 81];
        key[..32].copy_from_slice(&self.chain_id.to_be_bytes());
        key[32..36].copy_from_slice(&self.protocol_version.to_be_bytes());
        key[36..44].copy_from_slice(&self.height.to_be_bytes());
        key[44..48].copy_from_slice(&self.round.to_be_bytes());
        key[48] = self.step.as_u8();
        key[49..81].copy_from_slice(&self.signer.as_bytes());
        key
    }
}

/// Exact caller-supplied canonical message bytes; no codec is selected here.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SigningIntent {
    coordinate: SigningCoordinate,
    message: Vec<u8>,
}

impl SigningIntent {
    pub fn new(
        coordinate: SigningCoordinate,
        message: Vec<u8>,
        max_message_bytes: usize,
    ) -> Result<Self, SigningSafetyError> {
        if max_message_bytes == 0 {
            return Err(SigningSafetyError::ZeroMessageLimit);
        }
        if message.is_empty() {
            return Err(SigningSafetyError::EmptyMessage);
        }
        if message.len() > max_message_bytes {
            return Err(SigningSafetyError::MessageTooLarge);
        }
        Ok(Self {
            coordinate,
            message,
        })
    }

    pub const fn coordinate(&self) -> SigningCoordinate {
        self.coordinate
    }

    pub fn message(&self) -> &[u8] {
        &self.message
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReservationOutcome {
    Reserved,
    ExistingIdentical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum SigningSafetyError {
    #[error("chain ID must be non-zero")]
    ZeroChainId,
    #[error("protocol version must be non-zero")]
    ZeroProtocolVersion,
    #[error("signer identity must be non-zero")]
    ZeroSignerId,
    #[error("message byte limit must be non-zero")]
    ZeroMessageLimit,
    #[error("signing message must not be empty")]
    EmptyMessage,
    #[error("signing message exceeds its configured byte limit")]
    MessageTooLarge,
    #[error("this consensus coordinate is already reserved for different bytes")]
    DoubleSignPrevented,
    #[error("signing reservation storage is unavailable")]
    StorageUnavailable,
    #[error("signing reservation storage has an unsupported schema")]
    UnsupportedSchema,
    #[error("signing reservation storage is corrupt")]
    CorruptStorage,
}

/// Must atomically persist a first reservation before returning Reserved.
/// Exact retries are idempotent; conflicting bytes must fail closed.
pub trait SigningSafetyStore {
    fn reserve(&mut self, intent: &SigningIntent)
    -> Result<ReservationOutcome, SigningSafetyError>;
}

/// Process-local reference implementation for tests only.
#[derive(Clone, Debug, Default)]
pub struct InMemorySigningSafetyStore {
    reservations: HashMap<SigningCoordinate, Vec<u8>>,
}

impl SigningSafetyStore for InMemorySigningSafetyStore {
    fn reserve(
        &mut self,
        intent: &SigningIntent,
    ) -> Result<ReservationOutcome, SigningSafetyError> {
        match self.reservations.get(&intent.coordinate) {
            Some(existing) if existing == &intent.message => {
                Ok(ReservationOutcome::ExistingIdentical)
            }
            Some(_) => Err(SigningSafetyError::DoubleSignPrevented),
            None => {
                self.reservations
                    .insert(intent.coordinate, intent.message.clone());
                Ok(ReservationOutcome::Reserved)
            }
        }
    }
}

/// External key-custody boundary. Production implementations should call a
/// remote signer, HSM, or hardware-backed key provider; private keys never
/// enter this crate.
pub trait ExternalConsensusSigner {
    type Error;

    fn sign(&mut self, canonical_message: &[u8]) -> Result<Vec<u8>, Self::Error>;
}

/// Errors preserve the distinction between durable reservation failure and
/// external signer failure. A signer is never invoked unless reservation
/// persistence has returned successfully.
#[derive(Debug, PartialEq, Eq)]
pub enum ReserveThenSignError<SignerError> {
    Reservation(SigningSafetyError),
    Signer(SignerError),
}

/// Persist the exact consensus signing intent before invoking external key
/// custody. Conflicting bytes at the same (chain, version, height, round,
/// step, signer) coordinate are rejected before the signer is contacted.
///
/// IMPORTANT: this protects only calls routed through this function and the
/// supplied durable store. Production integration must intercept every
/// consensus signing path and use storage protected against rollback.
pub fn reserve_then_sign<S, K>(
    store: &mut S,
    signer: &mut K,
    intent: &SigningIntent,
) -> Result<(ReservationOutcome, Vec<u8>), ReserveThenSignError<K::Error>>
where
    S: SigningSafetyStore,
    K: ExternalConsensusSigner,
{
    let reservation = store
        .reserve(intent)
        .map_err(ReserveThenSignError::Reservation)?;
    let signature = signer
        .sign(intent.message())
        .map_err(ReserveThenSignError::Signer)?;
    Ok((reservation, signature))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn intent(message: &[u8]) -> SigningIntent {
        let signer = SignerId::new([7; 32]).unwrap();
        let coordinate = SigningCoordinate::new(
            ChainId256::from_u64(42),
            1,
            9,
            2,
            ConsensusStep::Precommit,
            signer,
        )
        .unwrap();
        SigningIntent::new(coordinate, message.to_vec(), 1024).unwrap()
    }

    #[test]
    fn first_reservation_and_exact_retry_are_idempotent() {
        let mut store = InMemorySigningSafetyStore::default();
        assert_eq!(
            store.reserve(&intent(b"vote-a")),
            Ok(ReservationOutcome::Reserved)
        );
        assert_eq!(
            store.reserve(&intent(b"vote-a")),
            Ok(ReservationOutcome::ExistingIdentical)
        );
    }

    #[test]
    fn conflicting_bytes_at_same_coordinate_are_rejected() {
        let mut store = InMemorySigningSafetyStore::default();
        store.reserve(&intent(b"vote-a")).unwrap();
        assert_eq!(
            store.reserve(&intent(b"vote-b")),
            Err(SigningSafetyError::DoubleSignPrevented)
        );
    }

    #[test]
    fn distinct_coordinates_can_be_reserved() {
        let mut store = InMemorySigningSafetyStore::default();
        let first = intent(b"vote-a");
        let coordinate = SigningCoordinate::new(
            first.coordinate.chain_id,
            1,
            10,
            0,
            ConsensusStep::Precommit,
            first.coordinate.signer,
        )
        .unwrap();
        let next = SigningIntent::new(coordinate, b"vote-b".to_vec(), 1024).unwrap();
        assert_eq!(store.reserve(&first), Ok(ReservationOutcome::Reserved));
        assert_eq!(store.reserve(&next), Ok(ReservationOutcome::Reserved));
    }

    #[test]
    fn rejects_zero_context_empty_and_oversized_messages() {
        let signer = SignerId::new([1; 32]).unwrap();
        assert_eq!(
            SigningCoordinate::new(
                ChainId256::from_u64(0),
                1,
                1,
                0,
                ConsensusStep::Proposal,
                signer
            ),
            Err(SigningSafetyError::ZeroChainId)
        );
        let coordinate = intent(b"x").coordinate();
        assert_eq!(
            SigningIntent::new(coordinate, vec![], 1),
            Err(SigningSafetyError::EmptyMessage)
        );
        assert_eq!(
            SigningIntent::new(coordinate, vec![1, 2], 1),
            Err(SigningSafetyError::MessageTooLarge)
        );
    }

    #[test]
    fn storage_key_binds_coordinate_and_is_fixed_width() {
        let first = intent(b"vote-a").coordinate().storage_key();
        let signer = SignerId::new([8; 32]).unwrap();
        let other = SigningCoordinate::new(
            ChainId256::from_u64(42),
            1,
            9,
            2,
            ConsensusStep::Precommit,
            signer,
        )
        .unwrap()
        .storage_key();
        assert_ne!(first, other);
        assert_eq!(first.len(), 81);
    }
}

#[cfg(test)]
mod reserve_then_sign_tests {
    use super::*;
    use apex_types::ChainId256;

    #[derive(Default)]
    struct RecordingSigner {
        calls: usize,
    }

    impl ExternalConsensusSigner for RecordingSigner {
        type Error = ();

        fn sign(&mut self, message: &[u8]) -> Result<Vec<u8>, Self::Error> {
            self.calls += 1;
            Ok(message.iter().map(|byte| byte ^ 0xA5).collect())
        }
    }

    fn intent(message: &[u8]) -> SigningIntent {
        let signer = SignerId::new([9; 32]).unwrap();
        let coordinate = SigningCoordinate::new(
            ChainId256::from_u64(9001),
            1,
            44,
            2,
            ConsensusStep::Precommit,
            signer,
        )
        .unwrap();
        SigningIntent::new(coordinate, message.to_vec(), 4096).unwrap()
    }

    #[test]
    fn reservation_precedes_external_signer_invocation() {
        let mut store = InMemorySigningSafetyStore::default();
        let mut signer = RecordingSigner::default();
        let request = intent(b"canonical-precommit");

        let (outcome, signature) = reserve_then_sign(&mut store, &mut signer, &request).unwrap();

        assert_eq!(outcome, ReservationOutcome::Reserved);
        assert_eq!(signer.calls, 1);
        assert_eq!(
            signature,
            request
                .message()
                .iter()
                .map(|byte| byte ^ 0xA5)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn conflicting_intent_is_rejected_without_calling_signer() {
        let mut store = InMemorySigningSafetyStore::default();
        let mut signer = RecordingSigner::default();
        reserve_then_sign(&mut store, &mut signer, &intent(b"block-a")).unwrap();

        let result = reserve_then_sign(&mut store, &mut signer, &intent(b"block-b"));

        assert_eq!(
            result,
            Err(ReserveThenSignError::Reservation(
                SigningSafetyError::DoubleSignPrevented
            ))
        );
        assert_eq!(signer.calls, 1);
    }

    #[test]
    fn signer_failure_keeps_the_reservation_in_place() {
        struct FailingSigner;
        impl ExternalConsensusSigner for FailingSigner {
            type Error = &'static str;

            fn sign(&mut self, _: &[u8]) -> Result<Vec<u8>, Self::Error> {
                Err("HSM unavailable")
            }
        }

        let mut store = InMemorySigningSafetyStore::default();
        let request = intent(b"reserved-before-hsm");
        assert_eq!(
            reserve_then_sign(&mut store, &mut FailingSigner, &request),
            Err(ReserveThenSignError::Signer("HSM unavailable"))
        );

        let mut available = RecordingSigner::default();
        assert_eq!(
            reserve_then_sign(&mut store, &mut available, &request)
                .unwrap()
                .0,
            ReservationOutcome::ExistingIdentical
        );
        assert_eq!(available.calls, 1);
    }
}
