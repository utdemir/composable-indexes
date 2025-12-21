PHONY: check format mutation-test coverage coverage-report coverage-open-html

check:
	cargo fmt --check

	env RUSTFLAGS="-D warnings" cargo check --all-targets

	cargo clippy

	cargo test

check-all:
	cargo fmt --check
	env RUSTFLAGS="-D warnings" cargo hack check --feature-powerset --all-targets
	cargo hack clippy --feature-powerset
	cargo hack test --feature-powerset

format:
	cargo fmt

mutation-test:
	cargo mutants -j 2 -p composable-indexes --test-workspace true

coverage:
	cargo llvm-cov clean --workspace
	cargo hack llvm-cov --no-report --each-feature
	cargo llvm-cov report --lcov --output-path coverage.lcov

coverage-report:
	cargo llvm-cov report 

coverage-open-html:
	cargo llvm-cov report --open

bench:
	rm -rf ./target/criterion
	cargo bench  --all-features -- --quick --plotting-backend plotters
	@echo "Benchmarks are saved to ./target/criterion/report/index.html"