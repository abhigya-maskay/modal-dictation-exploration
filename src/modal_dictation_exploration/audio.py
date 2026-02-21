import logging
import time

import librosa
import numpy as np
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


def record_audio(
    device_index: int,
    duration: float = 5.0,
    sample_rate: float | None = None,
) -> tuple[np.ndarray, float]:
    """Record audio from the specified device.

    Args:
        device_index: Index of the audio device to record from.
        duration: Recording duration in seconds (default: 5.0).
        sample_rate: Sample rate in Hz. If None, uses device's default.

    Returns:
        Tuple of (audio_data, actual_sample_rate) where audio_data is a numpy array.
    """
    chunks: list[np.ndarray] = []

    if sample_rate is None:
        device_info = sd.query_devices(device=device_index)
        sample_rate = device_info["default_samplerate"]

    def audio_callback(indata, frames, time, status):
        chunks.append(indata.copy())

    logger.info("Opening InputStream...")
    stream = sd.InputStream(
        samplerate=sample_rate,
        device=device_index,
        channels=1,
        dtype=np.float32,
        callback=audio_callback,
    )
    logger.info("Starting stream...")
    stream.start()
    logger.info("InputStream started, sleeping...")
    time.sleep(duration)
    logger.info("Sleep complete")
    logger.info(f"Chunks so far: {len(chunks)}")
    logger.info("Stopping stream...")
    stream.stop()
    logger.info("Stream stopped")
    logger.info(f"Collected {len(chunks)} chunks")
    audio_data = np.concatenate(chunks)

    return audio_data, sample_rate


def prepare_audio_for_transcription(
    audio_data: np.ndarray,
    sample_rate: float,
) -> np.ndarray:
    """Prepare recorded audio for transcription.

    Args:
        audio_data: Raw audio data as a numpy array.
        sample_rate: Sample rate of the audio in Hz.

    Returns:
        Processed audio as a 1D float32 numpy array at 16kHz.
    """
    if audio_data.ndim > 1 and audio_data.shape[1] > 1:
        mono_audio = np.mean(audio_data, axis=1)
    else:
        mono_audio = audio_data.flatten()

    target_sample_rate = 16000
    if sample_rate != target_sample_rate:
        mono_audio = librosa.resample(
            mono_audio,
            orig_sr=sample_rate,
            target_sr=target_sample_rate,
        )

    return mono_audio.astype(np.float32)
