# xrat Justfile

set shell := ["bash", "-euo", "pipefail", "-c"]

md_files := "README.md docs/**/*.md"
sqlite_migrations := "migrations/sqlite/*.sql"
postgres_migrations := "migrations/postgres/*.sql"
tape_dir := "docs/src/media/tapes"
gif_dir := "docs/src/media/gif"

# Override with: XRAT_POSTGRES_TEST_URL=postgres://... just test-postgres
postgres_test_url := env("XRAT_POSTGRES_TEST_URL", "postgres://xrat:xrat@localhost:54329/xrat")

_default:
    @just --list

# Build the project
build:
    cargo build --locked

# Run xrat from this checkout
run *args:
    cargo run --locked -- {{args}}

# Check the project with the locked dependency graph
check:
    cargo check --locked

# Build in release mode
release:
    cargo build --release --locked

# Build from this checkout and install via install.sh; pass installer flags after the recipe
install *installer_args:
    bash install.sh --from-source {{installer_args}}

# Run tests quietly
test:
    cargo test -q --locked

# Generate a terminal coverage summary
coverage:
    cargo llvm-cov --locked

# Generate an HTML coverage report
coverage-html:
    cargo llvm-cov --locked --html

# Generate an HTML coverage report and open it
coverage-html-open:
    cargo llvm-cov --locked --html --open

# Generate lcov output for CI/services
coverage-lcov:
    cargo llvm-cov --locked --lcov --output-path lcov.info

# Start the local PostgreSQL verification database
postgres-up:
    docker compose up -d postgres

# Build the local Docker image
docker-build tag="xrat:latest":
    docker build -t {{tag}} .

# Stop the local PostgreSQL verification database
postgres-down:
    docker compose down

# Stop the local PostgreSQL verification database and remove its volume
postgres-clean:
    docker compose down -v

# Run the PostgreSQL real-backend verification test
test-postgres:
    XRAT_POSTGRES_TEST_URL={{quote(postgres_test_url)}} cargo test -q --locked verifies_postgres_backend_when_url_is_set -- --nocapture

# Format Rust code
fmt-rust:
    cargo fmt

# Check Rust formatting without writing
fmt-rust-check:
    cargo fmt --check

# Format markdown
fmt-md:
    prettier --write {{md_files}}

# Format SQL migrations
fmt-sql:
    sqlfluff format --dialect sqlite {{sqlite_migrations}}
    sqlfluff format --dialect postgres {{postgres_migrations}}

# Format Rust code, markdown, and SQL
fmt: fmt-rust fmt-md fmt-sql

# Check Rust, markdown, and SQL formatting without writing
fmt-check:
    cargo fmt --check
    prettier --check {{md_files}}
    sqlfluff lint --rules layout --dialect sqlite {{sqlite_migrations}}
    sqlfluff lint --rules layout --dialect postgres {{postgres_migrations}}

# Run clippy lints (CI)
lint:
    cargo clippy --locked --all-targets -- -D warnings

# Run the same commands as .github/workflows/ci.yml
ci: fmt-rust-check lint test

# Run stricter local checks beyond GitHub CI
ci-full: fmt-check lint test mdbook-build

# Serve docs as an mdBook
mdbook:
    mdbook serve docs

# Build docs as an mdBook
mdbook-build:
    mdbook build docs

# Clean mdBook build output
mdbook-clean:
    rm -rf docs/book

# Run the TUI
tui:
    cargo run --locked -- tui

# Clean build artifacts
clean: mdbook-clean
    cargo clean

# Check required local development tools
tools-check:
    @missing=0; \
    for tool in prettier sqlfluff mdbook docker vhs fd; do \
        if ! command -v "$tool" >/dev/null; then \
            echo "missing: $tool"; \
            missing=1; \
        fi; \
    done; \
    if ! cargo llvm-cov --version >/dev/null 2>&1; then \
        echo "missing: cargo-llvm-cov"; \
        missing=1; \
    fi; \
    exit "$missing"

# Render one explicit tape file.
tape tape:
    mkdir -p {{gif_dir}}
    vhs --output "{{gif_dir}}/$(basename {{quote(tape)}} .tape).gif" {{ quote(tape) }}

# Render every .tape in docs/src/media/tapes except base.tape.
tapes:
    fd -e tape -E base.tape . {{tape_dir}} -x just tape {}
