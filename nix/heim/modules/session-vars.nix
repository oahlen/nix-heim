{
  config,
  lib,
  pkgs,
  ...
}:
let
  inherit (lib)
    concatStringsSep
    mapAttrsToList
    mkIf
    mkOption
    types
    ;

  export = name: value: ''export ${name}="${toString value}"'';

  prependToPath = v: "${concatStringsSep ":" v}\${PATH:+:}\$PATH";

  destination = "/share/heim/session-vars.sh";

  package = pkgs.writeTextFile {
    name = "heim-session-vars";

    inherit destination;

    text = ''
      ${concatStringsSep "\n" (mapAttrsToList export config.home.sessionVariables)};

      ${export "PATH" (prependToPath config.home.sessionPath)}
    '';
  };
in
{
  options = {
    home = {
      sessionVariables = mkOption {
        type = types.lazyAttrsOf (
          types.oneOf [
            types.str
            types.path
            types.int
            types.float
          ]
        );
        default = { };
        description = ''
          Session variables for this user exposed as a POSIX compliant shell script.

          Two ways of sourcing the script are available:

          1. Via the Nix store path exposed as `config.home.loadSessionVariables`, suitable for use in managed dotfiles:
             `. ''${config.home.loadSessionVariables}`

          2. Via `XDG_DATA_DIRS` discovery (requires `/share` in `home.pathsToLink` for standalone installs or `environment.pathsToLink` for nixos respectively):
             The script is installed to `<profile>/share/heim/session-vars.sh`.
        '';
        example = {
          EDITOR = "vim";
          PAGER = "less";
        };
      };

      sessionPath = mkOption {
        type = types.listOf types.str;
        default = [ ];
        description = "Directories to prepend the $PATH variable in the generated session variables script.";
        example = [
          "$HOME/.local/bin"
          "\${xdg.config.directory}/scripts"
        ];
      };

      loadSessionVariables = mkOption {
        type = types.path;
        readOnly = true;
        description = "Path to the POSIX compliant shell script containing the configured session variables.";
      };
    };
  };

  config = {
    home = {
      packages = mkIf (config.home.sessionVariables != { } || config.home.sessionPath != [ ]) [ package ];
      loadSessionVariables = "${package}${destination}";
    };
  };
}
