use tauri::Emitter;

#[tauri::command]
fn get_local_device_id() -> String {
    // In a real scenario, this is derived from a persistently stored private key.
    // For now, we return a mock ID.
    "LMN-8X92-QW10".to_string()
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

#[tauri::command]
async fn connect_to_device(
    app: tauri::AppHandle,
    partner_id: String,
    pin: String,
    save_machine: bool,
) -> Result<String, String> {
    tokio::time::sleep(std::time::Duration::from_millis(800)).await;
    
    if pin.len() < 12 {
        return Err("Invalid PIN format".into());
    }
    
    if save_machine {
        let file_path = get_machines_file(&app)?;
        let mut machines = get_saved_machines(app.clone()).unwrap_or_default();
        
        // Remove if exists
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
            get_saved_machines
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
