"""Tests for the tray module."""

from unittest.mock import MagicMock

import pytest
from PIL import Image

from modal_dictation_exploration.tray import (
    _create_icon_image,
    _on_quit,
    setup_tray,
)


class TestCreateIconImage:
    """Tests for _create_icon_image function."""

    def test_returns_image_instance(self):
        """Should return a PIL Image."""
        result = _create_icon_image()
        assert isinstance(result, Image.Image)

    def test_custom_size(self):
        """Should respect custom size parameter."""
        result = _create_icon_image(size=128)
        assert result.size == (128, 128)


class TestOnQuit:
    """Tests for _on_quit callback function."""

    def test_calls_icon_stop(self):
        """Should call stop() on the icon."""
        mock_icon = MagicMock()
        mock_item = MagicMock()

        _on_quit(mock_icon, mock_item)

        mock_icon.stop.assert_called_once()


class TestSetupTray:
    """Tests for setup_tray function."""

    def test_returns_icon_instance(self, pystray_mock):
        """Should return a pystray Icon."""
        mock_icon = MagicMock()
        pystray_mock.Icon.return_value = mock_icon

        result = setup_tray()

        assert result == mock_icon

    def test_creates_menu_with_quit_item(self, pystray_mock):
        """Should create a menu with a Quit option."""
        setup_tray()

        pystray_mock.MenuItem.assert_called_once()
        menu_item_args = pystray_mock.MenuItem.call_args
        assert menu_item_args[0][0] == "Quit"

    def test_raises_runtime_error_on_failure(self, pystray_mock):
        """Should raise RuntimeError if icon creation fails."""
        pystray_mock.Icon.side_effect = Exception("No display available")

        with pytest.raises(RuntimeError, match="Failed to set up system tray icon"):
            setup_tray()
