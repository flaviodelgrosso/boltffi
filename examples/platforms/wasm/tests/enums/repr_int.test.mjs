import { assert, assertArrayEqual, demo } from "../support/index.mjs";

export async function run() {
  assert.equal(demo.echoPriority(demo.Priority.High), demo.Priority.High);
  assert.equal(demo.priorityLabel(demo.Priority.Low), "low");
  assert.equal(demo.isHighPriority(demo.Priority.Critical), true);
  assert.equal(demo.isHighPriority(demo.Priority.Low), false);
  assert.equal(demo.echoLogLevel(demo.LogLevel.Info), demo.LogLevel.Info);
  assert.equal(demo.shouldLog(demo.LogLevel.Error, demo.LogLevel.Warn), true);
  assertArrayEqual(
    demo.echoVecLogLevel(Uint8Array.from([demo.LogLevel.Trace, demo.LogLevel.Info, demo.LogLevel.Error])),
    [demo.LogLevel.Trace, demo.LogLevel.Info, demo.LogLevel.Error],
  );
}
