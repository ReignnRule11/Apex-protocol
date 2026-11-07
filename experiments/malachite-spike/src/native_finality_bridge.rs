//! Isolated bridge from Malachite-native finality certificates to APEX block identity.
//!
//! This adapter never reinterprets Malachite signatures as APEX signatures.
//! Native certificate verification is delegated to the pinned Malachite verifier;
//! a separate binding implementation must map the native value ID to the
//! canonical APEX block identity before a verified token can be constructed.

use malachitebft_app_channel::app::types::core::{
    CommitCertificate, Context, Height, Round, ThresholdParams, ValueId,
};
use malachitebft_signing::VerifierExt;

/// APEX identity independently derived from canonical application block bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApexBlockIdentity {
    pub chain_id: [u8; 32],
    pub protocol_version: u32,
    pub height: u64,
    pub parent_hash: [u8; 32],
    pub block_hash: [u8; 32],
}

impl ApexBlockIdentity {
    pub fn validate(&self) -> Result<(), BridgeError> {
        if self.chain_id == [0; 32] || self.protocol_version == 0 {
            return Err(BridgeError::InvalidApexContext);
        }
        if self.block_hash == [0; 32] {
            return Err(BridgeError::NilApexBlockHash);
        }
        Ok(())
    }
}

/// Application-owned deterministic mapping from a native Malachite value ID
/// to the identity recomputed from canonical APEX block bytes.
///
/// Implementations must derive the mapping from canonical bytes and the pinned
/// Malachite value-ID algorithm. They must not trust a caller-provided hash.
pub trait ApexValueIdBinding<Ctx: Context> {
    fn verify_binding(
        &self,
        native_value_id: &ValueId<Ctx>,
        apex_identity: &ApexBlockIdentity,
    ) -> Result<(), ValueBindingError>;
}

/// Stable application commit identity derived only from a verified APEX block.
///
/// Durable application storage must enforce uniqueness on this complete
/// identity and treat retries using the same key as idempotent.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FinalityCommitKey {
    identity: ApexBlockIdentity,
}

impl FinalityCommitKey {
    /// Stable key for idempotent application commits and crash recovery.
    pub const fn commit_key(&self) -> FinalityCommitKey {
        FinalityCommitKey { identity: self.identity }
    }

    pub const fn identity(&self) -> ApexBlockIdentity {
        self.identity
    }
}

/// An opaque proof token. Its fields cannot be constructed outside this crate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerifiedNativeFinality {
    identity: ApexBlockIdentity,
    native_height: u64,
    native_round: Round,
}

impl VerifiedNativeFinality {
    /// Construct a verified token only inside deterministic gate tests.
    #[cfg(test)]
    pub(crate) fn test_only(identity: ApexBlockIdentity) -> Self {
        Self { identity, native_height: identity.height, native_round: Round::new(0) }
    }

    pub const fn identity(&self) -> ApexBlockIdentity {
        self.identity
    }

    pub const fn native_height(&self) -> u64 {
        self.native_height
    }

    pub const fn native_round(&self) -> Round {
        self.native_round
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueBindingError {
    Mismatch,
    UnsupportedValueId,
    NonCanonicalValue,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeError {
    InvalidApexContext,
    NilApexBlockHash,
    WrongNativeHeight,
    WrongNativeRound,
    WrongNativeValue,
    ValueBindingRejected(ValueBindingError),
    NativeCertificateRejected,
}

/// Verify the exact native certificate and bind its decision to one APEX block.
///
/// Callers must first obtain canonical block bytes from a bounded value store,
/// decode them strictly, recompute the APEX block hash and parent linkage, and
/// provide the trusted validator set/thresholds for this exact height. This
/// function does not perform decoding, validator governance, durable commit,
/// or Malachite application-channel acknowledgement.
pub async fn verify_and_bind_native_certificate<Ctx, V, B>(
    context: &Ctx,
    verifier: &V,
    certificate: &CommitCertificate<Ctx>,
    validator_set: &Ctx::ValidatorSet,
    thresholds: ThresholdParams,
    expected_height: Ctx::Height,
    expected_round: Round,
    expected_value_id: &ValueId<Ctx>,
    apex_identity: ApexBlockIdentity,
    value_binding: &B,
) -> Result<VerifiedNativeFinality, BridgeError>
where
    Ctx: Context,
    V: VerifierExt<Ctx> + Sync,
    B: ApexValueIdBinding<Ctx>,
{
    apex_identity.validate()?;

    if certificate.height != expected_height
        || apex_identity.height != expected_height.as_u64()
    {
        return Err(BridgeError::WrongNativeHeight);
    }
    if certificate.round != expected_round {
        return Err(BridgeError::WrongNativeRound);
    }
    if &certificate.value_id != expected_value_id {
        return Err(BridgeError::WrongNativeValue);
    }

    value_binding
        .verify_binding(&certificate.value_id, &apex_identity)
        .map_err(BridgeError::ValueBindingRejected)?;

    verifier
        .verify_commit_certificate(context, certificate, validator_set, thresholds)
        .await
        .map_err(|_| BridgeError::NativeCertificateRejected)?;

    Ok(VerifiedNativeFinality {
        identity: apex_identity,
        native_height: expected_height.as_u64(),
        native_round: expected_round,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apex_identity_rejects_zero_chain_context_and_block_hash() {
        let valid = ApexBlockIdentity {
            chain_id: [1; 32],
            protocol_version: 1,
            height: 7,
            parent_hash: [2; 32],
            block_hash: [3; 32],
        };
        assert_eq!(valid.validate(), Ok(()));

        assert_eq!(
            ApexBlockIdentity { chain_id: [0; 32], ..valid }.validate(),
            Err(BridgeError::InvalidApexContext)
        );
        assert_eq!(
            ApexBlockIdentity { protocol_version: 0, ..valid }.validate(),
            Err(BridgeError::InvalidApexContext)
        );
        assert_eq!(
            ApexBlockIdentity { block_hash: [0; 32], ..valid }.validate(),
            Err(BridgeError::NilApexBlockHash)
        );
    }
}
