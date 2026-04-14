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
        heim =
          let
            cfg = config.heim;
          in
          if cfg != null then
            cfg // { home.directory = cfg.home.directory or config.homeDir or "/home/${name}"; }
          else
            null;

        packages =
          let
            cfg = config.heim;

            manifest = pkgs.callPackage ../heim/manifest.nix { inherit (cfg) files; };

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
      description = "Common Heim modules to import.";
      default = [ ];
      example = lib.literalExpression "[ ./module.nix ]";
      type = types.listOf types.raw;
    };
  };
}
