//! Tauri NFC Plugin implementation

use crate::commands::NfcReaderState;
use std::sync::{Arc, Mutex};
use tauri::{plugin::PluginApi, AppHandle, Runtime, Emitter, Manager};

/// NFC Plugin for Tauri v2
pub struct NfcPlugin {
    invoke_handler: Box<dyn Fn(tauri::ipc::Invoke) + Send + Sync>,
}

impl NfcPlugin {
    /// Initialize the NFC plugin
    pub fn init<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
        tauri::plugin::Builder::new("nfc")
            .invoke_handler(tauri::generate_handler![
                crate::commands::nfc_init,
                crate::commands::nfc_get_status,
                crate::commands::nfc_start_monitoring,
                crate::commands::nfc_stop_monitoring,
                crate::commands::nfc_read_tags,
                crate::commands::nfc_read_tag_from_reader,
                crate::commands::nfc_get_reader_info,
                crate::commands::nfc_list_readers,
                crate::commands::nfc_is_monitoring,
            ])
            .setup(|app, _api| {
                // Initialize the NFC reader state
                let nfc_state: NfcReaderState = Arc::new(Mutex::new(None));
                app.manage(nfc_state);
                Ok(())
            })
            .build()
    }
}

/// Plugin API for advanced usage
pub struct NfcApi<R: Runtime> {
    app: AppHandle<R>,
}

impl<R: Runtime> NfcApi<R> {
    /// Create a new NFC API instance
    pub fn new(app: AppHandle<R>) -> Self {
        Self { app }
    }

    /// Get the NFC reader state
    pub fn get_nfc_state(&self) -> Option<NfcReaderState> {
        self.app.try_state::<NfcReaderState>().map(|state| state.inner().clone())
    }

    /// Emit a tag detection event to the frontend
    pub fn emit_tag_detected(&self, tag: &crate::nfc::NfcTag) -> tauri::Result<()> {
        self.app.emit("nfc://tag-detected", tag)
    }

    /// Emit a reader status change event
    pub fn emit_reader_status_changed(
        &self,
        reader_info: &crate::nfc::ReaderInfo,
    ) -> tauri::Result<()> {
        self.app.emit("nfc://reader-status-changed", reader_info)
    }

    /// Emit a monitoring status change event
    pub fn emit_monitoring_status_changed(&self, is_monitoring: bool) -> tauri::Result<()> {
        self.app.emit("nfc://monitoring-status-changed", is_monitoring)
    }
}

/// Extension trait for AppHandle to easily access NFC API
pub trait NfcExt<R: Runtime> {
    /// Get the NFC API
    fn nfc(&self) -> NfcApi<R>;
}

impl<R: Runtime> NfcExt<R> for AppHandle<R> {
    fn nfc(&self) -> NfcApi<R> {
        NfcApi::new(self.clone())
    }
}

/// Convenience macros for common operations
#[macro_export]
macro_rules! nfc_emit_tag {
    ($app:expr, $tag:expr) => {
        $app.nfc().emit_tag_detected($tag)
    };
}

#[macro_export]
macro_rules! nfc_emit_reader_status {
    ($app:expr, $reader_info:expr) => {
        $app.nfc().emit_reader_status_changed($reader_info)
    };
}

#[macro_export]
macro_rules! nfc_emit_monitoring_status {
    ($app:expr, $is_monitoring:expr) => {
        $app.nfc().emit_monitoring_status_changed($is_monitoring)
    };
}