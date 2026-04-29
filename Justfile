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

# Start the local PostgreSQL verification database
postgres-up:
    docker compose up -d postgres

# Stop the local PostgreSQL verification database
postgres-down:
    docker compose down

# Stop the local PostgreSQL verification database and remove its volume
postgres-clean:
    docker compose down -v

# Run the PostgreSQL real-backend verification test
test-postgres:
    XRAT_POSTGRES_TEST_URL=postgres://xrat:xrat@localhost:54329/xrat cargo test -q verifies_postgres_backend_when_url_is_set -- --nocapture

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
