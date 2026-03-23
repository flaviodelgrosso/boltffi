import { assert, assertArrayEqual, demo } from "../support/index.mjs";

export async function run() {
  const inventory = demo.Inventory.new();
  assert.equal(inventory.capacity(), 100);
  assert.equal(inventory.count(), 0);
  assert.equal(inventory.add("hammer"), true);
  assertArrayEqual(inventory.getAll(), ["hammer"]);
  assert.equal(inventory.remove(0), "hammer");
  assert.equal(inventory.remove(0), null);
  inventory.dispose();

  const fixedCapacityInventory = demo.Inventory.withCapacity(2);
  assert.equal(fixedCapacityInventory.capacity(), 2);
  assert.equal(fixedCapacityInventory.add("a"), true);
  assert.equal(fixedCapacityInventory.add("b"), true);
  assert.equal(fixedCapacityInventory.add("c"), false);
  assertArrayEqual(fixedCapacityInventory.getAll(), ["a", "b"]);
  fixedCapacityInventory.dispose();

  const tinyInventory = demo.Inventory.tryNew(1);
  assert.notEqual(tinyInventory, null);
  assert.equal(tinyInventory.capacity(), 1);
  assert.equal(tinyInventory.add("only"), true);
  assert.equal(tinyInventory.add("overflow"), false);
  tinyInventory.dispose();

  assert.equal(demo.Inventory.tryNew(0), null);
}
