//! Consensus primitives and engine-neutral protocol boundary checks.
//!
//! This crate does not implement BFT consensus, authenticate validator votes,
//! verify commit certificates, execute transactions, or establish finality.
//! Engine adapters must treat all engine-provided data as untrusted until the
//! selected engine's verification path has completed.

pub mod application_boundary;
pub mod certificate;
pub mod consensus_message;
pub mod validator_lifecycle;
pub mod validator_set;

use apex_types::{ChainId, Hash32, Height};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
