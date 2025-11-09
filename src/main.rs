mod gameserver_util;
mod frp_util;
mod const_value;
mod common;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use std::{sync::Arc, time::Duration};
use std::sync::Mutex;
use std::thread::sleep;
use axum::extract::ws;
use axum::http::StatusCode;
use futures::{SinkExt, StreamExt};
use tokio::sync::{broadcast};
use crate::common::get_index;
use crate::gameserver_util::{start_game_server};

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

    let (tx, _rx) = broadcast::channel(100);
    let app = Router::new()
        .route("/hello", get(|| async { "Hello, World!" }))
        .route("/status", get(status))
        .route("/start_7days", get(start_7days)).with_state(masterstate.clone())
        .route("/7daysserverlog", get(ws_handler))
        .with_state(Arc::new(AppState { tx }));

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