mod gameserver_util;
mod frp_util;
mod const_value;
mod common;

use axum::{extract::{
    ws::{Message, WebSocket, WebSocketUpgrade},
    State,
}, response::IntoResponse, routing::get, Json, Router};
use std::{sync::Arc, time::Duration};
use std::sync::Mutex;
use std::thread::sleep;
use axum::extract::ws;
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::post;
use futures::{SinkExt, StreamExt};
use tokio::sync::{broadcast};
use crate::common::get_index;
use crate::const_value::{FRPC_TOML_PATH, TCP_LOCAL_PORT, UDP_LOCAL_PORT};
use crate::frp_util::{frpc_config_read, frpc_config_reload, frpc_config_reset_by_index, frpc_config_write, Config, FrpcToml};
use crate::gameserver_util::{start_game_server};

enum AppError {
    // 专门用于处理读取配置文件失败的错误
    ConfigReadError(String),
    ConfigWriteError(String),
    ConfigReloadError(String),
    ConfigResetByIndexError(String),
    BadBodyError(String)
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::ConfigReadError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read config file: {}", msg),
            ),
            AppError::ConfigWriteError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to write config file: {}", msg),
            ),
            AppError::ConfigReloadError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to reload config file: {}", msg),
            ),
            AppError::BadBodyError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Bed body: {}", msg),
            ),
            AppError::ConfigResetByIndexError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to reset config file by index: {}", msg),
            )
        };

        (status, error_message).into_response()
    }
}

struct MasterState {
    gamer_server_running: bool,
    index: u8
}

#[tokio::main]
async fn main() {
    let index = get_index();
    println!("index is {}", index);
    let masterstate = Arc::new(Mutex::new(
        MasterState { gamer_server_running: false, index: index },
    ));

    let config = FrpcToml {
        server_addr: "124.223.27.133".to_string(),
        server_port: 7000,
        auth_token: "123456".to_string(),
        tcp_name: format!("7daysTodieServer-{}", index),
        tcp_remote_port: TCP_LOCAL_PORT + index as u16,
        udp_name: format!("7daysTodieServerUDP-{}", index),
        udp_remote_port: UDP_LOCAL_PORT + index as u16,
    };
    let _ = frpc_config_write(&config, FRPC_TOML_PATH).await.unwrap();
    let res = frpc_config_reload().await.unwrap();
    if !res.success() {
        return;
    }

    let (tx, _rx) = broadcast::channel(100);
    let app = Router::new()
        .route("/hello", get(|| async { "Hello, World!" }))
        .route("/status", get(status))
        .route("/start_7days", get(start_7days)).with_state(masterstate.clone())
        .route("/7daysserverlog", get(ws_handler))
        .with_state(Arc::new(AppState { tx }))
        .route("/get_frpc_toml", get(get_frpc_toml))
        .route("/reset_frpc_toml", post(reset_frpc_toml))
        .route("/reset_frpc_toml_by_index", post(reset_frpc_toml_by_index));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3005")
        .await
        .unwrap();

    let mut child = start_game_server().expect("Failed to start game server");
    {
        let mut state = masterstate.lock().unwrap(); // .unwrap() 用于处理锁可能被“毒化”的错误
        state.gamer_server_running = true;
        println!("Game server state set to running.");
    }
    tokio::spawn(async move {
        child.wait().await.expect("Failed to wait game server");
        let mut state = masterstate.lock().unwrap();
        state.gamer_server_running = false;
        println!("Game server shutdown");
    });

    println!("start listening on port 3005");
    axum::serve(listener, app).await.unwrap();
}

async fn status() -> &'static str {
    "index=0;7daysserrver=stop;"
}

async fn start_7days(State(masterstate): State<Arc<Mutex<MasterState>>>) -> (StatusCode, &'static str) {
    let mut state = masterstate.lock().unwrap();
    if state.gamer_server_running {
        return ( StatusCode::OK, "game server is running");
    }

    let masterstate2 = masterstate.clone();
    let mut child = start_game_server().expect("Failed to start game server");
    {
        let mut state = masterstate2.lock().unwrap(); // .unwrap() 用于处理锁可能被“毒化”的错误
        state.gamer_server_running = true;
        println!("Game server state set to running.");
    }

    let masterstate2 = masterstate.clone();
    tokio::spawn(async move {
        child.wait().await.expect("Failed to wait game server");
        let mut state = masterstate2.lock().unwrap();
        state.gamer_server_running = false;
        println!("Game server shutdown");
    });

    ( StatusCode::OK, "game server start to run." )
}

async fn get_frpc_toml() -> Result<Json<Config>, AppError> {
    match frpc_config_read(FRPC_TOML_PATH).await {
        Ok(config) => Ok(Json(config)),
        Err(e) => {
            println!("{:#?}", e);
            Err(AppError::ConfigReadError(e.to_string()))
        }
    }
}

async fn reset_frpc_toml(Json(config): Json<FrpcToml>) -> Result<StatusCode, AppError> {
    frpc_config_write(&config, FRPC_TOML_PATH)
        .await
        .map_err(|e| AppError::ConfigWriteError(e.to_string()))?;

    frpc_config_reload()
        .await
        .map_err(|e| AppError::ConfigReloadError(e.to_string()))?;

    Ok(StatusCode::OK)
}

async fn reset_frpc_toml_by_index(body: String) -> Result<StatusCode, AppError> {
    let index = body.parse::<u8>().map_err(|e| AppError::BadBodyError(e.to_string()))?;

    frpc_config_reset_by_index(FRPC_TOML_PATH, index)
        .await
        .map_err(|e| AppError::ConfigWriteError(e.to_string()))?;

    Ok(StatusCode::OK)
}

struct AppState {
    tx: broadcast::Sender<String>,
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    // Spawn a task to send messages to the client
    let mut send_task = tokio::spawn(async move {
        // while let Ok(msg) = rx.recv().await {
        //     let ws_msg = ws::Utf8Bytes::from(msg);
        //     if sender.send(Message::Text(ws_msg)).await.is_err() {
        //         break; // Client disconnected
        //     }
        // }
        let mut i = 0;
        loop {
            let ws_msg = ws::Utf8Bytes::from(format!("str line {}", i));
            if sender.send(Message::Text(ws_msg)).await.is_err() {
                break;
            }
            i += 1;
            sleep( Duration::from_secs(1));
        }
    });

    // Spawn a task to receive messages from the client
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                println!("Received: {}", text);
                // Broadcast received message to all connected clients
                // if state.tx.send(format!("Echo: {}", text)).is_err() {
                //     // No receivers, ignore
                // }
            }
        }
    });

    // Wait for either task to complete (e.g., client disconnects)
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
}

#[cfg(test)]
mod tests {
    use axum::Router;
    use axum::routing::get;
    use crate::{get_frpc_toml};

    #[tokio::test]
    async fn get_fpc_toml_test() {
        let app = Router::new()
            .route("/hello", get(|| async { "Hello, World!" }))
            .route("/get_fpc_toml", get(get_frpc_toml));

        let listener = tokio::net::TcpListener::bind("127.0.0.1:3105")
            .await
            .unwrap();

        axum::serve(listener, app).await.unwrap();
    }
}