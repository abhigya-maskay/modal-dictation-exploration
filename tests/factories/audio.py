"""Factories for audio module test data."""

import random

import pulsectl
from polyfactory.factories.pydantic_factory import ModelFactory
from pulsectl.pulsectl import EnumValue

from modal_dictation_exploration.audio import AudioDevice

PULSE_STATES = [
    pulsectl.PulseStateEnum.idle,
    pulsectl.PulseStateEnum.running,
    pulsectl.PulseStateEnum.suspended,
]


class AudioDeviceFactory(ModelFactory[AudioDevice]):
    """Factory for AudioDevice model."""

    @classmethod
    def state(cls) -> EnumValue:
        return random.choice(PULSE_STATES)
