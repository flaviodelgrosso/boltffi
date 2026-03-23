import { assert, assertPoint, demo } from "../support/index.mjs";

export async function run() {
  assertPoint(demo.echoPoint({ x: 1, y: 2 }), { x: 1, y: 2 });
  assertPoint(demo.tryMakePoint(2, 3), { x: 2, y: 3 });
  assert.equal(demo.tryMakePoint(0, 0), null);
  assertPoint(demo.makePoint(1, 2), { x: 1, y: 2 });
  assertPoint(demo.addPoints({ x: 3, y: 4 }, { x: 5, y: 6 }), { x: 8, y: 10 });

  const color = { r: 1, g: 2, b: 3, a: 255 };
  assert.deepEqual(demo.echoColor(color), color);
  assert.deepEqual(demo.makeColor(9, 8, 7, 6), { r: 9, g: 8, b: 7, a: 6 });
}
