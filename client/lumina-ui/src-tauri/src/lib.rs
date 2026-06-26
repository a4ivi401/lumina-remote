use tauri::Emitter;
use std::sync::Mutex;
use lazy_static::lazy_static;
use tokio::sync::mpsc;
use lumina_input::InputEvent;
use lumina_encoder::{VideoEncoder, SystemEncoder};

#[tauri::command]
fn get_local_device_id(app: tauri::AppHandle) -> String {
    let mut id_lock = DEVICE_ID.lock().unwrap();
    if id_lock.is_empty() {
        if let Ok(config) = load_config(&app) {
            if !config.device_id.is_empty() {
                *id_lock = config.device_id.clone();
                return config.device_id.clone();
            }
        }
        
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let r1: u32 = rng.gen_range(1000..9999);
        let r2: u32 = rng.gen_range(1000..9999);
        let new_id = format!("LMN-{}-{}", r1, r2);
        *id_lock = new_id.clone();
        
        if let Ok(mut config) = load_config(&app) {
            config.device_id = new_id.clone();
            let _ = save_config(&app, &config);
        }
        
        new_id
    } else {
        id_lock.clone()
    }
}

lazy_static! {
    static ref DEVICE_ID: Mutex<String> = Mutex::new(String::new());
    static ref HOST_PIN: Mutex<String> = Mutex::new(String::new());
    static ref INPUT_SENDER: Mutex<Option<mpsc::UnboundedSender<InputEvent>>> = Mutex::new(None);
    static ref PENDING_CONNECTION: Mutex<Option<tokio::sync::oneshot::Sender<bool>>> = Mutex::new(None);
}

#[tauri::command]
fn respond_to_connection(accept: bool) {
    let mut lock = PENDING_CONNECTION.lock().unwrap();
    if let Some(sender) = lock.take() {
        let _ = sender.send(accept);
    }
}

#[tauri::command]
fn send_input(event: String) {
    if let Ok(input_evt) = serde_json::from_str::<InputEvent>(&event) {
        if let Some(sender) = INPUT_SENDER.lock().unwrap().as_ref() {
            let _ = sender.send(input_evt);
        }
    }
}

#[tauri::command]
fn generate_session_pin() -> String {
    use rand::Rng;
    const ALPHABET: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";
    let mut rng = rand::thread_rng();
    let mut pin = String::with_capacity(14);
    for i in 0..12 {
        if i > 0 && i % 4 == 0 { pin.push('-'); }
        let idx = rng.gen_range(0..ALPHABET.len());
        pin.push(ALPHABET[idx] as char);
    }
    *HOST_PIN.lock().unwrap() = pin.clone();
    pin
}

use std::path::PathBuf;
use std::fs;
use serde::{Serialize, Deserialize};
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone)]
struct SavedMachine {
    id: String,
    name: String,
    is_online: bool,
    last_connected: u64,
}

fn get_machines_file(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    if !app_data.exists() { fs::create_dir_all(&app_data).map_err(|e| e.to_string())?; }
    Ok(app_data.join("machines.json"))
}

#[tauri::command]
fn get_saved_machines(app: tauri::AppHandle) -> Result<Vec<SavedMachine>, String> {
    let file_path = get_machines_file(&app)?;
    if !file_path.exists() { return Ok(Vec::new()); }
    let content = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let machines: Vec<SavedMachine> = serde_json::from_str(&content).unwrap_or_default();
    Ok(machines)
}

#[derive(Serialize, Deserialize, Clone)]
struct AppConfig {
    device_id: String,
    signal_server: String,
    stun_server: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            device_id: String::new(),
            signal_server: "ws://lumina.a4ivi4.dev/ws".to_string(),
            stun_server: "stun.l.google.com:19302".to_string(),
        }
    }
}

fn get_config_file(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    if !app_data.exists() { fs::create_dir_all(&app_data).map_err(|e| e.to_string())?; }
    Ok(app_data.join("config.json"))
}

fn save_config(app: &tauri::AppHandle, config: &AppConfig) -> Result<(), String> {
    let file_path = get_config_file(app)?;
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(file_path, content).map_err(|e| e.to_string())?;
    Ok(())
}

fn load_config(app: &tauri::AppHandle) -> Result<AppConfig, String> {
    let file_path = get_config_file(app)?;
    if !file_path.exists() {
        let default_config = AppConfig::default();
        let content = serde_json::to_string_pretty(&default_config).map_err(|e| e.to_string())?;
        fs::write(file_path, content).map_err(|e| e.to_string())?;
        return Ok(default_config);
    }
    let content = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let config: AppConfig = serde_json::from_str(&content).unwrap_or_default();
    Ok(config)
}

#[tauri::command]
async fn check_signal_server(_app: tauri::AppHandle) -> Result<bool, String> {
    Ok(true) 
}

#[tauri::command]
async fn get_local_network_devices(app: tauri::AppHandle) -> Result<Vec<SavedMachine>, String> {
    let local_id = get_local_device_id(app);
    let discovered = lumina_network::mdns_discovery::discover_all_local_hosts(2).await?;
    let mut machines = Vec::new();
    for id in discovered {
        if id != local_id {
            machines.push(SavedMachine {
                id: id.clone(),
                name: format!("{} (LAN)", id),
                is_online: true,
                last_connected: 0,
            });
        }
    }
    Ok(machines)
}

#[derive(Serialize, Clone)]
struct VideoFramePayload {
    data: String,
    is_keyframe: bool,
    timestamp_us: u64,
}

#[tauri::command]
async fn connect_to_device(
    app: tauri::AppHandle,
    partner_id: String,
    pin: String,
    save_machine: bool,
) -> Result<String, String> {
    let config = load_config(&app)?;
    let manager = lumina_network::manager::ConnectionManager::new(
        config.stun_server,
        config.signal_server,
    );
    let path = manager.establish_path(&partner_id).await?;
    
    let addr = match path {
        lumina_network::manager::ConnectionPath::DirectLan(addr) => addr,
        _ => return Err("Only LAN is currently supported for this video test.".into()),
    };

    println!("[Lumina] Found via LAN! Connecting QUIC to {}", addr);
    let client_addr: std::net::SocketAddr = "0.0.0.0:0".parse().unwrap();
    let endpoint = lumina_network::create_client_endpoint(client_addr)
        .map_err(|e| format!("Failed to create QUIC endpoint: {}", e))?;
        
    let connect_task = endpoint.connect(addr, "lumina.a4ivi4.dev")
        .map_err(|e| format!("Failed to connect: {}", e))?;
        
    let conn = connect_task.await
        .map_err(|e| format!("QUIC connection failed: {}", e))?;

    let clean_pin = pin.replace("-", "");
    let (secret, _) = lumina_core::derive_key_pair(&clean_pin);
    
    println!("[Lumina] Performing Zero-Trust Handshake...");
    lumina_network::handshake::perform_handshake(&conn, false, &secret)
        .await
        .map_err(|e| format!("Handshake failed (Wrong PIN or MITM): {}", e))?;
        
    println!("[Lumina] Connection ACCEPTED and secured!");

    let mut recv_video = conn.accept_uni().await
        .map_err(|e| format!("Failed to accept video stream: {}", e))?;
        
    let mut send_input = conn.open_uni().await
        .map_err(|e| format!("Failed to open input stream: {}", e))?;
        
    let (tx, mut rx) = mpsc::unbounded_channel::<InputEvent>();
    *INPUT_SENDER.lock().unwrap() = Some(tx);
    
    tokio::spawn(async move {
        while let Some(evt) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&evt) {
                let bytes = json.as_bytes();
                let size = bytes.len() as u32;
                if send_input.write_all(&size.to_be_bytes()).await.is_err() { break; }
                if send_input.write_all(bytes).await.is_err() { break; }
            }
        }
    });
    
    let app_handle = app.clone();
    tokio::spawn(async move {
        loop {
            let mut meta_buf = [0u8; 13]; 
            if recv_video.read_exact(&mut meta_buf).await.is_err() { break; }
            
            let size = u32::from_be_bytes(meta_buf[0..4].try_into().unwrap()) as usize;
            let is_keyframe = meta_buf[4] == 1;
            let timestamp_us = u64::from_be_bytes(meta_buf[5..13].try_into().unwrap());
            
            let mut frame_buf = vec![0u8; size];
            if recv_video.read_exact(&mut frame_buf).await.is_err() { break; }
            
            use base64::{Engine as _, engine::general_purpose::STANDARD};
            let b64 = STANDARD.encode(&frame_buf);
            
            let payload = VideoFramePayload {
                data: b64,
                is_keyframe,
                timestamp_us,
            };
            
            let _ = app_handle.emit("video-frame", payload);
        }
        println!("[Lumina] Video stream disconnected");
    });
    
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    
    if save_machine {
        let file_path = get_machines_file(&app)?;
        let mut machines = get_saved_machines(app.clone()).unwrap_or_default();
        machines.retain(|m| m.id != partner_id);
        machines.push(SavedMachine {
            id: partner_id.clone(),
            name: format!("Machine {}", &partner_id[..4]),
            is_online: true,
            last_connected: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        });
        let content = serde_json::to_string_pretty(&machines).map_err(|e| e.to_string())?;
        fs::write(file_path, content).map_err(|e| e.to_string())?;
    }
    
    Ok(format!("Successfully connected to {}", partner_id))
}

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let my_id = get_local_device_id(app_handle.clone());
                let port = 4433; 
                
                match lumina_network::mdns_discovery::advertise_local_service(port, &my_id) {
                    Ok(_daemon) => {
                        println!("[Lumina] Successfully advertising mDNS on LAN as: {}", my_id);
                        
                        let bind_addr: std::net::SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
                        if let Ok(endpoint) = lumina_network::create_server_endpoint(bind_addr) {
                            println!("[Lumina] Listening for incoming QUIC connections on port {}", port);
                            
                            loop {
                                if let Some(incoming) = endpoint.accept().await {
                                    println!("[Lumina] Incoming QUIC connection...");
                                    
                                    tokio::spawn(async move {
                                        match incoming.await {
                                            Ok(conn) => {
                                                let expected_pin = HOST_PIN.lock().unwrap().replace("-", "");
                                                let (secret, _) = lumina_core::derive_key_pair(&expected_pin);
                                                
                                                println!("[Lumina] Performing Zero-Trust handshake...");
                                                if let Err(e) = lumina_network::handshake::perform_handshake(&conn, true, &secret).await {
                                                    println!("[Lumina] Handshake rejected: {}", e);
                                                    return;
                                                }
                                                println!("[Lumina] Handshake verified. Connection secure.");
                                                
                                                let (tx, rx) = tokio::sync::oneshot::channel();
                                                *PENDING_CONNECTION.lock().unwrap() = Some(tx);
                                                
                                                let partner_addr = conn.remote_address().to_string();
                                                let _ = app_handle.emit("incoming-connection", partner_addr);
                                                
                                                if let Ok(accepted) = rx.await {
                                                    if !accepted {
                                                        println!("[Lumina] Connection rejected by user.");
                                                        return;
                                                    }
                                                } else {
                                                    return;
                                                }
                                                
                                                let mut send_video = match conn.open_uni().await {
                                                    Ok(s) => s,
                                                    Err(e) => { println!("Failed to open video stream: {}", e); return; }
                                                };
                                                
                                                let mut recv_input = match conn.accept_uni().await {
                                                    Ok(s) => s,
                                                    Err(e) => { println!("Failed to accept input stream: {}", e); return; }
                                                };
                                                
                                                let (input_tx, input_rx) = std::sync::mpsc::channel::<InputEvent>();
                                                
                                                std::thread::spawn(move || {
                                                    let mut controller = lumina_input::InputController::new();
                                                    while let Ok(evt) = input_rx.recv() {
                                                        controller.handle_event(evt);
                                                    }
                                                });

                                                tokio::spawn(async move {
                                                    loop {
                                                        let mut size_buf = [0u8; 4];
                                                        if recv_input.read_exact(&mut size_buf).await.is_err() { break; }
                                                        let size = u32::from_be_bytes(size_buf) as usize;
                                                        let mut json_buf = vec![0u8; size];
                                                        if recv_input.read_exact(&mut json_buf).await.is_err() { break; }
                                                        
                                                        if let Ok(json_str) = String::from_utf8(json_buf) {
                                                            if let Ok(evt) = serde_json::from_str::<InputEvent>(&json_str) {
                                                                let _ = input_tx.send(evt);
                                                            }
                                                        }
                                                    }
                                                });
                                                
                                                let (frame_tx, mut frame_rx) = tokio::sync::mpsc::channel::<lumina_encoder::EncodedPacket>(30);
                                                
                                                std::thread::spawn(move || {
                                                    match lumina_capture::create_capture_device() {
                                                        Ok(mut capturer) => {
                                                            println!("[Lumina] Capture device created. Starting Native OS H.264 Encoder...");
                                                            if let Ok(mut encoder) = SystemEncoder::new(1920, 1080, 30) {
                                                                loop {
                                                                    match capturer.capture_frame() {
                                                                        Ok(frame) => {
                                                                            if let Ok(packets) = encoder.encode_frame(&frame) {
                                                                                for pkt in packets {
                                                                                    if frame_tx.blocking_send(pkt).is_err() {
                                                                                        println!("[Lumina] Video stream receiver dropped.");
                                                                                        return;
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                        Err(e) => println!("[Lumina] capture_frame error: {}", e),
                                                                    }
                                                                    std::thread::sleep(std::time::Duration::from_millis(33));
                                                                }
                                                            } else {
                                                                println!("[Lumina] Failed to initialize FFmpeg Encoder!");
                                                            }
                                                        }
                                                        Err(e) => println!("[Lumina] FATAL: Failed to create capture device: {}", e),
                                                    }
                                                });

                                                tokio::spawn(async move {
                                                    while let Some(pkt) = frame_rx.recv().await {
                                                        let size = pkt.data.len() as u32;
                                                        let is_key = if pkt.is_keyframe { 1u8 } else { 0u8 };
                                                        let mut meta = [0u8; 13];
                                                        meta[0..4].copy_from_slice(&size.to_be_bytes());
                                                        meta[4] = is_key;
                                                        meta[5..13].copy_from_slice(&pkt.timestamp_us.to_be_bytes());
                                                        
                                                        if send_video.write_all(&meta).await.is_err() { break; }
                                                        if send_video.write_all(&pkt.data).await.is_err() { break; }
                                                    }
                                                });
                                            }
                                            Err(e) => println!("[Lumina] Connection failed: {}", e),
                                        }
                                    });
                                }
                            }
                        }
                    }
                    Err(e) => println!("[Lumina] Failed to advertise mDNS: {}", e),
                }
            });

            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show Dashboard", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .icon(app.default_window_icon().unwrap().clone())
                .build(app)?;
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_local_device_id,
            generate_session_pin,
            connect_to_device,
            get_saved_machines,
            get_local_network_devices,
            send_input,
            check_signal_server,
            respond_to_connection
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
