import pystray
from PIL import Image, ImageDraw

__all__ = ["setup_tray"]

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


def setup_tray() -> pystray.Icon:
    """Set up and return the system tray icon.

    Raises:
        RuntimeError: If tray icon setup fails (e.g., no display available).
    """
    try:
        image = _create_icon_image()
        menu = pystray.Menu(pystray.MenuItem("Quit", _on_quit))
        icon = pystray.Icon("modal-dictation", image, "Modal Dictation", menu)
        return icon
    except Exception as e:
        raise RuntimeError(f"Failed to set up system tray icon: {e}") from e
