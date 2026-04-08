public enum HotkeyAction: Sendable, Equatable {
    case press
    case release
}

public struct HotkeyEvent: Sendable, Equatable {
    public let action: HotkeyAction
    public let input: HotkeyInput
}

@MainActor
public protocol HotkeyListening: Sendable {
    func start(config: HotkeyConfig) throws -> AsyncStream<HotkeyEvent>
    func stop()
}
