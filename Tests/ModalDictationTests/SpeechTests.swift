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

    @Test func test_getCTCModels_throwsWhenNotLoaded() async {
        let store = ModelStore()
        await #expect(throws: ModelStoreError.self) {
            try await store.getCTCModels()
        }
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
        let (recognizer, _) = await makeSpeechRecognizer()

        let (commandAudio, _) = AsyncStream.makeStream(of: AVAudioPCMBuffer.self)
        _ = try await recognizer.startCommands(audio: commandAudio, commandsConfig: CommandsConfig())

        await #expect(throws: SpeechRecognizerError.self) {
            let (audio, _) = AsyncStream.makeStream(of: AVAudioPCMBuffer.self)
            try await recognizer.startDictation(audio: audio)
        }
    }

    @Test func test_startCommands_whileDictationActive_throwsAlreadyActive() async throws {
        let (recognizer, _) = await makeSpeechRecognizer()

        let (dictationAudio, _) = AsyncStream.makeStream(of: AVAudioPCMBuffer.self)
        _ = try await recognizer.startDictation(audio: dictationAudio)

        await #expect(throws: SpeechRecognizerError.self) {
            let (audio, _) = AsyncStream.makeStream(of: AVAudioPCMBuffer.self)
            try await recognizer.startCommands(audio: audio, commandsConfig: CommandsConfig())
        }
    }

    @Test func test_stopFromDictation_resetsToIdle() async throws {
        let (recognizer, _) = await makeSpeechRecognizer()

        let (audio, _) = AsyncStream.makeStream(of: AVAudioPCMBuffer.self)
        _ = try await recognizer.startDictation(audio: audio)

        await recognizer.stop()

        let mode = await recognizer.mode
        #expect(mode == .idle)
    }
}

@Suite("HybridRecognizer")
struct HybridRecognizerTests {

    @Test func test_eouCallback_triggersBatchTranscription() async throws {
        let streaming = MockStreamingTranscriber()
        let batch = MockBatchTranscriber()
        let expected = ASRResult(text: "hello", confidence: 0.9, duration: 1.0, processingTime: 0.1)
        await batch.setResult(expected)

        let recognizer = HybridRecognizer(streamingTranscriber: streaming, batchTranscriber: batch)

        let (audioStream, audioContinuation) = AsyncStream.makeStream(of: AVAudioPCMBuffer.self)
        let results = await recognizer.start(audio: audioStream)

        var processIter = streaming.processedStream.makeAsyncIterator()
        audioContinuation.yield(makeBuffer(sampleCount: 17000, amplitude: 0.5))
        await processIter.next()
        streaming.triggerEou("utterance ended")
        audioContinuation.finish()

        var texts: [String] = []
        for await result in results { texts.append(result.text) }

        let calls = await batch.calls
        #expect(calls.count >= 1)
        #expect(texts.contains("hello"))
    }

    @Test func test_shortAudio_skipsTranscription() async throws {
        let streaming = MockStreamingTranscriber()
        let batch = MockBatchTranscriber()
        let recognizer = HybridRecognizer(streamingTranscriber: streaming, batchTranscriber: batch)

        let (audioStream, audioContinuation) = AsyncStream.makeStream(of: AVAudioPCMBuffer.self)
        let results = await recognizer.start(audio: audioStream)

        audioContinuation.yield(makeBuffer(sampleCount: 8000, amplitude: 0.5))
        audioContinuation.finish()

        var texts: [String] = []
        for await result in results { texts.append(result.text) }

        let calls = await batch.calls
        #expect(calls.isEmpty)
        #expect(texts.isEmpty)
    }

    @Test func test_stop_flushesAndFinishes() async throws {
        let streaming = MockStreamingTranscriber()
        let batch = MockBatchTranscriber()
        let expected = ASRResult(text: "flushed", confidence: 0.9, duration: 1.0, processingTime: 0.1)
        await batch.setResult(expected)

        let recognizer = HybridRecognizer(streamingTranscriber: streaming, batchTranscriber: batch)

        let (audioStream, audioContinuation) = AsyncStream.makeStream(of: AVAudioPCMBuffer.self)
        let results = await recognizer.start(audio: audioStream)

        var processIter = streaming.processedStream.makeAsyncIterator()
        audioContinuation.yield(makeBuffer(sampleCount: 17000, amplitude: 0.5))
        await processIter.next()
        await recognizer.stop()

        var texts: [String] = []
        for await result in results { texts.append(result.text) }

        let calls = await batch.calls
        #expect(calls.count == 1)
        #expect(texts == ["flushed"])
        #expect(streaming.finishCalls == 1)
    }
}

@Suite("VocabularyBuilder")
struct VocabularyBuilderTests {

    @Test func test_build_collectsTermsFromAllDictionaries() {
        let config = CommandsConfig(
            actions: ["select all": "selectAll", "copy": "copy"],
            modifiers: ["shift": "shift"],
            keys: ["enter": "return", "tab": "tab"]
        )

        let context = VocabularyBuilder.build(from: config)
        let terms = context.terms.map(\.text)

        #expect(terms == ["copy", "enter", "select all", "shift", "tab"])
    }

    @Test func test_build_deduplicatesAcrossDictionaries() {
        let config = CommandsConfig(
            actions: ["shift": "shiftAction"],
            modifiers: ["shift": "shiftModifier"]
        )

        let context = VocabularyBuilder.build(from: config)

        #expect(context.terms.count == 1)
        #expect(context.terms.first?.text == "shift")
    }
}
