{ rootDir }:
{
  name,
  config,
  lib,
  ...
}:
let
  inherit (lib)
    mkEnableOption
    mkOption
    types
    ;
in
{
  options = {
    enable = mkEnableOption "Whether to enable this file." // {
      default = true;
      example = false;
    };

    target = mkOption {
      type = types.str;
      default = name;
      description = "Target path for the file or directory to install relative to the base directory.";
    };

    source = mkOption {
      type = types.nullOr (
        types.oneOf [
          types.path
          types.package
        ]
      );
      default = null;
      description = "Source path for the file or directory to install. Mutually exclusive with 'text'.";
    };

    text = mkOption {
      type = types.nullOr types.lines;
      default = null;
      description = "Contents of installed file. Mutually exclusive with 'source'.";
    };

    recursive = mkOption {
      type = types.bool;
      default = false;
      description = "Whether entries should be recursively symlinked or not. Only applicable for directory entries.";
    };

    executable = mkOption {
      type = types.nullOr types.bool;
      default = null;
      description = "Whether the installed file should be executable.";
    };

    relativeTo = mkOption {
      internal = true;
      type = types.path;
      default = rootDir;
      description = "Path that installed symlinks are relative to.";
      apply =
        x:
        assert (
          lib.hasPrefix "/" x || abort "Relative path ${x} cannot be used for files.<path>.relativeTo"
        );
        x;
    };
  };
}
