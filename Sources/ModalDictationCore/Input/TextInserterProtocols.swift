public protocol TextInserting: Sendable {
    func insert(_ text: String) async throws
}

public protocol PasteStrategy {
    func paste() throws
}
