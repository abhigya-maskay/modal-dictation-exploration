import CoreGraphics

public enum EmitterError: Error, Equatable {
    case unknownKey(String)
}

public enum KeystrokeEmitter {

    public static func emit(
        key: String,
        modifiers: [String] = [],
        repeat count: Int = 1
    ) throws {
        guard let mapping = KeyCodeMap.mapping(for: key) else {
            throw EmitterError.unknownKey(key)
        }

        var flags = ModifierMap.combinedFlags(for: modifiers)
        if mapping.shift {
            flags.insert(.maskShift)
        }

        for _ in 0..<count {
            try KeyEventPoster.post(keyCode: mapping.keyCode, flags: flags)
        }
    }

    public static func emit(command: MatchedCommand) throws {
        switch command {
        case let .keystroke(key, modifiers, count):
            try emit(key: key, modifiers: modifiers, repeat: count)
        case .action:
            break
        }
    }
}

public struct LiveKeystrokeEmitter: KeystrokeEmitting {
    public init() {}

    public func emit(command: MatchedCommand) throws {
        try KeystrokeEmitter.emit(command: command)
    }
}
