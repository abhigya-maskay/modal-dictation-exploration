@preconcurrency import AVFAudio

public struct CommandSession: Sendable {
    private let runner: SessionRunner
    private let recognizer: any SpeechRecognizing
    private let matcher: CommandMatcher
    private let emitter: any KeystrokeEmitting
    private let commandsConfig: CommandsConfig
    private let triggerMatcher: VoiceTriggerMatcher

    public init(
        engine: any AudioCapturing,
        recognizer: any SpeechRecognizing,
        matcher: CommandMatcher,
        emitter: any KeystrokeEmitting = LiveKeystrokeEmitter(),
        commandsConfig: CommandsConfig,
        deviceUID: String? = nil
    ) {
        self.runner = SessionRunner(engine: engine, deviceUID: deviceUID)
        self.recognizer = recognizer
        self.matcher = matcher
        self.emitter = emitter
        self.commandsConfig = commandsConfig
        self.triggerMatcher = VoiceTriggerMatcher(actions: commandsConfig.actions)
    }

    public func run() async throws -> SessionEvent {
        try await runner.run(work: executeCommands) {
            await recognizer.stop()
            await runner.engine.stop()
        }
    }

    private func executeCommands(
        audio: sending AsyncStream<AVAudioPCMBuffer>
    ) async throws -> SessionEvent {
        let results = try await recognizer.startCommands(
            audio: audio,
            commandsConfig: commandsConfig
        )

        return try await runner.processResults(
            results,
            triggerMatcher: triggerMatcher
        ) { result in
            for command in matcher.match(result.text) {
                try emitter.emit(command: command)
            }
        }
    }

}
