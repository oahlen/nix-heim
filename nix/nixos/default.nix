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
          inherit pkgs;
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
            files = [
              config.heim.home.files
              config.heim.xdg.config.files
              config.heim.xdg.data.files
              config.heim.xdg.state.files
            ];

            inherit (import ../manifest.nix { inherit lib pkgs; })
              generateManifest
              ;

            manifest = pkgs.writeText "manifest.json" (generateManifest files);

            linker = pkgs.callPackage ../../heim { };

          in
          lib.mkIf (config.heim != null) (
            [
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
