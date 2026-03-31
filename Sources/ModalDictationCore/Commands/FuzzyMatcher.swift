import Foundation

struct FuzzyMatcher: Sendable {

    private let soundexIndex: [String: [(word: String, category: PhraseCategory)]]
    private let allWords: [(word: String, category: PhraseCategory)]

    init(commands: CommandsConfig) {
        var index: [String: [(word: String, category: PhraseCategory)]] = [:]
        var words: [(word: String, category: PhraseCategory)] = []

        for (phrase, category) in commands.categorizedEntries() {
            let lower = phrase.lowercased()
            guard !lower.contains(" ") else { continue }
            let entry = (word: lower, category: category)
            words.append(entry)
            let code = StringDistance.soundex(lower)
            index[code, default: []].append(entry)
        }

        self.soundexIndex = index
        self.allWords = words
    }

    func match(word: String) -> PhraseCategory? {
        let lower = word.lowercased()
        let code = StringDistance.soundex(lower)

        if let candidates = soundexIndex[code] {
            return closest(to: lower, in: candidates)
        }

        // Soundex miss — fall back to brute-force levenshtein over the full vocabulary.
        return closest(to: lower, in: allWords)
    }

    private func closest(
        to input: String,
        in entries: [(word: String, category: PhraseCategory)]
    ) -> PhraseCategory? {
        var bestDistance = Int.max
        var bestEntry: (word: String, category: PhraseCategory)?
        for entry in entries {
            let distance = StringDistance.levenshtein(input, entry.word)
            if distance == 0 { return entry.category }
            if distance < bestDistance {
                bestDistance = distance
                bestEntry = entry
            }
        }
        guard let bestEntry, bestDistance <= Self.allowedDistance(for: bestEntry.word) else {
            return nil
        }
        return bestEntry.category
    }

    // Short words get stricter matching to prevent false positives (e.g. "go" ≠ "up").
    private static func allowedDistance(for candidate: String) -> Int {
        min(StringDistance.maxEditDistance, candidate.count / 2)
    }
}
