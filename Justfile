# xrat Justfile

_default:
    @just --list

# Build the project
build:
    cargo build

# Run xrat from this checkout
run +args:
    cargo run --locked -- {{args}}

# Check the project with the locked dependency graph
check:
    cargo check --locked

# Build in release mode
release:
    cargo build --release --locked

# Install xrat with cargo from this checkout
install:
    cargo install --path . --locked

# Install xrat with cargo from this checkout, replacing an existing binary
reinstall:
    cargo install --path . --locked --force

# Remove the cargo-installed xrat binary
uninstall:
    cargo uninstall xrat

# Print shell completions from the local source tree
completions shell:
    cargo run --locked -- completions {{shell}}

# Install man pages generated from the local source tree
install-manpages:
    mkdir -p "$HOME/.local/share/man/man1"
    cargo run --locked -- manpage --output "$HOME/.local/share/man/man1"
    command -v mandb >/dev/null && mandb "$HOME/.local/share/man" || true

# Run tests quietly
test:
    cargo test -q --locked

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

# Format Rust code
fmt-rust:
    cargo fmt

# Check Rust formatting without writing
fmt-rust-check:
    cargo fmt --check

# Format markdown
fmt-md:
    prettier --write "**/*.md"

# Format SQL migrations
fmt-sql:
    sqlfluff format --dialect sqlite migrations/sqlite/*.sql
    sqlfluff format --dialect postgres migrations/postgres/*.sql

# Format Rust code, markdown, and SQL
fmt: fmt-rust fmt-md fmt-sql

# Check Rust, markdown, and SQL formatting without writing
fmt-check:
    cargo fmt --check
    prettier --check "**/*.md"
    sqlfluff lint --rules layout --dialect sqlite migrations/sqlite/*.sql
    sqlfluff lint --rules layout --dialect postgres migrations/postgres/*.sql

# Run clippy lints (CI)
lint:
    cargo clippy --all-targets -- -D warnings

# Run the same commands as .github/workflows/ci.yml
ci: fmt-rust-check lint test

# Serve docs as an mdBook
docs:
    mdbook serve docs --open

# Clean build artifacts
clean:
    cargo clean
