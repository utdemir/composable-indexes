# AGENTS.md - Developer Guide for composable-indexes

## Table of Contents
1. [Project Overview](#project-overview)
2. [Repository Structure](#repository-structure)
3. [Makefile Targets](#makefile-targets)
4. [How Indexes Work](#how-indexes-work)
5. [Testing Infrastructure](#testing-infrastructure)
6. [Development Setup](#development-setup)
7. [Examples and Usage Patterns](#examples-and-usage-patterns)
8. [Best Practices](#best-practices)

---

## Project Overview

**composable-indexes** is a Rust library that provides in-memory collections with flexible and composable indexes. The library automatically keeps indexes in sync with the collection without requiring manual maintenance.

### Key Features
- **Batteries included**: Built-in indexes for common use cases (BTree, HashTable)
- **Fast**: Backed by performant data structures (hashbrown)
- **Composable**: Build complex indexes from simple ones using combinators
- **Extensible**: Create custom indexes and aggregations
- **Safe**: Small core, property-based tests, no unsafe code
- **Compact**: Single runtime dependency (`hashbrown`), `no_std` compatible
- **Optional features**: Persistent data structures via the `im` feature

### Crates in the Workspace
- **composable-indexes**: Main library implementing the collection and index system
- **composable-indexes-derive**: Procedural macros for deriving Index and ShallowClone traits

---

## Repository Structure

```
composable-indexes/
├── Cargo.toml              # Workspace configuration
├── Makefile                # Development automation
├── README.md               # User-facing documentation
├── AGENTS.md               # This file - Developer documentation
└── crates/
    ├── composable-indexes/          # Main library
    │   ├── Cargo.toml               # Package configuration
    │   ├── src/
    │   │   ├── lib.rs               # Library entry point
    │   │   ├── core/                # Core abstractions
    │   │   │   ├── collection.rs    # Collection implementation
    │   │   │   ├── index.rs         # Index trait definition
    │   │   │   ├── store.rs         # Storage abstraction
    │   │   │   └── ...
    │   │   ├── index/               # Index implementations
    │   │   │   ├── btree.rs         # BTree-based index
    │   │   │   ├── hashtable.rs     # HashMap-based index
    │   │   │   ├── premap.rs        # Transform before indexing
    │   │   │   ├── grouped.rs       # Grouped/multi-value index
    │   │   │   ├── filtered.rs      # Conditional indexing
    │   │   │   ├── zip.rs           # Combine multiple indexes
    │   │   │   └── ...
    │   │   └── aggregation/         # Aggregation indexes
    │   │       ├── stats.rs         # Count, sum, mean, etc.
    │   │       └── generic.rs       # Generic aggregation
    │   ├── examples/                # Usage examples
    │   │   ├── session.rs           # Session store example
    │   │   ├── session_im.rs        # With persistent data structures
    │   │   ├── session_noderive.rs  # Without derive macros
    │   │   └── graph.rs             # Graph data structure
    │   ├── tests/                   # Integration tests
    │   │   └── index_selection.rs
    │   └── benches/                 # Benchmarks
    │       ├── insertion_vs_hashmap.rs
    │       └── query_vs_sqlite.rs
    └── composable-indexes-derive/   # Derive macros
        ├── Cargo.toml
        ├── src/
        │   ├── lib.rs
        │   ├── derive_index.rs      # #[derive(Index)] macro
        │   └── derive_shallow_clone.rs
        └── tests/
```

---

## Makefile Targets

The Makefile provides convenient commands for common development tasks:

### Basic Checks
```bash
make check
```
Runs the standard quality checks:
1. `cargo fmt --check` - Verify code formatting
2. `cargo check --all-targets` - Check compilation (with warnings as errors)
3. `cargo clippy` - Run linter
4. `cargo test` - Run all tests

This is the primary command to run before committing changes.

### Comprehensive Testing
```bash
make check-all
```
Runs extensive checks across all feature combinations:
1. `cargo fmt --check` - Verify code formatting
2. `cargo hack check --feature-powerset --all-targets` - Check all feature combinations
3. `cargo hack clippy --feature-powerset` - Lint all feature combinations
4. `cargo hack test --feature-powerset` - Test all feature combinations

This is more thorough but slower than `make check`. Useful before major releases.

### Code Formatting
```bash
make format
```
Automatically formats all Rust code using `rustfmt`.

### Mutation Testing
```bash
make mutation-test
```
Runs mutation testing using `cargo-mutants` to verify test effectiveness:
- Tests the `composable-indexes` package
- Uses 2 parallel jobs (`-j 2`)
- Tests the entire workspace (`--test-workspace true`)

Mutation testing helps identify weak tests by introducing small changes (mutations) to the code and checking if tests catch them.

### Coverage Analysis
```bash
make coverage
```
Generates code coverage reports:
1. Cleans previous coverage data
2. Runs tests with coverage for each feature
3. Generates an LCOV report (`coverage.lcov`)

```bash
make coverage-report
```
Displays coverage summary in the terminal.

```bash
make coverage-open-html
```
Generates and opens an HTML coverage report in the browser.

---

## How Indexes Work

### Core Concepts

The library is built around three main abstractions:

#### 1. Collection
A `Collection<In, Ix>` stores items of type `In` and maintains an index of type `Ix`. It provides:
- **Insertion**: Add new items with `insert(value) -> Key`
- **Updates**: Modify items with `update_by_key()`, `adjust_by_key()`, etc.
- **Deletion**: Remove items with `delete_by_key()` or `delete(query)`
- **Queries**: Read indexed data with `query(|ix| ...)`

#### 2. Index Trait
All indexes implement the `Index<In>` trait:
```rust
pub trait Index<In> {
    fn insert(&mut self, op: &Insert<In>);
    fn remove(&mut self, op: &Remove<In>);
    fn update(&mut self, op: &Update<In>) { /* default impl */ }
}
```

When items are added, removed, or modified in a collection, the collection automatically calls the appropriate index methods to keep the index in sync.

#### 3. Key
Each item in a collection gets a unique `Key`:
```rust
pub struct Key {
    pub id: u64,
}
```

Keys are used to identify and retrieve specific items.

### Built-in Indexes

#### Basic Indexes
1. **Unit/Empty** (`()`): Does nothing, useful as a placeholder
2. **Keys** (`keys()`): Tracks all keys in the collection
3. **BTree** (`btree()`): Orders items, supports range queries
4. **HashTable** (`hashtable()`): Fast lookups by value

#### Index Combinators

These transform or combine indexes to create more complex behavior:

1. **Premap** (`premap(f, inner_index)`):
   - Transforms items before indexing
   - Example: `premap(|person| person.age, btree())` indexes people by age

2. **Grouped** (`grouped(key_fn, inner_index_fn)`):
   - Groups items by a key
   - Each group maintains its own inner index
   - Example: `grouped(|session| session.user_id, || keys())` groups sessions by user

3. **Filtered** (`filtered(predicate, inner_index)`):
   - Only indexes items matching a predicate
   - Example: `filtered(|person| person.age >= 18, keys())` indexes only adults

4. **Zip** (`zip!(index1, index2, ...)`):
   - Combines multiple indexes into one
   - Allows querying different aspects of the same collection
   - Supports up to 10 indexes (ZipIndex2 through ZipIndex10)

### Aggregation Indexes

Aggregation indexes compute statistics over the collection:

- **CountIndex** (`count()`): Counts items
- **SumIndex** (`sum()`): Sums numeric values
- **MeanIndex** (`mean()`): Calculates average
- **MinIndex** (`min()`) / **MaxIndex** (`max()`): Finds extremes
- **AggregateIndex**: Generic aggregation with custom operations

### How Indexes Stay in Sync

When you modify a collection:
1. The collection calls the index's `insert`, `remove`, or `update` method
2. Each index updates its internal data structures
3. For composite indexes (like ZipIndex), the operation propagates to all sub-indexes
4. Transforming indexes (like PremapIndex) apply their transformations and forward to inner indexes

This happens automatically—you never need to manually update indexes.

### Example: Session Store

```rust
use composable_indexes::{Collection, index, aggregation};

struct Session {
    session_id: String,
    user_id: u32,
    expiration_time: SystemTime,
    country: String,
}

// Derive macro creates a composite index
#[derive(composable_indexes::Index)]
#[index(Session)]
struct SessionIndex {
    // Look up by session ID
    by_session_id: index::PremapIndex<Session, String, index::HashTableIndex<String>>,
    
    // Range queries on expiration
    by_expiration: index::PremapIndex<Session, SystemTime, index::BTreeIndex<SystemTime>>,
    
    // All sessions per user
    by_user_id: index::GroupedIndex<Session, u32, index::KeysIndex>,
    
    // Count sessions per country
    by_country: index::GroupedIndex<Session, String, aggregation::CountIndex>,
}

// Manual construction (without derive macro)
fn new_index() -> SessionIndex {
    SessionIndex {
        by_session_id: index::premap(|s: &Session| s.session_id.clone(), index::hashtable()),
        by_expiration: index::premap(|s: &Session| s.expiration_time, index::btree()),
        by_user_id: index::grouped(|s: &Session| s.user_id, || index::keys()),
        by_country: index::grouped(|s: &Session| s.country.clone(), || aggregation::count()),
    }
}
```

---

## Testing Infrastructure

### Test Organization

1. **Unit Tests**: In `src/` files alongside the code (using `#[cfg(test)]`)
2. **Integration Tests**: In `crates/composable-indexes/tests/`
3. **Doc Tests**: In documentation comments (run with `cargo test --doc`)
4. **Property-Based Tests**: Using `proptest` for randomized testing

### Test Utilities

The `testutils` feature provides utilities for testing indexes:
- Enable with `features = ["testutils"]` in `Cargo.toml`
- Provides `test_index()` - a test index that records all operations
- Used to verify that collections call index methods correctly

Example:
```rust
use composable_indexes::{Collection, testutils::test_index};

#[test]
fn test_collection() {
    let mut db = Collection::<u32, _>::new(test_index());
    let key = db.insert(42);
    
    // Access recorded operations
    let ops = db.query(|ix| ix.ops.clone());
    // Verify operations are correct
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific package
cargo test -p composable-indexes

# Run a specific test
cargo test test_name

# Run tests with all features
cargo test --all-features

# Run tests with specific features
cargo test --features "imbl,derive"

# Run tests without default features
cargo test --no-default-features

# Run doc tests only
cargo test --doc

# Run tests with output
cargo test -- --nocapture

# Run tests in parallel (default)
cargo test -- --test-threads=4
```

### Feature Flags

- `std` (default): Standard library support
- `derive` (default): Enable derive macros
- `testutils`: Testing utilities (for internal use)
- `imbl`: Support for persistent data structures

### Writing Tests

When writing tests:
1. Test index behavior independently
2. Test collection operations with different index types
3. Use property-based testing for complex scenarios
4. Verify that indexes stay in sync with the collection
5. Test edge cases (empty collections, single items, etc.)

Example property-based test pattern:
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_property(values: Vec<u32>) {
        // Test logic here
    }
}
```

### Benchmarks

Run benchmarks to measure performance:
```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench insertion_vs_hashmap
```

Benchmarks compare composable-indexes against:
- Raw HashMap (insertion performance)
- SQLite (query performance)

---

## Development Setup

### Prerequisites

1. **Rust**: Install via [rustup](https://rustup.rs/)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Additional Tools**: Install cargo extensions
   ```bash
   # For running Makefile targets
   cargo install cargo-hack cargo-llvm-cov cargo-mutants
   ```

### First-Time Setup

```bash
# Clone the repository
git clone https://github.com/utdemir/composable-indexes.git
cd composable-indexes

# Build the project
cargo build

# Run tests
cargo test

# Run quality checks
make check
```

### Development Workflow

1. **Before Starting Work**:
   ```bash
   git checkout main
   git pull
   cargo build
   make check  # Ensure everything works
   ```

2. **While Developing**:
   ```bash
   # Format code frequently
   make format
   
   # Run tests after changes
   cargo test
   
   # Check compilation and lints
   make check
   ```

3. **Before Committing**:
   ```bash
   make check  # Must pass
   ```

4. **Before Opening PR**:
   ```bash
   make check-all     # Test all feature combinations
   make coverage      # Check test coverage
   ```

### IDE Setup

#### Visual Studio Code
Recommended extensions:
- `rust-analyzer`: Rust language server
- `crates`: Cargo.toml helper
- `Better TOML`: TOML syntax

Settings (`.vscode/settings.json`):
```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "editor.formatOnSave": true
}
```

#### IntelliJ IDEA / CLion
- Install the Rust plugin
- Enable rustfmt on save
- Configure external tools for Makefile targets

---

## Examples and Usage Patterns

### Running Examples

```bash
# Run the session store example
cargo run --example session

# Run with persistent data structures
cargo run --example session_im

# Run without derive macros
cargo run --example session_noderive

# Run graph example
cargo run --example graph
```

### Common Patterns

#### 1. Simple Index
```rust
use composable_indexes::{Collection, index};

let mut collection = Collection::<i32, _>::new(index::btree());
collection.insert(42);
collection.insert(10);

let max = collection.query(|ix| ix.max_one());  // Some(&42)
```

#### 2. Transformed Index
```rust
struct Person { name: String, age: u32 }

let mut people = Collection::<Person, _>::new(
    index::premap(|p: &Person| p.age, index::btree())
);

let oldest = people.query(|ix| ix.max_one());
```

#### 3. Multiple Indexes
```rust
let mut people = Collection::<Person, _>::new(index::zip!(
    index::premap(|p: &Person| p.name.clone(), index::hashtable()),
    index::premap(|p: &Person| p.age, index::btree()),
));

// Query by name
let alice = people.query(|ix| ix._1().get_one(&"Alice".to_string()));

// Query by age range
let adults = people.query(|ix| ix._2().range(18..));
```

#### 4. Grouped Index
```rust
struct Task { project: String, status: String }

let mut tasks = Collection::<Task, _>::new(
    index::grouped(
        |t: &Task| t.project.clone(),
        || index::grouped(
            |t: &Task| t.status.clone(),
            || aggregation::count()
        )
    )
);

// Count tasks by project and status
let count = tasks.query(|ix| 
    ix.get(&"ProjectA".to_string())
      .get(&"done".to_string())
      .get()
);
```

#### 5. Custom Index

To implement a custom index:
```rust
use composable_indexes::core::{Index, Insert, Remove, Update};

struct MyIndex {
    // Internal state
}

impl<In> Index<In> for MyIndex {
    fn insert(&mut self, op: &Insert<In>) {
        // Update index when item inserted
    }
    
    fn remove(&mut self, op: &Remove<In>) {
        // Update index when item removed
    }
    
    fn update(&mut self, op: &Update<In>) {
        // Optional: optimize updates
        // Default removes then inserts
    }
}
```

---

## Best Practices

### Code Style

1. **Follow Rust conventions**: Use `rustfmt` (run `make format`)
2. **Use clippy**: Fix all clippy warnings (run `cargo clippy`)
3. **Write documentation**: Document public APIs with doc comments
4. **Include examples**: Add examples in doc comments

### Performance

1. **Use appropriate indexes**: 
   - HashTable for lookups
   - BTree for ranges
   - Grouped for one-to-many relationships

2. **Avoid unnecessary transformations**: Minimize work in `premap` closures

3. **Profile before optimizing**: Use benchmarks to guide optimization

### Testing

1. **Test index behavior**: Ensure indexes produce correct results
2. **Test collection operations**: Verify inserts, updates, deletes work
3. **Use property-based tests**: For complex scenarios
4. **Test feature combinations**: Use `cargo hack` for thorough testing

### Type Signatures

The library's flexibility results in verbose type signatures. Strategies:

1. **Use type aliases**:
   ```rust
   type PersonIndex = index::PremapIndex<Person, u32, index::BTreeIndex<u32>>;
   ```

2. **Use derive macro** (when available):
   ```rust
   #[derive(composable_indexes::Index)]
   #[index(Person)]
   struct PersonIndex { /* ... */ }
   ```

3. **Let type inference help**: Often you don't need to write the full type

### Contributing

1. **Check existing issues**: Before starting work
2. **Open an issue first**: For significant changes
3. **Write tests**: All new features need tests
4. **Update documentation**: Keep docs in sync with code
5. **Run quality checks**: `make check` must pass
6. **Test feature combinations**: `make check-all` for significant changes

### Debugging

1. **Use test_index()**: To see what operations are called on indexes
2. **Enable logging**: Add `println!` or use the `log` crate
3. **Run single tests**: Isolate failing tests with `cargo test test_name`
4. **Check examples**: See if examples still work

---

## Additional Resources

- **Crates.io**: https://crates.io/crates/composable-indexes
- **Docs.rs**: https://docs.rs/composable-indexes
- **GitHub**: https://github.com/utdemir/composable-indexes
- **Issues**: https://github.com/utdemir/composable-indexes/issues

## Future Work

Areas for potential development:

1. **Fallible operations**: Support conflicts, postconditions, and transactionality
2. **Multi-collection operations**: Foreign keys, joins
3. **Sub-documents**: Better handling of nested data structures
4. **More aggregations**: Additional statistical operations
5. **Query optimization**: More efficient query planning

---

*This document is maintained as a living guide for developers working on composable-indexes.*
