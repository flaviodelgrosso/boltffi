import { assertArrayEqual, demo } from "../support/index.mjs";

export async function run() {
  assertArrayEqual(demo.echoBytes(Uint8Array.from([1, 2, 3, 4])), [1, 2, 3, 4]);
  assertArrayEqual(demo.echoBytes(Uint8Array.from([])), []);
  assertArrayEqual(demo.makeBytes(4), [0, 1, 2, 3]);
  assertArrayEqual(demo.reverseBytes(Uint8Array.from([1, 2, 3, 4])), [4, 3, 2, 1]);
  if (demo.bytesLength(Uint8Array.from([9, 8, 7])) !== 3) {
    throw new Error("bytesLength returned incorrect count");
  }
  if (demo.bytesSum(Uint8Array.from([1, 2, 3, 4])) !== 10) {
    throw new Error("bytesSum returned incorrect sum");
  }
}
