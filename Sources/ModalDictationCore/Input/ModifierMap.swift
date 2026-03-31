import CoreGraphics

public enum ModifierMap {

    public static func flags(for name: String) -> CGEventFlags? {
        table[name]
    }

    public static func combinedFlags(for names: [String]) -> CGEventFlags {
        var result = CGEventFlags()
        for name in names {
            if let flag = table[name] {
                result.insert(flag)
            }
        }
        return result
    }

    static let table: [String: CGEventFlags] = [
        "cmd": .maskCommand,
        "ctrl": .maskControl,
        "alt": .maskAlternate,
        "shift": .maskShift,
        "fn": .maskSecondaryFn,
    ]
}
