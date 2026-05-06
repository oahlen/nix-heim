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
          inherit pkgs;
        };
      }
    );
  };

  userSubmodule =
    { config, name, ... }:
    let
      cfg = global.config.users.users.${name};
    in
    {
      options = {
        heim = mkOption {
          description = "Nix-heim configuration";
          type = heimModule;
          default = { };
        };
      };

      config = {
        heim.home.directory = mkIf cfg.isNormalUser (mkDefault cfg.home);

        packages =
          let
            environment = callPackage ../heim/environment.nix { inherit (config.heim) files; };
            hasFiles = builtins.any (fileSet: fileSet != { }) config.heim.files;
            hasPackages = config.heim.packages != [ ];
          in
          mkIf (hasFiles || hasPackages) (
            with environment;
            [
              linker
              activate
              deactivate
              j
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
