@preconcurrency import AVFAudio
import FluidAudio
@testable import ModalDictationCore

extension ASRResult {
    static func stub(text: String) -> ASRResult {
        ASRResult(text: text, confidence: 1, duration: 1, processingTime: 0)
    }
}

actor MockAudioEngine: AudioCapturing {
    var startError: (any Error)?
    private(set) var startCalls = 0
    private(set) var stopCalls = 0

    func setStartError(_ error: any Error) { startError = error }

    func start(deviceUID: String?) async throws -> sending AsyncStream<AVAudioPCMBuffer> {
        startCalls += 1
        if let startError { throw startError }
        return AsyncStream { $0.finish() }
    }

    func stop() async {
        stopCalls += 1
    }
}

actor MockSpeechRecognizer: SpeechRecognizing {
    var startError: (any Error)?
    var resultStream: AsyncStream<ASRResult> = AsyncStream { $0.finish() }
    private(set) var startCalls = 0
    private(set) var stopCalls = 0

    func setResultStream(_ stream: AsyncStream<ASRResult>) { resultStream = stream }

    func startDictation(
        audio: sending AsyncStream<AVAudioPCMBuffer>
    ) async throws -> AsyncStream<ASRResult> {
        startCalls += 1
        if let startError { throw startError }
        return resultStream
    }

    func startCommands(
        audio: sending AsyncStream<AVAudioPCMBuffer>,
        commandsConfig: CommandsConfig
    ) async throws -> AsyncStream<ASRResult> {
        startCalls += 1
        if let startError { throw startError }
        return resultStream
    }

    func stop() async {
        stopCalls += 1
    }
}

// Class (not actor) because KeystrokeEmitting is non-isolated; actor conformance would make emit() async.
final class MockKeystrokeEmitter: KeystrokeEmitting, @unchecked Sendable {
    private(set) var emittedCommands: [MatchedCommand] = []

    func emit(command: MatchedCommand) throws {
        emittedCommands.append(command)
    }
}

actor MockTextInserter: TextInserting {
    private(set) var insertedTexts: [String] = []

    func insert(_ text: String) async throws {
        insertedTexts.append(text)
    }
}
