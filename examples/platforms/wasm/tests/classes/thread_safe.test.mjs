import { assert, demo } from "../support/index.mjs";

export async function run() {
  const counter = demo.SharedCounter.new(5);
  assert.equal(counter.get(), 5);
  counter.set(6);
  assert.equal(counter.get(), 6);
  assert.equal(counter.increment(), 7);
  assert.equal(counter.add(3), 10);
  assert.equal(await counter.asyncGet(), 10);
  assert.equal(await counter.asyncAdd(5), 15);
  counter.dispose();
}
