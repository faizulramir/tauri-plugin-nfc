//! Tauri commands for NFC operations

use crate::error::Error;
use crate::nfc::{NfcReader, NfcTag, ReaderInfo};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{command, State, AppHandle, Runtime};

/// Global NFC reader state
pub type NfcReaderState = Arc<Mutex<Option<NfcReader>>>;

/// NFC system status
#[derive(Debug, Serialize, Deserialize)]
pub struct NfcStatus {
    /// Whether monitoring is active
    pub is_monitoring: bool,
    /// Number of available readers
    pub readers_count: usize,
    /// List of reader names
    pub readers: Vec<String>,
    /// Detailed reader information
    pub reader_states: HashMap<String, ReaderInfo>,
}

/// Reader status information
#[derive(Debug, Serialize, Deserialize)]
pub struct ReaderStatus {
    /// Reader name
    pub name: String,
    /// Connection status
    pub connected: bool,
    /// Card presence
    pub card_present: bool,
    /// Last error message
    pub last_error: Option<String>,
}

/// Tag detection event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagDetectionEvent {
    /// Detected tag
    pub tag: NfcTag,
    /// Event type
    pub event_type: String,
}

/// Initialize the NFC system
#[command]
pub async fn nfc_init(nfc_state: State<'_, NfcReaderState>) -> Result<String, String> {
    info!("Initializing NFC system...");

    match NfcReader::new() {
        Ok(mut reader) => {
            // List available readers
            match reader.list_readers() {
                Ok(readers) => {
                    let reader_count = readers.len();

                    // Store the reader instance globally
                    {
                        let mut global_reader = nfc_state.lock().unwrap();
                        *global_reader = Some(reader);
                    }

                    info!(
                        "NFC system initialized successfully with {} readers",
                        reader_count
                    );
                    Ok(format!(
                        "NFC system initialized with {} readers",
                        reader_count
                    ))
                }
                Err(e) => {
                    error!("Failed to list NFC readers: {}", e);
                    Err(format!("Failed to list NFC readers: {}", e))
                }
            }
        }
        Err(e) => {
            error!("Failed to initialize NFC reader: {}", e);
            Err(format!("Failed to initialize NFC reader: {}", e))
        }
    }
}

/// Get NFC system status
#[command]
pub async fn nfc_get_status(nfc_state: State<'_, NfcReaderState>) -> Result<NfcStatus, String> {
    let reader_guard = nfc_state.lock().unwrap();

    match reader_guard.as_ref() {
        Some(reader) => Ok(NfcStatus {
            is_monitoring: reader.is_monitoring(),
            readers_count: reader.readers.len(),
            readers: reader.readers.clone(),
            reader_states: reader.get_reader_states(),
        }),
        None => Err("NFC system not initialized".to_string()),
    }
}

/// Start NFC monitoring
#[command]
pub async fn nfc_start_monitoring(nfc_state: State<'_, NfcReaderState>) -> Result<String, String> {
    let reader_guard = nfc_state.lock().unwrap();

    match reader_guard.as_ref() {
        Some(reader) => match reader.start_monitoring() {
            Ok(_) => {
                info!("NFC monitoring started");
                Ok("NFC monitoring started".to_string())
            }
            Err(e) => {
                error!("Failed to start NFC monitoring: {}", e);
                Err(format!("Failed to start monitoring: {}", e))
            }
        },
        None => Err("NFC system not initialized".to_string()),
    }
}

/// Stop NFC monitoring
#[command]
pub async fn nfc_stop_monitoring(nfc_state: State<'_, NfcReaderState>) -> Result<String, String> {
    let reader_guard = nfc_state.lock().unwrap();

    match reader_guard.as_ref() {
        Some(reader) => match reader.stop_monitoring() {
            Ok(_) => {
                info!("NFC monitoring stopped");
                Ok("NFC monitoring stopped".to_string())
            }
            Err(e) => {
                error!("Failed to stop NFC monitoring: {}", e);
                Err(format!("Failed to stop monitoring: {}", e))
            }
        },
        None => Err("NFC system not initialized".to_string()),
    }
}

/// Read NFC tags from all readers
#[command]
pub fn nfc_read_tags(nfc_state: State<'_, NfcReaderState>) -> Result<Vec<NfcTag>, String> {
    // Clone the reader to avoid holding the lock across await
    let reader_clone = {
        let reader_guard = nfc_state.lock().unwrap();

        match reader_guard.as_ref() {
            Some(reader) => reader.clone(),
            None => return Err("NFC system not initialized".to_string()),
        }
    };

    match reader_clone.monitor_all_readers() {
        Ok(tags) => Ok(tags),
        Err(e) => {
            error!("Failed to read NFC tags: {}", e);
            Err(format!("Failed to read NFC tags: {}", e))
        }
    }
}

/// Read NFC tag from a specific reader
#[command]
pub fn nfc_read_tag_from_reader(
    reader_name: String,
    nfc_state: State<'_, NfcReaderState>,
) -> Result<Option<NfcTag>, String> {
    // Clone the reader to avoid holding the lock across await
    let reader_clone = {
        let reader_guard = nfc_state.lock().unwrap();

        match reader_guard.as_ref() {
            Some(reader) => reader.clone(),
            None => return Err("NFC system not initialized".to_string()),
        }
    };

    match reader_clone.read_tag_sync(&reader_name) {
        Ok(tag) => Ok(tag),
        Err(e) => {
            error!("Failed to read NFC tag from reader {}: {}", reader_name, e);
            Err(format!("Failed to read NFC tag from reader {}: {}", reader_name, e))
        }
    }
}

/// Get detailed reader information
#[command]
pub async fn nfc_get_reader_info(
    reader_name: String,
    nfc_state: State<'_, NfcReaderState>,
) -> Result<ReaderInfo, String> {
    let reader_guard = nfc_state.lock().unwrap();

    match reader_guard.as_ref() {
        Some(reader) => {
            let states = reader.get_reader_states();
            match states.get(&reader_name) {
                Some(info) => Ok(info.clone()),
                None => Err(format!("Reader not found: {}", reader_name)),
            }
        }
        None => Err("NFC system not initialized".to_string()),
    }
}

/// List all available readers
#[command]
pub async fn nfc_list_readers(nfc_state: State<'_, NfcReaderState>) -> Result<Vec<String>, String> {
    let mut reader_guard = nfc_state.lock().unwrap();

    match reader_guard.as_mut() {
        Some(reader) => match reader.list_readers() {
            Ok(readers) => Ok(readers),
            Err(e) => {
                error!("Failed to list readers: {}", e);
                Err(format!("Failed to list readers: {}", e))
            }
        },
        None => Err("NFC system not initialized".to_string()),
    }
}

/// Check if monitoring is active
#[command]
pub async fn nfc_is_monitoring(nfc_state: State<'_, NfcReaderState>) -> Result<bool, String> {
    let reader_guard = nfc_state.lock().unwrap();

    match reader_guard.as_ref() {
        Some(reader) => Ok(reader.is_monitoring()),
        None => Err("NFC system not initialized".to_string()),
    }
}