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
- Live configuration reload

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

## License

TBD
