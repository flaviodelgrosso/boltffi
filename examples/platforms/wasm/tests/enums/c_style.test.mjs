import { assert, assertArrayEqual, demo } from "../support/index.mjs";

export async function run() {
  assert.equal(demo.echoStatus(demo.Status.Active), demo.Status.Active);
  assert.equal(demo.statusToString(demo.Status.Active), "active");
  assert.equal(demo.isActive(demo.Status.Pending), false);
  assertArrayEqual(demo.echoVecStatus([demo.Status.Active, demo.Status.Pending]), [demo.Status.Active, demo.Status.Pending]);
  assert.equal(demo.echoDirection(demo.Direction.East), demo.Direction.East);
  assert.equal(demo.oppositeDirection(demo.Direction.East), demo.Direction.West);
}
