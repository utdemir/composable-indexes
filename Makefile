.PHONY: check check-all format mutation-test coverage coverage-report coverage-open-html bench docs

check:
	env RUSTFLAGS="-D warnings" cargo check --all-targets
	cargo clippy
	cargo test
	cargo fmt --check

check-all:
	env RUSTFLAGS="-D warnings" cargo hack check --feature-powerset --all-targets
	cargo test --no-default-features
	cargo test --all-features
	cargo clippy --no-default-features
	cargo clippy --all-features
	cargo fmt --check

format:
	cargo fmt

mutation-test:
	cargo mutants -j 2 -p composable-indexes --test-workspace true

coverage:
	cargo llvm-cov clean --workspace
	cargo llvm-cov --no-report --no-default-features --all-targets
	cargo llvm-cov --no-report --all-features --all-targets
	cargo llvm-cov report --lcov --output-path coverage.lcov

coverage-report:
	cargo llvm-cov report 

coverage-open-html:
	cargo llvm-cov report --open

bench:
	rm -rf ./target/criterion 
	@mkdir -p ./crates/composable-indexes/doc_assets
	cargo bench  --all-features -- --quick --plotting-backend plotters

	rm -rf ./crates/composable-indexes/doc_assets
	cp ./target/criterion/indexing_overhead/report/lines.svg ./crates/composable-indexes/doc_assets/bench_indexing_overhead.svg
	@echo "Benchmarks are saved to ./target/criterion/report/index.html"

docs:
	cargo clean --doc
	cargo doc --open