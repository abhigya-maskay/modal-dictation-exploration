import Testing
@testable import ModalDictationCore

@Suite("KeyCodeMap")
struct KeyCodeMapTests {

    @Test func test_mapping_shiftFlag_correctForSymbolPairs() {
        let bracket = KeyCodeMap.mapping(for: "[")
        let curly = KeyCodeMap.mapping(for: "{")
        #expect(bracket != nil)
        #expect(curly != nil)
        #expect(bracket!.keyCode == curly!.keyCode)
        #expect(bracket!.shift == false)
        #expect(curly!.shift == true)

        let backtick = KeyCodeMap.mapping(for: "`")
        let tilde = KeyCodeMap.mapping(for: "~")
        #expect(backtick != nil)
        #expect(tilde != nil)
        #expect(backtick!.keyCode == tilde!.keyCode)
        #expect(backtick!.shift == false)
        #expect(tilde!.shift == true)
    }
}

@Suite("ModifierMap")
struct ModifierMapTests {

    @Test func test_combinedFlags_multipleModifiers_ORsTogether() {
        let flags = ModifierMap.combinedFlags(for: ["cmd", "shift"])
        #expect(flags.contains(.maskCommand))
        #expect(flags.contains(.maskShift))
    }

    @Test func test_combinedFlags_unknownModifier_skipped() {
        let flags = ModifierMap.combinedFlags(for: ["cmd", "bogus"])
        #expect(flags == .maskCommand)
    }
}

@Suite("KeystrokeEmitter")
struct KeystrokeEmitterTests {

    @Test func test_emit_unknownKey_throwsError() {
        #expect(throws: EmitterError.unknownKey("nonexistent")) {
            try KeystrokeEmitter.emit(key: "nonexistent")
        }
    }

    @Test func test_emitCommand_action_doesNotThrow() throws {
        try KeystrokeEmitter.emit(command: .action("mode:dictation"))
    }
}
