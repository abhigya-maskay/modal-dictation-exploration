import AppKit
import Carbon.HIToolbox

public enum InserterError: Error, Equatable {
    case pasteboardRestoreFailed
    case pasteboardWriteFailed
}

public protocol PasteStrategy {
    func paste() throws
}

public struct CGEventPasteStrategy: PasteStrategy {
    public init() {}
    public func paste() throws {
        try KeyEventPoster.post(keyCode: CGKeyCode(kVK_ANSI_V), flags: .maskCommand)
    }
}

public enum TextInserter {

    public static func insert(
        _ text: String,
        pasteStrategy: some PasteStrategy = CGEventPasteStrategy(),
        settleDelay: Duration = .milliseconds(50)
    ) async throws {
        let saver = PasteboardSaver()
        defer { try? saver.restore() }

        let board = NSPasteboard.general
        board.clearContents()
        guard board.setString(text, forType: .string) else {
            throw InserterError.pasteboardWriteFailed
        }

        try pasteStrategy.paste()
        try await Task.sleep(for: settleDelay)
    }
}

struct PasteboardSaver {
    typealias Item = (type: NSPasteboard.PasteboardType, data: Data)

    private let items: [Item]?

    init() {
        let board = NSPasteboard.general
        items = board.types?.compactMap { type in
            guard let data = board.data(forType: type) else { return nil }
            return (type, data)
        }
    }

    func restore() throws {
        let board = NSPasteboard.general
        guard let items else {
            board.clearContents()
            return
        }
        board.declareTypes(items.map(\.type), owner: nil)
        for item in items {
            guard board.setData(item.data, forType: item.type) else {
                throw InserterError.pasteboardRestoreFailed
            }
        }
    }
}
