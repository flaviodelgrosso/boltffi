import { assert, assertPoint, assertThrowsWithMessage, demo } from "../support/index.mjs";

export async function run() {
  const counter = demo.Counter.new(2);
  assert.equal(counter.get(), 2);
  counter.increment();
  assert.equal(counter.get(), 3);
  counter.add(7);
  assert.equal(counter.get(), 10);
  assert.equal(counter.tryGetPositive(), 10);
  assert.equal(counter.maybeDouble(), 20);
  assertPoint(counter.asPoint(), { x: 10, y: 0 });
  counter.reset();
  assert.equal(counter.get(), 0);
  assert.equal(counter.maybeDouble(), null);
  assertThrowsWithMessage(() => counter.tryGetPositive(), "count is not positive");
  counter.dispose();
}
