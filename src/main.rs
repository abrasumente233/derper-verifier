use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use std::str::FromStr;
use tokio::{fs::File, io::AsyncReadExt, net::TcpListener};
use tracing::{info, Level};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // init tracing
    let filter = Targets::from_str(std::env::var("RUST_LOG").as_deref().unwrap_or("info"))
        .expect("Invalid RUST_LOG value");
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish()
        .with(filter)
        .init();

    // get the listen address and port from env
    let addr = std::env::var("DERPER_VERIFIER_ADDR").unwrap_or("127.0.0.1".to_string());
    let port: u16 = std::env::var("DERPER_VERIFIER_PORT")
        .unwrap_or("3000".to_string())
        .parse()
        .unwrap();

    // read trusted clients from file
    let mut buf = String::new();
    let config_file =
        std::env::var("DERPER_VERIFIER_CONFIG").unwrap_or("trusted_clients.txt".to_string());
    File::open(config_file)
        .await
        .unwrap()
        .read_to_string(&mut buf)
        .await
        .unwrap();

    let trusted_clients: Vec<String> = buf
        .lines()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_owned())
        .collect();

    let app = Router::new()
        .route("/", post(root))
        .with_state(trusted_clients);

    let listener = TcpListener::bind((addr.as_str(), port)).await.unwrap();
    info!("Listening at {addr}:{port}");
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, serde::Deserialize)]
struct DERPAdmitClientRequest {
    /// key to query for admission
    node_public: String,
    // /// derp client's IP address
    // source_ip: String,
}

async fn root(
    State(trusted_clients): State<Vec<String>>,
    Json(payload): Json<DERPAdmitClientRequest>,
) -> (StatusCode, &'static str) {
    let is_trusted = trusted_clients.contains(&payload.node_public);
    info!(
        "Trusted: {} for node_public: {}",
        is_trusted, payload.node_public
    );
    if is_trusted {
        (StatusCode::OK, "OK")
    } else {
        (StatusCode::UNAUTHORIZED, "Unauthorized")
    }
}
