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

# Format Rust code, markdown, and SQL
fmt:
    cargo fmt
    prettier --write "**/*.md"
    sqlfluff format --dialect sqlite migrations/sqlite/*.sql
    sqlfluff format --dialect postgres migrations/postgres/*.sql

# Check formatting without writing (CI)
fmt-check:
    cargo fmt --check
    prettier --check "**/*.md"
    sqlfluff lint --rules layout --dialect sqlite migrations/sqlite/*.sql
    sqlfluff lint --rules layout --dialect postgres migrations/postgres/*.sql

# Run clippy lints (CI)
lint:
    cargo clippy

# Run fmt + lint + test (CI pipeline)
ci: fmt-check lint test

# Clean build artifacts
clean:
    cargo clean
