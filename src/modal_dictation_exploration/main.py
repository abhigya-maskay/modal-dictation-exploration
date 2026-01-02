import logging

from modal_dictation_exploration.state.app_state import AppState
from modal_dictation_exploration.state.async_behavior_subject import (
    AsyncBehaviorSubject,
)
from modal_dictation_exploration.tray import setup_tray

logger = logging.getLogger(__name__)


def create_app_state() -> AppState:
    """Create and return the initial application state."""
    return AppState(selected_device=AsyncBehaviorSubject(None))


def main() -> None:
    """Start the modal dictation application with a system tray icon."""
    logging.basicConfig(level=logging.INFO)
    logger.info("Modal dictation application started")

    state = create_app_state()
    icon = setup_tray(state)
    icon.run()


if __name__ == "__main__":
    main()
