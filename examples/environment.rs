use std::env;
use tower_webflow::{WebflowLayer, box_err_to_res};

use axum::{Router, response::Html, routing::post};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let secret = env::var("WEBFLOW_SECRET").expect("WEBFLOW_SECRET must be set");

    let webflow_layer = tower::ServiceBuilder::new()
        .layer(axum::error_handling::HandleErrorLayer::new(box_err_to_res))
        .layer(WebflowLayer::new(secret));

    let app = Router::new()
        .route("/", post(handler))
        .route_layer(webflow_layer);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await;
}

async fn handler() -> Html<&'static str> {
    tracing::info!("made it to the handler!");
    Html("<h1>Hello, World!</h1>")
}
