{
  mkFileModule =
    {
      rootDir,
      overwrite,
    }:
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
        enable = mkEnableOption "this file." // {
          default = true;
          example = false;
        };

        target = mkOption {
          type = types.str;
          default = name;
          description = "Target path for the file or directory to install relative to the base directory.";
          apply =
            x:
            lib.throwIf (lib.hasPrefix "/" x || lib.hasPrefix "~" x || lib.hasInfix "../" x) ''
              The target path '${x}' cannot be used for files.<path>.target.
              Absolute paths, tilde expansion or relative path traversal is not allowed.
            '' x;
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

        relativeTo = mkOption {
          internal = true;
          type = types.path;
          default = rootDir;
          description = "Path that installed symlinks are relative to.";
          apply =
            x:
            lib.throwIf (
              !lib.hasPrefix "/" x
            ) "Relative path '${x}' cannot be used for files.<path>.relativeTo" x;
        };

        overwrite = mkOption {
          type = types.bool;
          default = overwrite;
          description = ''
            Whether to overwrite existing file in the target install path.
            Takes precedence over the globally configured overwrite option.
          '';
        };
      };
    };
}
