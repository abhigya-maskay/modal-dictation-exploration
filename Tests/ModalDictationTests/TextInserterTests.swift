import AppKit
import Testing
@testable import ModalDictationCore

struct NoOpPasteStrategy: PasteStrategy {
    func paste() throws {}
}

@Suite("TextInserter", .serialized)
struct TextInserterTests {

    @Test func test_insert_succeedsAndRestoresPasteboard() async throws {
        let board = NSPasteboard.general
        board.clearContents()
        board.setString("original", forType: .string)

        try await TextInserter.insert("new text", pasteStrategy: NoOpPasteStrategy())

        let restored = board.string(forType: .string)
        #expect(restored == "original")
    }

    @Test func test_pasteboardSaver_capturesAndRestoresMultipleTypes() throws {
        let board = NSPasteboard.general
        let plainData = "hello".data(using: .utf8)!
        let rtfData = "{\\rtf1 bold}".data(using: .utf8)!

        board.clearContents()
        board.declareTypes([.string, .rtf], owner: nil)
        board.setData(plainData, forType: .string)
        board.setData(rtfData, forType: .rtf)

        let saver = PasteboardSaver()

        board.clearContents()
        #expect(board.data(forType: .string) == nil)

        try saver.restore()

        #expect(board.data(forType: .string) == plainData)
        #expect(board.data(forType: .rtf) == rtfData)
    }

    @Test func test_pasteboardSaver_restore_emptyBoard_clearsAddedContent() throws {
        let board = NSPasteboard.general
        board.clearContents()

        let saver = PasteboardSaver()

        board.setString("junk", forType: .string)
        #expect(board.string(forType: .string) == "junk")

        try saver.restore()

        #expect(board.string(forType: .string) == nil)
    }
}
