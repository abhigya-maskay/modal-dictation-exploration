"""Factories for sounddevice test data."""

from polyfactory.factories.typed_dict_factory import TypedDictFactory

from modal_dictation_exploration.types.sounddevice import DeviceInfo, HostApiInfo


class HostApiInfoFactory(TypedDictFactory[HostApiInfo]):
    """Factory for HostApiInfo TypedDict."""


class DeviceInfoFactory(TypedDictFactory[DeviceInfo]):
    """Factory for DeviceInfo TypedDict."""
