@preconcurrency import AVFAudio
import FluidAudio

struct SessionRunner {
    let engine: any AudioCapturing
    let deviceUID: String?

    func run(
        work: (sending AsyncStream<AVAudioPCMBuffer>) async throws -> SessionEvent,
        cleanup: @Sendable () async -> Void
    ) async throws -> SessionEvent {
        let audio = try await engine.start(deviceUID: deviceUID)
        do {
            let event = try await work(audio)
            await cleanup()
            return event
        } catch {
            await cleanup()
            throw error
        }
    }

    func processResults(
        _ results: AsyncStream<ASRResult>,
        triggerMatcher: VoiceTriggerMatcher,
        onResult: (ASRResult) async throws -> Void
    ) async throws -> SessionEvent {
        for await result in results {
            if let trigger = triggerMatcher.match(result.text) {
                return .voiceTrigger(trigger)
            }
            try await onResult(result)
        }
        return .completed
    }
}
