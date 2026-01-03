import logging

import sounddevice as sd
from pydantic import BaseModel

logger = logging.getLogger(__name__)


class AudioDevice(BaseModel, frozen=True):
    """Represents an audio input device.

    Attributes:
        index: Device index (primary identifier).
        name: Human-readable device name.
        channels: Maximum number of input channels.
        default_sample_rate: Default sample rate in Hz.
    """

    index: int
    name: str
    channels: int
    default_sample_rate: float


def list_input_devices() -> list[AudioDevice]:
    try:
        host_apis = sd.query_hostapis()
        pulse_api = next(
            (api for api in host_apis if api["name"] == "PulseAudio"), None
        )
        if pulse_api is None:
            logger.warning("PulseAudio backend not available")
            return []

        pulse_device_indexes = pulse_api["devices"]
        devices = [sd.query_devices(device=i) for i in pulse_device_indexes]
        input_devices = [d for d in devices if d["max_input_channels"] > 0]
        return [
            AudioDevice(
                index=d["index"],
                name=d["name"],
                channels=d["max_input_channels"],
                default_sample_rate=d["default_samplerate"],
            )
            for d in input_devices
        ]
    except sd.PortAudioError as e:
        logger.warning(f"Failed to query audio devices: {e}")
        return []
