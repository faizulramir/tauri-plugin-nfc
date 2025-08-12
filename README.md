# Tauri NFC Plugin

A Tauri v2 plugin for NFC card reading using PC/SC (Personal Computer/Smart Card) interface. This plugin provides cross-platform NFC tag reading capabilities for desktop applications.

## Features

- 🔌 **Cross-platform NFC reader support** via PC/SC
- 🏃 **Real-time tag detection and monitoring**
- 📡 **Multiple reader support**
- ⚡ **Async/await support**
- 🔒 **Type-safe Rust API** with Serde serialization
- 🎯 **Event-driven architecture** with frontend notifications
- 🛠️ **Easy integration** with existing Tauri v2 applications

## Prerequisites

- **Windows**: PC/SC is built into Windows
- **macOS**: PC/SC framework is included
- **Linux**: Install `pcscd` and `libpcsclite-dev`

```bash
# Ubuntu/Debian
sudo apt-get install pcscd libpcsclite-dev

# Fedora/RHEL
sudo dnf install pcsc-lite-devel

# Arch Linux
sudo pacman -S pcsclite
```

## Installation

Add this plugin to your Tauri project:

```toml
[dependencies]
tauri-plugin-nfc = { path = "../tauri-plugin-nfc" }
```

## Usage

### 1. Register the Plugin

In your `src-tauri/src/lib.rs` or `src-tauri/src/main.rs`:

```rust
use tauri_plugin_nfc::NfcPlugin;

fn main() {
    tauri::Builder::default()
        .plugin(NfcPlugin::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 2. Frontend Usage

#### Initialize NFC System

```javascript
import { invoke } from '@tauri-apps/api/core';

// Initialize the NFC system
try {
    const result = await invoke('plugin:nfc|nfc_init');
    console.log(result); // "NFC system initialized with X readers"
} catch (error) {
    console.error('Failed to initialize NFC:', error);
}
```

#### Start Monitoring

```javascript
// Start monitoring for NFC tags
try {
    await invoke('plugin:nfc|nfc_start_monitoring');
    console.log('NFC monitoring started');
} catch (error) {
    console.error('Failed to start monitoring:', error);
}
```

#### Read Tags

```javascript
// Read tags from all readers
try {
    const tags = await invoke('plugin:nfc|nfc_read_tags');
    tags.forEach(tag => {
        console.log(`Tag UID: ${tag.uid}`);
        console.log(`Reader: ${tag.reader_name}`);
        console.log(`Timestamp: ${tag.timestamp}`);
    });
} catch (error) {
    console.error('Failed to read tags:', error);
}
```

#### Get System Status

```javascript
// Get NFC system status
try {
    const status = await invoke('plugin:nfc|nfc_get_status');
    console.log(`Monitoring: ${status.is_monitoring}`);
    console.log(`Readers: ${status.readers_count}`);
    console.log('Available readers:', status.readers);
} catch (error) {
    console.error('Failed to get status:', error);
}
```

### 3. Event Listening

Listen for NFC events in your frontend:

```javascript
import { listen } from '@tauri-apps/api/event';

// Listen for tag detection events
const unlisten = await listen('nfc://tag-detected', (event) => {
    const tag = event.payload;
    console.log('Tag detected:', tag.uid);
});

// Listen for reader status changes
const unlistenReader = await listen('nfc://reader-status-changed', (event) => {
    const readerInfo = event.payload;
    console.log('Reader status changed:', readerInfo);
});

// Listen for monitoring status changes
const unlistenMonitoring = await listen('nfc://monitoring-status-changed', (event) => {
    const isMonitoring = event.payload;
    console.log('Monitoring status:', isMonitoring);
});
```

### 4. Advanced Usage with Rust API

For advanced usage in your Rust backend:

```rust
use tauri_plugin_nfc::{NfcExt, nfc_emit_tag};
use tauri::Manager;

#[tauri::command]
async fn my_custom_nfc_handler(app: tauri::AppHandle) -> Result<(), String> {
    // Access the NFC API
    let nfc_api = app.nfc();
    
    // Get NFC state
    if let Some(nfc_state) = nfc_api.get_nfc_state() {
        // Custom logic here
    }
    
    Ok(())
}
```

## API Reference

### Commands

| Command | Description | Parameters | Returns |
|---------|-------------|------------|----------|
| `nfc_init` | Initialize NFC system | None | `String` |
| `nfc_get_status` | Get system status | None | `NfcStatus` |
| `nfc_start_monitoring` | Start monitoring | None | `String` |
| `nfc_stop_monitoring` | Stop monitoring | None | `String` |
| `nfc_read_tags` | Read from all readers | None | `Vec<NfcTag>` |
| `nfc_read_tag_from_reader` | Read from specific reader | `reader_name: String` | `Option<NfcTag>` |
| `nfc_list_readers` | List available readers | None | `Vec<String>` |
| `nfc_is_monitoring` | Check monitoring status | None | `bool` |

### Types

#### NfcTag
```rust
pub struct NfcTag {
    pub uid: String,              // Hex-encoded UID
    pub timestamp: DateTime<Utc>, // Detection timestamp
    pub reader_name: String,      // Reader that detected the tag
    pub uid_bytes: Vec<u8>,       // Raw UID bytes
    pub tag_type: Option<String>, // Tag type (if detected)
}
```

#### NfcStatus
```rust
pub struct NfcStatus {
    pub is_monitoring: bool,                        // Monitoring active
    pub readers_count: usize,                       // Number of readers
    pub readers: Vec<String>,                       // Reader names
    pub reader_states: HashMap<String, ReaderInfo>, // Detailed states
}
```

#### ReaderInfo
```rust
pub struct ReaderInfo {
    pub name: String,                // Reader name
    pub connected: bool,             // Connection status
    pub card_present: bool,          // Card presence
    pub last_error: Option<String>,  // Last error message
}
```

### Events

- `nfc://tag-detected` - Emitted when a tag is detected
- `nfc://reader-status-changed` - Emitted when reader status changes
- `nfc://monitoring-status-changed` - Emitted when monitoring status changes

## Error Handling

The plugin uses a comprehensive error system:

```javascript
try {
    await invoke('plugin:nfc|nfc_init');
} catch (error) {
    switch (error.type) {
        case 'ContextError':
            console.error('PC/SC context error:', error.message);
            break;
        case 'NoReadersFound':
            console.error('No NFC readers found');
            break;
        case 'ReaderConnectionError':
            console.error('Reader connection failed:', error.message);
            break;
        default:
            console.error('Unknown error:', error);
    }
}
```

## Troubleshooting

### Common Issues

1. **No readers found**
   - Ensure NFC reader is connected and drivers are installed
   - Check if PC/SC service is running
   - Verify reader compatibility with PC/SC

2. **Permission denied**
   - On Linux, ensure user is in `pcscd` group
   - Check if PC/SC daemon is running: `sudo systemctl status pcscd`

3. **Reader connection failed**
   - Try disconnecting and reconnecting the reader
   - Check for conflicting applications using the reader
   - Restart PC/SC service

### Debug Mode

Enable debug logging:

```bash
RUST_LOG=debug cargo tauri dev
```

## License

MIT OR Apache-2.0

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.