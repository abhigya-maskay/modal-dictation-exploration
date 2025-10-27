# Phonesc

Modal voice dictation system for Linux with Wayland support.

## Overview

Phonesc is a voice-driven input system that combines wake-word detection, voice activity detection, and speech recognition to provide hands-free control and dictation capabilities. Built for Linux with PipeWire/PulseAudio audio backends and Wayland compositor integration.

## Features

- Wake word detection ("Quasar") using Porcupine
- Voice activity detection with Silero VAD
- Command recognition with closed-grammar ASR (whisper.cpp)
- Dictation via external faster-whisper service
- Wayland virtual keyboard integration
- Visual overlay indicator (Awake/Asleep/Error states)
- **Live configuration reload** - Edit your config file and see changes take effect immediately without restarting

## Development Environment

### Prerequisites

- [Nix](https://nixos.org/download.html) with flakes enabled
- Git

To enable flakes, add the following to your `~/.config/nix/nix.conf`:

```
experimental-features = nix-command flakes
```

### Getting Started

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd modal-dictation-exploration
   ```

2. Enter the development shell:
   ```bash
   nix develop
   ```

   This will provision a complete development environment with:
   - Rust stable toolchain (cargo, rustfmt, clippy, rust-analyzer)
   - Build tools (pkg-config, cmake, ninja, clang, gcc)
   - Audio dependencies (PipeWire, PulseAudio, ALSA, JACK)
   - Wayland libraries (wayland, wayland-protocols, libxkbcommon)
   - ML/ASR dependencies (ONNX Runtime)
   - Additional libraries (OpenSSL, SQLite)

3. Build the project:
   ```bash
   cargo build
   ```

4. Run the project:
   ```bash
   cargo run
   ```

### Without Nix

If you prefer not to use Nix, ensure you have the following dependencies installed on your system:

- Rust stable toolchain
- pkg-config, cmake, ninja, clang
- PipeWire or PulseAudio
- ALSA development libraries
- Wayland development libraries
- ONNX Runtime
- OpenSSL development libraries

Note: The Nix flake is the recommended and tested development environment.

## Architecture

See the [docs](./docs) directory for detailed architecture documentation:

- [Product Requirements](./docs/PRODUCT_REQUIREMENTS.md)
- [Technology Selection](./docs/TECHNOLOGY_SELECTION.md)
- [Technical Decisions](./docs/TECHNICAL_DECISIONS.md)

## Configuration

### Configuration File

Phonesc reads its configuration from `~/.config/phonesc/config.toml`. If the file doesn't exist, default values are used.

Example configuration:

```toml
auto_sleep_timeout_secs = 300
command_pause_threshold_ms = 700
dictation_pause_threshold_ms = 900
enable_activation_demo = false
activation_demo_interval_secs = 10

[overlay]
awake_color = "green"
asleep_color = "gray"
error_color = "red"
position = "top-right"

[dictation_service]
host = "127.0.0.1"
port = 5123
```

### Live Configuration Reload

Phonesc automatically detects changes to your configuration file and reloads them without requiring a restart. This enables you to:

- Adjust timing parameters on the fly
- Change overlay appearance in real-time
- Update service endpoints without interruption
- Test different configurations quickly during development

**How it works:**

1. The application watches `~/.config/phonesc/config.toml` for changes
2. When you save edits to the file, changes are detected automatically
3. After a 500ms debounce period (to handle editor write patterns), the config is reloaded
4. All active components receive the updated configuration
5. Changes take effect immediately

**Triggering a reload:**

Simply edit and save the config file:

```bash
# Edit the configuration
nano ~/.config/phonesc/config.toml

# Or use your preferred editor
vim ~/.config/phonesc/config.toml
```

The application will log the reload:

```
INFO phonesc::config::manager: Config file change detected, starting debounce timer
INFO phonesc::config::manager: Debounce period elapsed, reloading config
INFO phonesc::config::manager: Config reloaded successfully and broadcast to subscribers
INFO phonesc: Config updated!
```

**Error handling:**

- If the config file contains invalid TOML syntax, the error is logged and the last valid configuration remains active
- The application continues to watch for new changes, so you can fix the error and save again
- Parse errors are logged at ERROR level with details about what went wrong

**Notes:**

- The 500ms debounce period prevents excessive reloads when editors write files in multiple steps
- The config directory is watched (not just the file) to handle atomic writes from editors like vim and nano
- All subscribers receive updates simultaneously through an efficient broadcast channel

### Activation Demo Mode

For development and testing purposes, you can enable automatic activation cycling to visually verify the overlay indicator works correctly:

```toml
enable_activation_demo = true
activation_demo_interval_secs = 10
```

**Parameters:**

- `enable_activation_demo` (default: `false`) - Enables automatic wake word triggering for testing
- `activation_demo_interval_secs` (default: `10`) - How often (in seconds) to automatically trigger wake word

**When enabled:**

The system will automatically wake every N seconds as if a wake word was detected, then return to sleep after the configured `auto_sleep_timeout_secs`. This allows you to observe the overlay indicator cycling through Awake (green) and Asleep (gray) states without needing to trigger actual wake word detection.

**Usage:**

Enable this mode to verify that:
- The overlay indicator is displaying correctly
- State transitions are working as expected
- Overlay reconnection recovers properly after errors

**Note:** This is intended for development/testing only and should remain disabled (`false`) in production use.

## For Integrators

If you're integrating Phonesc into your application or building features on top of it, here's how to work with the activation system:

### Keeping the System Awake During Active Use

The `ActivationManager` includes an auto-sleep timer that puts the system to sleep after a configured period of inactivity. When your code is actively processing dictation or executing commands, you should notify the activation manager to prevent auto-sleep:

```rust
// During dictation processing, command execution, or any active user interaction:
activation_manager.notify_activity();
```

**When to call `notify_activity()`:**

- During voice activity detection (VAD) processing
- While transcribing audio (dictation mode)
- During command recognition and execution
- Any time the user is actively interacting with the voice system

**What it does:**

- Resets the inactivity timer, extending the awake period
- Does NOT change the system state (stays Awake if already Awake)
- Prevents the system from sleeping while the user is actively dictating or issuing commands

**Example integration:**

```rust
use std::sync::Arc;
use phonesc::activation::ActivationManager;

async fn process_dictation(
    activation: &Arc<ActivationManager>,
    audio_data: &[f32],
) -> Result<String, Error> {
    // Notify that we're actively processing user input
    activation.notify_activity();

    // Process the audio...
    let transcription = transcribe_audio(audio_data).await?;

    // Notify again if processing took significant time
    activation.notify_activity();

    Ok(transcription)
}
```

### Subscribing to State Changes

You can subscribe to activation state changes to update your UI or adjust behavior:

```rust
let mut state_rx = activation_manager.subscribe();

tokio::spawn(async move {
    while state_rx.changed().await.is_ok() {
        let (state, transition) = *state_rx.borrow();

        match state {
            SystemState::Awake => {
                // System is active - start listening for commands/dictation
            }
            SystemState::Asleep => {
                // System is sleeping - only listen for wake word
            }
        }
    }
});
```

### Updating Configuration at Runtime

Configuration changes are automatically broadcast to all subscribers. To react to config updates in your integration:

```rust
let mut config_rx = config_manager.subscribe();

tokio::spawn(async move {
    while config_rx.changed().await.is_ok() {
        let config = config_rx.borrow().clone();

        // Update your component with new config values
        update_pause_thresholds(
            config.command_pause_threshold_ms,
            config.dictation_pause_threshold_ms,
        );
    }
});
```

## Overlay Indicator

Phonesc displays a small 32x32px circular indicator in the top-right corner of your screen that shows the current system state:

### States

- **Green (Awake)**: The system is active and listening for commands/dictation
- **Gray (Asleep)**: The system is sleeping and only listening for the wake word
- **Red (Error)**: An error occurred (e.g., Wayland compositor disconnected). The system will attempt to recover automatically

### Configuration

Customize the overlay appearance in `~/.config/phonesc/config.toml`:

```toml
[overlay]
# Colors for each state (supports named colors and hex codes)
awake_color = "green"      # or "#00FF00"
asleep_color = "gray"      # or "#808080"
error_color = "red"        # or "#FF0000"

# Position: top-right, top-left, bottom-right, bottom-left
position = "top-right"
```

### Supported Color Names

Named colors: `green`, `lime`, `gray`, `grey`, `red`, `blue`, `yellow`, `cyan`, `magenta`, `white`, `black`, `orange`, `purple`, `pink`

Hex colors: `#RRGGBB` or `#RRGGBBAA` (with alpha channel)

### Real-time Updates

Changes to overlay configuration take effect immediately when you save the config file—no restart required! Edit colors or position and watch the indicator update in real-time.

### Error Recovery and Health Monitoring

If the Wayland compositor disconnects (e.g., when switching to a different TTY or desktop environment), the overlay will:

1. Display a red indicator
2. Log a warning message
3. Attempt to reconnect using exponential backoff (1s, 2s, 4s, 8s, 16s, 30s max)
4. Resume normal operation once reconnected

#### Monitoring Overlay Health

The application provides detailed logging to help you monitor and troubleshoot overlay connection issues:

**Connection Error Messages:**

When the overlay loses connection to the Wayland compositor, you'll see:

```
WARN  Overlay connection error - attempting reconnection
```

This indicates the overlay backend failed to update and will attempt to reconnect.

**Reconnection Progress:**

During reconnection attempts, the logs show the backoff progress:

```
WARN  Overlay reconnecting in 2s (attempt 2)
WARN  Overlay reconnecting in 4s (attempt 3)
```

The backoff sequence is: **1s → 2s → 4s → 8s → 16s → 30s (max)**

After each successful reconnection, the backoff timer resets to 1s.

**Successful Recovery:**

When the connection is restored:

```
INFO  Overlay connection restored
INFO  Overlay reconnected successfully
```

#### Monitoring Configuration Watcher Health

The `ConfigManager` watches your config file for changes and can self-heal if the file watcher fails. You can monitor its health programmatically:

**Checking Health Status:**

```rust
use phonesc::config::{ConfigManager, WatcherHealth};

let health = config_manager.health_status();

match health {
    WatcherHealth::Healthy => {
        // Config watcher is operating normally
    }
    WatcherHealth::Restarting { attempt } => {
        // Watcher is restarting after a failure (attempt N of 5)
    }
    WatcherHealth::Failed { reason } => {
        // Watcher failed permanently after 5 retry attempts
        // Config changes will no longer be detected automatically
    }
}
```

**Health Check API:**

```rust
// Simple boolean check
if !config_manager.is_healthy() {
    tracing::warn!("Config watcher is not healthy - check logs");
}

// Subscribe to health status changes
let mut health_rx = config_manager.health_subscribe();
tokio::spawn(async move {
    while health_rx.changed().await.is_ok() {
        let health = health_rx.borrow().clone();
        tracing::info!("Config watcher health: {:?}", health);
    }
});
```

**Watcher Restart Behavior:**

The config watcher uses a supervised restart strategy:

- **Max retry attempts**: 5
- **Backoff strategy**: Exponential (1s, 2s, 4s, 8s, 16s)
- **Reset condition**: If the watcher runs successfully for 60+ seconds, retry counter resets to 0
- **Permanent failure**: After 5 failed attempts, the watcher stops and health status becomes `Failed`

**Logs to Watch For:**

```
WARN  Config watcher exited unexpectedly after 2s
WARN  Config watcher will restart (attempt 1/5) after 1s
INFO  Config watcher healthy for 60s, resetting retry counter
ERROR Config watcher failed permanently after 5 attempts
```

**Troubleshooting:**

If you see `WatcherHealth::Failed`:
1. Check file system permissions on `~/.config/phonesc/`
2. Ensure the config directory hasn't been deleted or moved
3. Look for file system errors in system logs (e.g., `dmesg`, `journalctl`)
4. Restart the application to reinitialize the watcher

**Note:** Even if the config watcher fails, the application continues running with the last successfully loaded configuration. You can still manually restart the application to reload configuration changes.

## License

TBD
