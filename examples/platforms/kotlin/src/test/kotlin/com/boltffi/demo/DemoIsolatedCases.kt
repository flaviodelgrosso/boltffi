package com.boltffi.demo

import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.async
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.take
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.withTimeout

fun main(args: Array<String>) {
    when (args.single()) {
        "result-input-roundtrip" -> {
            check(resultToString(BoltFFIResult.Ok(7)) == "ok: 7")
            check(resultToString(BoltFFIResult.Err("bad")) == "err: bad")
        }
        "vec-processor-callback" -> {
            val vecProcessor = object : VecProcessor {
                override fun process(values: IntArray): IntArray = values.map { it * it }.toIntArray()
            }
            check(processVec(vecProcessor, intArrayOf(1, 2, 3)).contentEquals(intArrayOf(1, 4, 9)))
        }
        "async-callback-traits" -> runBlocking {
            withTimeout(10_000) {
                val asyncFetcher = object : AsyncFetcher {
                    override suspend fun fetchValue(key: Int): Int = key * 100
                    override suspend fun fetchString(input: String): String = input.uppercase()
                }
                val asyncOptionFetcher = object : AsyncOptionFetcher {
                    override suspend fun find(key: Int): Long? = key.takeIf { it > 0 }?.toLong()?.times(1000L)
                }
                check(fetchWithAsyncCallback(asyncFetcher, 5) == 500)
                check(fetchStringWithAsyncCallback(asyncFetcher, "boltffi") == "BOLTFFI")
                check(invokeAsyncOptionFetcher(asyncOptionFetcher, 7) == 7_000L)
                check(invokeAsyncOptionFetcher(asyncOptionFetcher, 0) == null)
            }
        }
        "event-bus-streams" -> runBlocking {
            withTimeout(10_000) {
                EventBus().use { bus ->
                    val valuesDeferred = async {
                        withTimeout(5_000) {
                            bus.subscribeValues().take(4).toList()
                        }
                    }
                    val pointsDeferred = async {
                        withTimeout(5_000) {
                            bus.subscribePoints().take(2).toList()
                        }
                    }
                    delay(100)
                    bus.emitValue(1)
                    check(bus.emitBatch(intArrayOf(2, 3, 4)) == 3u)
                    bus.emitPoint(Point(1.0, 2.0))
                    bus.emitPoint(Point(3.0, 4.0))
                    check(valuesDeferred.await() == listOf(1, 2, 3, 4))
                    val points = pointsDeferred.await()
                    check(points.size == 2)
                    check(points[0] == Point(1.0, 2.0))
                    check(points[1] == Point(3.0, 4.0))
                }
            }
        }
        else -> error("unknown isolated case ${args.singleOrNull()}")
    }
}
