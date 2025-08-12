//! Core NFC functionality using PC/SC

use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use hex;
use log::{debug, error, info, warn};
use pcsc::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

/// Represents an NFC tag that was detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NfcTag {
    /// Unique identifier of the tag (hex string)
    pub uid: String,
    /// Timestamp when the tag was detected
    pub timestamp: DateTime<Utc>,
    /// Name of the reader that detected the tag
    pub reader_name: String,
    /// Raw UID bytes (for advanced usage)
    pub uid_bytes: Vec<u8>,
    /// Additional tag information if available
    pub tag_type: Option<String>,
}

/// Information about an NFC reader
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReaderInfo {
    /// Reader name
    pub name: String,
    /// Whether the reader is currently connected
    pub connected: bool,
    /// Current card status
    pub card_present: bool,
    /// Last error if any
    pub last_error: Option<String>,
}

/// NFC Reader implementation using PC/SC
#[derive(Clone)]
pub struct NfcReader {
    context: Arc<Mutex<Context>>,
    pub(crate) readers: Vec<String>,
    is_running: Arc<Mutex<bool>>,
    reader_states: Arc<Mutex<HashMap<String, ReaderInfo>>>,
}

impl std::fmt::Debug for NfcReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NfcReader")
            .field("readers", &self.readers)
            .field("is_running", &self.is_running)
            .finish()
    }
}

impl NfcReader {
    /// Create a new NFC reader instance
    pub fn new() -> Result<Self> {
        let context = Context::establish(Scope::User)
            .map_err(|e| crate::Error::ContextError {
                message: e.to_string(),
            })?;

        Ok(NfcReader {
            context: Arc::new(Mutex::new(context)),
            readers: Vec::new(),
            is_running: Arc::new(Mutex::new(false)),
            reader_states: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// List all available NFC readers
    pub fn list_readers(&mut self) -> Result<Vec<String>> {
        let context = self.context.lock().unwrap();
        let readers_buf = context
            .list_readers_owned()
            .map_err(|e| crate::Error::Generic {
                message: format!("Failed to list readers: {}", e),
            })?;

        self.readers = readers_buf
            .into_iter()
            .map(|cstring| cstring.to_string_lossy().to_string())
            .collect();

        // Initialize reader states
        let mut states = self.reader_states.lock().unwrap();
        states.clear();
        for reader in &self.readers {
            states.insert(
                reader.clone(),
                ReaderInfo {
                    name: reader.clone(),
                    connected: false,
                    card_present: false,
                    last_error: None,
                },
            );
        }

        info!("Found {} NFC readers", self.readers.len());
        for (i, reader) in self.readers.iter().enumerate() {
            info!("Reader {}: {}", i, reader);
        }

        Ok(self.readers.clone())
    }

    /// Start monitoring for NFC tags
    pub fn start_monitoring(&self) -> Result<()> {
        let mut is_running = self.is_running.lock().unwrap();
        if *is_running {
            return Err(crate::Error::AlreadyMonitoring);
        }
        *is_running = true;
        info!("Started NFC monitoring");
        Ok(())
    }

    /// Stop monitoring for NFC tags
    pub fn stop_monitoring(&self) -> Result<()> {
        let mut is_running = self.is_running.lock().unwrap();
        if !*is_running {
            return Err(crate::Error::NotMonitoring);
        }
        *is_running = false;
        info!("Stopped NFC monitoring");
        Ok(())
    }

    /// Check if monitoring is active
    pub fn is_monitoring(&self) -> bool {
        *self.is_running.lock().unwrap()
    }

    /// Get current reader states
    pub fn get_reader_states(&self) -> HashMap<String, ReaderInfo> {
        self.reader_states.lock().unwrap().clone()
    }

    /// Read a tag from a specific reader
    pub fn read_tag_sync(&self, reader_name: &str) -> Result<Option<NfcTag>> {
        if !self.is_monitoring() {
            return Ok(None);
        }

        // Check if reader exists
        if !self.readers.contains(&reader_name.to_string()) {
            return Err(crate::Error::ReaderNotFound {
                reader: reader_name.to_string(),
            });
        }

        let context = self.context.lock().unwrap();

        let reader_cstr = std::ffi::CString::new(reader_name)
            .map_err(|e| crate::Error::InvalidResponse { message: format!("Invalid reader name: {}", e) })?;
        match context.connect(&reader_cstr, ShareMode::Shared, Protocols::ANY) {
            Ok(card) => {
                // Update reader state
                {
                    let mut states = self.reader_states.lock().unwrap();
                    if let Some(state) = states.get_mut(reader_name) {
                        state.connected = true;
                        state.card_present = true;
                        state.last_error = None;
                    }
                }

                // Get card UID
                match self.get_card_uid(&card) {
                    Ok(uid_bytes) => {
                        let uid = hex::encode(&uid_bytes).to_uppercase();
                        debug!("Read UID: {} from reader: {}", uid, reader_name);
                        
                        Ok(Some(NfcTag {
                            uid,
                            timestamp: Utc::now(),
                            reader_name: reader_name.to_string(),
                            uid_bytes,
                            tag_type: None, // Could be enhanced to detect tag type
                        }))
                    }
                    Err(e) => {
                        // Update reader state with error
                        {
                            let mut states = self.reader_states.lock().unwrap();
                            if let Some(state) = states.get_mut(reader_name) {
                                state.last_error = Some(e.to_string());
                            }
                        }
                        Err(e)
                    }
                }
            }
            Err(pcsc::Error::NoSmartcard) => {
                // No card present - update state
                {
                    let mut states = self.reader_states.lock().unwrap();
                    if let Some(state) = states.get_mut(reader_name) {
                        state.connected = true;
                        state.card_present = false;
                        state.last_error = None;
                    }
                }
                Ok(None)
            }
            Err(e) => {
                // Connection error - update state
                {
                    let mut states = self.reader_states.lock().unwrap();
                    if let Some(state) = states.get_mut(reader_name) {
                        state.connected = false;
                        state.card_present = false;
                        state.last_error = Some(e.to_string());
                    }
                }
                Err(crate::Error::ReaderConnectionError {
                    reader: reader_name.to_string(),
                    message: e.to_string(),
                })
            }
        }
    }

    /// Monitor all readers for tags
    pub fn monitor_all_readers(&self) -> Result<Vec<NfcTag>> {
        let mut tags = Vec::new();
        
        for reader_name in &self.readers {
            if let Ok(Some(tag)) = self.read_tag_sync(reader_name) {
                tags.push(tag);
            }
        }
        
        Ok(tags)
    }

    /// Get the UID from a connected card
    fn get_card_uid(&self, card: &Card) -> Result<Vec<u8>> {
        // ISO 14443-4 Get UID command
        let get_uid_command = [0xFF, 0xCA, 0x00, 0x00, 0x00];
        
        let mut response_buffer = [0; MAX_BUFFER_SIZE];
        
        match card.transmit(&get_uid_command, &mut response_buffer) {
            Ok(response) => {
                if response.len() >= 2 {
                    let status = &response[response.len() - 2..];
                    if status == [0x90, 0x00] {
                        // Success - return UID (excluding status bytes)
                        Ok(response[..response.len() - 2].to_vec())
                    } else {
                        Err(crate::Error::InvalidResponse {
                            message: format!("Card returned error status: {:02X}{:02X}", status[0], status[1]),
                        })
                    }
                } else {
                    Err(crate::Error::InvalidResponse {
                        message: "Response too short".to_string(),
                    })
                }
            }
            Err(e) => Err(crate::Error::CardError {
                message: e.to_string(),
            }),
        }
    }
}

impl Default for NfcReader {
    fn default() -> Self {
        Self::new().expect("Failed to create default NFC reader")
    }
}