//! APEX-owned EVM integration boundary.
//!
//! Phase 19 currently links the exact revm candidate to establish whether its
//! dependency graph builds on the pinned APEX toolchain. No EVM execution,
//! transaction authentication, fork selection, state adapter, fee settlement,
//! receipt encoding, or production behavior is implemented here.

use revm as _;

/// Exact upstream crate version under the Phase 19 compatibility spike.
///
/// This constant records the candidate under test; it is not a protocol
/// version and must not be used for consensus or wire-format decisions.
pub const REVM_CANDIDATE_VERSION: &str = "43.0.3";

#[cfg(test)]
mod tests {
    use super::REVM_CANDIDATE_VERSION;

    #[test]
    fn candidate_version_is_exactly_pinned_for_the_spike() {
        assert_eq!(REVM_CANDIDATE_VERSION, "43.0.3");
    }
}
