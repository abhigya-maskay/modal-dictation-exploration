"""Factory for creating mock PulseSourceInfo objects."""

import random
from dataclasses import dataclass

import pulsectl
from polyfactory.factories.dataclass_factory import DataclassFactory
from pulsectl.pulsectl import EnumValue

PULSE_STATES = [
    pulsectl.PulseStateEnum.idle,
    pulsectl.PulseStateEnum.running,
    pulsectl.PulseStateEnum.suspended,
]


@dataclass
class PulseSampleSpec:
    """Mock PulseSampleSpec for testing."""

    format: str
    rate: int
    channels: int


@dataclass
class PulseSourceInfo:
    """Mock PulseSourceInfo for testing."""

    index: int
    name: str
    description: str
    mute: int
    channel_count: int
    sample_spec: PulseSampleSpec
    state: EnumValue


class PulseSourceInfoFactory(DataclassFactory[PulseSourceInfo]):
    """Factory for PulseSourceInfo dataclass."""

    @classmethod
    def state(cls) -> EnumValue:
        return random.choice(PULSE_STATES)