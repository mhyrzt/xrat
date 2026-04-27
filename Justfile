# xrat Justfile

_default:
    @just --list

# Build the project
build:
    cargo build

# Build in release mode
release:
    cargo build --release

# Install xrat with cargo from this checkout
install:
    cargo install --path .

# Install xrat with cargo from this checkout, replacing an existing binary
reinstall:
    cargo install --path . --force

# Remove the cargo-installed xrat binary
uninstall:
    cargo uninstall xrat

# Run tests quietly
test:
    cargo test -q

# Format Rust code and markdown
fmt:
    cargo fmt
    prettier --write "**/*.md"

# Check formatting without writing (CI)
fmt-check:
    cargo fmt --check
    prettier --check "**/*.md"

# Run clippy lints (CI)
lint:
    cargo clippy

# Run fmt + lint + test (CI pipeline)
ci: fmt-check lint test

# Clean build artifacts
clean:
    cargo clean
