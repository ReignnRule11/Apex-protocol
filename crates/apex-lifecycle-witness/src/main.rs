use apex_lifecycle_redb::RedbLifecycleCheckpointStore;
use std::{env, error::Error, io, net::SocketAddr};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let database_path = env::var("APEX_WITNESS_DB_PATH")
        .unwrap_or_else(|_| "/var/lib/apex-witness/lifecycle.redb".to_owned());
    let listen: SocketAddr = env::var("APEX_WITNESS_LISTEN")
        .unwrap_or_else(|_| "127.0.0.1:8787".to_owned())
        .parse()?;

    apex_lifecycle_witness::validate_listen_address(listen.ip())
        .map_err(io::Error::other)?;
    let store = RedbLifecycleCheckpointStore::open(database_path)
        .map_err(io::Error::other)?;
    let app = apex_lifecycle_witness::router(store);

    eprintln!("APEX lifecycle witness listening on {listen}");
    let listener = tokio::net::TcpListener::bind(listen).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
