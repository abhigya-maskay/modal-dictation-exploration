@preconcurrency import AVFAudio
import CoreAudio

public enum AudioCaptureError: Error {
    case alreadyRunning
}

public actor AudioCaptureEngine: AudioCapturing {
    private let engine = AVAudioEngine()
    private let deviceManager = AudioDeviceManager()
    private var bufferContinuation: AsyncStream<AVAudioPCMBuffer>.Continuation?
    private var currentDeviceUID: String?

    public init() {}

    public func start(deviceUID: String? = nil) throws -> sending AsyncStream<AVAudioPCMBuffer> {
        guard bufferContinuation == nil else { throw AudioCaptureError.alreadyRunning }

        let device = try deviceManager.resolveDevice(preferredUID: deviceUID)

        var audioDeviceID = device.audioDeviceID
        let inputNode = engine.inputNode
        guard let audioUnit = inputNode.audioUnit else {
            throw AudioDeviceError.deviceNotFound(device.uid)
        }
        let status = AudioUnitSetProperty(
            audioUnit,
            kAudioOutputUnitProperty_CurrentDevice,
            kAudioUnitScope_Global,
            0,
            &audioDeviceID,
            UInt32(MemoryLayout<AudioDeviceID>.size)
        )
        guard status == noErr else { throw AudioDeviceError.coreAudioError(status) }

        var continuation: AsyncStream<AVAudioPCMBuffer>.Continuation!
        let stream = AsyncStream<AVAudioPCMBuffer> { continuation = $0 }
        bufferContinuation = continuation

        let format = inputNode.outputFormat(forBus: 0)
        inputNode.installTap(onBus: 0, bufferSize: 1024, format: format) { buffer, _ in
            continuation.yield(buffer)
        }

        do {
            try engine.start()
        } catch {
            engine.inputNode.removeTap(onBus: 0)
            bufferContinuation?.finish()
            bufferContinuation = nil
            throw error
        }

        currentDeviceUID = device.uid

        return stream
    }

    public func stop() {
        engine.stop()
        engine.inputNode.removeTap(onBus: 0)
        bufferContinuation?.finish()
        bufferContinuation = nil
        currentDeviceUID = nil
    }
}
