PHONY: check format mutation-test coverage coverage-report coverage-open-html

check:
	cargo fmt --check

	env RUSTFLAGS="-D warnings" cargo check

	cargo clippy

	cargo test

check-all:
	cargo fmt --check
	env RUSTFLAGS="-D warnings" cargo hack check --feature-powerset
	cargo hack clippy --feature-powerset
	cargo hack test --feature-powerset
	
format:
	cargo fmt

mutation-test:
	cargo mutants -j 2 -p composable-indexes --test-workspace true

coverage:
	cargo llvm-cov --lcov --output-path lcov.info 
	cargo llvm-cov report

coverage-report:
	cargo llvm-cov report

coverage-open-html:
	cargo llvm-cov report --open