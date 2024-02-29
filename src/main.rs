use axum::{http::StatusCode, routing::post, Json, Router};
use std::str::FromStr;
use tokio::net::TcpListener;
use tracing::{info, Level};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};

// FIXME: read from file
static TRUSTED_CLIENTS: [&str; 1] =
    ["nodekey:99233562637da21f590cbffa4b6b621adba721f12a2375ac1d63f8656cdb7a24"];

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

    let app = Router::new().route("/", post(root));
    // TODO: take port from env
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

async fn root(Json(payload): Json<DERPAdmitClientRequest>) -> (StatusCode, &'static str) {
    let is_trusted = TRUSTED_CLIENTS.contains(&payload.node_public.as_str());
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
