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
- Safe: Small core, property-based tests, no unsafe.

## Example

```rust
use composable_indexes::*;

struct Person { name: String, age: u32, ssn: String }

let mut collection = Collection::<Person, _>::new(
  index::zip!(
   // A hashtable for the ssn, for exact lookups
   index::premap(|p: &Person| p.ssn.clone(), index::hashtable()),
   
   // A btree index for age, for range lookups
   index::premap(|p: &Person| p.age, index::btree()),

   // Also keep track of the mean age
   index::premap(|p: &Person| p.age, aggregations::mean()),
  )
);

let alice = collection.insert(Person { name: "Alice".to_string(), /* ... */ });
collection.insert(Person { name: "Bob".to_string(), /* ... */ });
collection.adjust_mut(alice, |p| { p.age = 31; });
// ...

// SSN lookup
let _found = collection.query(|ix| ix._1().inner().get_one(&"123-45-6789".to_string()));

// Query the oldest person
let _oldest = collection.query(|ix| ix._2().inner().max_one());

// Query the mean age
let _mean_age = collection.query(|ix| ix._3().get());
```

See more examples at [crates/composable-indexes/examples](./crates/composable-indexes/examples).

## Limitations

- For performance reasons, we do not use boxing or dynamic dispatch. But this flexibility comes with verbose type signatures. 

## Future work

- Fallible operations (ie. conflicts, postconditions) w/transactionality
- Operations on more than one collection (ie, foreign keys, joins)
- Sub-documents, better handling of nested data structures