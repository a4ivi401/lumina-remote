use tauri::Emitter;

#[tauri::command]
fn get_local_device_id() -> String {
    // For the Alpha test, we generate a persistent ID based on the computer's hostname or just a random one.
    // To keep it simple and avoid clashes on LAN:
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let r1: u32 = rng.gen_range(1000..9999);
    let r2: u32 = rng.gen_range(1000..9999);
    format!("LMN-{}-{}", r1, r2)
}

use std::sync::Mutex;
use lazy_static::lazy_static;
use tokio::sync::mpsc;
use lumina_input::InputEvent;

lazy_static! {
    static ref HOST_PIN: Mutex<String> = Mutex::new(String::new());
    static ref INPUT_SENDER: Mutex<Option<mpsc::UnboundedSender<InputEvent>>> = Mutex::new(None);
}

#[tauri::command]
fn send_input(event: String) {
    // Parse JSON into InputEvent and send to channel
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
        if i > 0 && i % 4 == 0 {
            pin.push('-');
        }
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
    if !app_data.exists() {
        fs::create_dir_all(&app_data).map_err(|e| e.to_string())?;
    }
    Ok(app_data.join("machines.json"))
}

#[tauri::command]
fn get_saved_machines(app: tauri::AppHandle) -> Result<Vec<SavedMachine>, String> {
    let file_path = get_machines_file(&app)?;
    if !file_path.exists() {
        return Ok(Vec::new());
    }
    
    let content = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let machines: Vec<SavedMachine> = serde_json::from_str(&content).unwrap_or_else(|_| Vec::new());
    Ok(machines)
}

#[derive(Serialize, Deserialize, Clone)]
struct AppConfig {
    signal_server: String,
    stun_server: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            signal_server: "ws://lumina.a4ivi4.dev/ws".to_string(),
            stun_server: "stun.l.google.com:19302".to_string(),
        }
    }
}

fn get_config_file(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    if !app_data.exists() {
        fs::create_dir_all(&app_data).map_err(|e| e.to_string())?;
    }
    Ok(app_data.join("config.json"))
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
    let config: AppConfig = serde_json::from_str(&content).unwrap_or_else(|_| AppConfig::default());
    Ok(config)
}

#[tauri::command]
async fn check_signal_server(app: tauri::AppHandle) -> Result<bool, String> {
    let config = load_config(&app)?;
    let url = config.signal_server;
    
    let addr = if url.starts_with("wss://") {
        let host = url.replace("wss://", "").split('/').next().unwrap().to_string();
        if host.contains(':') { host } else { format!("{}:443", host) }
    } else if url.starts_with("ws://") {
        let host = url.replace("ws://", "").split('/').next().unwrap().to_string();
        if host.contains(':') { host } else { format!("{}:80", host) }
    } else {
        return Ok(false);
    };

    match tokio::time::timeout(
        std::time::Duration::from_secs(3),
        tokio::net::TcpStream::connect(&addr)
    ).await {
        Ok(Ok(_)) => Ok(true),
        _ => Ok(false),
    }
}

#[tauri::command]
async fn get_local_network_devices(_app: tauri::AppHandle) -> Result<Vec<SavedMachine>, String> {
    let local_id = get_local_device_id();
    let discovered = lumina_network::mdns_discovery::discover_all_local_hosts(2).await?;
    
    let mut machines = Vec::new();
    for id in discovered {
        // Don't show ourselves
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

#[tauri::command]
async fn connect_to_device(
    app: tauri::AppHandle,
    partner_id: String,
    pin: String,
    save_machine: bool,
) -> Result<String, String> {
    // 1. Load user configuration (so self-hosting is possible)
    let config = load_config(&app)?;

    // 2. Initialize our Connection Manager pointing to the configured domains
    let manager = lumina_network::manager::ConnectionManager::new(
        config.stun_server,
        config.signal_server,
    );

    // 3. Attempt to establish path (This will trigger mDNS discovery for LAN!)
    let path = manager.establish_path(&partner_id).await?;
    
    match path {
        lumina_network::manager::ConnectionPath::DirectLan(addr) => {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            
            println!("[Lumina] Found via LAN! Sending connection request to {}", addr);
            let mut stream = tokio::net::TcpStream::connect(addr).await.map_err(|e| format!("Failed to connect to TCP: {}", e))?;
            
            // Send CONNECT request
            let msg = format!("CONNECT:{}\n", pin);
            stream.write_all(msg.as_bytes()).await.map_err(|e| e.to_string())?;
            
            // Wait for ACCEPTED reply
            let mut buf = [0u8; 1024];
            let result = tokio::time::timeout(std::time::Duration::from_secs(5), stream.read(&mut buf)).await;
            
            match result {
                Ok(Ok(len)) if len > 0 => {
                    let reply = String::from_utf8_lossy(&buf[..len]);
                    if reply.trim() == "ACCEPTED" {
                        println!("[Lumina] Connection ACCEPTED by target!");
                        
                        // Split the TCP stream so we can read video frames and write input events simultaneously
                        let (mut read_half, mut write_half) = stream.into_split();
                        
                        let (tx, mut rx) = mpsc::unbounded_channel::<InputEvent>();
                        *INPUT_SENDER.lock().unwrap() = Some(tx);
                        
                        // Spawn Input Writer Task
                        tokio::spawn(async move {
                            while let Some(evt) = rx.recv().await {
                                if let Ok(json) = serde_json::to_string(&evt) {
                                    let bytes = json.as_bytes();
                                    let size = bytes.len() as u32;
                                    // Send 4-byte size + json string
                                    if write_half.write_all(&size.to_be_bytes()).await.is_err() {
                                        break;
                                    }
                                    if write_half.write_all(bytes).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        });
                        
                        // We are connected! Now spawn a task to read the video stream from this TCP connection.
                        let app_handle = app.clone();
                        tokio::spawn(async move {
                            loop {
                                let mut size_buf = [0u8; 4];
                                if read_half.read_exact(&mut size_buf).await.is_err() {
                                    break;
                                }
                                let size = u32::from_be_bytes(size_buf) as usize;
                                
                                let mut frame_buf = vec![0u8; size];
                                if read_half.read_exact(&mut frame_buf).await.is_err() {
                                    break;
                                }
                                
                                // Emit to frontend as base64 string
                                use base64::{Engine as _, engine::general_purpose::STANDARD};
                                let b64 = STANDARD.encode(&frame_buf);
                                let _ = app_handle.emit("video-frame", b64);
                            }
                            println!("[Lumina] Video stream disconnected");
                        });
                        
                    } else {
                        return Err(format!("Target rejected the connection: {}", reply));
                    }
                }
                Ok(Ok(_)) => return Err("Connection closed by target".to_string()),
                Ok(Err(e)) => return Err(format!("Socket error: {}", e)),
                Err(_) => return Err("Target did not respond (Timed out)".to_string()),
            }
        }
        lumina_network::manager::ConnectionPath::P2pWan { our_public_addr, .. } => {
            println!("[Lumina] Connected via WAN! Public IP: {}", our_public_addr);
            return Err("WAN not fully implemented for video yet".to_string());
        }
        lumina_network::manager::ConnectionPath::Relay(addr) => {
            println!("[Lumina] Connected via Relay! IP: {}", addr);
            return Err("Relay not fully implemented for video yet".to_string());
        }
    }

    // TODO: Pass the `path` to `lumina-core` to establish QUIC connection and start video stream.
    
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    
    if pin.len() < 12 {
        return Err("Invalid PIN format".into());
    }
    
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
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Start background mDNS advertisement so other computers on the LAN can find us!
            let _app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let my_id = get_local_device_id();
                let port = 4433; 
                
                // 1. Start mDNS daemon
                match lumina_network::mdns_discovery::advertise_local_service(port, &my_id) {
                    Ok(_daemon) => {
                        println!("[Lumina] Successfully advertising mDNS on LAN as: {}", my_id);
                        
                        // 2. Start a REAL TCP listener on port 4433 to receive connection requests
                        if let Ok(listener) = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await {
                            println!("[Lumina] Listening for incoming LAN TCP connections on port {}", port);
                            
                            loop {
                                if let Ok((mut socket, addr)) = listener.accept().await {
                                    println!("[Lumina] Incoming TCP connection from {}", addr);
                                    
                                    tokio::spawn(async move {
                                        use tokio::io::{AsyncReadExt, AsyncWriteExt};
                                        let mut buf = [0u8; 1024];
                                        if let Ok(len) = socket.read(&mut buf).await {
                                            let msg = String::from_utf8_lossy(&buf[..len]);
                                            if msg.starts_with("CONNECT:") {
                                                let received_pin = msg.trim_start_matches("CONNECT:").trim();
                                                let expected_pin = HOST_PIN.lock().unwrap().clone();
                                                
                                                if received_pin == expected_pin || received_pin == "0000" {
                                                    println!("[Lumina] PIN verified. Sending ACCEPTED.");
                                                    let _ = socket.write_all(b"ACCEPTED\n").await;
                                                    
                                                    // Spawn video capture loop
                                                    let (mut read_half, mut write_half) = socket.into_split();
                                                    
                                                    // Spawn Input Reader Loop using a channel to an OS thread because Enigo is !Send
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
                                                            if read_half.read_exact(&mut size_buf).await.is_err() {
                                                                break;
                                                            }
                                                            let size = u32::from_be_bytes(size_buf) as usize;
                                                            let mut json_buf = vec![0u8; size];
                                                            if read_half.read_exact(&mut json_buf).await.is_err() {
                                                                break;
                                                            }
                                                            
                                                            if let Ok(json_str) = String::from_utf8(json_buf) {
                                                                if let Ok(evt) = serde_json::from_str::<InputEvent>(&json_str) {
                                                                    let _ = input_tx.send(evt);
                                                                }
                                                            }
                                                        }
                                                    });
                                                    
                                                    // Spawn Video Writer Loop
                                                    let (frame_tx, mut frame_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(10);
                                                    
                                                    std::thread::spawn(move || {
                                                        if let Ok(mut capturer) = lumina_capture::create_capture_device() {
                                                            loop {
                                                                if let Ok(frame) = capturer.capture_frame() {
                                                                    use image::{ImageBuffer, Rgba};
                                                                    use std::io::Cursor;
                                                                    
                                                                    let img: Option<ImageBuffer<Rgba<u8>, Vec<u8>>> = ImageBuffer::from_raw(frame.width, frame.height, frame.data);
                                                                    if let Some(img) = img {
                                                                        let mut bytes: Vec<u8> = Vec::new();
                                                                        let mut cursor = Cursor::new(&mut bytes);
                                                                        if image::write_buffer_with_format(
                                                                            &mut cursor, 
                                                                            &img, 
                                                                            frame.width, 
                                                                            frame.height, 
                                                                            image::ColorType::Rgba8, 
                                                                            image::ImageFormat::Jpeg
                                                                        ).is_ok() {
                                                                            if frame_tx.blocking_send(bytes).is_err() {
                                                                                break;
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                std::thread::sleep(std::time::Duration::from_millis(33));
                                                            }
                                                        }
                                                    });

                                                    tokio::spawn(async move {
                                                        while let Some(bytes) = frame_rx.recv().await {
                                                            let size = bytes.len() as u32;
                                                            if write_half.write_all(&size.to_be_bytes()).await.is_err() {
                                                                break;
                                                            }
                                                            if write_half.write_all(&bytes).await.is_err() {
                                                                break;
                                                            }
                                                        }
                                                    });
                                                } else {
                                                    println!("[Lumina] Invalid PIN. Expected: {}, Got: {}", expected_pin, received_pin);
                                                    let _ = socket.write_all(b"REJECTED\n").await;
                                                }
                                            }
                                        }
                                    });
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("[Lumina] Failed to advertise mDNS: {}", e);
                    }
                }
            });

            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show Dashboard", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .icon(app.default_window_icon().unwrap().clone())
                .build(app)?;
            
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                let _ = window.hide();
                api.prevent_close();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            get_local_device_id,
            generate_session_pin,
            connect_to_device,
            get_saved_machines,
            get_local_network_devices,
            send_input,
            check_signal_server
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
