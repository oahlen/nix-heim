{
  config,
  lib,
  pkgs,
  ...
}:
let
  inherit (lib)
    hasPrefix
    mkOption
    throwIfNot
    types
    ;

  inherit (import ./file.nix) mkFileModule;

  mkFileModuleWithRoot = rootDir: [
    { _module.args = { inherit pkgs; }; }
    (mkFileModule {
      inherit rootDir;
      inherit (config) overwrite;
    })
  ];

  assertAbsolutePath =
    x: option: throwIfNot (hasPrefix "/" x) "Relative path '${x}' cannot be used for ${option}" x;
in
{
  options = {
    home = {
      directory = mkOption {
        type = types.str;
        description = ''
          Home directory of the user.
          When using nix-heim as a nixos module it will by default be set to `users.users.<name>.home`.
        '';
        apply = x: assertAbsolutePath x "<home.directory>";
      };

      files = mkOption {
        type = types.attrsOf (types.submodule (mkFileModuleWithRoot config.home.directory));
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
          type = types.attrsOf (types.submodule (mkFileModuleWithRoot config.xdg.config.directory));
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
          type = types.attrsOf (types.submodule (mkFileModuleWithRoot config.xdg.data.directory));
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
          type = types.attrsOf (types.submodule (mkFileModuleWithRoot config.xdg.state.directory));
          default = { };
          example = {
            "config.toml".source = ./config.toml;
            "example/generated.txt".text = "hello";
          };
          description = "Files to install under the configured XDG state directory.";
        };
      };
    };

    packages = mkOption {
      type = types.listOf types.package;
      default = [ ];
      description = "Packages to include in the resulting profile environment.";
    };

    pathsToLink = mkOption {
      type = types.listOf types.str;
      default = [ "/" ];
      description = ''
        Paths to link in the resulting user profile environment.
        This option has no effect when using nix-heim as a NixOS module.
        See `environment.pathsToLink` in NixOS to configure paths to link in user environments.
      '';
    };

    extraOutputsToInstall = mkOption {
      type = types.listOf types.str;
      default = [ ];
      description = ''
        Extra outputs to install for packages in the resulting user profile environment.
        This option has no effect when using nix-heim as a NixOS module.
        See `environment.extraOutputsToInstall` in NixOS to configure extra outputs in user environments.
      '';
    };

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

        1. Via the Nix store path exposed as `config.sessionVariablesPackage`, suitable for use in managed dotfiles:
           `. ''${config.sessionVariablesPackage}`

        2. Via `XDG_DATA_DIRS` discovery (requires `/share` in `pathsToLink` for standalone installs or `environment.pathsToLink` for nixos respectively):
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
        "\${xdg.config.directory}/scrips"
      ];
    };

    overwrite = mkOption {
      type = types.bool;
      default = false;
      description = ''
        Whether to overwrite existing files in the target install path.
        Can be overridden by individual file options.
      '';
    };

    # Internal options
    files = mkOption {
      type = types.listOf types.attrs;
      readOnly = true;
      visible = false;
    };

    sessionVariablesPackage = mkOption {
      type = types.package;
      readOnly = true;
      description = "Package containing the configured session variables.";
    };
  };

  config = {
    files = [
      config.home.files
      config.xdg.config.files
      config.xdg.data.files
      config.xdg.state.files
    ];

    sessionVariablesPackage = pkgs.callPackage ../session-vars.nix {
      inherit (config)
        sessionVariables
        sessionPath
        ;
    };

    packages = [ config.sessionVariablesPackage ];
  };
}
