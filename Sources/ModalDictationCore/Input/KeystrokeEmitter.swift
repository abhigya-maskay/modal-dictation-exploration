import ApplicationServices
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
            guard let keyDown = CGEvent(keyboardEventSource: nil, virtualKey: mapping.keyCode, keyDown: true),
                  let keyUp = CGEvent(keyboardEventSource: nil, virtualKey: mapping.keyCode, keyDown: false)
            else { continue }

            keyDown.flags = flags
            keyUp.flags = flags

            keyDown.post(tap: .cghidEventTap)
            keyUp.post(tap: .cghidEventTap)
        }
    }

    @discardableResult
    public static func ensureAccessibility() -> Bool {
        let key = "AXTrustedCheckOptionPrompt" as CFString
        let options = [key: true] as CFDictionary
        return AXIsProcessTrustedWithOptions(options)
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
