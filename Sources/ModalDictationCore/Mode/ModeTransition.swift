public enum SideEffect: Sendable, Equatable {
    case startDictation
    case stopDictation
    case startCommands
    case stopCommands
}

public struct ModeTransition: Sendable, Equatable {
    public let state: Mode
    public let sideEffects: [SideEffect]
}
