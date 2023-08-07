import { FoldIndex } from "./FoldIndex";
import assert from 'assert';
import test from "node:test";

test("Test FoldIndex method", () => {
  const foldIndex = new FoldIndex();
  const result = foldIndex.calculate(); // replaced 'method' with 'calculate'
  assert.strictEqual(result, 10); // replaced 'expected_result' with 10
});