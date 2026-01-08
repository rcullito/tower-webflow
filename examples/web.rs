use tower_webflow::{WebflowLayer, box_err_to_res};

use axum::{Router, response::Html, routing::get};

const secret: &'static str = "hi";

#[tokio::main]
async fn main() {
    let webflow_layer = tower::ServiceBuilder::new()
        .layer(axum::error_handling::HandleErrorLayer::new(box_err_to_res))
        .layer(WebflowLayer::new(secret));

        let app = Router::new()
        .route("/", get(handler))
        .route_layer(webflow_layer);

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await;
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
