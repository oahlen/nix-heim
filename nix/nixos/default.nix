{
  lib,
  pkgs,
  ...
}@global:
let
  inherit (lib)
    literalExpression
    mkDefault
    mkIf
    mkOption
    singleton
    types
    ;

  inherit (pkgs) callPackage;

  heimModule = types.submoduleWith {
    class = "heim";
    modules = singleton (
      { config, ... }:
      {
        imports = [ ../heim/modules/user.nix ] ++ global.config.heim.sharedModules;
        config._module.args = {
          inherit pkgs lib;
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
            environment = callPackage ../heim/environment.nix { inherit (config.heim) files; };
          in
          mkIf (config.heim != null) (
            with environment;
            [
              linker
              activate
              deactivate
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
