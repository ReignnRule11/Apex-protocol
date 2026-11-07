//! Engine-neutral boundary between consensus and deterministic application.
//! This is orchestration only: no certificate verifier, canonical codec, durable
//! replay protection, production execution, or atomic cross-system commit exists.

use apex_types::{ChainId256, Hash32, Height};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChainContext { chain_id: ChainId256, protocol_version: u32 }

impl ChainContext {
    pub fn new(chain_id: ChainId256, protocol_version: u32) -> Result<Self, BoundaryError> {
        if chain_id.is_zero() { return Err(BoundaryError::ZeroChainId); }
        if protocol_version == 0 { return Err(BoundaryError::ZeroProtocolVersion); }
        Ok(Self { chain_id, protocol_version })
    }
    pub const fn chain_id(self) -> ChainId256 { self.chain_id }
    pub const fn protocol_version(self) -> u32 { self.protocol_version }
}

/// Caller-configured resource bounds, not protocol constants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoundaryLimits { max_proposal_bytes: usize, max_finality_proof_bytes: usize }

impl BoundaryLimits {
    pub fn new(max_proposal_bytes: usize, max_finality_proof_bytes: usize) -> Result<Self, BoundaryError> {
        if max_proposal_bytes == 0 || max_finality_proof_bytes == 0 {
            return Err(BoundaryError::ZeroByteLimit);
        }
        Ok(Self { max_proposal_bytes, max_finality_proof_bytes })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParentBlock { context: ChainContext, height: Height, block_hash: Hash32 }

impl ParentBlock {
    pub const fn new(context: ChainContext, height: Height, block_hash: Hash32) -> Self {
        Self { context, height, block_hash }
    }
    pub const fn context(self) -> ChainContext { self.context }
    pub const fn height(self) -> Height { self.height }
    pub const fn block_hash(self) -> Hash32 { self.block_hash }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlockIdentity {
    context: ChainContext,
    height: Height,
    parent_hash: Hash32,
    block_hash: Hash32,
}

impl BlockIdentity {
    pub fn new(context: ChainContext, height: Height, parent_hash: Hash32, block_hash: Hash32) -> Result<Self, BoundaryError> {
        if block_hash == Hash32([0; 32]) { return Err(BoundaryError::ZeroBlockHash); }
        Ok(Self { context, height, parent_hash, block_hash })
    }
    pub const fn context(self) -> ChainContext { self.context }
    pub const fn height(self) -> Height { self.height }
    pub const fn parent_hash(self) -> Hash32 { self.parent_hash }
    pub const fn block_hash(self) -> Hash32 { self.block_hash }
}

/// Opaque payload. Application validation must perform canonical decoding and
/// derive/check the block hash; this type does not define a wire format.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalCandidate { identity: BlockIdentity, payload: Vec<u8> }

impl ProposalCandidate {
    pub fn new(identity: BlockIdentity, payload: Vec<u8>, limits: BoundaryLimits) -> Result<Self, BoundaryError> {
        if payload.is_empty() { return Err(BoundaryError::EmptyProposal); }
        if payload.len() > limits.max_proposal_bytes { return Err(BoundaryError::ProposalTooLarge); }
        Ok(Self { identity, payload })
    }
    pub const fn identity(&self) -> BlockIdentity { self.identity }
    pub fn payload(&self) -> &[u8] { &self.payload }
}

/// Opaque engine certificate bytes bound to one exact block identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinalityClaim { identity: BlockIdentity, proof: Vec<u8> }

impl FinalityClaim {
    pub fn new(identity: BlockIdentity, proof: Vec<u8>, limits: BoundaryLimits) -> Result<Self, BoundaryError> {
        if proof.is_empty() { return Err(BoundaryError::EmptyFinalityProof); }
        if proof.len() > limits.max_finality_proof_bytes { return Err(BoundaryError::FinalityProofTooLarge); }
        Ok(Self { identity, proof })
    }
    pub const fn identity(&self) -> BlockIdentity { self.identity }
    pub fn proof(&self) -> &[u8] { &self.proof }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum ApplicationError {
    #[error("proposal is invalid or non-canonical")] InvalidProposal,
    #[error("deterministic execution or protocol validation failed")] ExecutionRejected,
    #[error("required parent state is unavailable")] ParentStateUnavailable,
    #[error("application commit conflicts with the expected parent")] CommitConflict,
    #[error("application commit failed")] CommitFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum FinalityVerificationError {
    #[error("finality proof is invalid")] InvalidProof,
    #[error("certificate profile is unsupported")] UnsupportedCertificate,
    #[error("validator-set context is unavailable")] ValidatorSetUnavailable,
    #[error("finality verification is unavailable")] VerificationUnavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum BoundaryError {
    #[error("chain ID must be non-zero")] ZeroChainId,
    #[error("protocol version must be non-zero")] ZeroProtocolVersion,
    #[error("byte limits must be non-zero")] ZeroByteLimit,
    #[error("block hash must be non-zero")] ZeroBlockHash,
    #[error("proposal payload must not be empty")] EmptyProposal,
    #[error("proposal payload exceeds its configured byte limit")] ProposalTooLarge,
    #[error("finality proof must not be empty")] EmptyFinalityProof,
    #[error("finality proof exceeds its configured byte limit")] FinalityProofTooLarge,
    #[error("parent chain context does not match configured context")] ParentContextMismatch,
    #[error("proposal chain context does not match configured context")] ProposalContextMismatch,
    #[error("parent height cannot be incremented")] ParentHeightOverflow,
    #[error("proposal is not at the next height")] WrongProposalHeight,
    #[error("proposal parent hash does not match committed parent")] WrongParentHash,
    #[error("finality claim does not identify the validated proposal")] FinalityIdentityMismatch,
    #[error("application callback failed: {0}")] Application(ApplicationError),
    #[error("finality verifier failed: {0}")] Finality(FinalityVerificationError),
}

/// Must perform complete deterministic proposal validation against this parent.
/// No wall-clock, randomness, network calls or nondeterministic iteration.
///
/// Commit implementations must be atomic, idempotent by block identity and
/// reject stale parents. This trait cannot enforce those storage guarantees.
pub trait ApplicationAdapter {
    fn validate_proposal(&self, parent: ParentBlock, proposal: &ProposalCandidate) -> Result<(), ApplicationError>;
    fn commit_finalized(&mut self, block: &VerifiedFinalizedBlock) -> Result<(), ApplicationError>;
}

/// Engine-specific verifier must authenticate chain, protocol, height, block
/// hash, certificate domain, validator set, voting power and signature rules.
/// No production verifier is implemented by this phase.
pub trait FinalityVerifier {
    fn verify_finality(&self, claim: &FinalityClaim) -> Result<(), FinalityVerificationError>;
}

/// A candidate accepted by linkage checks and the application's validation callback.
#[derive(Debug, Eq, PartialEq)]
pub struct ValidatedProposal { parent: ParentBlock, proposal: ProposalCandidate }

impl ValidatedProposal {
    pub const fn identity(&self) -> BlockIdentity { self.proposal.identity }
    pub const fn parent(&self) -> ParentBlock { self.parent }
}

/// Constructed only after a verifier accepts a claim for the exact validated block.
#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedFinalizedBlock { parent: ParentBlock, identity: BlockIdentity, payload: Vec<u8> }

impl VerifiedFinalizedBlock {
    pub const fn parent(&self) -> ParentBlock { self.parent }
    pub const fn identity(&self) -> BlockIdentity { self.identity }
    pub fn payload(&self) -> &[u8] { &self.payload }
}

/// Checks identity/linkage and invokes full application validation before a
/// proposal can be represented as validated.
pub fn validate_proposal<A: ApplicationAdapter>(
    context: ChainContext, parent: ParentBlock, proposal: ProposalCandidate, application: &A,
) -> Result<ValidatedProposal, BoundaryError> {
    if parent.context != context { return Err(BoundaryError::ParentContextMismatch); }
    if proposal.identity.context != context { return Err(BoundaryError::ProposalContextMismatch); }
    let next = parent.height.checked_add(1).ok_or(BoundaryError::ParentHeightOverflow)?;
    if proposal.identity.height != next { return Err(BoundaryError::WrongProposalHeight); }
    if proposal.identity.parent_hash != parent.block_hash { return Err(BoundaryError::WrongParentHash); }
    application.validate_proposal(parent, &proposal).map_err(BoundaryError::Application)?;
    Ok(ValidatedProposal { parent, proposal })
}

/// Verifies finality for the exact previously validated proposal, then calls
/// application commit. Failed verification can never reach commit.
pub fn finalize_proposal<A: ApplicationAdapter, V: FinalityVerifier>(
    validated: ValidatedProposal, claim: FinalityClaim, verifier: &V, application: &mut A,
) -> Result<VerifiedFinalizedBlock, BoundaryError> {
    if claim.identity != validated.proposal.identity { return Err(BoundaryError::FinalityIdentityMismatch); }
    verifier.verify_finality(&claim).map_err(BoundaryError::Finality)?;
    let finalized = VerifiedFinalizedBlock {
        parent: validated.parent,
        identity: validated.proposal.identity,
        payload: validated.proposal.payload,
    };
    application.commit_finalized(&finalized).map_err(BoundaryError::Application)?;
    Ok(finalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct TestApp { reject_validation: bool, reject_commit: bool, commits: usize }
    impl TestApp { fn new() -> Self { Self { reject_validation: false, reject_commit: false, commits: 0 } } }
    impl ApplicationAdapter for TestApp {
        fn validate_proposal(&self, _: ParentBlock, _: &ProposalCandidate) -> Result<(), ApplicationError> {
            if self.reject_validation { Err(ApplicationError::InvalidProposal) } else { Ok(()) }
        }
        fn commit_finalized(&mut self, _: &VerifiedFinalizedBlock) -> Result<(), ApplicationError> {
            if self.reject_commit { return Err(ApplicationError::CommitFailed); }
            self.commits += 1; Ok(())
        }
    }

    struct TestVerifier { reject: bool, calls: Cell<usize> }
    impl FinalityVerifier for TestVerifier {
        fn verify_finality(&self, _: &FinalityClaim) -> Result<(), FinalityVerificationError> {
            self.calls.set(self.calls.get() + 1);
            if self.reject { Err(FinalityVerificationError::InvalidProof) } else { Ok(()) }
        }
    }

    fn fixtures() -> (ChainContext, ParentBlock, ProposalCandidate, BoundaryLimits) {
        let context = ChainContext::new(ChainId256::from_u64(7), 1).unwrap();
        let parent = ParentBlock::new(context, 10, Hash32([1; 32]));
        let id = BlockIdentity::new(context, 11, Hash32([1; 32]), Hash32([2; 32])).unwrap();
        let limits = BoundaryLimits::new(1024, 512).unwrap();
        let proposal = ProposalCandidate::new(id, vec![0xA5, 0x5A], limits).unwrap();
        (context, parent, proposal, limits)
    }

    #[test]
    fn validates_verifies_exact_identity_then_commits() {
        let (ctx, parent, proposal, limits) = fixtures();
        let mut app = TestApp::new();
        let prepared = validate_proposal(ctx, parent, proposal, &app).unwrap();
        let id = prepared.identity();
        let claim = FinalityClaim::new(id, vec![0xCC], limits).unwrap();
        let verifier = TestVerifier { reject: false, calls: Cell::new(0) };
        let result = finalize_proposal(prepared, claim, &verifier, &mut app).unwrap();
        assert_eq!(result.identity(), id);
        assert_eq!(result.payload(), &[0xA5, 0x5A]);
        assert_eq!(app.commits, 1);
        assert_eq!(verifier.calls.get(), 1);
    }

    #[test]
    fn rejects_parent_context_mismatch() {
        let (ctx, parent, proposal, _) = fixtures();
        let wrong = ChainContext::new(ChainId256::from_u64(8), 1).unwrap();
        assert_eq!(validate_proposal(wrong, parent, proposal, &TestApp::new()), Err(BoundaryError::ParentContextMismatch));
        assert_eq!(ctx.chain_id(), parent.context().chain_id());
    }

    #[test]
    fn rejects_wrong_proposal_context() {
        let (ctx, parent, proposal, limits) = fixtures();
        let other = ChainContext::new(ChainId256::from_u64(8), 1).unwrap();
        let id = BlockIdentity::new(other, 11, Hash32([1; 32]), Hash32([3; 32])).unwrap();
        let proposal = ProposalCandidate::new(id, proposal.payload().to_vec(), limits).unwrap();
        assert_eq!(validate_proposal(ctx, parent, proposal, &TestApp::new()), Err(BoundaryError::ProposalContextMismatch));
    }

    #[test]
    fn rejects_wrong_height_and_parent_hash() {
        let (ctx, parent, _, limits) = fixtures();
        let wrong_height = BlockIdentity::new(ctx, 12, Hash32([1; 32]), Hash32([3; 32])).unwrap();
        let p = ProposalCandidate::new(wrong_height, vec![1], limits).unwrap();
        assert_eq!(validate_proposal(ctx, parent, p, &TestApp::new()), Err(BoundaryError::WrongProposalHeight));
        let wrong_parent = BlockIdentity::new(ctx, 11, Hash32([9; 32]), Hash32([3; 32])).unwrap();
        let p = ProposalCandidate::new(wrong_parent, vec![1], limits).unwrap();
        assert_eq!(validate_proposal(ctx, parent, p, &TestApp::new()), Err(BoundaryError::WrongParentHash));
    }

    #[test]
    fn application_validation_failure_stops_proposal() {
        let (ctx, parent, proposal, _) = fixtures();
        let mut app = TestApp::new(); app.reject_validation = true;
        assert_eq!(validate_proposal(ctx, parent, proposal, &app), Err(BoundaryError::Application(ApplicationError::InvalidProposal)));
    }

    #[test]
    fn invalid_finality_never_reaches_commit() {
        let (ctx, parent, proposal, limits) = fixtures();
        let mut app = TestApp::new();
        let prepared = validate_proposal(ctx, parent, proposal, &app).unwrap();
        let claim = FinalityClaim::new(prepared.identity(), vec![1], limits).unwrap();
        let verifier = TestVerifier { reject: true, calls: Cell::new(0) };
        assert_eq!(finalize_proposal(prepared, claim, &verifier, &mut app), Err(BoundaryError::Finality(FinalityVerificationError::InvalidProof)));
        assert_eq!(app.commits, 0);
        assert_eq!(verifier.calls.get(), 1);
    }

    #[test]
    fn mismatched_finality_identity_is_rejected_before_verifier() {
        let (ctx, parent, proposal, limits) = fixtures();
        let mut app = TestApp::new();
        let prepared = validate_proposal(ctx, parent, proposal, &app).unwrap();
        let other = BlockIdentity::new(ctx, 11, Hash32([1; 32]), Hash32([3; 32])).unwrap();
        let claim = FinalityClaim::new(other, vec![1], limits).unwrap();
        let verifier = TestVerifier { reject: false, calls: Cell::new(0) };
        assert_eq!(finalize_proposal(prepared, claim, &verifier, &mut app), Err(BoundaryError::FinalityIdentityMismatch));
        assert_eq!(verifier.calls.get(), 0);
        assert_eq!(app.commits, 0);
    }

    #[test]
    fn bounds_proposal_and_proof_bytes() {
        let (ctx, _, _, _) = fixtures();
        let limits = BoundaryLimits::new(1, 1).unwrap();
        let id = BlockIdentity::new(ctx, 1, Hash32([0; 32]), Hash32([2; 32])).unwrap();
        assert_eq!(ProposalCandidate::new(id, vec![1, 2], limits), Err(BoundaryError::ProposalTooLarge));
        assert_eq!(FinalityClaim::new(id, vec![1, 2], limits), Err(BoundaryError::FinalityProofTooLarge));
        assert_eq!(ProposalCandidate::new(id, vec![], limits), Err(BoundaryError::EmptyProposal));
        assert_eq!(FinalityClaim::new(id, vec![], limits), Err(BoundaryError::EmptyFinalityProof));
    }

    #[test]
    fn rejects_zero_context_and_zero_limits() {
        assert_eq!(ChainContext::new(ChainId256::from_u64(0), 1), Err(BoundaryError::ZeroChainId));
        assert_eq!(ChainContext::new(ChainId256::from_u64(1), 0), Err(BoundaryError::ZeroProtocolVersion));
        assert_eq!(BoundaryLimits::new(0, 1), Err(BoundaryError::ZeroByteLimit));
    }

    #[test]
    fn parent_height_overflow_fails_closed() {
        let (ctx, _, proposal, _) = fixtures();
        let parent = ParentBlock::new(ctx, Height::MAX, Hash32([1; 32]));
        assert_eq!(validate_proposal(ctx, parent, proposal, &TestApp::new()), Err(BoundaryError::ParentHeightOverflow));
    }

    #[test]
    fn commit_failure_is_returned_after_verified_finality() {
        let (ctx, parent, proposal, limits) = fixtures();
        let mut app = TestApp::new();
        let prepared = validate_proposal(ctx, parent, proposal, &app).unwrap();
        let claim = FinalityClaim::new(prepared.identity(), vec![1], limits).unwrap();
        let verifier = TestVerifier { reject: false, calls: Cell::new(0) };
        app.reject_commit = true;
        assert_eq!(finalize_proposal(prepared, claim, &verifier, &mut app), Err(BoundaryError::Application(ApplicationError::CommitFailed)));
        assert_eq!(verifier.calls.get(), 1);
    }
}
