"""Factories for audio module test data."""

from polyfactory.factories.pydantic_factory import ModelFactory

from modal_dictation_exploration.audio import AudioDevice


class AudioDeviceFactory(ModelFactory[AudioDevice]):
    """Factory for AudioDevice model."""
