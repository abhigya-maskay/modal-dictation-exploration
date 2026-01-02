import logging

import pulsectl
from pulsectl.pulsectl import EnumValue
from pydantic import BaseModel

logger = logging.getLogger(__name__)

PULSE_CLIENT_NAME = "modal-dictation"


class AudioDevice(BaseModel, frozen=True, arbitrary_types_allowed=True):
    """Represents an audio input device from PulseAudio/PipeWire.

    Attributes:
        name: Stable PulseAudio source name (primary identifier for selection/recording).
        description: Human-readable name for UI display (e.g., "MOTU M2").
        index: Volatile PulseAudio index (may change between sessions).
        channels: Number of input channels.
        sample_rate: Sample rate in Hz.
        mute: Whether device is muted.
        state: Device state (running, idle, suspended).
    """

    name: str
    description: str
    index: int
    channels: int
    sample_rate: int
    mute: bool
    state: EnumValue


def list_input_devices() -> list[AudioDevice]:
    try:
        with pulsectl.Pulse(PULSE_CLIENT_NAME) as pulse:
            sources = [
                AudioDevice(
                    name=source.name,
                    description=source.description,
                    index=source.index,
                    channels=source.channel_count,
                    sample_rate=source.sample_spec.rate,
                    mute=bool(source.mute),
                    state=source.state,
                )
                for source in pulse.source_list()
                if not source.name.endswith(".monitor")
            ]
            return sources
    except pulsectl.PulseError as e:
        logger.warning(f"Failed to connect to PulseAudio: {e}")
        return []
