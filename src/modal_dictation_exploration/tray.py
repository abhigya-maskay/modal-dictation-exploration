import logging
from concurrent.futures import ThreadPoolExecutor

import pystray
from PIL import Image, ImageDraw

logger = logging.getLogger(__name__)

from modal_dictation_exploration.audio import (
    list_input_devices,
    prepare_audio_for_transcription,
    record_audio,
)
from modal_dictation_exploration.state.app_state import AppState
from modal_dictation_exploration.state.async_behavior_subject import (
    AsyncBehaviorSubject,
)
from modal_dictation_exploration.transcription import Transcriber

__all__ = [
    "setup_tray",
]

# Constants
DEFAULT_ICON_SIZE = 64
ICON_COLOR_ACTIVE = "green"

# Audio executor - created before pystray to avoid GTK/PulseAudio thread conflicts
_audio_executor = ThreadPoolExecutor(max_workers=1)


def _create_icon_image(size: int = DEFAULT_ICON_SIZE) -> Image.Image:
    """Create a green circle icon.

    Args:
        size: Icon dimensions in pixels. Defaults to 64.

    Returns:
        RGBA image with transparent background and green circle.
    """
    image = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)
    draw.ellipse((0, 0, size - 1, size - 1), fill=ICON_COLOR_ACTIVE)
    return image


def _on_quit(icon: pystray.Icon, _item: pystray.MenuItem) -> None:
    """Stop the tray icon and exit the application."""
    icon.stop()





def _create_record_transcribe_menu_item(
    selected_device: AsyncBehaviorSubject[int | None],
    transcriber: Transcriber,
) -> pystray.MenuItem:
    def make_record_transcribe_callback():
        def do_record_transcribe() -> None:
            try:
                device_index = selected_device.value
                assert device_index is not None

                logger.info(f"Starting recording from device {device_index}")
                audio_data, sample_rate = record_audio(device_index)
                logger.info(f"Recording complete: {audio_data.shape}, {sample_rate}Hz")
                logger.info(f"Audio stats: min={audio_data.min():.6f}, max={audio_data.max():.6f}, mean={audio_data.mean():.6f}")

                logger.info("Processing audio for transcription")
                processed_audio = prepare_audio_for_transcription(audio_data, sample_rate)
                logger.info(f"Audio processed: {processed_audio.shape}")
                logger.info(f"Processed audio stats: min={processed_audio.min():.6f}, max={processed_audio.max():.6f}, mean={processed_audio.mean():.6f}")

                logger.info("Starting transcription")
                transcription = transcriber.transcribe(processed_audio)
                logger.info(f"Transcription: {transcription}")
            except Exception as e:
                logger.exception(f"Error during record/transcribe: {e}")

        def callback(_icon: pystray.Icon, _item: pystray.MenuItem) -> None:
            logger.info("Record & Transcribe callback triggered")
            _audio_executor.submit(do_record_transcribe)
            logger.info("Task submitted to audio executor")

        return callback

    def is_enabled(_item: pystray.MenuItem) -> bool:
        return selected_device.value is not None

    return pystray.MenuItem(
        "Record & Transcribe (5s)",
        make_record_transcribe_callback(),
        enabled=is_enabled,
    )


def _create_audio_devices_menu_item(
    selected_device: AsyncBehaviorSubject[int | None],
) -> pystray.MenuItem:
    """Create the menu item for audio input device selection.

    Returns:
        A pystray.MenuItem containing either a submenu of available input devices,
        or a disabled "No devices found" item if no input devices are available.
    """

    def make_device_selected_callback(device_index: int):
        def callback(_icon: pystray.Icon, _item: pystray.MenuItem) -> None:
            selected_device.next(device_index)

        return callback

    def make_device_checked_callback(device_index: int):
        def callback(_item: pystray.MenuItem) -> bool:
            return selected_device.value == device_index

        return callback

    devices = list_input_devices()
    if not devices:
        return pystray.MenuItem("No devices found", None, enabled=False)

    menu_items = [
        pystray.MenuItem(
            device.name,
            make_device_selected_callback(device.index),
            checked=make_device_checked_callback(device.index),
        )
        for device in devices
    ]
    return pystray.MenuItem("Audio Devices", pystray.Menu(*menu_items))


def setup_tray(state: AppState, transcriber: Transcriber) -> pystray.Icon:
    """Set up and return the system tray icon.

    Raises:
        RuntimeError: If tray icon setup fails (e.g., no display available).
    """
    try:
        image = _create_icon_image()
        menu = pystray.Menu(
            _create_audio_devices_menu_item(state.selected_device),
            _create_record_transcribe_menu_item(state.selected_device, transcriber),
            pystray.MenuItem("Quit", _on_quit),
        )
        icon = pystray.Icon("modal-dictation", image, "Modal Dictation", menu)
        return icon
    except Exception as e:
        raise RuntimeError(f"Failed to set up system tray icon: {e}") from e
