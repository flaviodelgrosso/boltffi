import { assert, demo } from "../support/index.mjs";

export async function run() {
  const person = demo.makePerson("Ali", 30);
  assert.deepEqual(demo.echoPerson(person), person);
  assert.equal(demo.greetPerson(person), "Hello, Ali! You are 30 years old.");

  const address = { street: "Main St", city: "Amsterdam", zip: "1000AA" };
  assert.deepEqual(demo.echoAddress(address), address);
  assert.equal(demo.formatAddress(address), "Main St, Amsterdam, 1000AA");
}
