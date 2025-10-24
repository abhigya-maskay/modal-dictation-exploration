# Technology Selection

## Audio & Speech
- `cpal` for 16 kHz mono capture on PipeWire/PulseAudio desktops; ALSA-only setups are out of scope.
- Porcupine (FFI) powers the “Quasar” wake word, with its small model assets vendored in-repo and preloaded at startup.
- Silero VAD (ONNX) executed via `onnxruntime` for higher accuracy endpointing.
- `whisper-rs` (whisper.cpp) drives closed-grammar command recognition; the base.en model downloads on first run and grammar swaps occur on the next wake cycle.

## Runtime & Observability
- `tokio` provides the async runtime; subsystems run as actor-style tasks.
- Messaging uses `tokio::mpsc` for single-consumer paths, introducing broadcast channels only when multiple observers are required.
- `tracing` handles structured logging, with a `--debug` CLI flag to increase verbosity.

## Configuration & Storage
- Config parsing relies on `serde` + `toml`, loading into a typed struct.
- `notify` watches `~/.config/phonesc` for live reloads.
- Whisper and Silero models download to `~/.local/share/phonesc/models` on first run; small third-party assets (<~200 MB) may be vendored, larger models download on demand.
- Porcupine assets ship with the binary (repo-vendored) for immediate wake-word availability.

## UI & Input
- `smithay-client-toolkit` backs both the overlay surface and Wayland virtual keyboard integration.
- The overlay renders simple solid-color states (green awake, gray asleep, red error) without additional widgets.

## Networking & Services
- The Rust dictation client uses `reqwest` for single-attempt HTTP POSTs; failures surface via logs and the overlay error state.
- The external dictation service is built with FastAPI and faster-whisper (Distil-Whisper large-v3), exposing `/v1/transcribe` and `/healthz`.
- Transcription responses remain in-memory only; no server-side caching layer is introduced.

## Distribution & Testing
- Ship as a standalone Linux binary; environment dependencies (e.g., PipeWire, Python service) are user-managed.
- Limit automated coverage to unit tests, relying on manual verification for Wayland integration flows.
