import Foundation

public final class ConfigWatcher: @unchecked Sendable {
    private let filePath: String
    private let onChange: @Sendable () -> Void
    private let queue = DispatchQueue(label: "config-watcher", qos: .utility)
    private var fileSource: DispatchSourceFileSystemObject?
    private var dirSource: DispatchSourceFileSystemObject?

    public init(filePath: String, onChange: @escaping @Sendable () -> Void) {
        self.filePath = filePath
        self.onChange = onChange
    }

    public func start() {
        watchFile()
        watchDirectory()
    }

    public func stop() {
        cleanupFileSource()
        cleanupDirSource()
    }

    private func watchFile() {
        cleanupFileSource()

        let fd = open(filePath, O_EVTONLY)
        guard fd >= 0 else { return }

        let source = DispatchSource.makeFileSystemObjectSource(
            fileDescriptor: fd,
            eventMask: [.write, .delete, .rename],
            queue: queue
        )

        source.setEventHandler { [weak self] in
            guard let self else { return }
            let flags = source.data
            if flags.contains(.delete) || flags.contains(.rename) {
                self.cleanupFileSource()
            } else {
                self.onChange()
            }
        }

        source.setCancelHandler {
            close(fd)
        }

        fileSource = source
        source.resume()
    }

    private func watchDirectory() {
        cleanupDirSource()

        let dirPath = (filePath as NSString).deletingLastPathComponent
        let fd = open(dirPath, O_EVTONLY)
        guard fd >= 0 else { return }

        let source = DispatchSource.makeFileSystemObjectSource(
            fileDescriptor: fd,
            eventMask: .write,
            queue: queue
        )

        source.setEventHandler { [weak self] in
            guard let self else { return }
            if self.fileSource == nil && FileManager.default.fileExists(atPath: self.filePath) {
                self.watchFile()
                self.onChange()
            }
        }

        source.setCancelHandler {
            close(fd)
        }

        dirSource = source
        source.resume()
    }

    private func cleanupFileSource() {
        fileSource?.cancel()
        fileSource = nil
    }

    private func cleanupDirSource() {
        dirSource?.cancel()
        dirSource = nil
    }
}
