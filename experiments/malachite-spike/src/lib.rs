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
