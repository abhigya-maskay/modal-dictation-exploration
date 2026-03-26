import Foundation
@preconcurrency import AVFAudio
import FluidAudio
import ModalDictationCore

setbuf(stdout, nil)

let durationSeconds = 3.0
let outputPath = "/tmp/modal-dictation-test.wav"

let engine = AudioCaptureEngine()

Task {
    do {
        try ConfigReader.ensureConfigExists()
        let config = try ConfigReader.read(from: ConfigReader.configFilePath.path)
        let deviceUID = config.mic.deviceID

        print("Starting \(Int(durationSeconds))s audio capture...")
        try await engine.start(deviceUID: deviceUID)

        guard let stream = await engine.bufferStream else {
            print("No buffer stream available")
            exit(1)
        }

        let converter = AudioConverter()
        var allSamples: [Float] = []

        let deadline = Date().addingTimeInterval(durationSeconds)
        for await buffer in stream {
            let samples = try converter.resampleBuffer(buffer)
            allSamples.append(contentsOf: samples)
            if Date() >= deadline { break }
        }

        await engine.stop()

        let wavData = try AudioWAV.data(from: allSamples, sampleRate: 16000)
        try wavData.write(to: URL(fileURLWithPath: outputPath))
        print("Wrote \(allSamples.count) samples (\(String(format: "%.1f", Double(allSamples.count) / 16000))s) to \(outputPath)")
    } catch {
        print("Fatal: \(error)")
    }
    exit(0)
}

RunLoop.main.run()
