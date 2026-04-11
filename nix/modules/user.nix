{
  config,
  lib,
  ...
}:
let
  inherit (lib)
    mkOption
    types
    ;

  assertAbsolutePath =
    x: option: lib.throwIf (!lib.hasPrefix "/" x) "Relative path '${x}' cannot be used for ${option}" x;
in
{
  options = {
    home = {
      directory = mkOption {
        type = types.str;
        description = "Home directory of the user.";
        apply = x: assertAbsolutePath x "<home.directory>";
      };

      packages = lib.mkOption {
        type = lib.types.listOf lib.types.package;
        default = [ ];
        description = "Packages to include in the resulting profile environment.";
      };

      pathsToLink = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = [ "/bin" ];
        description = "Paths to link in the resulting profile environment.";
      };

      extraOutputsToInstall = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = [ ];
        description = "Extra outputs to install for packages in the resulting profile environment.";
      };

      files = mkOption {
        type = types.attrsOf (types.submodule (import ./file.nix { rootDir = config.home.directory; }));
        default = { };
        example = {
          "config.toml".source = ./config.toml;
          "example/generated.txt".text = "hello";
        };
        description = "Files to install in the configured home directory.";
      };
    };

    xdg = {
      config = {
        directory = mkOption {
          type = types.str;
          default = "${config.home.directory}/.config";
          defaultText = "$HOME/.config";
          description = "The XDG config directory for the user.";
          apply = x: assertAbsolutePath x "<xdg.config.directory>";
        };

        files = mkOption {
          type = types.attrsOf (
            types.submodule (import ./file.nix { rootDir = config.xdg.config.directory; })
          );
          default = { };
          example = {
            "config.toml".source = ./config.toml;
            "example/generated.txt".text = "hello";
          };
          description = "Files to install under the configured XDG config directory.";
        };
      };

      data = {
        directory = mkOption {
          type = types.str;
          default = "${config.home.directory}/.local/share";
          defaultText = "$HOME/.local/share";
          description = "The XDG data directory for the user.";
          apply = x: assertAbsolutePath x "<xdg.data.directory>";
        };

        files = mkOption {
          type = types.attrsOf (types.submodule (import ./file.nix { rootDir = config.xdg.data.directory; }));
          default = { };
          example = {
            "config.toml".source = ./config.toml;
            "example/generated.txt".text = "hello";
          };
          description = "Files to install under the configured XDG data directory.";
        };
      };

      state = {
        directory = mkOption {
          type = types.str;
          default = "${config.home.directory}/.local/state";
          defaultText = "$HOME/.local/state";
          description = "The XDG state directory for the user.";
          apply = x: assertAbsolutePath x "<xdg.state.directory>";
        };

        files = mkOption {
          type = types.attrsOf (
            types.submodule (import ./file.nix { rootDir = config.xdg.state.directory; })
          );
          default = { };
          example = {
            "config.toml".source = ./config.toml;
            "example/generated.txt".text = "hello";
          };
          description = "Files to install under the configured XDG state directory.";
        };
      };
    };
  };
}
