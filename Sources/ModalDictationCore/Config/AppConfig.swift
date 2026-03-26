import Foundation

public struct MicConfig: Sendable, Equatable {
    public var deviceID: String?

    public init(deviceID: String? = nil) {
        self.deviceID = deviceID
    }
}

public struct SpeechConfig: Sendable, Equatable {
    public var timeout: Double?
    public var autoSleepMinutes: Double?

    public init(timeout: Double? = nil, autoSleepMinutes: Double? = nil) {
        self.timeout = timeout
        self.autoSleepMinutes = autoSleepMinutes
    }
}

public struct AppConfig: Sendable, Equatable {
    public var hotkeys: HotkeyConfig
    public var mic: MicConfig
    public var speech: SpeechConfig
    public var commands: CommandsConfig

    public init(hotkeys: HotkeyConfig, mic: MicConfig, speech: SpeechConfig, commands: CommandsConfig) {
        self.hotkeys = hotkeys
        self.mic = mic
        self.speech = speech
        self.commands = commands
    }
}
