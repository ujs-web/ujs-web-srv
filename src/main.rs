mod js_bridge;
mod db_bridge;

use axum::{Router, routing::any};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/js/{*script_path}", any(js_bridge::handle_js_script));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
