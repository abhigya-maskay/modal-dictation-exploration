import Testing
@testable import ModalDictationCore

@Suite("ModeStateMachine")
struct ModeStateMachineTests {

    @Test
    func test_sleepingDictationHoldPress_entersDictation() {
        var sut = ModeStateMachine(lastActiveMode: .command)
        let transition = sut.handle(.hotkeyPress(.dictationHold))

        #expect(transition.state == .dictation)
        #expect(transition.sideEffects == [.startDictation])
        #expect(sut.lastActiveMode == .command)
    }

    @Test
    func test_sleepingSleepToggle_resumesLastActiveMode() {
        var sut = ModeStateMachine(lastActiveMode: .command)
        let transition = sut.handle(.hotkeyPress(.sleepToggle))

        #expect(transition.state == .command)
        #expect(transition.sideEffects == [.startCommands])
    }

    @Test
    func test_sleepingVoiceCommandMode_entersCommand() {
        var sut = ModeStateMachine()
        let transition = sut.handle(.voiceTrigger(.commandMode))

        #expect(transition.state == .command)
        #expect(transition.sideEffects == [.startCommands])
        #expect(sut.lastActiveMode == .command)
    }

    @Test
    func test_dictationHoldRelease_returnsSleeping() {
        var sut = ModeStateMachine()
        _ = sut.handle(.hotkeyPress(.dictationHold))
        let transition = sut.handle(.hotkeyRelease(.dictationHold))

        #expect(transition.state == .sleeping)
        #expect(transition.sideEffects == [.stopDictation])
        #expect(sut.modeBeforeHold == nil)
    }

    @Test
    func test_dictationVoiceCommandMode_switchesToCommand() {
        var sut = ModeStateMachine(state: .dictation)
        let transition = sut.handle(.voiceTrigger(.commandMode))

        #expect(transition.state == .command)
        #expect(transition.sideEffects == [.stopDictation, .startCommands])
        #expect(sut.lastActiveMode == .command)
    }

    @Test
    func test_commandSleepToggle_returnsSleeping() {
        var sut = ModeStateMachine(state: .command, lastActiveMode: .command)
        let transition = sut.handle(.hotkeyPress(.sleepToggle))

        #expect(transition.state == .sleeping)
        #expect(transition.sideEffects == [.stopCommands])
    }

    @Test
    func test_commandVoiceDictationMode_switchesToDictation() {
        var sut = ModeStateMachine(state: .command, lastActiveMode: .command)
        let transition = sut.handle(.voiceTrigger(.dictationMode))

        #expect(transition.state == .dictation)
        #expect(transition.sideEffects == [.stopCommands, .startDictation])
        #expect(sut.lastActiveMode == .dictation)
    }

    @Test
    func test_unmatchedEvent_noOp() {
        var sut = ModeStateMachine()
        let transition = sut.handle(.hotkeyRelease(.dictationHold))

        #expect(transition.state == .sleeping)
        #expect(transition.sideEffects == [])
    }

    @Test
    func test_holdPressFromCommand_entersDictation() {
        var sut = ModeStateMachine(state: .command, lastActiveMode: .command)
        let transition = sut.handle(.hotkeyPress(.dictationHold))

        #expect(transition.state == .dictation)
        #expect(transition.sideEffects == [.stopCommands, .startDictation])
        #expect(sut.lastActiveMode == .command)
        #expect(sut.modeBeforeHold == .command)
    }

    @Test
    func test_holdRoundTripFromCommand_restoresCommand() {
        var sut = ModeStateMachine(state: .command, lastActiveMode: .command)
        _ = sut.handle(.hotkeyPress(.dictationHold))
        let transition = sut.handle(.hotkeyRelease(.dictationHold))

        #expect(transition.state == .command)
        #expect(transition.sideEffects == [.stopDictation, .startCommands])
        #expect(sut.lastActiveMode == .command)
        #expect(sut.modeBeforeHold == nil)
    }

    @Test
    func test_sleepingVoiceDictationMode_entersDictation() {
        var sut = ModeStateMachine()
        let transition = sut.handle(.voiceTrigger(.dictationMode))

        #expect(transition.state == .dictation)
        #expect(transition.sideEffects == [.startDictation])
        #expect(sut.lastActiveMode == .dictation)
    }

    @Test
    func test_sleepingVoiceWakeUp_entersDictation() {
        var sut = ModeStateMachine()
        let transition = sut.handle(.voiceTrigger(.wakeUp))

        #expect(transition.state == .dictation)
        #expect(transition.sideEffects == [.startDictation])
        #expect(sut.lastActiveMode == .dictation)
    }

    @Test
    func test_dictationAutoSleepFired_returnsSleeping() {
        var sut = ModeStateMachine(state: .dictation)
        let transition = sut.handle(.autoSleepFired)

        #expect(transition.state == .sleeping)
        #expect(transition.sideEffects == [.stopDictation])
    }

    @Test
    func test_holdRelease_afterInterveningTransition_isNoOp() {
        var sut = ModeStateMachine(state: .command, lastActiveMode: .command)
        _ = sut.handle(.hotkeyPress(.dictationHold))
        _ = sut.handle(.hotkeyPress(.sleepToggle))
        _ = sut.handle(.voiceTrigger(.wakeUp))

        let release = sut.handle(.hotkeyRelease(.dictationHold))

        #expect(release.state == .dictation)
        #expect(release.sideEffects == [])
        #expect(sut.modeBeforeHold == nil)
    }

    @Test
    func test_holdPressAndReleaseDuringDictation_noOp() {
        var sut = ModeStateMachine(state: .dictation)

        let press = sut.handle(.hotkeyPress(.dictationHold))

        #expect(press.state == .dictation)
        #expect(press.sideEffects == [])
        #expect(sut.modeBeforeHold == nil)

        let release = sut.handle(.hotkeyRelease(.dictationHold))

        #expect(release.state == .dictation)
        #expect(release.sideEffects == [])
        #expect(sut.modeBeforeHold == nil)
    }

    @Test
    func test_commandAutoSleepFired_returnsSleeping() {
        var sut = ModeStateMachine(state: .command, lastActiveMode: .command)
        let transition = sut.handle(.autoSleepFired)

        #expect(transition.state == .sleeping)
        #expect(transition.sideEffects == [.stopCommands])
    }

    @Test
    func test_dictationVoiceSleep_returnsSleeping() {
        var sut = ModeStateMachine(state: .dictation)
        let transition = sut.handle(.voiceTrigger(.sleep))

        #expect(transition.state == .sleeping)
        #expect(transition.sideEffects == [.stopDictation])
    }

    @Test
    func test_sleepingAutoSleepFired_isNoOp() {
        var sut = ModeStateMachine()
        let transition = sut.handle(.autoSleepFired)

        #expect(transition.state == .sleeping)
        #expect(transition.sideEffects == [])
    }

    @Test
    func test_dictationVoiceDictationMode_isNoOp() {
        var sut = ModeStateMachine(state: .dictation)
        let transition = sut.handle(.voiceTrigger(.dictationMode))

        #expect(transition.state == .dictation)
        #expect(transition.sideEffects == [])
    }

    @Test
    func test_commandVoiceWakeUp_isNoOp() {
        var sut = ModeStateMachine(state: .command, lastActiveMode: .command)
        let transition = sut.handle(.voiceTrigger(.wakeUp))

        #expect(transition.state == .command)
        #expect(transition.sideEffects == [])
    }

    @Test
    func test_commandVoiceSleep_returnsSleeping() {
        var sut = ModeStateMachine(state: .command, lastActiveMode: .command)
        let transition = sut.handle(.voiceTrigger(.sleep))

        #expect(transition.state == .sleeping)
        #expect(transition.sideEffects == [.stopCommands])
    }
}
