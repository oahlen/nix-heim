set positional-arguments

# Show available commands
help:
    @just --list --unsorted

# Runs all formatters and checks
@checks: nix-checks rust-checks

# Run all nix formatters and checks
@nix-checks: nix-fmt nix-lint

# Builds and verifies the test manifest
@manifest:
    nix build -f tests manifest && jq < ./result

# Format nix code
@nix-fmt:
    treefmt

# Lint nix code
@nix-lint:
    statix check

# Run all rust formatters and checks
@rust-checks: rust-test rust-fmt rust-lint

# Run rust build
rust-build:
    cargo build --manifest-path heim/Cargo.toml

# Run rust tests
rust-test:
    cargo test --manifest-path heim/Cargo.toml

# Format rust code
@rust-fmt:
    cargo fmt --manifest-path heim/Cargo.toml

# Lint rust code
@rust-lint:
    cargo clippy --manifest-path heim/Cargo.toml
