import logging

import numpy as np
import nemo.collections.asr as nemo_asr

logger = logging.getLogger(__name__)


class Transcriber:
    def __init__(self) -> None:
        self.model = nemo_asr.models.ASRModel.from_pretrained(
            "nvidia/parakeet-tdt-0.6b-v2"
        )

    def transcribe(self, audio: np.ndarray) -> str:
        """Transcribe preprocessed audio to text.

        Args:
            audio: Preprocessed audio as a 1D float32 numpy array at 16kHz.

        Returns:
            Transcribed text.
        """
        logger.info(f"Transcribing audio: shape={audio.shape}, dtype={audio.dtype}")
        logger.info(f"Audio stats: min={audio.min():.6f}, max={audio.max():.6f}, mean={audio.mean():.6f}")
        
        result = self.model.transcribe([audio])
        
        logger.info(f"Raw result: {result}")
        logger.info(f"Result type: {type(result)}, Result[0] type: {type(result[0])}")

        return result[0].text
