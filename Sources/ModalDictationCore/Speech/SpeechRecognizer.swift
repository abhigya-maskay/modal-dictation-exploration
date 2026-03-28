@preconcurrency import AVFAudio
import FluidAudio

public enum SpeechRecognizerError: Error {
    case notInitialized
    case alreadyActive(String)
}

public actor SpeechRecognizer {

    public enum Mode: Sendable {
        case idle
        case dictation
        case command
    }

    private let modelStore: any ModelProviding
    public private(set) var mode: Mode = .idle

    private var dictationRecognizer: DictationRecognizer?
    private var commandRecognizer: CommandRecognizer?

    public init(modelStore: any ModelProviding) {
        self.modelStore = modelStore
    }

    public func initialize() async throws {
        try await modelStore.loadAll()
    }

    public func startDictation(
        audio: sending AsyncStream<AVAudioPCMBuffer>,
        timeout: Double = 0.3
    ) async throws -> AsyncStream<ASRResult> {
        guard mode == .idle else {
            throw SpeechRecognizerError.alreadyActive("Cannot start dictation while \(mode) is active")
        }

        let transcriber = try await modelStore.makeBatchTranscriber()
        let recognizer = DictationRecognizer(transcriber: transcriber, silenceTimeout: timeout)
        dictationRecognizer = recognizer
        mode = .dictation

        return await recognizer.start(audio: audio)
    }

    public func startCommands(
        audio: sending AsyncStream<AVAudioPCMBuffer>
    ) async throws -> AsyncStream<String> {
        guard mode == .idle else {
            throw SpeechRecognizerError.alreadyActive("Cannot start commands while \(mode) is active")
        }

        let eouManager = try await modelStore.getEOUManager()
        let recognizer = CommandRecognizer(streamingTranscriber: eouManager)
        commandRecognizer = recognizer
        mode = .command

        return await recognizer.start(audio: audio)
    }

    public func stop() async {
        switch mode {
        case .dictation:
            await dictationRecognizer?.stop()
            dictationRecognizer = nil
        case .command:
            await commandRecognizer?.stop()
            commandRecognizer = nil
        case .idle:
            break
        }
        mode = .idle
    }
}
