//! mTLS HTTP client for the independently operated APEX lifecycle witness.
//!
//! The constructor requires HTTPS, a client certificate/private key identity,
//! and a dedicated CA certificate used to authenticate the witness endpoint.
//! It implements both the lifecycle checkpoint store and startup witness
//! interfaces so the same remote head can be advanced and checked.

use apex_consensus::validator_lifecycle_checkpoint::{
    CheckpointStoreError, LifecycleCheckpointAnchor, MonotonicLifecycleCheckpointStore,
};
use apex_lifecycle_redb::{ExternalLifecycleRollbackWitness, RollbackWitnessError};
use reqwest::{
    blocking::Client,
    header::{AUTHORIZATION, HeaderValue},
    Certificate, Identity, StatusCode, Url,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WitnessClientConfigError {
    #[error("witness endpoint URL is invalid")]
    InvalidUrl,
    #[error("witness endpoint must use HTTPS and must not contain credentials or a fragment")]
    InsecureEndpoint,
    #[error("client certificate identity is invalid")]
    InvalidClientIdentity,
    #[error("witness CA certificate is invalid")]
    InvalidWitnessCa,
    #[error("witness API token must contain at least 32 bytes and valid header characters")]
    InvalidApiToken,
    #[error("HTTP client could not be configured")]
    HttpClient,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
struct WireAnchor {
    sequence: u64,
    checkpoint_hash: [u8; 32],
}

impl From<LifecycleCheckpointAnchor> for WireAnchor {
    fn from(value: LifecycleCheckpointAnchor) -> Self {
        Self { sequence: value.sequence, checkpoint_hash: value.checkpoint_hash }
    }
}

impl From<WireAnchor> for LifecycleCheckpointAnchor {
    fn from(value: WireAnchor) -> Self {
        Self { sequence: value.sequence, checkpoint_hash: value.checkpoint_hash }
    }
}

#[derive(Debug, Deserialize)]
struct HeadResponse {
    head: Option<WireAnchor>,
}

#[derive(Debug, Serialize)]
struct AdvanceRequest {
    expected: Option<WireAnchor>,
    next: WireAnchor,
}

#[derive(Debug, Deserialize)]
struct AdvanceResponse {
    advanced: bool,
}

pub struct HttpLifecycleWitnessClient {
    base_url: Url,
    client: Client,
}

impl HttpLifecycleWitnessClient {
    /// Configure a strict HTTPS client with mutual TLS credentials.
    ///
    /// The identity PEM must contain the client certificate chain and private
    /// key. The CA PEM must validate the witness server certificate. No
    /// invalid-certificate or plaintext fallback is available.
    pub fn from_pem(
        base_url: &str,
        client_identity_pem: &[u8],
        witness_ca_pem: &[u8],
        api_token: &str,
    ) -> Result<Self, WitnessClientConfigError> {
        let base_url = Url::parse(base_url).map_err(|_| WitnessClientConfigError::InvalidUrl)?;
        if base_url.scheme() != "https"
            || !base_url.username().is_empty()
            || base_url.password().is_some()
            || base_url.fragment().is_some()
        {
            return Err(WitnessClientConfigError::InsecureEndpoint);
        }

        if api_token.len() < 32 {
            return Err(WitnessClientConfigError::InvalidApiToken);
        }
        let mut authorization = HeaderValue::from_str(&format!("Bearer {api_token}"))
            .map_err(|_| WitnessClientConfigError::InvalidApiToken)?;
        authorization.set_sensitive(true);

        let identity = Identity::from_pem(client_identity_pem)
            .map_err(|_| WitnessClientConfigError::InvalidClientIdentity)?;
        let witness_ca = Certificate::from_pem(witness_ca_pem)
            .map_err(|_| WitnessClientConfigError::InvalidWitnessCa)?;
        let client = Client::builder()
            .https_only(true)
            .identity(identity)
            .add_root_certificate(witness_ca)
            .default_headers(std::iter::once((AUTHORIZATION, authorization)).collect())
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|_| WitnessClientConfigError::HttpClient)?;

        Ok(Self { base_url, client })
    }

    fn endpoint(&self, network_id: [u8; 32]) -> String {
        format!("{}/v1/networks/{}/head",
            self.base_url.as_str().trim_end_matches('/'), hex::encode(network_id))
    }

    fn read_head(
        &self,
        network_id: [u8; 32],
    ) -> Result<Option<LifecycleCheckpointAnchor>, CheckpointStoreError> {
        let response = self.client.get(self.endpoint(network_id)).send()
            .map_err(|_| CheckpointStoreError::Unavailable)?;
        if !response.status().is_success() {
            return Err(CheckpointStoreError::Unavailable);
        }
        let body: HeadResponse = response.json()
            .map_err(|_| CheckpointStoreError::Unavailable)?;
        Ok(body.head.map(LifecycleCheckpointAnchor::from))
    }
}

impl MonotonicLifecycleCheckpointStore for HttpLifecycleWitnessClient {
    fn load(
        &self,
        network_id: [u8; 32],
    ) -> Result<Option<LifecycleCheckpointAnchor>, CheckpointStoreError> {
        self.read_head(network_id)
    }

    fn compare_and_advance(
        &self,
        network_id: [u8; 32],
        expected: Option<LifecycleCheckpointAnchor>,
        next: LifecycleCheckpointAnchor,
    ) -> Result<bool, CheckpointStoreError> {
        let request = AdvanceRequest {
            expected: expected.map(WireAnchor::from),
            next: WireAnchor::from(next),
        };
        let response = self.client.post(self.endpoint(network_id)).json(&request).send()
            .map_err(|_| CheckpointStoreError::Unavailable)?;

        if response.status() == StatusCode::CONFLICT {
            return Ok(false);
        }
        if response.status() == StatusCode::UNPROCESSABLE_ENTITY {
            return Err(CheckpointStoreError::InvalidAdvance);
        }
        if !response.status().is_success() {
            return Err(CheckpointStoreError::Unavailable);
        }
        let body: AdvanceResponse = response.json()
            .map_err(|_| CheckpointStoreError::Unavailable)?;
        Ok(body.advanced)
    }
}

impl ExternalLifecycleRollbackWitness for HttpLifecycleWitnessClient {
    fn trusted_head(
        &self,
        network_id: [u8; 32],
    ) -> Result<Option<LifecycleCheckpointAnchor>, RollbackWitnessError> {
        self.read_head(network_id).map_err(|_| RollbackWitnessError::Unavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_plaintext_and_credential_bearing_endpoints_before_tls_setup() {
        for endpoint in [
            "http://witness.example.net",
            "https://user:secret@witness.example.net",
            "https://witness.example.net/#fragment",
        ] {
            assert!(matches!(
                HttpLifecycleWitnessClient::from_pem(endpoint, b"", b"", "test-token-with-at-least-32-bytes"),
                Err(WitnessClientConfigError::InsecureEndpoint)
            ));
        }
    }

    #[test]
    fn refuses_invalid_endpoint_syntax() {
        assert!(matches!(
            HttpLifecycleWitnessClient::from_pem("not a URL", b"", b"", "test-token-with-at-least-32-bytes"),
            Err(WitnessClientConfigError::InvalidUrl)
        ));
    }
}
