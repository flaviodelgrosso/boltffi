package com.boltffi.demo

import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull
import kotlin.test.assertTrue

class DemoCallbacksAndAsyncTest {
    @Test
    fun unaryClosureExportsInvokeKotlinClosuresCorrectly() {
        var observedValue: Int? = null

        assertEquals(10, applyClosure(ClosureI32ToI32 { it * 2 }, 5))
        applyVoidClosure(ClosureI32 { observedValue = it }, 42)
        assertEquals(42, observedValue)
        assertEquals(99, applyNullaryClosure(ClosureToI32 { 99 }))
        assertEquals("HELLO", applyStringClosure(ClosureStringToString { it.uppercase() }, "hello"))
        assertEquals(false, applyBoolClosure(ClosureBoolToBool { !it }, true))
        assertDoubleEquals(9.0, applyF64Closure(ClosureF64ToF64 { it * it }, 3.0))
        assertContentEquals(intArrayOf(2, 4, 6), mapVecWithClosure(ClosureI32ToI32 { it * 2 }, intArrayOf(1, 2, 3)))
        assertContentEquals(
            intArrayOf(2, 4),
            filterVecWithClosure(ClosureI32ToBool { it % 2 == 0 }, intArrayOf(1, 2, 3, 4))
        )
    }

    @Test
    fun binaryAndPointClosureExportsInvokeKotlinClosuresCorrectly() {
        assertEquals(7, applyBinaryClosure(ClosureI32I32ToI32 { left, right -> left + right }, 3, 4))
        assertPointEquals(2.0, 3.0, applyPointClosure(ClosurePointToPoint { Point(it.x + 1.0, it.y + 1.0) }, Point(1.0, 2.0)))
    }

    @Test
    fun scalarSynchronousCallbackTraitsUseTheCorrectBridgeConversions() {
        val doubler = object : ValueCallback {
            override fun onValue(value: Int): Int = value * 2
        }
        val tripler = object : ValueCallback {
            override fun onValue(value: Int): Int = value * 3
        }
        val pointTransformer = object : PointTransformer {
            override fun transform(point: Point): Point = Point(point.x + 10.0, point.y + 20.0)
        }
        val statusMapper = object : StatusMapper {
            override fun mapStatus(status: Status): Status = if (status == Status.PENDING) Status.ACTIVE else Status.INACTIVE
        }
        val multiMethod = object : MultiMethodCallback {
            override fun methodA(x: Int): Int = x + 1
            override fun methodB(x: Int, y: Int): Int = x * y
            override fun methodC(): Int = 5
        }
        val optionCallback = object : OptionCallback {
            override fun findValue(key: Int): Int? = key.takeIf { it > 0 }?.times(10)
        }
        val vecProcessor = object : VecProcessor {
            override fun process(values: IntArray): IntArray = values.map { it * it }.toIntArray()
        }

        assertEquals(8, invokeValueCallback(doubler, 4))
        assertEquals(14, invokeValueCallbackTwice(doubler, 3, 4))
        assertEquals(10, invokeBoxedValueCallback(doubler, 5))
        assertEquals(Status.ACTIVE, mapStatus(statusMapper, Status.PENDING))
        assertContentEquals(intArrayOf(1, 4, 9), processVec(vecProcessor, intArrayOf(1, 2, 3)))
        assertEquals(21, invokeMultiMethod(multiMethod, 3, 4))
        assertEquals(21, invokeMultiMethodBoxed(multiMethod, 3, 4))
        assertEquals(25, invokeTwoCallbacks(doubler, tripler, 5))
        assertEquals(70, invokeOptionCallback(optionCallback, 7))
        assertNull(invokeOptionCallback(optionCallback, 0))
    }

    @Test
    fun pointSynchronousCallbackTraitsUseTheCorrectBridgeConversions() {
        val pointTransformer = object : PointTransformer {
            override fun transform(point: Point): Point = Point(point.x + 10.0, point.y + 20.0)
        }

        assertPointEquals(11.0, 22.0, transformPoint(pointTransformer, Point(1.0, 2.0)))
        assertPointEquals(13.0, 24.0, transformPointBoxed(pointTransformer, Point(3.0, 4.0)))
    }

    @Test
    fun topLevelAsyncFunctionsRoundTripThroughKotlin() = runBlocking {
        withTimeout(10_000) {
            assertEquals(10, asyncAdd(3, 7))
            assertEquals("Echo: hello async", asyncEcho("hello async"))
            assertContentEquals(intArrayOf(2, 4, 6), asyncDoubleAll(intArrayOf(1, 2, 3)))
            assertEquals(5, asyncFindPositive(intArrayOf(-1, 0, 5, 3)))
            assertNull(asyncFindPositive(intArrayOf(-1, -2, -3)))
            assertEquals("a, b, c", asyncConcat(listOf("a", "b", "c")))
        }
    }

    @Test
    fun asyncResultFunctionsRoundTripThroughKotlin() = runBlocking {
        withTimeout(10_000) {
            assertEquals(5, asyncSafeDivide(10, 2))
            assertTrue(assertFailsWith<MathError> { asyncSafeDivide(1, 0) } is MathError.DivisionByZero)
            assertEquals("value_7", asyncFallibleFetch(7))
            assertMessageContains(assertFailsWith<FfiException> { asyncFallibleFetch(-1) }, "invalid key")
            assertEquals(40, asyncFindValue(4))
            assertNull(asyncFindValue(0))
            assertMessageContains(assertFailsWith<FfiException> { asyncFindValue(-1) }, "invalid key")
        }
    }

    @Test
    fun asyncCallbackTraitsRoundTripThroughKotlin() = runBlocking {
        withTimeout(10_000) {
            val asyncFetcher = object : AsyncFetcher {
                override suspend fun fetchValue(key: Int): Int = key * 100
                override suspend fun fetchString(input: String): String = input.uppercase()
            }
            val asyncOptionFetcher = object : AsyncOptionFetcher {
                override suspend fun find(key: Int): Long? = key.takeIf { it > 0 }?.toLong()?.times(1000L)
            }

            assertEquals(500, fetchWithAsyncCallback(asyncFetcher, 5))
            assertEquals("BOLTFFI", fetchStringWithAsyncCallback(asyncFetcher, "boltffi"))
            assertEquals(7_000L, invokeAsyncOptionFetcher(asyncOptionFetcher, 7))
            assertNull(invokeAsyncOptionFetcher(asyncOptionFetcher, 0))
        }
    }
}
