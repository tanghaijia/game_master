mod gameserver_util;
mod frp_util;
mod const_value;
mod common;
mod game_config_util;
mod data_server_util;
mod error;
mod s3;
mod archive;

use axum::{extract::{
    ws::{Message, WebSocket, WebSocketUpgrade},
    State,
}, response::IntoResponse, routing::get, Json, Router};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;
use std::thread::sleep;
use axum::extract::{ws, Query};
use axum::http::StatusCode;
use axum::routing::post;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::process::{Child, Command};
use tokio::sync::{broadcast};
use crate::archive::unzip;
use crate::common::get_index;
use crate::const_value::{FRPC_TOML_PATH, SEVENDAYS_SERVER_SAVEFILE_PARENT_PATH, SEVENDAYS_SERVER_SAVEFILE_PATH, TCP_LOCAL_PORT, TEMP_DIR, UDP_LOCAL_PORT};
use crate::data_server_util::{get_game_config_by_serverconfig_id, get_savefile_info_by_save_file_id};
use crate::error::AppError;
use crate::frp_util::{frpc_config_read, frpc_config_reload, frpc_config_reset_by_index, frpc_config_write, Config, FrpcToml};
use crate::game_config_util::{GameConfigUtil, ServerSettings};
use crate::gameserver_util::{start_game_server};
use crate::s3::{download_file, get_rustfs_client};

struct MasterState {
    gamer_server_running: bool,
    index: u8,
    seven_days_child: Option<Child>,
    days7_pid: Option<u32>,
}

async fn init() -> anyhow::Result<()> {
    fs::create_dir_all(TEMP_DIR).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    init().await.expect("Init failed");

    let index = get_index().unwrap();
    println!("index is {}", index);

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

    // 初始化GameConfigUtil
    let settings_data = ServerSettings {
        server_name: "Local Game Host".to_string(),
        server_description: "A 7 Days to Die server".to_string(),
        server_password: "".to_string(),
        language: "English".to_string(),
        server_max_player_count: 8,
        eac_enabled: false,
        game_difficulty: 1,
        party_shared_kill_range: 100,
        player_killing_mode: 3
    };
    let game_config_util = GameConfigUtil::new();
    game_config_util.set_serverconfig_xml(&settings_data).await.expect("Set serverconfig.xml Error");

    // 初始化state
    let masterstate = Arc::new(Mutex::new(
        MasterState { gamer_server_running: false, index: index, seven_days_child: None, days7_pid: None },
    ));

    let (tx, _rx) = broadcast::channel(100);
    let app = Router::new()
        .route("/hello", get(|| async { "Hello, World!" }))
        .route("/status", get(status))
            .with_state(masterstate.clone())
        .route("/start_7days", get(start_7days))
            .with_state(masterstate.clone())
        .route("/stop_7days", get(stop_7days))
            .with_state(masterstate.clone())
        .route("/7daysserverlog", get(ws_handler))
            .with_state(Arc::new(AppState { tx }))
        .route("/get_frpc_toml", get(get_frpc_toml))
        .route("/reset_frpc_toml", post(reset_frpc_toml))
        .route("/reset_frpc_toml_by_index", post(reset_frpc_toml_by_index));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3005")
        .await
        .unwrap();

    println!("start listening on port 3005");
    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize, Debug)]
struct Status {
    index: u8,
    days7server_running: bool,
}
async fn status(State(masterstate): State<Arc<Mutex<MasterState>>>) -> Result<Json<Status>, AppError> {
    let index = get_index().unwrap_or_else(|e| 255);
    let state = masterstate.lock().await;
    let status = Status {
        index,
        days7server_running: state.gamer_server_running,
    };
    Ok(Json(status))
}

#[derive(Deserialize, Debug)]
struct Start7DaysParam {
    serverconfig_id: i32,
    save_file_id: i32
}
#[axum::debug_handler]
async fn start_7days(
    State(masterstate): State<Arc<Mutex<MasterState>>>,
    Query(params): Query<Start7DaysParam>) -> Result<StatusCode, AppError> {
    println!("start 7days by serverconfig_id: {} save_file_id: {} ...", params.serverconfig_id, params.save_file_id);

    // // 配置serverconfig.xml
    let game_config = get_game_config_by_serverconfig_id(params.serverconfig_id)
        .await
        .map_err(|e| AppError::DataServerFucError(e.to_string()))?;
    let game_config_util = GameConfigUtil::new();
    game_config_util.set_serverconfig_xml(&game_config).await.map_err(|e| AppError::SetServerConfigXmlErrror(e.to_string()))?;

    // 拉取存档
    let savefile_info = get_savefile_info_by_save_file_id(params.save_file_id)
        .await
        .map_err(|e| AppError::DataServerFucError(e.to_string()))?;
    let s3client = get_rustfs_client(Some(savefile_info.host)).await.map_err(|e| AppError::GetS3ClientError(e.to_string()))?;
    let filepath = format!("{}/{}", TEMP_DIR, savefile_info.name);
    let _ = download_file(&s3client, filepath.as_str(), savefile_info.bucket_name.as_str(),
                  format!("{}/{}", savefile_info.user_id, savefile_info.name).as_str())
        .await
        .map_err(|e| AppError::DownloadError(e.to_string()))?;
    let _ = fs::remove_dir_all(SEVENDAYS_SERVER_SAVEFILE_PATH).await;
    unzip(filepath.as_str(), SEVENDAYS_SERVER_SAVEFILE_PARENT_PATH)
        .map_err(|e| AppError::UnzipError(e.to_string()))?;

    // 启动7days
    // 获取state
    {
        let mut state = masterstate.lock().await;
        if state.gamer_server_running {
            return Err(AppError::GameIsRunning);
        }
    }

    let masterstate2 = masterstate.clone();
    tokio::spawn(async move {
        let mut child = start_game_server().expect("Failed to start game server");
        {
            let mut state = masterstate2.lock().await;
            state.gamer_server_running = true;
            state.days7_pid = child.id();
            println!("Game server state set to running.");
        }
        child.wait().await.expect("Failed to wait game server");
        {
            let mut state = masterstate2.lock().await;
            state.gamer_server_running = false;
            state.seven_days_child = None;
            println!("Game server shutdown");
        };
    });

    Ok(StatusCode::OK)
}


#[axum::debug_handler]
async fn stop_7days(
    State(masterstate): State<Arc<Mutex<MasterState>>>) -> Result<StatusCode, AppError> {
    println!("stop 7days ...");

    // 启动7days
    // 获取state
    {
        let mut state = masterstate.lock().await;
        if !state.gamer_server_running {
            return Ok(StatusCode::OK);
        }

        let pid = state.days7_pid.take().unwrap();
        let mut cmd = Command::new("kill")
            .arg("-2")
            .arg(pid.to_string())
            .spawn()
            .map_err(|e| AppError::KillCommandError(e.to_string()))?;
        let _ = cmd.wait().await.map_err(|e| AppError::KillCommandError(e.to_string()))?;
        println!("Send kill command to {} prrocess", pid);
    }

    Ok(StatusCode::OK)
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
        .map_err(|e| AppError::ConfigResetByIndexError(e.to_string()))?;

    println!("reset frpc toml by index: {}", index);
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
    use tokio::fs;
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

    #[tokio::test]
    async fn create_dir_test() {
        fs::create_dir_all("C:\\Users\\89396\\projects\\game_master\\script").await.unwrap();
    }
}