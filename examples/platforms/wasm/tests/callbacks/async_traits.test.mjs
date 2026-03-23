import { assert, demo } from "../support/index.mjs";

export async function run() {
  const asyncFetcher = {
    fetchValue: async (key) => key * 100,
    fetchString: async (input) => input.toUpperCase(),
  };
  const asyncOptionFetcher = {
    find: async (key) => (key > 0 ? BigInt(key) * 1000n : null),
  };

  assert.equal(await demo.fetchWithAsyncCallback(asyncFetcher, 5), 500);
  assert.equal(await demo.fetchStringWithAsyncCallback(asyncFetcher, "boltffi"), "BOLTFFI");
  assert.equal(await demo.invokeAsyncOptionFetcher(asyncOptionFetcher, 7), 7_000n);
  assert.equal(await demo.invokeAsyncOptionFetcher(asyncOptionFetcher, 0), null);

  const asyncFetcherHandle = demo.registerAsyncFetcher(asyncFetcher);
  const asyncOptionFetcherHandle = demo.registerAsyncOptionFetcher(asyncOptionFetcher);

  assert.ok(asyncFetcherHandle > 0);
  assert.ok(asyncOptionFetcherHandle > 0);

  demo.unregisterAsyncFetcher(asyncFetcherHandle);
  demo.unregisterAsyncOptionFetcher(asyncOptionFetcherHandle);
}
