@preconcurrency import AVFAudio

public protocol AudioCapturing: Sendable {
    func start(deviceUID: String?) async throws -> sending AsyncStream<AVAudioPCMBuffer>
    func stop() async
}
