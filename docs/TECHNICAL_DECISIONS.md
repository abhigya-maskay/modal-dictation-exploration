# Technical Decisions

## High‑Level Architecture
- Rust core application:
  - Wake‑word (KWS) for “Quasar”
  - VAD + endpointing (Rust‑side)
  - Command ASR (closed grammar) with whisper.cpp
  - Dictation client (HTTP) to faster‑whisper service
  - Key injection (Wayland virtual keyboard)
  - Overlay indicator (top‑right, Awake/Asleep)
  - Config loader (TOML + live reload)
  - Model manager (first‑run download)
  - Minimal logging
- External service (Linux dictation):
  - Python + faster‑whisper (CTranslate2) using Distil‑Whisper large‑v3
  - HTTP POST /v1/transcribe: WAV in → JSON out (final only), 0.0.0.0:5123

## Audio, VAD, Endpointing
- Capture: 16 kHz mono, 16‑bit PCM via a Rust audio crate (e.g., cpal).
- VAD: WebRTC VAD + energy gating; endpoint on trailing silence.
- Timeouts:
  - Commands/letters: 700 ms
  - Dictation (scribe): 900 ms
- Transport to dictation service: buffer utterance WAV; send after endpoint.

## Wake/Sleep Logic
- KWS engine: local, enrolled for “Quasar” (10–20 samples), single shared model OK.
- While Asleep: only KWS runs; Whisper remains idle.
- While Awake: Command ASR runs; “Dormant” recognized by command ASR to sleep.
- Auto‑sleep: 5 minutes after last recognized command/dictation; reset only on recognized speech.

## Command ASR (Closed Vocabulary)
- Engine: whisper.cpp via Rust bindings (or FFI).
- Model: base.en (CPU), quantized if needed.
- Grammar: GBNF for strict closed vocabulary:
  - Police alphabet words → letters
  - Digits zero–nine → 0–9; concatenation per utterance
  - Modifiers: control, shift, alt; command/super synonyms mapped to platform
  - Keys: enter, escape, tab, space, backspace, delete, arrow keys
  - Symbols: single names (as specified)
  - Delimiters: “open/close” + paren/bracket/brace/angle
  - “word …” prefix (single lowercase token, letters/digits only)
  - “scratch that”
- Out‑of‑grammar or low confidence: ignore silently.

## Dictation Service (HTTP, Linux)
- Implementation: Python + faster‑whisper (CTranslate2)
  - Model: Distil‑Whisper large‑v3 (English), device=cuda, compute_type=float16
  - Preload and warm kernels at startup
  - No service‑side VAD; final‑only decoding per request
- API:
  - POST /v1/transcribe
  - Request: WAV (16 kHz mono 16‑bit), Content‑Type: audio/wav
  - Response: JSON: { "text": "<final transcript>" }
  - Keep‑alive enabled; minimal logging
- Binding: 0.0.0.0:5123 (LAN), no auth/TLS (MVP)

## Key Injection
- Wayland: zwp_virtual_keyboard_v1 (Hyprland). Sends:
  - Characters and keycodes for letters/digits/symbols
  - Modifier chords (e.g., Ctrl+Shift+P)
  - Special keys and arrows
- No auto‑pairing of delimiters; each command inserts one symbol.

## Overlay
- Rendering: lightweight Wayland client window (always‑on‑top if possible via compositor hints).
- Position: top‑right; colors: green (Awake), red (Asleep).
- Minimal content (no mic level, no partials).

## Configuration & Reload
- Path: ~/.config/phonesc/config.toml
- Live reload: watch parent directory (Nix symlink‑safe); on change, reparse and apply:
  - Timeouts, overlay position/colors, dictation service URL/port
  - Command vocabulary aliases/symbol name mapping (and regenerate GBNF)
- Restart required for: audio device changes, model path/type swaps, KWS model changes.

## Models & Storage
- First‑run download to ~/.local/share/phonesc/models:
  - whisper.cpp base.en (command model)
  - KWS model/artifacts for “Quasar”
- Dictation is external (service manages Distil‑Whisper).

## Networking & Offload (Future‑Ready)
- TCP/HTTP chosen for easy Mac→Linux offload later.
- Client logic: prefer remote service; if unreachable, fall back to local dictation backend (future addition).
- Commands can also be offloaded in the future via an additional HTTP endpoint, but local closed‑grammar remains as fallback.

## Logging & Privacy
- Default: minimal text logs only (state transitions, errors).
- No audio saved.
- Optional debug toggles can be added later (timings, ASR scores).

## Constraints & Non‑Goals (MVP)
- No partial transcript streaming (final‑only).
- No per‑app routing or password‑field detection.
- No automatic delimiter pairing.
- No repeats for navigation; rely on editor (e.g., Neovim) for repetition behavior.
- macOS path deferred; Linux‑only MVP.

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

