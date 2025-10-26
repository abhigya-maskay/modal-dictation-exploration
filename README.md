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

### Error Recovery

If the Wayland compositor disconnects (e.g., when switching to a different TTY or desktop environment), the overlay will:

1. Display a red indicator
2. Log a warning message
3. Attempt to reconnect using exponential backoff (1s, 2s, 4s, 8s, 16s, 30s max)
4. Resume normal operation once reconnected

## License

TBD
