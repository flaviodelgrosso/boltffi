import { assert, assertApprox, demo } from "../support/index.mjs";

export async function run() {
  assert.equal(demo.echoBool(true), true);
  assert.equal(demo.negateBool(false), true);
  assert.equal(demo.echoI8(-7), -7);
  assert.equal(demo.echoU8(255), 255);
  assert.equal(demo.echoI16(-1234), -1234);
  assert.equal(demo.echoU16(55_000), 55_000);
  assert.equal(demo.echoI32(-42), -42);
  assert.equal(demo.addI32(10, 20), 30);
  assert.equal(demo.echoU32(2_147_483_647), 2_147_483_647);
  assert.equal(demo.echoI64(-9_999_999_999n), -9_999_999_999n);
  assert.equal(demo.echoU64(9_999_999_999n), 9_999_999_999n);
  assertApprox(demo.echoF32(3.5), 3.5, 1e-6);
  assertApprox(demo.addF32(1.5, 2.5), 4.0, 1e-6);
  assertApprox(demo.echoF64(3.14159265359), 3.14159265359, 1e-12);
  assertApprox(demo.addF64(1.5, 2.5), 4.0, 1e-12);
  assert.equal(demo.echoUsize(123), 123);
  assert.equal(demo.echoIsize(-123), -123);
}
