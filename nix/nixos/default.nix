{
  lib,
  pkgs,
  ...
}@global:
let
  inherit (lib)
    getExe
    literalExpression
    mkDefault
    mkIf
    mkOption
    singleton
    types
    ;

  inherit (pkgs)
    callPackage
    writeShellScriptBin
    ;

  heimModule = types.submoduleWith {
    class = "heim";
    modules = singleton (
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
        heim.home.directory = mkDefault global.config.users.users.${name}.home;

        packages =
          let
            manifest = callPackage ../heim/manifest.nix { inherit (config.heim) files; };

            linker = callPackage ../../heim/package.nix { };

            activationScript = writeShellScriptBin "heim-activate" ''
              ${getExe linker} activate ${manifest}
            '';

            deactivationScript = writeShellScriptBin "heim-deactivate" ''
              ${getExe linker} deactivate ${manifest}
            '';
          in
          mkIf (config.heim != null) (
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
      example = literalExpression "[ ./module.nix ]";
      type = types.listOf types.raw;
    };
  };
}
