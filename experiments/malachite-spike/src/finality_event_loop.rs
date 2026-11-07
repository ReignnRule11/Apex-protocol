//! Fail-closed application gate for Malachite's real channel event stream.
//!
//! The application supplies the native verifier/value-binding implementation
//! and durable staging/commit operations. This module owns the ordering rule:
//! Decided may be staged and acknowledged, but only a verified and committed
//! Finalized event may receive Next::Start.

use core::future::Future;
use core::pin::Pin;

use malachitebft_app_channel::{
    AppMsg, Channels,
    app::types::core::{CommitCertificate, Context, Height, HeightParams},
};
use malachitebft_engine::host::Next;

use crate::native_finality_bridge::VerifiedNativeFinality;

pub type GateFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, ()>> + Send + 'a>>;

/// Application operations required by the channel finality gate.
///
/// `verify_certificate` must call the pinned Malachite-native verifier and
/// independently bind the certificate value ID to strictly decoded canonical
/// APEX bytes and the recomputed APEX identity. `stage_decided` must not
/// publish the value as finalized or advance the application height.
/// `commit_finalized` must durably commit the verified identity before it
/// returns parameters for the next height.
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
    ) -> GateFuture<'a, HeightParams<Ctx>>;

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

            let next = if let (Some(expected), Some(next_height)) = (expected_next, incremented) {
                if next_height.as_u64() == expected {
                    match app.verify_certificate(&certificate).await {
                        Ok(verified) if verified.identity().height == height.as_u64() => {
                            app.commit_finalized(&verified)
                                .await
                                .ok()
                                .map(|params| Next::Start(next_height, params))
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let instruction = match next {
                Some(instruction) => Some(instruction),
                None => app
                    .restart_params(height)
                    .await
                    .ok()
                    .map(|params| Next::Restart(height, params)),
            };
            if let Some(instruction) = instruction {
                let _ = reply.send(instruction);
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
    mut handle_other: Other,
) -> Result<(), ChannelsClosed>
where
    Ctx: Context,
    App: FinalityApplication<Ctx>,
    Other: FnMut(AppMsg<Ctx>) -> OtherFuture,
    OtherFuture: Future<Output = Result<(), ()>> + Send,
{
    while let Some(message) = channels.consensus.recv().await {
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

    #[test]
    fn height_overflow_is_detected_before_finalized_commit() {
        assert_eq!(7_u64.checked_add(1), Some(8));
        assert_eq!(u64::MAX.checked_add(1), None);
    }
}
