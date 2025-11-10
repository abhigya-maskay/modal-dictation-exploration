#!/usr/bin/env python3
"""Emit Waybar-compatible status for the Modal Dictation prototype."""
from __future__ import annotations

import json
import os
import sys
from pathlib import Path
from typing import Any

DEFAULT_STATUS: dict[str, Any] = {
    "text": "●",
    "tooltip": "Modal dictation idle",
    "class": "custom-modal modal-offline",
}


def _state_paths() -> list[Path]:
    """Return candidate paths to read dynamic state from."""
    env_path = os.environ.get("MODAL_STATUS_FILE")
    paths: list[Path] = []
    if env_path:
        paths.append(Path(env_path).expanduser())
    runtime_dir = Path(f"/run/user/{os.getuid()}/modal-dictation/status.json")
    paths.append(runtime_dir)
    return paths


def load_dynamic_state() -> dict[str, Any]:
    """Best-effort load of state overrides from the filesystem."""
    for path in _state_paths():
        try:
            data = path.read_text(encoding="utf-8")
        except FileNotFoundError:
            continue
        except OSError:
            continue
        try:
            parsed = json.loads(data)
        except json.JSONDecodeError:
            continue
        if isinstance(parsed, dict):
            return parsed
    return {}


def main() -> int:
    state = DEFAULT_STATUS.copy()
    dynamic = load_dynamic_state()
    if dynamic:
        state.update(dynamic)
    json.dump(state, sys.stdout, separators=(",", ":"))
    sys.stdout.write("\n")
    sys.stdout.flush()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
