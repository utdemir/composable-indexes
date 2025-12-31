# composable-indexes

[![Crates.io](https://img.shields.io/crates/v/composable_indexes.svg)](https://crates.io/crates/composable-indexes)
[![Docs.rs](https://img.shields.io/badge/docs.rs-composable--indexes-blue)](https://docs.rs/composable-indexes)
[![codecov](https://codecov.io/gh/utdemir/composable-indexes/branch/main/graph/badge.svg?token=CYXNRQQ07B)](https://codecov.io/gh/utdemir/composable-indexes)

A Rust library for collections with flexible and composable in-memory indexes. The indexes stay in sync with the collection without any extra effort.

## Features

- Batteries included - built-in indexes for common use cases.
  - Optional support for persistent data structures via the `im` feature.
  - Optional support for Roaring Bitmaps via the `roaring` feature.
- Fast - indexes are backed by performant data structures.
- Composable - build complex indexes from simple ones with combinators.
- Extensible - write your own index and aggregations.
- Safe - Small core, property-based tests, no unsafe.
- Compact - Single runtime dependency (`hashbrown`), `no_std` compatible.

## Example

See [session.rs](https://github.com/utdemir/composable-indexes/blob/main/crates/composable-indexes/examples/session.rs) for an example.

## Notes

### Roadmap

- Transactions (being able to do multiple operations atomically and means to discard changes)
- Fallible operations (i.e., conflicts, postconditions)

### Future & ideas

- On-disk storage backends
- Operations on more than one collection (i.e., foreign keys, joins)
- Change data capture
- Sub-documents, better handling of nested data structures