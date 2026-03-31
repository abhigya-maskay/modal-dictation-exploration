import FluidAudio

extension AsrManager: BatchTranscriber {
    public func transcribe(_ audioSamples: [Float]) async throws -> ASRResult {
        try await self.transcribe(audioSamples, source: .microphone)
    }
}

extension StreamingEouAsrManager: StreamingTranscriber {}
