import Foundation

public enum MatchedCommand: Sendable, Equatable {
    case keystroke(key: String, modifiers: [String], repeat: Int)
    case action(String)
}

public enum PhraseCategory: Sendable, Equatable {
    case modifier(String)
    case key(String)
    case action(String)
}
