let
  sources = import ./npins;
  pkgs = import sources.nixpkgs {
    config = { };
    overlays = [ ];
  };
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
    nixfmt-tree
    pkg-config
    rust-analyzer
    rustc
    rustfmt
    statix
  ];
}
