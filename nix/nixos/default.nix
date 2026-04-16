{
  lib,
  pkgs,
  ...
}@global:
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
        imports = [ ../heim/modules/user.nix ] ++ global.config.heim.sharedModules;
        config._module.args = {
          inherit lib pkgs;
        };
      }
    );
  };

  userSubmodule =
    { config, name, ... }:
    {
      options = {
        heim = mkOption {
          description = "Nix-heim configuration";
          type = types.nullOr heimModule;
          default = null;
        };
      };

      config = {
        heim.home.directory = lib.mkDefault global.config.users.users.${name}.home;

        packages =
          let
            manifest = pkgs.callPackage ../heim/manifest.nix { inherit (config.heim) files; };

            linker = pkgs.callPackage ../../heim/package.nix { };

            activationScript = pkgs.writeShellScriptBin "heim-activate" ''
              ${lib.getExe linker} activate ${manifest}
            '';

            deactivationScript = pkgs.writeShellScriptBin "heim-deactivate" ''
              ${lib.getExe linker} deactivate ${manifest}
            '';
          in
          lib.mkIf (config.heim != null) (
            [
              linker
              activationScript
              deactivationScript
            ]
            ++ config.heim.packages
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
      description = "Common Nix-heim modules to import.";
      default = [ ];
      example = lib.literalExpression "[ ./module.nix ]";
      type = types.listOf types.raw;
    };
  };
}
