import { assert, assertArrayEqual, assertPoint, demo } from "../support/index.mjs";

export async function run() {
  const doubler = { onValue: (value) => value * 2 };
  const tripler = { onValue: (value) => value * 3 };
  const pointTransformer = { transform: (point) => ({ x: point.x + 10, y: point.y + 20 }) };
  const statusMapper = {
    mapStatus: (status) => (status === demo.Status.Pending ? demo.Status.Active : demo.Status.Inactive),
  };
  const multiMethodCallback = {
    methodA: (value) => value + 1,
    methodB: (left, right) => left * right,
    methodC: () => 5,
  };
  const optionCallback = {
    findValue: (key) => (key > 0 ? key * 10 : null),
  };
  const vecProcessor = {
    process: (values) => values.map((value) => value * value),
  };

  assert.equal(demo.invokeValueCallback(doubler, 4), 8);
  assert.equal(demo.invokeValueCallbackTwice(doubler, 3, 4), 14);
  assert.equal(demo.invokeBoxedValueCallback(doubler, 5), 10);
  assertPoint(demo.transformPoint(pointTransformer, { x: 1, y: 2 }), { x: 11, y: 22 });
  assertPoint(demo.transformPointBoxed(pointTransformer, { x: 3, y: 4 }), { x: 13, y: 24 });
  assert.equal(demo.mapStatus(statusMapper, demo.Status.Pending), demo.Status.Active);
  assertArrayEqual(demo.processVec(vecProcessor, [1, 2, 3]), [1, 4, 9]);
  assert.equal(demo.invokeMultiMethod(multiMethodCallback, 3, 4), 21);
  assert.equal(demo.invokeMultiMethodBoxed(multiMethodCallback, 3, 4), 21);
  assert.equal(demo.invokeTwoCallbacks(doubler, tripler, 5), 25);
  assert.equal(demo.invokeOptionCallback(optionCallback, 7), 70);
  assert.equal(demo.invokeOptionCallback(optionCallback, 0), null);

  const valueCallbackHandle = demo.registerValueCallback(doubler);
  const pointTransformerHandle = demo.registerPointTransformer(pointTransformer);
  const statusMapperHandle = demo.registerStatusMapper(statusMapper);
  const vecProcessorHandle = demo.registerVecProcessor(vecProcessor);
  const multiMethodCallbackHandle = demo.registerMultiMethodCallback(multiMethodCallback);
  const optionCallbackHandle = demo.registerOptionCallback(optionCallback);

  assert.ok(valueCallbackHandle > 0);
  assert.ok(pointTransformerHandle > 0);
  assert.ok(statusMapperHandle > 0);
  assert.ok(vecProcessorHandle > 0);
  assert.ok(multiMethodCallbackHandle > 0);
  assert.ok(optionCallbackHandle > 0);

  demo.unregisterValueCallback(valueCallbackHandle);
  demo.unregisterPointTransformer(pointTransformerHandle);
  demo.unregisterStatusMapper(statusMapperHandle);
  demo.unregisterVecProcessor(vecProcessorHandle);
  demo.unregisterMultiMethodCallback(multiMethodCallbackHandle);
  demo.unregisterOptionCallback(optionCallbackHandle);
}
