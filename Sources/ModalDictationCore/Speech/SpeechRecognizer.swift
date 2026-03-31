@preconcurrency import AVFAudio
import FluidAudio

public enum SpeechRecognizerError: Error {
    case alreadyActive(String)
}

public actor SpeechRecognizer {

    public enum Mode: Sendable, CustomStringConvertible {
        case idle
        case dictation
        case command

        public var description: String {
            switch self {
            case .idle: "idle"
            case .dictation: "dictation"
            case .command: "command"
            }
        }
    }

    private let modelStore: any ModelProviding
    public private(set) var mode: Mode = .idle

    private var hybridRecognizer: HybridRecognizer?

    public init(modelStore: any ModelProviding) {
        self.modelStore = modelStore
    }

    public func initialize() async throws {
        try await modelStore.loadAll()
    }

    public func startDictation(
        audio: sending AsyncStream<AVAudioPCMBuffer>
    ) async throws -> AsyncStream<ASRResult> {
        try await start(audio: audio, mode: .dictation, commandsConfig: nil)
    }

    public func startCommands(
        audio: sending AsyncStream<AVAudioPCMBuffer>,
        commandsConfig: CommandsConfig
    ) async throws -> AsyncStream<ASRResult> {
        try await start(audio: audio, mode: .command, commandsConfig: commandsConfig)
    }

    private func start(
        audio: sending AsyncStream<AVAudioPCMBuffer>,
        mode: Mode,
        commandsConfig: CommandsConfig?
    ) async throws -> AsyncStream<ASRResult> {
        guard self.mode == .idle else {
            throw SpeechRecognizerError.alreadyActive("Cannot start \(mode) while \(self.mode) is active")
        }

        let streaming = try await modelStore.getEOUManager()
        let batch = try await modelStore.makeBatchTranscriber(commandsConfig: commandsConfig)
        let recognizer = HybridRecognizer(streamingTranscriber: streaming, batchTranscriber: batch)
        hybridRecognizer = recognizer
        self.mode = mode

        return await recognizer.start(audio: audio)
    }

    public func stop() async {
        await hybridRecognizer?.stop()
        hybridRecognizer = nil
        mode = .idle
    }
}
