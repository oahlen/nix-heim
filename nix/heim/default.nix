{
  callPackage,
  lib,
  modules ? [ ],
  pkgs,
  specialArgs ? { },
}:
let
  evaluated = lib.evalModules {
    class = "heim";

    specialArgs = {
      inherit pkgs lib;
    }
    // specialArgs;
    modules = [
      ./modules/user.nix
    ]
    ++ modules;
  };

  cfg = evaluated.config;

  environment = callPackage ./environment.nix {
    inherit (cfg)
      files
      extraOutputsToInstall
      pathsToLink
      packages
      ;
  };

in
environment
