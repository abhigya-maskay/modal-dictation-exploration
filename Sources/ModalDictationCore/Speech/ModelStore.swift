import CoreML
import FluidAudio

public enum ModelStoreError: Error {
    case modelsNotLoaded(String)
}

public actor ModelStore {

    public enum State: Sendable {
        case idle
        case loading
        case ready
        case failed(Error)
    }

    public private(set) var tdtState: State = .idle
    public private(set) var eouState: State = .idle

    private var tdtModels: AsrModels?
    private var eouManager: StreamingEouAsrManager?

    private let chunkSize: StreamingChunkSize
    private let progressHandler: DownloadUtils.ProgressHandler?

    public init(
        chunkSize: StreamingChunkSize = .ms320,
        progressHandler: DownloadUtils.ProgressHandler? = nil
    ) {
        self.chunkSize = chunkSize
        self.progressHandler = progressHandler
    }

    public func loadAll() async throws {
        async let tdt = loadTDT()
        async let eou = loadEOU()
        _ = try await (tdt, eou)
    }

    @discardableResult
    public func loadTDT() async throws -> AsrModels {
        if let cached = tdtModels { return cached }

        tdtState = .loading
        do {
            let models = try await AsrModels.downloadAndLoad(
                progressHandler: progressHandler
            )
            tdtModels = models
            tdtState = .ready
            return models
        } catch {
            tdtState = .failed(error)
            throw error
        }
    }

    @discardableResult
    public func loadEOU() async throws -> StreamingEouAsrManager {
        if let cached = eouManager { return cached }

        eouState = .loading
        do {
            let manager = StreamingEouAsrManager(chunkSize: chunkSize)
            try await manager.loadModelsFromHuggingFace(
                progressHandler: progressHandler
            )
            eouManager = manager
            eouState = .ready
            return manager
        } catch {
            eouState = .failed(error)
            throw error
        }
    }

    public func getTDTModels() throws -> AsrModels {
        guard let models = tdtModels else {
            throw ModelStoreError.modelsNotLoaded("TDT models not loaded — call loadTDT() first")
        }
        return models
    }

    public func getEOUManager() throws -> StreamingEouAsrManager {
        guard let manager = eouManager else {
            throw ModelStoreError.modelsNotLoaded("EOU models not loaded — call loadEOU() first")
        }
        return manager
    }
}
