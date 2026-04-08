import Testing
import FluidAudio
@testable import ModalDictationCore

@Suite("SessionRunner")
struct SessionRunnerTests {

    private static func makeSUT() -> (runner: SessionRunner, engine: MockAudioEngine, recognizer: MockSpeechRecognizer) {
        let engine = MockAudioEngine()
        let recognizer = MockSpeechRecognizer()
        let runner = SessionRunner(engine: engine, deviceUID: nil)
        return (runner, engine, recognizer)
    }

    private static func runDictation(
        runner: SessionRunner,
        engine: MockAudioEngine,
        recognizer: MockSpeechRecognizer,
        actions: [String: String] = [:],
        onResult: @escaping @Sendable (ASRResult) async throws -> Void = { _ in }
    ) async throws -> SessionEvent {
        try await runner.run(work: { audio in
            let results = try await recognizer.startDictation(audio: audio)
            return try await runner.processResults(
                results,
                triggerMatcher: VoiceTriggerMatcher(actions: actions),
                onResult: onResult
            )
        }, cleanup: {
            await recognizer.stop()
            await engine.stop()
        })
    }

    @Test
    func test_emptyStream_returnsCompleted() async throws {
        let (runner, engine, recognizer) = Self.makeSUT()

        let event = try await Self.runDictation(runner: runner, engine: engine, recognizer: recognizer)

        #expect(event == .completed)
        #expect(await recognizer.stopCalls == 1)
        #expect(await engine.stopCalls == 1)
    }

    @Test
    func test_voiceTriggerDetected_returnsTriggerEvent() async throws {
        let (runner, engine, recognizer) = Self.makeSUT()

        let (stream, continuation) = AsyncStream.makeStream(of: ASRResult.self)
        await recognizer.setResultStream(stream)
        continuation.yield(.stub(text: "go to sleep"))
        continuation.finish()

        let event = try await Self.runDictation(
            runner: runner, engine: engine, recognizer: recognizer,
            actions: ["go to sleep": "app:sleep"]
        )

        #expect(event == .voiceTrigger(.sleep))
        #expect(await recognizer.stopCalls == 1)
        #expect(await engine.stopCalls == 1)
    }

    @Test
    func test_engineStartThrows_propagatesError() async throws {
        let (runner, engine, recognizer) = Self.makeSUT()
        await engine.setStartError(AudioDeviceError.deviceNotFound("test"))

        await #expect(throws: AudioDeviceError.self) {
            try await Self.runDictation(runner: runner, engine: engine, recognizer: recognizer)
        }

        #expect(await recognizer.stopCalls == 0)
        #expect(await engine.stopCalls == 0)
    }
}
