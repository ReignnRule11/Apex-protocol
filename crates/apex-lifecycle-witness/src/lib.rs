//! Independently deployable HTTP service for monotonic lifecycle checkpoints.
//!
//! The service is intended to run on a separate host and behind a mutually
//! authenticated TLS reverse proxy. It binds only to loopback; the proxy is
//! responsible for requiring and validating client certificates. The durable
//! redb file must be on storage and backups administered independently from
//! the validator host.

use apex_consensus::validator_lifecycle_checkpoint::{
    LifecycleCheckpointAnchor, MonotonicLifecycleCheckpointStore,
};
use apex_lifecycle_redb::RedbLifecycleCheckpointStore;
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::{net::IpAddr, sync::Arc};
use subtle::ConstantTimeEq;

#[derive(Clone)]
struct AppState {
    store: Arc<RedbLifecycleCheckpointStore>,
    api_token: Arc<[u8]>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WireAnchor {
    pub sequence: u64,
    pub checkpoint_hash: [u8; 32],
}

impl From<LifecycleCheckpointAnchor> for WireAnchor {
    fn from(value: LifecycleCheckpointAnchor) -> Self {
        Self {
            sequence: value.sequence,
            checkpoint_hash: value.checkpoint_hash,
        }
    }
}

impl From<WireAnchor> for LifecycleCheckpointAnchor {
    fn from(value: WireAnchor) -> Self {
        Self {
            sequence: value.sequence,
            checkpoint_hash: value.checkpoint_hash,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HeadResponse {
    pub head: Option<WireAnchor>,
}

#[derive(Debug, Deserialize)]
pub struct AdvanceRequest {
    pub expected: Option<WireAnchor>,
    pub next: WireAnchor,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AdvanceResponse {
    pub advanced: bool,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: &'static str,
}

/// Build the production witness API around a real durable redb store.
pub fn router(store: RedbLifecycleCheckpointStore, api_token: String) -> Router {
    let state = AppState {
        store: Arc::new(store),
        api_token: Arc::from(api_token.into_bytes()),
    };
    Router::new()
        .route(
            "/v1/networks/{network_id}/head",
            get(read_head).post(advance_head),
        )
        .layer(DefaultBodyLimit::max(4096))
        .with_state(state)
}

async fn read_head(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(network_id): Path<String>,
) -> Result<Json<HeadResponse>, (StatusCode, Json<ErrorResponse>)> {
    authorize(&state, &headers)?;
    let network_id = parse_network_id(&network_id)?;
    let head = state
        .store
        .load(network_id)
        .map_err(|_| unavailable())?
        .map(WireAnchor::from);
    Ok(Json(HeadResponse { head }))
}

async fn advance_head(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(network_id): Path<String>,
    Json(request): Json<AdvanceRequest>,
) -> Result<(StatusCode, Json<AdvanceResponse>), (StatusCode, Json<ErrorResponse>)> {
    authorize(&state, &headers)?;
    let network_id = parse_network_id(&network_id)?;
    let expected = request.expected.map(LifecycleCheckpointAnchor::from);
    let next = LifecycleCheckpointAnchor::from(request.next);

    // Enforce the protocol's monotonic sequence contract at the service edge.
    // The database performs the final atomic compare-and-advance.
    let valid_sequence = match expected {
        None => next.sequence == 1,
        Some(anchor) => anchor.sequence.checked_add(1) == Some(next.sequence),
    };
    if !valid_sequence {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ErrorResponse {
                error: "invalid_sequence",
            }),
        ));
    }

    let advanced = state
        .store
        .compare_and_advance(network_id, expected, next)
        .map_err(|_| unavailable())?;
    let status = if advanced {
        StatusCode::OK
    } else {
        StatusCode::CONFLICT
    };
    Ok((status, Json(AdvanceResponse { advanced })))
}

fn authorize(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let supplied = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::as_bytes);
    let authorized =
        supplied.is_some_and(|value| bool::from(value.ct_eq(state.api_token.as_ref())));
    if authorized && state.api_token.len() >= 32 {
        Ok(())
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "unauthorized",
            }),
        ))
    }
}

fn parse_network_id(value: &str) -> Result<[u8; 32], (StatusCode, Json<ErrorResponse>)> {
    let decoded = hex::decode(value).map_err(|_| invalid_network())?;
    decoded.try_into().map_err(|_| invalid_network())
}

fn invalid_network() -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: "invalid_network_id",
        }),
    )
}

fn unavailable() -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ErrorResponse {
            error: "witness_storage_unavailable",
        }),
    )
}

/// Reject non-loopback listeners. Public exposure must occur only through the
/// separately configured mTLS reverse proxy described in the deployment guide.
pub fn validate_listen_address(address: IpAddr) -> Result<(), &'static str> {
    if address.is_loopback() {
        Ok(())
    } else {
        Err("witness service must bind to loopback behind an mTLS reverse proxy")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tempfile::tempdir;
    use tower::ServiceExt;

    #[tokio::test]
    async fn real_redb_service_reads_and_atomically_advances_heads() {
        let directory = tempdir().unwrap();
        let store =
            RedbLifecycleCheckpointStore::open(directory.path().join("witness.redb")).unwrap();
        let app = router(store, "test-only-token-with-at-least-32-bytes".to_owned());
        let network = [0x2a; 32];
        let path = format!("/v1/networks/{}/head", hex::encode(network));

        let unauthorized = app
            .clone()
            .oneshot(Request::builder().uri(&path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

        let missing = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&path)
                    .header(
                        "authorization",
                        "Bearer test-only-token-with-at-least-32-bytes",
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(missing.status(), StatusCode::OK);
        let body = axum::body::to_bytes(missing.into_body(), 4096)
            .await
            .unwrap();
        assert_eq!(&body[..], br#"{"head":null}"#);

        let first = WireAnchor {
            sequence: 1,
            checkpoint_hash: [0x11; 32],
        };
        let request = serde_json::json!({
            "expected": null,
            "next": first
        });
        let advanced = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&path)
                    .header("content-type", "application/json")
                    .header(
                        "authorization",
                        "Bearer test-only-token-with-at-least-32-bytes",
                    )
                    .body(Body::from(request.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(advanced.status(), StatusCode::OK);

        let stale = serde_json::json!({
            "expected": null,
            "next": {
                "sequence": 1,
                "checkpoint_hash": vec![0x22; 32]
            }
        });
        let conflict = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&path)
                    .header("content-type", "application/json")
                    .header(
                        "authorization",
                        "Bearer test-only-token-with-at-least-32-bytes",
                    )
                    .body(Body::from(stale.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(conflict.status(), StatusCode::CONFLICT);

        let current = app
            .oneshot(
                Request::builder()
                    .uri(&path)
                    .header(
                        "authorization",
                        "Bearer test-only-token-with-at-least-32-bytes",
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(current.into_body(), 4096)
            .await
            .unwrap();
        let response: HeadResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(response.head, Some(first));
    }

    #[test]
    fn refuses_public_listener_addresses() {
        assert!(validate_listen_address("127.0.0.1".parse().unwrap()).is_ok());
        assert!(validate_listen_address("::1".parse().unwrap()).is_ok());
        assert!(validate_listen_address("0.0.0.0".parse().unwrap()).is_err());
    }
}
