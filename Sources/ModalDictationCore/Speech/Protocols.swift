@preconcurrency import AVFAudio
import FluidAudio

public protocol BatchTranscriber: Sendable {
    func transcribe(_ audioSamples: [Float]) async throws -> ASRResult
}

public protocol StreamingTranscriber: Sendable {
    func process(audioBuffer: AVAudioPCMBuffer) async throws -> String
    func finish() async throws -> String
    func reset() async
    func setEouCallback(_ callback: @escaping @Sendable (String) -> Void) async
}

public protocol ModelProviding: Sendable {
    func loadAll() async throws
    func makeBatchTranscriber() async throws -> any BatchTranscriber
    func getEOUManager() async throws -> StreamingEouAsrManager
}

extension ModelStore: ModelProviding {
    public func makeBatchTranscriber() async throws -> any BatchTranscriber {
        let models = try getTDTModels()
        let manager = AsrManager()
        try await manager.initialize(models: models)
        return manager
    }
}

extension AsrManager: BatchTranscriber {
    public func transcribe(_ audioSamples: [Float]) async throws -> ASRResult {
        try await self.transcribe(audioSamples, source: .microphone)
    }
}

extension StreamingEouAsrManager: StreamingTranscriber {}
