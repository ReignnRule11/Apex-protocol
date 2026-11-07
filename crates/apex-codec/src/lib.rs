//! Canonical codec boundary. No wire codec is selected yet.
use thiserror::Error;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum CodecError {
    #[error("canonical codec is not configured")]
    NotConfigured,
}

/// Fails closed until a versioned canonical codec is selected.
pub fn encode_canonical<T>(_value: &T) -> Result<Vec<u8>, CodecError> {
    Err(CodecError::NotConfigured)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unconfigured_codec_fails_closed() {
        assert_eq!(encode_canonical(&42_u64), Err(CodecError::NotConfigured));
    }
}
