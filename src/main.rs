use axum::{http::StatusCode, routing::post, Json, Router};
use tokio::net::TcpListener;

// FIXME: read from file
static TRUSTED_CLIENTS: [&str; 1] =
    ["nodekey:99233562637da21f590cbffa4b6b621adba721f12a2375ac1d63f8656cdb7a24"];

#[tokio::main]
async fn main() {
    println!("running");
    let app = Router::new().route("/", post(root));
    // TODO: take port from env
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
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
    println!("payload: {:?}", payload);
    // check if node_public is in TRUSTED_CLIENTS
    if TRUSTED_CLIENTS.contains(&payload.node_public.as_str()) {
        (StatusCode::OK, "OK")
    } else {
        (StatusCode::UNAUTHORIZED, "Unauthorized")
    }
}
