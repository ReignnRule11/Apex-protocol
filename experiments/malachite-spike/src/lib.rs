//! Isolated, non-production compile probe for Malachite v0.8.2.
//!
//! This crate intentionally lives in its own Cargo workspace. It must not be
//! added to the APEX production workspace until dependency, signer, recovery,
//! and application-boundary reviews have passed.

pub use malachitebft_app_channel::{
    ConsensusContext, EngineBuilder, NetworkContext, RequestContext, SyncContext, WalContext,
};

/// Exact upstream release used by this isolated compatibility probe.
pub const MALACHITE_RELEASE: &str = "0.8.2";

/// Immutable upstream commit corresponding to the signed v0.8.2 tag.
pub const MALACHITE_REVISION: &str = "a453b32155f4ba9f959724279b52f601bae92e09";

/// Build a real Malachite v0.8.2 engine using the upstream channel-based
/// application API. This is an isolated integration adapter: callers still
/// own configuration, key custody, network identity, codecs, application
/// event handling, and the APEX-specific safety checks around finalization.
pub async fn build_engine<Ctx, Config, Wal, Net, Sync>(
    ctx: Ctx,
    config: Config,
    wal: WalContext<Wal>,
    network: NetworkContext<Net>,
    consensus: ConsensusContext<Ctx>,
    sync: SyncContext<Sync>,
    request: RequestContext,
) -> eyre::Result<(
    malachitebft_app_channel::Channels<Ctx>,
    malachitebft_app_channel::EngineHandle,
)>
where
    Ctx: malachitebft_app_channel::app::types::core::Context,
    Config: malachitebft_app_channel::app::config::NodeConfig,
    Wal: malachitebft_app_channel::app::types::codec::WalCodec<Ctx>,
    Net: malachitebft_app_channel::app::types::codec::ConsensusCodec<Ctx>
        + malachitebft_app_channel::app::types::codec::SyncCodec<Ctx>,
    Sync: malachitebft_app_channel::app::types::codec::SyncCodec<Ctx>,
{
    use malachitebft_app_channel::EngineBuilder;

    EngineBuilder::new(ctx, config)
        .with_default_wal(wal)
        .with_default_network(network)
        .with_default_consensus(consensus)
        .with_default_sync(sync)
        .with_default_request(request)
        .build()
        .await
}

pub mod finality_event_loop;
pub mod finality_journal;
pub mod native_finality_bridge;
pub mod redb_recovery_inspection;
pub mod reserved_signer;
pub mod wal_preflight;
pub mod wal_snapshot;
