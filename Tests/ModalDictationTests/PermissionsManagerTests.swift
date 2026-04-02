import Testing
@testable import ModalDictationCore

@MainActor
private final class FakeAccessibilityChecker: AccessibilityChecker {
    var trusted = false
    func isProcessTrusted() -> Bool { trusted }
}

@Suite("PermissionsManager")
struct PermissionsManagerTests {

    @Test @MainActor func test_checkAccessibility_trusted_setsGranted() {
        let checker = FakeAccessibilityChecker()
        checker.trusted = true
        let manager = PermissionsManager(accessibilityChecker: checker)
        manager.checkAccessibility()
        #expect(manager.accessibilityStatus == .granted)
    }

    @Test @MainActor func test_checkAccessibility_untrusted_setsDenied() {
        let checker = FakeAccessibilityChecker()
        let manager = PermissionsManager(accessibilityChecker: checker)
        manager.checkAccessibility()
        #expect(manager.accessibilityStatus == .denied)
    }

    @Test @MainActor func test_stopObserving_isIdempotent() {
        let manager = PermissionsManager()
        manager.startObserving()
        manager.stopObserving()
        manager.stopObserving()
        #expect(manager.accessibilityStatus != .unknown)
    }
}
