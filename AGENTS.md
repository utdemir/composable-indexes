# AGENTS.md - Developer Guide for composable-indexes

## Crates in the Workspace
- **composable-indexes**: Main library implementing the collection and index system
- **composable-indexes-derive**: Procedural macros for deriving Index and ShallowClone traits

## CI

- CI is using GitHub Actions

## Development tasks

Use top-level Makefile:

```
make check # Checks & tests the default feature set
make check-all # Checks all feature combinations
make format # Formats the code
make bench # Runs benchmarks (and vendors images from the benchmarks to the repo)
```

## Development notes

- Ensure that `make check-all` passes after making changes.
- Top-level `crates/composable-indexes/src/lib.rs` contains the main documentation. Make sure to read & update it as needed.
- When importing indexes, import `index` and `aggregations` qualified - e.g. prefer refering to things like `index::Premap` rather than `Premap` directly.