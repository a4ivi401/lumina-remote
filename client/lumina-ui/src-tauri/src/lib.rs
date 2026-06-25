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
            signal_server: "wss://lumina.a4ivi4.dev/ws".to_string(),
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
async fn get_local_network_devices(app: tauri::AppHandle) -> Result<Vec<SavedMachine>, String> {
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
            println!("[Lumina] Found via LAN! Sending connection request to {}", addr);
            let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await.map_err(|e| e.to_string())?;
            
            // Send CONNECT request
            let my_id = get_local_device_id();
            let msg = format!("CONNECT:{}", my_id);
            socket.send_to(msg.as_bytes(), addr).await.map_err(|e| e.to_string())?;
            
            // Wait for ACCEPTED reply (Timeout 5 seconds)
            let mut buf = [0u8; 1024];
            let result = tokio::time::timeout(std::time::Duration::from_secs(5), socket.recv_from(&mut buf)).await;
            
            match result {
                Ok(Ok((len, _))) => {
                    let reply = String::from_utf8_lossy(&buf[..len]);
                    if reply == "ACCEPTED" {
                        println!("[Lumina] Connection ACCEPTED by target!");
                    } else {
                        return Err("Target rejected the connection".to_string());
                    }
                }
                Ok(Err(e)) => return Err(format!("Socket error: {}", e)),
                Err(_) => return Err("Target did not respond (Timed out)".to_string()),
            }
        }
        lumina_network::manager::ConnectionPath::P2pWan { our_public_addr, .. } => {
            println!("[Lumina] Connected via WAN! Public IP: {}", our_public_addr);
        }
        lumina_network::manager::ConnectionPath::Relay(addr) => {
            println!("[Lumina] Connected via Relay! IP: {}", addr);
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
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let my_id = get_local_device_id();
                let port = 4433; 
                
                // 1. Start mDNS daemon
                match lumina_network::mdns_discovery::advertise_local_service(port, &my_id) {
                    Ok(_daemon) => {
                        println!("[Lumina] Successfully advertising mDNS on LAN as: {}", my_id);
                        
                        // 2. Start a REAL UDP listener on port 4433 to receive connection requests
                        if let Ok(socket) = tokio::net::UdpSocket::bind(format!("0.0.0.0:{}", port)).await {
                            println!("[Lumina] Listening for incoming LAN connections on UDP port {}", port);
                            let mut buf = [0u8; 1024];
                            loop {
                                if let Ok((len, addr)) = socket.recv_from(&mut buf).await {
                                    let msg = String::from_utf8_lossy(&buf[..len]);
                                    if msg.starts_with("CONNECT:") {
                                        let partner_id = msg.trim_start_matches("CONNECT:");
                                        println!("[Lumina] Incoming connection request from {} ({})", partner_id, addr);
                                        // Emit event to frontend to show "Accept/Reject" popup
                                        let _ = app_handle.emit("incoming-connection", partner_id);
                                        
                                        // Auto-reply ACCEPTED for the Alpha test
                                        // In Beta, this will wait for the user to click "Accept" in the UI.
                                        let _ = socket.send_to(b"ACCEPTED", addr).await;
                                    }
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
            get_local_network_devices
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
