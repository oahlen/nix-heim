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

    inherit specialArgs;

    modules = [
      {
        _module.args = {
          inherit pkgs;
        };
      }
      ./modules/user.nix
    ]
    ++ modules;
  };

  cfg = evaluated.config;

  environment = callPackage ./environment.nix {
    inherit (cfg)
      files
      packages
      pathsToLink
      extraOutputsToInstall
      ;
  };

in
environment
