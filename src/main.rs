mod js_bridge;
mod db_bridge;
mod static_server;
mod test_utils;

use axum::{
    Router,
    routing::{any, post},
};
use db_bridge::establish_connection_pool;
use crate::js_bridge::jsonrpc::handle_json_rpc;
use crate::static_server::StaticServerConfig;

#[tokio::main]
async fn main() {
    let pool = establish_connection_pool();

    // 配置静态服务器
    let static_config = StaticServerConfig::new()
        .add_dir(
            crate::static_server::StaticDirConfig::new("/static", "static")
                .with_compression(true)
                .with_cache_max_age(3600), // 1 小时
        )
        .add_dir(
            crate::static_server::StaticDirConfig::new("/assets", "static")
                .with_compression(true)
                .with_cache_max_age(86400), // 24 小时
        )
        .add_dir(
            crate::static_server::StaticDirConfig::new("/images", "static/images")
                .with_compression(true)
                .with_cache_max_age(604800), // 7 天
        )
        .with_cors(true)
        .with_trace(false);

    let static_router = static_config.build_router();

    let app = Router::new()
        .route("/js/{*script_path}", any(js_bridge::handle_js_script))
        .route("/rpc", post(handle_json_rpc))
        .with_state(pool)
        .merge(static_router);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
