//! APEX node lifecycle safety preflight.
//!
//! This executable validates protocol identity and candidate validator
//! configuration, then verifies that every configured network has an
//! initialized local lifecycle checkpoint matching an independently operated
//! HTTPS/mTLS witness. It deliberately does not start consensus or networking.

mod genesis_commitment;
mod protocol_config;

use apex_lifecycle_redb::{RedbLifecycleCheckpointStore, verify_lifecycle_startup};
use apex_lifecycle_witness_client::HttpLifecycleWitnessClient;
use genesis_commitment::GenesisProtocolConfig;
use protocol_config::NodeProtocolConfig;
use std::{env, fs, process::ExitCode};

fn required_env(name: &str) -> Result<String, String> {
    env::var(name).map_err(|_| format!("required environment variable {name} is missing"))
}

fn required_file(name: &str) -> Result<Vec<u8>, String> {
    let path = required_env(name)?;
    fs::read(&path).map_err(|_| format!("could not read required file from {name}"))
}

/// Parse comma-separated, 32-byte network IDs encoded as exactly 64 hex chars.
fn parse_network_ids(value: &str) -> Result<Vec<[u8; 32]>, String> {
    if value.is_empty() {
        return Err("APEX_LIFECYCLE_NETWORK_IDS must not be empty".to_owned());
    }

    value
        .split(',')
        .map(|entry| {
            if entry.len() != 64 {
                return Err(
                    "each lifecycle network ID must contain exactly 64 hex characters".to_owned(),
                );
            }
            let decoded = hex::decode(entry)
                .map_err(|_| "lifecycle network ID contains invalid hexadecimal".to_owned())?;
            decoded
                .try_into()
                .map_err(|_| "lifecycle network ID must decode to exactly 32 bytes".to_owned())
        })
        .collect()
}

fn run_preflight() -> Result<(), String> {
    let protocol = NodeProtocolConfig::from_env()
        .map_err(|error| format!("protocol configuration rejected: {error}"))?;
    let network_ids = parse_network_ids(&required_env("APEX_LIFECYCLE_NETWORK_IDS")?)?;

    let database_path = required_env("APEX_LIFECYCLE_DB_PATH")?;
    let mut local = RedbLifecycleCheckpointStore::open(&database_path)
        .map_err(|error| format!("local lifecycle store could not be opened: {error}"))?;
    local
        .check_integrity()
        .map_err(|error| format!("local lifecycle store integrity check failed: {error}"))?;

    let witness_url = required_env("APEX_WITNESS_URL")?;
    let client_identity = required_file("APEX_WITNESS_CLIENT_IDENTITY_PEM")?;
    let witness_ca = required_file("APEX_WITNESS_CA_PEM")?;
    let api_token = required_env("APEX_WITNESS_API_TOKEN")?;

    let witness = HttpLifecycleWitnessClient::from_pem(
        &witness_url,
        &client_identity,
        &witness_ca,
        &api_token,
    )
    .map_err(|error| format!("independent witness client configuration failed: {error}"))?;

    verify_lifecycle_startup(&local, &witness, &network_ids)
        .map_err(|error| format!("lifecycle startup verification rejected: {error}"))?;

    println!(
        "APEX protocol configuration validated: chain ID {}, protocol version {}, {} candidate validator(s), total voting power {}.",
        hex::encode(protocol.chain_id.to_be_bytes()),
        protocol.protocol_version,
        protocol.validators.validators().len(),
        protocol.validators.total_voting_power()
    );
    let genesis = GenesisProtocolConfig {
        chain_id: protocol.chain_id,
        protocol_version: protocol.protocol_version,
        validators: protocol.validators.clone(),
    };
    let commitment = genesis
        .commitment()
        .map_err(|error| format!("genesis protocol commitment rejected: {error}"))?;

    println!(
        "Canonical genesis protocol commitment (SHA-256): {}",
        hex::encode(commitment.digest)
    );
    println!(
        "APEX lifecycle preflight passed for {} configured network(s).",
        network_ids.len()
    );
    println!(
        "No networking, consensus, signing, or transaction services were started; this binary remains a safety preflight only."
    );
    Ok(())
}

fn main() -> ExitCode {
    match run_preflight() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("APEX startup preflight FAILED: {error}");
            eprintln!("Failing closed: no node services were started.");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_one_and_multiple_network_ids() {
        let ids = parse_network_ids(&format!("{},{}", "01".repeat(32), "ab".repeat(32))).unwrap();
        assert_eq!(ids, vec![[1; 32], [0xab; 32]]);
    }

    #[test]
    fn rejects_empty_network_configuration() {
        assert!(parse_network_ids("").is_err());
    }

    #[test]
    fn rejects_wrong_length_network_ids() {
        assert!(parse_network_ids(&"01".repeat(31)).is_err());
    }

    #[test]
    fn rejects_non_hexadecimal_network_ids() {
        assert!(parse_network_ids(&"zz".repeat(32)).is_err());
    }

    #[test]
    fn rejects_empty_entries_in_network_list() {
        assert!(parse_network_ids(&format!("{},", "01".repeat(32))).is_err());
    }
}
