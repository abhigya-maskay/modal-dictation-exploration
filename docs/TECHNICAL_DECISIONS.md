# Technical Decisions

## High‑Level Architecture
- Rust core application:
  - Wake‑word (KWS) for “Quasar”
  - VAD + endpointing (Rust‑side)
  - Command ASR (closed grammar) with `whisper-rs` (whisper.cpp)
  - Dictation client (`reqwest`, single HTTP attempt per utterance) to faster-whisper service
  - Key injection (Wayland virtual keyboard via smithay-client-toolkit)
  - Overlay indicator (smithay-client-toolkit, top-right, Awake/Asleep/Error colors)
  - Config loader (TOML + live reload)
  - Model manager (first‑run download)
  - Minimal logging
- External service (Linux dictation):
  - Python + faster‑whisper (CTranslate2) using Distil‑Whisper large‑v3
  - HTTP POST /v1/transcribe: WAV in → JSON out (final only), 0.0.0.0:5123

## Concurrency & Runtime
- Runtime: `tokio`.
- Structure: actor-style tasks per subsystem (audio, wake-word, command ASR, overlay, timers).
- Messaging: prefer `tokio::mpsc` for single-consumer channels; introduce broadcast channels only when multiple observers are required.

## Audio, VAD, Endpointing
- Capture: 16 kHz mono, 16-bit PCM via `cpal` (PipeWire backend expected).
- VAD: Silero VAD (ONNX) executed with `onnxruntime`; endpoint on trailing silence.
- Environment: PipeWire/PulseAudio desktops; ALSA-only setups are out of scope.
- Timeouts:
  - Commands/letters: 700 ms
  - Dictation (scribe): 900 ms
- Transport to dictation service: buffer utterance WAV; send after endpoint.

## Wake/Sleep Logic
- KWS engine: Porcupine (FFI) with bundled “Quasar” model; preload detector at startup.
- While Asleep: only KWS runs; Whisper remains idle.
- While Awake: Command ASR runs; “Dormant” recognized by command ASR to sleep.
- Auto-sleep: 5 minutes after last recognized command/dictation; only recognized commands/dictation reset the timer (wake word alone does not).

## Command ASR (Closed Vocabulary)
- Engine: `whisper-rs` (whisper.cpp bindings).
- Model: base.en (CPU), quantized if needed.
- Grammar: GBNF for strict closed vocabulary:
  - Police alphabet words → letters
  - Digits zero–nine → 0–9; concatenation per utterance
  - Modifiers: control, shift, alt; command/super synonyms mapped to platform
  - Keys: enter, escape, tab, space, backspace, delete, arrow keys
  - Symbols: single names (as specified)
  - Delimiters: “open/close” + paren/bracket/brace/angle
  - “word …” prefix (single lowercase token, letters/digits only)
-  - “scratch that”
-  - Regenerated on config reload; new grammar activated on the next wake cycle.
- Out‑of‑grammar or low confidence: ignore silently.

## Dictation Service (HTTP, Linux)
- Implementation: Python + FastAPI + faster-whisper (CTranslate2)
  - Model: Distil‑Whisper large‑v3 (English), device=cuda, compute_type=float16
  - Preload and warm kernels at startup
  - No service‑side VAD; final‑only decoding per request
- API:
  - POST /v1/transcribe
  - Request: WAV (16 kHz mono 16-bit), Content-Type: audio/wav
  - Response: JSON: { "text": "<final transcript>" }
  - GET /healthz for readiness checks
- No transcript caching beyond request scope (in-memory only).
  - Keep‑alive enabled; minimal logging
- Binding: 0.0.0.0:5123 (LAN), no auth/TLS (MVP)

## Key Injection
- Wayland: zwp_virtual_keyboard_v1 (Hyprland). Sends:
  - Characters and keycodes for letters/digits/symbols
  - Modifier chords (e.g., Ctrl+Shift+P)
  - Special keys and arrows
- No auto‑pairing of delimiters; each command inserts one symbol.

## Overlay
- Rendering: lightweight smithay-client-toolkit surface (always-on-top if possible via compositor hints) with solid-color primitives.
- Position: top-right; colors: green (Awake), gray (Asleep), red (error).
- Minimal content (no mic level, no partials).

## Configuration & Reload
- Path: ~/.config/phonesc/config.toml
- Parsing: `serde` + `toml` crate into a typed config struct.
- Live reload: watch parent directory via `notify` (Nix symlink-safe); on change, reparse and apply:
  - Timeouts, overlay position/colors, dictation service URL/port
  - Command vocabulary aliases/symbol name mapping (regenerate GBNF for next wake cycle)
- Restart required for: audio device changes, model path/type swaps, KWS model changes.

## Models & Storage
- First-run download to ~/.local/share/phonesc/models:
  - whisper.cpp base.en (command model)
  - Silero VAD ONNX model
- Porcupine wake-word assets are bundled with the distribution (repo-vendored).
- Dictation is external (service manages Distil-Whisper).
- Small third-party assets (<~200 MB) may be vendored; larger models download on demand.

## Networking & Offload (Future‑Ready)
- TCP/HTTP chosen for easy Mac→Linux offload later.
- Client logic: prefer remote service; if unreachable, fall back to local dictation backend (future addition).
- Commands can also be offloaded in the future via an additional HTTP endpoint, but local closed-grammar remains as fallback.
- HTTP requests are single-attempt; failures surface via logging and overlay error state.

## Logging & Privacy
- Default: `tracing` logs (state transitions, errors); `--debug` flag increases verbosity.
- No audio saved.
- Optional debug toggles can be added later (timings, ASR scores).

## Constraints & Non‑Goals (MVP)
- No partial transcript streaming (final‑only).
- No per‑app routing or password‑field detection.
- No automatic delimiter pairing.
- No repeats for navigation; rely on editor (e.g., Neovim) for repetition behavior.
- macOS path deferred; Linux‑only MVP.

- No automated Wayland integration tests; rely on unit tests and manual verification.
- Distribution beyond a standalone binary is out of scope for the MVP.

## Example Utterances (Behavioral Reference)
- Wake: “Quasar”
- Sleep: “Dormant”
- Letters: “Adam Boy Charles” → “abc”; “shift Paul” → “P”
- Digits: “one two three” → “123”
- Chord: “control shift Paul”
- Symbols: “minus”, “pipe”, “hash”
- Delimiters: “open paren” → “(”; “close brace” → “}”
- Special keys: “enter”, “tab”, “backspace”, “delete”, “left”
- Dictation: “scribe create a new function called parse input”
- Single word: “word hello”
- Undo: “scratch that” (within ~5 seconds)
