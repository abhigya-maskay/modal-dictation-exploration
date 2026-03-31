import FluidAudio

public enum VocabularyBuilder {
    public static func build(from config: CommandsConfig) -> CustomVocabularyContext {
        let spokenForms = Set(config.categorizedEntries().map(\.phrase))
        let terms = spokenForms.sorted().map { CustomVocabularyTerm(text: $0) }
        return CustomVocabularyContext(terms: terms)
    }
}
