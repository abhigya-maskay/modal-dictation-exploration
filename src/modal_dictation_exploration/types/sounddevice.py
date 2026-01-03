"""Type definitions for sounddevice library."""

from typing import TypedDict


class HostApiInfo(TypedDict):
    name: str
    devices: list[int]
    default_input_device: int
    default_output_device: int


class DeviceInfo(TypedDict):
    name: str
    index: int
    hostapi: int
    max_input_channels: int
    max_output_channels: int
    default_low_input_latency: float
    default_low_output_latency: float
    default_high_input_latency: float
    default_high_output_latency: float
    default_samplerate: float
