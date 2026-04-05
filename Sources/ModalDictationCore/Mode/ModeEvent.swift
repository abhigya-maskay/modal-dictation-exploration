public enum HotkeyRole: Sendable, Equatable {
    case dictationHold
    case sleepToggle
}

public enum VoiceTrigger: Sendable, Equatable {
    case sleep
    case wakeUp
    case dictationMode
    case commandMode
}

public enum ModeEvent: Sendable, Equatable {
    case hotkeyPress(HotkeyRole)
    case hotkeyRelease(HotkeyRole)
    case voiceTrigger(VoiceTrigger)
    case autoSleepFired
}
