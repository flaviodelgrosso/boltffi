import { assert, demo } from "../support/index.mjs";

export async function run() {
  assert.equal(demo.echoString("hello 🌍"), "hello 🌍");
  assert.equal(demo.concatStrings("foo", "bar"), "foobar");
  assert.equal(demo.stringLength("café"), 5);
  assert.equal(demo.stringIsEmpty(""), true);
  assert.equal(demo.repeatString("ab", 3), "ababab");
}
