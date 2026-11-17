# CI checks - run locally before pushing to verify branch is green
ci-check: check fmt clippy test

# Check compilation
check:
    cargo check

# Format check
fmt:
    cargo fmt --all -- --check

# Linter
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Tests
test:
    cargo test --verbose --all --locked

# Detect unused dependencies
shear:
    cargo shear

# Documentation check
docs:
    cargo doc --no-deps --all-features

# Help
help:
    @just --list
