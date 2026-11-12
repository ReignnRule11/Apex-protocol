//! Malachite signer decorator that reserves every consensus signing intent.
//!
//! This remains in the isolated Malachite spike until application-specific
//! canonical-byte adapters and durable rollback protection are reviewed.

use apex_signer_safety::{ConsensusStep, SigningIntent, SigningSafetyError, SigningSafetyStore};
use async_trait::async_trait;
use malachitebft_core_types::{Context, Vote, VoteExtensionScope};
use malachitebft_signing::{Error, Signer};
use std::marker::PhantomData;
use std::sync::Mutex;

/// Application-specific bridge from Malachite messages to the exact APEX
/// reservation coordinate and the exact bytes that Malachite will sign.
///
/// Implementations MUST use the same canonical preimage as the inner
/// Malachite signer. Vote extensions must bind the extension to the precommit
/// scope and use the same domain-separated envelope as the configured verifier.
pub trait SigningIntentFactory<Ctx: Context>: Send + Sync {
    fn vote_intent(&self, vote: &Ctx::Vote) -> Result<SigningIntent, SigningSafetyError>;
    fn proposal_intent(
        &self,
        proposal: &Ctx::Proposal,
    ) -> Result<SigningIntent, SigningSafetyError>;
    fn vote_extension_intent(
        &self,
        scope: &VoteExtensionScope<Ctx>,
        extension: &Ctx::Extension,
    ) -> Result<SigningIntent, SigningSafetyError>;
}

/// Decorates Malachite signing with a durable APEX reservation.
///
/// The reservation lock is released before awaiting the underlying signer.
/// The store must be durable and independently rollback-protected in
/// production; the current redb store alone does not provide that guarantee.
/// Validator proofs are delegated unchanged because they are network-identity
/// proofs, not height/round consensus messages.
pub struct ReservedMalachiteSigner<Ctx, Inner, Store, Factory>
where
    Ctx: Context,
{
    inner: Inner,
    store: Mutex<Store>,
    factory: Factory,
    _context: PhantomData<fn() -> Ctx>,
}

impl<Ctx, Inner, Store, Factory> ReservedMalachiteSigner<Ctx, Inner, Store, Factory>
where
    Ctx: Context,
    Store: SigningSafetyStore,
{
    pub fn new(inner: Inner, store: Store, factory: Factory) -> Self {
        Self {
            inner,
            store: Mutex::new(store),
            factory,
            _context: PhantomData,
        }
    }

    pub fn inner(&self) -> &Inner {
        &self.inner
    }

    fn reserve(&self, intent: SigningIntent, expected_step: ConsensusStep) -> Result<(), Error> {
        if intent.coordinate().step() != expected_step {
            return Err(Error::from_source(format!(
                "APEX signing adapter returned {:?} coordinate for {:?} request",
                intent.coordinate().step(),
                expected_step
            )));
        }

        let mut store = self.store.lock().map_err(|_| {
            Error::from_source("APEX signing reservation mutex is poisoned".to_owned())
        })?;

        store.reserve(&intent).map(|_| ()).map_err(|error| {
            Error::from_source(format!("APEX signing reservation rejected: {error}"))
        })
    }
}

#[async_trait]
impl<Ctx, Inner, Store, Factory> Signer<Ctx> for ReservedMalachiteSigner<Ctx, Inner, Store, Factory>
where
    Ctx: Context,
    Inner: Signer<Ctx>,
    Store: SigningSafetyStore + Send,
    Factory: SigningIntentFactory<Ctx>,
{
    async fn sign_vote(
        &self,
        vote: Ctx::Vote,
    ) -> Result<malachitebft_core_types::SignedVote<Ctx>, Error> {
        let step = match vote.vote_type() {
            malachitebft_core_types::VoteType::Prevote => ConsensusStep::Prevote,
            malachitebft_core_types::VoteType::Precommit => ConsensusStep::Precommit,
        };
        let intent = self
            .factory
            .vote_intent(&vote)
            .map_err(|error| Error::from_source(error.to_string()))?;
        self.reserve(intent, step)?;
        self.inner.sign_vote(vote).await
    }

    async fn sign_proposal(
        &self,
        proposal: Ctx::Proposal,
    ) -> Result<malachitebft_core_types::SignedProposal<Ctx>, Error> {
        let intent = self
            .factory
            .proposal_intent(&proposal)
            .map_err(|error| Error::from_source(error.to_string()))?;
        self.reserve(intent, ConsensusStep::Proposal)?;
        self.inner.sign_proposal(proposal).await
    }

    async fn sign_vote_extension(
        &self,
        scope: VoteExtensionScope<Ctx>,
        extension: Ctx::Extension,
    ) -> Result<malachitebft_core_types::SignedExtension<Ctx>, Error> {
        let intent = self
            .factory
            .vote_extension_intent(&scope, &extension)
            .map_err(|error| Error::from_source(error.to_string()))?;
        self.reserve(intent, ConsensusStep::Precommit)?;
        self.inner.sign_vote_extension(scope, extension).await
    }

    async fn sign_validator_proof(
        &self,
        public_key: Vec<u8>,
        peer_id: Vec<u8>,
    ) -> Result<malachitebft_core_types::ValidatorProof<Ctx>, Error> {
        self.inner.sign_validator_proof(public_key, peer_id).await
    }
}
