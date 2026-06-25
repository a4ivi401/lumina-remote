use serde::{Deserialize, Serialize};

/// The main enumeration of all messages sent over the established QUIC P2P tunnel.
/// We use `bincode` for ultra-fast binary serialization.
#[derive(Serialize, Deserialize, Debug)]
pub enum LuminaMessage {
    /// A compressed video frame (H.264) sent from Host -> Client
    Video(VideoFrame),
    
    /// An input command (Mouse/Keyboard) sent from Client -> Host
    Input(InputEvent),
    
    /// Quality of Service (QoS) telemetry sent periodically from Client -> Host
    Qos(QosMetrics),
    
    /// Commands to control the flow of the video stream (e.g. pause/resume)
    StreamControl(StreamState),
}

/// Represents the requested state of the video stream.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamState {
    /// The client window is active and focused. Host should send frames normally.
    Active,
    /// The client window is minimized or out of focus. Host should pause capturing to save CPU/GPU.
    Paused,
}

/// Represents a single video frame in the pipeline.
#[derive(Serialize, Deserialize, Debug)]
pub struct VideoFrame {
    /// Incrementing counter to detect lost or out-of-order frames
    pub sequence_number: u64,
    /// Time when the frame was captured (useful for latency calculations)
    pub timestamp_ms: u64,
    /// True if this is an I-Frame (can be decoded independently)
    pub is_keyframe: bool,
    /// The compressed video payload
    pub data: Vec<u8>,
}

/// Represents a user input action.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum InputEvent {
    /// Mouse movement. Coordinates are normalized (0.0 to 1.0) so they 
    /// work correctly regardless of the Client's window size.
    MouseMove { x: f32, y: f32 },
    /// Mouse button press/release
    MouseClick { button: u8, down: bool },
    /// Keyboard key press/release
    KeyPress { key_code: u32, down: bool },
}

/// Telemetry metrics sent from the Client back to the Host every ~500ms.
/// The Host uses this data to adjust the Video Encoder (Adaptive Bitrate).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QosMetrics {
    /// Round Trip Time (latency) in milliseconds
    pub rtt_ms: u32,
    /// Number of video frames lost or dropped since the last report
    pub dropped_frames: u32,
    /// Average time it takes the Client's hardware to decode a frame (ms)
    pub avg_decode_time_ms: u32,
    /// (Optional) Client's estimation of available network bandwidth in kbps
    pub estimated_bandwidth_kbps: Option<u32>,
}
