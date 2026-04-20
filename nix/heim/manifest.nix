{
  files ? [ ],
  lib,
  writeText,
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

  joinTarget =
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
        relativePath = joinTarget prefix name;
        childPath = dir + "/${name}"; # Avoids coercing linked source file into the nix store
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
      targetRoot = joinTarget file.relativeTo file.target;

      variants = concatLists (
        mapAttrsToList (name: file: [
          {
            inherit name;
            source = toString file.resolvedSource;
          }
        ]) file.variants
      );

      mkEntry = target: source: {
        source = toString source;
        inherit target variants;
        inherit (file) overwrite;
      };

      entries =
        if file.isDirectory && builtins.attrNames file.variants != [ ] then
          throw "files.${name} is a directory and cannot have variants."
        else if file.isDirectory then
          map (entry: mkEntry (joinTarget targetRoot entry.relative) entry.source) (
            listFilesRecursive "" file.resolvedSource
          )
        else
          [ (mkEntry targetRoot file.resolvedSource) ];

    in
    entries;

  expandFiles =
    files:
    let
      expandFileSets =
        files:
        concatMap (fileSet: mapAttrsToList expandFile (filterAttrs (_: file: file.enable) fileSet)) files;

      sortFiles = files: builtins.sort (a: b: builtins.lessThan a.target b.target) files;
    in
    sortFiles (concatLists (expandFileSets files));

  validate =
    files:
    let
      grouped = builtins.groupBy (file: file.target) files;
      duplicates = builtins.filter (key: builtins.length grouped.${key} > 1) (builtins.attrNames grouped);
    in
    if duplicates == [ ] then files else throw "Duplicate targets found: ${builtins.toJSON duplicates}";

  generateManifest =
    files:
    let
      payload = {
        files = validate (expandFiles files);
        inherit version;
      };
    in
    builtins.toJSON payload;
in
writeText "manifest.json" (generateManifest files)
