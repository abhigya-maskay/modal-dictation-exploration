# Product Requirements (Linux MVP)

## Overview
- Purpose: Voice-driven, modal keyboard input for coding and window navigation.
- Modes: Single “Normal” mode with closed‑vocabulary commands. Temporary dictation via the “scribe …” prefix.
- Platforms: Linux Wayland (Hyprland). macOS support and offload come later.

## Activation & State
- Wake word: “Quasar” (voice activation). Goes from Asleep → Awake.
- Sleep word: “Dormant” (recognized only while Awake).
- Auto‑sleep: After 5 minutes without recognized speech (commands or dictation). Only recognized commands or dictation reset the timer (wake word alone does not).
- Indicator: Small overlay, top-right of screen.
  - Awake = green
  - Asleep = gray
  - Error = red

## Microphone & Audio
- Input: Default system microphone.
- Listening: Always on for wake word; runs command recognition only when Awake.
- Utterance end (pause) thresholds:
  - 700 ms for commands/letters.
  - 900 ms for “scribe …” dictation.

## Normal Mode (Closed Vocabulary)
- Behavior: Only the specified command grammar is accepted; out‑of‑grammar is ignored silently.
- Letters (lowercase by default):
  - Police alphabet only: Adam, Boy, Charles, David, Edward, Frank, George, Henry, Ida, John, King, Lincoln, Mary, Nora, Ocean, Paul, Queen, Robert, Sam, Tom, Union, Victor, William, X‑ray, Young, Zebra.
  - Uppercase: say “shift <police-letter>” (e.g., “shift Adam” → “A”).
  - Multiple letters in one utterance concatenate with no spaces (“Adam Boy Charles” → “abc”).
- Digits:
  - Only “zero … nine”. Multi‑digit sequences type continuous numbers (“one two three” → “123”). No “oh” for zero.
- Modifiers and chords:
  - Words: “control”, “shift”, “alt”; “command” and “super” are synonyms (map to platform).
  - Chords: speak modifiers + target (e.g., “control shift Paul” → Ctrl+Shift+P).
  - Multi‑step sequences are entered one by one (no “then …” needed).
- Navigation and specials:
  - Keys: “enter”, “escape”, “tab”, “space”, “backspace” (backward delete), “delete” (forward delete), “left”, “right”, “up”, “down”.
- Symbols (single names):
  - Period, comma, colon, semicolon
  - Minus, underscore, plus
  - Asterisk, slash, backslash, pipe
  - Ampersand, at, hash, dollar, percent, caret, tilde, equals
  - Less, greater, question mark
  - Quotes: “quote” (double), “apostrophe” (single), “backtick” (backtick)
  - Delimiters: “open/close paren”, “open/close bracket”, “open/close brace”, “open/close angle”
  - Inserts single characters (no auto‑pairing).
- “word …”:
  - One utterance: “word hello” → inserts “hello” (single lowercase token; no symbols).

## Dictation (“scribe …”)
- Trigger: Say “scribe …” to dictate a short span. Auto‑ends on pause (≈900 ms).
- Output: Auto‑punctuated dictation using the model’s defaults.
  - No post‑processing to force literal words; output is inserted as recognized.
  - If you say “comma/period”, the model may produce punctuation; app does not override.

## Undo
- “scratch that”: Deletes the last insertion produced by this app within ~5 seconds. Tracks only the most recent insertion (not repeatable). Does not revert navigation/chords.

## Key Injection
- Wayland: Uses virtual keyboard protocol to type into the currently focused field.
- Scope: Global (focused app). No per‑app routing in MVP.

## Configuration
- File: ~/.config/phonesc/config.toml
- Live reload: Changes to configurable behavior (e.g., timeouts, symbol names, overlay colors/position, service URL) apply without restart.
- Models: Download on first run to ~/.local/share/phonesc/models

## Startup
- MVP: Manual CLI launch.
- Autostart: Can be added later (user‑level service).

## Networking (Dictation Service)
- Endpoint: HTTP POST on 127.0.0.1:5123 (LAN accessible).
- Payload: Single WAV (16 kHz, mono, 16‑bit). Returns JSON with final transcript.
- Security: No auth/TLS in MVP (private LAN assumed).

## Privacy & Logging
- No audio is saved. Minimal text logs only.
- Debug logging can be enabled later if needed.
