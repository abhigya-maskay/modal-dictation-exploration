import Testing
@testable import ModalDictationCore

@Suite("AudioDeviceManager")
struct AudioDeviceManagerTests {
    let manager = AudioDeviceManager()

    @Test func test_listInputDevices_returnsDevicesWithValidProperties() throws {
        let devices = try manager.listInputDevices()
        #expect(!devices.isEmpty)
        for device in devices {
            #expect(!device.uid.isEmpty)
            #expect(!device.name.isEmpty)
            #expect(device.sampleRate > 0)
        }
    }

    @Test func test_defaultInputDevice_returnsValidDevice() throws {
        let device = try manager.defaultInputDevice()
        #expect(!device.uid.isEmpty)
        #expect(!device.name.isEmpty)
        #expect(device.sampleRate > 0)
    }

    @Test func test_resolveDevice_nilUID_returnsDefault() throws {
        let resolved = try manager.resolveDevice(preferredUID: nil)
        let defaultDevice = try manager.defaultInputDevice()
        #expect(resolved == defaultDevice)
    }

    @Test func test_resolveDevice_unknownUID_fallsBackToDefault() throws {
        let resolved = try manager.resolveDevice(preferredUID: "nonexistent-device-uid")
        let defaultDevice = try manager.defaultInputDevice()
        #expect(resolved == defaultDevice)
    }
}
