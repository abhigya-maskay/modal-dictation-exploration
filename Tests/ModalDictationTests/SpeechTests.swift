@preconcurrency import AVFAudio
import FluidAudio
import Testing
@testable import ModalDictationCore

@Suite("ModelStore")
struct ModelStoreTests {

    @Test func test_getTDTModels_throwsWhenNotLoaded() async {
        let store = ModelStore()
        await #expect(throws: ModelStoreError.self) {
            try await store.getTDTModels()
        }
    }

    @Test func test_getEOUManager_throwsWhenNotLoaded() async {
        let store = ModelStore()
        await #expect(throws: ModelStoreError.self) {
            try await store.getEOUManager()
        }
    }
}

@Suite("DictationRecognizer")
struct DictationRecognizerTests {

    @Test func test_transcribesOnStreamEnd() async throws {
        let mock = MockBatchTranscriber()
        await mock.setResult(ASRResult(text: "hello", confidence: 0.9, duration: 1.0, processingTime: 0.1))
        let recognizer = DictationRecognizer(transcriber: mock, silenceTimeout: 10.0)

        let (audioStream, continuation) = AsyncStream.makeStream(of: AVAudioPCMBuffer.self)
        let results = await recognizer.start(audio: audioStream)

        continuation.yield(makeBuffer(sampleCount: 17000, amplitude: 0.5))
        continuation.finish()

        var texts: [String] = []
        for await result in results { texts.append(result.text) }

        let calls = await mock.calls
        #expect(calls.count == 1)
        #expect(texts == ["hello"])
    }

    @Test func test_stopFlushesAccumulatedAudio() async throws {
        let mock = MockBatchTranscriber()
        await mock.setResult(ASRResult(text: "stopped", confidence: 0.9, duration: 1.0, processingTime: 0.1))
        let recognizer = DictationRecognizer(transcriber: mock, silenceTimeout: 10.0)

        let (audioStream, continuation) = AsyncStream.makeStream(of: AVAudioPCMBuffer.self)
        let results = await recognizer.start(audio: audioStream)

        continuation.yield(makeBuffer(sampleCount: 17000, amplitude: 0.5))
        try await Task.sleep(for: .milliseconds(50))
        await recognizer.stop()

        var texts: [String] = []
        for await result in results { texts.append(result.text) }

        let calls = await mock.calls
        #expect(calls.count == 1)
        #expect(texts == ["stopped"])
    }

    @Test func test_shortAudioSkipsTranscription() async throws {
        let mock = MockBatchTranscriber()
        let recognizer = DictationRecognizer(transcriber: mock, silenceTimeout: 10.0)

        let (audioStream, continuation) = AsyncStream.makeStream(of: AVAudioPCMBuffer.self)
        let results = await recognizer.start(audio: audioStream)

        continuation.yield(makeBuffer(sampleCount: 8000, amplitude: 0.5))
        continuation.finish()

        var texts: [String] = []
        for await result in results { texts.append(result.text) }

        let calls = await mock.calls
        #expect(calls.isEmpty)
        #expect(texts.isEmpty)
    }

    @Test func test_silenceTriggersTranscription() async throws {
        let mock = MockBatchTranscriber()
        await mock.setResult(ASRResult(text: "silence", confidence: 0.9, duration: 1.0, processingTime: 0.1))
        let recognizer = DictationRecognizer(transcriber: mock, silenceTimeout: 0.01)

        let (audioStream, continuation) = AsyncStream.makeStream(of: AVAudioPCMBuffer.self)
        _ = await recognizer.start(audio: audioStream)

        continuation.yield(makeBuffer(sampleCount: 17000, amplitude: 0.5))
        try await Task.sleep(for: .milliseconds(20))
        continuation.yield(makeBuffer(sampleCount: 160, amplitude: 0.0))
        try await Task.sleep(for: .milliseconds(50))

        let calls = await mock.calls
        #expect(calls.count == 1)
    }
}

@Suite("CommandRecognizer")
struct CommandRecognizerTests {

    @Test func test_eouCallbackYieldsText() async throws {
        let mock = MockStreamingTranscriber()
        let recognizer = CommandRecognizer(streamingTranscriber: mock)

        let (audioStream, audioContinuation) = AsyncStream.makeStream(of: AVAudioPCMBuffer.self)
        let results = await recognizer.start(audio: audioStream)

        mock.triggerEou("hello world")
        audioContinuation.finish()

        var texts: [String] = []
        for await text in results { texts.append(text) }

        #expect(texts.contains("hello world"))
    }

    @Test func test_stopYieldsFinalText() async throws {
        let mock = MockStreamingTranscriber()
        mock.finishResult = "final"
        let recognizer = CommandRecognizer(streamingTranscriber: mock)

        let (audioStream, _) = AsyncStream.makeStream(of: AVAudioPCMBuffer.self)
        let results = await recognizer.start(audio: audioStream)

        try await Task.sleep(for: .milliseconds(50))
        await recognizer.stop()

        var texts: [String] = []
        for await text in results { texts.append(text) }

        #expect(texts.contains("final"))
    }

    @Test func test_emptyEouCallbackFiltered() async throws {
        let mock = MockStreamingTranscriber()
        let recognizer = CommandRecognizer(streamingTranscriber: mock)

        let (audioStream, audioContinuation) = AsyncStream.makeStream(of: AVAudioPCMBuffer.self)
        let results = await recognizer.start(audio: audioStream)

        mock.triggerEou("")
        audioContinuation.finish()

        var texts: [String] = []
        for await text in results { texts.append(text) }

        #expect(texts.isEmpty)
    }

    @Test func test_resetDelegates() async {
        let mock = MockStreamingTranscriber()
        let recognizer = CommandRecognizer(streamingTranscriber: mock)

        await recognizer.reset()

        #expect(mock.resetCalls == 1)
    }
}

@Suite("SpeechRecognizer")
struct SpeechRecognizerTests {

    @Test func test_stopWhileIdle_remainsIdle() async {
        let recognizer = SpeechRecognizer(modelStore: MockModelProvider())
        await recognizer.stop()
        let mode = await recognizer.mode
        #expect(mode == .idle)
    }

    @Test func test_startDictation_whileCommandActive_throwsAlreadyActive() async throws {
        let mock = MockModelProvider()
        await mock.setEOUManager(StreamingEouAsrManager(chunkSize: .ms320))
        let recognizer = SpeechRecognizer(modelStore: mock)

        let (commandAudio, _) = AsyncStream.makeStream(of: AVAudioPCMBuffer.self)
        _ = try await recognizer.startCommands(audio: commandAudio)

        await #expect(throws: SpeechRecognizerError.self) {
            let (audio, _) = AsyncStream.makeStream(of: AVAudioPCMBuffer.self)
            try await recognizer.startDictation(audio: audio)
        }
    }

    @Test func test_startCommands_whileDictationActive_throwsAlreadyActive() async throws {
        let mock = MockModelProvider()
        await mock.setBatchTranscriber(MockBatchTranscriber())
        let recognizer = SpeechRecognizer(modelStore: mock)

        let (dictationAudio, _) = AsyncStream.makeStream(of: AVAudioPCMBuffer.self)
        _ = try await recognizer.startDictation(audio: dictationAudio)

        await #expect(throws: SpeechRecognizerError.self) {
            let (audio, _) = AsyncStream.makeStream(of: AVAudioPCMBuffer.self)
            try await recognizer.startCommands(audio: audio)
        }
    }

    @Test func test_stopFromDictation_resetsToIdle() async throws {
        let mock = MockModelProvider()
        await mock.setBatchTranscriber(MockBatchTranscriber())
        let recognizer = SpeechRecognizer(modelStore: mock)

        let (audio, _) = AsyncStream.makeStream(of: AVAudioPCMBuffer.self)
        _ = try await recognizer.startDictation(audio: audio)

        await recognizer.stop()

        let mode = await recognizer.mode
        #expect(mode == .idle)
    }
}
