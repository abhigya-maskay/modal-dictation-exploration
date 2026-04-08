public enum HotkeyRole: Sendable, Equatable {
    case dictationHold
    case sleepToggle
}

public enum VoiceTrigger: String, Sendable, Equatable, CaseIterable {
    case sleep = "app:sleep"
    case wakeUp = "app:wake"
    case dictationMode = "mode:dictation"
    case commandMode = "mode:command"
}

public enum ModeEvent: Sendable, Equatable {
    case hotkeyPress(HotkeyRole)
    case hotkeyRelease(HotkeyRole)
    case voiceTrigger(VoiceTrigger)
    case autoSleepFired
}
