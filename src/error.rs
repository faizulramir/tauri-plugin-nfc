//! Error types for the NFC plugin

use serde::{Deserialize, Serialize};
use std::fmt;

/// Result type alias for NFC operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types that can occur during NFC operations
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum Error {
    /// PC/SC context establishment failed
    #[error("Failed to establish PC/SC context: {message}")]
    ContextError { message: String },

    /// No NFC readers found
    #[error("No NFC readers found")]
    NoReadersFound,

    /// Reader connection failed
    #[error("Failed to connect to reader '{reader}': {message}")]
    ReaderConnectionError { reader: String, message: String },

    /// Card communication error
    #[error("Card communication error: {message}")]
    CardError { message: String },

    /// Invalid card response
    #[error("Invalid card response: {message}")]
    InvalidResponse { message: String },

    /// Reader not found
    #[error("Reader '{reader}' not found")]
    ReaderNotFound { reader: String },

    /// Operation timeout
    #[error("Operation timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    /// Plugin not initialized
    #[error("NFC plugin not initialized")]
    NotInitialized,

    /// Already monitoring
    #[error("NFC monitoring is already active")]
    AlreadyMonitoring,

    /// Not monitoring
    #[error("NFC monitoring is not active")]
    NotMonitoring,

    /// Serialization error
    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    /// Generic error
    #[error("NFC error: {message}")]
    Generic { message: String },
}

impl From<pcsc::Error> for Error {
    fn from(err: pcsc::Error) -> Self {
        match err {
            pcsc::Error::NoReadersAvailable => Error::NoReadersFound,
            _ => Error::Generic {
                message: err.to_string(),
            },
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerializationError {
            message: err.to_string(),
        }
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Generic {
            message: err.to_string(),
        }
    }
}