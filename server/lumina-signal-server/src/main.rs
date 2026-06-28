use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use dashmap::DashMap;
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, net::SocketAddr};
use tokio::sync::mpsc;
use tracing::{info, warn};

/// Shared in-memory state for the signal server.
struct AppState {
    // Session ID -> Host's TX channel
    hosts: DashMap<String, mpsc::Sender<String>>,
    // Session ID -> Client's TX channel
    clients: DashMap<String, mpsc::Sender<String>>,
    
    // WebSocket Relay Tunnel Channels
    tunnel_hosts: DashMap<String, mpsc::Sender<Message>>,
    tunnel_clients: DashMap<String, mpsc::Sender<Message>>,
}

/// The messages the Signal Server expects to receive.
#[derive(Deserialize, Debug)]
#[serde(tag = "action")]
enum IncomingMessage {
    RegisterHost { session_id: String },
    RegisterClient { session_id: String },
    SignalToClient { session_id: String, payload: String },
    SignalToHost { session_id: String, payload: String },
}

#[tokio::main]
async fn main() {
    // Initialize tracing (logging)
    tracing_subscriber::fmt::init();

    let state = Arc::new(AppState {
        hosts: DashMap::new(),
        clients: DashMap::new(),
        tunnel_hosts: DashMap::new(),
        tunnel_clients: DashMap::new(),
    });

    let app = Router::new()
        .route("/ws", get(ws_handler))
        // FIX: axum 0.7 uses {param} syntax instead of :param
        .route("/tunnel/{session_id}/{role}", get(tunnel_handler))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Signal server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // Create a channel so other tasks (like another WebSocket handler) can send messages to this WebSocket
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Forwarding task: take messages from the channel and send them over the WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let mut session_id = String::new();
    let mut role = String::new();

    let state_clone = state.clone();

    // Receiving task: read messages from the WebSocket and route them
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            match serde_json::from_str::<IncomingMessage>(&text) {
                Ok(msg) => {
                    match msg {
                        IncomingMessage::RegisterHost { session_id: sid } => {
                            info!("Host registered for session: {}", sid);
                            session_id = sid.clone();
                            role = "host".to_string();
                            state_clone.hosts.insert(sid, tx.clone());
                        }
                        IncomingMessage::RegisterClient { session_id: sid } => {
                            info!("Client registered for session: {}", sid);
                            session_id = sid.clone();
                            role = "client".to_string();
                            state_clone.clients.insert(sid, tx.clone());
                        }
                        IncomingMessage::SignalToClient { session_id: sid, payload } => {
                            if let Some(client_tx) = state_clone.clients.get(&sid) {
                                let _ = client_tx.send(payload).await;
                            } else {
                                warn!("Client not found for session: {}", sid);
                            }
                        }
                        IncomingMessage::SignalToHost { session_id: sid, payload } => {
                            if let Some(host_tx) = state_clone.hosts.get(&sid) {
                                let _ = host_tx.send(payload).await;
                            } else {
                                warn!("Host not found for session: {}", sid);
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to parse message: {}. Payload: {}", e, text);
                }
            }
        }

        // Cleanup when the WebSocket disconnects
        if !session_id.is_empty() {
            if role == "host" {
                state_clone.hosts.remove(&session_id);
                info!("Host disconnected from session: {}", session_id);
            } else if role == "client" {
                state_clone.clients.remove(&session_id);
                info!("Client disconnected from session: {}", session_id);
            }
        }
    });

    // If any one of the tasks complete (meaning connection closed or error), abort the other
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
}

async fn tunnel_handler(
    ws: WebSocketUpgrade,
    Path((session_id, role)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_tunnel_socket(socket, session_id, role, state))
}

async fn handle_tunnel_socket(socket: WebSocket, session_id: String, role: String, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    
    // Channel buffer size increased to 256 for video stream throughput
    let (tx, mut rx) = mpsc::channel::<Message>(256);
    
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    if role == "host" {
        state.tunnel_hosts.insert(session_id.clone(), tx.clone());
        info!("Host joined tunnel: {}", session_id);
    } else {
        state.tunnel_clients.insert(session_id.clone(), tx.clone());
        info!("Client joined tunnel: {}", session_id);
    }

    let state_clone = state.clone();
    let session_clone = session_id.clone();
    let role_clone = role.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            // Forward all message types (Text, Binary, Ping, Pong) between peers
            match &msg {
                Message::Text(_) | Message::Binary(_) => {
                    if role_clone == "host" {
                        if let Some(client_tx) = state_clone.tunnel_clients.get(&session_clone) {
                            let _ = client_tx.send(msg).await;
                        }
                    } else {
                        if let Some(host_tx) = state_clone.tunnel_hosts.get(&session_clone) {
                            let _ = host_tx.send(msg).await;
                        }
                    }
                }
                Message::Ping(data) => {
                    // Respond with pong
                    if role_clone == "host" {
                        if let Some(client_tx) = state_clone.tunnel_clients.get(&session_clone) {
                            let _ = client_tx.send(Message::Pong(data.clone())).await;
                        }
                    } else {
                        if let Some(host_tx) = state_clone.tunnel_hosts.get(&session_clone) {
                            let _ = host_tx.send(Message::Pong(data.clone())).await;
                        }
                    }
                }
                _ => {}
            }
        }
        
        if role_clone == "host" {
            state_clone.tunnel_hosts.remove(&session_clone);
        } else {
            state_clone.tunnel_clients.remove(&session_clone);
        }
        info!("{} disconnected from tunnel: {}", role_clone, session_clone);
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
}
