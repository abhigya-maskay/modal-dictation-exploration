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
    func makeBatchTranscriber(
        commandsConfig: CommandsConfig?
    ) async throws -> any BatchTranscriber
    func getEOUManager() async throws -> any StreamingTranscriber
}

