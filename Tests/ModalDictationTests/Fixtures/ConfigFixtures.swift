import Foundation
@testable import ModalDictationCore

extension HIDDeviceRef {
    static func fixture(
        vendorID: Int = .random(in: 1...0xFFFF),
        productID: Int = .random(in: 1...0xFFFF),
        button: Int = .random(in: 1...10)
    ) -> HIDDeviceRef {
        HIDDeviceRef(vendorID: vendorID, productID: productID, button: button)
    }
}

extension HotkeyConfig {
    static func fixture(
        dictationHold: HotkeyInput? = .keyboard("f\(Int.random(in: 1...20))"),
        sleepToggle: HotkeyInput? = .keyboard("f\(Int.random(in: 1...20))")
    ) -> HotkeyConfig {
        HotkeyConfig(dictationHold: dictationHold, sleepToggle: sleepToggle)
    }
}

extension MicConfig {
    static func fixture(
        deviceID: String? = "device-\(UUID().uuidString.prefix(8))"
    ) -> MicConfig {
        MicConfig(deviceID: deviceID)
    }
}

extension SpeechConfig {
    static func fixture(
        timeout: Double? = Double.random(in: 0.1...2.0),
        autoSleepMinutes: Double? = Double.random(in: 1.0...10.0)
    ) -> SpeechConfig {
        SpeechConfig(timeout: timeout, autoSleepMinutes: autoSleepMinutes)
    }
}

extension CommandsConfig {
    static func fixture(
        actions: [String: String] = [:],
        modifiers: [String: String] = [:],
        keys: [String: String] = [:]
    ) -> CommandsConfig {
        CommandsConfig(actions: actions, modifiers: modifiers, keys: keys)
    }
}

extension AppConfig {
    static func fixture(
        hotkeys: HotkeyConfig = .fixture(),
        mic: MicConfig = .fixture(),
        speech: SpeechConfig = .fixture(),
        commands: CommandsConfig = .fixture()
    ) -> AppConfig {
        AppConfig(hotkeys: hotkeys, mic: mic, speech: speech, commands: commands)
    }
}
