mod db_bridge;
mod js_bridge;
mod static_server;
mod test_utils;
mod websocket;

use crate::js_bridge::jsonrpc::handle_json_rpc;
use crate::static_server::StaticServerConfig;
use axum::{
    Router,
    routing::{any, post},
};
use db_bridge::establish_connection_pool;
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use time::UtcOffset;
use tracing::Level;

#[tokio::main]
async fn main() {
    // 初始化日志系统,默认级别为 info
    // 创建 logs 文件夹并按天分割日志
    let file_appender = tracing_appender::rolling::daily("logs", "ujs-web-svr.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // 配置环境过滤器
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing::info!("env_filter: {}",env_filter.to_string());
    // 设置时区为北京时间（UTC+8）
    let beijing_offset = UtcOffset::from_hms(8, 0, 0).unwrap();
    tracing_subscriber::registry()
        .with(env_filter) // 设置日志级别
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking) // 输出到文件
                .with_ansi(false) // 禁用 ANSI 颜色代码
                .with_file(true) //打印文件名
                .with_line_number(true) //打印行号
                .with_thread_ids(true) //打印线程ID
                .with_thread_names(true) //打印线程名称
                .with_target(false) //不打印target
                .with_timer(tracing_subscriber::fmt::time::OffsetTime::new(beijing_offset, time::format_description::well_known::Iso8601::DEFAULT)),
        )
        .init();

    let pool = establish_connection_pool();
    let ws_state = websocket::create_websocket_state();

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

    let trace_layer = TraceLayer::new_for_http()
        // 1. 在 Span 里带上 “method + uri”
        // .make_span_with(DefaultMakeSpan::new()
        //     .level(Level::INFO)
        //     .include_headers(false))
        // 2. 请求到达时打印一行
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        // 3. 响应返回时打印一行（含 status + latency）
        .on_response(DefaultOnResponse::new().level(Level::INFO))
    ;

    let app = Router::new()
        .route("/js/{*script_path}", any(js_bridge::handle_js_script))
        .route("/rpc", post(handle_json_rpc))
        .route("/ws", axum::routing::get(websocket::handle_websocket))
        .with_state((pool, ws_state))
        .merge(static_router)
        .layer(trace_layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    tracing::info!("rust_demo listening on {}", listener.local_addr().unwrap());
    println!("listening on {}", listener.local_addr().unwrap());

    // 保持 guard 存活,确保日志写入器不被关闭
    let _guard = guard;
    axum::serve(listener, app).await.unwrap();
}
