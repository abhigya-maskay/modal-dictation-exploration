import CoreGraphics
import Foundation
import IOKit
import IOKit.hid

public enum ListenerError: Error, Equatable {
    case eventTapFailed
    case hidManagerOpenFailed
}

@MainActor
public final class HotkeyListener: HotkeyListening {

    private static let hidOptionsNone = IOOptionBits(kIOHIDOptionsTypeNone)
    private var continuation: AsyncStream<HotkeyEvent>.Continuation?
    private var eventTap: CFMachPort?
    private var runLoopSource: CFRunLoopSource?
    private var hidManager: IOHIDManager?
    var config: HotkeyConfig?
    private var cachedKeyCodes: [CGKeyCode: HotkeyInput] = [:]
    private var cachedHIDRefs: [HIDDeviceRef] = []

    public init() {}

    public func start(config: HotkeyConfig) throws -> AsyncStream<HotkeyEvent> {
        stop()
        self.config = config
        cachedKeyCodes = keyboardKeyCodes()
        cachedHIDRefs = hidDeviceRefs()

        let stream = AsyncStream<HotkeyEvent> { self.continuation = $0 }

        do {
            if hasKeyboardHotkeys { try installEventTap() }
            if hasHIDHotkeys { try installHIDManager() }
        } catch {
            stop()
            throw error
        }

        return stream
    }

    public func stop() {
        if let tap = eventTap {
            CGEvent.tapEnable(tap: tap, enable: false)
            eventTap = nil
        }
        if let source = runLoopSource {
            CFRunLoopRemoveSource(CFRunLoopGetMain(), source, .commonModes)
            runLoopSource = nil
        }
        if let hid = hidManager {
            IOHIDManagerClose(hid, Self.hidOptionsNone)
            hidManager = nil
        }
        continuation?.finish()
        continuation = nil
        config = nil
        cachedKeyCodes = [:]
        cachedHIDRefs = []
    }

    private static let hidInputCallback: IOHIDValueCallback = { context, _, _, value in
        guard let context else { return }
        let listener = Unmanaged<HotkeyListener>.fromOpaque(context).takeUnretainedValue()

        let element = IOHIDValueGetElement(value)
        let usage = IOHIDElementGetUsage(element)
        let integerValue = IOHIDValueGetIntegerValue(value)

        let device = IOHIDElementGetDevice(element)
        let vendorID = IOHIDDeviceGetProperty(device, kIOHIDVendorIDKey as CFString) as? Int ?? 0
        let productID = IOHIDDeviceGetProperty(device, kIOHIDProductIDKey as CFString) as? Int ?? 0

        MainActor.assumeIsolated {
            let refs = listener.cachedHIDRefs
            guard let ref = refs.first(where: {
                $0.vendorID == vendorID && $0.productID == productID && $0.button == Int(usage)
            }) else { return }

            let isPressed = integerValue >= 1
            let action: HotkeyAction = isPressed ? .press : .release
            listener.continuation?.yield(HotkeyEvent(action: action, input: .device(ref)))
        }
    }

    private var allInputs: [HotkeyInput] {
        guard let config else { return [] }
        return [config.dictationHold, config.sleepToggle].compactMap { $0 }
    }

    private func hidDeviceRefs() -> [HIDDeviceRef] {
        allInputs.compactMap { if case .device(let ref) = $0 { ref } else { nil } }
    }

    private var hasKeyboardHotkeys: Bool {
        allInputs.contains { if case .keyboard = $0 { true } else { false } }
    }

    private var hasHIDHotkeys: Bool {
        allInputs.contains { if case .device = $0 { true } else { false } }
    }

    private var pointer: UnsafeMutableRawPointer {
        Unmanaged.passUnretained(self).toOpaque()
    }

    private func installEventTap() throws {
        let mask: CGEventMask = (1 << CGEventType.keyDown.rawValue)
            | (1 << CGEventType.keyUp.rawValue)

        let selfPtr = pointer

        guard let tap = CGEvent.tapCreate(
            tap: .cgSessionEventTap,
            place: .tailAppendEventTap,
            options: .listenOnly,
            eventsOfInterest: mask,
            callback: Self.eventTapCallback,
            userInfo: selfPtr
        ) else { throw ListenerError.eventTapFailed }

        eventTap = tap
        runLoopSource = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, tap, 0)
        CFRunLoopAddSource(CFRunLoopGetMain(), runLoopSource, .commonModes)
        CGEvent.tapEnable(tap: tap, enable: true)
    }

    private static let eventTapCallback: CGEventTapCallBack = { _, type, event, userInfo in
        guard let userInfo else { return Unmanaged.passUnretained(event) }
        let listener = Unmanaged<HotkeyListener>.fromOpaque(userInfo).takeUnretainedValue()

        if type == .tapDisabledByTimeout {
            MainActor.assumeIsolated {
                if let tap = listener.eventTap { CGEvent.tapEnable(tap: tap, enable: true) }
            }
            return Unmanaged.passUnretained(event)
        }

        let keyCode = CGKeyCode(event.getIntegerValueField(.keyboardEventKeycode))

        MainActor.assumeIsolated {
            let keyCodes = listener.cachedKeyCodes
            guard let input = keyCodes[keyCode] else { return }

            let action: HotkeyAction = type == .keyDown ? .press : .release
            listener.continuation?.yield(HotkeyEvent(action: action, input: input))
        }

        return Unmanaged.passUnretained(event)
    }

    func keyboardKeyCodes() -> [CGKeyCode: HotkeyInput] {
        var result: [CGKeyCode: HotkeyInput] = [:]
        for input in allInputs {
            if case .keyboard(let name) = input,
               let mapping = KeyCodeMap.mapping(for: name) {
                result[mapping.keyCode] = input
            }
        }
        return result
    }

    private func installHIDManager() throws {
        let manager = IOHIDManagerCreate(kCFAllocatorDefault, Self.hidOptionsNone)
        let matchingDicts = cachedHIDRefs.map { ref in
            [
                kIOHIDVendorIDKey: ref.vendorID,
                kIOHIDProductIDKey: ref.productID,
            ] as NSDictionary
        }
        IOHIDManagerSetDeviceMatchingMultiple(manager, matchingDicts as CFArray)
        IOHIDManagerScheduleWithRunLoop(manager, CFRunLoopGetMain(), CFRunLoopMode.commonModes.rawValue)

        let status = IOHIDManagerOpen(manager, Self.hidOptionsNone)
        guard status == kIOReturnSuccess else {
            IOHIDManagerClose(manager, Self.hidOptionsNone)
            throw ListenerError.hidManagerOpenFailed
        }

        let selfPtr = pointer
        IOHIDManagerRegisterInputValueCallback(manager, Self.hidInputCallback, selfPtr)

        hidManager = manager
    }
}
