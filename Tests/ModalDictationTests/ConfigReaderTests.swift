import Testing
@testable import ModalDictationCore

@Suite("ConfigReader")
struct ConfigReaderTests {

    @Test func test_parse_fullConfig_returnsAllSections() throws {
        let toml = """
        [hotkeys]
        dictation_hold = "fn"
        sleep_toggle = "f15"

        [mic]

        [speech]
        timeout = 0.3
        auto_sleep_minutes = 3.0

        [commands.actions]
        "dictation mode" = "mode:dictation"
        "command mode" = "mode:command"
        "go to sleep" = "app:sleep"
        "wake up" = "app:wake"

        [commands.modifiers]
        "command" = "cmd"
        "shift" = "shift"

        [commands.keys]
        "adam" = "a"
        "boy" = "b"
        "return" = "return"
        """

        let config = try ConfigReader.parse(toml)

        #expect(config.hotkeys.dictationHold == .keyboard("fn"))
        #expect(config.hotkeys.sleepToggle == .keyboard("f15"))
        #expect(config.mic.deviceID == nil)
        #expect(config.speech.timeout == 0.3)
        #expect(config.speech.autoSleepMinutes == 3.0)
        #expect(config.commands.actions.count == 4)
        #expect(config.commands.actions["dictation mode"] == "mode:dictation")
        #expect(config.commands.modifiers.count == 2)
        #expect(config.commands.modifiers["command"] == "cmd")
        #expect(config.commands.keys.count == 3)
        #expect(config.commands.keys["adam"] == "a")
    }

    @Test func test_parse_invalidToml_throwsParseError() throws {
        #expect(throws: ConfigError.self) {
            try ConfigReader.parse("[broken")
        }
    }

    @Test func test_parse_emptyToml_returnsAllDefaults() throws {
        let config = try ConfigReader.parse("")

        #expect(config.hotkeys.dictationHold == nil)
        #expect(config.hotkeys.sleepToggle == nil)
        #expect(config.mic.deviceID == nil)
        #expect(config.speech.timeout == nil)
        #expect(config.speech.autoSleepMinutes == nil)
        #expect(config.commands.actions.isEmpty)
        #expect(config.commands.modifiers.isEmpty)
        #expect(config.commands.keys.isEmpty)
    }

    @Test func test_parse_deviceHotkey_prefersDeviceOverKeyboard() throws {
        let toml = """
        [hotkeys]
        dictation_hold = "f18"

        [hotkeys.dictation_hold_device]
        vendor_id = 0x1234
        product_id = 0x5678
        button = 3
        """

        let config = try ConfigReader.parse(toml)

        let expected = HIDDeviceRef(vendorID: 0x1234, productID: 0x5678, button: 3)
        #expect(config.hotkeys.dictationHold == .device(expected))
    }

    @Test func test_parse_incompleteDeviceHotkey_fallsBackToKeyboard() throws {
        let toml = """
        [hotkeys]
        dictation_hold = "f18"

        [hotkeys.dictation_hold_device]
        vendor_id = 0x1234
        product_id = 0x5678
        """

        let config = try ConfigReader.parse(toml)

        #expect(config.hotkeys.dictationHold == .keyboard("f18"))
    }

    @Test func test_parse_emptyHotkeysSection_returnsNilHotkeys() throws {
        let toml = """
        [hotkeys]
        """

        let config = try ConfigReader.parse(toml)

        #expect(config.hotkeys.dictationHold == nil)
        #expect(config.hotkeys.sleepToggle == nil)
    }

    @Test func test_parse_nonStringCommandValues_skipsEntries() throws {
        let toml = """
        [commands.keys]
        "adam" = "a"
        "broken" = 42
        "boy" = "b"
        """

        let config = try ConfigReader.parse(toml)

        #expect(config.commands.keys.count == 2)
        #expect(config.commands.keys["adam"] == "a")
        #expect(config.commands.keys["boy"] == "b")
        #expect(config.commands.keys["broken"] == nil)
    }

    @Test func test_read_nonexistentPath_throwsFileNotFound() throws {
        #expect(throws: ConfigError.self) {
            try ConfigReader.read(from: "/nonexistent/path.toml")
        }
    }
}
