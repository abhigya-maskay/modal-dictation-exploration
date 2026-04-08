import Testing
import FluidAudio
@testable import ModalDictationCore

@Suite("DictationSession")
struct DictationSessionTests {

    private static func makeSUT(
        actions: [String: String] = [:]
    ) -> (session: DictationSession, engine: MockAudioEngine, recognizer: MockSpeechRecognizer, inserter: MockTextInserter) {
        let engine = MockAudioEngine()
        let recognizer = MockSpeechRecognizer()
        let inserter = MockTextInserter()
        let session = DictationSession(
            engine: engine,
            recognizer: recognizer,
            inserter: inserter,
            actions: actions
        )
        return (session, engine, recognizer, inserter)
    }

    @Test
    func test_nonTriggerResults_insertText() async throws {
        let (session, _, recognizer, inserter) = Self.makeSUT()

        let (stream, continuation) = AsyncStream.makeStream(of: ASRResult.self)
        await recognizer.setResultStream(stream)
        continuation.yield(.stub(text: "hello"))
        continuation.yield(.stub(text: "world"))
        continuation.finish()

        let event = try await session.run()
        #expect(event == .completed)
        #expect(await inserter.insertedTexts == ["hello", "world"])
    }

}
