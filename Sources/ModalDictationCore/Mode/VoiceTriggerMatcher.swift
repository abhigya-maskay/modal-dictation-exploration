public struct VoiceTriggerMatcher: Sendable {
    static let actionToTrigger: [String: VoiceTrigger] = Dictionary(
        uniqueKeysWithValues: VoiceTrigger.allCases.map { ($0.rawValue, $0) }
    )

    private let sortedPhrases: [(phrase: String, trigger: VoiceTrigger)]

    public init(actions: [String: String]) {
        sortedPhrases = actions
            .sorted { $0.key.count > $1.key.count }
            .compactMap { (phrase, action) in
                Self.actionToTrigger[action].map { (phrase.lowercased(), $0) }
            }
    }

    public func match(_ text: String) -> VoiceTrigger? {
        let normalized = text.lowercased()
        for (phrase, trigger) in sortedPhrases {
            if normalized.contains(phrase) {
                return trigger
            }
        }
        return nil
    }
}
