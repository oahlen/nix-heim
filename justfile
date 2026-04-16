set positional-arguments

# Show available commands
help:
    @just --list --unsorted

# Runs all formatters and checks
@checks: nix-checks rust-checks

# Runs all tests
@tests: nix-tests rust-test

#  _   _ _____  __
# | \ | |_ _\ \/ /
# |  \| || | \  /
# | |\  || | /  \
# |_| \_|___/_/\_\

# Run all nix tests
@nix-tests: nix-test-manifest nix-test-standalone nix-test-nixos

# Builds and verifies the test manifest
@nix-test-manifest:
    nix build -f tests manifest && jq < ./result

# Builds and verifies the standalone test module
@nix-test-standalone:
    nix build -f tests/default.nix

# Builds and verifies the nixos test module
@nix-test-nixos:
    nixos-rebuild build-vm -f ./tests/nixos.nix

# Run all nix formatters and checks
@nix-checks: nix-fmt nix-lint

# Format nix code
@nix-fmt:
    treefmt

# Lint nix code
@nix-lint:
    statix check

#  ____  _   _ ____ _____
# |  _ \| | | / ___|_   _|
# | |_) | | | \___ \ | |
# |  _ <| |_| |___) || |
# |_| \_\\___/|____/ |_|

# Run rust build
rust-build:
    cargo build --manifest-path heim/Cargo.toml

# Run rust tests
rust-test:
    cargo test --manifest-path heim/Cargo.toml

# Run all rust formatters and checks
@rust-checks: rust-fmt rust-lint

# Format rust code
@rust-fmt:
    cargo fmt --manifest-path heim/Cargo.toml

# Lint rust code
@rust-lint:
    cargo clippy --manifest-path heim/Cargo.toml
