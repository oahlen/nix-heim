{
  config,
  lib,
  pkgs,
  ...
}:
let
  inherit (lib)
    concatStringsSep
    mkIf
    mkOption
    types
    ;
in
{
  options = {
    hooks = mkOption {
      type = types.listOf types.str;
      default = [ ];
      description = ''
        Shell script snippets that will be concatenated into a final activation script for this user.

        Snippets should ideally be independent and idempotent since there is no specific guarantee on ordering or whether the activation script is only run once.
      '';
      example = [
        ''echo "Running activation hook"''
        "bat cache --build"
      ];
    };

    hooksScript = mkOption {
      type = types.nullOr types.package;
      readOnly = true;
      description = "The concatenated hooks script package, or null if no hooks are defined.";
    };
  };

  config = {
    hooksScript = mkIf (config.home.hooks != [ ]) (
      pkgs.writeShellScript "heim-hooks" (concatStringsSep "\n" config.home.hooks)
    );
  };
}
