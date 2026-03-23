import Demo
import XCTest

final class DemoClassesAndStreamsTests: XCTestCase {
    func testInventoryCounterAndMathUtilsExerciseConstructorsAndMethods() throws {
        let inventory = Inventory()
        XCTAssertEqual(inventory.capacity(), 100)
        XCTAssertEqual(inventory.count(), 0)
        XCTAssertEqual(inventory.add(item: "hammer"), true)
        XCTAssertEqual(inventory.getAll(), ["hammer"])
        XCTAssertEqual(inventory.remove(index: 0), "hammer")
        XCTAssertNil(inventory.remove(index: 0))

        let smallInventory = Inventory(withCapacity: 2)
        XCTAssertEqual(smallInventory.capacity(), 2)
        XCTAssertEqual(smallInventory.add(item: "a"), true)
        XCTAssertEqual(smallInventory.add(item: "b"), true)
        XCTAssertEqual(smallInventory.add(item: "c"), false)
        XCTAssertEqual(smallInventory.getAll(), ["a", "b"])

        let tryInventory = try Inventory(tryNew: 1)
        XCTAssertEqual(tryInventory.capacity(), 1)
        XCTAssertEqual(tryInventory.add(item: "only"), true)
        XCTAssertEqual(tryInventory.add(item: "overflow"), false)
        assertThrowsMessageContains("capacity must be greater than zero", try Inventory(tryNew: 0))

        let counter = Counter(initial: 2)
        XCTAssertEqual(counter.get(), 2)
        counter.increment()
        XCTAssertEqual(counter.get(), 3)
        counter.add(amount: 7)
        XCTAssertEqual(counter.get(), 10)
        XCTAssertEqual(try counter.tryGetPositive(), 10)
        XCTAssertEqual(counter.maybeDouble(), 20)
        XCTAssertEqual(counter.asPoint(), Point(x: 10.0, y: 0.0))
        counter.reset()
        XCTAssertEqual(counter.get(), 0)
        XCTAssertNil(counter.maybeDouble())
        assertThrowsMessageContains("count is not positive", try counter.tryGetPositive())

        let mathUtils = MathUtils(precision: 2)
        XCTAssertEqual(mathUtils.round(value: 3.14159), 3.14, accuracy: 1e-9)
        XCTAssertEqual(MathUtils.add(a: 4, b: 5), 9)
        XCTAssertEqual(MathUtils.clamp(value: 12.0, min: 0.0, max: 10.0), 10.0, accuracy: 1e-9)
        XCTAssertEqual(MathUtils.distanceBetween(a: Point(x: 0.0, y: 0.0), b: Point(x: 3.0, y: 4.0)), 5.0, accuracy: 1e-9)
        XCTAssertEqual(MathUtils.midpoint(a: Point(x: 1.0, y: 2.0), b: Point(x: 3.0, y: 4.0)), Point(x: 2.0, y: 3.0))
        XCTAssertEqual(try MathUtils.parseInt(input: "42"), 42)
        assertThrowsMessageContains("invalid digit found in string", try MathUtils.parseInt(input: "nope"))
        XCTAssertEqual(MathUtils.safeSqrt(value: 9.0), Optional(3.0))
        XCTAssertNil(MathUtils.safeSqrt(value: -1.0))
    }

    func testAsyncWorkerSharedCounterAndStateHolderExerciseSyncAndAsyncMethods() async throws {
        let worker = AsyncWorker(prefix: "test")
        XCTAssertEqual(worker.getPrefix(), "test")
        let workerProcessResult = try await worker.process(input: "data")
        XCTAssertEqual(workerProcessResult, "test: data")
        let workerTryProcessResult = try await worker.tryProcess(input: "data")
        XCTAssertEqual(workerTryProcessResult, "test: data")
        await assertAsyncThrowsMessageContains("input must not be empty") { try await worker.tryProcess(input: "") }
        let workerFindItemResult = try await worker.findItem(id: 42)
        XCTAssertEqual(workerFindItemResult, "test_42")
        let workerFindItemMissing = try await worker.findItem(id: -1)
        XCTAssertNil(workerFindItemMissing)
        let workerProcessBatchResult = try await worker.processBatch(inputs: ["x", "y"])
        XCTAssertEqual(workerProcessBatchResult, ["test: x", "test: y"])

        let sharedCounter = SharedCounter(initial: 5)
        XCTAssertEqual(sharedCounter.get(), 5)
        sharedCounter.set(value: 6)
        XCTAssertEqual(sharedCounter.get(), 6)
        XCTAssertEqual(sharedCounter.increment(), 7)
        XCTAssertEqual(sharedCounter.add(amount: 3), 10)
        let sharedCounterAsyncGetResult = try await sharedCounter.asyncGet()
        XCTAssertEqual(sharedCounterAsyncGetResult, 10)
        let sharedCounterAsyncAddResult = try await sharedCounter.asyncAdd(amount: 5)
        XCTAssertEqual(sharedCounterAsyncAddResult, 15)

        let stateHolder = StateHolder(label: "local")
        XCTAssertEqual(stateHolder.getLabel(), "local")
        XCTAssertEqual(stateHolder.getValue(), 0)
        stateHolder.setValue(value: 5)
        XCTAssertEqual(stateHolder.getValue(), 5)
        XCTAssertEqual(stateHolder.increment(), 6)
        stateHolder.addItem(item: "a")
        stateHolder.addItem(item: "b")
        XCTAssertEqual(stateHolder.itemCount(), 2)
        XCTAssertEqual(stateHolder.getItems(), ["a", "b"])
        XCTAssertEqual(stateHolder.removeLast(), "b")
        XCTAssertEqual(stateHolder.transformValue(f: { $0 / 2 }), 3)
        let stateHolderAsyncGetValueResult = try await stateHolder.asyncGetValue()
        XCTAssertEqual(stateHolderAsyncGetValueResult, 3)
        try await stateHolder.asyncSetValue(value: 9)
        XCTAssertEqual(stateHolder.getValue(), 9)
        let stateHolderAsyncAddItemResult = try await stateHolder.asyncAddItem(item: "z")
        XCTAssertEqual(stateHolderAsyncAddItemResult, 2)
        XCTAssertEqual(stateHolder.getItems(), ["a", "z"])
        stateHolder.clear()
        XCTAssertEqual(stateHolder.getValue(), 0)
        XCTAssertEqual(stateHolder.getItems(), [])
    }

    func testEventBusStreamsDeliverValuesAndPoints() async throws {
        let bus = EventBus()
        async let values: [Int32] = collectPrefix(bus.subscribeValues(), count: 4)
        async let points: [Point] = collectPrefix(bus.subscribePoints(), count: 2)

        try await _Concurrency.Task.sleep(nanoseconds: 100_000_000)
        bus.emitValue(value: 1)
        XCTAssertEqual(bus.emitBatch(values: [2, 3, 4]), 3)
        bus.emitPoint(point: Point(x: 1.0, y: 2.0))
        bus.emitPoint(point: Point(x: 3.0, y: 4.0))

        let emittedValues = await values
        XCTAssertEqual(emittedValues, [1, 2, 3, 4])
        let emittedPoints = await points
        XCTAssertEqual(emittedPoints, [Point(x: 1.0, y: 2.0), Point(x: 3.0, y: 4.0)])
    }
}
