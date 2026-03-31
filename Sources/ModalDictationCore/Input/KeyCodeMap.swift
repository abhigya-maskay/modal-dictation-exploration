import Carbon.HIToolbox
import CoreGraphics

public struct KeyMapping: Sendable, Equatable {
    public let keyCode: CGKeyCode
    public let shift: Bool

    public init(_ carbonKeyCode: Int, shift: Bool = false) {
        self.keyCode = CGKeyCode(carbonKeyCode)
        self.shift = shift
    }
}

public enum KeyCodeMap {

    public static func mapping(for name: String) -> KeyMapping? {
        table[name]
    }

    static let table: [String: KeyMapping] = [
        "a": KeyMapping(kVK_ANSI_A),
        "b": KeyMapping(kVK_ANSI_B),
        "c": KeyMapping(kVK_ANSI_C),
        "d": KeyMapping(kVK_ANSI_D),
        "e": KeyMapping(kVK_ANSI_E),
        "f": KeyMapping(kVK_ANSI_F),
        "g": KeyMapping(kVK_ANSI_G),
        "h": KeyMapping(kVK_ANSI_H),
        "i": KeyMapping(kVK_ANSI_I),
        "j": KeyMapping(kVK_ANSI_J),
        "k": KeyMapping(kVK_ANSI_K),
        "l": KeyMapping(kVK_ANSI_L),
        "m": KeyMapping(kVK_ANSI_M),
        "n": KeyMapping(kVK_ANSI_N),
        "o": KeyMapping(kVK_ANSI_O),
        "p": KeyMapping(kVK_ANSI_P),
        "q": KeyMapping(kVK_ANSI_Q),
        "r": KeyMapping(kVK_ANSI_R),
        "s": KeyMapping(kVK_ANSI_S),
        "t": KeyMapping(kVK_ANSI_T),
        "u": KeyMapping(kVK_ANSI_U),
        "v": KeyMapping(kVK_ANSI_V),
        "w": KeyMapping(kVK_ANSI_W),
        "x": KeyMapping(kVK_ANSI_X),
        "y": KeyMapping(kVK_ANSI_Y),
        "z": KeyMapping(kVK_ANSI_Z),

        "0": KeyMapping(kVK_ANSI_0),
        "1": KeyMapping(kVK_ANSI_1),
        "2": KeyMapping(kVK_ANSI_2),
        "3": KeyMapping(kVK_ANSI_3),
        "4": KeyMapping(kVK_ANSI_4),
        "5": KeyMapping(kVK_ANSI_5),
        "6": KeyMapping(kVK_ANSI_6),
        "7": KeyMapping(kVK_ANSI_7),
        "8": KeyMapping(kVK_ANSI_8),
        "9": KeyMapping(kVK_ANSI_9),

        "up": KeyMapping(kVK_UpArrow),
        "down": KeyMapping(kVK_DownArrow),
        "left": KeyMapping(kVK_LeftArrow),
        "right": KeyMapping(kVK_RightArrow),

        "f1": KeyMapping(kVK_F1),
        "f2": KeyMapping(kVK_F2),
        "f3": KeyMapping(kVK_F3),
        "f4": KeyMapping(kVK_F4),
        "f5": KeyMapping(kVK_F5),
        "f6": KeyMapping(kVK_F6),
        "f7": KeyMapping(kVK_F7),
        "f8": KeyMapping(kVK_F8),
        "f9": KeyMapping(kVK_F9),
        "f10": KeyMapping(kVK_F10),
        "f11": KeyMapping(kVK_F11),
        "f12": KeyMapping(kVK_F12),
        "f13": KeyMapping(kVK_F13),
        "f14": KeyMapping(kVK_F14),
        "f15": KeyMapping(kVK_F15),
        "f16": KeyMapping(kVK_F16),
        "f17": KeyMapping(kVK_F17),
        "f18": KeyMapping(kVK_F18),
        "f19": KeyMapping(kVK_F19),
        "f20": KeyMapping(kVK_F20),

        "return": KeyMapping(kVK_Return),
        "enter": KeyMapping(kVK_ANSI_KeypadEnter),
        "escape": KeyMapping(kVK_Escape),
        "tab": KeyMapping(kVK_Tab),
        "space": KeyMapping(kVK_Space),
        "backspace": KeyMapping(kVK_Delete),
        "delete": KeyMapping(kVK_ForwardDelete),
        "pageup": KeyMapping(kVK_PageUp),
        "pagedown": KeyMapping(kVK_PageDown),
        "home": KeyMapping(kVK_Home),
        "end": KeyMapping(kVK_End),
        "help": KeyMapping(kVK_Help),
        "minus": KeyMapping(kVK_ANSI_Minus),

        "`": KeyMapping(kVK_ANSI_Grave),
        ",": KeyMapping(kVK_ANSI_Comma),
        ".": KeyMapping(kVK_ANSI_Period),
        ";": KeyMapping(kVK_ANSI_Semicolon),
        "'": KeyMapping(kVK_ANSI_Quote),
        "/": KeyMapping(kVK_ANSI_Slash),
        "\\": KeyMapping(kVK_ANSI_Backslash),
        "[": KeyMapping(kVK_ANSI_LeftBracket),
        "]": KeyMapping(kVK_ANSI_RightBracket),
        "-": KeyMapping(kVK_ANSI_Minus),
        "=": KeyMapping(kVK_ANSI_Equal),

        "~": KeyMapping(kVK_ANSI_Grave, shift: true),
        "!": KeyMapping(kVK_ANSI_1, shift: true),
        "@": KeyMapping(kVK_ANSI_2, shift: true),
        "#": KeyMapping(kVK_ANSI_3, shift: true),
        "$": KeyMapping(kVK_ANSI_4, shift: true),
        "%": KeyMapping(kVK_ANSI_5, shift: true),
        "^": KeyMapping(kVK_ANSI_6, shift: true),
        "&": KeyMapping(kVK_ANSI_7, shift: true),
        "*": KeyMapping(kVK_ANSI_8, shift: true),
        "(": KeyMapping(kVK_ANSI_9, shift: true),
        ")": KeyMapping(kVK_ANSI_0, shift: true),
        "_": KeyMapping(kVK_ANSI_Minus, shift: true),
        "+": KeyMapping(kVK_ANSI_Equal, shift: true),
        "{": KeyMapping(kVK_ANSI_LeftBracket, shift: true),
        "}": KeyMapping(kVK_ANSI_RightBracket, shift: true),
        "|": KeyMapping(kVK_ANSI_Backslash, shift: true),
        ":": KeyMapping(kVK_ANSI_Semicolon, shift: true),
        "\"": KeyMapping(kVK_ANSI_Quote, shift: true),
        "<": KeyMapping(kVK_ANSI_Comma, shift: true),
        ">": KeyMapping(kVK_ANSI_Period, shift: true),
        "?": KeyMapping(kVK_ANSI_Slash, shift: true),
    ]
}
