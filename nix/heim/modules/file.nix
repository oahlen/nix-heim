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
      pkgs,
      ...
    }:
    let
      inherit (lib)
        hasInfix
        hasPrefix
        isDerivation
        mkEnableOption
        mkOption
        throwIf
        throwIfNot
        types
        ;

      isDirectory = source: !isDerivation source && builtins.readFileType source == "directory";

      resolveSource =
        label: config:
        if config.source == null && config.text == null then
          throw "files.\"${name}\"${label} must define either source or text."
        else if config.source != null && config.text != null then
          throw "files.\"${name}\"${label} must define only one of source and text."
        else if config.source == null then
          pkgs.writeText (lib.strings.sanitizeDerivationName name) config.text
        else if !builtins.pathExists config.source then
          throw "files.\"${name}\"${label}.source does not exist: ${toString config.source}"
        else
          config.source;

      resolveRootSource =
        config:
        if config.source == null && config.text == null && config.variants == { } then
          throw "files.\"${name}\" must define either source, text or variants."
        else if (config.source != null || config.text != null) && config.variants != { } then
          throw "files.\"${name}\" must define only one of source, text and variants."
        else if config.variants == { } then
          resolveSource "" config
        else
          null;

      resolveVariantSource =
        label: config:
        let
          source = resolveSource label config;
        in
        if isDirectory source then
          throw "files.\"${name}\".variants.${label}.source must resolve to a file: ${toString source}"
        else
          source;

      sourceOptions = {
        source = mkOption {
          type = types.nullOr (
            types.oneOf [
              types.path
              types.package
            ]
          );
          default = null;
          description = ''
            Source path for the file or directory to install.
            Mutually exclusive with 'text' and 'variants'.
            NOTE that directories are not allowed for file variants.
          '';
        };

        text = mkOption {
          type = types.nullOr types.lines;
          default = null;
          description = ''
            Contents of installed file.
            Mutually exclusive with 'source' and 'variants'.
          '';
        };

        resolvedSource = mkOption {
          type = types.nullOr (
            types.oneOf [
              types.path
              types.package
            ]
          );
          internal = true;
          visible = false;
          description = "Resolved derivation for source or text.";
        };
      };

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
            throwIf (hasPrefix "/" x || hasPrefix "~" x || hasInfix "../" x) ''
              The target path '${x}' cannot be used for files.<path>.target.
              Absolute paths, tilde expansion or relative path traversal is not allowed.
            '' x;
        };

        inherit (sourceOptions)
          source
          text
          resolvedSource
          ;

        variants = mkOption {
          type = types.attrsOf (
            types.submodule [
              { _module.args = { inherit pkgs; }; }
              (
                { name, config, ... }:
                {
                  options = sourceOptions // {
                    default = mkOption {
                      type = types.bool;
                      default = false;
                      description = ''
                        Whether this file variant is the default.
                        Only a single file variant can be set as default.
                      '';
                    };

                  };
                  config = {
                    resolvedSource = resolveVariantSource name config;
                  };
                }
              )
            ]
          );
          default = { };
          description = ''
            Extra file variants that can be installed instead of the default.
            Applied with the `--variant` option when activating the configuration.'';
        };

        overwrite = mkOption {
          type = types.bool;
          default = overwrite;
          description = ''
            Whether to overwrite existing file in the target install path.
            Takes precedence over the globally configured overwrite option.
          '';
        };

        relativeTo = mkOption {
          internal = true;
          visible = false;
          type = types.path;
          default = rootDir;
          description = "Path that installed symlinks are relative to.";
          apply =
            x: throwIfNot (hasPrefix "/" x) "Relative path '${x}' cannot be used for files.<path>.relativeTo" x;
        };

        isDirectory = mkOption {
          internal = true;
          visible = false;
          type = types.bool;
          description = "True if the resolved source of this entry is a directory.";
        };
      };

      config = {
        resolvedSource = resolveRootSource config;
        isDirectory = if config.resolvedSource != null then isDirectory config.resolvedSource else false;
      };
    };
}
