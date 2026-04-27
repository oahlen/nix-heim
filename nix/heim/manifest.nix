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

  mkDefaultSources = source: [
    {
      name = "default";
      source = toString source;
      default = true;
    }
  ];

  mkVariantSources =
    name: file:
    let
      variants = mapAttrsToList (name: file: {
        inherit name;
        inherit (file) default;
        source = toString file.resolvedSource;
      }) file.variants;

      defaults = builtins.filter (v: v.default) variants;
    in
    if builtins.length defaults > 1 then
      throw "Multiple default variants found for files.${name}."
    else if builtins.length defaults == 0 then
      throw "No default variant found for files.${name}."
    else
      variants;

  expandFile =
    name: file:
    let
      targetRoot = joinTarget file.relativeTo file.target;

      mkEntry = target: sources: {
        inherit target sources;
        inherit (file) overwrite;
      };

      entries =
        if file.resolvedSource == null then
          [ (mkEntry targetRoot (mkVariantSources name file)) ]
        else if file.isDirectory then
          map (entry: mkEntry (joinTarget targetRoot entry.relative) (mkDefaultSources entry.source)) (
            listFilesRecursive "" file.resolvedSource
          )
        else
          [ (mkEntry targetRoot (mkDefaultSources file.resolvedSource)) ];

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
