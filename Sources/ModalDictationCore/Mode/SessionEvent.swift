public enum SessionEvent: Sendable, Equatable {
    case completed
    case voiceTrigger(VoiceTrigger)
}
