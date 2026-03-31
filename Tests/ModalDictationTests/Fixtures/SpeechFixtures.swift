@preconcurrency import AVFAudio
import FluidAudio
@testable import ModalDictationCore

actor MockBatchTranscriber: BatchTranscriber {
    var result: ASRResult = ASRResult(text: "", confidence: 0, duration: 0, processingTime: 0)
    var error: (any Error)?
    private(set) var calls: [[Float]] = []

    func setResult(_ result: ASRResult) { self.result = result }

    func transcribe(_ audioSamples: [Float]) async throws -> ASRResult {
        calls.append(audioSamples)
        if let error { throw error }
        return result
    }
}

final class MockStreamingTranscriber: StreamingTranscriber, @unchecked Sendable {
    var finishResult: String = ""
    private(set) var eouCallback: (@Sendable (String) -> Void)?
    private(set) var processCalls: Int = 0
    private(set) var finishCalls: Int = 0
    private(set) var resetCalls: Int = 0

    let processedStream: AsyncStream<Void>
    private let processedContinuation: AsyncStream<Void>.Continuation

    init() {
        let (stream, continuation) = AsyncStream.makeStream(of: Void.self)
        processedStream = stream
        processedContinuation = continuation
    }

    func process(audioBuffer: AVAudioPCMBuffer) async throws -> String {
        processCalls += 1
        processedContinuation.yield()
        return ""
    }
    func finish() async throws -> String { finishCalls += 1; return finishResult }
    func reset() async { resetCalls += 1 }
    func setEouCallback(_ callback: @escaping @Sendable (String) -> Void) async { eouCallback = callback }

    func triggerEou(_ text: String) { eouCallback?(text) }
}

actor MockModelProvider: ModelProviding {
    var loadAllError: (any Error)?
    private(set) var loadAllCalls = 0

    func loadAll() async throws {
        loadAllCalls += 1
        if let error = loadAllError { throw error }
    }

    private var batchTranscriber: (any BatchTranscriber)?

    func setBatchTranscriber(_ transcriber: any BatchTranscriber) { batchTranscriber = transcriber }

    func makeBatchTranscriber(
        commandsConfig: CommandsConfig? = nil
    ) async throws -> any BatchTranscriber {
        guard let batchTranscriber else { throw ModelStoreError.modelsNotLoaded("mock transcriber not set") }
        return batchTranscriber
    }

    private var eouManager: (any StreamingTranscriber)?

    func setEOUManager(_ manager: any StreamingTranscriber) { eouManager = manager }

    func getEOUManager() async throws -> any StreamingTranscriber {
        guard let eouManager else { throw ModelStoreError.modelsNotLoaded("mock EOU not loaded") }
        return eouManager
    }
}

func makeSpeechRecognizer() async -> (SpeechRecognizer, MockModelProvider) {
    let provider = MockModelProvider()
    await provider.setEOUManager(MockStreamingTranscriber())
    await provider.setBatchTranscriber(MockBatchTranscriber())
    return (SpeechRecognizer(modelStore: provider), provider)
}

func makeBuffer(sampleCount: Int, amplitude: Float = 0.0) -> AVAudioPCMBuffer {
    let format = AVAudioFormat(standardFormatWithSampleRate: 16000, channels: 1)!
    let buffer = AVAudioPCMBuffer(pcmFormat: format, frameCapacity: AVAudioFrameCount(sampleCount))!
    buffer.frameLength = AVAudioFrameCount(sampleCount)
    let data = buffer.floatChannelData![0]
    for i in 0..<sampleCount { data[i] = amplitude }
    return buffer
}
