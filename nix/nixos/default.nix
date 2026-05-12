{
  lib,
  pkgs,
  ...
}@global:
let
  inherit (lib)
    literalExpression
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
          inherit pkgs;
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
        packages =
          let
            environment = callPackage ../heim/environment.nix {
              inherit (config.heim)
                files
                user
                ;
            };
          in
          mkIf (config.heim != null) (
            with environment;
            [
              linker
              activate
              deactivate
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
      description = "Common Nix-heim modules to import.";
      default = [ ];
      example = literalExpression "[ ./module.nix ]";
      type = types.listOf types.raw;
    };
  };
}
