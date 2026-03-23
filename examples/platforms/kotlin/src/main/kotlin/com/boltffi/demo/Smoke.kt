package com.boltffi.demo

private fun requireThat(condition: Boolean, message: String) {
    if (!condition) {
        throw IllegalStateException(message)
    }
}

fun main() {
    requireThat(
        echoVecIsize(longArrayOf(-3L, 0L, 9L)).contentEquals(longArrayOf(-3L, 0L, 9L)),
        "echoVecIsize failed",
    )
    requireThat(
        echoVecUsize(longArrayOf(0L, 7L, 42L)).contentEquals(longArrayOf(0L, 7L, 42L)),
        "echoVecUsize failed",
    )

    Inventory.tryNew(3u).use { inventory ->
        requireThat(inventory.capacity() == 3u, "Inventory.tryNew capacity failed")
        requireThat(inventory.count() == 0u, "Inventory.tryNew initial count failed")
        requireThat(inventory.add("alpha"), "Inventory.add failed")
        requireThat(inventory.count() == 1u, "Inventory.count after add failed")
    }

    val toggled = mapStatus(
        mapper = object : StatusMapper {
            override fun mapStatus(status: Status): Status =
                if (status == Status.ACTIVE) Status.INACTIVE else Status.ACTIVE
        },
        status = Status.ACTIVE,
    )
    requireThat(toggled == Status.INACTIVE, "mapStatus callback failed")

    requireThat(Direction.cardinal() == Direction.NORTH, "Direction.cardinal failed")
    requireThat(oppositeDirection(Direction.NORTH) == Direction.SOUTH, "oppositeDirection failed")

    val origin = Point.origin()
    requireThat(origin == Point(0.0, 0.0), "Point.origin failed")

    println("Kotlin smoke test passed")
}
