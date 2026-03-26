import Foundation

public struct HIDDeviceRef: Sendable, Equatable {
    public let vendorID: Int
    public let productID: Int
    public let button: Int

    public init(vendorID: Int, productID: Int, button: Int) {
        self.vendorID = vendorID
        self.productID = productID
        self.button = button
    }
}

public enum HotkeyInput: Sendable, Equatable {
    case keyboard(String)
    case device(HIDDeviceRef)
}

public struct HotkeyConfig: Sendable, Equatable {
    public var dictationHold: HotkeyInput?
    public var sleepToggle: HotkeyInput?

    public init(dictationHold: HotkeyInput? = nil, sleepToggle: HotkeyInput? = nil) {
        self.dictationHold = dictationHold
        self.sleepToggle = sleepToggle
    }
}
