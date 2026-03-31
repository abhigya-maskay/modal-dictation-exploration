import Foundation

public struct PhraseTrie: Sendable {

    final class Node: @unchecked Sendable {
        var children: [String: Node] = [:]
        var match: PhraseCategory?
    }

    private let root = Node()

    public init(commands: CommandsConfig) {
        for (phrase, category) in commands.categorizedEntries() {
            insert(phrase: phrase, category: category)
        }
    }

    private func insert(phrase: String, category: PhraseCategory) {
        let words = phrase.lowercased().split(separator: " ").map(String.init)
        guard !words.isEmpty else { return }
        var current = root
        for word in words {
            if let child = current.children[word] {
                current = child
            } else {
                let child = Node()
                current.children[word] = child
                current = child
            }
        }
        current.match = category
    }

    public func longestMatch(in tokens: [String], startingAt index: Int) -> (length: Int, category: PhraseCategory)? {
        var current = root
        var best: (length: Int, category: PhraseCategory)?
        for i in index..<tokens.count {
            guard let child = current.children[tokens[i].lowercased()] else { break }
            current = child
            if let category = current.match {
                best = (length: i - index + 1, category: category)
            }
        }
        return best
    }
}
