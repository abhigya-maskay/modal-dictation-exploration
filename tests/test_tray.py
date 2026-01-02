"""Tests for the tray module."""

from unittest.mock import MagicMock, patch

import pytest
from PIL import Image

from modal_dictation_exploration.main import create_app_state
from modal_dictation_exploration.tray import (
    _create_icon_image,
    _on_quit,
    make_device_checked_callback,
    make_device_selected_callback,
    setup_tray,
)
from tests.factories.audio import AudioDeviceFactory


def test_create_icon_image_returns_image_instance():
    """Should return a PIL Image."""
    result = _create_icon_image()
    assert isinstance(result, Image.Image)


def test_create_icon_image_custom_size():
    """Should respect custom size parameter."""
    result = _create_icon_image(size=128)
    assert result.size == (128, 128)


def test_on_quit_calls_icon_stop():
    """Should call stop() on the icon."""
    mock_icon = MagicMock()
    mock_item = MagicMock()

    _on_quit(mock_icon, mock_item)

    mock_icon.stop.assert_called_once()


def test_setup_tray_returns_icon_instance(pystray_mock):
    """Should return a pystray Icon."""
    mock_icon = MagicMock()
    pystray_mock.Icon.return_value = mock_icon
    state = create_app_state()

    result = setup_tray(state)

    assert result == mock_icon


def test_setup_tray_creates_menu_with_quit_item(pystray_mock):
    """Should create a menu with a Quit option."""
    state = create_app_state()

    setup_tray(state)

    quit_calls = [
        call
        for call in pystray_mock.MenuItem.call_args_list
        if call[0][0] == "Quit"
    ]
    assert len(quit_calls) == 1


def test_setup_tray_raises_runtime_error_on_failure(pystray_mock):
    """Should raise RuntimeError if icon creation fails."""
    pystray_mock.Icon.side_effect = Exception("No display available")
    state = create_app_state()

    with pytest.raises(RuntimeError, match="Failed to set up system tray icon"):
        setup_tray(state)


def test_setup_tray_menu_contains_audio_devices_submenu(pystray_mock):
    """Menu has 'Audio Devices' submenu when devices exist."""
    state = create_app_state()
    devices = [AudioDeviceFactory.build(), AudioDeviceFactory.build()]

    with patch(
        "modal_dictation_exploration.tray.list_input_devices", return_value=devices
    ):
        setup_tray(state)

        audio_devices_calls = [
            call
            for call in pystray_mock.MenuItem.call_args_list
            if call[0][0] == "Audio Devices"
        ]
        assert len(audio_devices_calls) == 1


def test_setup_tray_audio_devices_submenu_lists_input_devices(pystray_mock):
    """Submenu contains menu items for each input device."""
    state = create_app_state()
    devices = [
        AudioDeviceFactory.build(description="MOTU M2"),
        AudioDeviceFactory.build(description="Built-in Microphone"),
    ]

    with patch(
        "modal_dictation_exploration.tray.list_input_devices", return_value=devices
    ):
        setup_tray(state)

        menu_item_names = [
            call[0][0] for call in pystray_mock.MenuItem.call_args_list
        ]
        assert "MOTU M2" in menu_item_names
        assert "Built-in Microphone" in menu_item_names


def test_setup_tray_audio_devices_submenu_shows_no_devices_when_empty(pystray_mock):
    """Shows disabled 'No devices found' when no devices."""
    state = create_app_state()

    with patch(
        "modal_dictation_exploration.tray.list_input_devices", return_value=[]
    ):
        setup_tray(state)

        menu_item_calls = pystray_mock.MenuItem.call_args_list
        menu_item_names = [call[0][0] for call in menu_item_calls]

        assert "Audio Devices" not in menu_item_names

        no_devices_calls = [
            call for call in menu_item_calls if call[0][0] == "No devices found"
        ]
        assert len(no_devices_calls) == 1
        assert no_devices_calls[0].kwargs.get("enabled") is False


def test_make_device_selected_callback_updates_selected_device():
    """Invoking the callback updates selected_device state."""
    state = create_app_state()

    callback = make_device_selected_callback(state.selected_device, "alsa_input.usb-MOTU_M2")
    callback(MagicMock(), MagicMock())

    assert state.selected_device.value == "alsa_input.usb-MOTU_M2"


def test_make_device_checked_callback_returns_true_when_device_is_selected():
    """Returns True when the device name matches selected_device."""
    state = create_app_state()
    state.selected_device.next("alsa_input.usb-MOTU_M2")

    callback = make_device_checked_callback(state.selected_device, "alsa_input.usb-MOTU_M2")

    assert callback(MagicMock()) is True
