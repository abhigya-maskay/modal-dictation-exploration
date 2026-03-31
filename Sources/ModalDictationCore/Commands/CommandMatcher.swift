import Foundation

public struct CommandMatcher: Sendable {

    private let trie: PhraseTrie
    private let fuzzy: FuzzyMatcher

    public init(commands: CommandsConfig) {
        self.trie = PhraseTrie(commands: commands)
        self.fuzzy = FuzzyMatcher(commands: commands)
    }

    public func match(_ transcription: String) -> [MatchedCommand] {
        let tokens = transcription.split(separator: " ").map(String.init)
        guard !tokens.isEmpty else { return [] }
        return parse(tokens: tokens)
    }

    private func parse(tokens: [String]) -> [MatchedCommand] {
        var results: [MatchedCommand] = []
        var modifiers: [String] = []
        var index = 0

        while index < tokens.count {
            let match: (length: Int, category: PhraseCategory)
            if let exact = trie.longestMatch(in: tokens, startingAt: index) {
                match = exact
            } else if let fuzzyCategory = fuzzy.match(word: tokens[index]) {
                match = (length: 1, category: fuzzyCategory)
            } else {
                index += 1
                continue
            }
            let (length, category) = match

            switch category {
            case .modifier(let value):
                modifiers.append(value)
                index += length

            case .key(let value):
                index += length
                let repeatCount = parseRepeatSuffix(tokens: tokens, index: &index)
                results.append(.keystroke(key: value, modifiers: modifiers, repeat: repeatCount))
                modifiers = []

            case .action(let value):
                results.append(.action(value))
                // Actions are self-contained — discard any preceding modifiers.
                modifiers = []
                index += length
            }
        }

        return results
    }

    private func parseRepeatSuffix(tokens: [String], index: inout Int) -> Int {
        guard let (number, consumed) = NumberParser.parse(from: tokens, startingAt: index) else {
            return 1
        }
        let timesIndex = index + consumed
        guard timesIndex < tokens.count, tokens[timesIndex].lowercased() == "times" else {
            return 1
        }
        index = timesIndex + 1
        return number
    }

}
