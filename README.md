# composable-indexes

![Crates.io](https://img.shields.io/crates/v/composable_indexes.svg)
![Docs.rs](https://img.shields.io/badge/docs.rs-composable--indexes-blue)
[![codecov](https://codecov.io/gh/utdemir/composable-indexes/branch/main/graph/badge.svg?token=CYXNRQQ07B)](https://codecov.io/gh/utdemir/composable-indexes)

A Rust library for in-memory collections with flexible and composable indexes.

## Features

- Batteries included - built-in indexes for common use cases.
- Fast - indexes are backed by performant data structures.
- Composable - build complex indexes from simple ones with combinators.
- Extensible - write your own index and aggregations.
- Safe: Small core, property-based testing.

## Usage

```rust
use composable_indexes::*;

fn main() {
   let mut collection = Collection::new();
}