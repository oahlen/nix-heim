{ lib, rustPlatform }:
let
  fs = lib.fileset;
  sourceFiles = fs.unions [
    ./Cargo.toml
    ./Cargo.lock
    (fs.fileFilter (file: file.hasExt "rs") ./src)
  ];

in
rustPlatform.buildRustPackage {
  pname = "nix-heim";
  version = "1.0.0";

  src = fs.toSource {
    root = ./.;
    fileset = sourceFiles;
  };

  cargoLock.lockFile = ./Cargo.lock;

  meta.mainProgram = "heim";
}
