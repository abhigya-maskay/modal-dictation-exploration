import Testing
@testable import ModalDictationCore

@Suite("VoiceTriggerMatcher")
struct VoiceTriggerMatcherTests {

    static let triggerCases: [(phrase: String, action: String, expected: VoiceTrigger)] = [
        ("go to sleep", "app:sleep", .sleep),
        ("wake up", "app:wake", .wakeUp),
        ("dictation mode", "mode:dictation", .dictationMode),
        ("command mode", "mode:command", .commandMode),
    ]

    @Test(arguments: triggerCases)
    func test_allTriggerTypes_returnCorrectValues(phrase: String, action: String, expected: VoiceTrigger) {
        let matcher = VoiceTriggerMatcher(actions: [phrase: action])
        #expect(matcher.match(phrase) == expected)
    }

    @Test
    func test_caseInsensitiveMatch_returnsTrigger() {
        let matcher = VoiceTriggerMatcher(actions: ["go to sleep": "app:sleep"])
        #expect(matcher.match("GO TO SLEEP") == .sleep)
    }

    @Test
    func test_substringMatch_returnsTrigger() {
        let matcher = VoiceTriggerMatcher(actions: ["go to sleep": "app:sleep"])
        #expect(matcher.match("okay go to sleep now") == .sleep)
    }

    @Test
    func test_noMatchingPhrase_returnsNil() {
        let matcher = VoiceTriggerMatcher(actions: ["go to sleep": "app:sleep"])
        #expect(matcher.match("hello world") == nil)
    }

    @Test
    func test_allVoiceTriggerCases_haveMappedAction() {
        let mapped = Set(VoiceTriggerMatcher.actionToTrigger.values)
        for trigger in VoiceTrigger.allCases {
            #expect(mapped.contains(trigger), "VoiceTrigger.\(trigger) has no entry in actionToTrigger")
        }
    }

    @Test
    func test_overlappingPhrases_prefersLongestMatch() {
        let matcher = VoiceTriggerMatcher(actions: [
            "sleep": "app:sleep",
            "go to sleep": "mode:command",
        ])
        #expect(matcher.match("go to sleep") == .commandMode)
    }

    @Test
    func test_unknownAction_returnsNil() {
        let matcher = VoiceTriggerMatcher(actions: ["do something": "custom:foo"])
        #expect(matcher.match("do something") == nil)
    }
}
