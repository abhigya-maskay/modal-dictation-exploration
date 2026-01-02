"""Type definitions for pulsectl library."""

from typing import TypedDict


class PulseSampleSpecDict(TypedDict):
    """Sample spec dict for PulseAudio sources."""

    format: str
    rate: int
    channels: int


class PulseSourceInfoDict(TypedDict):
    """Source info dict as returned by pulsectl."""

    index: int
    name: str
    description: str
    mute: int
    channel_count: int
    sample_spec: PulseSampleSpecDict
    state: str