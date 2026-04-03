import Testing
@testable import ModalDictationCore

@Suite("HotkeyListener")
@MainActor
struct HotkeyListenerTests {

    @Test func test_stop_finishesStream_andIsIdempotent() async throws {
        let listener = HotkeyListener()
        let config = HotkeyConfig.fixture(dictationHold: nil, sleepToggle: nil)
        let stream = try listener.start(config: config)

        let consumed = Task { for await _ in stream {} }

        listener.stop()
        try await awaitCompletion(of: consumed)

        listener.stop()
    }

    @Test func test_start_restart_finishesPreviousStream() async throws {
        let listener = HotkeyListener()
        let config = HotkeyConfig.fixture(dictationHold: nil, sleepToggle: nil)
        let firstStream = try listener.start(config: config)

        let consumed = Task { for await _ in firstStream {} }

        _ = try listener.start(config: config)
        try await awaitCompletion(of: consumed)

        listener.stop()
    }

    @Test func test_keyboardKeyCodes_mapsConfiguredKeys() {
        let listener = HotkeyListener()
        let dictationInput: HotkeyInput = .keyboard("f18")
        let sleepInput: HotkeyInput = .keyboard("f15")
        listener.config = .fixture(dictationHold: dictationInput, sleepToggle: sleepInput)

        let keyCodes = listener.keyboardKeyCodes()

        let f18Code = KeyCodeMap.mapping(for: "f18")!.keyCode
        let f15Code = KeyCodeMap.mapping(for: "f15")!.keyCode
        #expect(keyCodes[f18Code] == dictationInput)
        #expect(keyCodes[f15Code] == sleepInput)
    }

    @Test func test_keyboardKeyCodes_emptyForDeviceOnlyConfig() {
        let listener = HotkeyListener()
        listener.config = .fixture(dictationHold: .device(.fixture()), sleepToggle: nil)

        #expect(listener.keyboardKeyCodes().isEmpty)
    }
}
