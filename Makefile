check:
	env RUSTFLAGS="-D warnings" cargo check
	cargo fmt --check
	cargo test

mutation-test:
	cargo mutants -j 2 -p composable-indexes

coverage-machine:
	cargo llvm-cov --lcov --output-path lcov.info 

coverage-html:
	cargo llvm-cov --open