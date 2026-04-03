import Foundation

func awaitCompletion(of task: Task<Void, Never>, timeout: Duration = .seconds(2)) async throws {
    try await withThrowingTaskGroup(of: Void.self) { group in
        group.addTask { await task.value }
        group.addTask {
            try await Task.sleep(for: timeout)
            struct Elapsed: Error {}
            throw Elapsed()
        }
        try await group.next()
        group.cancelAll()
    }
}
