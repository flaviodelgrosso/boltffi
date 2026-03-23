import { assert, assertArrayEqual, demo } from "../support/index.mjs";

export async function run() {
  assert.equal(await demo.asyncAdd(3, 7), 10);
  assert.equal(await demo.asyncEcho("hello async"), "Echo: hello async");
  assertArrayEqual(await demo.asyncDoubleAll([1, 2, 3]), [2, 4, 6]);
  assert.equal(await demo.asyncFindPositive([-1, 0, 5, 3]), 5);
  assert.equal(await demo.asyncFindPositive([-1, -2, -3]), null);
  assert.equal(await demo.asyncConcat(["a", "b", "c"]), "a, b, c");
}
