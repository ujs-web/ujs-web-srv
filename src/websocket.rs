use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::stream::StreamExt;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct WebSocketState {
    pub tx: broadcast::Sender<String>,
}

pub async fn handle_websocket(
    ws: WebSocketUpgrade,
    State((_, state)): State<(crate::db_bridge::DbPool, Arc<WebSocketState>)>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<WebSocketState>) {
    use futures_util::sink::SinkExt;
    
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    let receive_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    tracing::info!("收到消息: {}", text);
                    
                    // 回显消息
                    let response = format!("服务器回复: {}", text);
                    if let Err(e) = state.tx.send(response) {
                        tracing::error!("广播消息失败: {}", e);
                    }
                }
                Message::Close(_) => {
                    tracing::info!("客户端断开连接");
                    break;
                }
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = receive_task => {},
    }
}

pub fn create_websocket_state() -> Arc<WebSocketState> {
    let (tx, _) = broadcast::channel(100);
    Arc::new(WebSocketState { tx })
}