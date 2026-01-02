import pystray
from PIL import Image, ImageDraw

from modal_dictation_exploration.audio import list_input_devices
from modal_dictation_exploration.state.app_state import AppState
from modal_dictation_exploration.state.async_behavior_subject import (
    AsyncBehaviorSubject,
)

__all__ = [
    "setup_tray",
    "make_device_selected_callback",
    "make_device_checked_callback",
]

# Constants
DEFAULT_ICON_SIZE = 64
ICON_COLOR_ACTIVE = "green"


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


def make_device_selected_callback(
    selected_device: AsyncBehaviorSubject[str | None],
    device_name: str,
):
    """Create a callback that selects an audio device.

    Args:
        selected_device: The subject to update when the device is selected.
        device_index: The device index to set when the callback is invoked.

    Returns:
        A callback function compatible with pystray.MenuItem.
    """

    def callback(_icon: pystray.Icon, _item: pystray.MenuItem) -> None:
        selected_device.next(device_name)

    return callback


def make_device_checked_callback(
    selected_device: AsyncBehaviorSubject[str | None],
    device_name: str,
):
    """Create a callback that checks if a device is selected.

    Args:
        selected_device: The subject containing the currently selected device.
        device_index: The device index to check against.

    Returns:
        A callback function compatible with pystray.MenuItem's checked parameter.
    """

    def callback(_item: pystray.MenuItem) -> bool:
        return selected_device.value == device_name

    return callback


def _create_audio_devices_menu_item(
    selected_device: AsyncBehaviorSubject[str | None],
) -> pystray.MenuItem:
    """Create the menu item for audio input device selection.

    Returns:
        A pystray.MenuItem containing either a submenu of available input devices,
        or a disabled "No devices found" item if no input devices are available.
    """
    devices = list_input_devices()
    if not devices:
        return pystray.MenuItem("No devices found", None, enabled=False)

    menu_items = [
        pystray.MenuItem(
            device.description,
            make_device_selected_callback(selected_device, device.name),
            checked=make_device_checked_callback(selected_device, device.name),
        )
        for device in devices
    ]
    return pystray.MenuItem("Audio Devices", pystray.Menu(*menu_items))


def setup_tray(state: AppState) -> pystray.Icon:
    """Set up and return the system tray icon.

    Raises:
        RuntimeError: If tray icon setup fails (e.g., no display available).
    """
    try:
        image = _create_icon_image()
        menu = pystray.Menu(
            _create_audio_devices_menu_item(state.selected_device),
            pystray.MenuItem("Quit", _on_quit),
        )
        icon = pystray.Icon("modal-dictation", image, "Modal Dictation", menu)
        return icon
    except Exception as e:
        raise RuntimeError(f"Failed to set up system tray icon: {e}") from e
