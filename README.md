# composable-indexes

[![Crates.io](https://img.shields.io/crates/v/composable_indexes.svg)](https://crates.io/composable-indexes)
[![Docs.rs](https://img.shields.io/badge/docs.rs-composable--indexes-blue)](https://docs.rs/composable-indexes)
[![codecov](https://codecov.io/gh/utdemir/composable-indexes/branch/main/graph/badge.svg?token=CYXNRQQ07B)](https://codecov.io/gh/utdemir/composable-indexes)

A Rust library for collections with flexible and composable in-memory indexes. The indexes stay in sync with the collection without any extra effort.

## Features

- Batteries included - built-in indexes for common use cases.
- Fast - indexes are backed by performant data structures.
- Composable - build complex indexes from simple ones with combinators.
- Extensible - write your own index and aggregations.
- Safe - Small core, property-based tests, no unsafe.
- Compact - No runtime dependencies (only `hashbrown` for `no_std`).

## Example

See [crates/composable-indexes/examples/session.rs](./crates/composable-indexes/examples/session.rs).

## Notes

### Limitations

- For performance reasons, we do not use boxing or dynamic dispatch. But this flexibility comes with verbose type signatures. 

### Future work

- Fallible operations (ie. conflicts, postconditions) w/transactionality
- Operations on more than one collection (ie, foreign keys, joins)
- Sub-documents, better handling of nested data structures