use axum::{
    extract::Json,
    routing::{get, post},
    Router,
};
use ligames::TangoGenerator;
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    let app = Router::new()
        .route(
            "/api/tango-board",
            get(|| async {
                axum::Json(serde_json::json!(
                    TangoGenerator::generate_one_solution_tango()
                ))
            }),
        )
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
