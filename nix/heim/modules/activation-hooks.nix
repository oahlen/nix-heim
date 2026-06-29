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
    activationHooks = mkOption {
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

    activationScript = mkOption {
      type = types.nullOr types.package;
      readOnly = true;
      description = "The concatenated activation hooks script package, or null if no hooks are defined.";
    };
  };

  config = {
    activationScript = mkIf (config.activationHooks != [ ]) (
      pkgs.writeShellScript "heim-activation-script" (concatStringsSep "\n" config.activationHooks)
    );
  };
}
