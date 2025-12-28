import logging

from modal_dictation_exploration.tray import setup_tray

logger = logging.getLogger(__name__)


def main() -> None:
    """Start the modal dictation application with a system tray icon."""
    logging.basicConfig(level=logging.INFO)
    logger.info("Modal dictation application started")

    icon = setup_tray()
    icon.run()


if __name__ == "__main__":
    main()
