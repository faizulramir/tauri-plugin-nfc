//! # Tauri NFC Plugin
//!
//! A Tauri v2 plugin for NFC card reading using PC/SC (Personal Computer/Smart Card) interface.
//! This plugin provides cross-platform NFC tag reading capabilities for desktop applications.
//!
//! ## Features
//!
//! - Cross-platform NFC reader support via PC/SC
//! - Real-time tag detection and monitoring
//! - Multiple reader support
//! - Async/await support
//! - Type-safe Rust API with Serde serialization
//!
//! ## Usage
//!
//! Add this plugin to your Tauri application:
//!
//! ```rust
//! use tauri_plugin_nfc::NfcPlugin;
//!
//! tauri::Builder::default()
//!     .plugin(NfcPlugin::init())
//!     .run(tauri::generate_context!())
//!     .expect("error while running tauri application");
//! ```

pub mod commands;
mod error;
mod nfc;
mod plugin;

pub use error::{Error, Result};
pub use nfc::{NfcReader, NfcTag, ReaderInfo};
pub use plugin::NfcPlugin;

// Re-export commonly used types
pub use commands::{
    NfcStatus, ReaderStatus, TagDetectionEvent
};