test:
	cargo test --workspace --all-features

# Verify the workspace compiles without `std` (the Wasm/runtime deployment path).
check-no-std:
	cargo check --workspace --no-default-features

# Standard pre-commit gate: unit tests plus the no_std build check.
ci: test check-no-std
