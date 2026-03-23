import Demo
import Foundation
import XCTest

func assertPointEquals(_ point: Point, _ expectedX: Double, _ expectedY: Double, accuracy: Double = 1e-9, file: StaticString = #filePath, line: UInt = #line) {
    XCTAssertEqual(point.x, expectedX, accuracy: accuracy, file: file, line: line)
    XCTAssertEqual(point.y, expectedY, accuracy: accuracy, file: file, line: line)
}

func assertThrowsMessageContains<T>(_ expectedFragment: String, _ expression: @autoclosure () throws -> T, file: StaticString = #filePath, line: UInt = #line) {
    XCTAssertThrowsError(try expression(), file: file, line: line) { error in
        let message = String(describing: error)
        XCTAssertTrue(message.contains(expectedFragment), "expected message to contain \(expectedFragment), got \(message)", file: file, line: line)
    }
}

func assertAsyncThrowsMessageContains<T>(_ expectedFragment: String, _ expression: @escaping () async throws -> T, file: StaticString = #filePath, line: UInt = #line) async {
    do {
        _ = try await expression()
        XCTFail("expected async throw containing \(expectedFragment)", file: file, line: line)
    } catch {
        let message = String(describing: error)
        XCTAssertTrue(message.contains(expectedFragment), "expected message to contain \(expectedFragment), got \(message)", file: file, line: line)
    }
}

func collectPrefix<T>(_ stream: AsyncStream<T>, count: Int) async -> [T] {
    var items: [T] = []
    for await item in stream {
        items.append(item)
        if items.count == count {
            break
        }
    }
    return items
}
