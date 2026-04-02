import ApplicationServices
import AppKit
import AVFoundation

public enum PermissionStatus: Sendable, Equatable {
    case unknown
    case granted
    case denied
    case restricted
}

@MainActor
public protocol AccessibilityChecker {
    func isProcessTrusted() -> Bool
}

public struct SystemAccessibilityChecker: AccessibilityChecker {
    public init() {}
    public func isProcessTrusted() -> Bool { AXIsProcessTrusted() }
}

@MainActor @Observable
public final class PermissionsManager {
    public var microphoneStatus = PermissionStatus.unknown
    public var accessibilityStatus = PermissionStatus.unknown

    private let accessibilityChecker: any AccessibilityChecker

    public init(accessibilityChecker: any AccessibilityChecker = SystemAccessibilityChecker()) {
        self.accessibilityChecker = accessibilityChecker
    }

    public func checkMicrophone() {
        let status = AVCaptureDevice.authorizationStatus(for: .audio)
        microphoneStatus = switch status {
        case .authorized: .granted
        case .denied: .denied
        case .restricted: .restricted
        case .notDetermined: .unknown
        @unknown default: .unknown
        }
    }

    public func requestMicrophoneAccess() async {
        await AVCaptureDevice.requestAccess(for: .audio)
        checkMicrophone()
    }

    public func checkAccessibility() {
        accessibilityStatus = accessibilityChecker.isProcessTrusted() ? .granted : .denied
    }

    // swiftlint:disable:next force_unwrapping
    private static let microphoneSettingsURL = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone")!
    // swiftlint:disable:next force_unwrapping
    private static let accessibilitySettingsURL = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")!
    private static let pollInterval: TimeInterval = 2

    private var pollTask: Task<Void, Never>?

    public func startObserving() {
        stopObserving()
        checkMicrophone()
        checkAccessibility()
        pollTask = Task { [weak self] in
            while !Task.isCancelled {
                try? await Task.sleep(for: .seconds(Self.pollInterval))
                guard let self else { break }
                self.checkMicrophone()
                self.checkAccessibility()
            }
        }
    }

    public func stopObserving() {
        pollTask?.cancel()
        pollTask = nil
    }

    public func openMicrophoneSettings() {
        NSWorkspace.shared.open(Self.microphoneSettingsURL)
    }

    public func openAccessibilitySettings() {
        NSWorkspace.shared.open(Self.accessibilitySettingsURL)
    }
}
