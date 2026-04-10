set positional-arguments

# Show available commands
help:
    @just --list --unsorted

# Run tests
test:
    nix build -f tests && ./result/bin/activate | jq

# Format and lint nix code
@fmt: nix lint

# Format nix code
@nix:
    treefmt

# Lint nix code
@lint:
    statix check
