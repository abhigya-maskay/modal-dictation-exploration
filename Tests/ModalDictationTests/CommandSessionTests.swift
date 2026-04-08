import Testing
import FluidAudio
@testable import ModalDictationCore

@Suite("CommandSession")
struct CommandSessionTests {

    private static func makeSUT(
        commandsConfig: CommandsConfig = CommandsConfig()
    ) -> (session: CommandSession, engine: MockAudioEngine, recognizer: MockSpeechRecognizer, emitter: MockKeystrokeEmitter) {
        let engine = MockAudioEngine()
        let recognizer = MockSpeechRecognizer()
        let emitter = MockKeystrokeEmitter()
        let session = CommandSession(
            engine: engine,
            recognizer: recognizer,
            matcher: CommandMatcher(commands: commandsConfig),
            emitter: emitter,
            commandsConfig: commandsConfig
        )
        return (session, engine, recognizer, emitter)
    }

    @Test
    func test_nonTriggerResults_emitMatchedCommands() async throws {
        let config = CommandsConfig(keys: ["alpha": "a"])
        let (session, _, recognizer, emitter) = Self.makeSUT(commandsConfig: config)

        let (stream, continuation) = AsyncStream.makeStream(of: ASRResult.self)
        await recognizer.setResultStream(stream)
        continuation.yield(.stub(text: "alpha"))
        continuation.finish()

        let event = try await session.run()
        #expect(event == .completed)
        #expect(emitter.emittedCommands == [.keystroke(key: "a", modifiers: [], repeat: 1)])
    }

}
