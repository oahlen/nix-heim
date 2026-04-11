{ rustPlatform }:
rustPlatform.buildRustPackage {
  pname = "nix-heim";
  version = "1.0.0";

  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;

  meta.mainProgram = "heim";
}
