import { assert, demo } from "../support/index.mjs";

export async function run() {
  const task = demo.makeTask("ship bindings", demo.Priority.Critical);
  assert.deepEqual(demo.echoTask(task), task);
  assert.equal(task.completed, false);
  assert.equal(demo.isUrgent(task), true);

  const notification = { message: "heads up", priority: demo.Priority.High, read: false };
  assert.deepEqual(demo.echoNotification(notification), notification);
}
