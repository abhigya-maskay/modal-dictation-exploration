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

    func categorizedEntries() -> [(phrase: String, category: PhraseCategory)] {
        let sources: [([String: String], (String) -> PhraseCategory)] = [
            (modifiers, { .modifier($0) }),
            (keys, { .key($0) }),
            (actions, { .action($0) }),
        ]
        return sources.flatMap { dict, makeCategory in
            dict.map { (phrase: $0.key, category: makeCategory($0.value)) }
        }
    }
}
