//! Fail-closed application gate for Malachite's real channel event stream.
//!
//! The application supplies the native verifier/value-binding implementation
//! and durable staging/commit operations. This module owns the ordering rule:
//! Decided may be staged and acknowledged, but only a verified and committed
//! Finalized event may receive Next::Start.

use core::future::Future;
use core::pin::Pin;

use tokio::sync::mpsc;

use malachitebft_app_channel::{
    AppMsg, Channels,
    app::types::core::{CommitCertificate, Context, Height, HeightParams},
};
use malachitebft_engine::host::Next;

use crate::native_finality_bridge::VerifiedNativeFinality;

pub type GateFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, ()>> + Send + 'a>>;

/// A commit error that distinguishes a proven abort from an uncertain outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FinalityCommitError {
    /// Storage proves that no application state was committed.
    DefinitelyNotCommitted,
    /// The commit may have succeeded; recovery must inspect its durable key.
    Ambiguous,
    /// A different finalized identity already occupies this height.
    ConflictingFinality,
}

pub type CommitFuture<'a, Ctx> = Pin<Box<
    dyn Future<Output = Result<HeightParams<Ctx>, FinalityCommitError>> + Send + 'a,
>>;

/// Application operations required by the channel finality gate.
///
/// `verify_certificate` must call the pinned Malachite-native verifier and
/// independently bind the certificate value ID to strictly decoded canonical
/// APEX bytes and the recomputed APEX identity. `stage_decided` must not
/// publish the value as finalized or advance the application height.
/// `commit_finalized` must atomically persist application state and the
/// verified token's stable `commit_key()` before returning next-height params.
/// Same-key retries must be idempotent. Only a proven non-commit may be
/// reported as `DefinitelyNotCommitted`; uncertain/conflicting outcomes stall.
pub trait FinalityApplication<Ctx: Context>: Send {
    fn verify_certificate<'a>(
        &'a mut self,
        certificate: &'a CommitCertificate<Ctx>,
    ) -> GateFuture<'a, VerifiedNativeFinality>;

    fn stage_decided<'a>(
        &'a mut self,
        verified: &'a VerifiedNativeFinality,
    ) -> GateFuture<'a, ()>;

    fn commit_finalized<'a>(
        &'a mut self,
        verified: &'a VerifiedNativeFinality,
    ) -> CommitFuture<'a, Ctx>;

    fn restart_params<'a>(
        &'a mut self,
        height: Ctx::Height,
    ) -> GateFuture<'a, HeightParams<Ctx>>;
}

/// Result of offering one real Malachite application-channel message to the gate.
pub enum GateDispatch<Ctx: Context> {
    /// The message was consumed by the finality gate.
    Handled,
    /// The message is not a finality event and must be processed by the caller's
    /// ordinary application event handler.
    Unhandled(AppMsg<Ctx>),
}

/// Process one message from `Channels::consensus`.
///
/// Decided is acknowledged only after native certificate verification and
/// durable staging. It never returns a next-height instruction. Finalized is
/// acknowledged with Next::Start only after native verification, identity
/// binding and durable application commit. Every verification/staging/commit
/// failure takes the fail-closed restart path; if restart parameters cannot be
/// obtained, the reply is dropped and consensus remains stalled.
pub async fn handle_finality_message<Ctx, App>(
    message: AppMsg<Ctx>,
    app: &mut App,
) -> GateDispatch<Ctx>
where
    Ctx: Context,
    App: FinalityApplication<Ctx>,
{
    match message {
        AppMsg::Decided {
            certificate,
            reply,
            ..
        } => {
            if let Ok(verified) = app.verify_certificate(&certificate).await {
                if verified.identity().height == certificate.height.as_u64()
                    && app.stage_decided(&verified).await.is_ok()
                {
                    let _ = reply.send(());
                }
            }
            GateDispatch::Handled
        }
        AppMsg::Finalized {
            certificate,
            reply,
            ..
        } => {
            let height = certificate.height;
            let expected_next = height.as_u64().checked_add(1);
            let incremented = if expected_next.is_some() {
                Some(height.increment())
            } else {
                None
            };

            let restart_is_safe = if let (Some(expected), Some(next_height)) = (expected_next, incremented) {
                if next_height.as_u64() != expected {
                    true
                } else {
                    match app.verify_certificate(&certificate).await {
                        Ok(verified) if verified.identity().height == height.as_u64() => {
                            match app.commit_finalized(&verified).await {
                                Ok(params) => {
                                    let _ = reply.send(Next::Start(next_height, params));
                                    return GateDispatch::Handled;
                                }
                                Err(FinalityCommitError::DefinitelyNotCommitted) => true,
                                Err(FinalityCommitError::Ambiguous | FinalityCommitError::ConflictingFinality) => {
                                    // Never blindly replay an uncertain or conflicting commit.
                                    return GateDispatch::Handled;
                                }
                            }
                        }
                        _ => true,
                    }
                }
            } else {
                true
            };

            if restart_is_safe {
                if let Ok(params) = app.restart_params(height).await {
                    let _ = reply.send(Next::Restart(height, params));
                }
            }
            GateDispatch::Handled
        }
        other => GateDispatch::Unhandled(other),
    }
}

/// Run the finality gate on the actual receiver supplied by
/// `EngineBuilder::build`. Non-finality messages are delegated unchanged to
/// the caller's normal application handler. The caller should clone any
/// network/request senders it needs before entering this loop.
pub async fn run_channels_finality_gate<Ctx, App, Other, OtherFuture>(
    channels: &mut Channels<Ctx>,
    app: &mut App,
    handle_other: Other,
) -> Result<(), ChannelsClosed>
where
    Ctx: Context,
    App: FinalityApplication<Ctx>,
    Other: FnMut(AppMsg<Ctx>) -> OtherFuture,
    OtherFuture: Future<Output = Result<(), ()>> + Send,
{
    run_finality_receiver(&mut channels.consensus, app, handle_other).await
}

/// Drive the same receiver type exposed by Malachite's Channels. Extracted so
/// deterministic tests can exercise queued messages and closure without
/// constructing unrelated network, event and request actors.
async fn run_finality_receiver<Ctx, App, Other, OtherFuture>(
    receiver: &mut mpsc::Receiver<AppMsg<Ctx>>,
    app: &mut App,
    mut handle_other: Other,
) -> Result<(), ChannelsClosed>
where
    Ctx: Context,
    App: FinalityApplication<Ctx>,
    Other: FnMut(AppMsg<Ctx>) -> OtherFuture,
    OtherFuture: Future<Output = Result<(), ()>> + Send,
{
    while let Some(message) = receiver.recv().await {
        if let GateDispatch::Unhandled(other) =
            handle_finality_message(message, app).await
        {
            handle_other(other).await.map_err(|_| ChannelsClosed)?;
        }
    }
    Err(ChannelsClosed)
}

/// The consensus application channel closed unexpectedly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChannelsClosed;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_finality_bridge::ApexBlockIdentity;
    use malachitebft_app_channel::app::types::{
        MisbehaviorEvidence,
        core::{CommitCertificate, HeightParams, LinearTimeouts, Round, VoteExtensions},
    };
    use malachitebft_engine::host::Next;
    use malachitebft_test::{Height as TestHeight, PrivateKey, TestContext, Validator, ValidatorSet, ValueId};
    use rand::{SeedableRng, rngs::StdRng};
    use tokio::sync::oneshot;

    fn identity(height: u64) -> ApexBlockIdentity {
        ApexBlockIdentity { chain_id: [1; 32], protocol_version: 1, height, parent_hash: [2; 32], block_hash: [3; 32] }
    }
    fn certificate(height: u64) -> CommitCertificate<TestContext> {
        CommitCertificate::new(TestHeight::new(height), Round::new(0), ValueId::new(9), Vec::new())
    }
    fn extensions() -> VoteExtensions<TestContext> { VoteExtensions::new(Default::default()) }
    fn evidence() -> MisbehaviorEvidence<TestContext> {
        MisbehaviorEvidence { proposals: Default::default(), votes: Default::default() }
    }
    fn params() -> HeightParams<TestContext> {
        let mut rng = StdRng::seed_from_u64(0xA0E_204);
        let key = PrivateKey::generate(&mut rng);
        let validator_set = ValidatorSet::new([Validator::new(key.public_key(), 1)]);
        HeightParams::new(validator_set, LinearTimeouts::default(), None)
    }

    #[derive(Debug)]
    struct TestApplication {
        events: Vec<&'static str>,
        verify_ok: bool,
        stage_ok: bool,
        commit_ok: bool,
        commit_failure: Option<FinalityCommitError>,
        restart_ok: bool,
        verified_height: u64,
    }
    impl Default for TestApplication {
        fn default() -> Self {
            Self { events: Vec::new(), verify_ok: true, stage_ok: true, commit_ok: true, commit_failure: None, restart_ok: true, verified_height: 7 }
        }
    }
    impl FinalityApplication<TestContext> for TestApplication {
        fn verify_certificate<'a>(&'a mut self, _certificate: &'a CommitCertificate<TestContext>) -> GateFuture<'a, VerifiedNativeFinality> {
            self.events.push("verify");
            let ok = self.verify_ok;
            let token = VerifiedNativeFinality::test_only(identity(self.verified_height));
            Box::pin(async move { if ok { Ok(token) } else { Err(()) } })
        }
        fn stage_decided<'a>(&'a mut self, _verified: &'a VerifiedNativeFinality) -> GateFuture<'a, ()> {
            self.events.push("stage");
            let ok = self.stage_ok;
            Box::pin(async move { if ok { Ok(()) } else { Err(()) } })
        }
        fn commit_finalized<'a>(&'a mut self, _verified: &'a VerifiedNativeFinality) -> CommitFuture<'a, TestContext> {
            self.events.push("commit");
            let ok = self.commit_ok;
            let failure = self.commit_failure;
            Box::pin(async move {
                if let Some(error) = failure {
                    Err(error)
                } else if ok {
                    Ok(params())
                } else {
                    Err(FinalityCommitError::DefinitelyNotCommitted)
                }
            })
        }
        fn restart_params<'a>(&'a mut self, _height: TestHeight) -> GateFuture<'a, HeightParams<TestContext>> {
            self.events.push("restart");
            let ok = self.restart_ok;
            Box::pin(async move { if ok { Ok(params()) } else { Err(()) } })
        }
    }

    #[tokio::test]
    async fn decided_is_staged_before_ack_and_never_commits_or_advances() {
        let mut app = TestApplication::default();
        let (reply, acknowledged) = oneshot::channel();
        let msg = AppMsg::Decided { certificate: certificate(7), extensions: extensions(), reply };
        assert!(matches!(handle_finality_message(msg, &mut app).await, GateDispatch::Handled));
        acknowledged.await.expect("Decided acknowledgement");
        assert_eq!(app.events, ["verify", "stage"]);
    }

    #[tokio::test]
    async fn decided_verification_and_staging_failures_drop_acknowledgement() {
        for mut app in [
            TestApplication { verify_ok: false, ..Default::default() },
            TestApplication { stage_ok: false, ..Default::default() },
        ] {
            let (reply, acknowledged) = oneshot::channel();
            let msg = AppMsg::Decided { certificate: certificate(7), extensions: extensions(), reply };
            handle_finality_message(msg, &mut app).await;
            assert!(acknowledged.await.is_err());
        }
    }

    #[tokio::test]
    async fn finalized_commits_before_next_height_start() {
        let mut app = TestApplication::default();
        let (reply, response) = oneshot::channel();
        let msg = AppMsg::Finalized { certificate: certificate(7), extensions: extensions(), evidence: evidence(), reply };
        handle_finality_message(msg, &mut app).await;
        assert!(matches!(response.await.expect("Finalized response"), Next::Start(h, _) if h.as_u64() == 8));
        assert_eq!(app.events, ["verify", "commit"]);
    }

    #[tokio::test]
    async fn definitely_uncommitted_finalized_attempt_restarts_current_height() {
        let mut app = TestApplication { commit_ok: false, ..Default::default() };
        let (reply, response) = oneshot::channel();
        let msg = AppMsg::Finalized { certificate: certificate(7), extensions: extensions(), evidence: evidence(), reply };
        handle_finality_message(msg, &mut app).await;
        assert!(matches!(response.await.expect("restart response"), Next::Restart(h, _) if h.as_u64() == 7));
        assert_eq!(app.events, ["verify", "commit", "restart"]);
    }

    #[tokio::test]
    async fn ambiguous_or_conflicting_commit_outcomes_drop_reply_without_restart() {
        for failure in [
            FinalityCommitError::Ambiguous,
            FinalityCommitError::ConflictingFinality,
        ] {
            let mut app = TestApplication {
                commit_failure: Some(failure),
                ..Default::default()
            };
            let (reply, response) = oneshot::channel();
            let msg = AppMsg::Finalized {
                certificate: certificate(7),
                extensions: extensions(),
                evidence: evidence(),
                reply,
            };
            handle_finality_message(msg, &mut app).await;
            assert!(response.await.is_err());
            assert_eq!(app.events, ["verify", "commit"]);
        }
    }

    #[tokio::test]
    async fn finalized_verification_and_identity_height_failures_never_commit() {
        for mut app in [
            TestApplication { verify_ok: false, ..Default::default() },
            TestApplication { verified_height: 8, ..Default::default() },
        ] {
            let (reply, response) = oneshot::channel();
            let msg = AppMsg::Finalized { certificate: certificate(7), extensions: extensions(), evidence: evidence(), reply };
            handle_finality_message(msg, &mut app).await;
            assert!(matches!(response.await.expect("restart response"), Next::Restart(h, _) if h.as_u64() == 7));
            assert!(!app.events.contains(&"commit"));
        }
    }

    #[tokio::test]
    async fn restart_parameter_failure_drops_finalized_reply() {
        let mut app = TestApplication { commit_ok: false, restart_ok: false, ..Default::default() };
        let (reply, response) = oneshot::channel();
        let msg = AppMsg::Finalized { certificate: certificate(7), extensions: extensions(), evidence: evidence(), reply };
        handle_finality_message(msg, &mut app).await;
        assert!(response.await.is_err());
    }

    #[tokio::test]
    async fn height_overflow_cannot_start_next_height() {
        let mut app = TestApplication::default();
        let (reply, response) = oneshot::channel();
        let msg = AppMsg::Finalized { certificate: certificate(u64::MAX), extensions: extensions(), evidence: evidence(), reply };
        handle_finality_message(msg, &mut app).await;
        assert!(matches!(response.await.expect("restart response"), Next::Restart(h, _) if h.as_u64() == u64::MAX));
        assert_eq!(app.events, ["restart"]);
    }

    #[tokio::test]
    async fn actual_consensus_receiver_processes_decided_before_closure() {
        let mut app = TestApplication::default();
        let (sender, mut receiver) = mpsc::channel(1);
        let (reply, acknowledged) = oneshot::channel();
        sender.send(AppMsg::Decided { certificate: certificate(7), extensions: extensions(), reply }).await.expect("enqueue");
        drop(sender);
        let result = run_finality_receiver(&mut receiver, &mut app, |_other| async { Ok(()) }).await;
        assert_eq!(result, Err(ChannelsClosed));
        acknowledged.await.expect("channel acknowledgement");
        assert_eq!(app.events, ["verify", "stage"]);
    }
}
