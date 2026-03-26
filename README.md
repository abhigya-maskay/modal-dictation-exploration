# Modal Dictation

A macOS app for voice-driven input with two distinct modes — inspired by vim's modal philosophy.

## Modes

**Dictation Mode** — Speech is transcribed and inserted as text. Optimized for natural language input with punctuation and capitalization.

**Command Mode** — Speech is interpreted as keyboard actions (key presses, modifier combos, navigation). A fixed vocabulary of commands is matched against speech input using acoustic rescoring and phonetic fuzzy matching.

## Speech Recognition

All speech recognition runs locally on-device via [FluidAudio](https://github.com/FluidInference/FluidAudio) and NVIDIA Parakeet models converted to CoreML for Apple Silicon.

- **Dictation** — Parakeet TDT v2 (0.6B) for high-accuracy batch transcription
- **Commands** — Parakeet EOU (120M) for low-latency streaming recognition with custom vocabulary boosting

## Requirements

- macOS 14+
- Apple Silicon (M1 or later)
