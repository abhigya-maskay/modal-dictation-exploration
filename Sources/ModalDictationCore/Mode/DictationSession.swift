@preconcurrency import AVFAudio

public struct DictationSession: Sendable {
    private let runner: SessionRunner
    private let recognizer: any SpeechRecognizing
    private let inserter: any TextInserting
    private let triggerMatcher: VoiceTriggerMatcher

    public init(
        engine: any AudioCapturing,
        recognizer: any SpeechRecognizing,
        inserter: any TextInserting = PasteboardTextInserter(),
        actions: [String: String],
        deviceUID: String? = nil
    ) {
        self.runner = SessionRunner(engine: engine, deviceUID: deviceUID)
        self.recognizer = recognizer
        self.inserter = inserter
        self.triggerMatcher = VoiceTriggerMatcher(actions: actions)
    }

    public func run() async throws -> SessionEvent {
        try await runner.run(work: dictate) {
            await recognizer.stop()
            await runner.engine.stop()
        }
    }

    private func dictate(
        audio: sending AsyncStream<AVAudioPCMBuffer>
    ) async throws -> SessionEvent {
        let results = try await recognizer.startDictation(audio: audio)

        return try await runner.processResults(
            results,
            triggerMatcher: triggerMatcher
        ) { result in
            try await inserter.insert(result.text)
        }
    }

}
