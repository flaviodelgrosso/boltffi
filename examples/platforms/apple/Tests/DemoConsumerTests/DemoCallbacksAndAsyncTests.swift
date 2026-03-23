import Demo
import XCTest

final class DemoCallbacksAndAsyncTests: XCTestCase {
    final class SwiftAsyncFetcher: AsyncFetcher {
        func fetchValue(key: Int32) async -> Int32 { key * 100 }
        func fetchString(input: String) async -> String { input.uppercased() }
    }

    final class SwiftAsyncOptionFetcher: AsyncOptionFetcher {
        func find(key: Int32) async -> Int64? { key > 0 ? Int64(key) * 1000 : nil }
    }

    final class Doubler: ValueCallback {
        func onValue(value: Int32) -> Int32 { value * 2 }
    }

    final class Tripler: ValueCallback {
        func onValue(value: Int32) -> Int32 { value * 3 }
    }

    final class SwiftPointTransformer: PointTransformer {
        func transform(point: Point) -> Point { Point(x: point.x + 10.0, y: point.y + 20.0) }
    }

    final class SwiftStatusMapper: StatusMapper {
        func mapStatus(status: Status) -> Status { status == .pending ? .active : .inactive }
    }

    final class SwiftVecProcessor: VecProcessor {
        func process(values: [Int32]) -> [Int32] { values.map { $0 * $0 } }
    }

    final class SwiftMultiMethodCallback: MultiMethodCallback {
        func methodA(x: Int32) -> Int32 { x + 1 }
        func methodB(x: Int32, y: Int32) -> Int32 { x * y }
        func methodC() -> Int32 { 5 }
    }

    final class SwiftOptionCallback: OptionCallback {
        func findValue(key: Int32) -> Int32? { key > 0 ? key * 10 : nil }
    }

    func testClosureExportsInvokeSwiftClosuresCorrectly() {
        var observedValue: Int32?

        XCTAssertEqual(applyClosure(f: { $0 * 2 }, value: 5), 10)
        applyVoidClosure(f: { observedValue = $0 }, value: 42)
        XCTAssertEqual(observedValue, 42)
        XCTAssertEqual(applyNullaryClosure(f: { 99 }), 99)
        XCTAssertEqual(applyStringClosure(f: { $0.uppercased() }, s: "hello"), "HELLO")
        XCTAssertEqual(applyBoolClosure(f: { !$0 }, v: true), false)
        XCTAssertEqual(applyF64Closure(f: { $0 * $0 }, v: 3.0), 9.0, accuracy: 1e-9)
        XCTAssertEqual(mapVecWithClosure(f: { $0 * 2 }, values: [1, 2, 3]), [2, 4, 6])
        XCTAssertEqual(filterVecWithClosure(f: { $0 % 2 == 0 }, values: [1, 2, 3, 4]), [2, 4])
        XCTAssertEqual(applyBinaryClosure(f: +, a: 3, b: 4), 7)
        XCTAssertEqual(applyPointClosure(f: { Point(x: $0.x + 1.0, y: $0.y + 1.0) }, p: Point(x: 1.0, y: 2.0)), Point(x: 2.0, y: 3.0))
    }

    func testSynchronousCallbackTraitsUseCorrectBridgeConversions() {
        let doubler = Doubler()
        let tripler = Tripler()
        let pointTransformer = SwiftPointTransformer()
        let statusMapper = SwiftStatusMapper()
        let multiMethod = SwiftMultiMethodCallback()
        let optionCallback = SwiftOptionCallback()
        let vecProcessor = SwiftVecProcessor()

        XCTAssertEqual(invokeValueCallback(callback: doubler, input: 4), 8)
        XCTAssertEqual(invokeValueCallbackTwice(callback: doubler, a: 3, b: 4), 14)
        XCTAssertEqual(invokeBoxedValueCallback(callback: doubler, input: 5), 10)
        XCTAssertEqual(transformPoint(transformer: pointTransformer, point: Point(x: 1.0, y: 2.0)), Point(x: 11.0, y: 22.0))
        XCTAssertEqual(transformPointBoxed(transformer: pointTransformer, point: Point(x: 3.0, y: 4.0)), Point(x: 13.0, y: 24.0))
        XCTAssertEqual(mapStatus(mapper: statusMapper, status: .pending), .active)
        XCTAssertEqual(processVec(processor: vecProcessor, values: [1, 2, 3]), [1, 4, 9])
        XCTAssertEqual(invokeMultiMethod(callback: multiMethod, x: 3, y: 4), 21)
        XCTAssertEqual(invokeMultiMethodBoxed(callback: multiMethod, x: 3, y: 4), 21)
        XCTAssertEqual(invokeTwoCallbacks(first: doubler, second: tripler, value: 5), 25)
        XCTAssertEqual(invokeOptionCallback(callback: optionCallback, key: 7), 70)
        XCTAssertNil(invokeOptionCallback(callback: optionCallback, key: 0))
    }

    func testTopLevelAsyncFunctionsAndAsyncCallbackTraitsRoundTrip() async throws {
        let asyncAddResult = try await asyncAdd(a: 3, b: 7)
        XCTAssertEqual(asyncAddResult, 10)
        let asyncEchoResult = try await asyncEcho(message: "hello async")
        XCTAssertEqual(asyncEchoResult, "Echo: hello async")
        let asyncDoubleAllResult = try await asyncDoubleAll(values: [1, 2, 3])
        XCTAssertEqual(asyncDoubleAllResult, [2, 4, 6])
        let asyncFindPositiveResult = try await asyncFindPositive(values: [-1, 0, 5, 3])
        XCTAssertEqual(asyncFindPositiveResult, 5)
        let asyncFindPositiveMissing = try await asyncFindPositive(values: [-1, -2, -3])
        XCTAssertNil(asyncFindPositiveMissing)
        let asyncConcatResult = try await asyncConcat(strings: ["a", "b", "c"])
        XCTAssertEqual(asyncConcatResult, "a, b, c")

        let asyncSafeDivideResult = try await asyncSafeDivide(a: 10, b: 2)
        XCTAssertEqual(asyncSafeDivideResult, 5)
        do {
            _ = try await asyncSafeDivide(a: 1, b: 0)
            XCTFail("expected divisionByZero")
        } catch let error as MathError {
            XCTAssertEqual(error, .divisionByZero)
        }
        let asyncFallibleFetchResult = try await asyncFallibleFetch(key: 7)
        XCTAssertEqual(asyncFallibleFetchResult, "value_7")
        await assertAsyncThrowsMessageContains("invalid key") { try await asyncFallibleFetch(key: -1) }
        let asyncFindValueResult = try await asyncFindValue(key: 4)
        XCTAssertEqual(asyncFindValueResult, 40)
        let asyncFindValueMissing = try await asyncFindValue(key: 0)
        XCTAssertNil(asyncFindValueMissing)
        await assertAsyncThrowsMessageContains("invalid key") { try await asyncFindValue(key: -1) }

        let asyncFetcher = SwiftAsyncFetcher()
        let asyncOptionFetcher = SwiftAsyncOptionFetcher()
        let fetchWithAsyncCallbackResult = try await fetchWithAsyncCallback(fetcher: asyncFetcher, key: 5)
        XCTAssertEqual(fetchWithAsyncCallbackResult, 500)
        let fetchStringWithAsyncCallbackResult = try await fetchStringWithAsyncCallback(fetcher: asyncFetcher, input: "boltffi")
        XCTAssertEqual(fetchStringWithAsyncCallbackResult, "BOLTFFI")
        let invokeAsyncOptionFetcherResult = try await invokeAsyncOptionFetcher(fetcher: asyncOptionFetcher, key: 7)
        XCTAssertEqual(invokeAsyncOptionFetcherResult, 7_000)
        let invokeAsyncOptionFetcherMissing = try await invokeAsyncOptionFetcher(fetcher: asyncOptionFetcher, key: 0)
        XCTAssertNil(invokeAsyncOptionFetcherMissing)
    }
}
