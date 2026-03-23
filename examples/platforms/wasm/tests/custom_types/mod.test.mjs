import { assert, demo } from "../support/index.mjs";

export async function run() {
  const email = "ali@example.com";
  assert.equal(demo.echoEmail(email), email);
  assert.equal(demo.emailDomain(email), "example.com");

  const datetime = 1_701_234_567_890n;
  assert.equal(demo.echoDatetime(datetime), datetime);
  assert.equal(demo.datetimeToMillis(datetime), datetime);
}
