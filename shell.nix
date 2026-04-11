let
  sources = import ./npins;
  pkgs = import sources.nixpkgs { };
in
pkgs.mkShell {
  NIX_SHELL = "nix-heim";

  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

  packages = with pkgs; [
    bash
    cargo
    clippy
    jq
    just
    pkg-config
    rust-analyzer
    rustc
    rustfmt
  ];
}
