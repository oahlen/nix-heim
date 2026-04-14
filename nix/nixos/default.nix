{
  lib,
  pkgs,
  ...
}@scope:
let
  inherit (lib)
    mkOption
    types
    ;

  heimModule = types.submoduleWith {
    class = "heim";
    modules = lib.singleton (
      { config, ... }:
      {
        imports = [ ../modules/user.nix ] ++ scope.config.heim.sharedModules;
        config._module.args = {
          inherit lib pkgs;
        };
      }
    );
  };

  userSubmodule =
    { config, ... }:
    {
      options = {
        heim = mkOption {
          description = "Nix-heim configuration";
          type = types.nullOr heimModule;
          default = null;
        };
      };

      config = {
        packages =
          let
            manifest = pkgs.callPackage ../heim/manifest.nix { inherit (config.heim) files; };
            linker = pkgs.callPackage ../../heim/package.nix { };
          in
          lib.mkIf (config.heim != null) (
            [
              linker
              (pkgs.writeShellScriptBin "heim-activate" ''
                ${lib.getExe linker} activate ${manifest}
              '')
            ]
            ++ config.heim.home.packages
          );
      };
    };
in
{
  options = {
    users.users = mkOption {
      type = types.attrsOf (types.submodule userSubmodule);
    };

    heim.sharedModules = mkOption {
      description = "Common Heim modules to import.";
      default = [ ];
      example = lib.literalExpression "[ ./module.nix ]";
      type = types.listOf types.raw;
    };
  };
}
