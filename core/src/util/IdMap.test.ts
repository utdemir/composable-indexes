import test from "node:test";
import { deepStrictEqual } from "node:assert";

import fc from "fast-check";
import { IdMap } from "./IdMap";
import Long from "long";
import { Id } from "../core/simple_types";

test("LongMap", async (t) => {
  await test("ref", () => {
    return fc.assert(
      fc.property(fc.array(arbCall({ value: fc.boolean() })), (calls) => {
        const map = new IdMap<boolean>();
        const ref = new Map<string, boolean>();

        for (const call of calls) {
          switch (call.type) {
            case "set":
              map.set(call.id, call.value);
              ref.set(call.id.asLong.toString(16), call.value);
              break;
            case "delete":
              map.delete(call.id);
              ref.delete(call.id.asLong.toString(16));
              break;
          }
        }

        const actual: [string, boolean][] = [];
        map.forEach((value, key) => {
          actual.push([key.asLong.toString(16), value]);
        });

        const expected = Array.from(ref.entries());

        actual.sort((a, b) => a[0].localeCompare(b[0]));
        expected.sort((a, b) => a[0].localeCompare(b[0]));

        deepStrictEqual(actual, expected);
      }),
      {
        numRuns: 10000,
      }
    );
  });
});

const arbId = fc
  .tuple(
    fc.integer({ min: 0, max: 2 ** 20 - 1 }),
    fc.integer({ min: 0, max: 2 ** 20 - 1 })
  )
  .map(([hi, lo]) => Id.fromLong(new Long(hi, lo, true)));

type Call<T> =
  | {
      type: "set";
      id: Id;
      value: T;
    }
  | {
      type: "delete";
      id: Id;
    };

export function arbCall<T>(args: {
  value: fc.Arbitrary<T>;
}): fc.Arbitrary<Call<T>> {
  return fc.oneof(
    fc.record({
      type: fc.constant<"set">("set"),
      id: arbId,
      value: args.value,
    }),
    fc.record({
      type: fc.constant<"delete">("delete"),
      id: arbId,
    })
  );
}
