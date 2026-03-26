import Foundation
import FluidAudio
import ModalDictationCore

setbuf(stdout, nil)

do {
    try ConfigReader.ensureConfigExists()
    let config = try ConfigReader.read(from: ConfigReader.configFilePath.path)
    print("Config loaded: \(config.commands.keys.count) keys, \(config.commands.modifiers.count) modifiers, \(config.commands.actions.count) actions")

    let watcher = ConfigWatcher(filePath: ConfigReader.configFilePath.path) {
        do {
            let updated = try ConfigReader.read(from: ConfigReader.configFilePath.path)
            print("Config reloaded: \(updated.commands.keys.count) keys")
        } catch {
            print("Config reload failed: \(error)")
        }
    }
    watcher.start()

    RunLoop.main.run()
} catch {
    print("Fatal: \(error)")
}
