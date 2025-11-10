# Modal Dictation Exploration

High-level notes for a personal Wayland voice-control prototype.

## Vision
- Provide reliable voice-driven control for a Linux Wayland desktop (primary test bed: NixOS on Hyprland).
- Capture microphone audio, run it through a speech-to-text engine (e.g., OpenAI Whisper), and translate utterances into system actions.

## Initial Scope
- **Keyboard simulation**: emit key events for letters (spoken via the NATO/“police” alphabet), digits (0‑9), modifier keys, and common symbols.
- **Dictation toggle**: switch between command mode and free dictation for sentences/phrases.
- **Single user**: optimized for the project author’s workflow; not intended for public distribution yet.

## Audio + Speech Pipeline
- Capture audio from the default PipeWire microphone node (matches the desktop’s existing setup).
- Run transcription through a GPU-accelerated `whisper.cpp` build using Distil-Whisper checkpoints (NVIDIA GeForce 3080, English-only).
- Support collapsed vocabularies/grammars per mode to keep constrained inputs accurate:
  - **Spelling / numeric / symbol modes**: restrict Whisper to the allowed phrases (NATO alphabet, digits, modifiers, symbols) via grammars or prompt bias.
  - **Dictation mode**: switch to Distil-Whisper Large v3 English for high-quality free-form text with acceptable latency on this hardware.

## Key Injection Strategy
- Use `wtype` (Wayland virtual-keyboard client) to emit key events that Hyprland accepts out of the box.
- Map each spoken token to `wtype` sequences (letters, digits, modifiers, symbols) while dictation mode routes text as literal strings.
- Extend or wrap `wtype` later if specialized events or composition behaviors are needed.

More detail will be added as the design solidifies.

## Development Environment

This repo is managed with Nix flakes and Poetry.

1. Enter the dev shell: `nix develop`.
2. Install Python deps (first run only): `poetry install`.
3. Install git hooks: `poetry run pre-commit install`.
4. Run formatting/linting/tests (hooks run these automatically on commit):
   - `poetry run black .`
   - `ruff check .`
   - `poetry run pytest`
   - `poetry run pyright`
