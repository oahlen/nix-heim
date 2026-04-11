set positional-arguments

# Show available commands
help:
    @just --list --unsorted

# Runs all formatters and checks
@checks: nix-checks rust-checks

# Run all nix formatters and checks
@nix-checks: nix-test nix-fmt nix-lint

# Run nix tests
nix-test:
    nix build -f tests && ./result/bin/activate | jq

# Format nix code
@nix-fmt:
    treefmt

# Lint nix code
@nix-lint:
    statix check

# Run all rust formatters and checks
@rust-checks: rust-test rust-fmt rust-lint

# Run rust tests
rust-test:
    cargo test --manifest-path heim/Cargo.toml

# Format rust code
@rust-fmt:
    cargo fmt --manifest-path heim/Cargo.toml

# Lint rust code
@rust-lint:
    cargo clippy --manifest-path heim/Cargo.toml
