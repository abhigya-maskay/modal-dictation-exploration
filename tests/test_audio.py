"""Tests for the audio module."""

from unittest.mock import MagicMock, patch

import pytest

from modal_dictation_exploration.audio import list_input_devices
from tests.factories.pulsectl import PulseSourceInfoFactory


@pytest.fixture
def mock_pulse():
    """Patch pulsectl.Pulse and yield the mock pulse instance."""
    mock = MagicMock()

    with patch("modal_dictation_exploration.audio.pulsectl.Pulse") as mock_pulse_class:
        mock_pulse_class.return_value.__enter__.return_value = mock
        yield mock


def test_list_input_devices_returns_audio_device_instances_with_correct_fields(mock_pulse):
    """Returned items are AudioDevice instances with correct field mapping."""
    source = PulseSourceInfoFactory.build()
    mock_pulse.source_list.return_value = [source]

    result = list_input_devices()

    assert len(result) == 1
    device = result[0]
    assert device.name == source.name
    assert device.description == source.description
    assert device.index == source.index
    assert device.channels == source.channel_count
    assert device.sample_rate == source.sample_spec.rate
    assert device.mute == bool(source.mute)
    assert device.state == source.state


def test_list_input_devices_filters_out_monitor_sources(mock_pulse):
    """Sources ending in .monitor are excluded from results."""
    normal_source = PulseSourceInfoFactory.build(name="alsa_input.usb-MOTU_M2")
    monitor_source = PulseSourceInfoFactory.build(name="alsa_output.usb-MOTU_M2.monitor")
    mock_pulse.source_list.return_value = [normal_source, monitor_source]

    result = list_input_devices()

    assert len(result) == 1
    assert result[0].name == normal_source.name


def test_list_input_devices_returns_empty_list_when_no_sources(mock_pulse):
    """Returns empty list when no sources exist."""
    mock_pulse.source_list.return_value = []

    result = list_input_devices()

    assert result == []


def test_list_input_devices_returns_empty_list_when_all_monitors(mock_pulse):
    """Returns empty list when all sources are monitors."""
    monitor1 = PulseSourceInfoFactory.build(name="alsa_output.speakers.monitor")
    monitor2 = PulseSourceInfoFactory.build(name="alsa_output.headphones.monitor")
    mock_pulse.source_list.return_value = [monitor1, monitor2]

    result = list_input_devices()

    assert result == []


def test_list_input_devices_returns_empty_list_on_connection_failure():
    """Returns empty list with warning when PulseAudio connection fails."""
    import pulsectl

    with patch("modal_dictation_exploration.audio.pulsectl.Pulse") as mock_pulse_class:
        mock_pulse_class.side_effect = pulsectl.PulseError("Connection refused")
        result = list_input_devices()

    assert result == []
