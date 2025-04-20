check:
	env RUSTFLAGS="-D warnings" cargo check
	cargo fmt --check
	cargo clippy
	cargo test

mutation-test:
	cargo mutants -j 2 -p composable-indexes-core -p composable-indexes --test-workspace true

coverage:
	cargo llvm-cov --lcov --output-path lcov.info 
	cargo llvm-cov report

coverage-report:
	cargo llvm-cov report

coverage-open-html:
	cargo llvm-cov report --open