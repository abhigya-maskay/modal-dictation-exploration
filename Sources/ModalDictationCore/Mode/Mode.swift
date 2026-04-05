public enum Mode: Sendable, Equatable {
    case sleeping
    case dictation
    case command

    internal var startEffect: SideEffect? {
        switch self {
        case .dictation: .startDictation
        case .command: .startCommands
        case .sleeping: nil
        }
    }

    internal var stopEffect: SideEffect? {
        switch self {
        case .dictation: .stopDictation
        case .command: .stopCommands
        case .sleeping: nil
        }
    }
}
