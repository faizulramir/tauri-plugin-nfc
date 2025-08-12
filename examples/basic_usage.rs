//! Basic usage example for the Tauri NFC Plugin
//!
//! This example demonstrates how to integrate the NFC plugin into a Tauri application
//! and use its basic functionality.

use tauri_plugin_nfc::{NfcPlugin, NfcExt};
use tauri::Manager;

#[tauri::command]
async fn custom_nfc_handler(app: tauri::AppHandle) -> Result<String, String> {
    // Access the NFC API through the extension trait
    let nfc_api = app.nfc();
    
    // Example: Get NFC state and emit a custom event
    if let Some(_nfc_state) = nfc_api.get_nfc_state() {
        // You can perform custom operations here
        println!("NFC system is available");
        
        // Emit a custom event
        if let Err(e) = app.emit("custom-nfc-event", "NFC system accessed") {
            return Err(format!("Failed to emit event: {}", e));
        }
        
        Ok("Custom NFC handler executed successfully".to_string())
    } else {
        Err("NFC system not initialized".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Register the NFC plugin
        .plugin(NfcPlugin::init())
        // Add custom commands
        .invoke_handler(tauri::generate_handler![custom_nfc_handler])
        .setup(|app| {
            // Optional: Setup event listeners or perform initialization
            println!("Application setup complete with NFC plugin");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}