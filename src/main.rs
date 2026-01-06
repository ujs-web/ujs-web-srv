mod js_bridge;
mod db_bridge;

use axum::{Router, routing::{any, post}};
use db_bridge::establish_connection_pool;

#[tokio::main]
async fn main() {
    let pool = establish_connection_pool();
    let app = Router::new()
        .route("/js/{*script_path}", any(js_bridge::handle_js_script))
        .route("/rpc", post(js_bridge::handle_json_rpc))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
