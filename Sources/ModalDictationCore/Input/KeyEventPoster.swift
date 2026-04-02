import CoreGraphics

public enum KeyEventError: Error, Equatable {
    case eventCreationFailed
}

enum KeyEventPoster {

    static func post(keyCode: CGKeyCode, flags: CGEventFlags) throws {
        guard let keyDown = CGEvent(keyboardEventSource: nil, virtualKey: keyCode, keyDown: true),
              let keyUp = CGEvent(keyboardEventSource: nil, virtualKey: keyCode, keyDown: false)
        else { throw KeyEventError.eventCreationFailed }

        keyDown.flags = flags
        keyUp.flags = flags

        keyDown.post(tap: .cghidEventTap)
        keyUp.post(tap: .cghidEventTap)
    }
}
