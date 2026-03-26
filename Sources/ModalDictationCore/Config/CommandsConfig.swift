import Foundation

public struct CommandsConfig: Sendable, Equatable {
    public var actions: [String: String]
    public var modifiers: [String: String]
    public var keys: [String: String]

    public init(actions: [String: String] = [:], modifiers: [String: String] = [:], keys: [String: String] = [:]) {
        self.actions = actions
        self.modifiers = modifiers
        self.keys = keys
    }
}
