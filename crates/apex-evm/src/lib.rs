//! APEX-owned EVM integration boundary.
//!
//! The crate contains a full-width immutable state model, revm Database and
//! state-diff adapters, plus an explicit, bounded revm environment mapping.
//! These are integration prototypes, not a complete transaction pipeline.

pub mod execution_context;
pub mod revm_environment;
pub mod state_diff;
