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
[group('development')]
build:
    cargo build --locked

# Run xrat from this checkout
[group('development')]
run *args:
    cargo run --locked -- {{ args }}

# Check the project with the locked dependency graph
[group('development')]
check:
    cargo check --locked

# Build from this checkout and install via install.sh; pass installer flags after the recipe
[group('development')]
install *installer_args:
    bash install.sh --from-source {{ installer_args }}

# Build the local Docker image
[group('development')]
docker-build tag="xrat:latest":
    docker build -t {{ tag }} .

# Run the TUI
[group('development')]
tui:
    cargo run --locked -- tui

# Clean build artifacts
[group('development')]
clean: mdbook-clean
    cargo clean

# Check required local development tools
[group('development')]
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

# Update all workspace versions and refresh the lockfile
[group('release')]
set-version version:
    #!/usr/bin/env bash
    set -euo pipefail
    version={{ quote(version) }}
    [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$ ]] || { echo 'invalid version' >&2; exit 1; }
    XRAT_RELEASE_VERSION="$version" perl -0pi -e 's/(\[workspace.package\]\s*\nversion\s*=\s*")[^"]+/$1$ENV{XRAT_RELEASE_VERSION}/; s/(^\s*[\w-]+\s*=\s*\{[^\n]*path\s*=[^\n]*version\s*=\s*")[^"]+/$1$ENV{XRAT_RELEASE_VERSION}/mg' Cargo.toml
    cargo update --workspace --offline

# Check workspace version consistency and release tooling
[group('release')]
version-check:
    cargo test --locked -p xrat --test release_tooling

# Build in release mode
[group('release')]
release:
    cargo build --release --locked

# Run tests quietly
[group('checks')]
test:
    cargo test -q --locked --workspace

# Format Rust code
[group('checks')]
fmt-rust:
    cargo fmt --all

# Check Rust formatting without writing
[group('checks')]
fmt-rust-check:
    cargo fmt --all --check

# Format markdown
[group('checks')]
fmt-md:
    prettier --write {{ md_files }}

# Format SQL migrations
[group('checks')]
fmt-sql:
    sqlfluff format --dialect sqlite {{ sqlite_migrations }}
    sqlfluff format --dialect postgres {{ postgres_migrations }}

# Format Rust code, markdown, and SQL
[group('checks')]
fmt: fmt-rust fmt-md fmt-sql

# Check Rust, markdown, and SQL formatting without writing
[group('checks')]
fmt-check:
    cargo fmt --all --check
    prettier --check {{ md_files }}
    sqlfluff lint --rules layout --dialect sqlite {{ sqlite_migrations }}
    sqlfluff lint --rules layout --dialect postgres {{ postgres_migrations }}

# Run clippy lints (CI)
[group('checks')]
lint:
    cargo clippy --locked --workspace --all-targets -- -D warnings

# Run the same commands as .github/workflows/ci.yml
[group('checks')]
ci: version-check fmt-rust-check lint test

# Run stricter local checks beyond GitHub CI
[group('checks')]
ci-full: fmt-check lint test mdbook-build

# Generate a terminal coverage summary
[group('coverage')]
coverage:
    cargo llvm-cov --locked

# Generate an HTML coverage report
[group('coverage')]
coverage-html:
    cargo llvm-cov --locked --html

# Generate an HTML coverage report and open it
[group('coverage')]
coverage-html-open:
    cargo llvm-cov --locked --html --open

# Generate lcov output for CI/services
[group('coverage')]
coverage-lcov:
    cargo llvm-cov --locked --lcov --output-path lcov.info

# Verify public SDK features, docs, dependency boundaries and standalone usage
[group('sdk')]
sdk-check:
    cargo test --locked -p xrat-sdk
    cargo test --locked -p xrat-sdk --all-targets --features services
    cargo clippy --locked -p xrat-sdk --all-targets --all-features -- -D warnings
    RUSTDOCFLAGS="-D warnings" cargo doc --locked -p xrat-sdk --all-features --no-deps
    cargo test --locked -p xrat --test sdk_boundary
    cargo test --locked -p xrat --test sdk_boundary standalone_consumer -- --ignored --exact --nocapture

# Run representative pinned engine validation and local probe lifecycle tests
[group('sdk')]
sdk-native xray_binary singbox_binary:
    XRAT_SDK_XRAY_BINARY={{ quote(xray_binary) }} XRAT_SDK_SINGBOX_BINARY={{ quote(singbox_binary) }} cargo test --locked -p xrat-sdk --test native -- --ignored

# Verify normal crates.io installation after publication
[group('sdk')]
sdk-registry version:
    XRAT_SDK_REGISTRY_VERSION={{ quote(version) }} cargo test --locked -p xrat --test sdk_boundary standalone_consumer -- --ignored --exact --nocapture

# Download checksum-pinned Linux amd64 test engines
[group('runtime')]
runtime-engines directory:
    #!/usr/bin/env bash
    set -euo pipefail
    directory={{ quote(directory) }}
    mkdir -p "$directory"
    receipts="$directory/engine-receipts.json"
    printf '[\n' > "$receipts"
    separator=''
    while read -r name version checksum url member; do
        archive="$directory/$version-${url##*/}"
        if [[ ! -f "$archive" ]]; then
            curl -fL --retry 3 "$url" -o "$archive.download"
            mv "$archive.download" "$archive"
        fi
        if command -v sha256sum >/dev/null; then
            actual=$(sha256sum "$archive")
        else
            actual=$(shasum -a 256 "$archive")
        fi
        [[ "${actual%% *}" == "$checksum" ]] || { echo "$name: archive checksum mismatch" >&2; exit 1; }
        if [[ "$archive" == *.zip ]]; then
            unzip -p "$archive" "$member" > "$directory/$name.extract"
        else
            tar -xOf "$archive" "$member" > "$directory/$name.extract"
        fi
        chmod 755 "$directory/$name.extract"
        mv "$directory/$name.extract" "$directory/$name"
        printf '%s{"engine":"%s","version":"%s","sha256":"%s","url":"%s","member":"%s"}' "$separator" "$name" "$version" "$checksum" "$url" "$member" >> "$receipts"
        separator=$',\n'
    done <<'ENGINES'
    xray 26.7.11 aa11c3685c71da0ffc71e511db50404609e7e963bb914b048f59a6a00af8930e https://github.com/XTLS/Xray-core/releases/download/v26.7.11/Xray-linux-64.zip xray
    xray-old 26.3.27 23cd9af937744d97776ee35ecad4972cf4b2109d1e0fe6be9930467608f7c8ae https://github.com/XTLS/Xray-core/releases/download/v26.3.27/Xray-linux-64.zip xray
    xray-new 26.9.30 f851110beaff16e78d643f0ccfd9524b4a44dfd59bae3e34bb52bba378f7690e https://github.com/XTLS/Xray-core/releases/download/v26.9.30/Xray-linux-64.zip xray
    sing-box 1.13.21 24f9ef8e7234e13e71e74c3598a4164c5fe07b7b67ccc6e96cf68b54789f72cd https://github.com/SagerNet/sing-box/releases/download/v1.13.21/sing-box-1.13.21-linux-amd64.tar.gz sing-box-1.13.21-linux-amd64/sing-box
    ENGINES
    printf '\n]\n' >> "$receipts"

# Validate the generated sing-box matrix and application DNS policies
[group('runtime')]
singbox-conformance binary output:
    XRAT_CONFORMANCE_SINGBOX={{ quote(binary) }} XRAT_CONFORMANCE_OUTPUT={{ quote(output) }} cargo test --locked -p xrat-engines --test singbox_conformance -- --include-ignored --nocapture
    XRAT_CONFORMANCE_SINGBOX={{ quote(binary) }} XRAT_CONFORMANCE_OUTPUT={{ quote(output) }} cargo test --locked -p xrat-app native_singbox_dns_conformance -- --ignored --nocapture

# Validate generated managed DNS fixtures with pinned native cores
[group('runtime')]
runtime-native xray_binary singbox_binary output:
    XRAT_RUNTIME_XRAY={{ quote(xray_binary) }} XRAT_RUNTIME_SINGBOX={{ quote(singbox_binary) }} XRAT_RUNTIME_FIXTURE_DIR={{ quote(output) }} cargo test --locked -p xrat-app dns_runtime_native_fixtures -- --ignored

# Exercise real resolver paths and FakeIP in a disposable namespace
[group('runtime')]
runtime-dns xray_binary singbox_binary fixtures:
    XRAT_RUNTIME_XRAY={{ quote(xray_binary) }} XRAT_RUNTIME_SINGBOX={{ quote(singbox_binary) }} XRAT_RUNTIME_FIXTURE_DIR={{ quote(fixtures) }} cargo test --locked -p xrat --test runtime_network dns_network -- --ignored --exact --nocapture

# Exercise managed CLI/daemon capture and cleanup in disposable namespaces
[group('runtime')]
runtime-tun xrat_binary xray_binary singbox_binary output:
    XRAT_RUNTIME_CLI={{ quote(xrat_binary) }} XRAT_RUNTIME_XRAY={{ quote(xray_binary) }} XRAT_RUNTIME_SINGBOX={{ quote(singbox_binary) }} XRAT_RUNTIME_OUTPUT={{ quote(output) }} cargo test --locked -p xrat --test runtime_network tun_network -- --ignored --exact --nocapture

# Exercise both managed Linux gateway engines in an isolated shared-LAN topology
runtime-gateway xrat_binary xray_binary singbox_binary output:
    XRAT_RUNTIME_CLI={{ quote(xrat_binary) }} XRAT_RUNTIME_XRAY={{ quote(xray_binary) }} XRAT_RUNTIME_SINGBOX={{ quote(singbox_binary) }} XRAT_RUNTIME_OUTPUT={{ quote(output) }} cargo test --locked -p xrat --test runtime_network gateway_managed_engines -- --ignored --exact --nocapture

# Start the local PostgreSQL verification database
[group('database')]
postgres-up:
    docker compose up -d postgres

# Stop the local PostgreSQL verification database
[group('database')]
postgres-down:
    docker compose down

# Stop the local PostgreSQL verification database and remove its volume
[group('database')]
postgres-clean:
    docker compose down -v

# Run the PostgreSQL real-backend verification test
[group('database')]
test-postgres:
    XRAT_POSTGRES_TEST_URL={{ quote(postgres_test_url) }} cargo test -q --locked --workspace verifies_postgres_backend_when_url_is_set -- --nocapture

# Serve docs as an mdBook
[group('docs')]
mdbook:
    mdbook serve docs

# Build docs as an mdBook
[group('docs')]
mdbook-build:
    mdbook build docs

# Clean mdBook build output
[group('docs')]
mdbook-clean:
    rm -rf docs/book

# Render one explicit tape file.
[group('docs')]
tape tape:
    mkdir -p {{ gif_dir }}
    vhs --output "{{ gif_dir }}/$(basename {{ quote(tape) }} .tape).gif" {{ quote(tape) }}

# Render every .tape in docs/src/media/tapes except base.tape.
[group('docs')]
tapes:
    fd -e tape -E base.tape . {{ tape_dir }} -x just tape {}

# Fetch checksum-pinned Linux x86_64 frontend build tools
[group('web')]
web-tools:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p target/web-tools
    if [[ "$(uname -s)-$(uname -m)" != "Linux-x86_64" ]]; then
        command -v npx >/dev/null || { echo 'Node.js/npm is required for Tailwind on this build host' >&2; exit 1; }
        cargo install wasm-bindgen-cli --locked --version 0.2.122 --root target/web-tools
        cp target/web-tools/bin/wasm-bindgen target/web-tools/wasm-bindgen
        exit 0
    fi
    cd target/web-tools
    verified() { [[ -f "$1" ]] && echo "$2  $1" | sha256sum --status -c -; }
    fetch() {
        local url="$1" destination="$2" checksum="$3"
        if verified "$destination" "$checksum"; then return; fi
        curl -fL --retry 3 --continue-at - "$url" -o "$destination.part"
        verified "$destination.part" "$checksum" || { rm -f "$destination.part"; echo "Checksum mismatch: $destination" >&2; return 1; }
        mv "$destination.part" "$destination"
    }
    if [[ ! -x wasm-bindgen ]] || [[ "$(./wasm-bindgen --version 2>/dev/null)" != 'wasm-bindgen 0.2.122' ]]; then
        fetch https://github.com/wasm-bindgen/wasm-bindgen/releases/download/0.2.122/wasm-bindgen-0.2.122-x86_64-unknown-linux-musl.tar.gz wasm-bindgen.tar.gz 122a3fe2ea9c6e3e89b50e42dd1a346e499f3f65a54aae0b4eaeb658139d1e0e
        staging="$(mktemp -d .wasm-bindgen.XXXXXX)"
        trap 'rm -rf "$staging"' EXIT
        tar -xzf wasm-bindgen.tar.gz --strip-components=1 -C "$staging"
        [[ "$("$staging/wasm-bindgen" --version)" == 'wasm-bindgen 0.2.122' ]]
        mv "$staging/wasm-bindgen" wasm-bindgen
    fi
    fetch https://github.com/tailwindlabs/tailwindcss/releases/download/v4.1.18/tailwindcss-linux-x64 tailwindcss 737becf8d4ad1115ea98df69fa94026d402ca8feb91306a035b5b004167da8dd
    chmod +x tailwindcss

# Build browser assets before building the XRAT binary
[group('web')]
web-build:
    cargo build --locked --release -p xrat-web --target wasm32-unknown-unknown
    target/web-tools/wasm-bindgen target/wasm32-unknown-unknown/release/xrat-web.wasm --target web --out-name xrat_web --out-dir crates/xrat-app/web-assets
    if test -x target/web-tools/tailwindcss; then target/web-tools/tailwindcss -i crates/xrat-web/assets/app.css -o crates/xrat-app/web-assets/app.css --minify; else npx --yes @tailwindcss/cli@4.1.18 -i crates/xrat-web/assets/app.css -o crates/xrat-app/web-assets/app.css --minify; fi
    cp crates/xrat-web/index.html crates/xrat-web/assets/favicon.ico crates/xrat-web/assets/manifest.webmanifest crates/xrat-web/assets/sw.js crates/xrat-web/assets/offline.html crates/xrat-app/web-assets/
    rm -rf crates/xrat-app/web-assets/controllers crates/xrat-app/web-assets/components.js crates/xrat-app/web-assets/pwa.js crates/xrat-app/web-assets/boot.js crates/xrat-app/web-assets/xrat-logo.png
    cp -R crates/xrat-web/assets/icons crates/xrat-app/web-assets/
    cp -R crates/xrat-web/public/. crates/xrat-app/web-assets/

# Check browser-safe contracts and the Dioxus WebAssembly target
[group('web')]
web-check:
    cargo check --locked -p xrat-web --target wasm32-unknown-unknown
    cargo clippy --locked -p xrat-web -- -D warnings
