"""Tests for the audio module."""

from unittest.mock import MagicMock, patch

from modal_dictation_exploration.audio import list_input_devices
from modal_dictation_exploration.types.sounddevice import DeviceInfo, HostApiInfo
from tests.factories.sounddevice import DeviceInfoFactory, HostApiInfoFactory


def setup_sounddevice_mock(
    mock_sd: MagicMock,
    host_apis: tuple[HostApiInfo, ...],
    devices: list[DeviceInfo],
) -> None:
    """Configure sounddevice mock with host APIs and devices."""
    mock_sd.query_hostapis.return_value = host_apis
    if len(devices) == 1:
        mock_sd.query_devices.return_value = devices[0]
    else:
        mock_sd.query_devices.side_effect = devices


@patch("modal_dictation_exploration.audio.sd")
def test_list_input_devices_returns_audio_device_instances_with_correct_fields(mock_sd):
    host_api = HostApiInfoFactory.build(name="PulseAudio", devices=[0])
    device_info = DeviceInfoFactory.build(index=0, max_input_channels=2)
    setup_sounddevice_mock(mock_sd, (host_api,), [device_info])

    result = list_input_devices()

    assert len(result) == 1
    device = result[0]
    assert device.index == device_info["index"]
    assert device.name == device_info["name"]
    assert device.channels == device_info["max_input_channels"]
    assert device.default_sample_rate == device_info["default_samplerate"]


@patch("modal_dictation_exploration.audio.sd")
def test_list_input_devices_filters_out_output_only_devices(mock_sd):
    host_api = HostApiInfoFactory.build(name="PulseAudio", devices=[0, 1])
    input_device = DeviceInfoFactory.build(index=0, max_input_channels=2)
    output_device = DeviceInfoFactory.build(index=1, max_input_channels=0)
    setup_sounddevice_mock(mock_sd, (host_api,), [input_device, output_device])

    result = list_input_devices()

    assert len(result) == 1
    assert result[0].index == input_device["index"]


@patch("modal_dictation_exploration.audio.sd")
def test_list_input_devices_filters_out_non_pulseaudio_devices(mock_sd):
    alsa_api = HostApiInfoFactory.build(name="ALSA", devices=[0, 1])
    pulse_api = HostApiInfoFactory.build(name="PulseAudio", devices=[2, 3])
    pulse_device_1 = DeviceInfoFactory.build(index=2, max_input_channels=2)
    pulse_device_2 = DeviceInfoFactory.build(index=3, max_input_channels=2)
    setup_sounddevice_mock(
        mock_sd, (alsa_api, pulse_api), [pulse_device_1, pulse_device_2]
    )

    result = list_input_devices()

    assert len(result) == 2
    queried_indices = [
        call.kwargs["device"] for call in mock_sd.query_devices.call_args_list
    ]
    assert queried_indices == [2, 3]


@patch("modal_dictation_exploration.audio.sd")
def test_list_input_devices_returns_empty_list_when_no_devices(mock_sd):
    host_api = HostApiInfoFactory.build(name="PulseAudio", devices=[])
    setup_sounddevice_mock(mock_sd, (host_api,), [])

    result = list_input_devices()

    assert result == []


@patch("modal_dictation_exploration.audio.sd")
def test_list_input_devices_returns_empty_list_on_port_audio_error(mock_sd):
    mock_sd.query_hostapis.side_effect = mock_sd.PortAudioError("Device unavailable")

    result = list_input_devices()

    assert result == []
