public struct ModeStateMachine: Sendable, Equatable {
    public private(set) var state: Mode
    public private(set) var lastActiveMode: Mode
    public private(set) var modeBeforeHold: Mode?

    public init(state: Mode = .sleeping, lastActiveMode: Mode = .dictation) {
        self.state = state
        self.lastActiveMode = lastActiveMode
    }

    public mutating func handle(_ event: ModeEvent) -> ModeTransition {
        switch (state, event) {

        // Already dictating — hold has nothing to restore to, so skip setting modeBeforeHold.
        case (.dictation, .hotkeyPress(.dictationHold)):
            return ModeTransition(state: state, sideEffects: [])

        case (_, .hotkeyPress(.dictationHold)):
            return enterHold(from: state)

        case (.dictation, .hotkeyRelease(.dictationHold)):
            guard let restoreMode = modeBeforeHold else {
                return ModeTransition(state: state, sideEffects: [])
            }
            modeBeforeHold = nil
            return enter(restoreMode)

        case (_, .hotkeyRelease(.dictationHold)):
            modeBeforeHold = nil
            return ModeTransition(state: state, sideEffects: [])

        case (.sleeping, .hotkeyPress(.sleepToggle)):
            return enter(lastActiveMode)

        case (.sleeping, .voiceTrigger(.wakeUp)),
             (.sleeping, .voiceTrigger(.dictationMode)):
            return enter(.dictation)

        case (.sleeping, .voiceTrigger(.commandMode)):
            return enter(.command)

        case (_, .hotkeyPress(.sleepToggle)),
             (_, .autoSleepFired),
             (_, .voiceTrigger(.sleep)):
            return enter(.sleeping)

        case (.dictation, .voiceTrigger(.commandMode)):
            return enter(.command)

        case (.command, .voiceTrigger(.dictationMode)):
            return enter(.dictation)

        default:
            return ModeTransition(state: state, sideEffects: [])
        }
    }

    // Bypasses enter() to avoid updating lastActiveMode — hold is temporary.
    private mutating func enterHold(from priorMode: Mode) -> ModeTransition {
        modeBeforeHold = priorMode
        let effects = Self.sideEffects(from: state, to: .dictation)
        state = .dictation
        return ModeTransition(state: .dictation, sideEffects: effects)
    }

    private mutating func enter(_ newMode: Mode) -> ModeTransition {
        let effects = Self.sideEffects(from: state, to: newMode)
        modeBeforeHold = nil
        state = newMode
        if newMode != .sleeping { lastActiveMode = newMode }
        return ModeTransition(state: newMode, sideEffects: effects)
    }

    private static func sideEffects(from oldMode: Mode, to newMode: Mode) -> [SideEffect] {
        guard oldMode != newMode else { return [] }
        return [oldMode.stopEffect, newMode.startEffect].compactMap { $0 }
    }

}
