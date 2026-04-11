{
  pkgs,
  lib,
}:
let
  inherit (lib)
    concatLists
    concatMap
    filterAttrs
    mapAttrsToList
    sort
    ;

  version = 1;

  joinPaths =
    base: suffix:
    if base == "" then
      suffix
    else if suffix == "" then
      base
    else
      "${base}/${suffix}";

  listFilesRecursive =
    prefix: dir:
    let
      entries = builtins.readDir dir;
      names = sort builtins.lessThan (builtins.attrNames entries);
    in
    concatMap (
      name:
      let
        fileType = entries.${name};
        relativePath = joinPaths prefix name;
        childPath = joinPaths dir name;
      in
      if fileType == "directory" then
        listFilesRecursive relativePath childPath
      else if fileType == "regular" || fileType == "symlink" then
        [
          {
            relative = relativePath;
            source = childPath;
          }
        ]
      else
        [ ]
    ) names;

  expandFile =
    name: file:
    let
      targetRoot = joinPaths file.relativeTo file.target;

      sourcePath =
        if file.source == null && file.text == null then
          throw "files.${name} must define either source or text."
        else if file.source != null && file.text != null then
          throw "files.${name} must not define both source and text."
        else if file.source == null then
          pkgs.writeText (lib.strings.sanitizeDerivationName name) file.text
        else
          file.source;

      checkedSourcePath =
        if lib.isDerivation sourcePath || builtins.pathExists sourcePath then
          sourcePath
        else
          throw "files.${name}.source does not exist: ${toString sourcePath}";

      mkEntry = target: source: {
        source = toString source;
        inherit target;
        inherit (file) executable overwrite;
      };

      isDir =
        !lib.isDerivation checkedSourcePath && builtins.readFileType checkedSourcePath == "directory";
    in
    if isDir && file.recursive then
      map (entry: mkEntry (joinPaths targetRoot entry.relative) entry.source) (
        listFilesRecursive "" checkedSourcePath
      )
    else
      [ (mkEntry targetRoot checkedSourcePath) ];
in
{
  generateManifest =
    files:
    let
      resultingFiles = concatMap (
        fileSet: mapAttrsToList expandFile (filterAttrs (_: file: file.enable) fileSet)
      ) files;

      payload = {
        files = concatLists resultingFiles;
        inherit version;
      };
    in
    builtins.toJSON payload;
}
