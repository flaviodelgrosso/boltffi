import { assert, demo } from "../support/index.mjs";

export async function run() {
  const task = demo.makeTask("ship bindings", demo.Priority.Critical);
  assert.deepEqual(demo.echoTask(task), task);
  assert.equal(task.completed, false);
  assert.equal(demo.isUrgent(task), true);

  const notification = { message: "heads up", priority: demo.Priority.High, read: false };
  assert.deepEqual(demo.echoNotification(notification), notification);

  const triangle = demo.makeTriangleHolder();
  assert.equal(triangle.shape.tag, "Triangle");
  assert.deepEqual(demo.echoHolder(triangle), triangle);

  const header = demo.makeCriticalTaskHeader(42n);
  assert.equal(header.id, 42n);
  assert.equal(header.priority, demo.Priority.Critical);
  assert.equal(header.completed, false);
  assert.deepEqual(demo.echoTaskHeader(header), header);

  const logEntry = demo.makeErrorLogEntry(1234567890n, 42);
  assert.equal(logEntry.timestamp, 1234567890n);
  assert.equal(logEntry.level, demo.LogLevel.Error);
  assert.equal(logEntry.code, 42);
  assert.deepEqual(demo.echoLogEntry(logEntry), logEntry);
}
