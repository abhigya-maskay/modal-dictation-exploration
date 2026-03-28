@preconcurrency import AVFAudio
import FluidAudio

public actor CommandRecognizer {

    private let streamingTranscriber: any StreamingTranscriber

    private var processingTask: Task<Void, Never>?
    private var resultContinuation: AsyncStream<String>.Continuation?

    public init(streamingTranscriber: any StreamingTranscriber) {
        self.streamingTranscriber = streamingTranscriber
    }

    public func start(audio: AsyncStream<AVAudioPCMBuffer>) async -> AsyncStream<String> {
        let (stream, continuation) = AsyncStream.makeStream(of: String.self)
        resultContinuation = continuation

        await streamingTranscriber.setEouCallback { text in
            guard !text.isEmpty else { return }
            continuation.yield(text)
        }

        processingTask = Task {
            for await buffer in audio {
                guard !Task.isCancelled else { break }
                do {
                    _ = try await streamingTranscriber.process(audioBuffer: buffer)
                } catch {
                    continue
                }
            }
            guard !Task.isCancelled else { return }
            _ = try? await streamingTranscriber.finish()
            continuation.finish()
        }

        return stream
    }

    public func stop() async {
        processingTask?.cancel()
        processingTask = nil

        if let finalText = try? await streamingTranscriber.finish(), !finalText.isEmpty {
            resultContinuation?.yield(finalText)
        }

        resultContinuation?.finish()
        resultContinuation = nil
    }

    public func reset() async {
        await streamingTranscriber.reset()
    }
}
