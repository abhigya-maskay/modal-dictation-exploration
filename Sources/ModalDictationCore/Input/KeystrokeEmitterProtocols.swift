public protocol KeystrokeEmitting: Sendable {
    func emit(command: MatchedCommand) throws
}
