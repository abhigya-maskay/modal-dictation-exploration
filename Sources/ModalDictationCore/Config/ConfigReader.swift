import Foundation
import TOMLKit

public enum ConfigError: Error, CustomStringConvertible {
    case fileNotFound(String)
    case parseError(String)

    public var description: String {
        switch self {
        case .fileNotFound(let path): "Config file not found: \(path)"
        case .parseError(let detail): "Failed to parse config: \(detail)"
        }
    }
}

public enum ConfigReader {

    public static let configDirectory = FileManager.default
        .homeDirectoryForCurrentUser
        .appendingPathComponent(".modal-dictation")

    public static let configFilePath = configDirectory.appendingPathComponent("config.toml")

    public static func ensureConfigExists() throws {
        let fm = FileManager.default
        if !fm.fileExists(atPath: configFilePath.path) {
            try fm.createDirectory(at: configDirectory, withIntermediateDirectories: true)
            guard let bundledURL = Bundle.module.url(forResource: "default-config", withExtension: "toml") else {
                throw ConfigError.parseError("Bundled default-config.toml not found in resources")
            }
            try fm.copyItem(at: bundledURL, to: configFilePath)
        }
    }

    public static func read(from path: String) throws -> AppConfig {
        guard FileManager.default.fileExists(atPath: path) else {
            throw ConfigError.fileNotFound(path)
        }
        let content = try String(contentsOfFile: path, encoding: .utf8)
        return try parse(content)
    }

    public static func parse(_ tomlString: String) throws -> AppConfig {
        let table: TOMLTable
        do {
            table = try TOMLTable(string: tomlString)
        } catch {
            throw ConfigError.parseError(error.localizedDescription)
        }

        let hotkeys = parseHotkeyConfig(table)
        let mic = parseMicConfig(table)
        let speech = parseSpeechConfig(table)
        let commands = parseCommandsConfig(table)

        return AppConfig(hotkeys: hotkeys, mic: mic, speech: speech, commands: commands)
    }

    private static func parseHotkeyConfig(_ root: TOMLTable) -> HotkeyConfig {
        guard let hotkeysTable = root["hotkeys"]?.tomlValue.table else {
            return HotkeyConfig(dictationHold: nil, sleepToggle: nil)
        }
        let dictationHold = parseHotkeyInput(
            hotkeysTable, key: "dictation_hold", deviceKey: "dictation_hold_device"
        )
        let sleepToggle = parseHotkeyInput(
            hotkeysTable, key: "sleep_toggle", deviceKey: "sleep_toggle_device"
        )
        return HotkeyConfig(dictationHold: dictationHold, sleepToggle: sleepToggle)
    }

    private static func parseHotkeyInput(
        _ table: TOMLTable, key: String, deviceKey: String
    ) -> HotkeyInput? {
        if let deviceTable = table[deviceKey]?.tomlValue.table,
           let vendorID = deviceTable["vendor_id"]?.tomlValue.int,
           let productID = deviceTable["product_id"]?.tomlValue.int,
           let button = deviceTable["button"]?.tomlValue.int {
            return .device(HIDDeviceRef(vendorID: vendorID, productID: productID, button: button))
        }

        if let keyString = table[key]?.tomlValue.string {
            return .keyboard(keyString)
        }

        return nil
    }

    private static func parseMicConfig(_ root: TOMLTable) -> MicConfig {
        let deviceID = root["mic"]?.tomlValue.table?["device_id"]?.tomlValue.string
        return MicConfig(deviceID: deviceID)
    }

    private static func parseSpeechConfig(_ root: TOMLTable) -> SpeechConfig {
        let speechTable = root["speech"]?.tomlValue.table
        let timeout = speechTable?["timeout"]?.tomlValue.double
        let autoSleep = speechTable?["auto_sleep_minutes"]?.tomlValue.double
        return SpeechConfig(timeout: timeout, autoSleepMinutes: autoSleep)
    }

    private static func parseCommandsConfig(_ root: TOMLTable) -> CommandsConfig {
        let commandsTable = root["commands"]?.tomlValue.table
        let actions = parseStringDict(commandsTable?["actions"]?.tomlValue.table)
        let modifiers = parseStringDict(commandsTable?["modifiers"]?.tomlValue.table)
        let keys = parseStringDict(commandsTable?["keys"]?.tomlValue.table)
        return CommandsConfig(actions: actions, modifiers: modifiers, keys: keys)
    }

    private static func parseStringDict(_ table: TOMLTable?) -> [String: String] {
        guard let table else { return [:] }
        return Dictionary(uniqueKeysWithValues: table.compactMap { key, value in
            value.tomlValue.string.map { (key, $0) }
        })
    }
}
