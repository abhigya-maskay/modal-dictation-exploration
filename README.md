# Modal Dictation Exploration

Voice control software for Linux and macOS.

## Prerequisites

### Linux

Install PortAudio for audio device support:

```bash
# Debian/Ubuntu
sudo apt install libportaudio2

# Fedora
sudo dnf install portaudio

# Arch
sudo pacman -S portaudio
```

### macOS

No additional dependencies required - PortAudio is bundled with the Python package.

## Development

### Quick Start

```bash
# Install dependencies
uv sync

# Run the application
uv run python -m modal_dictation_exploration.main
```

### Available Commands

```bash
# Format code
uv run ruff format .

# Lint code
uv run ruff check --fix .

# Type check
uv run ty check

# Run tests
uv run pytest
```

## Tools

- **uv** - Fast Python package and project manager
- **ruff** - Extremely fast Python linter and formatter
- **ty** - Fast Python type checker
- **pytest** - Testing framework
