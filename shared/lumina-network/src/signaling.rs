use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

/// Messages matching the Signal Server's `IncomingMessage`.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action")]
pub enum SignalingMessage {
    RegisterHost { session_id: String },
    RegisterClient { session_id: String },
    SignalToClient { session_id: String, payload: String },
    SignalToHost { session_id: String, payload: String },
}

/// Client for connecting to the Lumina Signal Server to exchange IP/ICE data.
pub struct SignalingClient {
    pub session_id: String,
    pub is_host: bool,
    sender: mpsc::Sender<SignalingMessage>,
}

impl SignalingClient {
    /// Connects to the WebSocket signal server and starts background read/write tasks.
    pub async fn connect(url: &str, session_id: String, is_host: bool) -> Result<(Self, mpsc::Receiver<String>), String> {
        let parsed_url = Url::parse(url).map_err(|e| format!("Invalid URL: {}", e))?;
        
        let (ws_stream, _) = connect_async(parsed_url.as_str()).await.map_err(|e| format!("WS Connect error: {}", e))?;
        let (mut write, mut read) = ws_stream.split();

        // Channels to interface with the background tasks
        let (tx_out, mut rx_out) = mpsc::channel::<SignalingMessage>(100);
        let (tx_in, rx_in) = mpsc::channel::<String>(100);

        // Send initial registration packet
        let reg_msg = if is_host {
            SignalingMessage::RegisterHost { session_id: session_id.clone() }
        } else {
            SignalingMessage::RegisterClient { session_id: session_id.clone() }
        };
        
        let reg_text = serde_json::to_string(&reg_msg).unwrap();
        write.send(Message::Text(reg_text.into())).await.map_err(|e| e.to_string())?;

        // Background task for reading messages
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                if let Ok(Message::Text(text)) = msg {
                    // Send the raw payload to the caller
                    let _ = tx_in.send(text.to_string()).await;
                }
            }
        });

        // Background task for writing messages
        tokio::spawn(async move {
            while let Some(msg) = rx_out.recv().await {
                if let Ok(text) = serde_json::to_string(&msg) {
                    if write.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }
            }
        });

        Ok((
            Self {
                session_id,
                is_host,
                sender: tx_out,
            },
            rx_in,
        ))
    }

    /// Sends a payload to the peer (Host -> Client, or Client -> Host).
    pub async fn send_payload(&self, payload: String) -> Result<(), String> {
        let msg = if self.is_host {
            SignalingMessage::SignalToClient {
                session_id: self.session_id.clone(),
                payload,
            }
        } else {
            SignalingMessage::SignalToHost {
                session_id: self.session_id.clone(),
                payload,
            }
        };

        self.sender.send(msg).await.map_err(|e| e.to_string())
    }
}
